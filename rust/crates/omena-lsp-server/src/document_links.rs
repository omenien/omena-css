//! textDocument/documentLink for style documents: every dependency
//! specifier (`@use` / `@forward` / `@import`, `composes: … from`,
//! `@value … from`, ICSS `:import`) becomes a clickable link to the file
//! the RESOLVER — the same one navigation uses — says it is.
//!
//! Link ranges come from exact quoted occurrences of the specifier string
//! in the document text: the parser facts carry the specifier VALUES but no
//! spans, and a link's only semantic content is its target — the resolver
//! stays the single authority for that. An unresolvable specifier yields no
//! link (never a broken one). Runs on the dispatched query lane because
//! resolution may probe disk.

use crate::{LspShellState, protocol::parser_position_for_byte_offset};
use serde_json::{Value, json};

pub(crate) fn resolve_lsp_document_links(state: &LspShellState, params: Option<&Value>) -> Value {
    let document_uri = params
        .and_then(|params| params.pointer("/textDocument/uri"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let Some(document) = state.document(document_uri) else {
        return json!([]);
    };
    if document.style_summary.is_none() {
        return json!([]);
    }
    let facts = omena_query::summarize_omena_query_omena_parser_style_facts(
        document.text.as_str(),
        crate::query_style_dialect_for_uri(document.uri.as_str()),
    );
    let mut specifiers: Vec<&str> = Vec::new();
    specifiers.extend(facts.sass_module_use_sources.iter().map(String::as_str));
    specifiers.extend(facts.sass_module_forward_sources.iter().map(String::as_str));
    specifiers.extend(facts.sass_module_import_sources.iter().map(String::as_str));
    specifiers.extend(
        facts
            .css_module_composes_import_sources
            .iter()
            .map(String::as_str),
    );
    specifiers.extend(
        facts
            .css_module_value_import_sources
            .iter()
            .map(String::as_str),
    );
    specifiers.extend(facts.icss_import_sources.iter().map(String::as_str));
    specifiers.sort_unstable();
    specifiers.dedup();

    let mut links = Vec::new();
    for specifier in specifiers {
        if specifier.is_empty() {
            continue;
        }
        let Some(target) = crate::resolve_lsp_style_uri_for_specifier(state, document, specifier)
        else {
            continue;
        };
        for range in quoted_occurrence_ranges(document.text.as_str(), specifier) {
            links.push(json!({
                "range": range,
                "target": target,
            }));
        }
    }
    json!(links)
}

/// Ranges of the specifier text INSIDE its quotes, for every exact quoted
/// occurrence (`"spec"` or `'spec'`) whose line reads as a dependency
/// statement (`@use`/`@forward`/`@import`, `composes … from`, `@value …
/// from`, ICSS `:import`) BEFORE the quote — the same string inside a
/// comment, `content:`, or `url()` must not become a link (review
/// finding). Still lexical on purpose: the parser facts carry no spans,
/// and the resolver stays the only semantic authority.
fn quoted_occurrence_ranges(text: &str, specifier: &str) -> Vec<Value> {
    let mut ranges = Vec::new();
    for quote in ['"', '\''] {
        let needle = format!("{quote}{specifier}{quote}");
        let mut cursor = 0usize;
        while let Some(relative) = text[cursor..].find(needle.as_str()) {
            let quote_offset = cursor + relative;
            let start = quote_offset + quote.len_utf8();
            let end = start + specifier.len();
            cursor = end;
            if !dependency_statement_precedes(text, quote_offset) {
                continue;
            }
            ranges.push(json!({
                "start": parser_position_for_byte_offset(text, start),
                "end": parser_position_for_byte_offset(text, end),
            }));
        }
    }
    ranges
}

fn dependency_statement_precedes(text: &str, quote_offset: usize) -> bool {
    let line_start = text[..quote_offset]
        .rfind('\n')
        .map_or(0, |index| index + 1);
    let prefix = &text[line_start..quote_offset];
    ["@use", "@forward", "@import", "from", ":import"]
        .iter()
        .any(|token| prefix.contains(token))
}
