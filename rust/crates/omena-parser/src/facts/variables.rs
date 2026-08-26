//! Parser facts for Sass and CSS variable-like declarations and references.
//!
//! The collector distinguishes declaration/reference positions at token level
//! so later layers can resolve scope and module visibility explicitly.

use cstree::text::TextRange;
use omena_syntax::{
    StyleDialect, SyntaxKind,
    ident::{AuthoredPropertyTextV0, CanonicalCustomPropertyNameV0, PropertyNameV0},
};
use std::{collections::BTreeMap, fmt};

#[cfg(test)]
use crate::ParseResult;
use crate::{
    Token, containing_at_rule_header_name, matches_ignore_ascii_case, next_non_trivia_token,
    previous_non_trivia_token, previous_non_trivia_token_index,
};

use super::StyleFactSink;

#[derive(Debug, Clone)]
pub struct ParsedVariableFact {
    pub kind: ParsedVariableFactKind,
    pub name: ParsedVariableFactNameV0,
    pub property_key: Option<CanonicalCustomPropertyNameV0>,
    pub range: TextRange,
    /// For a `CustomPropertyReference` written as `var(--x, fallback)`, records that a
    /// top-level fallback argument is present. The reference cannot be "missing" in any
    /// observable way — the fallback guarantees a value — so the `missingCustomProperty`
    /// lint must skip it. `false` for declarations and fallback-less references.
    pub has_fallback: bool,
    pub value_repr: Option<Box<str>>,
    pub defaulted: bool,
    pub is_top_level: bool,
}

#[derive(Clone)]
pub enum ParsedVariableFactNameV0 {
    NonProperty(String),
    CustomProperty(AuthoredPropertyTextV0),
}

impl fmt::Debug for ParsedVariableFactNameV0 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonProperty(name) => fmt::Debug::fmt(name, formatter),
            Self::CustomProperty(name) => {
                let mut rendered = String::new();
                name.write_into(&mut rendered)?;
                fmt::Debug::fmt(&rendered, formatter)
            }
        }
    }
}

impl ParsedVariableFactNameV0 {
    pub fn as_non_property(&self) -> Option<&str> {
        match self {
            Self::NonProperty(name) => Some(name),
            Self::CustomProperty(_) => None,
        }
    }

    pub fn as_custom_property(&self) -> Option<&AuthoredPropertyTextV0> {
        match self {
            Self::NonProperty(_) => None,
            Self::CustomProperty(name) => Some(name),
        }
    }

    fn identity(&self) -> ParsedVariableFactNameIdentityV0 {
        match self {
            Self::NonProperty(name) => ParsedVariableFactNameIdentityV0::NonProperty(name.clone()),
            Self::CustomProperty(name) => {
                ParsedVariableFactNameIdentityV0::CustomProperty(name.to_custom_key())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum ParsedVariableFactNameIdentityV0 {
    NonProperty(String),
    CustomProperty(CanonicalCustomPropertyNameV0),
}

impl PartialEq for ParsedVariableFactNameV0 {
    fn eq(&self, other: &Self) -> bool {
        self.identity() == other.identity()
    }
}

impl Eq for ParsedVariableFactNameV0 {}

impl PartialEq for ParsedVariableFact {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.name == other.name
            && self.property_key == other.property_key
            && self.range == other.range
            && self.has_fallback == other.has_fallback
            && self.value_repr == other.value_repr
            && self.defaulted == other.defaulted
            && self.is_top_level == other.is_top_level
    }
}

impl Eq for ParsedVariableFact {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedVariableFactKind {
    ScssDeclaration,
    ScssReference,
    LessDeclaration,
    LessReference,
    CustomPropertyDeclaration,
    CustomPropertyReference,
}

#[cfg(test)]
pub(crate) fn collect_variable_facts_from_cst(
    text: &str,
    parsed: &ParseResult,
) -> Vec<ParsedVariableFact> {
    let sink = StyleFactSink::from_cst(text, parsed);
    collect_variable_facts_from_sink(&sink)
}

pub(crate) fn collect_variable_facts_from_sink(
    sink: &StyleFactSink<'_>,
) -> Vec<ParsedVariableFact> {
    let mut variables = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for node in sink.nodes().filter(|node| node.is_top_level) {
        for fact in variable_facts_from_token_view(sink.node_tokens(node)) {
            push_variable_fact(&mut variables, &mut seen, fact);
        }
    }
    if !matches!(sink.dialect(), StyleDialect::Scss | StyleDialect::Sass)
        || !variables
            .iter()
            .any(|fact| fact.kind == ParsedVariableFactKind::ScssDeclaration)
    {
        return variables;
    }
    let declaration_metadata = scss_variable_declaration_metadata_from_sink(sink);
    for fact in &mut variables {
        let key = (u32::from(fact.range.start()), u32::from(fact.range.end()));
        if let Some(metadata) = declaration_metadata.get(&key) {
            fact.value_repr = metadata.value_repr.clone();
            fact.defaulted = metadata.defaulted;
            fact.is_top_level = metadata.is_top_level;
        }
    }
    variables
}

fn variable_facts_from_token_view(tokens: &[Token<'_>]) -> Vec<ParsedVariableFact> {
    let mut variables = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        let kind = match token.kind {
            SyntaxKind::ScssVariable => {
                if scss_variable_token_is_declaration(tokens, index) {
                    ParsedVariableFactKind::ScssDeclaration
                } else {
                    ParsedVariableFactKind::ScssReference
                }
            }
            SyntaxKind::LessVariable => {
                if next_non_trivia_token(tokens, index + 1)
                    .is_some_and(|candidate| candidate.kind == SyntaxKind::Colon)
                {
                    ParsedVariableFactKind::LessDeclaration
                } else {
                    ParsedVariableFactKind::LessReference
                }
            }
            SyntaxKind::CustomPropertyName => {
                if previous_non_trivia_token(tokens, 0, index).is_some_and(|candidate| {
                    matches!(candidate.kind, SyntaxKind::Ampersand | SyntaxKind::Dot)
                }) {
                    continue;
                }
                if let Some(at_rule_name) = containing_at_rule_header_name(tokens, index) {
                    if at_rule_name == "@property" {
                        ParsedVariableFactKind::CustomPropertyDeclaration
                    } else {
                        continue;
                    }
                } else if next_non_trivia_token(tokens, index + 1)
                    .is_some_and(|candidate| candidate.kind == SyntaxKind::Colon)
                {
                    ParsedVariableFactKind::CustomPropertyDeclaration
                } else {
                    ParsedVariableFactKind::CustomPropertyReference
                }
            }
            _ => continue,
        };
        let has_fallback = kind == ParsedVariableFactKind::CustomPropertyReference
            && custom_property_reference_has_var_fallback(tokens, index);
        let property_key = matches!(
            kind,
            ParsedVariableFactKind::CustomPropertyDeclaration
                | ParsedVariableFactKind::CustomPropertyReference
        )
        .then(|| PropertyNameV0::canonical_custom_key(token.text));
        let name = if property_key.is_some() {
            ParsedVariableFactNameV0::CustomProperty(AuthoredPropertyTextV0::new(token.text))
        } else {
            ParsedVariableFactNameV0::NonProperty(token.text.to_string())
        };
        variables.push(ParsedVariableFact {
            kind,
            name,
            property_key,
            range: token.range,
            has_fallback,
            value_repr: None,
            defaulted: false,
            is_top_level: false,
        });
    }
    variables
}

#[derive(Debug, Clone)]
struct ScssVariableDeclarationMetadata {
    value_repr: Option<Box<str>>,
    defaulted: bool,
    is_top_level: bool,
}

fn scss_variable_declaration_metadata_from_sink(
    sink: &StyleFactSink<'_>,
) -> BTreeMap<(u32, u32), ScssVariableDeclarationMetadata> {
    sink.nodes()
        .filter(|node| node.kind == SyntaxKind::ScssVariableDeclaration)
        .filter_map(|node| {
            let tokens = sink.node_tokens(node);
            let variable_index = tokens
                .iter()
                .position(|token| token.kind == SyntaxKind::ScssVariable)?;
            let colon_index = tokens
                .iter()
                .enumerate()
                .skip(variable_index + 1)
                .find_map(|(index, token)| (token.kind == SyntaxKind::Colon).then_some(index))?;
            let (value_end, defaulted) =
                scss_variable_value_end_and_default(tokens, colon_index + 1);
            let value_repr = tokens[colon_index + 1..value_end]
                .iter()
                .map(|token| token.text)
                .collect::<String>();
            let variable = tokens[variable_index];
            Some((
                (
                    u32::from(variable.range.start()),
                    u32::from(variable.range.end()),
                ),
                ScssVariableDeclarationMetadata {
                    value_repr: (!value_repr.trim().is_empty())
                        .then(|| value_repr.trim().to_string().into_boxed_str()),
                    defaulted,
                    is_top_level: node.is_top_level,
                },
            ))
        })
        .collect()
}

fn scss_variable_value_end_and_default(tokens: &[Token<'_>], start: usize) -> (usize, bool) {
    let mut end = tokens.len();
    let mut defaulted = false;
    for index in start..tokens.len() {
        if matches!(
            tokens[index].kind,
            SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon
        ) {
            end = end.min(index);
            break;
        }
        if tokens[index].kind != SyntaxKind::Delim || tokens[index].text != "!" {
            continue;
        }
        let Some(flag) = next_non_trivia_token(tokens, index + 1) else {
            continue;
        };
        if flag.kind == SyntaxKind::Ident
            && matches_ignore_ascii_case(flag.text, &["default", "global"])
        {
            end = end.min(index);
            defaulted |= matches_ignore_ascii_case(flag.text, &["default"]);
        }
    }
    (end, defaulted)
}

fn push_variable_fact(
    variables: &mut Vec<ParsedVariableFact>,
    seen: &mut std::collections::BTreeSet<(
        ParsedVariableFactKind,
        ParsedVariableFactNameIdentityV0,
        u32,
        u32,
        bool,
    )>,
    fact: ParsedVariableFact,
) {
    if seen.insert((
        fact.kind,
        fact.name.identity(),
        u32::from(fact.range.start()),
        u32::from(fact.range.end()),
        fact.has_fallback,
    )) {
        variables.push(fact);
    }
}

/// Detect a `var(--x, fallback)` fallback for the `CustomPropertyName` at `index`.
///
/// True iff the reference is the first argument of an enclosing `var(` call *and* a
/// top-level comma follows it before that call's closing paren. Scoped per-`var()`: in
/// `var(--a, var(--b))` only `--a` carries a fallback; the nested `--b` (no fallback of
/// its own) is unaffected and stays a live `missingCustomProperty` candidate.
fn custom_property_reference_has_var_fallback(tokens: &[Token<'_>], index: usize) -> bool {
    // The reference must be the leading argument of a `var(` call: its immediate
    // non-trivia predecessor is `(`, preceded by an identifier `var`.
    let Some(open_index) = previous_non_trivia_token_index(tokens, index, 0) else {
        return false;
    };
    if tokens[open_index].kind != SyntaxKind::LeftParen {
        return false;
    }
    let Some(callee_index) = previous_non_trivia_token_index(tokens, open_index, 0) else {
        return false;
    };
    if tokens[callee_index].kind != SyntaxKind::Ident
        || !matches_ignore_ascii_case(tokens[callee_index].text, &["var"])
    {
        return false;
    }
    // Scan forward at this call's paren depth for a top-level comma before its close.
    let mut depth = 0usize;
    let mut cursor = open_index;
    while cursor < tokens.len() {
        match tokens[cursor].kind {
            SyntaxKind::LeftParen => depth += 1,
            SyntaxKind::RightParen => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return false;
                }
            }
            SyntaxKind::Comma if depth == 1 => return true,
            _ => {}
        }
        cursor += 1;
    }
    false
}

pub(crate) fn scss_variable_token_is_declaration(tokens: &[Token<'_>], index: usize) -> bool {
    if scss_loop_variable_token_is_binding(tokens, index) {
        return true;
    }
    next_non_trivia_token(tokens, index + 1).is_some_and(|candidate| {
        candidate.kind == SyntaxKind::Colon
            || (matches!(candidate.kind, SyntaxKind::Comma | SyntaxKind::RightParen)
                && containing_at_rule_header_name(tokens, index)
                    .is_some_and(|name| matches_ignore_ascii_case(name, &["@mixin", "@function"])))
    })
}

/// Positional guard for `@each` / `@for` loop bindings.
///
/// In `@each $k, $v in $map` the `$k`/`$v` are *bindings* (declarations), while
/// the iterable `$map` after `in` is a *reference*. In `@for $i from $start
/// through $end` the `$i` is a binding, while `$start`/`$end` after `from` are
/// references. A `$var` is a binding iff it sits in the loop header *before* the
/// top-level separator keyword (`in` for `@each`, `from` for `@for`). `@while` /
/// `@if` headers introduce no bindings and stay reference-only.
fn scss_loop_variable_token_is_binding(tokens: &[Token<'_>], index: usize) -> bool {
    let Some(header_index) = containing_at_rule_header_index(tokens, index) else {
        return false;
    };
    let separator = match () {
        _ if matches_ignore_ascii_case(tokens[header_index].text, &["@each"]) => "in",
        _ if matches_ignore_ascii_case(tokens[header_index].text, &["@for"]) => "from",
        _ => return false,
    };
    // Scan the header from just after the at-keyword up to (but excluding) the
    // variable token. If the top-level separator keyword has already appeared,
    // the variable is part of the iterable/bounds expression -> reference.
    let mut paren_depth = 0usize;
    for token in &tokens[header_index + 1..index] {
        match token.kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::Ident
                if paren_depth == 0 && matches_ignore_ascii_case(token.text, &[separator]) =>
            {
                return false;
            }
            _ => {}
        }
    }
    true
}

/// Like [`containing_at_rule_header_name`] but returns the index of the
/// enclosing `@`-keyword token rather than its text.
pub(crate) fn containing_at_rule_header_index(tokens: &[Token<'_>], index: usize) -> Option<usize> {
    let mut current = index;
    while current > 0 {
        current -= 1;
        let token = tokens.get(current)?;
        if token.kind.is_trivia() {
            continue;
        }
        if matches!(
            token.kind,
            SyntaxKind::Semicolon
                | SyntaxKind::SassOptionalSemicolon
                | SyntaxKind::LeftBrace
                | SyntaxKind::RightBrace
                | SyntaxKind::SassIndent
                | SyntaxKind::SassDedent
        ) {
            return None;
        }
        if token.kind == SyntaxKind::AtKeyword {
            return Some(current);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{StyleDialect, parse};

    #[test]
    fn parsed_variable_fact_name_identity_uses_custom_property_keys() {
        let name = |value: &str| {
            ParsedVariableFactNameV0::CustomProperty(AuthoredPropertyTextV0::new(value))
        };

        assert_eq!(name(r"--f\6f o"), name("--foo"));
        assert_ne!(name("--foo"), name("--FOO"));
    }

    #[test]
    fn parsed_variable_fact_identity_uses_custom_property_keys() -> Result<(), String> {
        let source = ":root { --foo: red; }";
        let parsed = parse(source, StyleDialect::Css);
        let decoded = collect_variable_facts_from_cst(source, &parsed)
            .into_iter()
            .find(|fact| fact.name.as_custom_property().is_some())
            .ok_or_else(|| "custom-property declaration produces a variable fact".to_string())?;
        let mut escaped = decoded.clone();
        escaped.name =
            ParsedVariableFactNameV0::CustomProperty(AuthoredPropertyTextV0::new(r"--f\6f o"));

        assert_eq!(escaped, decoded);
        Ok(())
    }

    #[test]
    fn scss_declarations_expose_values_and_default_flags_from_cst() {
        let source = "$theme: (primary: red, accent: blue) !default;\n.scope { $local: 2px; }";
        let parsed = parse(source, StyleDialect::Scss);
        let facts = collect_variable_facts_from_cst(source, &parsed);

        let theme = facts.iter().find(|fact| {
            fact.kind == ParsedVariableFactKind::ScssDeclaration
                && fact.name.as_non_property() == Some("$theme")
        });
        assert!(theme.is_some(), "top-level variable declaration");
        let Some(theme) = theme else {
            return;
        };
        assert_eq!(
            theme.value_repr.as_deref(),
            Some("(primary: red, accent: blue)")
        );
        assert!(theme.defaulted);
        assert!(theme.is_top_level);

        let local = facts.iter().find(|fact| {
            fact.kind == ParsedVariableFactKind::ScssDeclaration
                && fact.name.as_non_property() == Some("$local")
        });
        assert!(local.is_some(), "local variable declaration");
        let Some(local) = local else {
            return;
        };
        assert_eq!(local.value_repr.as_deref(), Some("2px"));
        assert!(!local.defaulted);
        assert!(!local.is_top_level);
    }
}
