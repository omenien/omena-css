//! Parser facts for CSS Modules `:export`, `:import`, `@value`, and `composes`.
//!
//! This module stays syntax-only: it records local edges and references so
//! query/resolution layers can perform cross-file interpretation later.

use cstree::{syntax::SyntaxNode, text::TextRange};
use omena_syntax::SyntaxKind;
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ParseResult, SelectorBranch, Token, css_module_block_scope_marker_in_header,
    find_block_after_header, next_non_trivia_token_index_until, previous_non_trivia_token_index,
    resolve_selector_header, skip_statement, skip_trivia_tokens, style_wrapper_at_rule,
    top_level_token_kind_index, top_level_token_text_index,
};

use super::tokens_from_syntax_node;

#[cfg(feature = "internal-oracle")]
use crate::matching_right_brace;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCssModuleValueFact {
    pub kind: ParsedCssModuleValueFactKind,
    pub name: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedCssModuleValueFactKind {
    Definition,
    Reference,
    ImportSource,
}

#[cfg(feature = "internal-oracle")]
pub(crate) fn collect_css_module_value_facts_from_tokens(
    tokens: &[Token<'_>],
) -> Vec<ParsedCssModuleValueFact> {
    css_module_value_facts_from_token_view(tokens)
}

pub(crate) fn collect_css_module_value_facts_from_cst(
    text: &str,
    parsed: &ParseResult,
) -> Vec<ParsedCssModuleValueFact> {
    css_module_value_facts_from_cst_nodes(text, parsed)
}

#[cfg(feature = "internal-oracle")]
fn css_module_value_facts_from_token_view(tokens: &[Token<'_>]) -> Vec<ParsedCssModuleValueFact> {
    let mut values = Vec::new();
    let mut seen = BTreeSet::new();
    let value_path_aliases = collect_css_module_value_path_aliases_from_tokens(tokens);
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::AtKeyword || !token.text.eq_ignore_ascii_case("@value") {
            continue;
        }

        let start = skip_trivia_tokens(tokens, index + 1, tokens.len());
        let end = css_module_value_statement_end(tokens, start);
        let colon_index = top_level_token_kind_index(tokens, start, end, SyntaxKind::Colon);
        let from_index = top_level_token_text_index(tokens, start, end, "from");

        if let Some(from_index) = from_index
            && match colon_index {
                Some(colon_index) => from_index < colon_index,
                None => true,
            }
        {
            collect_css_module_value_import_facts(
                tokens,
                start,
                from_index,
                end,
                &value_path_aliases,
                &mut values,
                &mut seen,
            );
            continue;
        }

        if let Some(colon_index) = colon_index {
            if css_module_value_path_alias_from_tokens(tokens, start, colon_index, end).is_some() {
                continue;
            }
            collect_css_module_value_definition_facts(
                tokens,
                start,
                colon_index,
                &mut values,
                &mut seen,
            );
            collect_css_module_value_reference_facts(
                tokens,
                colon_index + 1,
                end,
                &mut values,
                &mut seen,
            );
        } else {
            collect_css_module_value_definition_facts(tokens, start, end, &mut values, &mut seen);
        }
    }
    let local_value_names = values
        .iter()
        .filter(|value| value.kind == ParsedCssModuleValueFactKind::Definition)
        .map(|value| value.name.clone())
        .collect::<BTreeSet<_>>();
    collect_css_module_value_declaration_reference_facts(
        tokens,
        0,
        tokens.len(),
        &local_value_names,
        &mut values,
        &mut seen,
    );
    values
}

fn css_module_value_facts_from_cst_nodes(
    text: &str,
    parsed: &ParseResult,
) -> Vec<ParsedCssModuleValueFact> {
    let mut values = Vec::new();
    let mut seen = BTreeSet::new();
    let value_path_aliases = collect_css_module_value_path_aliases_from_cst_nodes(text, parsed);
    for tokens in css_module_value_statement_tokens_from_cst(text, parsed) {
        collect_css_module_value_statement_facts(
            &tokens,
            &value_path_aliases,
            &mut values,
            &mut seen,
        );
    }
    let local_value_names = values
        .iter()
        .filter(|value| value.kind == ParsedCssModuleValueFactKind::Definition)
        .map(|value| value.name.clone())
        .collect::<BTreeSet<_>>();
    for tokens in css_module_value_reference_declaration_tokens_from_cst(text, parsed) {
        collect_css_module_value_declaration_reference_facts_from_declaration_tokens(
            &tokens,
            &local_value_names,
            &mut values,
            &mut seen,
        );
    }
    values
}

fn collect_css_module_value_statement_facts(
    tokens: &[Token<'_>],
    value_path_aliases: &BTreeMap<String, String>,
    values: &mut Vec<ParsedCssModuleValueFact>,
    seen: &mut BTreeSet<(ParsedCssModuleValueFactKind, String, u32, u32)>,
) {
    let Some(index) = tokens.iter().position(|token| {
        token.kind == SyntaxKind::AtKeyword && token.text.eq_ignore_ascii_case("@value")
    }) else {
        return;
    };

    let start = skip_trivia_tokens(tokens, index + 1, tokens.len());
    let end = css_module_value_statement_end(tokens, start);
    let colon_index = top_level_token_kind_index(tokens, start, end, SyntaxKind::Colon);
    let from_index = top_level_token_text_index(tokens, start, end, "from");

    if let Some(from_index) = from_index
        && match colon_index {
            Some(colon_index) => from_index < colon_index,
            None => true,
        }
    {
        collect_css_module_value_import_facts(
            tokens,
            start,
            from_index,
            end,
            value_path_aliases,
            values,
            seen,
        );
        return;
    }

    if let Some(colon_index) = colon_index {
        if css_module_value_path_alias_from_tokens(tokens, start, colon_index, end).is_some() {
            return;
        }
        collect_css_module_value_definition_facts(tokens, start, colon_index, values, seen);
        collect_css_module_value_reference_facts(tokens, colon_index + 1, end, values, seen);
    } else {
        collect_css_module_value_definition_facts(tokens, start, end, values, seen);
    }
}

#[cfg(feature = "internal-oracle")]
fn collect_css_module_value_path_aliases_from_tokens(
    tokens: &[Token<'_>],
) -> BTreeMap<String, String> {
    let mut aliases = BTreeMap::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::AtKeyword || !token.text.eq_ignore_ascii_case("@value") {
            continue;
        }

        let start = skip_trivia_tokens(tokens, index + 1, tokens.len());
        let end = css_module_value_statement_end(tokens, start);
        let Some(colon_index) = top_level_token_kind_index(tokens, start, end, SyntaxKind::Colon)
        else {
            continue;
        };
        if top_level_token_text_index(tokens, start, end, "from").is_some() {
            continue;
        }
        if let Some((name, target)) =
            css_module_value_path_alias_from_tokens(tokens, start, colon_index, end)
        {
            aliases.insert(name, target);
        }
    }
    aliases
}

fn collect_css_module_value_path_aliases_from_cst_nodes(
    text: &str,
    parsed: &ParseResult,
) -> BTreeMap<String, String> {
    let mut aliases = BTreeMap::new();
    for tokens in css_module_value_statement_tokens_from_cst(text, parsed) {
        collect_css_module_value_path_aliases_from_statement_tokens(&tokens, &mut aliases);
    }
    aliases
}

fn collect_css_module_value_path_aliases_from_statement_tokens(
    tokens: &[Token<'_>],
    aliases: &mut BTreeMap<String, String>,
) {
    let Some(index) = tokens.iter().position(|token| {
        token.kind == SyntaxKind::AtKeyword && token.text.eq_ignore_ascii_case("@value")
    }) else {
        return;
    };

    let start = skip_trivia_tokens(tokens, index + 1, tokens.len());
    let end = css_module_value_statement_end(tokens, start);
    let Some(colon_index) = top_level_token_kind_index(tokens, start, end, SyntaxKind::Colon)
    else {
        return;
    };
    if top_level_token_text_index(tokens, start, end, "from").is_some() {
        return;
    }
    if let Some((name, target)) =
        css_module_value_path_alias_from_tokens(tokens, start, colon_index, end)
    {
        aliases.insert(name, target);
    }
}

fn css_module_value_path_alias_from_tokens(
    tokens: &[Token<'_>],
    start: usize,
    colon_index: usize,
    end: usize,
) -> Option<(String, String)> {
    let name_index = next_non_trivia_token_index_until(tokens, start, colon_index)?;
    let name_token = tokens[name_index];
    if !css_module_value_name_token_can_define(name_token) {
        return None;
    }
    let source_index = next_non_trivia_token_index_until(tokens, colon_index + 1, end)?;
    let source_token = tokens[source_index];
    if !matches!(source_token.kind, SyntaxKind::String | SyntaxKind::Url) {
        return None;
    }
    let source = css_module_value_source_name(source_token);
    css_module_value_source_looks_like_style_request(&source)
        .then(|| (name_token.text.to_string(), source))
}

pub(crate) fn css_module_value_statement_end(tokens: &[Token<'_>], start: usize) -> usize {
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            SyntaxKind::Semicolon
            | SyntaxKind::SassOptionalSemicolon
            | SyntaxKind::LeftBrace
            | SyntaxKind::RightBrace
            | SyntaxKind::SassIndent
            | SyntaxKind::SassDedent
                if paren_depth == 0 && bracket_depth == 0 =>
            {
                return index;
            }
            _ => {}
        }
        index += 1;
    }
    index
}

fn collect_css_module_value_import_facts(
    tokens: &[Token<'_>],
    start: usize,
    from_index: usize,
    end: usize,
    value_path_aliases: &BTreeMap<String, String>,
    values: &mut Vec<ParsedCssModuleValueFact>,
    seen: &mut BTreeSet<(ParsedCssModuleValueFactKind, String, u32, u32)>,
) {
    collect_css_module_value_import_names(tokens, start, from_index, values, seen);
    if let Some((source_name, source_range)) =
        css_module_value_import_edge_source(tokens, from_index + 1, end, value_path_aliases)
    {
        push_css_module_value_fact(
            values,
            seen,
            ParsedCssModuleValueFactKind::ImportSource,
            source_name,
            source_range,
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCssModuleValueImportEdgeFact {
    pub remote_name: String,
    pub local_name: String,
    pub import_source: String,
    pub local_range: TextRange,
    pub remote_range: TextRange,
    pub range: TextRange,
}

#[cfg(feature = "internal-oracle")]
pub(crate) fn collect_css_module_value_import_edge_facts_from_tokens(
    tokens: &[Token<'_>],
) -> Vec<ParsedCssModuleValueImportEdgeFact> {
    css_module_value_import_edge_facts_from_token_view(tokens)
}

pub(crate) fn collect_css_module_value_import_edge_facts_from_cst(
    text: &str,
    parsed: &ParseResult,
) -> Vec<ParsedCssModuleValueImportEdgeFact> {
    css_module_value_import_edge_facts_from_cst_nodes(text, parsed)
}

#[cfg(feature = "internal-oracle")]
fn css_module_value_import_edge_facts_from_token_view(
    tokens: &[Token<'_>],
) -> Vec<ParsedCssModuleValueImportEdgeFact> {
    let mut edges = Vec::new();
    let value_path_aliases = collect_css_module_value_path_aliases_from_tokens(tokens);
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::AtKeyword || !token.text.eq_ignore_ascii_case("@value") {
            continue;
        }

        let start = skip_trivia_tokens(tokens, index + 1, tokens.len());
        let end = css_module_value_statement_end(tokens, start);
        let colon_index = top_level_token_kind_index(tokens, start, end, SyntaxKind::Colon);
        let from_index = top_level_token_text_index(tokens, start, end, "from");
        let Some(from_index) = from_index else {
            continue;
        };
        if colon_index.is_some_and(|colon_index| from_index > colon_index) {
            continue;
        }
        let Some((import_source, _source_range)) =
            css_module_value_import_edge_source(tokens, from_index + 1, end, &value_path_aliases)
        else {
            continue;
        };

        collect_css_module_value_import_edges(tokens, start, from_index, import_source, &mut edges);
    }
    edges
}

fn css_module_value_import_edge_facts_from_cst_nodes(
    text: &str,
    parsed: &ParseResult,
) -> Vec<ParsedCssModuleValueImportEdgeFact> {
    let mut edges = Vec::new();
    let value_path_aliases = collect_css_module_value_path_aliases_from_cst_nodes(text, parsed);
    for tokens in css_module_value_statement_tokens_from_cst(text, parsed) {
        collect_css_module_value_import_edge_statement_facts(
            &tokens,
            &value_path_aliases,
            &mut edges,
        );
    }
    edges
}

fn collect_css_module_value_import_edge_statement_facts(
    tokens: &[Token<'_>],
    value_path_aliases: &BTreeMap<String, String>,
    edges: &mut Vec<ParsedCssModuleValueImportEdgeFact>,
) {
    let Some(index) = tokens.iter().position(|token| {
        token.kind == SyntaxKind::AtKeyword && token.text.eq_ignore_ascii_case("@value")
    }) else {
        return;
    };

    let start = skip_trivia_tokens(tokens, index + 1, tokens.len());
    let end = css_module_value_statement_end(tokens, start);
    let colon_index = top_level_token_kind_index(tokens, start, end, SyntaxKind::Colon);
    let from_index = top_level_token_text_index(tokens, start, end, "from");
    let Some(from_index) = from_index else {
        return;
    };
    if colon_index.is_some_and(|colon_index| from_index > colon_index) {
        return;
    }
    let Some((import_source, _source_range)) =
        css_module_value_import_edge_source(tokens, from_index + 1, end, value_path_aliases)
    else {
        return;
    };

    collect_css_module_value_import_edges(tokens, start, from_index, import_source, edges);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCssModuleValueDefinitionEdgeFact {
    pub definition_name: String,
    pub reference_names: Vec<String>,
    pub range: TextRange,
}

#[cfg(feature = "internal-oracle")]
pub(crate) fn collect_css_module_value_definition_edge_facts_from_tokens(
    tokens: &[Token<'_>],
) -> Vec<ParsedCssModuleValueDefinitionEdgeFact> {
    css_module_value_definition_edge_facts_from_token_view(tokens)
}

pub(crate) fn collect_css_module_value_definition_edge_facts_from_cst(
    text: &str,
    parsed: &ParseResult,
) -> Vec<ParsedCssModuleValueDefinitionEdgeFact> {
    css_module_value_definition_edge_facts_from_cst_nodes(text, parsed)
}

#[cfg(feature = "internal-oracle")]
fn css_module_value_definition_edge_facts_from_token_view(
    tokens: &[Token<'_>],
) -> Vec<ParsedCssModuleValueDefinitionEdgeFact> {
    let mut edges = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::AtKeyword || !token.text.eq_ignore_ascii_case("@value") {
            continue;
        }

        let start = skip_trivia_tokens(tokens, index + 1, tokens.len());
        let end = css_module_value_statement_end(tokens, start);
        let colon_index = top_level_token_kind_index(tokens, start, end, SyntaxKind::Colon);
        let from_index = top_level_token_text_index(tokens, start, end, "from");
        let Some(colon_index) = colon_index else {
            continue;
        };
        if from_index.is_some_and(|from_index| from_index < colon_index) {
            continue;
        }

        let definition_names = collect_css_module_value_definition_edge_names(
            tokens,
            start,
            colon_index,
            |tokens, index| css_module_value_name_token_can_define(tokens[index]),
        );
        let reference_names = collect_css_module_value_definition_edge_names(
            tokens,
            colon_index + 1,
            end,
            css_module_value_reference_token_can_be_name,
        );
        if reference_names.is_empty() {
            continue;
        }
        let range_end = end
            .checked_sub(1)
            .and_then(|end| tokens.get(end))
            .map(|token| token.range.end())
            .unwrap_or_else(|| tokens[index].range.end());

        for definition_name in definition_names {
            edges.push(ParsedCssModuleValueDefinitionEdgeFact {
                definition_name,
                reference_names: reference_names.clone(),
                range: TextRange::new(tokens[index].range.start(), range_end),
            });
        }
    }
    edges
}

fn css_module_value_definition_edge_facts_from_cst_nodes(
    text: &str,
    parsed: &ParseResult,
) -> Vec<ParsedCssModuleValueDefinitionEdgeFact> {
    let mut edges = Vec::new();
    for tokens in css_module_value_statement_tokens_from_cst(text, parsed) {
        collect_css_module_value_definition_edge_statement_facts(&tokens, &mut edges);
    }
    edges
}

fn collect_css_module_value_definition_edge_statement_facts(
    tokens: &[Token<'_>],
    edges: &mut Vec<ParsedCssModuleValueDefinitionEdgeFact>,
) {
    let Some(index) = tokens.iter().position(|token| {
        token.kind == SyntaxKind::AtKeyword && token.text.eq_ignore_ascii_case("@value")
    }) else {
        return;
    };

    let start = skip_trivia_tokens(tokens, index + 1, tokens.len());
    let end = css_module_value_statement_end(tokens, start);
    let colon_index = top_level_token_kind_index(tokens, start, end, SyntaxKind::Colon);
    let from_index = top_level_token_text_index(tokens, start, end, "from");
    let Some(colon_index) = colon_index else {
        return;
    };
    if from_index.is_some_and(|from_index| from_index < colon_index) {
        return;
    }

    let definition_names = collect_css_module_value_definition_edge_names(
        tokens,
        start,
        colon_index,
        |tokens, index| css_module_value_name_token_can_define(tokens[index]),
    );
    let reference_names = collect_css_module_value_definition_edge_names(
        tokens,
        colon_index + 1,
        end,
        css_module_value_reference_token_can_be_name,
    );
    if reference_names.is_empty() {
        return;
    }
    let range_end = end
        .checked_sub(1)
        .and_then(|end| tokens.get(end))
        .map(|token| token.range.end())
        .unwrap_or_else(|| tokens[index].range.end());

    for definition_name in definition_names {
        edges.push(ParsedCssModuleValueDefinitionEdgeFact {
            definition_name,
            reference_names: reference_names.clone(),
            range: TextRange::new(tokens[index].range.start(), range_end),
        });
    }
}

pub(crate) fn collect_css_module_value_definition_edge_names(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    predicate: impl Fn(&[Token<'_>], usize) -> bool,
) -> Vec<String> {
    let mut names = Vec::new();
    let mut index = start;
    while index < end {
        if predicate(tokens, index) && !names.iter().any(|name| name == tokens[index].text) {
            names.push(tokens[index].text.to_string());
        }
        index += 1;
    }
    names
}

fn css_module_value_statement_tokens_from_cst<'text>(
    text: &'text str,
    parsed: &ParseResult,
) -> Vec<Vec<Token<'text>>> {
    parsed
        .syntax()
        .descendants()
        .filter(|node| {
            matches!(
                node.kind(),
                SyntaxKind::CssModuleExportBlock
                    | SyntaxKind::CssModuleImportBlock
                    | SyntaxKind::BogusCssModuleBlock
            )
        })
        .map(|node| tokens_from_syntax_node(text, node))
        .collect()
}

fn css_module_value_reference_declaration_tokens_from_cst<'text>(
    text: &'text str,
    parsed: &ParseResult,
) -> Vec<Vec<Token<'text>>> {
    parsed
        .syntax()
        .descendants()
        .filter(|node| {
            matches!(
                node.kind(),
                SyntaxKind::Declaration | SyntaxKind::CssModuleComposesDeclaration
            )
        })
        .map(|node| tokens_from_syntax_node(text, node))
        .collect()
}

fn css_module_composes_declaration_tokens_from_cst<'text>(
    text: &'text str,
    parsed: &ParseResult,
) -> Vec<Vec<Token<'text>>> {
    parsed
        .syntax()
        .descendants()
        .filter(|node| node.kind() == SyntaxKind::CssModuleComposesDeclaration)
        .map(|node| tokens_from_syntax_node(text, node))
        .collect()
}

fn css_module_value_import_edge_source(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    value_path_aliases: &BTreeMap<String, String>,
) -> Option<(String, TextRange)> {
    let source_index = next_non_trivia_token_index_until(tokens, start, end)?;
    let token = tokens[source_index];
    if matches!(token.kind, SyntaxKind::String | SyntaxKind::Url) {
        return Some((css_module_value_source_name(token), token.range));
    }
    if css_module_value_name_token_can_define(token) {
        return css_module_value_source_alias_target(token.text, token.range, value_path_aliases);
    }
    None
}

fn css_module_value_source_alias_target(
    name: &str,
    range: TextRange,
    value_path_aliases: &BTreeMap<String, String>,
) -> Option<(String, TextRange)> {
    value_path_aliases
        .get(name)
        .map(|source| (source.clone(), range))
}

fn collect_css_module_value_import_edges(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    import_source: String,
    edges: &mut Vec<ParsedCssModuleValueImportEdgeFact>,
) {
    let mut index = start;
    while index < end {
        let token = tokens[index];
        if !css_module_value_name_token_can_define(token) {
            index += 1;
            continue;
        }
        if previous_non_trivia_token_index(tokens, index, start)
            .is_some_and(|previous| tokens[previous].text == "as")
        {
            index += 1;
            continue;
        }
        let remote_name = token.text.to_string();
        let mut local_name = remote_name.clone();
        let mut local_range = token.range;
        if let Some(as_index) = next_non_trivia_token_index_until(tokens, index + 1, end)
            && tokens[as_index].text == "as"
            && let Some(local_index) = next_non_trivia_token_index_until(tokens, as_index + 1, end)
            && css_module_value_name_token_can_define(tokens[local_index])
        {
            local_name = tokens[local_index].text.to_string();
            local_range = tokens[local_index].range;
            index = local_index + 1;
        } else {
            index += 1;
        }
        edges.push(ParsedCssModuleValueImportEdgeFact {
            remote_name,
            local_name,
            import_source: import_source.clone(),
            local_range,
            remote_range: token.range,
            range: token.range,
        });
    }
}

fn collect_css_module_value_import_names(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    values: &mut Vec<ParsedCssModuleValueFact>,
    seen: &mut BTreeSet<(ParsedCssModuleValueFactKind, String, u32, u32)>,
) {
    let mut index = start;
    while index < end {
        let token = tokens[index];
        if css_module_value_name_token_can_define(token) {
            let previous = previous_non_trivia_token_index(tokens, index, start);
            let next = next_non_trivia_token_index_until(tokens, index + 1, end);
            let kind = if previous.is_some_and(|previous| tokens[previous].text == "as") {
                Some(ParsedCssModuleValueFactKind::Definition)
            } else if next.is_some_and(|next| tokens[next].text == "as") {
                Some(ParsedCssModuleValueFactKind::Reference)
            } else {
                Some(ParsedCssModuleValueFactKind::Definition)
            };
            if let Some(kind) = kind {
                push_css_module_value_fact(values, seen, kind, token.text.to_string(), token.range);
            }
        }
        index += 1;
    }
}

fn collect_css_module_value_definition_facts(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    values: &mut Vec<ParsedCssModuleValueFact>,
    seen: &mut BTreeSet<(ParsedCssModuleValueFactKind, String, u32, u32)>,
) {
    let mut index = start;
    while index < end {
        let token = tokens[index];
        if css_module_value_name_token_can_define(token) {
            push_css_module_value_fact(
                values,
                seen,
                ParsedCssModuleValueFactKind::Definition,
                token.text.to_string(),
                token.range,
            );
        }
        index += 1;
    }
}

fn collect_css_module_value_reference_facts(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    values: &mut Vec<ParsedCssModuleValueFact>,
    seen: &mut BTreeSet<(ParsedCssModuleValueFactKind, String, u32, u32)>,
) {
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }
        if paren_depth == 0
            && bracket_depth == 0
            && css_module_value_reference_token_can_be_name(tokens, index)
        {
            push_css_module_value_fact(
                values,
                seen,
                ParsedCssModuleValueFactKind::Reference,
                tokens[index].text.to_string(),
                tokens[index].range,
            );
        }
        index += 1;
    }
}

#[cfg(feature = "internal-oracle")]
fn collect_css_module_value_declaration_reference_facts(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    local_value_names: &BTreeSet<String>,
    values: &mut Vec<ParsedCssModuleValueFact>,
    seen: &mut BTreeSet<(ParsedCssModuleValueFactKind, String, u32, u32)>,
) {
    if local_value_names.is_empty() {
        return;
    }

    let mut index = start;
    while index < end {
        index = skip_trivia_tokens(tokens, index, end);
        if index >= end {
            break;
        }

        if tokens[index].kind == SyntaxKind::AtKeyword {
            let block = find_block_after_header(tokens, index, end);
            if let Some((open, close)) = block {
                if style_wrapper_at_rule(tokens[index].text) {
                    collect_css_module_value_declaration_reference_facts(
                        tokens,
                        open + 1,
                        close,
                        local_value_names,
                        values,
                        seen,
                    );
                }
                index = close + 1;
            } else {
                index = skip_statement(tokens, index, end);
            }
            continue;
        }

        let statement_end = css_module_value_statement_end(tokens, index);
        if statement_end < end && tokens[statement_end].kind == SyntaxKind::LeftBrace {
            if let Some(close) = matching_right_brace(tokens, statement_end, end) {
                collect_css_module_value_declaration_reference_facts(
                    tokens,
                    statement_end + 1,
                    close,
                    local_value_names,
                    values,
                    seen,
                );
                index = close + 1;
            } else {
                index = statement_end + 1;
            }
            continue;
        }

        if let Some(colon_index) = declaration_colon_index(tokens, index, statement_end.min(end)) {
            collect_known_css_module_value_reference_facts(
                tokens,
                colon_index + 1,
                statement_end.min(end),
                local_value_names,
                values,
                seen,
            );
        }

        if statement_end >= end || tokens[statement_end].kind == SyntaxKind::RightBrace {
            break;
        }
        index = statement_end + 1;
    }
}

fn collect_css_module_value_declaration_reference_facts_from_declaration_tokens(
    tokens: &[Token<'_>],
    local_value_names: &BTreeSet<String>,
    values: &mut Vec<ParsedCssModuleValueFact>,
    seen: &mut BTreeSet<(ParsedCssModuleValueFactKind, String, u32, u32)>,
) {
    if local_value_names.is_empty() {
        return;
    }
    if let Some(colon_index) = declaration_colon_index(tokens, 0, tokens.len()) {
        collect_known_css_module_value_reference_facts(
            tokens,
            colon_index + 1,
            tokens.len(),
            local_value_names,
            values,
            seen,
        );
    }
}

pub(crate) fn declaration_colon_index(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Option<usize> {
    let colon_index = top_level_token_kind_index(tokens, start, end, SyntaxKind::Colon)?;
    let property_index = previous_non_trivia_token_index(tokens, colon_index, start)?;
    if !matches!(
        tokens[property_index].kind,
        SyntaxKind::Ident
            | SyntaxKind::CustomPropertyName
            | SyntaxKind::ScssVariable
            | SyntaxKind::LessVariable
            | SyntaxKind::LessPropertyVariableToken
    ) {
        return None;
    }
    let value_index = next_non_trivia_token_index_until(tokens, colon_index + 1, end)?;
    if matches!(
        tokens[value_index].kind,
        SyntaxKind::LeftBrace | SyntaxKind::LeftParen | SyntaxKind::LeftBracket
    ) {
        return None;
    }
    Some(colon_index)
}

fn collect_known_css_module_value_reference_facts(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    local_value_names: &BTreeSet<String>,
    values: &mut Vec<ParsedCssModuleValueFact>,
    seen: &mut BTreeSet<(ParsedCssModuleValueFactKind, String, u32, u32)>,
) {
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }
        if paren_depth == 0
            && bracket_depth == 0
            && css_module_value_reference_token_can_be_name(tokens, index)
            && local_value_names.contains(tokens[index].text)
        {
            push_css_module_value_fact(
                values,
                seen,
                ParsedCssModuleValueFactKind::Reference,
                tokens[index].text.to_string(),
                tokens[index].range,
            );
        }
        index += 1;
    }
}

fn push_css_module_value_fact(
    values: &mut Vec<ParsedCssModuleValueFact>,
    seen: &mut BTreeSet<(ParsedCssModuleValueFactKind, String, u32, u32)>,
    kind: ParsedCssModuleValueFactKind,
    name: String,
    range: TextRange,
) {
    if seen.insert((
        kind,
        name.clone(),
        u32::from(range.start()),
        u32::from(range.end()),
    )) {
        values.push(ParsedCssModuleValueFact { kind, name, range });
    }
}

fn css_module_value_name_token_can_define(token: Token<'_>) -> bool {
    matches!(
        token.kind,
        SyntaxKind::Ident | SyntaxKind::CustomPropertyName
    ) && !matches!(token.text, "as" | "from")
}

pub(crate) fn css_module_value_reference_token_can_be_name(
    tokens: &[Token<'_>],
    index: usize,
) -> bool {
    let token = tokens[index];
    if !matches!(
        token.kind,
        SyntaxKind::Ident | SyntaxKind::CustomPropertyName
    ) {
        return false;
    }
    if let Some(next_index) = next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
        && tokens[next_index].kind == SyntaxKind::LeftParen
    {
        return false;
    }
    !css_module_value_literal_ident_is_not_reference(token.text)
}

fn css_module_value_literal_ident_is_not_reference(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "initial"
            | "inherit"
            | "unset"
            | "revert"
            | "revert-layer"
            | "none"
            | "auto"
            | "normal"
            | "transparent"
            | "currentcolor"
            | "black"
            | "white"
            | "red"
            | "green"
            | "blue"
            | "yellow"
            | "magenta"
            | "cyan"
            | "solid"
            | "dashed"
            | "block"
            | "inline"
            | "flex"
            | "grid"
    )
}

pub(crate) fn css_module_value_source_name(token: Token<'_>) -> String {
    token
        .text
        .trim_matches(|character| character == '"' || character == '\'')
        .to_string()
}

fn css_module_value_source_looks_like_style_request(source: &str) -> bool {
    let lower = source.to_ascii_lowercase();
    (lower.starts_with('/') || lower.starts_with("./") || lower.starts_with("../"))
        && (lower.ends_with(".css")
            || lower.ends_with(".scss")
            || lower.ends_with(".sass")
            || lower.ends_with(".less"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCssModuleComposesFact {
    pub kind: ParsedCssModuleComposesFactKind,
    pub name: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedCssModuleComposesFactKind {
    Target,
    ImportSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCssModuleComposesEdgeFact {
    pub kind: ParsedCssModuleComposesEdgeKind,
    pub owner_selector_names: Vec<String>,
    pub target_names: Vec<String>,
    pub import_source: Option<String>,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedCssModuleComposesEdgeKind {
    Local,
    Global,
    External,
}

#[cfg(feature = "internal-oracle")]
pub(crate) fn collect_css_module_composes_facts_from_tokens(
    tokens: &[Token<'_>],
) -> Vec<ParsedCssModuleComposesFact> {
    css_module_composes_facts_from_token_view(tokens)
}

pub(crate) fn collect_css_module_composes_facts_from_cst(
    text: &str,
    parsed: &ParseResult,
) -> Vec<ParsedCssModuleComposesFact> {
    css_module_composes_facts_from_cst_nodes(text, parsed)
}

#[cfg(feature = "internal-oracle")]
fn css_module_composes_facts_from_token_view(
    tokens: &[Token<'_>],
) -> Vec<ParsedCssModuleComposesFact> {
    let mut composes = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::Ident || !token.text.eq_ignore_ascii_case("composes") {
            continue;
        }
        let Some(colon_index) = next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
        else {
            continue;
        };
        if tokens[colon_index].kind != SyntaxKind::Colon {
            continue;
        }

        let start = colon_index + 1;
        let end = css_module_value_statement_end(tokens, start);
        let from_index = top_level_token_text_index(tokens, start, end, "from");
        let target_end = from_index.unwrap_or(end);
        collect_css_module_composes_targets(tokens, start, target_end, &mut composes, &mut seen);
        if let Some(from_index) = from_index {
            collect_css_module_composes_import_source(
                tokens,
                from_index + 1,
                end,
                &mut composes,
                &mut seen,
            );
        }
    }
    composes
}

fn css_module_composes_facts_from_cst_nodes(
    text: &str,
    parsed: &ParseResult,
) -> Vec<ParsedCssModuleComposesFact> {
    let mut composes = Vec::new();
    let mut seen = BTreeSet::new();
    for tokens in css_module_composes_declaration_tokens_from_cst(text, parsed) {
        collect_css_module_composes_statement_facts(&tokens, &mut composes, &mut seen);
    }
    composes
}

fn collect_css_module_composes_statement_facts(
    tokens: &[Token<'_>],
    composes: &mut Vec<ParsedCssModuleComposesFact>,
    seen: &mut BTreeSet<(ParsedCssModuleComposesFactKind, String, u32, u32)>,
) {
    let Some(index) = tokens.iter().position(|token| {
        token.kind == SyntaxKind::Ident && token.text.eq_ignore_ascii_case("composes")
    }) else {
        return;
    };
    let Some(colon_index) = next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
    else {
        return;
    };
    if tokens[colon_index].kind != SyntaxKind::Colon {
        return;
    }

    let start = colon_index + 1;
    let end = css_module_value_statement_end(tokens, start);
    let from_index = top_level_token_text_index(tokens, start, end, "from");
    let target_end = from_index.unwrap_or(end);
    collect_css_module_composes_targets(tokens, start, target_end, composes, seen);
    if let Some(from_index) = from_index {
        collect_css_module_composes_import_source(tokens, from_index + 1, end, composes, seen);
    }
}

#[cfg(feature = "internal-oracle")]
pub(crate) fn collect_css_module_composes_edge_facts_from_tokens(
    tokens: &[Token<'_>],
) -> Vec<ParsedCssModuleComposesEdgeFact> {
    css_module_composes_edge_facts_from_token_view(tokens)
}

pub(crate) fn collect_css_module_composes_edge_facts_from_cst(
    text: &str,
    parsed: &ParseResult,
) -> Vec<ParsedCssModuleComposesEdgeFact> {
    let mut edges = Vec::new();
    for declaration in parsed
        .syntax()
        .descendants()
        .filter(|node| node.kind() == SyntaxKind::CssModuleComposesDeclaration)
    {
        let owner_branches = css_module_composes_owner_branches_from_cst(text, declaration);
        if owner_branches.is_empty() {
            continue;
        }
        let tokens = tokens_from_syntax_node(text, declaration);
        collect_immediate_css_module_composes_edge_facts(
            &tokens,
            0,
            tokens.len(),
            &owner_branches,
            &mut edges,
        );
    }
    edges
}

#[cfg(feature = "internal-oracle")]
fn css_module_composes_edge_facts_from_token_view(
    tokens: &[Token<'_>],
) -> Vec<ParsedCssModuleComposesEdgeFact> {
    let mut edges = Vec::new();
    collect_css_module_composes_edge_facts_in_range(tokens, 0, tokens.len(), &[], None, &mut edges);
    edges
}

fn css_module_composes_owner_branches_from_cst(
    text: &str,
    declaration: &SyntaxNode<SyntaxKind>,
) -> Vec<SelectorBranch> {
    let mut branches = Vec::new();
    let mut css_module_scope = None;
    let mut ancestors = declaration.ancestors().collect::<Vec<_>>();
    ancestors.reverse();
    for ancestor in ancestors {
        match ancestor.kind() {
            SyntaxKind::Rule | SyntaxKind::NestRule => {
                let tokens = tokens_from_syntax_node(text, ancestor);
                let Some(open) = first_block_open_token_index(&tokens) else {
                    continue;
                };
                let header_start = if ancestor.kind() == SyntaxKind::NestRule {
                    tokens
                        .iter()
                        .position(|token| token.kind == SyntaxKind::AtKeyword)
                        .map_or(0, |index| index + 1)
                } else {
                    0
                };
                let effective_scope = css_module_scope.or_else(|| {
                    css_module_block_scope_marker_in_header(&tokens, header_start, open)
                });
                if effective_scope == Some("global") {
                    branches.clear();
                } else {
                    branches = resolve_selector_header(&tokens, header_start, open, &branches);
                }
                css_module_scope = effective_scope;
            }
            SyntaxKind::CssModuleGlobalBlock => {
                branches.clear();
                css_module_scope = Some("global");
            }
            SyntaxKind::CssModuleLocalBlock if css_module_scope.is_none() => {
                css_module_scope = Some("local");
            }
            _ => {}
        }
    }
    if css_module_scope == Some("global") {
        Vec::new()
    } else {
        branches
    }
}

fn first_block_open_token_index(tokens: &[Token<'_>]) -> Option<usize> {
    tokens
        .iter()
        .position(|token| matches!(token.kind, SyntaxKind::LeftBrace | SyntaxKind::SassIndent))
}

fn collect_css_module_composes_edge_facts_in_range(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    parent_branches: &[SelectorBranch],
    css_module_scope: Option<&'static str>,
    edges: &mut Vec<ParsedCssModuleComposesEdgeFact>,
) {
    let mut index = start;
    while index < end {
        index = skip_trivia_tokens(tokens, index, end);
        if index >= end {
            break;
        }

        if tokens[index].kind == SyntaxKind::AtKeyword {
            let block = find_block_after_header(tokens, index, end);
            if let Some((open, close)) = block {
                if tokens[index].text == "@nest" {
                    if css_module_scope == Some("global") {
                        collect_css_module_composes_edge_facts_in_range(
                            tokens,
                            open + 1,
                            close,
                            &[],
                            css_module_scope,
                            edges,
                        );
                    } else {
                        let branches =
                            resolve_selector_header(tokens, index + 1, open, parent_branches);
                        collect_immediate_css_module_composes_edge_facts(
                            tokens,
                            open + 1,
                            close,
                            &branches,
                            edges,
                        );
                        collect_css_module_composes_edge_facts_in_range(
                            tokens,
                            open + 1,
                            close,
                            &branches,
                            css_module_scope,
                            edges,
                        );
                    }
                } else if style_wrapper_at_rule(tokens[index].text) {
                    collect_css_module_composes_edge_facts_in_range(
                        tokens,
                        open + 1,
                        close,
                        parent_branches,
                        css_module_scope,
                        edges,
                    );
                }
                index = close + 1;
            } else {
                index = skip_statement(tokens, index, end);
            }
            continue;
        }

        let Some((open, close)) = find_block_after_header(tokens, index, end) else {
            index = skip_statement(tokens, index, end);
            continue;
        };

        let effective_scope = css_module_scope
            .or_else(|| css_module_block_scope_marker_in_header(tokens, index, open));
        if effective_scope == Some("global") {
            collect_css_module_composes_edge_facts_in_range(
                tokens,
                open + 1,
                close,
                &[],
                effective_scope,
                edges,
            );
        } else {
            let branches = resolve_selector_header(tokens, index, open, parent_branches);
            collect_immediate_css_module_composes_edge_facts(
                tokens,
                open + 1,
                close,
                &branches,
                edges,
            );
            collect_css_module_composes_edge_facts_in_range(
                tokens,
                open + 1,
                close,
                &branches,
                effective_scope,
                edges,
            );
        }
        index = close + 1;
    }
}

fn collect_immediate_css_module_composes_edge_facts(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    owner_branches: &[SelectorBranch],
    edges: &mut Vec<ParsedCssModuleComposesEdgeFact>,
) {
    let owner_selector_names = sorted_selector_branch_names(owner_branches);
    let mut index = start;
    let mut block_depth = 0usize;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftBrace | SyntaxKind::SassIndent => {
                block_depth += 1;
                index += 1;
                continue;
            }
            SyntaxKind::RightBrace | SyntaxKind::SassDedent => {
                block_depth = block_depth.saturating_sub(1);
                index += 1;
                continue;
            }
            _ => {}
        }
        if block_depth > 0
            || tokens[index].kind != SyntaxKind::Ident
            || !tokens[index].text.eq_ignore_ascii_case("composes")
        {
            index += 1;
            continue;
        }
        let Some(colon_index) = next_non_trivia_token_index_until(tokens, index + 1, end) else {
            index += 1;
            continue;
        };
        if tokens[colon_index].kind != SyntaxKind::Colon {
            index += 1;
            continue;
        }

        let value_start = colon_index + 1;
        let value_end = css_module_value_statement_end(tokens, value_start).min(end);
        let from_index = top_level_token_text_index(tokens, value_start, value_end, "from");
        let target_end = from_index.unwrap_or(value_end);
        let target_names =
            collect_css_module_composes_target_names(tokens, value_start, target_end);
        if target_names.is_empty() {
            index = value_end;
            continue;
        }

        let (kind, import_source) = from_index
            .and_then(|from_index| {
                css_module_composes_import_edge_source(tokens, from_index + 1, value_end)
            })
            .map(|source| {
                if source == "global" {
                    (ParsedCssModuleComposesEdgeKind::Global, Some(source))
                } else {
                    (ParsedCssModuleComposesEdgeKind::External, Some(source))
                }
            })
            .unwrap_or((ParsedCssModuleComposesEdgeKind::Local, None));
        let range_end = value_end
            .checked_sub(1)
            .and_then(|end| tokens.get(end))
            .map(|token| token.range.end())
            .unwrap_or_else(|| tokens[index].range.end());

        edges.push(ParsedCssModuleComposesEdgeFact {
            kind,
            owner_selector_names: owner_selector_names.clone(),
            target_names,
            import_source,
            range: TextRange::new(tokens[index].range.start(), range_end),
        });
        index = value_end;
    }
}

fn sorted_selector_branch_names(branches: &[SelectorBranch]) -> Vec<String> {
    branches
        .iter()
        .map(|branch| branch.name.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn collect_css_module_composes_target_names(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Vec<String> {
    let mut names = Vec::new();
    let mut index = start;
    while index < end {
        if matches!(
            tokens[index].kind,
            SyntaxKind::Ident | SyntaxKind::CustomPropertyName
        ) && !tokens[index].text.eq_ignore_ascii_case("from")
            && !names.iter().any(|name| name == tokens[index].text)
        {
            names.push(tokens[index].text.to_string());
        }
        index += 1;
    }
    names
}

fn css_module_composes_import_edge_source(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Option<String> {
    let source_index = next_non_trivia_token_index_until(tokens, start, end)?;
    let token = tokens[source_index];
    matches!(
        token.kind,
        SyntaxKind::String | SyntaxKind::Url | SyntaxKind::Ident
    )
    .then(|| css_module_value_source_name(token))
}

fn collect_css_module_composes_targets(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    composes: &mut Vec<ParsedCssModuleComposesFact>,
    seen: &mut BTreeSet<(ParsedCssModuleComposesFactKind, String, u32, u32)>,
) {
    let mut index = start;
    while index < end {
        if matches!(
            tokens[index].kind,
            SyntaxKind::Ident | SyntaxKind::CustomPropertyName
        ) && !tokens[index].text.eq_ignore_ascii_case("from")
        {
            push_css_module_composes_fact(
                composes,
                seen,
                ParsedCssModuleComposesFactKind::Target,
                tokens[index].text.to_string(),
                tokens[index].range,
            );
        }
        index += 1;
    }
}

fn collect_css_module_composes_import_source(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    composes: &mut Vec<ParsedCssModuleComposesFact>,
    seen: &mut BTreeSet<(ParsedCssModuleComposesFactKind, String, u32, u32)>,
) {
    if let Some(source_index) = next_non_trivia_token_index_until(tokens, start, end) {
        let token = tokens[source_index];
        if matches!(
            token.kind,
            SyntaxKind::String | SyntaxKind::Url | SyntaxKind::Ident
        ) {
            push_css_module_composes_fact(
                composes,
                seen,
                ParsedCssModuleComposesFactKind::ImportSource,
                css_module_value_source_name(token),
                token.range,
            );
        }
    }
}

fn push_css_module_composes_fact(
    composes: &mut Vec<ParsedCssModuleComposesFact>,
    seen: &mut BTreeSet<(ParsedCssModuleComposesFactKind, String, u32, u32)>,
    kind: ParsedCssModuleComposesFactKind,
    name: String,
    range: TextRange,
) {
    if seen.insert((
        kind,
        name.clone(),
        u32::from(range.start()),
        u32::from(range.end()),
    )) {
        composes.push(ParsedCssModuleComposesFact { kind, name, range });
    }
}
