//! workspace/symbol: name-search over the corpus's style DECLARATIONS —
//! class selectors, Sass variables/mixins/functions, custom properties —
//! read exclusively from the precomputed per-document candidates, so a
//! query never builds an index or touches disk. Cmd+T for "where is this
//! class/token defined?".

use crate::{LspDocumentOrigin, LspQueryReadView, protocol::is_style_document_uri};
use serde_json::{Value, json};

/// Bounded so a one-letter query cannot flood the client; VS Code refines
/// the query as the user types.
const MAX_WORKSPACE_SYMBOLS: usize = 256;

pub(crate) fn resolve_lsp_workspace_symbols(
    state: &dyn LspQueryReadView,
    params: Option<&Value>,
) -> Value {
    let query = params
        .and_then(|params| params.pointer("/query"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_lowercase();
    let mut symbols = Vec::new();
    'documents: for document in state.query_documents().values() {
        // LOCAL declarations only: the corpus also admits foreign
        // (node_modules) stylesheets for resolution, and surfacing their
        // internals would drown the user's own symbols (review finding).
        if document.origin != LspDocumentOrigin::Local
            || !is_style_document_uri(document.uri.as_str())
        {
            continue;
        }
        for candidate in &document.style_candidates {
            let Some((display_name, symbol_kind)) =
                workspace_symbol_shape(candidate.kind, candidate.name.as_str())
            else {
                continue;
            };
            if !query.is_empty() && !display_name.to_lowercase().contains(query.as_str()) {
                continue;
            }
            symbols.push(json!({
                "name": display_name,
                "kind": symbol_kind,
                "location": {
                    "uri": document.uri,
                    "range": candidate.range,
                },
                "containerName": document
                    .uri
                    .rsplit('/')
                    .next()
                    .unwrap_or(document.uri.as_str()),
            }));
            if symbols.len() >= MAX_WORKSPACE_SYMBOLS {
                break 'documents;
            }
        }
    }
    symbols.sort_by(|left, right| {
        left.pointer("/name")
            .and_then(Value::as_str)
            .cmp(&right.pointer("/name").and_then(Value::as_str))
    });
    json!(symbols)
}

/// DECLARATION candidate kinds only — references would flood the list and
/// belong to textDocument/references. LSP SymbolKind: Class=5, Method=6,
/// Property=7, Function=12, Variable=13.
fn workspace_symbol_shape(candidate_kind: &str, name: &str) -> Option<(String, u8)> {
    match candidate_kind {
        "selector" => Some((format!(".{name}"), 5)),
        "sassVariableDeclaration" => Some((format!("${name}"), 13)),
        "sassMixinDeclaration" => Some((format!("@mixin {name}"), 6)),
        "sassFunctionDeclaration" => Some((format!("@function {name}"), 12)),
        "sassSymbolDeclaration" => Some((format!("${name}"), 13)),
        "customPropertyDeclaration" => Some((name.to_string(), 7)),
        _ => None,
    }
}
