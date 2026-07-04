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

#[cfg(feature = "salsa-style-diagnostics")]
#[test]
fn style_text_edit_republishes_only_dependent_open_source_documents() {
    let workspace_uri = "file:///workspace-source-republish";
    let style_uri = "file:///workspace-source-republish/src/Widget.module.scss";
    let related_source_uri = "file:///workspace-source-republish/src/App.tsx";
    let source_uri = "file:///workspace-source-republish/src/Unrelated.tsx";
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
                        "name": "source-republish",
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
                    "text": "const view = <section />;",
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
                    "uri": related_source_uri,
                    "languageId": "typescriptreact",
                    "version": 1,
                    "text": "import styles from \"./Widget.module.scss\";\nconst view = <section className={styles.root} />;",
                },
            },
        }),
    );

    let unrelated_before = resolve_source_diagnostics_for_uri(&state, source_uri);
    crate::diagnostics_scheduler::reset_source_change_republish_fanout_for_test();
    let outputs = handle_lsp_message_outputs(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {
                    "uri": style_uri,
                    "version": 2,
                },
                "contentChanges": [
                    {
                        "text": ".other { color: blue; }",
                    },
                ],
            },
        }),
    );
    let published_uris = outputs
        .iter()
        .filter_map(|value| value.pointer("/params/uri").and_then(Value::as_str))
        .collect::<Vec<_>>();

    assert!(
        published_uris.contains(&related_source_uri),
        "dependent open source document must be republished: {published_uris:?}"
    );
    assert!(
        !published_uris.contains(&source_uri),
        "unrelated open source document must not be republished: {published_uris:?}"
    );
    assert_eq!(
        crate::diagnostics_scheduler::read_source_change_republish_fanout_for_test(),
        1
    );

    let published_by_uri = published_diagnostics_by_uri(outputs.as_slice());
    let expected_related_diagnostics =
        resolve_source_diagnostics_for_uri(&state, related_source_uri);
    let expected_related_outputs =
        crate::diagnostics_scheduler::publish_tiered_diagnostics_notifications(
            &mut state,
            related_source_uri,
            expected_related_diagnostics,
        );
    assert_eq!(
        published_by_uri.get(related_source_uri),
        published_diagnostics_by_scheduled_output(expected_related_outputs.as_slice())
            .get(related_source_uri)
    );
    assert_eq!(
        unrelated_before,
        resolve_source_diagnostics_for_uri(&state, source_uri),
        "skipped unrelated source diagnostics must stay byte-identical to a fresh recompute"
    );
}

#[test]
fn watched_style_change_fails_open_when_dependency_scope_is_unavailable() {
    let workspace_uri = "file:///workspace-source-republish-fail-open";
    let style_uri = "file:///workspace-source-republish-fail-open/src/Widget.module.scss";
    let source_uri = "file:///workspace-source-republish-fail-open/src/Unrelated.tsx";
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
                        "name": "source-republish-fail-open",
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
                    "uri": source_uri,
                    "languageId": "typescriptreact",
                    "version": 1,
                    "text": "const view = <section />;",
                },
            },
        }),
    );

    crate::diagnostics_scheduler::reset_source_change_republish_fanout_for_test();
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
                ],
            },
        }),
    );
    let published_uris = outputs
        .iter()
        .filter_map(|value| value.pointer("/params/uri").and_then(Value::as_str))
        .collect::<Vec<_>>();

    assert!(
        published_uris.contains(&source_uri),
        "source documents must still republish when dependency scope is unavailable: {published_uris:?}"
    );
    assert_eq!(
        crate::diagnostics_scheduler::read_source_change_republish_fanout_for_test(),
        1
    );
}

#[cfg(feature = "salsa-style-diagnostics")]
#[test]
fn style_text_edit_republishes_transitive_source_dependents() {
    let workspace_uri = "file:///workspace-source-republish-transitive";
    let base_uri = "file:///workspace-source-republish-transitive/src/Base.module.scss";
    let mid_uri = "file:///workspace-source-republish-transitive/src/Mid.module.scss";
    let source_uri = "file:///workspace-source-republish-transitive/src/App.tsx";
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
                        "name": "source-republish-transitive",
                    },
                ],
            },
        }),
    );
    for (uri, text) in [
        (base_uri, ".base { color: red; }"),
        (
            mid_uri,
            ".mid { composes: base from \"./Base.module.scss\"; color: blue; }",
        ),
    ] {
        let _ = handle_lsp_message_outputs(
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
                    "text": "import styles from \"./Mid.module.scss\";\nconst view = <section className={styles.mid} />;",
                },
            },
        }),
    );

    crate::diagnostics_scheduler::reset_source_change_republish_fanout_for_test();
    let outputs = handle_lsp_message_outputs(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {
                    "uri": base_uri,
                    "version": 2,
                },
                "contentChanges": [
                    {
                        "text": ".base { color: green; }",
                    },
                ],
            },
        }),
    );
    let published_uris = outputs
        .iter()
        .filter_map(|value| value.pointer("/params/uri").and_then(Value::as_str))
        .collect::<Vec<_>>();

    assert!(
        published_uris.contains(&source_uri),
        "transitive source dependent must be republished: {published_uris:?}"
    );
    assert_eq!(
        crate::diagnostics_scheduler::read_source_change_republish_fanout_for_test(),
        1
    );
}

#[cfg(feature = "salsa-style-diagnostics")]
fn published_diagnostics_by_uri(outputs: &[Value]) -> std::collections::BTreeMap<String, Value> {
    outputs
        .iter()
        .filter(|value| value.get("method") == Some(&json!("textDocument/publishDiagnostics")))
        .filter_map(|value| {
            let uri = value.pointer("/params/uri").and_then(Value::as_str)?;
            let diagnostics = value.pointer("/params/diagnostics")?.clone();
            Some((uri.to_string(), diagnostics))
        })
        .collect()
}

#[cfg(feature = "salsa-style-diagnostics")]
fn published_diagnostics_by_scheduled_output(
    outputs: &[crate::ScheduledLspOutput],
) -> std::collections::BTreeMap<String, Value> {
    outputs
        .iter()
        .map(|output| &output.value)
        .filter(|value| value.get("method") == Some(&json!("textDocument/publishDiagnostics")))
        .filter_map(|value| {
            let uri = value.pointer("/params/uri").and_then(Value::as_str)?;
            let diagnostics = value.pointer("/params/diagnostics")?.clone();
            Some((uri.to_string(), diagnostics))
        })
        .collect()
}

// rfcs#61 FIX-1: a watched change to a style file must also republish diagnostics for
// OPEN style documents that (transitively) import it — resolved over disk, so an
// intermediate partial that is neither open nor indexed does not break the importer
// chain. An unrelated open style document must NOT be republished.
#[test]
fn refreshes_open_style_importer_after_watched_transitive_dependency_change() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-style-peer-refresh-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let consumer_path = workspace_path.join("src/App.module.scss");
    let unrelated_path = workspace_path.join("src/Other.module.scss");
    let mid_path = workspace_path.join("src/partials/_mid.scss");
    let leaf_path = workspace_path.join("src/partials/_leaf.scss");
    fs::create_dir_all(fixture_parent(
        consumer_path.as_path(),
        "consumer fixture path has parent directory",
    )?)?;
    fs::create_dir_all(fixture_parent(
        mid_path.as_path(),
        "partials fixture path has parent directory",
    )?)?;
    let consumer_text = "@use \"./partials/mid\";\n.app { color: red; }\n";
    let unrelated_text = ".other { color: blue; }\n";
    fs::write(consumer_path.as_path(), consumer_text)?;
    fs::write(unrelated_path.as_path(), unrelated_text)?;
    fs::write(mid_path.as_path(), "@use \"./leaf\";\n")?;
    fs::write(leaf_path.as_path(), "$tone: red;\n")?;

    let workspace_uri = path_to_file_uri(workspace_path.as_path());
    let consumer_uri = path_to_file_uri(consumer_path.as_path());
    let unrelated_uri = path_to_file_uri(unrelated_path.as_path());
    let leaf_uri = path_to_file_uri(leaf_path.as_path());

    let mut state = LspShellState::default();
    let _ = handle_lsp_message_outputs(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    { "uri": workspace_uri, "name": "style-peer-refresh" },
                ],
            },
        }),
    );
    for (uri, text) in [
        (consumer_uri.as_str(), consumer_text),
        (unrelated_uri.as_str(), unrelated_text),
    ] {
        let _ = handle_lsp_message_outputs(
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

    let outputs = handle_lsp_message_outputs(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeWatchedFiles",
            "params": {
                "changes": [
                    { "uri": leaf_uri, "type": 2 },
                ],
            },
        }),
    );
    let published_uris = outputs
        .iter()
        .filter_map(|value| value.pointer("/params/uri").and_then(Value::as_str))
        .collect::<Vec<_>>();

    assert!(
        published_uris.contains(&consumer_uri.as_str()),
        "open importer must be republished after its transitive dependency changes: {published_uris:?}"
    );
    assert!(
        !published_uris.contains(&unrelated_uri.as_str()),
        "unrelated open style document must not be republished: {published_uris:?}"
    );

    let _ = fs::remove_dir_all(workspace_path.as_path());
    Ok(())
}
