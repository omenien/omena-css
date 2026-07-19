//! Parser facts for Sass symbols, module edges, includes, and extend targets.
//!
//! These records intentionally stop at syntax-level visibility and target
//! extraction; module graph resolution is owned by downstream query layers.

use cstree::text::{TextRange, TextSize};
use omena_syntax::SyntaxKind;
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ParseResult, Token, containing_at_rule_header_name, css_module_value_source_name,
    css_module_value_statement_end, matches_ignore_ascii_case, next_non_trivia_token,
    next_non_trivia_token_index_until, previous_non_trivia_token, previous_non_trivia_token_index,
    skip_trivia_tokens, top_level_token_text_index,
};

use super::scss_variable_token_is_declaration;
use super::syntax_node_is_top_level;
use super::tokens_from_syntax_node;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSassSymbolFact {
    pub kind: ParsedSassSymbolFactKind,
    pub symbol_kind: &'static str,
    pub name: String,
    pub role: &'static str,
    pub namespace: Option<String>,
    pub range: TextRange,
    pub callable_signature: Option<ParsedSassCallableSignatureFact>,
    pub is_top_level: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSassCallableSignatureFact {
    pub parameters: Vec<ParsedSassCallableParameterFact>,
    pub accepts_content: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSassCallableParameterFact {
    pub name: String,
    pub default_repr: Option<String>,
    pub variadic: bool,
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

pub(crate) fn collect_sass_symbol_facts_from_cst(
    text: &str,
    parsed: &ParseResult,
) -> Vec<ParsedSassSymbolFact> {
    let statement_tokens = sass_symbol_statement_tokens_from_cst(text, parsed);
    let declared_functions = statement_tokens
        .iter()
        .flat_map(|tokens| collect_sass_callable_declaration_names(tokens, "@function"))
        .collect::<BTreeSet<_>>();
    let mut facts = statement_tokens
        .iter()
        .flat_map(|tokens| {
            sass_symbol_facts_from_token_view_with_declared_functions(tokens, &declared_functions)
        })
        .collect::<Vec<_>>();
    let declaration_metadata = sass_callable_declaration_metadata_from_cst(text, parsed);
    for fact in &mut facts {
        let key = (
            fact.kind,
            u32::from(fact.range.start()),
            u32::from(fact.range.end()),
        );
        if let Some(metadata) = declaration_metadata.get(&key) {
            fact.callable_signature = Some(metadata.signature.clone());
            fact.is_top_level = metadata.is_top_level;
        }
    }
    facts
}

fn sass_symbol_statement_tokens_from_cst<'text>(
    text: &'text str,
    parsed: &ParseResult,
) -> Vec<Vec<Token<'text>>> {
    parsed
        .syntax()
        .children()
        .map(|node| tokens_from_syntax_node(text, parsed, node))
        .collect()
}

fn sass_symbol_facts_from_token_view_with_declared_functions(
    tokens: &[Token<'_>],
    declared_functions: &BTreeSet<String>,
) -> Vec<ParsedSassSymbolFact> {
    let mut symbols = Vec::new();

    for (index, token) in tokens.iter().enumerate() {
        match token.kind {
            SyntaxKind::ScssVariable => {
                let kind = if scss_variable_token_is_declaration(tokens, index) {
                    ParsedSassSymbolFactKind::VariableDeclaration
                } else {
                    ParsedSassSymbolFactKind::VariableReference
                };
                let namespace = (!scss_variable_token_is_declaration(tokens, index))
                    .then(|| sass_member_namespace_before(tokens, index))
                    .flatten();
                symbols.push(ParsedSassSymbolFact {
                    kind,
                    symbol_kind: "variable",
                    name: token.text.trim_start_matches('$').to_string(),
                    role: match kind {
                        ParsedSassSymbolFactKind::VariableDeclaration => "declaration",
                        _ => "reference",
                    },
                    namespace,
                    range: sass_symbol_variable_range(token, kind),
                    callable_signature: None,
                    is_top_level: false,
                });
            }
            SyntaxKind::AtKeyword if matches_ignore_ascii_case(token.text, &["@mixin"]) => {
                if let Some(name) = sass_callable_name_after_at_rule(tokens, index) {
                    symbols.push(ParsedSassSymbolFact {
                        kind: ParsedSassSymbolFactKind::MixinDeclaration,
                        symbol_kind: "mixin",
                        name: name.text.to_string(),
                        role: "declaration",
                        namespace: None,
                        range: name.range,
                        callable_signature: None,
                        is_top_level: false,
                    });
                }
            }
            SyntaxKind::AtKeyword if matches_ignore_ascii_case(token.text, &["@include"]) => {
                if let Some((name, namespace)) = sass_include_name_after_at_rule(tokens, index) {
                    symbols.push(ParsedSassSymbolFact {
                        kind: ParsedSassSymbolFactKind::MixinInclude,
                        symbol_kind: "mixin",
                        name: name.text.to_string(),
                        role: "include",
                        namespace,
                        range: name.range,
                        callable_signature: None,
                        is_top_level: false,
                    });
                }
            }
            SyntaxKind::AtKeyword if matches_ignore_ascii_case(token.text, &["@function"]) => {
                if let Some(name) = sass_callable_name_after_at_rule(tokens, index) {
                    symbols.push(ParsedSassSymbolFact {
                        kind: ParsedSassSymbolFactKind::FunctionDeclaration,
                        symbol_kind: "function",
                        name: name.text.to_string(),
                        role: "declaration",
                        namespace: None,
                        range: name.range,
                        callable_signature: None,
                        is_top_level: false,
                    });
                }
            }
            SyntaxKind::Ident
                if (declared_functions.contains(&canonical_sass_callable_name(token.text))
                    || sass_member_namespace_before(tokens, index).is_some())
                    && next_non_trivia_token(tokens, index + 1)
                        .is_some_and(|candidate| candidate.kind == SyntaxKind::LeftParen)
                    && !containing_at_rule_header_name(tokens, index)
                        .is_some_and(|name| matches_ignore_ascii_case(name, &["@include"]))
                    && previous_non_trivia_token(tokens, 0, index).is_none_or(|candidate| {
                        !matches!(candidate.kind, SyntaxKind::AtKeyword)
                    }) =>
            {
                symbols.push(ParsedSassSymbolFact {
                    kind: ParsedSassSymbolFactKind::FunctionCall,
                    symbol_kind: "function",
                    name: token.text.to_string(),
                    role: "call",
                    namespace: sass_member_namespace_before(tokens, index),
                    range: token.range,
                    callable_signature: None,
                    is_top_level: false,
                });
            }
            _ => {}
        }
    }

    symbols
}

#[derive(Debug, Clone)]
struct SassCallableDeclarationMetadata {
    signature: ParsedSassCallableSignatureFact,
    is_top_level: bool,
}

fn sass_callable_declaration_metadata_from_cst(
    text: &str,
    parsed: &ParseResult,
) -> BTreeMap<(ParsedSassSymbolFactKind, u32, u32), SassCallableDeclarationMetadata> {
    parsed
        .syntax()
        .descendants()
        .filter_map(|node| {
            let fact_kind = match node.kind() {
                SyntaxKind::ScssMixinDeclaration => ParsedSassSymbolFactKind::MixinDeclaration,
                SyntaxKind::ScssFunctionDeclaration => {
                    ParsedSassSymbolFactKind::FunctionDeclaration
                }
                _ => return None,
            };
            let tokens = tokens_from_syntax_node(text, parsed, node);
            let at_rule_index = tokens
                .iter()
                .position(|token| token.kind == SyntaxKind::AtKeyword)?;
            let name = sass_callable_name_after_at_rule(&tokens, at_rule_index)?;
            let signature = ParsedSassCallableSignatureFact {
                parameters: sass_callable_parameters_from_tokens(&tokens, at_rule_index),
                accepts_content: fact_kind == ParsedSassSymbolFactKind::MixinDeclaration
                    && sass_callable_node_accepts_content(node),
            };
            let key = (
                fact_kind,
                u32::from(name.range.start()),
                u32::from(name.range.end()),
            );
            let metadata = SassCallableDeclarationMetadata {
                signature,
                is_top_level: syntax_node_is_top_level(node),
            };
            Some((key, metadata))
        })
        .collect()
}

fn sass_callable_node_accepts_content(
    declaration: &cstree::syntax::SyntaxNode<SyntaxKind>,
) -> bool {
    declaration
        .descendants()
        .filter(|node| node.kind() == SyntaxKind::ScssContentRule)
        .any(|content| {
            content
                .ancestors()
                .skip(1)
                .take_while(|ancestor| *ancestor != declaration)
                .all(|ancestor| {
                    !matches!(
                        ancestor.kind(),
                        SyntaxKind::ScssMixinDeclaration | SyntaxKind::ScssFunctionDeclaration
                    )
                })
        })
}

fn sass_callable_parameters_from_tokens(
    tokens: &[Token<'_>],
    at_rule_index: usize,
) -> Vec<ParsedSassCallableParameterFact> {
    let Some(open_index) = tokens
        .iter()
        .enumerate()
        .skip(at_rule_index + 1)
        .find_map(|(index, token)| (token.kind == SyntaxKind::LeftParen).then_some(index))
    else {
        return Vec::new();
    };
    let Some(close_index) = matching_right_paren_index(tokens, open_index) else {
        return Vec::new();
    };

    split_sass_parameter_ranges(tokens, open_index + 1, close_index)
        .into_iter()
        .filter_map(|(start, end)| sass_parameter_from_token_range(tokens, start, end))
        .collect()
}

fn matching_right_paren_index(tokens: &[Token<'_>], open_index: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(open_index) {
        match token.kind {
            SyntaxKind::LeftParen => depth += 1,
            SyntaxKind::RightParen => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn split_sass_parameter_ranges(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut segment_start = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    for (index, token) in tokens.iter().enumerate().take(end).skip(start) {
        match token.kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            SyntaxKind::LeftBrace => brace_depth += 1,
            SyntaxKind::RightBrace => brace_depth = brace_depth.saturating_sub(1),
            SyntaxKind::Comma if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                ranges.push((segment_start, index));
                segment_start = index + 1;
            }
            _ => {}
        }
    }
    ranges.push((segment_start, end));
    ranges
}

fn sass_parameter_from_token_range(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Option<ParsedSassCallableParameterFact> {
    let variable_index =
        (start..end).find(|index| tokens[*index].kind == SyntaxKind::ScssVariable)?;
    let colon_index =
        (variable_index + 1..end).find(|index| tokens[*index].kind == SyntaxKind::Colon);
    let default_repr = colon_index
        .map(|colon| {
            tokens[colon + 1..end]
                .iter()
                .map(|token| token.text)
                .collect::<String>()
        })
        .map(|value| value.trim().trim_end_matches("...").trim().to_string())
        .filter(|value| !value.is_empty());
    let suffix = tokens[variable_index + 1..end]
        .iter()
        .map(|token| token.text)
        .collect::<String>();
    Some(ParsedSassCallableParameterFact {
        name: tokens[variable_index]
            .text
            .trim_start_matches('$')
            .to_string(),
        default_repr,
        variadic: suffix.trim().ends_with("..."),
    })
}

fn sass_symbol_variable_range(token: &Token<'_>, kind: ParsedSassSymbolFactKind) -> TextRange {
    if kind == ParsedSassSymbolFactKind::VariableDeclaration && token.text.starts_with('$') {
        let start = u32::from(token.range.start());
        let end = u32::from(token.range.end());
        if start < end {
            return TextRange::new(TextSize::from(start + 1), TextSize::from(end));
        }
    }
    token.range
}

fn collect_sass_callable_declaration_names(
    tokens: &[Token<'_>],
    at_keyword: &str,
) -> BTreeSet<String> {
    tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            (token.kind == SyntaxKind::AtKeyword
                && matches_ignore_ascii_case(token.text, &[at_keyword]))
            .then(|| sass_callable_name_after_at_rule(tokens, index))
            .flatten()
            .map(|name| canonical_sass_callable_name(name.text))
        })
        .collect()
}

fn canonical_sass_callable_name(name: &str) -> String {
    name.trim().replace('_', "-")
}

fn sass_callable_name_after_at_rule<'text>(
    tokens: &[Token<'text>],
    at_rule_index: usize,
) -> Option<Token<'text>> {
    let statement_end = css_module_value_statement_end(tokens, at_rule_index + 1);
    let name_index = next_non_trivia_token_index_until(tokens, at_rule_index + 1, statement_end)?;
    let name = tokens[name_index];
    if name.kind != SyntaxKind::Ident {
        return None;
    }
    if next_non_trivia_token_index_until(tokens, name_index + 1, statement_end)
        .is_some_and(|next| tokens[next].kind == SyntaxKind::Dot)
    {
        return None;
    }
    Some(name)
}

fn sass_include_name_after_at_rule<'text>(
    tokens: &[Token<'text>],
    at_rule_index: usize,
) -> Option<(Token<'text>, Option<String>)> {
    let statement_end = css_module_value_statement_end(tokens, at_rule_index + 1);
    let first_index = next_non_trivia_token_index_until(tokens, at_rule_index + 1, statement_end)?;
    let first = tokens[first_index];
    if first.kind != SyntaxKind::Ident {
        return None;
    }
    let Some(dot_index) = next_non_trivia_token_index_until(tokens, first_index + 1, statement_end)
    else {
        return Some((first, None));
    };
    if tokens[dot_index].kind != SyntaxKind::Dot {
        return Some((first, None));
    }
    let member_index = next_non_trivia_token_index_until(tokens, dot_index + 1, statement_end)?;
    let member = tokens[member_index];
    (member.kind == SyntaxKind::Ident).then(|| (member, Some(first.text.to_string())))
}

fn sass_member_namespace_before(tokens: &[Token<'_>], member_index: usize) -> Option<String> {
    let dot_index = previous_non_trivia_token_index(tokens, member_index, 0)?;
    if tokens[dot_index].kind != SyntaxKind::Dot {
        return None;
    }
    let namespace = tokens[previous_non_trivia_token_index(tokens, dot_index, 0)?];
    (namespace.kind == SyntaxKind::Ident).then(|| namespace.text.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSassIncludeFact {
    pub name: String,
    pub namespace: Option<String>,
    pub params: String,
    pub range: TextRange,
}

pub(crate) fn collect_sass_include_facts_from_cst(
    source: &str,
    parsed: &ParseResult,
) -> Vec<ParsedSassIncludeFact> {
    let mut includes = Vec::new();
    for tokens in scss_include_rule_tokens_from_cst(source, parsed) {
        collect_sass_include_facts_from_rule_tokens(&tokens, &mut includes);
    }
    includes
}

fn collect_sass_include_facts_from_rule_tokens(
    tokens: &[Token<'_>],
    includes: &mut Vec<ParsedSassIncludeFact>,
) {
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::AtKeyword
            || !matches_ignore_ascii_case(token.text, &["@include"])
        {
            continue;
        }
        let statement_end = css_module_value_statement_end(tokens, index + 1);
        let Some((name, namespace)) = sass_include_name_after_at_rule(tokens, index) else {
            continue;
        };
        let header_end = previous_non_trivia_token_index(tokens, statement_end, index + 1)
            .map(|previous| tokens[previous].range.end())
            .unwrap_or(name.range.end());
        let params = token_text_between_offsets(tokens, name.range.end(), header_end)
            .trim()
            .to_string();
        includes.push(ParsedSassIncludeFact {
            name: name.text.to_string(),
            namespace,
            params,
            range: TextRange::new(token.range.start(), header_end),
        });
    }
}

fn scss_include_rule_tokens_from_cst<'text>(
    text: &'text str,
    parsed: &ParseResult,
) -> Vec<Vec<Token<'text>>> {
    parsed
        .syntax()
        .descendants()
        .filter(|node| node.kind() == SyntaxKind::ScssIncludeRule)
        .map(|node| tokens_from_syntax_node(text, parsed, node))
        .collect()
}

fn token_text_between_offsets(
    tokens: &[Token<'_>],
    start: cstree::text::TextSize,
    end: cstree::text::TextSize,
) -> String {
    tokens
        .iter()
        .filter(|token| token.range.start() >= start && token.range.end() <= end)
        .map(|token| token.text)
        .collect()
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

pub(crate) fn collect_sass_module_edge_facts_from_cst(
    text: &str,
    parsed: &ParseResult,
) -> Vec<ParsedSassModuleEdgeFact> {
    let mut edges = Vec::new();
    let mut seen = BTreeSet::new();
    for tokens in sass_module_rule_tokens_from_cst(text, parsed) {
        collect_sass_module_edge_facts_from_rule_tokens(&tokens, &mut edges, &mut seen);
    }
    edges
}

fn collect_sass_module_edge_facts_from_rule_tokens(
    tokens: &[Token<'_>],
    edges: &mut Vec<ParsedSassModuleEdgeFact>,
    seen: &mut BTreeSet<(ParsedSassModuleEdgeFactKind, String, u32, u32)>,
) {
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::AtKeyword {
            continue;
        }
        let Some(kind) = sass_module_edge_kind(token.text) else {
            continue;
        };
        let start = skip_trivia_tokens(tokens, index + 1, tokens.len());
        let end = css_module_value_statement_end(tokens, start);
        if kind == ParsedSassModuleEdgeFactKind::Import {
            collect_sass_import_module_edges(tokens, start, end, edges, seen);
            continue;
        }
        let Some(source_index) = next_non_trivia_token_index_until(tokens, start, end) else {
            continue;
        };
        let source = tokens[source_index];
        if !matches!(source.kind, SyntaxKind::String | SyntaxKind::Url) {
            continue;
        }
        let source_name = css_module_value_source_name(source);
        let (namespace_kind, namespace) = if kind == ParsedSassModuleEdgeFactKind::Use {
            sass_module_use_namespace(tokens, source_name.as_str(), source_index + 1, end)
        } else {
            (None, None)
        };
        let (visibility_filter_kind, visibility_filter_names) =
            if kind == ParsedSassModuleEdgeFactKind::Forward {
                sass_module_forward_visibility_filter(tokens, source_index + 1, end)
            } else {
                (None, Vec::new())
            };
        let forward_prefix = if kind == ParsedSassModuleEdgeFactKind::Forward {
            sass_module_forward_prefix(tokens, source_index + 1, end)
        } else {
            None
        };
        push_sass_module_edge_fact(
            edges,
            seen,
            ParsedSassModuleEdgeFact {
                kind,
                source: source_name,
                namespace_kind,
                namespace,
                forward_prefix,
                visibility_filter_kind,
                visibility_filter_names,
                media_qualified: false,
                range: source.range,
            },
        );
    }
}

fn sass_module_rule_tokens_from_cst<'text>(
    text: &'text str,
    parsed: &ParseResult,
) -> Vec<Vec<Token<'text>>> {
    parsed
        .syntax()
        .descendants()
        .filter(|node| {
            matches!(
                node.kind(),
                SyntaxKind::ScssUseRule | SyntaxKind::ScssForwardRule | SyntaxKind::ImportRule
            )
        })
        .map(|node| tokens_from_syntax_node(text, parsed, node))
        .collect()
}

fn sass_module_edge_kind(text: &str) -> Option<ParsedSassModuleEdgeFactKind> {
    if matches_ignore_ascii_case(text, &["@use"]) {
        Some(ParsedSassModuleEdgeFactKind::Use)
    } else if matches_ignore_ascii_case(text, &["@forward"]) {
        Some(ParsedSassModuleEdgeFactKind::Forward)
    } else if matches_ignore_ascii_case(text, &["@import"]) {
        Some(ParsedSassModuleEdgeFactKind::Import)
    } else {
        None
    }
}

fn collect_sass_import_module_edges(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    edges: &mut Vec<ParsedSassModuleEdgeFact>,
    seen: &mut BTreeSet<(ParsedSassModuleEdgeFactKind, String, u32, u32)>,
) {
    for index in start..end {
        let token = tokens[index];
        if !matches!(token.kind, SyntaxKind::String | SyntaxKind::Url) {
            continue;
        }
        // A trailing media qualifier keeps `@import` as plain CSS. Classify per
        // comma-peer target: `@import "a", "b" screen` qualifies only `"b"`.
        let media_qualified = next_non_trivia_token_index_until(tokens, index + 1, end)
            .is_some_and(|next| tokens[next].kind != SyntaxKind::Comma);
        push_sass_module_edge_fact(
            edges,
            seen,
            ParsedSassModuleEdgeFact {
                kind: ParsedSassModuleEdgeFactKind::Import,
                source: css_module_value_source_name(token),
                namespace_kind: None,
                namespace: None,
                forward_prefix: None,
                visibility_filter_kind: None,
                visibility_filter_names: Vec::new(),
                media_qualified,
                range: token.range,
            },
        );
    }
}

fn sass_module_use_namespace(
    tokens: &[Token<'_>],
    source: &str,
    start: usize,
    end: usize,
) -> (Option<&'static str>, Option<String>) {
    let Some(as_index) = top_level_token_text_index(tokens, start, end, "as") else {
        return (
            Some("default"),
            sass_module_default_namespace(source).map(str::to_string),
        );
    };
    let Some(namespace_index) = next_non_trivia_token_index_until(tokens, as_index + 1, end) else {
        return (Some("invalid"), None);
    };
    let namespace = tokens[namespace_index];
    match namespace.kind {
        SyntaxKind::Star => (Some("wildcard"), None),
        SyntaxKind::Ident => (Some("alias"), Some(namespace.text.to_string())),
        _ => (Some("invalid"), None),
    }
}

fn sass_module_forward_prefix(tokens: &[Token<'_>], start: usize, end: usize) -> Option<String> {
    let as_index = top_level_token_text_index(tokens, start, end, "as")?;
    let prefix_index = next_non_trivia_token_index_until(tokens, as_index + 1, end)?;
    let prefix = tokens[prefix_index].text.trim();
    if prefix.is_empty() {
        return None;
    }
    Some(prefix.to_string())
}

fn sass_module_forward_visibility_filter(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> (Option<&'static str>, Vec<String>) {
    let show_index = top_level_token_text_index(tokens, start, end, "show");
    let hide_index = top_level_token_text_index(tokens, start, end, "hide");
    let (filter_kind, filter_index) = match (show_index, hide_index) {
        (Some(show_index), Some(hide_index)) if show_index <= hide_index => ("show", show_index),
        (Some(_), Some(hide_index)) => ("hide", hide_index),
        (Some(show_index), None) => ("show", show_index),
        (None, Some(hide_index)) => ("hide", hide_index),
        (None, None) => return (None, Vec::new()),
    };
    let clause_end =
        top_level_token_text_index(tokens, filter_index + 1, end, "with").unwrap_or(end);
    (
        Some(filter_kind),
        sass_module_visibility_filter_names(tokens, filter_index + 1, clause_end),
    )
}

fn sass_module_visibility_filter_names(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Vec<String> {
    let mut names = BTreeSet::new();
    for token in &tokens[start..end] {
        match token.kind {
            SyntaxKind::Ident | SyntaxKind::ScssVariable => {
                if matches_ignore_ascii_case(token.text, &["show", "hide", "with", "as"]) {
                    continue;
                }
                let name = token.text.trim_start_matches('$');
                if !name.is_empty() {
                    names.insert(name.to_string());
                }
            }
            _ => {}
        }
    }
    names.into_iter().collect()
}

fn sass_module_default_namespace(source: &str) -> Option<&str> {
    let basename = source
        .rsplit(['/', '\\', ':'])
        .next()
        .unwrap_or(source)
        .trim_start_matches('_');
    let namespace = basename.split('.').next().unwrap_or(basename);
    (!namespace.is_empty()).then_some(namespace)
}

fn push_sass_module_edge_fact(
    edges: &mut Vec<ParsedSassModuleEdgeFact>,
    seen: &mut BTreeSet<(ParsedSassModuleEdgeFactKind, String, u32, u32)>,
    edge: ParsedSassModuleEdgeFact,
) {
    let start: u32 = edge.range.start().into();
    let end: u32 = edge.range.end().into();
    if seen.insert((edge.kind, edge.source.clone(), start, end)) {
        edges.push(edge);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSassPlaceholderDefinitionFact {
    pub name: String,
    pub range: TextRange,
    pub is_top_level: bool,
}

pub(crate) fn collect_sass_placeholder_definition_facts_from_cst(
    text: &str,
    parsed: &ParseResult,
) -> Vec<ParsedSassPlaceholderDefinitionFact> {
    parsed
        .syntax()
        .descendants()
        .filter(|node| node.kind() == SyntaxKind::ScssPlaceholderSelector)
        .filter_map(|node| {
            let placeholder = tokens_from_syntax_node(text, parsed, node)
                .into_iter()
                .find(|token| token.kind == SyntaxKind::ScssPlaceholder)?;
            let rule = node
                .ancestors()
                .find(|ancestor| ancestor.kind() == SyntaxKind::Rule)?;
            Some(ParsedSassPlaceholderDefinitionFact {
                name: placeholder.text.trim_start_matches('%').to_string(),
                range: placeholder.range,
                is_top_level: syntax_node_is_top_level(rule),
            })
        })
        .collect()
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
pub(crate) fn collect_extend_target_facts_from_cst(
    text: &str,
    parsed: &ParseResult,
) -> Vec<ParsedExtendTargetFact> {
    let mut targets = Vec::new();
    for tokens in scss_extend_rule_tokens_from_cst(text, parsed) {
        collect_extend_target_facts_from_rule_tokens(&tokens, &mut targets);
    }
    targets
}

fn collect_extend_target_facts_from_rule_tokens(
    tokens: &[Token<'_>],
    targets: &mut Vec<ParsedExtendTargetFact>,
) {
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::AtKeyword
            || !matches_ignore_ascii_case(token.text, &["@extend"])
        {
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
}

fn scss_extend_rule_tokens_from_cst<'text>(
    text: &'text str,
    parsed: &ParseResult,
) -> Vec<Vec<Token<'text>>> {
    parsed
        .syntax()
        .descendants()
        .filter(|node| node.kind() == SyntaxKind::ScssExtendRule)
        .map(|node| tokens_from_syntax_node(text, parsed, node))
        .collect()
}

fn extend_statement_has_optional_flag(tokens: &[Token<'_>], start: usize, end: usize) -> bool {
    let mut index = start;
    while index < end {
        if tokens[index].kind == SyntaxKind::Delim
            && tokens[index].text == "!"
            && let Some(next_index) = next_non_trivia_token_index_until(tokens, index + 1, end)
            && tokens[next_index].kind == SyntaxKind::Ident
            && matches_ignore_ascii_case(tokens[next_index].text, &["optional"])
        {
            return true;
        }
        index += 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{StyleDialect, parse};

    #[test]
    fn callable_declarations_expose_export_signatures_from_cst() {
        let source = r#"
@mixin surface($tone: red, $parts...) {
  @content;
}

@function scale($value: 1) {
  @return $value;
}
"#;
        let parsed = parse(source, StyleDialect::Scss);
        let facts = collect_sass_symbol_facts_from_cst(source, &parsed);

        let mixin = facts
            .iter()
            .find(|fact| fact.kind == ParsedSassSymbolFactKind::MixinDeclaration);
        assert!(mixin.is_some(), "mixin declaration fact");
        let Some(mixin) = mixin else {
            return;
        };
        assert!(mixin.is_top_level);
        let signature = mixin.callable_signature.as_ref();
        assert!(signature.is_some(), "mixin signature");
        let Some(signature) = signature else {
            return;
        };
        assert_eq!(
            signature.parameters,
            vec![
                ParsedSassCallableParameterFact {
                    name: "tone".to_string(),
                    default_repr: Some("red".to_string()),
                    variadic: false,
                },
                ParsedSassCallableParameterFact {
                    name: "parts".to_string(),
                    default_repr: None,
                    variadic: true,
                },
            ]
        );
        assert!(signature.accepts_content);

        let function = facts
            .iter()
            .find(|fact| fact.kind == ParsedSassSymbolFactKind::FunctionDeclaration);
        assert!(function.is_some(), "function declaration fact");
        let Some(function) = function else {
            return;
        };
        assert!(function.is_top_level);
        let function_signature = function.callable_signature.as_ref();
        assert!(function_signature.is_some(), "function signature");
        let Some(function_signature) = function_signature else {
            return;
        };
        assert_eq!(
            function_signature.parameters[0].default_repr.as_deref(),
            Some("1")
        );
        assert!(!function_signature.accepts_content);
    }

    #[test]
    fn placeholder_definitions_remain_distinct_from_extend_targets() {
        let source = "%surface { color: red; }\n.card { @extend %surface; }";
        let parsed = parse(source, StyleDialect::Scss);

        let definitions = collect_sass_placeholder_definition_facts_from_cst(source, &parsed);
        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].name, "surface");
        assert!(definitions[0].is_top_level);

        let targets = collect_extend_target_facts_from_cst(source, &parsed);
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].name, "surface");
    }
}
