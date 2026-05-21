use super::*;

#[test]
fn resolves_style_diagnostics_and_code_actions_from_opened_style_documents() {
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
                    "text": ":root { --brand: red; }\n.alert { color: var(--missing); }",
                },
            },
        }),
    );

    let diagnostics_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": STYLE_DIAGNOSTICS_REQUEST,
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
            },
        }),
    );
    assert_eq!(
        diagnostics_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/message")),
        Some(&json!(
            "CSS custom property '--missing' not found in indexed style tokens."
        )),
    );
    assert_eq!(
        diagnostics_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/range")),
        Some(&json!({
            "start": {
                "line": 1,
                "character": 20,
            },
            "end": {
                "line": 1,
                "character": 29,
            },
        })),
    );
    assert_eq!(
        diagnostics_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/data/createCustomProperty/range")),
        Some(&json!({
            "start": {
                "line": 1,
                "character": 33,
            },
            "end": {
                "line": 1,
                "character": 33,
            },
        })),
    );
    assert_eq!(
        diagnostics_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/data/querySeverity")),
        Some(&json!("warning")),
    );
    assert_eq!(
        diagnostics_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/data/provenance/0")),
        Some(&json!("omena-parser.custom-property-facts")),
    );

    let diagnostic = diagnostics_response
        .as_ref()
        .and_then(|value| value.pointer("/result/0"))
        .cloned()
        .unwrap_or(Value::Null);
    let code_action_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/codeAction",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "range": {
                    "start": {
                        "line": 1,
                        "character": 20,
                    },
                    "end": {
                        "line": 1,
                        "character": 29,
                    },
                },
                "context": {
                    "diagnostics": [diagnostic],
                },
            },
        }),
    );
    assert_eq!(
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/title")),
        Some(&json!("Add '--missing' to App.module.scss")),
    );
    assert_eq!(
        code_action_response.as_ref().and_then(|value| {
            value.pointer(
                "/result/0/edit/changes/file:~1~1~1workspace-a~1src~1App.module.scss/0/newText",
            )
        }),
        Some(&json!("\n\n:root {\n  --missing: ;\n}\n")),
    );
    assert_eq!(
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/data/source")),
        Some(&json!("omenaQueryStyleDiagnosticsForFile")),
    );
}

#[test]
fn resolves_graph_aware_sass_diagnostics_from_opened_style_documents() {
    let mut state = LspShellState::default();
    for (uri, text) in [
        (
            "file:///workspace-a/src/App.module.scss",
            "@use \"./tokens\" as tokens;\n.button { color: tokens.$brand; padding: $missing; }",
        ),
        ("file:///workspace-a/src/_tokens.scss", "$brand: red;"),
    ] {
        handle_lsp_message(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": uri,
                        "languageId": "scss",
                        "version": 1,
                        "text": text,
                    },
                },
            }),
        );
    }

    let diagnostics_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": STYLE_DIAGNOSTICS_REQUEST,
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
            },
        }),
    );
    let diagnostics = diagnostics_response
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .expect("style diagnostics response contains an array");
    let missing_sass_messages = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.pointer("/code") == Some(&json!("missingSassSymbol")))
        .map(|diagnostic| {
            diagnostic
                .pointer("/message")
                .and_then(Value::as_str)
                .expect("missing Sass symbol diagnostic has a message")
        })
        .collect::<Vec<_>>();

    assert_eq!(
        missing_sass_messages,
        vec!["Sass variable '$missing' not found in the visible Sass module graph."]
    );
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.pointer("/code") == Some(&json!("missingSassSymbol"))
                && diagnostic.pointer("/data/provenance/1")
                    == Some(&json!("omena-query.graph-aware-sass-diagnostics"))
        }),
        "Rust LSP style diagnostics should consume graph-aware omena-query Sass diagnostics"
    );
}

#[test]
fn resolves_unnecessary_tags_for_cascade_style_diagnostics() -> TestResult {
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
                    "text": ":root { --brand: red; }\n.btn { color: red; color: blue; }",
                },
            },
        }),
    );

    let diagnostics_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": STYLE_DIAGNOSTICS_REQUEST,
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
            },
        }),
    );
    let diagnostics = diagnostics_response
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("style diagnostics result"))?;
    let unreachable = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.pointer("/code") == Some(&json!("unreachableDeclaration")))
        .ok_or_else(|| std::io::Error::other("unreachable declaration diagnostic"))?;
    assert_eq!(unreachable.pointer("/severity"), Some(&json!(4)));
    assert_eq!(unreachable.pointer("/tags"), Some(&json!([1])));
    assert_eq!(
        unreachable.pointer("/data/provenance/0"),
        Some(&json!("omena-checker.cascade-rules")),
    );
    Ok(())
}

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
