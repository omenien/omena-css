//! Parser facts for Sass and CSS variable-like declarations and references.
//!
//! The collector distinguishes declaration/reference positions at token level
//! so later layers can resolve scope and module visibility explicitly.

use cstree::text::TextRange;
use omena_syntax::SyntaxKind;

use crate::{
    ParseResult, Token, containing_at_rule_header_name, matches_ignore_ascii_case,
    next_non_trivia_token, previous_non_trivia_token, previous_non_trivia_token_index,
};

use super::tokens_from_syntax_node;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedVariableFact {
    pub kind: ParsedVariableFactKind,
    pub name: String,
    pub range: TextRange,
    /// For a `CustomPropertyReference` written as `var(--x, fallback)`, records that a
    /// top-level fallback argument is present. The reference cannot be "missing" in any
    /// observable way — the fallback guarantees a value — so the `missingCustomProperty`
    /// lint must skip it. `false` for declarations and fallback-less references.
    pub has_fallback: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedVariableFactKind {
    ScssDeclaration,
    ScssReference,
    LessDeclaration,
    LessReference,
    CustomPropertyDeclaration,
    CustomPropertyReference,
}

pub(crate) fn collect_variable_facts_from_cst(
    text: &str,
    parsed: &ParseResult,
) -> Vec<ParsedVariableFact> {
    let mut variables = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for tokens in variable_fact_statement_tokens_from_cst(text, parsed) {
        for fact in variable_facts_from_token_view(&tokens) {
            push_variable_fact(&mut variables, &mut seen, fact);
        }
    }
    variables
}

fn variable_fact_statement_tokens_from_cst<'text>(
    text: &'text str,
    parsed: &ParseResult,
) -> Vec<Vec<Token<'text>>> {
    parsed
        .syntax()
        .children()
        .map(|node| tokens_from_syntax_node(text, parsed, node))
        .collect()
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
        variables.push(ParsedVariableFact {
            kind,
            name: token.text.to_string(),
            range: token.range,
            has_fallback,
        });
    }
    variables
}

fn push_variable_fact(
    variables: &mut Vec<ParsedVariableFact>,
    seen: &mut std::collections::BTreeSet<(ParsedVariableFactKind, String, u32, u32, bool)>,
    fact: ParsedVariableFact,
) {
    if seen.insert((
        fact.kind,
        fact.name.clone(),
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
