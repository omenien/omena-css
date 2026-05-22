use super::*;

#[test]
fn refreshes_open_document_diagnostics_after_initialized_indexing() {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-initialized-diagnostics-{}",
        std::process::id()
    ));
    let src_dir = workspace_root.join("src");
    let style_path = src_dir.join("App.module.scss");
    let _ = std::fs::remove_dir_all(&workspace_root);
    let create_dir_result = std::fs::create_dir_all(&src_dir);
    assert!(
        create_dir_result.is_ok(),
        "create initialized-diagnostics fixture directory: {:?}",
        create_dir_result.err(),
    );
    let write_result = std::fs::write(&style_path, ".known { color: red; }");
    assert!(
        write_result.is_ok(),
        "write initialized-diagnostics style fixture: {:?}",
        write_result.err(),
    );

    let workspace_uri = format!("file://{}", workspace_root.display());
    let source_uri = format!("file://{}/src/App.tsx", workspace_root.display());
    let mut state = LspShellState::default();
    let initialize_outputs = handle_lsp_message_outputs(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "initialized-diagnostics",
                    },
                ],
            },
        }),
    );
    assert_eq!(initialize_outputs.len(), 1);

    let open_outputs = handle_lsp_message_outputs(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                    "languageId": "typescriptreact",
                    "version": 1,
                    "text": "const view = <div className=\"missing\" />;",
                },
            },
        }),
    );
    assert_eq!(
        open_outputs
            .first()
            .and_then(|value| value.pointer("/params/diagnostics")),
        Some(&json!([])),
    );

    let initialized_outputs = handle_lsp_message_outputs(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {},
        }),
    );
    assert_eq!(
        initialized_outputs
            .first()
            .and_then(|value| value.pointer("/params/uri")),
        Some(&json!(source_uri)),
    );
    assert_eq!(
        initialized_outputs
            .first()
            .and_then(|value| value.pointer("/params/diagnostics/0/range")),
        Some(&json!({
            "start": {
                "line": 0,
                "character": 29,
            },
            "end": {
                "line": 0,
                "character": 36,
            },
        })),
    );
    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn dedupes_watched_style_diagnostics_notifications() {
    let workspace_uri = "file:///workspace-dedupe";
    let source_uri = "file:///workspace-dedupe/src/App.tsx";
    let style_uri = "file:///workspace-dedupe/src/App.module.scss";
    let mut state = LspShellState::default();

    let _ = handle_lsp_message_outputs(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "workspace-dedupe",
                    },
                ],
            },
        }),
    );
    let _ = handle_lsp_message_outputs(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": style_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": ".root { color: red; }",
                },
            },
        }),
    );
    let _ = handle_lsp_message_outputs(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                    "languageId": "typescriptreact",
                    "version": 1,
                    "text": "import styles from \"./App.module.scss\";\nconst view = <div className={styles.root} />;",
                },
            },
        }),
    );

    let outputs = handle_lsp_message_outputs(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeWatchedFiles",
            "params": {
                "changes": [
                    {
                        "uri": style_uri,
                        "type": 2,
                    },
                    {
                        "uri": style_uri,
                        "type": 2,
                    },
                ],
            },
        }),
    );
    let published_uris = outputs
        .iter()
        .filter_map(|value| value.pointer("/params/uri").and_then(Value::as_str))
        .collect::<Vec<_>>();

    assert_eq!(published_uris, vec![style_uri, source_uri]);
}
