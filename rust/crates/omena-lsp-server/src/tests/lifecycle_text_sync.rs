use super::*;

#[test]
fn tracks_text_document_lifecycle_notifications() {
    let mut state = LspShellState::default();

    assert!(
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
                        "text": "const tone = 'blue';",
                    },
                },
            }),
        )
        .is_none()
    );
    assert_eq!(state.document_count(), 1);
    assert_eq!(
        state
            .document("file:///workspace-a/src/App.tsx")
            .map(|document| document.text.as_str()),
        Some("const tone = 'blue';"),
    );
    assert_eq!(
        state
            .document("file:///workspace-a/src/App.tsx")
            .and_then(|document| document.workspace_folder_uri.as_deref()),
        None,
    );

    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.tsx",
                    "version": 2,
                },
                "contentChanges": [
                    {
                        "text": "const tone = 'red';",
                    },
                ],
            },
        }),
    );
    let document = state.document("file:///workspace-a/src/App.tsx");
    assert_eq!(document.map(|document| document.version), Some(2));
    assert_eq!(
        document.map(|document| document.text.as_str()),
        Some("const tone = 'red';"),
    );

    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.tsx",
                    "version": 3,
                },
                "contentChanges": [
                    {
                        "range": {
                            "start": { "line": 0, "character": 14 },
                            "end": { "line": 0, "character": 17 },
                        },
                        "text": "green",
                    },
                ],
            },
        }),
    );
    let document = state.document("file:///workspace-a/src/App.tsx");
    assert_eq!(document.map(|document| document.version), Some(3));
    assert_eq!(
        document.map(|document| document.text.as_str()),
        Some("const tone = 'green';"),
    );

    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didClose",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.tsx",
                },
            },
        }),
    );
    assert_eq!(state.document_count(), 0);
}
