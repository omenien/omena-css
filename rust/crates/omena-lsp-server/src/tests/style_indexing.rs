use super::*;

#[test]
fn indexes_style_documents_on_open_and_change() {
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                    "languageId": "scss",
                    "version": 1,
                    "text": ".root { color: var(--brand); } :root { --brand: red; }",
                },
            },
        }),
    );
    let summary = state
        .document("file:///workspace-a/src/App.module.scss")
        .and_then(|document| document.style_summary.as_ref());
    assert_eq!(
        summary.map(|summary| summary.selector_names.clone()),
        Some(vec!["root".to_string()]),
    );
    assert_eq!(
        summary.map(|summary| summary.custom_property_decl_names.clone()),
        Some(vec!["--brand".to_string()]),
    );
    assert_eq!(
        summary.map(|summary| summary.custom_property_ref_names.clone()),
        Some(vec!["--brand".to_string()]),
    );

    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                    "version": 2,
                },
                "contentChanges": [
                    {
                        "text": ".card { --gap: 4px; }",
                    },
                ],
            },
        }),
    );
    let updated = state
        .document("file:///workspace-a/src/App.module.scss")
        .and_then(|document| document.style_summary.as_ref());
    assert_eq!(
        updated.map(|summary| summary.selector_names.clone()),
        Some(vec!["card".to_string()]),
    );
    assert_eq!(
        updated.map(|summary| summary.custom_property_decl_names.clone()),
        Some(vec!["--gap".to_string()]),
    );

    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                    "version": 3,
                },
                "contentChanges": [
                    {
                        "range": {
                            "start": { "line": 0, "character": 1 },
                            "end": { "line": 0, "character": 5 },
                        },
                        "text": "panel",
                    },
                ],
            },
        }),
    );
    let incrementally_updated = state
        .document("file:///workspace-a/src/App.module.scss")
        .and_then(|document| document.style_summary.as_ref());
    assert_eq!(
        incrementally_updated.map(|summary| summary.selector_names.clone()),
        Some(vec!["panel".to_string()]),
    );
}

#[test]
fn keeps_style_summary_cache_style_document_only() {
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.tsx",
                    "languageId": "typescriptreact",
                    "version": 1,
                    "text": "const tone = 'red';",
                },
            },
        }),
    );

    let source_document_cache_state =
        state
            .document("file:///workspace-a/src/App.tsx")
            .map(|document| {
                (
                    document.style_summary.is_none(),
                    document.style_candidates.is_empty(),
                )
            });
    assert_eq!(source_document_cache_state, Some((true, true)));
}
