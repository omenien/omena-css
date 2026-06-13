use cstree::text::TextRange;
use omena_syntax::SyntaxKind;

use crate::{
    Token, css_module_value_statement_end, next_non_trivia_token_index_until, skip_trivia_tokens,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSassSymbolFact {
    pub kind: ParsedSassSymbolFactKind,
    pub symbol_kind: &'static str,
    pub name: String,
    pub role: &'static str,
    pub namespace: Option<String>,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedSassSymbolFactKind {
    VariableDeclaration,
    VariableReference,
    MixinDeclaration,
    MixinInclude,
    FunctionDeclaration,
    FunctionCall,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSassIncludeFact {
    pub name: String,
    pub namespace: Option<String>,
    pub params: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSassModuleEdgeFact {
    pub kind: ParsedSassModuleEdgeFactKind,
    pub source: String,
    pub namespace_kind: Option<&'static str>,
    pub namespace: Option<String>,
    pub forward_prefix: Option<String>,
    pub visibility_filter_kind: Option<&'static str>,
    pub visibility_filter_names: Vec<String>,
    /// RFC-0007-D1 (#44): whether this `@import` target carries a trailing media
    /// qualifier (`@import "foo" screen`, `@import "foo" (min-width: 100px)`). Sass
    /// keeps media-qualified imports as plain CSS (NOT deprecated). Recoverable only
    /// in the parser, where the target's comma-peer segment is still tokenized: a
    /// non-`Comma` significant token after the target String marks the qualifier.
    /// Always `false` for `Use`/`Forward` edges (media qualifiers are `@import`-only).
    pub media_qualified: bool,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedSassModuleEdgeFactKind {
    Use,
    Forward,
    Import,
}

/// RFC-0007-E1 (#45): the target of an `@extend` rule. The `ScssExtendRule` node previously
/// parsed and then discarded its target, so an `@extend %nonexistent` / `@extend .missing`
/// (a dart-sass hard error) went unreported. This fact captures the simple target selector,
/// whether it carries the `!optional` flag, and its source range for diagnostic anchoring.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedExtendTargetFact {
    pub kind: ParsedExtendTargetFactKind,
    pub name: String,
    pub optional: bool,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedExtendTargetFactKind {
    Class,
    Placeholder,
}

/// Capture the target of each `@extend` rule. For each `@extend` keyword, the
/// statement runs to the next `;`/`}`/indent boundary. Within it we capture the
/// first simple target: a `%placeholder` token or a `.class` token pair. Compound
/// targets record only the first simple selector; dart-sass rejects compound
/// `@extend` targets, so the first-simple capture is sufficient for missing-target
/// checks without over-reporting. Interpolated targets produce no simple token
/// here and are skipped because they are not statically checkable.
pub(crate) fn collect_extend_target_facts_from_tokens(
    tokens: &[Token<'_>],
) -> Vec<ParsedExtendTargetFact> {
    let mut targets = Vec::new();

    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::AtKeyword || !token.text.eq_ignore_ascii_case("@extend") {
            continue;
        }
        let start = skip_trivia_tokens(tokens, index + 1, tokens.len());
        let end = css_module_value_statement_end(tokens, start);

        // `!optional` may appear after the target; scan the whole statement for it first.
        let optional = extend_statement_has_optional_flag(tokens, start, end);

        let mut cursor = start;
        let mut captured: Option<ParsedExtendTargetFact> = None;
        while cursor < end {
            let current = tokens[cursor];
            if current.kind == SyntaxKind::ScssPlaceholder {
                captured = Some(ParsedExtendTargetFact {
                    kind: ParsedExtendTargetFactKind::Placeholder,
                    name: current.text.trim_start_matches('%').to_string(),
                    optional,
                    range: current.range,
                });
                break;
            }
            if current.kind == SyntaxKind::Dot
                && let Some(name_index) = next_non_trivia_token_index_until(tokens, cursor + 1, end)
                && tokens[name_index].kind == SyntaxKind::Ident
            {
                let name_token = tokens[name_index];
                let range = TextRange::new(current.range.start(), name_token.range.end());
                captured = Some(ParsedExtendTargetFact {
                    kind: ParsedExtendTargetFactKind::Class,
                    name: name_token.text.to_string(),
                    optional,
                    range,
                });
                break;
            }
            cursor += 1;
        }

        if let Some(target) = captured {
            targets.push(target);
        }
    }

    targets
}

fn extend_statement_has_optional_flag(tokens: &[Token<'_>], start: usize, end: usize) -> bool {
    let mut index = start;
    while index < end {
        if tokens[index].kind == SyntaxKind::Delim
            && tokens[index].text == "!"
            && let Some(next_index) = next_non_trivia_token_index_until(tokens, index + 1, end)
            && tokens[next_index].kind == SyntaxKind::Ident
            && tokens[next_index].text.eq_ignore_ascii_case("optional")
        {
            return true;
        }
        index += 1;
    }
    false
}
