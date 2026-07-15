//! Parser facts for ICSS import/export blocks.
//!
//! ICSS facts preserve the raw local names and specifier spans needed by
//! resolver and CSS Modules consumers.

use cstree::text::TextRange;
use omena_syntax::SyntaxKind;
use std::collections::BTreeSet;

use crate::{
    ParseResult, Token, collect_css_module_value_definition_edge_names,
    css_module_value_reference_token_can_be_name, css_module_value_source_name,
    css_module_value_statement_end, find_block_after_header, matches_ignore_ascii_case,
    next_non_trivia_token_index_until,
};

use super::tokens_from_syntax_node;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedIcssFact {
    pub kind: ParsedIcssFactKind,
    pub name: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedIcssFactKind {
    ExportName,
    ImportLocalName,
    ImportRemoteName,
    ImportSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedIcssImportEdgeFact {
    pub local_name: String,
    pub remote_name: String,
    pub import_source: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedIcssExportEdgeFact {
    pub export_name: String,
    pub reference_names: Vec<String>,
    pub range: TextRange,
}

pub fn collect_icss_export_values_from_cst(
    text: &str,
    parsed: &ParseResult,
) -> Vec<(String, String)> {
    let mut values = Vec::new();
    for tokens in icss_block_tokens_from_cst(text, parsed) {
        collect_icss_export_values_from_block_tokens(&tokens, &mut values);
    }
    values
}

pub(crate) fn collect_icss_facts_from_cst(text: &str, parsed: &ParseResult) -> Vec<ParsedIcssFact> {
    let mut icss = Vec::new();
    let mut seen = BTreeSet::new();
    for tokens in icss_block_tokens_from_cst(text, parsed) {
        collect_icss_facts_from_block_tokens(&tokens, &mut icss, &mut seen);
    }
    icss
}

fn collect_icss_facts_from_block_tokens(
    tokens: &[Token<'_>],
    icss: &mut Vec<ParsedIcssFact>,
    seen: &mut BTreeSet<(ParsedIcssFactKind, String, u32, u32)>,
) {
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::Colon {
            continue;
        }
        let Some(name_index) = next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
        else {
            continue;
        };
        let name = tokens[name_index].text;
        if !matches!(tokens[name_index].kind, SyntaxKind::Ident) {
            continue;
        }
        if matches_ignore_ascii_case(name, &["export"]) {
            if let Some((open, close)) =
                find_block_after_header(tokens, name_index + 1, tokens.len())
            {
                collect_icss_export_names(tokens, open + 1, close, icss, seen);
            }
            continue;
        }
        if matches_ignore_ascii_case(name, &["import"]) {
            collect_icss_import_source(tokens, name_index + 1, icss, seen);
            if let Some((open, close)) =
                find_block_after_header(tokens, name_index + 1, tokens.len())
            {
                collect_icss_import_names(tokens, open + 1, close, icss, seen);
            }
        }
    }
}

pub(crate) fn collect_icss_import_edge_facts_from_cst(
    text: &str,
    parsed: &ParseResult,
) -> Vec<ParsedIcssImportEdgeFact> {
    let mut edges = Vec::new();
    for tokens in icss_block_tokens_from_cst(text, parsed) {
        collect_icss_import_edge_facts_from_block_tokens(&tokens, &mut edges);
    }
    edges
}

fn collect_icss_import_edge_facts_from_block_tokens(
    tokens: &[Token<'_>],
    edges: &mut Vec<ParsedIcssImportEdgeFact>,
) {
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::Colon {
            continue;
        }
        let Some(name_index) = next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
        else {
            continue;
        };
        if tokens[name_index].kind != SyntaxKind::Ident
            || !matches_ignore_ascii_case(tokens[name_index].text, &["import"])
        {
            continue;
        }
        let Some(import_source) = icss_import_edge_source(tokens, name_index + 1) else {
            continue;
        };
        if let Some((open, close)) = find_block_after_header(tokens, name_index + 1, tokens.len()) {
            collect_icss_import_edges(tokens, open + 1, close, import_source, edges);
        }
    }
}

pub(crate) fn collect_icss_export_edge_facts_from_cst(
    text: &str,
    parsed: &ParseResult,
) -> Vec<ParsedIcssExportEdgeFact> {
    let mut edges = Vec::new();
    for tokens in icss_block_tokens_from_cst(text, parsed) {
        collect_icss_export_edge_facts_from_block_tokens(&tokens, &mut edges);
    }
    edges
}

fn collect_icss_export_edge_facts_from_block_tokens(
    tokens: &[Token<'_>],
    edges: &mut Vec<ParsedIcssExportEdgeFact>,
) {
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::Colon {
            continue;
        }
        let Some(name_index) = next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
        else {
            continue;
        };
        if tokens[name_index].kind != SyntaxKind::Ident
            || !matches_ignore_ascii_case(tokens[name_index].text, &["export"])
        {
            continue;
        }
        if let Some((open, close)) = find_block_after_header(tokens, name_index + 1, tokens.len()) {
            collect_icss_export_edges(tokens, open + 1, close, edges);
        }
    }
}

fn icss_block_tokens_from_cst<'text>(
    text: &'text str,
    parsed: &ParseResult,
) -> Vec<Vec<Token<'text>>> {
    parsed
        .syntax()
        .descendants()
        .filter(|node| {
            matches!(
                node.kind(),
                SyntaxKind::CssModuleExportBlock | SyntaxKind::CssModuleImportBlock
            )
        })
        .map(|node| tokens_from_syntax_node(text, parsed, node))
        .collect()
}

fn collect_icss_export_edges(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    edges: &mut Vec<ParsedIcssExportEdgeFact>,
) {
    let mut index = start;
    while index < end {
        let token = tokens[index];
        if matches!(
            token.kind,
            SyntaxKind::Ident | SyntaxKind::CustomPropertyName
        ) && let Some(colon_index) = next_non_trivia_token_index_until(tokens, index + 1, end)
            && tokens[colon_index].kind == SyntaxKind::Colon
        {
            let value_end = css_module_value_statement_end(tokens, colon_index + 1).min(end);
            let reference_names = collect_css_module_value_definition_edge_names(
                tokens,
                colon_index + 1,
                value_end,
                css_module_value_reference_token_can_be_name,
            );
            if !reference_names.is_empty() {
                let range_end = value_end
                    .checked_sub(1)
                    .and_then(|end| tokens.get(end))
                    .map(|token| token.range.end())
                    .unwrap_or_else(|| token.range.end());
                edges.push(ParsedIcssExportEdgeFact {
                    export_name: token.text.to_string(),
                    reference_names,
                    range: TextRange::new(token.range.start(), range_end),
                });
            }
            index = value_end;
            continue;
        }
        index += 1;
    }
}

fn collect_icss_export_values_from_block_tokens(
    tokens: &[Token<'_>],
    values: &mut Vec<(String, String)>,
) {
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::Colon {
            continue;
        }
        let Some(name_index) = next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
        else {
            continue;
        };
        if tokens[name_index].kind != SyntaxKind::Ident
            || !matches_ignore_ascii_case(tokens[name_index].text, &["export"])
        {
            continue;
        }
        let Some((open, close)) = find_block_after_header(tokens, name_index + 1, tokens.len())
        else {
            continue;
        };
        let mut declaration_index = open + 1;
        while declaration_index < close {
            let declaration = tokens[declaration_index];
            if matches!(
                declaration.kind,
                SyntaxKind::Ident | SyntaxKind::CustomPropertyName
            ) && let Some(colon_index) =
                next_non_trivia_token_index_until(tokens, declaration_index + 1, close)
                && tokens[colon_index].kind == SyntaxKind::Colon
            {
                let value_end = css_module_value_statement_end(tokens, colon_index + 1).min(close);
                let value = tokens[colon_index + 1..value_end]
                    .iter()
                    .map(|token| token.text)
                    .collect::<String>()
                    .trim()
                    .to_string();
                values.push((declaration.text.to_string(), value));
                declaration_index = value_end;
                continue;
            }
            declaration_index += 1;
        }
    }
}

fn icss_import_edge_source(tokens: &[Token<'_>], start: usize) -> Option<String> {
    let open_index = next_non_trivia_token_index_until(tokens, start, tokens.len())?;
    if tokens[open_index].kind != SyntaxKind::LeftParen {
        return None;
    }
    let source_index = next_non_trivia_token_index_until(tokens, open_index + 1, tokens.len())?;
    let token = tokens[source_index];
    matches!(
        token.kind,
        SyntaxKind::String | SyntaxKind::Url | SyntaxKind::Ident
    )
    .then(|| css_module_value_source_name(token))
}

fn collect_icss_import_edges(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    import_source: String,
    edges: &mut Vec<ParsedIcssImportEdgeFact>,
) {
    let mut index = start;
    while index < end {
        let token = tokens[index];
        if matches!(
            token.kind,
            SyntaxKind::Ident | SyntaxKind::CustomPropertyName
        ) && let Some(colon_index) = next_non_trivia_token_index_until(tokens, index + 1, end)
            && tokens[colon_index].kind == SyntaxKind::Colon
            && let Some(remote_index) =
                next_non_trivia_token_index_until(tokens, colon_index + 1, end)
            && matches!(
                tokens[remote_index].kind,
                SyntaxKind::Ident | SyntaxKind::CustomPropertyName
            )
        {
            edges.push(ParsedIcssImportEdgeFact {
                local_name: token.text.to_string(),
                remote_name: tokens[remote_index].text.to_string(),
                import_source: import_source.clone(),
                range: token.range,
            });
            index = css_module_value_statement_end(tokens, colon_index + 1);
            continue;
        }
        index += 1;
    }
}

fn collect_icss_export_names(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    icss: &mut Vec<ParsedIcssFact>,
    seen: &mut BTreeSet<(ParsedIcssFactKind, String, u32, u32)>,
) {
    let mut index = start;
    while index < end {
        let token = tokens[index];
        if matches!(
            token.kind,
            SyntaxKind::Ident | SyntaxKind::CustomPropertyName
        ) && let Some(colon_index) = next_non_trivia_token_index_until(tokens, index + 1, end)
            && tokens[colon_index].kind == SyntaxKind::Colon
        {
            push_icss_fact(
                icss,
                seen,
                ParsedIcssFactKind::ExportName,
                token.text.to_string(),
                token.range,
            );
            index = css_module_value_statement_end(tokens, colon_index + 1);
            continue;
        }
        index += 1;
    }
}

fn collect_icss_import_source(
    tokens: &[Token<'_>],
    start: usize,
    icss: &mut Vec<ParsedIcssFact>,
    seen: &mut BTreeSet<(ParsedIcssFactKind, String, u32, u32)>,
) {
    let Some(open_index) = next_non_trivia_token_index_until(tokens, start, tokens.len()) else {
        return;
    };
    if tokens[open_index].kind != SyntaxKind::LeftParen {
        return;
    }
    let Some(source_index) =
        next_non_trivia_token_index_until(tokens, open_index + 1, tokens.len())
    else {
        return;
    };
    let token = tokens[source_index];
    if matches!(
        token.kind,
        SyntaxKind::String | SyntaxKind::Url | SyntaxKind::Ident
    ) {
        push_icss_fact(
            icss,
            seen,
            ParsedIcssFactKind::ImportSource,
            css_module_value_source_name(token),
            token.range,
        );
    }
}

fn collect_icss_import_names(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    icss: &mut Vec<ParsedIcssFact>,
    seen: &mut BTreeSet<(ParsedIcssFactKind, String, u32, u32)>,
) {
    let mut index = start;
    while index < end {
        let token = tokens[index];
        if matches!(
            token.kind,
            SyntaxKind::Ident | SyntaxKind::CustomPropertyName
        ) && let Some(colon_index) = next_non_trivia_token_index_until(tokens, index + 1, end)
            && tokens[colon_index].kind == SyntaxKind::Colon
        {
            push_icss_fact(
                icss,
                seen,
                ParsedIcssFactKind::ImportLocalName,
                token.text.to_string(),
                token.range,
            );
            if let Some(remote_index) =
                next_non_trivia_token_index_until(tokens, colon_index + 1, end)
                && matches!(
                    tokens[remote_index].kind,
                    SyntaxKind::Ident | SyntaxKind::CustomPropertyName
                )
            {
                push_icss_fact(
                    icss,
                    seen,
                    ParsedIcssFactKind::ImportRemoteName,
                    tokens[remote_index].text.to_string(),
                    tokens[remote_index].range,
                );
            }
            index = css_module_value_statement_end(tokens, colon_index + 1);
            continue;
        }
        index += 1;
    }
}

fn push_icss_fact(
    icss: &mut Vec<ParsedIcssFact>,
    seen: &mut BTreeSet<(ParsedIcssFactKind, String, u32, u32)>,
    kind: ParsedIcssFactKind,
    name: String,
    range: TextRange,
) {
    if seen.insert((
        kind,
        name.clone(),
        u32::from(range.start()),
        u32::from(range.end()),
    )) {
        icss.push(ParsedIcssFact { kind, name, range });
    }
}
