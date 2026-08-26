use super::*;
use std::path::PathBuf;

#[cfg(all(feature = "test-support", feature = "salsa-style-diagnostics"))]
#[test]
fn style_diagnostics_streaming_does_not_recompute_without_committed_summary() -> TestResult {
    let app_uri = "file:///workspace-a/src/App.module.scss";
    let app_source = r#".app { composes: base from "./Base.module.scss"; }"#;
    let inputs = crate::style_diagnostics_snapshot::LspStyleDiagnosticsRenderInputsV0 {
        document_uri: app_uri,
        document_text: app_source,
        query_candidates: &[],
        snapshot_id: None,
        deep_analysis: false,
        configured_severity: 1,
    };

    omena_query::reset_workspace_cross_file_summary_direct_recompute_count_for_test();
    let diagnostics = crate::style_diagnostics::finish_style_diagnostics_value(&inputs, None, None);
    let diagnostic_items = diagnostics
        .as_array()
        .ok_or_else(|| std::io::Error::other("style diagnostics should render an array"))?;
    assert!(
        diagnostic_items.iter().all(|diagnostic| {
            diagnostic.pointer("/code") != Some(&json!("crossFileStreamingReachability"))
        }),
        "streaming reachability must require a committed graph summary: {diagnostics:?}",
    );
    assert_eq!(
        omena_query::read_workspace_cross_file_summary_direct_recompute_count_for_test(),
        0,
        "LSP streaming diagnostics must not fall back to direct workspace summary recompute",
    );
    Ok(())
}

#[cfg(all(feature = "test-support", feature = "salsa-style-diagnostics"))]
#[test]
fn style_diagnostics_render_reports_workspace_snapshot_id() -> TestResult {
    let snapshot_id = omena_query::OmenaWorkspaceSnapshotIdV0::from_revision(
        omena_incremental::IncrementalRevisionV0 { value: 7 },
    );
    let inputs = crate::style_diagnostics_snapshot::LspStyleDiagnosticsRenderInputsV0 {
        document_uri: "file:///workspace-a/src/App.module.scss",
        document_text: ".app { color: red; }",
        query_candidates: &[],
        snapshot_id: Some(snapshot_id),
        deep_analysis: false,
        configured_severity: 1,
    };
    let summary = omena_query::OmenaQueryStyleDiagnosticsForFileV0 {
        schema_version: "0",
        product: "omena-query.diagnostics-for-file",
        file_uri: inputs.document_uri.to_string(),
        file_kind: "style",
        diagnostic_count: 1,
        diagnostics: vec![omena_query::OmenaQueryStyleDiagnosticV0 {
            code: "fixtureDiagnostic",
            severity: "warning",
            provenance: vec!["fixture"],
            range: omena_query::ParserRangeV0::default(),
            message: "fixture diagnostic".to_string(),
            tags: Vec::new(),
            create_custom_property: None,
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
        }],
        ready_surfaces: vec!["fixtureDiagnostics"],
        suppression_summary: None,
    };
    let diagnostics =
        crate::style_diagnostics::finish_style_diagnostics_value(&inputs, Some(summary), None);
    assert_eq!(
        diagnostics.pointer("/0/data/snapshotId/value"),
        Some(&json!(7))
    );
    Ok(())
}

#[cfg(all(feature = "test-support", feature = "salsa-style-diagnostics"))]
#[test]
fn style_diagnostics_streaming_reads_committed_cross_file_summary() -> TestResult {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-committed-cross-file-summary-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&workspace_root);
    std::fs::create_dir_all(workspace_root.join("src").as_path())?;

    let workspace_uri = crate::protocol::path_to_file_uri(workspace_root.as_path());
    let app_uri = format!("{workspace_uri}/src/App.module.scss");
    let base_uri = format!("{workspace_uri}/src/Base.module.scss");
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "committed-cross-file-summary",
                    },
                ],
            },
        }),
    );
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": base_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": ".base { color: red; }",
                },
            },
        }),
    );
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": app_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": ".app { composes: base from \"./Base.module.scss\"; }",
                },
            },
        }),
    );

    omena_query::reset_workspace_cross_file_summary_direct_recompute_count_for_test();
    omena_query::reset_sass_module_resolution_direct_recompute_count_for_test();
    omena_query::reset_committed_style_semantic_graph_compute_count_for_test();
    let diagnostics = resolve_style_diagnostics_for_uri(&state, app_uri.as_str());
    assert!(diagnostics.as_array().is_some());
    let repeated_diagnostics = resolve_style_diagnostics_for_uri(&state, app_uri.as_str());
    assert!(repeated_diagnostics.as_array().is_some());
    assert_eq!(
        omena_query::read_committed_style_semantic_graph_compute_count_for_test(),
        1,
        "style diagnostics should reuse the committed graph across unchanged selector reads",
    );
    assert_eq!(
        omena_query::read_workspace_cross_file_summary_direct_recompute_count_for_test(),
        0,
        "style diagnostics should read the committed selector summary instead of calling the direct workspace summary API",
    );
    assert_eq!(
        omena_query::read_sass_module_resolution_direct_recompute_count_for_test(),
        0,
        "style diagnostics should read committed Sass resolution instead of the direct workspace API",
    );

    let _ = std::fs::remove_dir_all(&workspace_root);
    Ok(())
}

#[cfg(all(feature = "test-support", feature = "salsa-style-diagnostics"))]
#[test]
fn deferred_style_diagnostics_streaming_reads_committed_cross_file_summary() -> TestResult {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-deferred-committed-cross-file-summary-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&workspace_root);
    std::fs::create_dir_all(workspace_root.join("src").as_path())?;

    let workspace_uri = crate::protocol::path_to_file_uri(workspace_root.as_path());
    let app_uri = format!("{workspace_uri}/src/App.module.scss");
    let base_uri = format!("{workspace_uri}/src/Base.module.scss");
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "deferred-committed-cross-file-summary",
                    },
                ],
            },
        }),
    );
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": base_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": ".base { color: red; }",
                },
            },
        }),
    );
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": app_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": ".app { composes: base from \"./Base.module.scss\"; }",
                },
            },
        }),
    );

    let effects = crate::diagnostics_scheduler::run_diagnostics_schedule_effects(
        &mut state,
        crate::diagnostics_scheduler::DiagnosticsScheduleEvent::TextDocument {
            uri: app_uri.clone(),
            is_close: false,
            content_changed: false,
        },
    );
    assert_eq!(
        effects.deferred_diagnostics.len(),
        1,
        "style diagnostics should schedule one deferred full diagnostics dispatch",
    );

    omena_query::reset_workspace_cross_file_summary_direct_recompute_count_for_test();
    omena_query::reset_sass_module_resolution_direct_recompute_count_for_test();
    omena_query::reset_committed_style_semantic_graph_compute_count_for_test();
    let mut host = omena_query::OmenaQueryStyleMemoHostV0::new();
    let notification = crate::resolve_deferred_diagnostics_notification(
        &mut host,
        effects
            .deferred_diagnostics
            .first()
            .ok_or_else(|| std::io::Error::other("missing deferred diagnostics dispatch"))?,
    );
    assert_eq!(
        notification.get("method"),
        Some(&json!("textDocument/publishDiagnostics")),
        "deferred full diagnostics should render as a publishDiagnostics notification",
    );
    assert_eq!(
        omena_query::read_committed_style_semantic_graph_compute_count_for_test(),
        1,
        "deferred diagnostics should compute one committed graph for the selector",
    );
    assert_eq!(
        omena_query::read_workspace_cross_file_summary_direct_recompute_count_for_test(),
        0,
        "deferred diagnostics should pass the committed selector summary into streaming analysis",
    );
    assert_eq!(
        omena_query::read_sass_module_resolution_direct_recompute_count_for_test(),
        0,
        "deferred diagnostics should read committed Sass resolution instead of the direct workspace API",
    );

    let _ = std::fs::remove_dir_all(&workspace_root);
    Ok(())
}

#[test]
fn indexes_watched_style_file_changes_from_disk() {
    let workspace_root =
        std::env::temp_dir().join(format!("omena-lsp-server-watched-{}", std::process::id()));
    let src_dir = workspace_root.join("src");
    let style_path = src_dir.join("App.module.scss");
    let _ = std::fs::remove_dir_all(&workspace_root);
    let create_dir_result = std::fs::create_dir_all(&src_dir);
    assert!(
        create_dir_result.is_ok(),
        "create watched fixture directory: {:?}",
        create_dir_result.err(),
    );
    let write_result = std::fs::write(&style_path, ".fromDisk { color: red; }");
    assert!(
        write_result.is_ok(),
        "write watched style fixture: {:?}",
        write_result.err(),
    );

    let workspace_uri = format!("file://{}", workspace_root.display());
    let style_uri = format!("file://{}", style_path.display());
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "watched",
                    },
                ],
            },
        }),
    );
    handle_lsp_message(
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

    let indexed = state
        .document(style_uri.as_str())
        .and_then(|document| document.style_summary.as_ref());
    assert_eq!(
        indexed.map(|summary| summary.selector_names.clone()),
        Some(vec!["fromDisk".to_string()]),
    );
    assert_eq!(state.snapshot().watched_file_event_count, 1);

    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": style_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": ".openBuffer { color: blue; }",
                },
            },
        }),
    );
    let write_while_open_result = std::fs::write(&style_path, ".diskUpdate { color: green; }");
    assert!(
        write_while_open_result.is_ok(),
        "write watched open-buffer fixture: {:?}",
        write_while_open_result.err(),
    );
    handle_lsp_message(
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
    let open_buffer = state
        .document(style_uri.as_str())
        .and_then(|document| document.style_summary.as_ref());
    assert_eq!(
        open_buffer.map(|summary| summary.selector_names.clone()),
        Some(vec!["openBuffer".to_string()]),
    );

    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didClose",
            "params": {
                "textDocument": {
                    "uri": style_uri,
                },
            },
        }),
    );
    let reloaded_after_close = state
        .document(style_uri.as_str())
        .and_then(|document| document.style_summary.as_ref());
    assert_eq!(
        reloaded_after_close.map(|summary| summary.selector_names.clone()),
        Some(vec!["diskUpdate".to_string()]),
    );

    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeWatchedFiles",
            "params": {
                "changes": [
                    {
                        "uri": style_uri,
                        "type": 3,
                    },
                ],
            },
        }),
    );
    assert!(state.document(style_uri.as_str()).is_none());
    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn defers_workspace_style_file_index_until_initialized_notification() {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-initial-index-{}",
        std::process::id()
    ));
    let src_dir = workspace_root.join("src");
    let style_path = src_dir.join("Initial.module.scss");
    let _ = std::fs::remove_dir_all(&workspace_root);
    let create_dir_result = std::fs::create_dir_all(&src_dir);
    assert!(
        create_dir_result.is_ok(),
        "create initial-index fixture directory: {:?}",
        create_dir_result.err(),
    );
    let write_result = std::fs::write(&style_path, ".initial { color: red; }");
    assert!(
        write_result.is_ok(),
        "write initial-index style fixture: {:?}",
        write_result.err(),
    );

    let workspace_uri = format!("file://{}", workspace_root.display());
    let style_uri = format!("file://{}", style_path.display());
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "initial-index",
                    },
                ],
            },
        }),
    );

    let not_indexed_yet = state
        .document(style_uri.as_str())
        .and_then(|document| document.style_summary.as_ref());
    assert!(not_indexed_yet.is_none());

    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {},
        }),
    );

    let indexed = state
        .document(style_uri.as_str())
        .and_then(|document| document.style_summary.as_ref());
    assert_eq!(
        indexed.map(|summary| summary.selector_names.clone()),
        Some(vec!["initial".to_string()]),
    );
    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn indexes_workspace_source_files_from_disk() -> TestResult {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-source-index-{}",
        std::process::id()
    ));
    let src_dir = workspace_root.join("src");
    let source_path = src_dir.join("App.tsx");
    let style_path = src_dir.join("Button.module.scss");
    let _ = std::fs::remove_dir_all(&workspace_root);
    std::fs::create_dir_all(&src_dir)?;
    std::fs::write(&style_path, ".root { color: red; }")?;
    std::fs::write(
        &source_path,
        "import styles from \"./Button.module.scss\";\nconst view = <div className={styles.root} />;",
    )?;

    let workspace_uri = crate::protocol::path_to_file_uri(workspace_root.as_path());
    let source_uri = crate::protocol::path_to_file_uri(source_path.as_path());
    let style_uri = crate::protocol::path_to_file_uri(style_path.as_path());
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "source-index",
                    },
                ],
            },
        }),
    );
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {},
        }),
    );

    let source_document = state
        .document(source_uri.as_str())
        .ok_or_else(|| std::io::Error::other("source document should be indexed from disk"))?;
    assert!(
        !state.has_open_document_uri(source_uri.as_str()),
        "disk-indexed source documents must not be treated as open buffers"
    );
    assert_eq!(source_document.language_id, "typescriptreact");
    let imported_style_bindings = &source_document.source_syntax_index.imported_style_bindings;
    assert_eq!(imported_style_bindings.len(), 1);
    assert_eq!(imported_style_bindings[0].binding, "styles");
    assert!(
        file_uri_equivalent(
            imported_style_bindings[0].style_uri.as_str(),
            style_uri.as_str()
        ),
        "indexed source binding should target the imported CSS module: {imported_style_bindings:?}"
    );
    assert!(
        source_document
            .source_syntax_index
            .style_property_accesses
            .iter()
            .any(|access| {
                source_document
                    .text
                    .get(access.byte_span.start..access.byte_span.end)
                    == Some("root")
                    && access
                        .target_style_uri
                        .as_deref()
                        .is_some_and(|target| file_uri_equivalent(target, style_uri.as_str()))
            }),
        "disk-indexed source syntax should resolve CSS Module property access to the imported target"
    );
    let _ = std::fs::remove_dir_all(&workspace_root);
    Ok(())
}

#[test]
fn style_open_refreshes_source_syntax_index_for_source_first_order() -> TestResult {
    let workspace_uri = "file:///tmp/cme-rust-lsp-source-first-refresh";
    let source_uri = format!("{workspace_uri}/src/App.tsx");
    let style_uri = format!("{workspace_uri}/src/App.module.scss");
    let source_text = "import styles from \"./App.module.scss\";\nconst view = styles.root;\n";
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "source-first-refresh",
                    },
                ],
            },
        }),
    );
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                    "languageId": "typescriptreact",
                    "version": 1,
                    "text": source_text,
                },
            },
        }),
    );
    handle_lsp_message(
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

    let source_document = state
        .document(source_uri.as_str())
        .ok_or_else(|| std::io::Error::other("source document should stay indexed"))?;
    assert!(
        source_document
            .source_syntax_index
            .imported_style_bindings
            .iter()
            .any(|binding| {
                binding.binding == "styles"
                    && file_uri_equivalent(binding.style_uri.as_str(), style_uri.as_str())
            }),
        "style open should refresh source import bindings: {:?}",
        source_document.source_syntax_index.imported_style_bindings
    );
    assert!(
        source_document
            .source_syntax_index
            .style_property_accesses
            .iter()
            .any(|access| {
                source_document
                    .text
                    .get(access.byte_span.start..access.byte_span.end)
                    == Some("root")
                    && access
                        .target_style_uri
                        .as_deref()
                        .is_some_and(|target| file_uri_equivalent(target, style_uri.as_str()))
            }),
        "style open should refresh source property access targets: {:?}",
        source_document.source_syntax_index.style_property_accesses
    );
    Ok(())
}

#[test]
fn style_diagnostics_reuse_source_syntax_index_after_source_first_open() -> TestResult {
    let workspace_uri = "file:///tmp/cme-rust-lsp-source-first-diagnostics";
    let source_uri = format!("{workspace_uri}/src/App.tsx");
    let style_uri = format!("{workspace_uri}/src/App.module.scss");
    let source_text = "import styles from \"./App.module.scss\";\nconst view = <div className={styles.root} />;\nconst bracket = styles[\"theme\"];\n";
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "source-first-diagnostics",
                    },
                ],
            },
        }),
    );
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                    "languageId": "typescriptreact",
                    "version": 1,
                    "text": source_text,
                },
            },
        }),
    );
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": style_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": ".root { color: red; }\n.theme { color: blue; }\n.alert { color: green; }",
                },
            },
        }),
    );

    let source_document = state
        .document(source_uri.as_str())
        .ok_or_else(|| std::io::Error::other("source document should stay indexed"))?;
    for expected_selector in ["root", "theme"] {
        assert!(
            source_document
                .source_syntax_index
                .style_property_accesses
                .iter()
                .any(|access| {
                    source_document
                        .text
                        .get(access.byte_span.start..access.byte_span.end)
                        == Some(expected_selector)
                        && access
                            .target_style_uri
                            .as_deref()
                            .is_some_and(|target| file_uri_equivalent(target, style_uri.as_str()))
                }),
            "source index should target {expected_selector}: {:?}",
            source_document.source_syntax_index.style_property_accesses
        );
    }

    let style_sources =
        style_sources_from_open_documents(&state, Some(workspace_uri), Some(style_uri.as_str()));
    let source_documents = source_documents_from_open_documents(&state, Some(workspace_uri));
    assert_eq!(
        source_documents.len(),
        1,
        "workspace-compatible source document should be forwarded into style diagnostics"
    );
    assert!(
        source_documents[0]
            .source_syntax_index
            .as_ref()
            .is_some_and(|index| index.style_property_accesses.iter().any(|access| {
                source_documents[0]
                    .source_source
                    .get(access.byte_span.start..access.byte_span.end)
                    == Some("root")
                    && access
                        .target_style_uri
                        .as_deref()
                        .is_some_and(|target| file_uri_equivalent(target, style_uri.as_str()))
            })),
        "forwarded source document should retain target-aware source syntax index: {:?}",
        source_documents[0].source_syntax_index
    );
    let direct_summary = omena_query::summarize_omena_query_style_diagnostics_for_workspace_file(
        style_uri.as_str(),
        style_sources.as_slice(),
        source_documents.as_slice(),
        &[],
        None,
    )
    .ok_or_else(|| std::io::Error::other("direct workspace style diagnostics should resolve"))?;
    let direct_unused_messages = direct_summary
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "unusedSelector")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();
    assert!(
        !direct_unused_messages
            .iter()
            .any(|message| message.contains("'.root'")),
        "direct source index usage should mark .root as referenced: {direct_unused_messages:?}"
    );

    let diagnostics = resolve_style_diagnostics_for_uri(&state, style_uri.as_str());
    let empty = Vec::new();
    let unused_messages = diagnostics
        .as_array()
        .unwrap_or(&empty)
        .iter()
        .filter(|diagnostic| diagnostic.get("code") == Some(&json!("unusedSelector")))
        .filter_map(|diagnostic| diagnostic.get("message").and_then(Value::as_str))
        .collect::<Vec<_>>();
    assert!(
        !unused_messages
            .iter()
            .any(|message| message.contains("'.root'")),
        "source-first index reuse should mark .root as referenced: {unused_messages:?}"
    );
    assert!(
        !unused_messages
            .iter()
            .any(|message| message.contains("'.theme'")),
        "source-first index reuse should mark .theme as referenced: {unused_messages:?}"
    );
    assert!(
        unused_messages
            .iter()
            .any(|message| message.contains("'.alert'")),
        "style diagnostics should still report genuinely unused selectors: {unused_messages:?}"
    );
    Ok(())
}

#[test]
fn scheduled_initialized_indexes_workspace_sources_on_background_result() -> TestResult {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-background-source-index-{}",
        std::process::id()
    ));
    let src_dir = workspace_root.join("src");
    let source_path = src_dir.join("App.tsx");
    let style_path = src_dir.join("Button.module.scss");
    let _ = std::fs::remove_dir_all(&workspace_root);
    std::fs::create_dir_all(&src_dir)?;
    std::fs::write(&style_path, ".root { color: red; }")?;
    std::fs::write(
        &source_path,
        "import styles from \"./Button.module.scss\";\nconst view = <div className={styles.root} />;",
    )?;

    let workspace_uri = format!("file://{}", workspace_root.display());
    let source_uri = format!("file://{}", source_path.display());
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "background-source-index",
                    },
                ],
            },
        }),
    );

    let turn = handle_lsp_message_scheduled_outputs_or_dispatch(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {},
        }),
    );
    let workspace_index_jobs = match turn {
        LspLoopTurnV0::OutputsAndDeferredDiagnostics {
            workspace_index_jobs,
            ..
        } => workspace_index_jobs,
        other => {
            return Err(std::io::Error::other(format!(
                "initialized should schedule background workspace indexing: {other:?}"
            ))
            .into());
        }
    };
    assert_eq!(workspace_index_jobs.len(), 1);
    assert!(
        state.document(source_uri.as_str()).is_none(),
        "stdio scheduled path must not index source documents on the loop turn"
    );

    let result = collect_background_workspace_index(
        workspace_index_jobs
            .into_iter()
            .next()
            .ok_or_else(|| std::io::Error::other("missing workspace index job"))?,
    );
    apply_background_workspace_index_result(&mut state, result);

    let source_document = state
        .document(source_uri.as_str())
        .ok_or_else(|| std::io::Error::other("background result should index source document"))?;
    assert_eq!(source_document.language_id, "typescriptreact");
    assert!(
        !state.has_open_document_uri(source_uri.as_str()),
        "background-indexed source documents must not become open buffers"
    );

    let _ = std::fs::remove_dir_all(&workspace_root);
    Ok(())
}

#[test]
fn background_indexed_source_files_feed_references_and_drop_stale_results() -> TestResult {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-background-source-occurrences-{}",
        std::process::id()
    ));
    let src_dir = workspace_root.join("src");
    let source_path = src_dir.join("App.tsx");
    let style_path = src_dir.join("Button.module.scss");
    let _ = std::fs::remove_dir_all(&workspace_root);
    std::fs::create_dir_all(&src_dir)?;
    std::fs::write(&style_path, ".root { color: red; }")?;
    std::fs::write(
        &source_path,
        "import styles from \"./Button.module.scss\";\nconst view = <div className={styles.root} />;",
    )?;

    let workspace_uri = crate::protocol::path_to_file_uri(workspace_root.as_path());
    let source_uri = crate::protocol::path_to_file_uri(source_path.as_path());
    let style_uri = crate::protocol::path_to_file_uri(style_path.as_path());
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "background-source-occurrences",
                    },
                ],
            },
        }),
    );

    let first_turn = handle_lsp_message_scheduled_outputs_or_dispatch(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {},
        }),
    );
    let first_job = match first_turn {
        LspLoopTurnV0::OutputsAndDeferredDiagnostics {
            mut workspace_index_jobs,
            ..
        } => workspace_index_jobs
            .pop()
            .ok_or_else(|| std::io::Error::other("missing first workspace index job"))?,
        other => {
            return Err(std::io::Error::other(format!(
                "initialized should schedule first background workspace indexing job: {other:?}"
            ))
            .into());
        }
    };
    let stale_result = collect_background_workspace_index(first_job);

    let second_turn = handle_lsp_message_scheduled_outputs_or_dispatch(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {},
        }),
    );
    let second_job = match second_turn {
        LspLoopTurnV0::OutputsAndDeferredDiagnostics {
            mut workspace_index_jobs,
            ..
        } => workspace_index_jobs
            .pop()
            .ok_or_else(|| std::io::Error::other("missing second workspace index job"))?,
        other => {
            return Err(std::io::Error::other(format!(
                "initialized should schedule second background workspace indexing job: {other:?}"
            ))
            .into());
        }
    };

    apply_background_workspace_index_result(&mut state, stale_result);
    assert!(
        state.document(source_uri.as_str()).is_none(),
        "stale background index results must not repopulate the document map"
    );
    let fresh_result = collect_background_workspace_index(second_job);
    apply_background_workspace_index_result(&mut state, fresh_result);

    let references_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/references",
            "params": {
                "textDocument": {
                    "uri": style_uri,
                },
                "position": {
                    "line": 0,
                    "character": 2,
                },
                "context": {
                    "includeDeclaration": false,
                },
            },
        }),
    );
    let reference_locations = references_response
        .as_ref()
        .and_then(|response| response.pointer("/result"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("references response should contain locations"))?;
    assert!(
        reference_locations.iter().any(|location| location
            .get("uri")
            .and_then(Value::as_str)
            .is_some_and(|uri| file_uri_equivalent(uri, source_uri.as_str()))),
        "background-indexed source occurrence should appear in references: {references_response:?}"
    );

    let _ = std::fs::remove_dir_all(&workspace_root);
    Ok(())
}

#[test]
fn background_source_index_uses_persisted_source_syntax_sidecar() -> TestResult {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-source-syntax-sidecar-{}",
        std::process::id()
    ));
    let src_dir = workspace_root.join("src");
    let source_path = src_dir.join("App.tsx");
    let style_path = src_dir.join("Button.module.scss");
    let _ = std::fs::remove_dir_all(&workspace_root);
    std::fs::create_dir_all(&src_dir)?;
    std::fs::write(&style_path, ".cachedRoot { color: red; }")?;
    let source_text = "import styles from \"./Button.module.scss\";\nconst view = <div />;";
    std::fs::write(&source_path, source_text)?;

    let workspace_uri = crate::protocol::path_to_file_uri(workspace_root.as_path());
    let source_uri = crate::protocol::path_to_file_uri(source_path.as_path());
    let style_uri = crate::protocol::path_to_file_uri(style_path.as_path());
    let resolution_inputs =
        load_lsp_workspace_style_resolution_inputs(Some(workspace_uri.as_str()), &[]);
    let selector_start = source_text
        .find("styles")
        .ok_or_else(|| std::io::Error::other("fixture should contain styles binding"))?;
    let selector_span = ParserByteSpanV0 {
        start: selector_start,
        end: selector_start + "styles".len(),
    };
    let skipped_reasons = [
        "identifierPathAwaitingTypeProvider",
        "lexicallyResolvedExpression",
        "unsupportedCallExpression",
        "unsupportedArithmeticExpression",
        "unsupportedLogicalExpression",
        "unsupportedComputedMemberExpression",
        "unsupportedNestedTemplateExpression",
        "unsupportedMultipleTemplateInterpolations",
        "unsupportedExpressionShape",
    ];
    let cached_index = SourceSyntaxIndex {
        schema_version: "0",
        product: "omena-bridge.source-syntax-index",
        imported_style_bindings: vec![ImportedStyleBinding {
            binding: "styles".to_string(),
            style_uri: style_uri.clone(),
        }],
        class_string_literals: Vec::new(),
        style_property_accesses: vec![omena_query::OmenaQuerySourceStylePropertyAccessFactV0 {
            byte_span: selector_span,
            target_style_uri: Some(style_uri.clone()),
        }],
        inline_style_declarations: vec![
            omena_query::OmenaQuerySourceInlineStyleDeclarationFactV0 {
                byte_span: selector_span,
                value_byte_span: Some(selector_span),
                property_name: omena_syntax::ident::AuthoredPropertyTextV0::new("color"),
                value: Some("red".to_string()),
                target_style_uri: Some(style_uri.clone()),
                cascade_tier: "authorInlineStyle",
                important: false,
                static_value: true,
            },
        ],
        selector_references: vec![SourceSelectorReferenceFact {
            byte_span: selector_span,
            selector_name: Some("cachedRoot".to_string()),
            match_kind: SourceSelectorReferenceMatchKind::Exact,
            target_style_uri: Some(style_uri.clone()),
            surface: SourceSelectorReferenceSurface::OmenaTsgoTypeFactProjection,
        }],
        type_fact_targets: Vec::new(),
        type_fact_target_skipped: skipped_reasons
            .iter()
            .enumerate()
            .map(
                |(index, reason)| omena_query::OmenaQuerySourceTypeFactTargetSkippedFactV0 {
                    byte_span: selector_span,
                    expression_id: format!("fixture-type-fact-target-{index}"),
                    target_style_uri: Some(style_uri.clone()),
                    reason,
                },
            )
            .collect(),
        type_fact_target_skipped_count: skipped_reasons.len(),
        type_fact_provider_unavailable: Vec::new(),
        class_value_universes: vec![omena_query::OmenaQuerySourceClassValueUniverseEntryV0 {
            plugin_id: "cva-recipe-domain",
            domain: "cva-recipe",
            owner_name: "buttonRecipe".to_string(),
            class_names: vec!["button_primary".to_string()],
            axes: vec![omena_query::OmenaQuerySourceClassValueUniverseAxisV0 {
                axis_name: "intent".to_string(),
                values: vec!["primary".to_string()],
            }],
            patterns: Vec::new(),
            unresolved: Vec::new(),
            byte_span: selector_span,
        }],
        domain_class_references: vec![omena_query::OmenaQuerySourceDomainClassReferenceFactV0 {
            byte_span: selector_span,
            plugin_id: "cva-recipe-domain",
            domain: "cva-recipe",
            owner_name: "buttonRecipe".to_string(),
            axis_name: "intent".to_string(),
            option_name: Some("primary".to_string()),
            prefix: None,
        }],
        source_elements: vec![
            omena_query::OmenaQuerySourceElementFactV0 {
                identity: omena_query::OmenaQuerySourceElementIdentityFactV0 {
                    source_path: source_uri.clone(),
                    byte_span: ParserByteSpanV0 {
                        start: selector_span.start,
                        end: selector_span.start + 1,
                    },
                },
                intrinsic_tag_name: Some("main".to_string()),
                static_class_names: vec!["scope-root".to_string()],
                classes_are_exact: true,
            },
            omena_query::OmenaQuerySourceElementFactV0 {
                identity: omena_query::OmenaQuerySourceElementIdentityFactV0 {
                    source_path: source_uri.clone(),
                    byte_span: selector_span,
                },
                intrinsic_tag_name: Some("span".to_string()),
                static_class_names: Vec::new(),
                classes_are_exact: true,
            },
        ],
        element_parent_edges: vec![omena_query::OmenaQuerySourceElementParentFactV0 {
            child: omena_query::OmenaQuerySourceElementIdentityFactV0 {
                source_path: source_uri.clone(),
                byte_span: selector_span,
            },
            parent: omena_query::OmenaQuerySourceElementIdentityFactV0 {
                source_path: source_uri.clone(),
                byte_span: ParserByteSpanV0 {
                    start: selector_span.start,
                    end: selector_span.start + 1,
                },
            },
        }],
    };
    let source_type_fact_attempts = [
        (
            omena_query::OmenaQuerySourceTypeFactExpressionShapeV0::IdentifierPath,
            omena_query::OmenaQuerySourceTypeFactLexicalDispositionV0::TypeProviderCandidate,
        ),
        (
            omena_query::OmenaQuerySourceTypeFactExpressionShapeV0::LexicallyEnumerable,
            omena_query::OmenaQuerySourceTypeFactLexicalDispositionV0::Resolved,
        ),
        (
            omena_query::OmenaQuerySourceTypeFactExpressionShapeV0::Call,
            omena_query::OmenaQuerySourceTypeFactLexicalDispositionV0::Unresolved,
        ),
        (
            omena_query::OmenaQuerySourceTypeFactExpressionShapeV0::Arithmetic,
            omena_query::OmenaQuerySourceTypeFactLexicalDispositionV0::Unresolved,
        ),
        (
            omena_query::OmenaQuerySourceTypeFactExpressionShapeV0::LogicalOperator,
            omena_query::OmenaQuerySourceTypeFactLexicalDispositionV0::Unresolved,
        ),
        (
            omena_query::OmenaQuerySourceTypeFactExpressionShapeV0::ComputedNonLiteral,
            omena_query::OmenaQuerySourceTypeFactLexicalDispositionV0::Unresolved,
        ),
        (
            omena_query::OmenaQuerySourceTypeFactExpressionShapeV0::NestedTemplate,
            omena_query::OmenaQuerySourceTypeFactLexicalDispositionV0::Unresolved,
        ),
        (
            omena_query::OmenaQuerySourceTypeFactExpressionShapeV0::MultiInterpolation,
            omena_query::OmenaQuerySourceTypeFactLexicalDispositionV0::Unresolved,
        ),
        (
            omena_query::OmenaQuerySourceTypeFactExpressionShapeV0::Other,
            omena_query::OmenaQuerySourceTypeFactLexicalDispositionV0::Unresolved,
        ),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, (shape_class, lexical_disposition))| {
        omena_query::OmenaQuerySourceTypeFactLexicalAttemptV0::new(
            selector_span,
            format!("fixture-type-fact-target-{index}"),
            Some(style_uri.clone()),
            shape_class,
            lexical_disposition,
        )
    })
    .collect::<Vec<_>>();
    let text_hash = crate::source_document_cache::source_document_text_hash(source_text);
    let cache_storage = crate::cache_root::LspCacheStorageConfigV0::default();
    crate::source_document_cache::store_source_document_index_sidecar(
        &cache_storage,
        Some(workspace_uri.as_str()),
        source_uri.as_str(),
        "typescriptreact",
        text_hash.as_str(),
        &resolution_inputs,
        &cached_index,
        source_type_fact_attempts.as_slice(),
        false,
    );
    let sidecar_path =
        crate::source_document_cache::source_document_index_sidecar_file_path_for_test(
            &cache_storage,
            Some(workspace_uri.as_str()),
            source_uri.as_str(),
            "typescriptreact",
        )
        .ok_or_else(|| std::io::Error::other("source document sidecar path should resolve"))?;
    assert!(
        sidecar_path.exists(),
        "fixture should persist a source syntax sidecar: {sidecar_path:?}"
    );

    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "source-syntax-sidecar",
                    },
                ],
            },
        }),
    );
    let turn = handle_lsp_message_scheduled_outputs_or_dispatch(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {},
        }),
    );
    let workspace_index_job = match turn {
        LspLoopTurnV0::OutputsAndDeferredDiagnostics {
            mut workspace_index_jobs,
            ..
        } => workspace_index_jobs
            .pop()
            .ok_or_else(|| std::io::Error::other("missing workspace index job"))?,
        other => {
            return Err(std::io::Error::other(format!(
                "initialized should schedule background workspace indexing: {other:?}"
            ))
            .into());
        }
    };
    let result = collect_background_workspace_index(workspace_index_job);
    apply_background_workspace_index_result(&mut state, result);
    let indexed_source = state
        .document(source_uri.as_str())
        .ok_or_else(|| std::io::Error::other("source sidecar should index source document"))?;
    assert_eq!(
        indexed_source
            .source_syntax_index
            .style_property_accesses
            .len(),
        1,
        "source syntax sidecar must preserve style property accesses"
    );
    assert_eq!(
        indexed_source
            .source_syntax_index
            .inline_style_declarations
            .len(),
        1,
        "source syntax sidecar must preserve inline style declarations"
    );
    assert_eq!(
        indexed_source.source_syntax_index.selector_references[0].surface,
        SourceSelectorReferenceSurface::OmenaTsgoTypeFactProjection,
        "source syntax sidecar must preserve selector-reference provenance"
    );
    assert_eq!(
        indexed_source
            .source_syntax_index
            .type_fact_target_skipped_count,
        skipped_reasons.len(),
        "source syntax sidecar must preserve skipped type-fact observability"
    );
    assert_eq!(
        indexed_source.source_type_fact_lexical_attempts, source_type_fact_attempts,
        "source syntax sidecar must preserve every expression shape and lexical disposition"
    );
    assert_eq!(
        indexed_source
            .source_syntax_index
            .class_value_universes
            .len(),
        1,
        "source syntax sidecar must preserve class value universes"
    );
    assert_eq!(
        indexed_source
            .source_syntax_index
            .domain_class_references
            .len(),
        1,
        "source syntax sidecar must preserve domain class references"
    );
    assert_eq!(indexed_source.source_syntax_index.source_elements.len(), 2);
    assert_eq!(
        indexed_source.source_syntax_index.source_elements[0].static_class_names,
        vec!["scope-root"]
    );
    assert!(
        indexed_source.source_syntax_index.source_elements[0].classes_are_exact,
        "source syntax sidecar must preserve exact static classes"
    );
    assert_eq!(
        indexed_source
            .source_syntax_index
            .element_parent_edges
            .len(),
        1
    );

    let references_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/references",
            "params": {
                "textDocument": {
                    "uri": style_uri,
                },
                "position": {
                    "line": 0,
                    "character": 2,
                },
                "context": {
                    "includeDeclaration": false,
                },
            },
        }),
    );
    let reference_locations = references_response
        .as_ref()
        .and_then(|response| response.pointer("/result"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("references response should contain locations"))?;
    assert!(
        reference_locations.iter().any(|location| location
            .get("uri")
            .and_then(Value::as_str)
            .is_some_and(|uri| file_uri_equivalent(uri, source_uri.as_str()))),
        "background index should use the persisted source syntax sidecar: {references_response:?}"
    );

    let _ = std::fs::remove_dir_all(&workspace_root);
    Ok(())
}

#[test]
fn indexed_source_files_feed_references_and_rename() -> TestResult {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-source-occurrences-{}",
        std::process::id()
    ));
    let src_dir = workspace_root.join("src");
    let source_path = src_dir.join("App.tsx");
    let style_path = src_dir.join("Button.module.scss");
    let _ = std::fs::remove_dir_all(&workspace_root);
    std::fs::create_dir_all(&src_dir)?;
    std::fs::write(&style_path, ".root { color: red; }")?;
    std::fs::write(
        &source_path,
        "import styles from \"./Button.module.scss\";\nconst view = <div className={styles.root} />;",
    )?;

    let workspace_uri = format!("file://{}", workspace_root.display());
    let source_uri = format!("file://{}", source_path.display());
    let style_uri = format!("file://{}", style_path.display());
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "source-occurrences",
                    },
                ],
            },
        }),
    );
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {},
        }),
    );

    let references_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/references",
            "params": {
                "textDocument": {
                    "uri": style_uri,
                },
                "position": {
                    "line": 0,
                    "character": 2,
                },
                "context": {
                    "includeDeclaration": false,
                },
            },
        }),
    );
    let reference_locations = references_response
        .as_ref()
        .and_then(|response| response.pointer("/result"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("references response should contain locations"))?;
    assert!(
        reference_locations.iter().any(|location| location
            .get("uri")
            .and_then(Value::as_str)
            .is_some_and(|uri| file_uri_equivalent(uri, source_uri.as_str()))),
        "disk-indexed source occurrence should appear in references: {references_response:?}"
    );
    assert!(
        state.workspace_occurrence_index_memo_lock().is_some(),
        "references should populate the workspace occurrence memo"
    );
    for removed_sidecar_dir in [
        "source-occurrence-index-v1",
        "style-symbol-occurrence-index-v1",
    ] {
        let path = workspace_root
            .join(".cache")
            .join("omena")
            .join(removed_sidecar_dir);
        assert!(
            !path.exists(),
            "occurrence queries must not recreate write-only sidecar directory {removed_sidecar_dir}: {path:?}"
        );
    }
    *state.workspace_occurrence_index_memo_lock() = None;
    state
        .document_mut(source_uri.as_str())
        .ok_or_else(|| std::io::Error::other("source document should remain indexed"))?
        .source_selector_candidates
        .clear();

    let cached_references_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "textDocument/references",
            "params": {
                "textDocument": {
                    "uri": style_uri,
                },
                "position": {
                    "line": 0,
                    "character": 2,
                },
                "context": {
                    "includeDeclaration": false,
                },
            },
        }),
    );
    let cached_reference_locations = cached_references_response
        .as_ref()
        .and_then(|response| response.pointer("/result"))
        .and_then(Value::as_array)
        .ok_or_else(|| {
            std::io::Error::other("cached references response should contain locations")
        })?;
    assert!(
        cached_reference_locations.iter().any(|location| location
            .get("uri")
            .and_then(Value::as_str)
            .is_some_and(|uri| file_uri_equivalent(uri, source_uri.as_str()))),
        "disk sidecar should rehydrate source references without source candidate rescanning: {cached_references_response:?}"
    );
    let memo_after_cached_references = state
        .workspace_occurrence_index_memo_lock()
        .as_ref()
        .map(|memo| std::sync::Arc::clone(&memo.source_selector_index))
        .ok_or_else(|| {
            std::io::Error::other("cached references should populate source occurrence memo")
        })?;
    crate::style_symbol_provider::reset_workspace_occurrence_extractor_counters_for_test();

    let rename_response =
        crate::workspace_occurrence_cache::with_workspace_occurrence_shadow_none_for_test(|| {
            handle_lsp_message(
                &mut state,
                json!({
                    "jsonrpc": "2.0",
                    "id": 3,
                    "method": "textDocument/rename",
                    "params": {
                        "textDocument": {
                            "uri": style_uri,
                        },
                        "position": {
                            "line": 0,
                            "character": 2,
                        },
                        "newName": "button",
                    },
                }),
            )
        });
    let changes = rename_response
        .as_ref()
        .and_then(|response| response.pointer("/result/changes"))
        .and_then(Value::as_object)
        .ok_or_else(|| std::io::Error::other("rename response should contain changes"))?;
    assert!(
        changes
            .keys()
            .any(|uri| file_uri_equivalent(uri.as_str(), source_uri.as_str())),
        "disk-indexed source occurrence should receive rename edits: {rename_response:?}"
    );
    assert!(
        changes
            .keys()
            .any(|uri| file_uri_equivalent(uri.as_str(), style_uri.as_str())),
        "style definition should still receive rename edits: {rename_response:?}"
    );
    let memo_after_rename = state
        .workspace_occurrence_index_memo_lock()
        .as_ref()
        .map(|memo| std::sync::Arc::clone(&memo.source_selector_index))
        .ok_or_else(|| std::io::Error::other("rename should retain source occurrence memo"))?;
    let shadow_verifications =
        crate::style_symbol_provider::workspace_occurrence_shadow_verification_total_for_test();
    assert_eq!(
        shadow_verifications, 0,
        "forcing shadow verification off should exercise the ordinary memo-hit branch",
    );
    assert!(
        std::sync::Arc::ptr_eq(&memo_after_cached_references, &memo_after_rename),
        "rename should reuse the rehydrated index when shadow verification is forced off",
    );
    assert_eq!(
        serde_json::to_vec(memo_after_cached_references.as_ref())?,
        serde_json::to_vec(memo_after_rename.as_ref())?,
        "the force-off memo-hit branch must preserve the rehydrated source occurrence bytes",
    );
    let _ = std::fs::remove_dir_all(&workspace_root);
    Ok(())
}

#[test]
fn editor_storage_initialization_keeps_the_workspace_cache_clean() -> TestResult {
    let nonce = format!("{}", std::process::id());
    let workspace_root =
        std::env::temp_dir().join(format!("omena-lsp-server-editor-storage-workspace-{nonce}"));
    let editor_storage_root =
        std::env::temp_dir().join(format!("omena-lsp-server-editor-storage-root-{nonce}"));
    let _ = std::fs::remove_dir_all(&workspace_root);
    let _ = std::fs::remove_dir_all(&editor_storage_root);
    let src_dir = workspace_root.join("src");
    std::fs::create_dir_all(&src_dir)?;
    let style_path = src_dir.join("Button.module.scss");
    let source_path = src_dir.join("App.tsx");
    std::fs::write(&style_path, ".root { color: red; }")?;
    std::fs::write(
        &source_path,
        "import styles from \"./Button.module.scss\";\nconst view = <div className={styles.root} />;",
    )?;

    let workspace_uri = path_to_file_uri(workspace_root.as_path());
    let style_uri = path_to_file_uri(style_path.as_path());
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "initializationOptions": {
                    "storage": {
                        "globalStoragePath": editor_storage_root.join("global"),
                        "workspaceStoragePath": editor_storage_root.join("workspace"),
                        "logPath": editor_storage_root.join("logs"),
                        "location": "editor",
                    },
                },
                "workspaceFolders": [{"uri": workspace_uri, "name": "editor-storage"}],
            },
        }),
    );
    handle_lsp_message(
        &mut state,
        json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}),
    );
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/references",
            "params": {
                "textDocument": {"uri": style_uri},
                "position": {"line": 0, "character": 2},
                "context": {"includeDeclaration": false},
            },
        }),
    );
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": STYLE_DIAGNOSTICS_REQUEST,
            "params": {"textDocument": {"uri": style_uri}},
        }),
    );

    let workspace_cache = workspace_root.join(".cache").join("omena");
    assert!(
        !workspace_cache.exists(),
        "editor storage must keep the opened workspace clean: {workspace_cache:?}"
    );
    assert!(
        editor_storage_root.exists(),
        "editor storage must receive cache writes: {editor_storage_root:?}"
    );
    let resolved = crate::cache_root::process_cache_roots(
        &state.resolution.cache_storage,
        workspace_uri.as_str(),
        workspace_root.as_path(),
    );
    assert_eq!(
        resolved.source,
        crate::cache_root::CacheRootSourceV0::InitializationOptions
    );
    let resolved_workspace_root = resolved.workspace.ok_or_else(|| {
        std::io::Error::other("editor initialization must resolve a workspace cache root")
    })?;
    for cache_dir_name in [
        "diagnostics-cache-v1",
        "source-document-index-v1",
        "workspace-occurrence-shards-v2",
        "source-type-fact-cache-v1",
    ] {
        assert!(
            resolved_workspace_root
                .join(cache_dir_name)
                .starts_with(editor_storage_root.join("workspace")),
            "{cache_dir_name} must resolve below editor workspace storage"
        );
    }
    assert!(
        crate::workspace_index::should_skip_workspace_index_dir(
            workspace_root.join(".cache").as_path()
        ),
        "workspace indexing must keep skipping the shared .cache directory"
    );
    eprintln!(
        "editorStorage rung=initializationOptions resolvedRoot={} workspaceCacheExists={}",
        resolved_workspace_root.display(),
        workspace_cache.exists(),
    );

    let _ = std::fs::remove_dir_all(workspace_root);
    let _ = std::fs::remove_dir_all(editor_storage_root);
    Ok(())
}

#[test]
fn cache_location_configuration_switches_all_caches_to_workspace_mode() -> TestResult {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-cache-location-workspace-{}",
        std::process::id()
    ));
    let editor_storage_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-cache-location-editor-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&workspace_root);
    let _ = std::fs::remove_dir_all(&editor_storage_root);
    std::fs::create_dir_all(&workspace_root)?;
    let workspace_uri = path_to_file_uri(workspace_root.as_path());
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "initializationOptions": {
                    "storage": {
                        "globalStoragePath": editor_storage_root.join("global"),
                        "workspaceStoragePath": editor_storage_root.join("workspace"),
                        "location": "editor",
                    },
                },
                "workspaceFolders": [{"uri": workspace_uri, "name": "cache-location"}],
            },
        }),
    );
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeConfiguration",
            "params": {"settings": {"omena": {"cache": {"location": "workspace"}}}},
        }),
    );

    let roots = crate::cache_root::process_cache_roots(
        &state.resolution.cache_storage,
        workspace_uri.as_str(),
        workspace_root.as_path(),
    );
    assert_eq!(
        roots.source,
        crate::cache_root::CacheRootSourceV0::Workspace
    );
    let workspace_cache = workspace_root.join(".cache").join("omena");
    assert_eq!(roots.workspace.as_deref(), Some(workspace_cache.as_path()));
    for cache_dir_name in [
        "diagnostics-cache-v1",
        "source-document-index-v1",
        "workspace-occurrence-shards-v2",
        "source-type-fact-cache-v1",
    ] {
        assert_eq!(
            crate::cache_root::resolved_workspace_cache_dir(
                &state.resolution.cache_storage,
                workspace_uri.as_str(),
                workspace_root.as_path(),
                cache_dir_name,
            ),
            Some(workspace_cache.join(cache_dir_name))
        );
    }

    let _ = std::fs::remove_dir_all(workspace_root);
    let _ = std::fs::remove_dir_all(editor_storage_root);
    Ok(())
}

#[test]
fn initialization_sweeps_only_owned_workspace_cache_paths() -> TestResult {
    const RELOCATED_LEGACY_CACHE_DIRS: [&str; 9] = [
        "diagnostics-cache-v0",
        "source-document-index-v0",
        "source-occurrence-index-v0",
        "source-occurrence-index-v1",
        "source-type-fact-cache-v0",
        "style-symbol-occurrence-index-v0",
        "style-symbol-occurrence-index-v1",
        "workspace-occurrence-shards-v0",
        "workspace-occurrence-shards-v1",
    ];
    const RELOCATED_CURRENT_CACHE_DIRS: [&str; 5] = [
        "diagnostics-cache-v1",
        "source-document-index-v1",
        "source-type-fact-cache-v1",
        "workspace-occurrence-shards-v2",
        "external-sif-v0",
    ];

    let nonce = format!("{}", std::process::id());
    let fixture_root =
        std::env::temp_dir().join(format!("omena-lsp-server-cache-sweep-workspace-{nonce}"));
    let editor_storage_root =
        std::env::temp_dir().join(format!("omena-lsp-server-cache-sweep-editor-{nonce}"));
    let _ = std::fs::remove_dir_all(&fixture_root);
    let _ = std::fs::remove_dir_all(&editor_storage_root);
    std::fs::create_dir_all(&fixture_root)?;

    let run_git = |args: &[&str]| -> Result<std::process::Output, std::io::Error> {
        std::process::Command::new("git")
            .args(args)
            .current_dir(&fixture_root)
            .output()
    };
    assert!(run_git(&["init", "--quiet"])?.status.success());
    assert!(
        run_git(&["config", "user.email", "cache-sweep@example.invalid"])?
            .status
            .success()
    );
    assert!(
        run_git(&["config", "user.name", "Cache Sweep Fixture"])?
            .status
            .success()
    );

    let mut rows = Vec::new();
    for residual_mask in 0_u8..(1 << RELOCATED_CURRENT_CACHE_DIRS.len()) {
        for markers_present in [false, true] {
            let row_name = format!(
                "row-{residual_mask:02}-markers-{}",
                if markers_present { "present" } else { "absent" }
            );
            let workspace_root = fixture_root.join(row_name.as_str());
            let omena_root = workspace_root.join(".cache").join("omena");
            std::fs::create_dir_all(&omena_root)?;
            let foreign_file = omena_root.join("foreign-tool-state.txt");
            std::fs::write(&foreign_file, b"keep")?;
            let foreign_cache_file = workspace_root.join(".cache").join("other-tool-state.txt");
            std::fs::write(&foreign_cache_file, b"keep")?;
            rows.push((
                residual_mask,
                markers_present,
                workspace_root,
                foreign_file,
                foreign_cache_file,
            ));
        }
    }
    let workspace_mode_root = fixture_root.join("workspace-mode-control");
    let workspace_mode_omena_root = workspace_mode_root.join(".cache").join("omena");
    std::fs::create_dir_all(&workspace_mode_omena_root)?;
    let workspace_mode_foreign = workspace_mode_omena_root.join("foreign-tool-state.txt");
    std::fs::write(&workspace_mode_foreign, b"keep")?;
    std::fs::write(
        workspace_mode_omena_root.join(".gitignore"),
        crate::disk_cache::OMENA_CACHE_GITIGNORE_BYTES,
    )?;
    std::fs::write(
        workspace_mode_omena_root.join("CACHEDIR.TAG"),
        crate::disk_cache::OMENA_CACHEDIR_TAG_BYTES,
    )?;
    std::fs::write(
        workspace_mode_omena_root.join(".omena-cache-owner.json"),
        serde_json::to_vec(&json!({
            "schemaVersion": "0",
            "product": "omena.cache-root-attribution",
            "workspaceIdentity": workspace_mode_root.to_string_lossy(),
        }))?,
    )?;

    let marker_control_roots = (0..4)
        .map(|ordinal| fixture_root.join(format!("foreign-marker-control-{ordinal}")))
        .collect::<Vec<_>>();
    for (ordinal, workspace_root) in marker_control_roots.iter().enumerate() {
        let omena_root = workspace_root.join(".cache").join("omena");
        std::fs::create_dir_all(&omena_root)?;
        let attribution = if ordinal == 0 {
            serde_json::to_vec(&json!({
                "schemaVersion": "0",
                "product": "foreign.cache-root-attribution",
                "workspaceIdentity": workspace_root.to_string_lossy(),
            }))?
        } else {
            serde_json::to_vec(&json!({
                "schemaVersion": "0",
                "product": "omena.cache-root-attribution",
                "workspaceIdentity": workspace_root.to_string_lossy(),
            }))?
        };
        std::fs::write(omena_root.join(".omena-cache-owner.json"), attribution)?;
        if ordinal == 3 {
            std::fs::create_dir_all(omena_root.join("CACHEDIR.TAG"))?;
            std::fs::write(omena_root.join("CACHEDIR.TAG").join("foreign.txt"), b"keep")?;
        } else {
            let cachedir_bytes = if ordinal == 1 {
                b"foreign cachedir marker\n".as_slice()
            } else {
                crate::disk_cache::OMENA_CACHEDIR_TAG_BYTES
            };
            std::fs::write(omena_root.join("CACHEDIR.TAG"), cachedir_bytes)?;
        }
        let gitignore_bytes = if ordinal == 2 {
            b"# foreign ignore policy\nforeign-only\n".as_slice()
        } else {
            crate::disk_cache::OMENA_CACHE_GITIGNORE_BYTES
        };
        std::fs::write(omena_root.join(".gitignore"), gitignore_bytes)?;
    }

    #[cfg(unix)]
    let symlink_control = {
        let workspace_root = fixture_root.join("symlink-root-control");
        let cache_root = workspace_root.join(".cache");
        let external_omena_root =
            std::env::temp_dir().join(format!("omena-lsp-server-cache-sweep-external-{nonce}"));
        let _ = std::fs::remove_dir_all(&external_omena_root);
        std::fs::create_dir_all(&cache_root)?;
        std::fs::create_dir_all(&external_omena_root)?;
        std::os::unix::fs::symlink(&external_omena_root, cache_root.join("omena"))?;
        Some((workspace_root, external_omena_root))
    };
    #[cfg(not(unix))]
    let symlink_control: Option<(std::path::PathBuf, std::path::PathBuf)> = None;

    let owned_only_control_root = fixture_root.join("owned-only-control");
    let hidden_foreign_control_root = fixture_root.join("hidden-foreign-control");

    assert!(run_git(&["add", "-f", "."])?.status.success());
    assert!(
        run_git(&["commit", "--quiet", "-m", "baseline foreign cache files"])?
            .status
            .success()
    );

    for (residual_mask, markers_present, workspace_root, _, _) in &rows {
        let omena_root = workspace_root.join(".cache").join("omena");
        for cache_dir_name in RELOCATED_LEGACY_CACHE_DIRS {
            let legacy_dir = omena_root.join(cache_dir_name);
            std::fs::create_dir_all(&legacy_dir)?;
            std::fs::write(legacy_dir.join("dead.json"), b"{}")?;
        }
        for (ordinal, cache_dir_name) in RELOCATED_CURRENT_CACHE_DIRS.iter().enumerate() {
            if residual_mask & (1 << ordinal) == 0 {
                continue;
            }
            let cache_dir = omena_root.join(cache_dir_name);
            std::fs::create_dir_all(&cache_dir)?;
            std::fs::write(cache_dir.join("stale.json"), b"{}")?;
        }
        if *markers_present {
            std::fs::write(
                omena_root.join(".gitignore"),
                crate::disk_cache::OMENA_CACHE_GITIGNORE_BYTES,
            )?;
            std::fs::write(
                omena_root.join("CACHEDIR.TAG"),
                crate::disk_cache::OMENA_CACHEDIR_TAG_BYTES,
            )?;
            std::fs::write(
                omena_root.join(".omena-cache-owner.json"),
                serde_json::to_vec(&json!({
                    "schemaVersion": "0",
                    "product": "omena.cache-root-attribution",
                    "workspaceIdentity": workspace_root.to_string_lossy(),
                }))?,
            )?;
        }
    }
    for cache_dir_name in RELOCATED_LEGACY_CACHE_DIRS
        .into_iter()
        .chain(RELOCATED_CURRENT_CACHE_DIRS)
    {
        let cache_dir = workspace_mode_omena_root.join(cache_dir_name);
        std::fs::create_dir_all(&cache_dir)?;
        std::fs::write(cache_dir.join("shard.json"), b"{}")?;
    }
    for workspace_root in &marker_control_roots {
        let cache_dir = workspace_root
            .join(".cache")
            .join("omena")
            .join("diagnostics-cache-v1");
        std::fs::create_dir_all(&cache_dir)?;
        std::fs::write(cache_dir.join("stale.json"), b"{}")?;
    }
    if let Some((_, external_omena_root)) = &symlink_control {
        let external_cache = external_omena_root.join("diagnostics-cache-v1");
        std::fs::create_dir_all(&external_cache)?;
        std::fs::write(external_cache.join("must-survive.json"), b"{}")?;
    }
    for workspace_root in [&owned_only_control_root, &hidden_foreign_control_root] {
        let omena_root = workspace_root.join(".cache").join("omena");
        let current_cache = omena_root.join("diagnostics-cache-v1");
        std::fs::create_dir_all(&current_cache)?;
        std::fs::write(current_cache.join("stale.json"), b"{}")?;
        std::fs::write(
            omena_root.join(".gitignore"),
            crate::disk_cache::OMENA_CACHE_GITIGNORE_BYTES,
        )?;
        std::fs::write(
            omena_root.join("CACHEDIR.TAG"),
            crate::disk_cache::OMENA_CACHEDIR_TAG_BYTES,
        )?;
        std::fs::write(
            omena_root.join(".omena-cache-owner.json"),
            serde_json::to_vec(&json!({
                "schemaVersion": "0",
                "product": "omena.cache-root-attribution",
                "workspaceIdentity": workspace_root.to_string_lossy(),
            }))?,
        )?;
    }
    let hidden_foreign_file = hidden_foreign_control_root
        .join(".cache")
        .join("omena")
        .join("foreign-untracked-state.txt");
    std::fs::write(&hidden_foreign_file, b"keep hidden")?;
    let hidden_status_before = run_git(&[
        "status",
        "--porcelain",
        "--",
        "hidden-foreign-control/.cache/omena",
    ])?;
    assert!(hidden_status_before.status.success());
    assert_eq!(
        String::from_utf8_lossy(&hidden_status_before.stdout),
        "",
        "the exact ignore marker must hide the foreign entry before the sweep"
    );

    let cache_storage = crate::cache_root::LspCacheStorageConfigV0 {
        initialization_global_storage: Some(editor_storage_root.join("global")),
        initialization_workspace_storage: Some(editor_storage_root.join("workspace")),
        log_path: None,
        command_cache_dir: None,
        location: crate::cache_root::CacheLocationV0::Editor,
    };
    let mut workspace_folders = Vec::new();
    let mut resolved_root_survivors = Vec::new();
    for (_, _, workspace_root, _, _) in &rows {
        let workspace_uri = path_to_file_uri(workspace_root.as_path());
        workspace_folders.push(json!({"uri": workspace_uri, "name": "cache-sweep"}));
        let resolved_root_survivor = crate::cache_root::process_cache_roots(
            &cache_storage,
            workspace_uri.as_str(),
            workspace_root.as_path(),
        )
        .workspace
        .ok_or_else(|| std::io::Error::other("editor cache root should resolve"))?
        .join("diagnostics-cache-v1")
        .join("preexisting-shard.json");
        std::fs::create_dir_all(
            resolved_root_survivor
                .parent()
                .ok_or_else(|| std::io::Error::other("resolved survivor parent"))?,
        )?;
        std::fs::write(&resolved_root_survivor, b"keep")?;
        resolved_root_survivors.push(resolved_root_survivor);
    }
    for workspace_root in &marker_control_roots {
        workspace_folders.push(json!({
            "uri": path_to_file_uri(workspace_root.as_path()),
            "name": "foreign-marker-control",
        }));
    }
    if let Some((workspace_root, _)) = &symlink_control {
        workspace_folders.push(json!({
            "uri": path_to_file_uri(workspace_root.as_path()),
            "name": "symlink-root-control",
        }));
    }
    for workspace_root in [&owned_only_control_root, &hidden_foreign_control_root] {
        workspace_folders.push(json!({
            "uri": path_to_file_uri(workspace_root.as_path()),
            "name": "marker-atomicity-control",
        }));
    }

    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "initializationOptions": {
                    "storage": {
                        "globalStoragePath": editor_storage_root.join("global"),
                        "workspaceStoragePath": editor_storage_root.join("workspace"),
                        "location": "editor",
                    },
                },
                "workspaceFolders": workspace_folders,
            },
        }),
    );

    let hidden_foreign_omena_root = hidden_foreign_control_root.join(".cache").join("omena");
    let hidden_status_after = run_git(&[
        "status",
        "--porcelain",
        "--",
        "hidden-foreign-control/.cache/omena",
    ])?;
    assert!(hidden_status_after.status.success());
    assert_eq!(
        String::from_utf8_lossy(&hidden_status_after.stdout),
        "",
        "the relocated-cache sweep must not expose a hidden foreign entry"
    );

    for (
        (residual_mask, markers_present, workspace_root, foreign_file, foreign_cache_file),
        resolved_root_survivor,
    ) in rows.iter().zip(&resolved_root_survivors)
    {
        let omena_root = workspace_root.join(".cache").join("omena");
        for cache_dir_name in RELOCATED_LEGACY_CACHE_DIRS {
            assert!(
                !omena_root.join(cache_dir_name).exists(),
                "row mask={residual_mask:02} markers={markers_present}: legacy cache {cache_dir_name} survived"
            );
        }
        for cache_dir_name in RELOCATED_CURRENT_CACHE_DIRS {
            assert!(
                !omena_root.join(cache_dir_name).exists(),
                "row mask={residual_mask:02} markers={markers_present}: relocated cache {cache_dir_name} survived"
            );
        }
        for marker_name in [".gitignore", "CACHEDIR.TAG", ".omena-cache-owner.json"] {
            assert_eq!(
                omena_root.join(marker_name).is_file(),
                *markers_present,
                "row mask={residual_mask:02} markers={markers_present}: a foreign entry must retain an existing marker barrier"
            );
        }
        assert!(
            foreign_file.is_file(),
            "row mask={residual_mask:02} markers={markers_present}: foreign file was removed"
        );
        assert!(
            foreign_cache_file.is_file(),
            "row mask={residual_mask:02} markers={markers_present}: enclosing cache file was removed"
        );
        assert!(
            resolved_root_survivor.is_file(),
            "row mask={residual_mask:02} markers={markers_present}: resolved editor shard was removed"
        );
    }
    for workspace_root in &marker_control_roots {
        let omena_root = workspace_root.join(".cache").join("omena");
        assert!(
            !omena_root.join("diagnostics-cache-v1").exists(),
            "foreign marker ownership must not disable owned-cache removal"
        );
        for marker_name in [".gitignore", "CACHEDIR.TAG", ".omena-cache-owner.json"] {
            assert!(
                omena_root.join(marker_name).exists(),
                "foreign or blocked marker {marker_name} must survive"
            );
        }
    }
    if let Some((workspace_root, external_omena_root)) = &symlink_control {
        assert!(
            std::fs::symlink_metadata(workspace_root.join(".cache").join("omena"))
                .is_ok_and(|metadata| metadata.file_type().is_symlink()),
            "the workspace cache-root symlink must survive"
        );
        assert!(
            external_omena_root
                .join("diagnostics-cache-v1")
                .join("must-survive.json")
                .is_file(),
            "the sweep must not follow a workspace cache-root symlink"
        );
    }
    assert!(
        std::fs::symlink_metadata(owned_only_control_root.join(".cache").join("omena"))
            .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound),
        "a root containing only owned cache paths and markers must be removed"
    );
    assert!(
        !hidden_foreign_omena_root
            .join("diagnostics-cache-v1")
            .exists(),
        "the hidden-foreign control must still remove its owned cache"
    );
    assert!(
        hidden_foreign_file.is_file(),
        "the hidden foreign entry must survive"
    );
    for marker_name in [".gitignore", "CACHEDIR.TAG", ".omena-cache-owner.json"] {
        assert!(
            hidden_foreign_omena_root.join(marker_name).is_file(),
            "the marker barrier must survive while a hidden foreign entry remains"
        );
    }
    let mut workspace_mode_state = LspShellState::default();
    handle_lsp_message(
        &mut workspace_mode_state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "initialize",
            "params": {
                "initializationOptions": {
                    "storage": {
                        "location": "workspace",
                    },
                },
                "workspaceFolders": [{
                    "uri": path_to_file_uri(workspace_mode_root.as_path()),
                    "name": "workspace-mode-control",
                }],
            },
        }),
    );
    for cache_dir_name in RELOCATED_LEGACY_CACHE_DIRS {
        assert!(
            !workspace_mode_omena_root.join(cache_dir_name).exists(),
            "workspace mode must still remove retired cache {cache_dir_name}"
        );
    }
    for cache_dir_name in RELOCATED_CURRENT_CACHE_DIRS {
        assert!(
            workspace_mode_omena_root.join(cache_dir_name).is_dir(),
            "workspace mode must preserve live cache {cache_dir_name}"
        );
    }
    for marker_name in [".gitignore", "CACHEDIR.TAG", ".omena-cache-owner.json"] {
        assert!(
            workspace_mode_omena_root.join(marker_name).is_file(),
            "workspace mode must preserve marker {marker_name} while live caches remain"
        );
    }
    assert!(workspace_mode_foreign.is_file());

    let git_status = run_git(&["status", "--porcelain", "--", "."])?;
    assert!(git_status.status.success());
    assert_eq!(
        String::from_utf8_lossy(&git_status.stdout),
        "",
        "the relocated-cache sweep must leave no git-visible residue"
    );
    eprintln!(
        "cacheSweepMatrix rows={} residualSubsets={} markerStates=2 legacyNames={} currentNames={} ownedCacheResiduals=0 retainedMarkerBarrierRows={} foreignSurvivors={} resolvedRootSurvivors={} workspaceModeCurrentCaches=5 workspaceModeMarkers=3 foreignMarkerControls=4 symlinkRootControls={} ownedOnlyMarkerRemovalControls=1 hiddenForeignBarrierControls=1 gitVisibleResidues=0",
        rows.len(),
        1 << RELOCATED_CURRENT_CACHE_DIRS.len(),
        RELOCATED_LEGACY_CACHE_DIRS.len(),
        RELOCATED_CURRENT_CACHE_DIRS.len(),
        rows.iter().filter(|(_, markers, _, _, _)| *markers).count(),
        rows.iter()
            .filter(|(_, _, _, path, _)| path.is_file())
            .count(),
        resolved_root_survivors
            .iter()
            .filter(|path| path.is_file())
            .count(),
        usize::from(symlink_control.is_some()),
    );

    let _ = std::fs::remove_dir_all(fixture_root);
    let _ = std::fs::remove_dir_all(editor_storage_root);
    if let Some((_, external_omena_root)) = symlink_control {
        let _ = std::fs::remove_dir_all(external_omena_root);
    }
    Ok(())
}

#[test]
fn uncreatable_cache_root_refuses_without_workspace_fallback() -> TestResult {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-cache-root-refusal-workspace-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&workspace_root);
    let src_dir = workspace_root.join("src");
    std::fs::create_dir_all(&src_dir)?;
    let source_path = src_dir.join("App.tsx");
    std::fs::write(&source_path, "export const app = 'cache-refusal';")?;
    let fixture_path = std::env::var_os(crate::cache_root::OMENA_CACHE_DIR_ENV)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/cache-root-refusal-not-a-directory")
        });
    assert!(
        fixture_path.is_file(),
        "the refusal perturbation must be a regular file: {fixture_path:?}"
    );

    let workspace_uri = path_to_file_uri(workspace_root.as_path());
    let source_uri = path_to_file_uri(source_path.as_path());
    let mut state = LspShellState::default();
    let command_cache_dir = std::env::var_os(crate::cache_root::OMENA_CACHE_DIR_ENV)
        .is_none()
        .then_some(fixture_path.clone());
    state.configure_standalone_cache_storage(command_cache_dir);
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [{"uri": workspace_uri, "name": "cache-refusal"}],
            },
        }),
    );
    handle_lsp_message(
        &mut state,
        json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}),
    );

    let roots = crate::cache_root::process_cache_roots(
        &state.resolution.cache_storage,
        workspace_uri.as_str(),
        workspace_root.as_path(),
    );
    assert_eq!(
        roots.source,
        crate::cache_root::CacheRootSourceV0::Environment,
        "refusal reason must name the forced environment/CLI rung"
    );
    let sidecar_path =
        crate::source_document_cache::source_document_index_sidecar_file_path_for_test(
            &state.resolution.cache_storage,
            Some(workspace_uri.as_str()),
            source_uri.as_str(),
            "typescriptreact",
        )
        .ok_or_else(|| std::io::Error::other("forced sidecar path must resolve syntactically"))?;
    assert!(
        !sidecar_path.exists(),
        "an uncreatable forced root must refuse the sidecar write: {sidecar_path:?}"
    );
    assert!(
        !workspace_root.join(".cache").join("omena").exists(),
        "an uncreatable forced root must never fall back into the workspace"
    );
    eprintln!(
        "cacheRootRefusal rung=environment forcedRoot={} sidecarExists={} workspaceCacheExists={}",
        fixture_path.display(),
        sidecar_path.exists(),
        workspace_root.join(".cache").join("omena").exists(),
    );

    let _ = std::fs::remove_dir_all(workspace_root);
    Ok(())
}

#[test]
fn background_workspace_index_resumes_past_already_indexed_source_files() -> TestResult {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-background-source-resume-{}",
        std::process::id()
    ));
    let src_dir = workspace_root.join("src");
    let style_path = src_dir.join("Button.module.scss");
    let late_source_path = src_dir.join("ZTarget.tsx");
    let _ = std::fs::remove_dir_all(&workspace_root);
    std::fs::create_dir_all(&src_dir)?;
    std::fs::write(&style_path, ".root { color: red; }")?;
    for index in 0..520 {
        std::fs::write(
            src_dir.join(format!("A{index:04}.tsx")),
            format!("export const value{index} = {index};"),
        )?;
    }
    std::fs::write(
        &late_source_path,
        "import styles from \"./Button.module.scss\";\nconst view = <div className={styles.root} />;",
    )?;

    let workspace_uri = crate::protocol::path_to_file_uri(workspace_root.as_path());
    let style_uri = crate::protocol::path_to_file_uri(style_path.as_path());
    let late_source_uri = crate::protocol::path_to_file_uri(late_source_path.as_path());
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "background-source-resume",
                    },
                ],
            },
        }),
    );

    let turn = handle_lsp_message_scheduled_outputs_or_dispatch(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {},
        }),
    );
    let job = match turn {
        LspLoopTurnV0::OutputsAndDeferredDiagnostics {
            mut workspace_index_jobs,
            ..
        } => workspace_index_jobs
            .pop()
            .ok_or_else(|| std::io::Error::other("missing resumable workspace index job"))?,
        other => {
            return Err(std::io::Error::other(format!(
                "initialized should schedule resumable workspace indexing: {other:?}"
            ))
            .into());
        }
    };
    let first_result = collect_background_workspace_index(job);
    assert!(
        first_result.exhausted,
        "first tick should hit the per-tick file budget"
    );
    assert!(
        !first_result.pending_file_uris.is_empty(),
        "exhausted workspace index results must carry a continuation frontier"
    );
    let mut pending_counts = vec![first_result.pending_file_count];
    let mut pending_file_uris = first_result.pending_file_uris.clone();
    apply_background_workspace_index_result(&mut state, first_result);

    while !pending_file_uris.is_empty() {
        let continuation =
            prepare_background_workspace_index_continuation_job(&mut state, pending_file_uris);
        let result = collect_background_workspace_index(continuation);
        pending_file_uris = result.pending_file_uris.clone();
        pending_counts.push(result.pending_file_count);
        apply_background_workspace_index_result(&mut state, result);
    }

    assert!(
        state.document(late_source_uri.as_str()).is_some(),
        "background workspace indexing should advance beyond the first file-budget window"
    );
    assert_eq!(
        state.snapshot().workspace_index_pending_file_count,
        0,
        "workspace index pending count should reach zero after continuation ticks"
    );
    assert!(
        pending_counts
            .windows(2)
            .all(|window| window[1] < window[0]),
        "workspace index pending count should strictly shrink per continuation tick: {pending_counts:?}"
    );
    let references_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/references",
            "params": {
                "textDocument": {
                    "uri": style_uri,
                },
                "position": {
                    "line": 0,
                    "character": 2,
                },
                "context": {
                    "includeDeclaration": false,
                },
            },
        }),
    );
    let reference_locations = references_response
        .as_ref()
        .and_then(|response| response.pointer("/result"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("references response should contain locations"))?;
    assert!(
        reference_locations.iter().any(|location| location
            .get("uri")
            .and_then(Value::as_str)
            .is_some_and(|uri| file_uri_equivalent(uri, late_source_uri.as_str()))),
        "late indexed source occurrence should appear in references: {references_response:?}"
    );

    let _ = std::fs::remove_dir_all(&workspace_root);
    Ok(())
}

#[test]
fn background_workspace_index_admits_foreign_dependencies_from_new_batch_only() -> TestResult {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-background-delta-admit-{}",
        std::process::id()
    ));
    let src_dir = workspace_root.join("src");
    let external_path = workspace_root.join("tokens.scss");
    let _ = std::fs::remove_dir_all(&workspace_root);
    std::fs::create_dir_all(&src_dir)?;
    let lock = omena_sif::OmenaLockV1::new(Vec::new());
    std::fs::write(
        workspace_root.join("omena.lock"),
        omena_sif::write_omena_lock_json_v1(&lock)?,
    )?;
    std::fs::write(external_path.as_path(), "$brand: blue;\n")?;
    let external_uri = crate::protocol::path_to_file_uri(external_path.as_path());
    let mut style_uris = Vec::new();
    for index in 0..4 {
        let path = src_dir.join(format!("Style{index}.module.scss"));
        std::fs::write(path.as_path(), format!(".item{index} {{ color: red; }}"))?;
        style_uris.push(crate::protocol::path_to_file_uri(path.as_path()));
    }

    let workspace_uri = crate::protocol::path_to_file_uri(workspace_root.as_path());
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "background-delta-admit",
                    },
                ],
            },
        }),
    );

    let resolution_inputs =
        resolution_inputs_for_workspace_uri(&state, Some(workspace_uri.as_str()));
    for uri in style_uris.iter().take(3) {
        state.insert_document(
            uri.as_str(),
            lsp_text_document_state(
                uri.clone(),
                Some(workspace_uri.clone()),
                "scss".to_string(),
                0,
                ".old { color: red; }".to_string(),
                &resolution_inputs,
            ),
        );
    }

    crate::document_refresh::reset_foreign_style_dependency_scan_count_for_test();
    let bridge_generations_before = state.external_sif_bridge_generation_count;
    let new_uri = style_uris
        .get(3)
        .ok_or_else(|| std::io::Error::other("missing new style uri"))?;
    let new_source =
        format!("@use \"{external_uri}\" as tokens;\n.new {{ color: tokens.$brand; }}");
    let result = LspWorkspaceIndexResultV0 {
        revision: state.workspace_index_revision,
        progress_token: None,
        documents: vec![lsp_text_document_state(
            new_uri.clone(),
            Some(workspace_uri.clone()),
            "scss".to_string(),
            0,
            new_source,
            &resolution_inputs,
        )],
        pending_file_uris: Vec::new(),
        indexed_count: 1,
        pending_file_count: 0,
        exhausted: false,
    };
    assert!(apply_background_workspace_index_result(&mut state, result));
    assert_eq!(
        crate::document_refresh::foreign_style_dependency_scan_count_for_test(),
        1,
        "background result apply must scan only newly indexed style documents"
    );
    assert_eq!(
        state.external_sif_bridge_generation_count - bridge_generations_before,
        1,
        "background result apply should generate only the newly indexed bridge SIF"
    );
    assert!(
        state
            .resolution
            .external_sifs
            .iter()
            .any(|input| input.canonical_url == external_uri),
        "background result apply should admit the new bridge SIF through a source delta",
    );

    let _ = std::fs::remove_dir_all(&workspace_root);
    Ok(())
}

#[cfg(feature = "test-support")]
#[test]
fn background_workspace_index_delta_diagnostics_recompute_only_changed_style_fact() -> TestResult {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-background-delta-recompute-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&workspace_root);
    std::fs::create_dir_all(workspace_root.join("src").as_path())?;

    let workspace_uri = crate::protocol::path_to_file_uri(workspace_root.as_path());
    let app_uri = format!("{workspace_uri}/src/App.module.scss");
    let theme_uri = format!("{workspace_uri}/src/Theme.module.scss");
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "background-delta-recompute",
                    },
                ],
            },
        }),
    );
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": app_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": "@use \"./Theme\";\n.app { color: $brand; }",
                },
            },
        }),
    );
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(&state, Some(workspace_uri.as_str()));

    let initial_result = LspWorkspaceIndexResultV0 {
        revision: state.workspace_index_revision,
        progress_token: None,
        documents: vec![lsp_text_document_state(
            theme_uri.clone(),
            Some(workspace_uri.clone()),
            "scss".to_string(),
            0,
            "$brand: red;".to_string(),
            &resolution_inputs,
        )],
        pending_file_uris: Vec::new(),
        indexed_count: 1,
        pending_file_count: 0,
        exhausted: false,
    };
    assert!(apply_background_workspace_index_result(
        &mut state,
        initial_result
    ));
    #[cfg(feature = "salsa-style-diagnostics")]
    assert_eq!(
        state
            .style_memo_host
            .borrow()
            .as_ref()
            .map(|host| host.registered_style_path_count()),
        Some(1),
        "background index application must register the admitted foreign style path",
    );

    omena_query::reset_style_fact_entry_probe_for_test();
    let _ = crate::diagnostics_scheduler::run_diagnostics_schedule(
        &mut state,
        crate::diagnostics_scheduler::DiagnosticsScheduleEvent::TextDocument {
            uri: app_uri.clone(),
            is_close: false,
            content_changed: false,
        },
    );
    assert_eq!(
        omena_query::read_style_fact_entry_probe_for_test(),
        std::collections::BTreeSet::from([app_uri.clone(), theme_uri.clone()]),
        "initial diagnostics after background admission must collect every style fact once",
    );
    #[cfg(feature = "salsa-style-diagnostics")]
    assert_eq!(
        state
            .style_memo_host
            .borrow()
            .as_ref()
            .map(|host| host.registered_style_path_count()),
        Some(2),
        "diagnostics must add the open style document to the registered workspace",
    );

    let edited_result = LspWorkspaceIndexResultV0 {
        revision: state.workspace_index_revision,
        progress_token: None,
        documents: vec![lsp_text_document_state(
            theme_uri.clone(),
            Some(workspace_uri.clone()),
            "scss".to_string(),
            1,
            "$brand: blue;".to_string(),
            &resolution_inputs,
        )],
        pending_file_uris: Vec::new(),
        indexed_count: 1,
        pending_file_count: 0,
        exhausted: false,
    };
    assert!(apply_background_workspace_index_result(
        &mut state,
        edited_result
    ));
    #[cfg(feature = "salsa-style-diagnostics")]
    assert_eq!(
        state
            .style_memo_host
            .borrow()
            .as_ref()
            .map(|host| host.registered_style_path_count()),
        Some(2),
        "editing an already registered indexed style must not grow the registered workspace",
    );

    omena_query::reset_style_fact_entry_probe_for_test();
    let _ = crate::diagnostics_scheduler::run_diagnostics_schedule(
        &mut state,
        crate::diagnostics_scheduler::DiagnosticsScheduleEvent::TextDocument {
            uri: app_uri.clone(),
            is_close: false,
            content_changed: false,
        },
    );
    assert_eq!(
        omena_query::read_style_fact_entry_probe_for_test(),
        std::collections::BTreeSet::from([theme_uri.clone()]),
        "background-indexed style edits must recompute only the changed style fact",
    );

    let extra_documents = (0..4)
        .map(|index| {
            let extra_uri = format!("{workspace_uri}/src/Extra{index}.module.scss");
            lsp_text_document_state(
                extra_uri,
                Some(workspace_uri.clone()),
                "scss".to_string(),
                0,
                format!(".extra{index} {{ color: red; }}"),
                &resolution_inputs,
            )
        })
        .collect::<Vec<_>>();
    let extra_result = LspWorkspaceIndexResultV0 {
        revision: state.workspace_index_revision,
        progress_token: None,
        documents: extra_documents,
        pending_file_uris: Vec::new(),
        indexed_count: 4,
        pending_file_count: 0,
        exhausted: false,
    };
    assert!(apply_background_workspace_index_result(
        &mut state,
        extra_result
    ));
    #[cfg(feature = "salsa-style-diagnostics")]
    assert_eq!(
        state
            .style_memo_host
            .borrow()
            .as_ref()
            .map(|host| host.registered_style_path_count()),
        Some(6),
        "unrelated indexed styles should extend the registered workspace without changing delta semantics",
    );

    omena_query::reset_style_fact_entry_probe_for_test();
    let _ = crate::diagnostics_scheduler::run_diagnostics_schedule(
        &mut state,
        crate::diagnostics_scheduler::DiagnosticsScheduleEvent::TextDocument {
            uri: app_uri.clone(),
            is_close: false,
            content_changed: false,
        },
    );
    assert_eq!(
        omena_query::read_style_fact_entry_probe_for_test(),
        std::collections::BTreeSet::from_iter(
            (0..4).map(|index| format!("{workspace_uri}/src/Extra{index}.module.scss"))
        ),
        "newly registered unrelated style files should be collected once before the steady-state delta assertion",
    );

    let second_edited_result = LspWorkspaceIndexResultV0 {
        revision: state.workspace_index_revision,
        progress_token: None,
        documents: vec![lsp_text_document_state(
            theme_uri.clone(),
            Some(workspace_uri.clone()),
            "scss".to_string(),
            2,
            "$brand: green;".to_string(),
            &resolution_inputs,
        )],
        pending_file_uris: Vec::new(),
        indexed_count: 1,
        pending_file_count: 0,
        exhausted: false,
    };
    assert!(apply_background_workspace_index_result(
        &mut state,
        second_edited_result
    ));
    omena_query::reset_style_fact_entry_probe_for_test();
    let _ = crate::diagnostics_scheduler::run_diagnostics_schedule(
        &mut state,
        crate::diagnostics_scheduler::DiagnosticsScheduleEvent::TextDocument {
            uri: app_uri.clone(),
            is_close: false,
            content_changed: false,
        },
    );
    assert_eq!(
        omena_query::read_style_fact_entry_probe_for_test(),
        std::collections::BTreeSet::from([theme_uri.clone()]),
        "background-indexed style edits must stay delta-scoped after unrelated indexed files are registered",
    );

    let _ = std::fs::remove_dir_all(&workspace_root);
    Ok(())
}

#[test]
fn workspace_index_follow_up_wave_count_stays_within_baseline() -> TestResult {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-follow-up-wave-count-{}",
        std::process::id()
    ));
    let src_dir = workspace_root.join("src");
    let app_package = workspace_root.join("node_modules/@app/theme");
    let design_package = workspace_root.join("node_modules/@design/tokens");
    let _ = std::fs::remove_dir_all(&workspace_root);
    std::fs::create_dir_all(&src_dir)?;
    std::fs::create_dir_all(app_package.as_path())?;
    std::fs::create_dir_all(design_package.as_path())?;
    std::fs::write(
        app_package.join("package.json"),
        r#"{"exports":{"./index":{"sass":"./index.scss"}}}"#,
    )?;
    std::fs::write(
        design_package.join("package.json"),
        r#"{"exports":{"./colors":{"sass":"./colors.scss"}}}"#,
    )?;
    std::fs::write(
        app_package.join("index.scss"),
        "@forward \"@design/tokens/colors\";\n",
    )?;
    std::fs::write(design_package.join("colors.scss"), "$ds_gray-700: #333;\n")?;

    let baseline = read_warmup_wave_count_baseline()?;
    let workspace_uri = crate::protocol::path_to_file_uri(workspace_root.as_path());
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "follow-up-wave-count",
                    },
                ],
            },
        }),
    );
    enable_deferred_external_sif_refresh(&mut state);
    if let Some(job) = prepare_deferred_external_sif_refresh_job(&mut state) {
        let result = collect_deferred_external_sif_refresh(job);
        let _ = apply_external_sif_refresh_result_follow_up_diagnostics_effects(&mut state, result);
    }

    let resolution_inputs =
        resolution_inputs_for_workspace_uri(&state, Some(workspace_uri.as_str()));
    crate::diagnostics_follow_up::warmup_wave_count_probe::reset();
    let uri = crate::protocol::path_to_file_uri(src_dir.join("Wave.module.scss").as_path());
    let result = LspWorkspaceIndexResultV0 {
        revision: state.workspace_index_revision,
        progress_token: None,
        documents: vec![lsp_text_document_state(
            uri,
            Some(workspace_uri.clone()),
            "scss".to_string(),
            0,
            "@use \"@app/theme/index\" as ds;\n.wave { color: ds.$ds_gray-700; }".to_string(),
            &resolution_inputs,
        )],
        pending_file_uris: Vec::new(),
        indexed_count: 1,
        pending_file_count: 0,
        exhausted: false,
    };
    assert!(apply_background_workspace_index_result(&mut state, result));
    let job = prepare_deferred_external_sif_refresh_job(&mut state).ok_or_else(|| {
        std::io::Error::other("workspace-index result did not schedule external SIF refresh")
    })?;
    // One settle window admits exactly one refresh tide: the lane is drained
    // and in flight, so a second prepare must be a no-op.
    let refresh_revision_delta: u64 =
        if prepare_deferred_external_sif_refresh_job(&mut state).is_some() {
            2
        } else {
            1
        };
    let result = collect_deferred_external_sif_refresh(job);
    let effects =
        apply_external_sif_refresh_result_follow_up_diagnostics_effects(&mut state, result);
    assert!(
        !effects.outputs.is_empty() || !effects.deferred_diagnostics.is_empty(),
        "the production deferred external-SIF drain path should schedule follow-up diagnostics"
    );

    let wave_count = crate::diagnostics_follow_up::warmup_wave_count_probe::read();
    assert!(
        wave_count > 0,
        "the production deferred external-SIF drain path should exercise follow-up diagnostics"
    );
    assert!(
        refresh_revision_delta <= baseline.external_sif_refresh_revision_delta,
        "workspace index must not admit more external-SIF refresh waves than the committed baseline: observed={refresh_revision_delta}, baseline={}",
        baseline.external_sif_refresh_revision_delta
    );
    assert!(
        wave_count <= baseline.follow_up_wave_count,
        "workspace follow-up diagnostics wave count must not exceed the committed baseline: observed={wave_count}, baseline={}",
        baseline.follow_up_wave_count
    );
    let _ = std::fs::remove_dir_all(&workspace_root);
    Ok(())
}

struct WarmupWaveCountBaseline {
    external_sif_refresh_revision_delta: u64,
    follow_up_wave_count: usize,
}

fn read_warmup_wave_count_baseline() -> Result<WarmupWaveCountBaseline, Box<dyn std::error::Error>>
{
    let baseline_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("baselines")
        .join("z5-warmup-wave-count-baseline-v0.json");
    let baseline: Value = serde_json::from_str(std::fs::read_to_string(baseline_path)?.as_str())?;
    Ok(WarmupWaveCountBaseline {
        external_sif_refresh_revision_delta: baseline
            .get("externalSifRefreshRevisionDelta")
            .and_then(Value::as_u64)
            .ok_or_else(|| std::io::Error::other("missing externalSifRefreshRevisionDelta"))?,
        follow_up_wave_count: baseline
            .get("followUpWaveCount")
            .and_then(Value::as_u64)
            .ok_or_else(|| std::io::Error::other("missing followUpWaveCount"))?
            .try_into()?,
    })
}

#[test]
fn index_settle_steady_state_counter_tuple_matches_committed_baseline() -> TestResult {
    let observed = run_index_settle_steady_state_fixture()?.counter_tuple;
    let baseline = read_index_settle_steady_state_baseline()?;
    assert_eq!(
        observed, baseline,
        "index-settle steady-state counters must match the committed baseline"
    );
    Ok(())
}

#[test]
fn index_settle_steady_state_counter_tuple_is_deterministic() -> TestResult {
    let first = run_index_settle_steady_state_fixture()?.counter_tuple;
    let second = run_index_settle_steady_state_fixture()?.counter_tuple;
    assert_eq!(
        first, second,
        "index-settle steady-state counters must be deterministic within one process",
    );
    Ok(())
}

#[test]
fn index_settle_republish_waits_for_frontier_and_flushes_once() -> TestResult {
    let outcome = run_index_settle_steady_state_fixture()?;
    assert_eq!(
        outcome.mid_frontier_refresh_jobs, 0,
        "background-index SIF refresh must wait while the index frontier is pending"
    );
    assert_eq!(
        outcome.frontier_bypass_refresh_jobs, 1,
        "the same mid-frontier fixture must become refresh-eligible when the frontier predicate is opened"
    );
    assert_eq!(
        outcome.counter_tuple.pointer("/followUpWaveCount"),
        Some(&json!(1)),
        "settled cold-open follow-up must publish exactly one diagnostics wave",
    );
    assert_eq!(
        outcome.actual_notifications, outcome.reference_notifications,
        "settled diagnostics consistency echo must match the separately accumulated terminal reference; the frontier counterfactual above is the falsifiable gate check"
    );
    Ok(())
}

struct IndexSettleSteadyStateOutcome {
    counter_tuple: Value,
    actual_notifications: std::collections::BTreeMap<String, Value>,
    reference_notifications: std::collections::BTreeMap<String, Value>,
    mid_frontier_refresh_jobs: usize,
    frontier_bypass_refresh_jobs: usize,
}

fn run_index_settle_steady_state_fixture()
-> Result<IndexSettleSteadyStateOutcome, Box<dyn std::error::Error>> {
    let fixture_name = index_settle_unique_fixture_name();
    let mut actual = index_settle_fixture_state(fixture_name.as_str())?;
    let mut reference = index_settle_fixture_state(fixture_name.as_str())?;
    let mut bypass = index_settle_fixture_state(fixture_name.as_str())?;

    drain_startup_external_sif_refresh(&mut actual)?;
    drain_startup_external_sif_refresh(&mut reference)?;
    drain_startup_external_sif_refresh(&mut bypass)?;

    reset_index_settle_counter_probes();
    let start = actual.snapshot();

    apply_index_settle_batch(&mut actual, fixture_name.as_str(), 0, true, 1)?;
    let mid_frontier_refresh_jobs =
        usize::from(prepare_deferred_external_sif_refresh_job(&mut actual).is_some());
    assert_eq!(
        crate::diagnostics_follow_up::warmup_wave_count_probe::read(),
        0,
        "pending index frontier must not publish a follow-up wave"
    );

    apply_index_settle_batch(&mut bypass, fixture_name.as_str(), 0, true, 1)?;
    let frontier_bypass_refresh_jobs =
        usize::from(prepare_external_sif_refresh_job_with_open_frontier(&mut bypass).is_some());

    apply_index_settle_batch(&mut actual, fixture_name.as_str(), 1, false, 0)?;
    let actual_job = prepare_deferred_external_sif_refresh_job(&mut actual).ok_or_else(|| {
        std::io::Error::other("settled index frontier should flush the external SIF refresh")
    })?;
    let external_sif_refresh_job_count = 1;
    let actual_result = collect_deferred_external_sif_refresh(actual_job);
    let actual_effects =
        apply_external_sif_refresh_result_follow_up_diagnostics_effects(&mut actual, actual_result);
    assert!(
        !actual_effects.deferred_diagnostics.is_empty() || !actual_effects.outputs.is_empty(),
        "settled follow-up should publish client-visible diagnostics"
    );

    let counter_tuple =
        index_settle_steady_state_counter_tuple(&actual, &start, external_sif_refresh_job_count);
    assert_eq!(
        counter_tuple.pointer("/followUpWaveCount"),
        Some(&json!(1)),
        "accumulated follow-up waves should collapse to one settled wave",
    );
    assert_eq!(
        counter_tuple.pointer("/workspaceIndexPendingFileCount"),
        Some(&json!(0)),
        "fixture must settle the workspace index frontier",
    );
    assert!(
        counter_tuple
            .pointer("/externalSifRefreshJobCount")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            > 0,
        "fixture must exercise the production external-SIF refresh path"
    );
    assert!(
        counter_tuple
            .pointer("/externalSifBridgeGenerationCount")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            > 0,
        "fixture must admit bridge-generated external SIFs"
    );
    assert_eq!(
        counter_tuple.pointer("/committedStyleSemanticGraphComputeCount"),
        Some(&json!(0)),
        "the production settle drain must not build the committed graph on the loop at all — fan-out scoping reads the reverse-dependency memo, and the diagnostics worker owns the single settled-corpus build",
    );
    assert_eq!(
        counter_tuple.pointer("/workspaceCrossFileSummaryDirectRecomputeCount"),
        Some(&json!(0)),
        "settled diagnostics must not fall back to the direct workspace summary API",
    );
    assert_eq!(
        counter_tuple.pointer("/sassModuleResolutionDirectRecomputeCount"),
        Some(&json!(0)),
        "settled diagnostics must not fall back to the direct Sass module resolution API",
    );

    let mut actual_host = omena_query::OmenaQueryStyleMemoHostV0::new();
    omena_query::reset_committed_style_semantic_graph_compute_count_for_test();
    let actual_notifications =
        resolved_deferred_diagnostics_by_uri(&mut actual_host, &actual_effects);
    let diagnostics_worker_graph_count =
        omena_query::read_committed_style_semantic_graph_compute_count_for_test();
    assert!(
        !actual_notifications.is_empty(),
        "settled follow-up should resolve at least one diagnostics notification"
    );
    assert_eq!(
        diagnostics_worker_graph_count, 1,
        "a cold diagnostics worker host should share one committed graph across the settled style publishes",
    );

    let mut reference_host = omena_query::OmenaQueryStyleMemoHostV0::new();
    let mut reference_notifications = std::collections::BTreeMap::new();
    drive_reference_external_sif_refresh_batch(
        &mut reference,
        fixture_name.as_str(),
        0,
        true,
        1,
        &mut reference_host,
        &mut reference_notifications,
    )?;
    drive_reference_external_sif_refresh_batch(
        &mut reference,
        fixture_name.as_str(),
        1,
        false,
        0,
        &mut reference_host,
        &mut reference_notifications,
    )?;

    cleanup_index_settle_fixture(fixture_name.as_str());

    Ok(IndexSettleSteadyStateOutcome {
        counter_tuple,
        actual_notifications,
        reference_notifications,
        mid_frontier_refresh_jobs,
        frontier_bypass_refresh_jobs,
    })
}

fn drive_reference_external_sif_refresh_batch(
    state: &mut LspShellState,
    fixture_name: &str,
    index: usize,
    exhausted: bool,
    pending_file_count: usize,
    host: &mut omena_query::OmenaQueryStyleMemoHostV0,
    notifications: &mut std::collections::BTreeMap<String, Value>,
) -> TestResult {
    apply_index_settle_batch(state, fixture_name, index, exhausted, pending_file_count)?;
    let job = prepare_external_sif_refresh_job_with_open_frontier(state)
        .ok_or_else(|| std::io::Error::other("reference batch should flush independently"))?;
    let result = collect_deferred_external_sif_refresh(job);
    crate::apply_deferred_external_sif_refresh_result(state, result);
    let effects = crate::external_sif_refresh_follow_up_diagnostics_effects(state);
    notifications.extend(resolved_deferred_diagnostics_by_uri(host, &effects));
    Ok(())
}

fn prepare_external_sif_refresh_job_with_open_frontier(
    state: &mut LspShellState,
) -> Option<crate::LspExternalSifRefreshJobV0> {
    let pending_file_count = state.workspace_index_pending_file_count;
    state.workspace_index_pending_file_count = 0;
    let job = prepare_deferred_external_sif_refresh_job(state);
    state.workspace_index_pending_file_count = pending_file_count;
    job
}

fn resolved_deferred_diagnostics_by_uri(
    host: &mut omena_query::OmenaQueryStyleMemoHostV0,
    effects: &crate::LspDiagnosticsFollowUpEffectsV0,
) -> std::collections::BTreeMap<String, Value> {
    effects
        .deferred_diagnostics
        .iter()
        .map(|dispatch| {
            (
                dispatch.uri.clone(),
                crate::resolve_deferred_diagnostics_notification(host, dispatch),
            )
        })
        .collect()
}

fn index_settle_fixture_state(name: &str) -> Result<LspShellState, Box<dyn std::error::Error>> {
    let workspace_root = index_settle_fixture_root(name);
    let src_dir = workspace_root.join("src");
    let external_dir = workspace_root.join("external");
    let _ = std::fs::remove_dir_all(workspace_root.as_path());
    std::fs::create_dir_all(src_dir.as_path())?;
    std::fs::create_dir_all(external_dir.as_path())?;
    std::fs::write(src_dir.join("Base.module.scss"), ".base { color: red; }\n")?;
    std::fs::write(
        src_dir.join("Open.module.scss"),
        ".open { composes: base from \"./Base.module.scss\"; }\n",
    )?;
    std::fs::write(external_dir.join("one.scss"), "$brand-one: #111;\n")?;
    std::fs::write(external_dir.join("two.scss"), "$brand-two: #222;\n")?;

    let workspace_uri = crate::protocol::path_to_file_uri(workspace_root.as_path());
    let base_uri = crate::protocol::path_to_file_uri(src_dir.join("Base.module.scss").as_path());
    let open_uri = crate::protocol::path_to_file_uri(src_dir.join("Open.module.scss").as_path());
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": format!("index-settle-{name}"),
                    },
                ],
            },
        }),
    );
    for (uri, text) in [
        (base_uri.as_str(), ".base { color: red; }\n"),
        (
            open_uri.as_str(),
            ".open { composes: base from \"./Base.module.scss\"; }\n",
        ),
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
    enable_deferred_external_sif_refresh(&mut state);
    Ok(state)
}

fn apply_index_settle_batch(
    state: &mut LspShellState,
    fixture_name: &str,
    index: usize,
    exhausted: bool,
    pending_file_count: usize,
) -> TestResult {
    let workspace_root = index_settle_fixture_root(fixture_name);
    let workspace_uri = crate::protocol::path_to_file_uri(workspace_root.as_path());
    let src_dir = workspace_root.join("src");
    let external_dir = workspace_root.join("external");
    let style_path = src_dir.join(format!("Indexed{index}.module.scss"));
    let style_uri = crate::protocol::path_to_file_uri(style_path.as_path());
    let external_uri = crate::protocol::path_to_file_uri(
        external_dir
            .join(if index == 0 { "one.scss" } else { "two.scss" })
            .as_path(),
    );
    let variable = if index == 0 { "brand-one" } else { "brand-two" };
    let text = format!(
        "@use \"{external_uri}\" as tokens;\n.indexed{index} {{ color: tokens.${variable}; }}\n"
    );
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, Some(workspace_uri.as_str()));
    let result = LspWorkspaceIndexResultV0 {
        revision: state.workspace_index_revision,
        progress_token: None,
        documents: vec![lsp_text_document_state(
            style_uri,
            Some(workspace_uri),
            "scss".to_string(),
            0,
            text,
            &resolution_inputs,
        )],
        pending_file_uris: if pending_file_count == 0 {
            Vec::new()
        } else {
            vec![format!("file:///pending-index-settle-{index}")]
        },
        indexed_count: 1,
        pending_file_count,
        exhausted,
    };
    assert!(apply_background_workspace_index_result(state, result));
    Ok(())
}

fn drain_startup_external_sif_refresh(state: &mut LspShellState) -> TestResult {
    if let Some(job) = prepare_deferred_external_sif_refresh_job(state) {
        let result = collect_deferred_external_sif_refresh(job);
        let _ = apply_external_sif_refresh_result_follow_up_diagnostics_effects(state, result);
    }
    Ok(())
}

fn index_settle_steady_state_counter_tuple(
    state: &LspShellState,
    start: &LspShellStateSnapshot,
    external_sif_refresh_job_count: u64,
) -> Value {
    let snapshot = state.snapshot();
    json!({
        "schemaVersion": "0",
        "product": "omena-lsp-server.index-settle-steady-state-baseline",
        "metric": "cold-open-index-settle-logical-counters",
        "followUpWaveCount": crate::diagnostics_follow_up::warmup_wave_count_probe::read(),
        "committedStyleSemanticGraphComputeCount": omena_query::read_committed_style_semantic_graph_compute_count_for_test(),
        "workspaceCrossFileSummaryDirectRecomputeCount": omena_query::read_workspace_cross_file_summary_direct_recompute_count_for_test(),
        "sassModuleResolutionDirectRecomputeCount": omena_query::read_sass_module_resolution_direct_recompute_count_for_test(),
        "externalSifRefreshJobCount": external_sif_refresh_job_count,
        "tideEpochDelta": snapshot.tide_epoch.saturating_sub(start.tide_epoch),
        "externalSifBridgeGenerationCount": snapshot.external_sif_bridge_generation_count.saturating_sub(start.external_sif_bridge_generation_count),
        "workspaceStyleIndexExhaustedCount": snapshot.workspace_style_index_exhausted_count.saturating_sub(start.workspace_style_index_exhausted_count),
        "workspaceIndexPendingFileCount": snapshot.workspace_index_pending_file_count,
    })
}

fn reset_index_settle_counter_probes() {
    crate::diagnostics_follow_up::warmup_wave_count_probe::reset();
    omena_query::reset_workspace_cross_file_summary_direct_recompute_count_for_test();
    omena_query::reset_sass_module_resolution_direct_recompute_count_for_test();
    omena_query::reset_committed_style_semantic_graph_compute_count_for_test();
}

fn read_index_settle_steady_state_baseline() -> Result<Value, Box<dyn std::error::Error>> {
    let baseline_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("baselines")
        .join("index-settle-steady-state-baseline-v0.json");
    Ok(serde_json::from_str(
        std::fs::read_to_string(baseline_path)?.as_str(),
    )?)
}

fn index_settle_fixture_root(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "omena-lsp-server-index-settle-{name}-{}",
        std::process::id()
    ))
}

fn index_settle_unique_fixture_name() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("shared-{nanos}-{:?}", std::thread::current().id())
        .replace(['(', ')'], "-")
        .replace(' ', "-")
}

fn cleanup_index_settle_fixture(name: &str) {
    let _ = std::fs::remove_dir_all(index_settle_fixture_root(name).as_path());
}

#[test]
fn background_workspace_index_prioritizes_candidates_near_open_documents() -> TestResult {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-background-proximity-{}",
        std::process::id()
    ));
    let far_dir = workspace_root.join("aaa");
    let near_dir = workspace_root.join("zzz");
    let open_source_path = near_dir.join("App.tsx");
    let near_style_path = near_dir.join("Near.module.scss");
    let _ = std::fs::remove_dir_all(&workspace_root);
    std::fs::create_dir_all(&far_dir)?;
    std::fs::create_dir_all(&near_dir)?;
    for index in 0..520 {
        std::fs::write(
            far_dir.join(format!("Style{index:04}.module.scss")),
            format!(".far{index} {{ color: red; }}"),
        )?;
    }
    std::fs::write(
        open_source_path.as_path(),
        "import styles from \"./Near.module.scss\";\nconst view = <div className={styles.near} />;",
    )?;
    std::fs::write(near_style_path.as_path(), ".near { color: blue; }")?;

    let workspace_uri = crate::protocol::path_to_file_uri(workspace_root.as_path());
    let open_source_uri = crate::protocol::path_to_file_uri(open_source_path.as_path());
    let near_style_uri = crate::protocol::path_to_file_uri(near_style_path.as_path());
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "background-proximity",
                    },
                ],
            },
        }),
    );
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": open_source_uri,
                    "languageId": "typescriptreact",
                    "version": 1,
                    "text": "import styles from \"./Near.module.scss\";\nconst view = <div className={styles.near} />;",
                },
            },
        }),
    );

    let job = prepare_background_workspace_index_job(&mut state);
    let result = collect_background_workspace_index(job);
    assert!(
        result.exhausted,
        "fixture should exceed the per-tick file budget"
    );
    assert!(
        result
            .documents
            .iter()
            .any(|document| file_uri_equivalent(document.uri.as_str(), near_style_uri.as_str())),
        "first background batch should include the style candidate near the open source document"
    );

    let _ = std::fs::remove_dir_all(&workspace_root);
    Ok(())
}

#[test]
fn background_workspace_index_reaches_sources_past_dir_budget() -> TestResult {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-background-dir-frontier-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&workspace_root);
    std::fs::create_dir_all(workspace_root.as_path())?;
    let style_path = workspace_root.join("Button.module.scss");
    let late_dir = workspace_root.join("ZTargetDir");
    let late_source_path = late_dir.join("Target.tsx");
    std::fs::write(&style_path, ".root { color: red; }")?;
    for index in 0..2050 {
        std::fs::create_dir_all(workspace_root.join(format!("A{index:04}")))?;
    }
    std::fs::create_dir_all(late_dir.as_path())?;
    std::fs::write(
        &late_source_path,
        "import styles from \"../Button.module.scss\";\nconst view = <div className={styles.root} />;",
    )?;

    let workspace_uri = crate::protocol::path_to_file_uri(workspace_root.as_path());
    let style_uri = crate::protocol::path_to_file_uri(style_path.as_path());
    let late_source_uri = crate::protocol::path_to_file_uri(late_source_path.as_path());
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "background-dir-frontier",
                    },
                ],
            },
        }),
    );
    let turn = handle_lsp_message_scheduled_outputs_or_dispatch(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {},
        }),
    );
    let job = match turn {
        LspLoopTurnV0::OutputsAndDeferredDiagnostics {
            mut workspace_index_jobs,
            ..
        } => workspace_index_jobs
            .pop()
            .ok_or_else(|| std::io::Error::other("missing workspace index job"))?,
        other => {
            return Err(std::io::Error::other(format!(
                "initialized should schedule workspace indexing: {other:?}"
            ))
            .into());
        }
    };
    let result = collect_background_workspace_index(job);
    apply_background_workspace_index_result(&mut state, result);

    assert!(
        state.document(late_source_uri.as_str()).is_some(),
        "background workspace indexing should reach sources beyond the former dir-budget frontier"
    );
    let references_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/references",
            "params": {
                "textDocument": {
                    "uri": style_uri,
                },
                "position": {
                    "line": 0,
                    "character": 2,
                },
                "context": {
                    "includeDeclaration": false,
                },
            },
        }),
    );
    let reference_locations = references_response
        .as_ref()
        .and_then(|response| response.pointer("/result"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("references response should contain locations"))?;
    assert!(
        reference_locations.iter().any(|location| location
            .get("uri")
            .and_then(Value::as_str)
            .is_some_and(|uri| file_uri_equivalent(uri, late_source_uri.as_str()))),
        "dir-frontier source occurrence should appear in references: {references_response:?}"
    );
    let _ = std::fs::remove_dir_all(&workspace_root);
    Ok(())
}

#[test]
fn watched_source_file_change_refreshes_indexed_occurrences() -> TestResult {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-watched-source-occurrences-{}",
        std::process::id()
    ));
    let src_dir = workspace_root.join("src");
    let source_path = src_dir.join("App.tsx");
    let style_path = src_dir.join("Button.module.scss");
    let _ = std::fs::remove_dir_all(&workspace_root);
    std::fs::create_dir_all(&src_dir)?;
    std::fs::write(
        &style_path,
        ".root { color: red; }\n.other { color: blue; }",
    )?;
    std::fs::write(
        &source_path,
        "import styles from \"./Button.module.scss\";\nconst view = <div className={styles.root} />;",
    )?;

    let workspace_uri = crate::protocol::path_to_file_uri(workspace_root.as_path());
    let source_uri = crate::protocol::path_to_file_uri(source_path.as_path());
    let style_uri = crate::protocol::path_to_file_uri(style_path.as_path());
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "watched-source-occurrences",
                    },
                ],
            },
        }),
    );
    let turn = handle_lsp_message_scheduled_outputs_or_dispatch(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {},
        }),
    );
    let job = match turn {
        LspLoopTurnV0::OutputsAndDeferredDiagnostics {
            mut workspace_index_jobs,
            ..
        } => workspace_index_jobs
            .pop()
            .ok_or_else(|| std::io::Error::other("missing workspace index job"))?,
        other => {
            return Err(std::io::Error::other(format!(
                "initialized should schedule workspace indexing: {other:?}"
            ))
            .into());
        }
    };
    let result = collect_background_workspace_index(job);
    apply_background_workspace_index_result(&mut state, result);

    let first_references = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/references",
            "params": {
                "textDocument": {
                    "uri": style_uri,
                },
                "position": {
                    "line": 0,
                    "character": 2,
                },
                "context": {
                    "includeDeclaration": false,
                },
            },
        }),
    );
    assert!(
        first_references
            .as_ref()
            .and_then(|response| response.pointer("/result"))
            .and_then(Value::as_array)
            .is_some_and(|locations| locations.iter().any(|location| location
                .get("uri")
                .and_then(Value::as_str)
                .is_some_and(|uri| file_uri_equivalent(uri, source_uri.as_str())))),
        "initial indexed source reference should be visible: {first_references:?}"
    );

    std::fs::write(
        &source_path,
        "import styles from \"./Button.module.scss\";\nconst view = <div className={styles.other} />;",
    )?;
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeWatchedFiles",
            "params": {
                "changes": [
                    {
                        "uri": source_uri,
                        "type": 2,
                    },
                ],
            },
        }),
    );

    let refreshed_references = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/references",
            "params": {
                "textDocument": {
                    "uri": style_uri,
                },
                "position": {
                    "line": 0,
                    "character": 2,
                },
                "context": {
                    "includeDeclaration": false,
                },
            },
        }),
    );
    assert!(
        refreshed_references
            .as_ref()
            .and_then(|response| response.pointer("/result"))
            .and_then(Value::as_array)
            .is_some_and(|locations| locations.iter().all(|location| location
                .get("uri")
                .and_then(Value::as_str)
                .is_none_or(|uri| !file_uri_equivalent(uri, source_uri.as_str())))),
        "watched source change should remove stale root reference: {refreshed_references:?}"
    );

    let _ = std::fs::remove_dir_all(&workspace_root);
    Ok(())
}

#[test]
fn indexed_source_files_do_not_receive_style_change_diagnostics_until_open() -> TestResult {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-source-publish-bound-{}",
        std::process::id()
    ));
    let src_dir = workspace_root.join("src");
    let source_path = src_dir.join("App.tsx");
    let style_path = src_dir.join("Button.module.scss");
    let package_json_path = workspace_root.join("package.json");
    let _ = std::fs::remove_dir_all(&workspace_root);
    std::fs::create_dir_all(&src_dir)?;
    let source_text = "import styles from \"./Button.module.scss\";\nconst view = <div className={styles.root} />;";
    std::fs::write(&style_path, ".root { color: red; }")?;
    std::fs::write(&source_path, source_text)?;
    std::fs::write(&package_json_path, r#"{"name":"source-publish-bound"}"#)?;

    let workspace_uri = crate::protocol::path_to_file_uri(workspace_root.as_path());
    let source_uri = crate::protocol::path_to_file_uri(source_path.as_path());
    let style_uri = crate::protocol::path_to_file_uri(style_path.as_path());
    let package_json_uri = crate::protocol::path_to_file_uri(package_json_path.as_path());
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "source-publish-bound",
                    },
                ],
            },
        }),
    );
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {},
        }),
    );
    assert!(state.document(source_uri.as_str()).is_some());
    assert!(!state.has_open_document_uri(source_uri.as_str()));

    let config_change_outputs = handle_lsp_message_outputs(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeWatchedFiles",
            "params": {
                "changes": [
                    {
                        "uri": package_json_uri,
                        "type": 2,
                    },
                ],
            },
        }),
    );
    let published_after_config_change =
        published_diagnostics_uris(config_change_outputs.as_slice());
    assert!(
        !published_after_config_change.contains(&source_uri),
        "never-opened indexed source documents must not receive publishDiagnostics after config changes: {published_after_config_change:?}"
    );
    assert!(
        published_after_config_change.contains(&style_uri),
        "the indexed style payload must be delivered before the duplicate-open check: {published_after_config_change:?}"
    );

    let open_style_outputs = handle_lsp_message_outputs(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": style_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": ".root { color: blue; }",
                },
            },
        }),
    );
    let published_uris = published_diagnostics_uris(open_style_outputs.as_slice());
    assert!(
        !published_uris.contains(&style_uri),
        "style open must not redeliver diagnostics already published by the config refresh: {published_uris:?}"
    );
    assert!(
        !published_uris.contains(&source_uri),
        "never-opened indexed source documents must not receive publishDiagnostics: {published_uris:?}"
    );

    handle_lsp_message_outputs(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                    "languageId": "typescriptreact",
                    "version": 1,
                    "text": source_text,
                },
            },
        }),
    );
    let changed_style_outputs = handle_lsp_message_outputs(
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
                        "text": ".renamed { color: green; }",
                    },
                ],
            },
        }),
    );
    let published_after_open = published_diagnostics_uris(changed_style_outputs.as_slice());
    assert!(
        published_after_open.contains(&source_uri),
        "open source documents should still be republished after their referenced style changes: {published_after_open:?}"
    );
    let _ = std::fs::remove_dir_all(&workspace_root);
    Ok(())
}

#[test]
fn indexed_source_diagnostics_use_persisted_source_syntax_without_provider_candidates() -> TestResult
{
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-source-diagnostics-indexed-{}",
        std::process::id()
    ));
    let src_dir = workspace_root.join("src");
    let source_path = src_dir.join("App.tsx");
    let style_path = src_dir.join("Button.module.scss");
    let _ = std::fs::remove_dir_all(&workspace_root);
    std::fs::create_dir_all(&src_dir)?;
    let source_text = concat!(
        "const view = styles.ghost;\n",
        "const resolved = primary;\n",
        "const prefix = lost;\n",
        "const domain = empty;\n",
    );
    let ghost_start = fixture_find(
        source_text,
        "ghost",
        "source fixture contains static selector reference",
    )?;
    let primary_start = fixture_find(
        source_text,
        "primary",
        "source fixture contains resolved class value reference",
    )?;
    let lost_start = fixture_find(
        source_text,
        "lost",
        "source fixture contains template prefix reference",
    )?;
    let empty_start = fixture_find(
        source_text,
        "empty",
        "source fixture contains resolved domain reference",
    )?;
    std::fs::write(&source_path, source_text)?;
    std::fs::write(&style_path, ".root { color: red; }")?;

    let workspace_uri = crate::protocol::path_to_file_uri(workspace_root.as_path());
    let source_uri = crate::protocol::path_to_file_uri(source_path.as_path());
    let style_uri = crate::protocol::path_to_file_uri(style_path.as_path());
    let resolution_inputs =
        load_lsp_workspace_style_resolution_inputs(Some(workspace_uri.as_str()), &[]);
    let cached_index = SourceSyntaxIndex {
        schema_version: "0",
        product: "omena-bridge.source-syntax-index",
        imported_style_bindings: vec![ImportedStyleBinding {
            binding: "styles".to_string(),
            style_uri: style_uri.clone(),
        }],
        class_string_literals: Vec::new(),
        style_property_accesses: Vec::new(),
        inline_style_declarations: Vec::new(),
        selector_references: vec![
            SourceSelectorReferenceFact {
                byte_span: ParserByteSpanV0 {
                    start: ghost_start,
                    end: ghost_start + "ghost".len(),
                },
                selector_name: Some("ghost".to_string()),
                match_kind: SourceSelectorReferenceMatchKind::Exact,
                target_style_uri: Some(style_uri.clone()),
                surface: SourceSelectorReferenceSurface::OmenaQuerySourceSyntaxIndex,
            },
            SourceSelectorReferenceFact {
                byte_span: ParserByteSpanV0 {
                    start: primary_start,
                    end: primary_start + "primary".len(),
                },
                selector_name: Some("buttonPrimary".to_string()),
                match_kind: SourceSelectorReferenceMatchKind::Exact,
                target_style_uri: Some(style_uri.clone()),
                surface: SourceSelectorReferenceSurface::OmenaQuerySourceSyntaxIndex,
            },
            SourceSelectorReferenceFact {
                byte_span: ParserByteSpanV0 {
                    start: lost_start,
                    end: lost_start + "lost".len(),
                },
                selector_name: Some("lost".to_string()),
                match_kind: SourceSelectorReferenceMatchKind::Prefix,
                target_style_uri: Some(style_uri.clone()),
                surface: SourceSelectorReferenceSurface::OmenaQuerySourceSyntaxIndex,
            },
            SourceSelectorReferenceFact {
                byte_span: ParserByteSpanV0 {
                    start: empty_start,
                    end: empty_start + "empty".len(),
                },
                selector_name: Some("emptyGhost".to_string()),
                match_kind: SourceSelectorReferenceMatchKind::Prefix,
                target_style_uri: Some(style_uri.clone()),
                surface: SourceSelectorReferenceSurface::OmenaQuerySourceSyntaxIndex,
            },
        ],
        type_fact_targets: Vec::new(),
        type_fact_target_skipped: Vec::new(),
        type_fact_target_skipped_count: 0,
        type_fact_provider_unavailable: Vec::new(),
        class_value_universes: Vec::new(),
        domain_class_references: Vec::new(),
        source_elements: Vec::new(),
        element_parent_edges: Vec::new(),
    };
    let text_hash = crate::source_document_cache::source_document_text_hash(source_text);
    let cache_storage = crate::cache_root::LspCacheStorageConfigV0::default();
    crate::source_document_cache::store_source_document_index_sidecar(
        &cache_storage,
        Some(workspace_uri.as_str()),
        source_uri.as_str(),
        "typescriptreact",
        text_hash.as_str(),
        &resolution_inputs,
        &cached_index,
        &[],
        false,
    );

    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "source-diagnostics-indexed",
                    },
                ],
            },
        }),
    );
    let turn = handle_lsp_message_scheduled_outputs_or_dispatch(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {},
        }),
    );
    let workspace_index_job = match turn {
        LspLoopTurnV0::OutputsAndDeferredDiagnostics {
            mut workspace_index_jobs,
            ..
        } => workspace_index_jobs
            .pop()
            .ok_or_else(|| std::io::Error::other("missing workspace index job"))?,
        other => {
            return Err(std::io::Error::other(format!(
                "initialized should schedule background workspace indexing: {other:?}"
            ))
            .into());
        }
    };
    let result = collect_background_workspace_index(workspace_index_job);
    apply_background_workspace_index_result(&mut state, result);
    state
        .document_mut(source_uri.as_str())
        .ok_or_else(|| std::io::Error::other("source sidecar should index source document"))?
        .source_selector_candidates
        .clear();

    let diagnostics = resolve_source_diagnostics_for_uri(&state, source_uri.as_str());
    let diagnostics_items = diagnostics
        .as_array()
        .ok_or_else(|| std::io::Error::other("source diagnostics should be an array"))?;
    for code in [
        "missingStaticClass",
        "missingResolvedClassValues",
        "missingTemplatePrefix",
        "missingResolvedClassDomain",
    ] {
        assert!(
            diagnostics_items
                .iter()
                .any(|diagnostic| diagnostic.get("code") == Some(&json!(code))
                    && diagnostic
                        .pointer("/data/provenance")
                        .and_then(Value::as_array)
                        .is_some_and(|provenance| provenance
                            .iter()
                            .any(|item| item == "omena-query.source-syntax-index"))),
            "source diagnostics should consume the persisted source syntax index for {code} without provider candidates: {diagnostics:?}"
        );
    }

    let _ = std::fs::remove_dir_all(&workspace_root);
    Ok(())
}

#[test]
fn persisted_source_syntax_sidecar_feeds_unused_selector_diagnostics_without_reparse() -> TestResult
{
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-unused-selector-sidecar-{}",
        std::process::id()
    ));
    let src_dir = workspace_root.join("src");
    let source_path = src_dir.join("App.tsx");
    let style_path = src_dir.join("Button.module.scss");
    let _ = std::fs::remove_dir_all(&workspace_root);
    std::fs::create_dir_all(&src_dir)?;
    let source_text = "const view = null;";
    std::fs::write(&source_path, source_text)?;
    std::fs::write(
        &style_path,
        ".cachedRoot { color: red; }\n.orphan { color: blue; }",
    )?;

    let workspace_uri = crate::protocol::path_to_file_uri(workspace_root.as_path());
    let source_uri = crate::protocol::path_to_file_uri(source_path.as_path());
    let style_uri = crate::protocol::path_to_file_uri(style_path.as_path());
    let resolution_inputs =
        load_lsp_workspace_style_resolution_inputs(Some(workspace_uri.as_str()), &[]);
    let cached_index = SourceSyntaxIndex {
        schema_version: "0",
        product: "omena-bridge.source-syntax-index",
        imported_style_bindings: vec![ImportedStyleBinding {
            binding: "styles".to_string(),
            style_uri: style_uri.clone(),
        }],
        class_string_literals: Vec::new(),
        style_property_accesses: Vec::new(),
        inline_style_declarations: Vec::new(),
        selector_references: vec![SourceSelectorReferenceFact {
            byte_span: ParserByteSpanV0 { start: 0, end: 0 },
            selector_name: Some("cachedRoot".to_string()),
            match_kind: SourceSelectorReferenceMatchKind::Exact,
            target_style_uri: Some(style_uri.clone()),
            surface: SourceSelectorReferenceSurface::OmenaQuerySourceSyntaxIndex,
        }],
        type_fact_targets: Vec::new(),
        type_fact_target_skipped: Vec::new(),
        type_fact_target_skipped_count: 0,
        type_fact_provider_unavailable: Vec::new(),
        class_value_universes: Vec::new(),
        domain_class_references: Vec::new(),
        source_elements: Vec::new(),
        element_parent_edges: Vec::new(),
    };
    let text_hash = crate::source_document_cache::source_document_text_hash(source_text);
    let cache_storage = crate::cache_root::LspCacheStorageConfigV0::default();
    crate::source_document_cache::store_source_document_index_sidecar(
        &cache_storage,
        Some(workspace_uri.as_str()),
        source_uri.as_str(),
        "typescriptreact",
        text_hash.as_str(),
        &resolution_inputs,
        &cached_index,
        &[],
        false,
    );

    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "unused-selector-sidecar",
                    },
                ],
            },
        }),
    );
    let turn = handle_lsp_message_scheduled_outputs_or_dispatch(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {},
        }),
    );
    let workspace_index_job = match turn {
        LspLoopTurnV0::OutputsAndDeferredDiagnostics {
            mut workspace_index_jobs,
            ..
        } => workspace_index_jobs
            .pop()
            .ok_or_else(|| std::io::Error::other("missing workspace index job"))?,
        other => {
            return Err(std::io::Error::other(format!(
                "initialized should schedule background workspace indexing: {other:?}"
            ))
            .into());
        }
    };
    let result = collect_background_workspace_index(workspace_index_job);
    apply_background_workspace_index_result(&mut state, result);
    assert!(
        !state.has_open_document_uri(source_uri.as_str()),
        "background-indexed source documents must not become open buffers"
    );

    let diagnostics = resolve_style_diagnostics_for_uri(&state, style_uri.as_str());
    let empty = Vec::new();
    let unused_messages = diagnostics
        .as_array()
        .unwrap_or(&empty)
        .iter()
        .filter(|diagnostic| diagnostic.get("code") == Some(&json!("unusedSelector")))
        .filter_map(|diagnostic| diagnostic.get("message").and_then(Value::as_str))
        .collect::<Vec<_>>();
    assert!(
        !unused_messages
            .iter()
            .any(|message| message.contains("'.cachedRoot'")),
        "persisted source syntax sidecar should mark .cachedRoot as referenced without reparsing source text: {unused_messages:?}"
    );
    assert!(
        unused_messages
            .iter()
            .any(|message| message.contains("'.orphan'")),
        "unused selector diagnostics should still report genuinely unused selectors: {unused_messages:?}"
    );

    let _ = std::fs::remove_dir_all(&workspace_root);
    Ok(())
}

#[test]
fn indexed_style_files_feed_custom_property_references_and_rename() -> TestResult {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-style-symbol-custom-property-{}",
        std::process::id()
    ));
    let src_dir = workspace_root.join("src");
    let app_path = src_dir.join("App.module.scss");
    let tokens_path = src_dir.join("tokens.scss");
    let _ = std::fs::remove_dir_all(&workspace_root);
    std::fs::create_dir_all(&src_dir)?;
    let app_text = ".root { color: var(--brand); }\n";
    let tokens_text = ":root { --brand: red; }\n";
    std::fs::write(&app_path, app_text)?;
    std::fs::write(&tokens_path, tokens_text)?;

    let workspace_uri = path_to_file_uri(workspace_root.as_path());
    let app_uri = path_to_file_uri(app_path.as_path());
    let tokens_uri = path_to_file_uri(tokens_path.as_path());
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "style-symbol-custom-property",
                    },
                ],
            },
        }),
    );
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {},
        }),
    );

    let reference_position = parser_position_for_byte_offset(
        app_text,
        fixture_find(
            app_text,
            "--brand",
            "app style contains custom property reference",
        )?,
    );
    let references_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/references",
            "params": {
                "textDocument": {
                    "uri": app_uri,
                },
                "position": reference_position,
                "context": {
                    "includeDeclaration": true,
                },
            },
        }),
    );
    let reference_locations = references_response
        .as_ref()
        .and_then(|response| response.pointer("/result"))
        .and_then(Value::as_array)
        .ok_or_else(|| {
            std::io::Error::other("custom property references should return locations")
        })?;
    assert!(
        reference_locations.iter().any(|location| location
            .get("uri")
            .and_then(Value::as_str)
            .is_some_and(|uri| file_uri_equivalent(uri, app_uri.as_str()))),
        "indexed custom property references should include the referencing style: {references_response:?}"
    );
    assert!(
        reference_locations.iter().any(|location| location
            .get("uri")
            .and_then(Value::as_str)
            .is_some_and(|uri| file_uri_equivalent(uri, tokens_uri.as_str()))),
        "indexed custom property references should include the declaring style: {references_response:?}"
    );

    let definition_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": app_uri,
                },
                "position": reference_position,
            },
        }),
    );
    assert!(
        definition_response
            .as_ref()
            .and_then(|response| response.pointer("/result"))
            .and_then(Value::as_array)
            .is_some_and(|locations| locations.iter().any(|location| location
                .get("uri")
                .and_then(Value::as_str)
                .is_some_and(|uri| file_uri_equivalent(uri, tokens_uri.as_str())))),
        "custom property definition should resolve through the indexed style-symbol occurrence index: {definition_response:?}"
    );
    assert!(
        state.workspace_occurrence_index_memo_lock().is_some(),
        "custom property definition should populate the workspace occurrence memo"
    );
    *state.workspace_occurrence_index_memo_lock() = None;
    state
        .document_mut(tokens_uri.as_str())
        .ok_or_else(|| std::io::Error::other("tokens style should remain indexed"))?
        .style_candidates
        .clear();
    let cached_definition_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": app_uri,
                },
                "position": reference_position,
            },
        }),
    );
    assert!(
        cached_definition_response
            .as_ref()
            .and_then(|response| response.pointer("/result"))
            .and_then(Value::as_array)
            .is_some_and(|locations| locations.iter().any(|location| location
                .get("uri")
                .and_then(Value::as_str)
                .is_some_and(|uri| file_uri_equivalent(uri, tokens_uri.as_str())))),
        "workspace occurrence shards should rehydrate custom property definitions without rescanning the declaring style candidates: {cached_definition_response:?}"
    );

    let rename_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "textDocument/rename",
            "params": {
                "textDocument": {
                    "uri": app_uri,
                },
                "position": reference_position,
                "newName": "--accent",
            },
        }),
    );
    let changes = rename_response
        .as_ref()
        .and_then(|response| response.pointer("/result/changes"))
        .and_then(Value::as_object)
        .ok_or_else(|| std::io::Error::other("custom property rename should return changes"))?;
    assert!(
        changes
            .keys()
            .any(|uri| file_uri_equivalent(uri.as_str(), app_uri.as_str())),
        "custom property rename should edit the referencing style: {rename_response:?}"
    );
    assert!(
        changes
            .keys()
            .any(|uri| file_uri_equivalent(uri.as_str(), tokens_uri.as_str())),
        "custom property rename should edit the declaring style: {rename_response:?}"
    );

    let _ = std::fs::remove_dir_all(&workspace_root);
    Ok(())
}

#[test]
fn indexed_style_files_feed_sass_symbol_references_and_rename() -> TestResult {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-style-symbol-sass-{}",
        std::process::id()
    ));
    let src_dir = workspace_root.join("src");
    let app_path = src_dir.join("App.module.scss");
    let other_path = src_dir.join("Other.module.scss");
    let tokens_path = src_dir.join("_tokens.scss");
    let _ = std::fs::remove_dir_all(&workspace_root);
    std::fs::create_dir_all(&src_dir)?;
    let app_text = "@use \"./tokens\" as *;\n.root { color: $brand; }\n";
    let other_text = "@use \"./tokens\" as *;\n.other { background: $brand; }\n";
    let tokens_text = "$brand: red;\n";
    std::fs::write(&app_path, app_text)?;
    std::fs::write(&other_path, other_text)?;
    std::fs::write(&tokens_path, tokens_text)?;

    let workspace_uri = path_to_file_uri(workspace_root.as_path());
    let app_uri = path_to_file_uri(app_path.as_path());
    let other_uri = path_to_file_uri(other_path.as_path());
    let tokens_uri = path_to_file_uri(tokens_path.as_path());
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "style-symbol-sass",
                    },
                ],
            },
        }),
    );
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {},
        }),
    );

    let declaration_position = parser_position_for_byte_offset(
        tokens_text,
        fixture_find(
            tokens_text,
            "$brand",
            "tokens style contains Sass variable declaration",
        )? + 1,
    );
    let references_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/references",
            "params": {
                "textDocument": {
                    "uri": tokens_uri,
                },
                "position": declaration_position,
                "context": {
                    "includeDeclaration": true,
                },
            },
        }),
    );
    let reference_locations = references_response
        .as_ref()
        .and_then(|response| response.pointer("/result"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("Sass references should return locations"))?;
    assert!(
        reference_locations.iter().any(|location| location
            .get("uri")
            .and_then(Value::as_str)
            .is_some_and(|uri| file_uri_equivalent(uri, tokens_uri.as_str()))),
        "Sass references should include the declaration style: {references_response:?}"
    );
    assert!(
        reference_locations.iter().any(|location| location
            .get("uri")
            .and_then(Value::as_str)
            .is_some_and(|uri| file_uri_equivalent(uri, app_uri.as_str()))),
        "Sass references should include the first indexed consumer style: {references_response:?}"
    );
    assert!(
        reference_locations.iter().any(|location| location
            .get("uri")
            .and_then(Value::as_str)
            .is_some_and(|uri| file_uri_equivalent(uri, other_uri.as_str()))),
        "Sass references should include the second indexed consumer style: {references_response:?}"
    );
    assert!(
        state.workspace_occurrence_index_memo_lock().is_some(),
        "Sass references should populate the workspace occurrence memo"
    );
    *state.workspace_occurrence_index_memo_lock() = None;
    state
        .document_mut(app_uri.as_str())
        .ok_or_else(|| std::io::Error::other("app style should remain indexed"))?
        .style_candidates
        .clear();
    state
        .document_mut(other_uri.as_str())
        .ok_or_else(|| std::io::Error::other("other style should remain indexed"))?
        .style_candidates
        .clear();
    let cached_references_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "textDocument/references",
            "params": {
                "textDocument": {
                    "uri": tokens_uri,
                },
                "position": declaration_position,
                "context": {
                    "includeDeclaration": true,
                },
            },
        }),
    );
    let cached_reference_locations = cached_references_response
        .as_ref()
        .and_then(|response| response.pointer("/result"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("cached Sass references should return locations"))?;
    assert!(
        cached_reference_locations.iter().any(|location| location
            .get("uri")
            .and_then(Value::as_str)
            .is_some_and(|uri| file_uri_equivalent(uri, app_uri.as_str()))),
        "workspace occurrence shards should rehydrate the first Sass consumer without rescanning style candidates: {cached_references_response:?}"
    );
    assert!(
        cached_reference_locations.iter().any(|location| location
            .get("uri")
            .and_then(Value::as_str)
            .is_some_and(|uri| file_uri_equivalent(uri, other_uri.as_str()))),
        "workspace occurrence shards should rehydrate the second Sass consumer without rescanning style candidates: {cached_references_response:?}"
    );

    let rename_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/rename",
            "params": {
                "textDocument": {
                    "uri": tokens_uri,
                },
                "position": declaration_position,
                "newName": "accent",
            },
        }),
    );
    let changes = rename_response
        .as_ref()
        .and_then(|response| response.pointer("/result/changes"))
        .and_then(Value::as_object)
        .ok_or_else(|| std::io::Error::other("Sass rename should return changes"))?;
    assert!(
        changes
            .keys()
            .any(|uri| file_uri_equivalent(uri.as_str(), tokens_uri.as_str())),
        "Sass rename should edit the declaration style: {rename_response:?}"
    );
    assert!(
        changes
            .keys()
            .any(|uri| file_uri_equivalent(uri.as_str(), app_uri.as_str())),
        "Sass rename should edit the first indexed consumer style: {rename_response:?}"
    );
    assert!(
        changes
            .keys()
            .any(|uri| file_uri_equivalent(uri.as_str(), other_uri.as_str())),
        "Sass rename should edit the second indexed consumer style: {rename_response:?}"
    );

    let _ = std::fs::remove_dir_all(&workspace_root);
    Ok(())
}

#[test]
fn indexes_workspace_style_files_from_dist_artifacts() {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-dist-index-{}",
        std::process::id()
    ));
    let dist_dir = workspace_root.join("dist");
    let style_path = dist_dir.join("Theme.module.scss");
    let _ = std::fs::remove_dir_all(&workspace_root);
    let create_dir_result = std::fs::create_dir_all(&dist_dir);
    assert!(
        create_dir_result.is_ok(),
        "create dist-index fixture directory: {:?}",
        create_dir_result.err(),
    );
    let write_result = std::fs::write(&style_path, ".fromDist { color: red; }");
    assert!(
        write_result.is_ok(),
        "write dist-index style fixture: {:?}",
        write_result.err(),
    );

    let workspace_uri = format!("file://{}", workspace_root.display());
    let style_uri = format!("file://{}", style_path.display());
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "dist-index",
                    },
                ],
            },
        }),
    );
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {},
        }),
    );

    let indexed = state
        .document(style_uri.as_str())
        .and_then(|document| document.style_summary.as_ref());
    assert_eq!(
        indexed.map(|summary| summary.selector_names.clone()),
        Some(vec!["fromDist".to_string()]),
    );
    let _ = std::fs::remove_dir_all(&workspace_root);
}

fn published_diagnostics_uris(outputs: &[Value]) -> Vec<String> {
    outputs
        .iter()
        .filter_map(|output| {
            if output.get("method") == Some(&json!("textDocument/publishDiagnostics")) {
                output
                    .pointer("/params/uri")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn bounds_workspace_style_indexing_by_budget() {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-index-budget-{}",
        std::process::id()
    ));
    let src_dir = workspace_root.join("src");
    let style_path = src_dir.join("Budget.module.scss");
    let _ = std::fs::remove_dir_all(&workspace_root);
    let create_dir_result = std::fs::create_dir_all(&src_dir);
    assert!(
        create_dir_result.is_ok(),
        "create index-budget fixture directory: {:?}",
        create_dir_result.err(),
    );
    let write_result = std::fs::write(&style_path, ".budget { color: red; }");
    assert!(
        write_result.is_ok(),
        "write index-budget style fixture: {:?}",
        write_result.err(),
    );

    let workspace_uri = format!("file://{}", workspace_root.display());
    let style_uri = format!("file://{}", style_path.display());
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "index-budget",
                    },
                ],
            },
        }),
    );
    let mut budget = WorkspaceStyleIndexBudget::with_limits(1, 1, 0);
    index_workspace_style_files_with_budget(&mut state, &mut budget);

    assert!(state.document(style_uri.as_str()).is_none());
    assert_eq!(state.snapshot().workspace_style_index_exhausted_count, 1);
    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn workspace_occurrence_shards_follow_each_document_read_set() -> TestResult {
    omena_testkit::with_instrumentation_session(
        omena_testkit::InstrumentationSessionV0::default(),
        || {
            let WorkspaceOccurrenceReadSetFixtureV0 {
                mut state,
                workspace_root,
                workspace_uri,
                app_uri,
                tokens_uri,
                unrelated_uri,
                ..
            } = workspace_occurrence_read_set_fixture("read-set")?;

            let cold_started = std::time::Instant::now();
            crate::workspace_occurrences::workspace_occurrence_indexes_from_documents(
                &state,
                Some(workspace_uri.as_str()),
            );
            let cold_elapsed = cold_started.elapsed();

            crate::style_symbol_provider::reset_workspace_occurrence_extractor_counters_for_test();
            let warm_started = std::time::Instant::now();
            crate::workspace_occurrences::workspace_occurrence_indexes_from_documents(
                &state,
                Some(workspace_uri.as_str()),
            );
            let warm_elapsed = warm_started.elapsed();
            assert_eq!(
                crate::style_symbol_provider::workspace_occurrence_extractor_rebuild_count_for_test(
                    app_uri.as_str(),
                ),
                0,
                "an unchanged aggregate memo hit must not rebuild A"
            );

            change_occurrence_style_document(
                &mut state,
                unrelated_uri.as_str(),
                2,
                ".unrelated { color: green; }\n",
            );
            crate::style_symbol_provider::reset_workspace_occurrence_extractor_counters_for_test();
            crate::workspace_occurrence_cache::reset_workspace_occurrence_shard_read_counts_for_test();
            crate::workspace_occurrences::reset_workspace_occurrence_memo_hit_counts_for_test();
            let unrelated_started = std::time::Instant::now();
            crate::workspace_occurrences::workspace_occurrence_indexes_from_documents(
                &state,
                Some(workspace_uri.as_str()),
            );
            let unrelated_elapsed = unrelated_started.elapsed();
            assert_eq!(
                crate::style_symbol_provider::workspace_occurrence_extractor_rebuild_count_for_test(
                    app_uri.as_str(),
                ),
                0,
                "editing unrelated B must not rebuild A"
            );
            assert_eq!(
                crate::workspace_occurrences::workspace_occurrence_memo_hit_count_for_test(
                    app_uri.as_str(),
                ),
                1,
                "editing unrelated B must serve A from the narrowed RAM memo"
            );
            assert_eq!(
                crate::workspace_occurrence_cache::workspace_occurrence_shard_read_count_for_test(
                    app_uri.as_str(),
                ),
                0,
                "an unrelated edit must not read A's disk shard"
            );
            assert_eq!(
                crate::style_symbol_provider::workspace_occurrence_read_set_recomputation_count_for_test(
                    app_uri.as_str(),
                ),
                0,
                "an unrelated edit must not recompute A's read set"
            );

            let later_uri =
                path_to_file_uri(workspace_root.join("src/Later.module.scss").as_path());
            std::fs::write(
                workspace_root.join("src/Later.module.scss"),
                ".later { color: purple; }\n",
            )?;
            open_occurrence_style_document(
                &mut state,
                later_uri.as_str(),
                ".later { color: purple; }\n",
            );
            crate::style_symbol_provider::reset_workspace_occurrence_extractor_counters_for_test();
            crate::workspace_occurrence_cache::reset_workspace_occurrence_shard_read_counts_for_test();
            crate::workspace_occurrences::reset_workspace_occurrence_memo_hit_counts_for_test();
            crate::workspace_occurrences::workspace_occurrence_indexes_from_documents(
                &state,
                Some(workspace_uri.as_str()),
            );
            assert_eq!(
                crate::style_symbol_provider::workspace_occurrence_extractor_rebuild_count_for_test(
                    app_uri.as_str(),
                ),
                0,
                "opening unrelated B must not rebuild A"
            );
            assert_eq!(
                crate::workspace_occurrences::workspace_occurrence_memo_hit_count_for_test(
                    app_uri.as_str(),
                ),
                1,
            );
            assert_eq!(
                crate::workspace_occurrence_cache::workspace_occurrence_shard_read_count_for_test(
                    app_uri.as_str(),
                ),
                0,
            );

            change_occurrence_style_document(&mut state, tokens_uri.as_str(), 2, "$brand: blue;\n");
            crate::style_symbol_provider::reset_workspace_occurrence_extractor_counters_for_test();
            crate::workspace_occurrence_cache::reset_workspace_occurrence_shard_read_counts_for_test();
            crate::workspace_occurrences::reset_workspace_occurrence_memo_hit_counts_for_test();
            crate::workspace_occurrences::workspace_occurrence_indexes_from_documents(
                &state,
                Some(workspace_uri.as_str()),
            );
            assert_eq!(
                crate::style_symbol_provider::workspace_occurrence_extractor_rebuild_count_for_test(
                    app_uri.as_str(),
                ),
                1,
                "editing a document in A's read set must rebuild A exactly once"
            );
            assert_eq!(
                crate::workspace_occurrences::workspace_occurrence_memo_hit_count_for_test(
                    app_uri.as_str(),
                ),
                0,
                "an in-read-set edit must invalidate A's memo entry"
            );
            assert_eq!(
                crate::workspace_occurrence_cache::workspace_occurrence_shard_read_count_for_test(
                    app_uri.as_str(),
                ),
                1,
                "the affected entry alone may consult its self-validating disk shard"
            );
            assert_eq!(
                crate::style_symbol_provider::workspace_occurrence_read_set_recomputation_count_for_test(
                    app_uri.as_str(),
                ),
                1,
                "an in-read-set edit must recompute A's read set exactly once"
            );
            eprintln!(
                "workspace-occurrence-read-set cold_ms={} warm_memo_ms={} unrelated_edit_ms={} unrelated_edit_app_memo_hits=1 unrelated_edit_app_shard_reads=0 unrelated_edit_app_read_set_recomputes=0 dependency_edit_app_rebuilds=1 dependency_edit_app_read_set_recomputes=1",
                cold_elapsed.as_millis(),
                warm_elapsed.as_millis(),
                unrelated_elapsed.as_millis(),
            );

            let _ = std::fs::remove_dir_all(workspace_root);
            Ok(())
        },
    )
}

#[test]
fn workspace_occurrence_source_entries_follow_imported_style_read_set() -> TestResult {
    let WorkspaceOccurrenceReadSetFixtureV0 {
        mut state,
        workspace_root,
        workspace_uri,
        app_uri,
        usage_uri,
        unrelated_uri,
        ..
    } = workspace_occurrence_read_set_fixture("source-read-set")?;
    crate::workspace_occurrences::workspace_occurrence_indexes_from_documents(
        &state,
        Some(workspace_uri.as_str()),
    );

    change_occurrence_style_document(
        &mut state,
        unrelated_uri.as_str(),
        2,
        ".other { color: green; }\n",
    );
    crate::style_symbol_provider::reset_workspace_occurrence_extractor_counters_for_test();
    crate::workspace_occurrence_cache::reset_workspace_occurrence_shard_read_counts_for_test();
    crate::workspace_occurrences::reset_workspace_occurrence_memo_hit_counts_for_test();
    crate::workspace_occurrences::workspace_occurrence_indexes_from_documents(
        &state,
        Some(workspace_uri.as_str()),
    );
    assert_eq!(
        crate::workspace_occurrences::workspace_occurrence_memo_hit_count_for_test(
            usage_uri.as_str(),
        ),
        1,
        "an unrelated style rename must keep the importing source entry in the RAM memo",
    );
    assert_eq!(
        crate::style_symbol_provider::workspace_occurrence_extractor_rebuild_count_for_test(
            usage_uri.as_str(),
        ),
        0,
        "an unrelated style rename must not re-extract the source document",
    );
    assert_eq!(
        crate::workspace_occurrence_cache::workspace_occurrence_shard_read_count_for_test(
            usage_uri.as_str(),
        ),
        0,
        "an unrelated style rename must not consult the source document's disk shard",
    );

    *state.workspace_occurrence_index_memo_lock() = None;
    crate::style_symbol_provider::reset_workspace_occurrence_extractor_counters_for_test();
    crate::workspace_occurrence_cache::reset_workspace_occurrence_shard_read_counts_for_test();
    crate::workspace_occurrences::reset_workspace_occurrence_memo_hit_counts_for_test();
    crate::workspace_occurrences::workspace_occurrence_indexes_from_documents(
        &state,
        Some(workspace_uri.as_str()),
    );
    assert_eq!(
        crate::workspace_occurrence_cache::workspace_occurrence_shard_read_count_for_test(
            usage_uri.as_str(),
        ),
        1,
        "the narrowed source read-set key must rehydrate the same disk shard after an unrelated rename",
    );
    assert_eq!(
        crate::style_symbol_provider::workspace_occurrence_extractor_rebuild_count_for_test(
            usage_uri.as_str(),
        ),
        0,
        "the narrowed source shard key must avoid re-extraction after an unrelated rename",
    );

    change_occurrence_style_document(
        &mut state,
        app_uri.as_str(),
        2,
        ".renamed { color: red; }\n",
    );
    crate::style_symbol_provider::reset_workspace_occurrence_extractor_counters_for_test();
    crate::workspace_occurrence_cache::reset_workspace_occurrence_shard_read_counts_for_test();
    crate::workspace_occurrences::reset_workspace_occurrence_memo_hit_counts_for_test();
    crate::workspace_occurrences::workspace_occurrence_indexes_from_documents(
        &state,
        Some(workspace_uri.as_str()),
    );
    assert_eq!(
        crate::workspace_occurrences::workspace_occurrence_memo_hit_count_for_test(
            usage_uri.as_str(),
        ),
        0,
        "renaming a selector in the imported style must invalidate the source memo entry",
    );
    assert_eq!(
        crate::style_symbol_provider::workspace_occurrence_extractor_rebuild_count_for_test(
            usage_uri.as_str(),
        ),
        1,
        "an imported-style edit must re-extract exactly the affected source document",
    );
    eprintln!(
        "workspace-occurrence-source-read-set unrelated_style_rename_source_memo_hits=1 unrelated_style_rename_source_rebuilds=0 unrelated_style_rename_source_shard_reads=0 cold_memo_rehydrate_source_shard_reads=1 cold_memo_rehydrate_source_rebuilds=0 imported_style_rename_source_memo_hits=0 imported_style_rename_source_rebuilds=1",
    );

    let _ = std::fs::remove_dir_all(workspace_root);
    Ok(())
}

#[test]
fn workspace_occurrence_unscoped_prefix_unions_imported_and_workspace_definitions() -> TestResult {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-workspace-occurrence-mixed-prefix-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&workspace_root);
    std::fs::create_dir_all(workspace_root.join("src"))?;
    let workspace_uri = path_to_file_uri(workspace_root.as_path());
    let app_uri = path_to_file_uri(workspace_root.join("src/App.module.scss").as_path());
    let other_uri = path_to_file_uri(workspace_root.join("src/Other.module.scss").as_path());
    let latent_uri = path_to_file_uri(workspace_root.join("src/Latent.module.scss").as_path());
    let usage_uri = path_to_file_uri(workspace_root.join("src/Usage.tsx").as_path());
    let app_text = ".root { color: red; }\n";
    let other_text = ".btn-primary { color: blue; }\n.btn-ghost { color: gray; }\n";
    let latent_text = "/* no selectors yet */\n";
    let usage_text = concat!(
        "import styles from './App.module.scss';\n",
        "declare const kind: string;\n",
        "void styles.root;\n",
        "export const view = <div className={`btn-${kind}`} />;\n",
    );
    std::fs::write(workspace_root.join("src/App.module.scss"), app_text)?;
    std::fs::write(workspace_root.join("src/Other.module.scss"), other_text)?;
    std::fs::write(workspace_root.join("src/Latent.module.scss"), latent_text)?;
    std::fs::write(workspace_root.join("src/Usage.tsx"), usage_text)?;

    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [{"uri": workspace_uri, "name": "mixed-prefix"}],
            },
        }),
    );
    open_occurrence_style_document(&mut state, app_uri.as_str(), app_text);
    open_occurrence_style_document(&mut state, other_uri.as_str(), other_text);
    open_occurrence_style_document(&mut state, latent_uri.as_str(), latent_text);
    open_occurrence_source_document(&mut state, usage_uri.as_str(), usage_text);
    let source_document = state
        .document(usage_uri.as_str())
        .ok_or("mixed-prefix source document should be indexed")?;
    assert!(
        source_document
            .source_selector_candidates
            .iter()
            .any(|candidate| {
                candidate.kind == "sourceSelectorPrefixReference"
                    && candidate.target_style_uri.is_none()
            }),
        "the mixed fixture must contain an unscoped prefix candidate: {:?}",
        source_document.source_selector_candidates,
    );

    let cold = crate::workspace_occurrences::workspace_occurrence_indexes_from_documents(
        &state,
        Some(workspace_uri.as_str()),
    );
    let mut cold_names = cold
        .source_selector_index
        .occurrences
        .iter()
        .filter(|occurrence| occurrence.uri == usage_uri)
        .map(|occurrence| occurrence.selector_name.clone())
        .collect::<Vec<_>>();
    cold_names.sort();
    assert_eq!(
        cold_names,
        vec!["btn-ghost", "btn-primary", "root"],
        "an unscoped prefix must union workspace definitions with imported-style definitions",
    );

    change_occurrence_style_document(
        &mut state,
        latent_uri.as_str(),
        2,
        ".btn-solid { color: black; }\n",
    );
    crate::style_symbol_provider::reset_workspace_occurrence_extractor_counters_for_test();
    crate::workspace_occurrences::reset_workspace_occurrence_memo_hit_counts_for_test();
    let refreshed = crate::workspace_occurrences::workspace_occurrence_indexes_from_documents(
        &state,
        Some(workspace_uri.as_str()),
    );
    let mut refreshed_names = refreshed
        .source_selector_index
        .occurrences
        .iter()
        .filter(|occurrence| occurrence.uri == usage_uri)
        .map(|occurrence| occurrence.selector_name.clone())
        .collect::<Vec<_>>();
    refreshed_names.sort();
    assert_eq!(
        refreshed_names,
        vec!["btn-ghost", "btn-primary", "btn-solid", "root"],
        "editing any workspace definition read by an unscoped prefix must refresh the source entry",
    );
    assert_eq!(
        crate::workspace_occurrences::workspace_occurrence_memo_hit_count_for_test(
            usage_uri.as_str(),
        ),
        0,
        "an unscoped prefix has a workspace-wide dependency and must not stale-serve its RAM entry",
    );
    assert_eq!(
        crate::style_symbol_provider::workspace_occurrence_extractor_rebuild_count_for_test(
            usage_uri.as_str(),
        ),
        1,
        "an unscoped-prefix dependency edit must rebuild the affected source entry once",
    );

    let fresh_checked =
        crate::workspace_occurrence_cache::with_workspace_occurrence_shadow_all_for_test(|| {
            crate::workspace_occurrences::workspace_occurrence_indexes_from_documents(
                &state,
                Some(workspace_uri.as_str()),
            )
        });
    assert_eq!(
        serde_json::to_vec(refreshed.source_selector_index.as_ref())?,
        serde_json::to_vec(fresh_checked.source_selector_index.as_ref())?,
        "the mixed-prefix memo value must equal a forced fresh extraction byte-for-byte",
    );

    *state.workspace_occurrence_index_memo_lock() = None;
    let rehydrated = crate::workspace_occurrences::workspace_occurrence_indexes_from_documents(
        &state,
        Some(workspace_uri.as_str()),
    );
    assert_eq!(
        serde_json::to_vec(refreshed.source_selector_index.as_ref())?,
        serde_json::to_vec(rehydrated.source_selector_index.as_ref())?,
        "the workspace-wide prefix read set must rehydrate byte-identically",
    );

    let _ = std::fs::remove_dir_all(workspace_root);
    Ok(())
}

#[test]
fn workspace_occurrence_ram_memo_shadow_repairs_content_divergence() -> TestResult {
    let WorkspaceOccurrenceReadSetFixtureV0 {
        state,
        workspace_root,
        workspace_uri,
        usage_uri,
        ..
    } = workspace_occurrence_read_set_fixture("memo-value-shadow")?;
    let cold = crate::workspace_occurrences::workspace_occurrence_indexes_from_documents(
        &state,
        Some(workspace_uri.as_str()),
    );
    assert!(
        cold.source_selector_index
            .occurrences
            .iter()
            .any(|occurrence| occurrence.uri == usage_uri),
        "the source fixture must produce an occurrence before corrupting the memo value",
    );
    let usage_file_id = state
        .document_file_id(usage_uri.as_str())
        .ok_or("missing source fixture file id")?;
    let shadow_mismatch_count = {
        let mut memo = state.workspace_occurrence_index_memo_lock();
        let memo = memo.as_mut().ok_or("missing workspace occurrence memo")?;
        memo.document_entries
            .get_mut(&usage_file_id)
            .ok_or("missing source memo entry")?
            .occurrences
            .clear();
        std::sync::Arc::clone(&memo.shadow_mismatch_count)
    };
    crate::style_symbol_provider::reset_workspace_occurrence_extractor_counters_for_test();
    let repaired =
        crate::workspace_occurrence_cache::with_workspace_occurrence_shadow_all_for_test(|| {
            crate::workspace_occurrence_cache::with_workspace_occurrence_shadow_recovery_for_test(
                || {
                    crate::workspace_occurrences::workspace_occurrence_indexes_from_documents(
                        &state,
                        Some(workspace_uri.as_str()),
                    )
                },
            )
        });
    assert!(
        repaired
            .source_selector_index
            .occurrences
            .iter()
            .any(|occurrence| occurrence.uri == usage_uri),
        "the RAM memo value oracle must replace a content-divergent cached value",
    );
    let verified_after_repair =
        crate::style_symbol_provider::workspace_occurrence_shadow_verification_total_for_test();
    assert!(
        verified_after_repair > 0,
        "a sampled memo revision must byte-check its reusable entries",
    );
    let repeated =
        crate::workspace_occurrence_cache::with_workspace_occurrence_shadow_all_for_test(|| {
            crate::workspace_occurrences::workspace_occurrence_indexes_from_documents(
                &state,
                Some(workspace_uri.as_str()),
            )
        });
    assert_eq!(
        serde_json::to_vec(repaired.workspace_index.as_ref())?,
        serde_json::to_vec(repeated.workspace_index.as_ref())?,
        "a verified revision must serve byte-identically",
    );
    assert_eq!(
        crate::style_symbol_provider::workspace_occurrence_shadow_verification_total_for_test(),
        verified_after_repair,
        "a sampled revision must be byte-checked once, then keep the aggregate fast path",
    );
    assert_eq!(
        shadow_mismatch_count.load(std::sync::atomic::Ordering::Relaxed),
        1,
        "the RAM memo value oracle must account the repaired mismatch",
    );

    let _ = std::fs::remove_dir_all(workspace_root);
    Ok(())
}

#[test]
fn workspace_occurrence_shadow_samples_both_production_serve_arms() -> TestResult {
    for arm in [
        WorkspaceOccurrenceServeArmV0::Style,
        WorkspaceOccurrenceServeArmV0::Source,
    ] {
        let WorkspaceOccurrenceReadSetFixtureV0 {
            state,
            workspace_root,
            workspace_uri,
            app_uri,
            usage_uri,
            ..
        } = workspace_occurrence_sampled_read_set_fixture(
            "shadow-coverage",
            arm,
            WorkspaceOccurrenceSampleKeyV0::Actual,
        )?;
        crate::workspace_occurrences::workspace_occurrence_indexes_from_documents(
            &state,
            Some(workspace_uri.as_str()),
        );
        *state.workspace_occurrence_index_memo_lock() = None;
        crate::style_symbol_provider::reset_workspace_occurrence_extractor_counters_for_test();
        crate::workspace_occurrences::workspace_occurrence_indexes_from_documents(
            &state,
            Some(workspace_uri.as_str()),
        );
        let target_uri = match arm {
            WorkspaceOccurrenceServeArmV0::Style => app_uri.as_str(),
            WorkspaceOccurrenceServeArmV0::Source => usage_uri.as_str(),
        };
        assert_eq!(
            crate::style_symbol_provider::workspace_occurrence_shadow_verification_count_for_test(
                target_uri,
            ),
            1,
            "the production 1/16 sampler must verify the selected {arm:?} serve arm",
        );
        let _ = std::fs::remove_dir_all(workspace_root);
    }
    Ok(())
}

#[test]
fn workspace_occurrence_shadow_rejects_a_dropped_read_set_key_field() -> TestResult {
    omena_testkit::with_instrumentation_session(
        omena_testkit::InstrumentationSessionV0::default(),
        || {
            let WorkspaceOccurrenceReadSetFixtureV0 {
                mut state,
                workspace_root,
                workspace_uri,
                tokens_uri,
                ..
            } = workspace_occurrence_sampled_read_set_fixture(
                "shadow-drop",
                WorkspaceOccurrenceServeArmV0::Style,
                WorkspaceOccurrenceSampleKeyV0::DependencyDropped,
            )?;
            let mutation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                crate::workspace_occurrence_cache::with_workspace_occurrence_key_dependency_drop_for_test(
                    || {
                        crate::workspace_occurrences::workspace_occurrence_indexes_from_documents(
                            &state,
                            Some(workspace_uri.as_str()),
                        );
                        change_occurrence_style_document(
                            &mut state,
                            tokens_uri.as_str(),
                            2,
                            "$other: blue;\n",
                        );
                        *state.workspace_occurrence_index_memo_lock() = None;
                        crate::workspace_occurrences::workspace_occurrence_indexes_from_documents(
                            &state,
                            Some(workspace_uri.as_str()),
                        );
                    },
                );
            }));
            let _ = std::fs::remove_dir_all(workspace_root);
            assert!(
                mutation.is_err(),
                "dropping the dependency digest must serve a stale shard and make the shadow oracle RED"
            );
            Ok(())
        },
    )
}

#[test]
fn workspace_occurrence_shadow_rejects_a_dropped_extractor_workspace_scope() -> TestResult {
    omena_testkit::with_instrumentation_session(
        omena_testkit::InstrumentationSessionV0::default(),
        || {
            let WorkspaceOccurrenceReadSetFixtureV0 {
                state,
                workspace_root,
                workspace_uri,
                app_uri,
                ..
            } = workspace_occurrence_sampled_read_set_fixture(
                "workspace-scope-drop",
                WorkspaceOccurrenceServeArmV0::Style,
                WorkspaceOccurrenceSampleKeyV0::WorkspaceScopeDropped,
            )?;
            let app_document = state
                .document(app_uri.as_str())
                .ok_or("missing workspace-scope fixture document")?;
            let scoped = crate::style_symbol_provider::extract_fresh_style_symbol_workspace_occurrences_for_document(
                &state,
                app_document,
                Some(workspace_uri.as_str()),
            );
            let unscoped = crate::style_symbol_provider::extract_fresh_style_symbol_workspace_occurrences_for_document(
                &state,
                app_document,
                None,
            );
            assert_ne!(
                scoped, unscoped,
                "the fixture must make the extractor workspace scope observable before exercising the seeded key drop",
            );
            let mutation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                crate::workspace_occurrence_cache::with_workspace_occurrence_key_workspace_folder_uri_drop_for_test(
                    || {
                        crate::workspace_occurrences::workspace_occurrence_indexes_from_documents(
                            &state,
                            Some(workspace_uri.as_str()),
                        );
                        *state.workspace_occurrence_index_memo_lock() = None;
                        crate::workspace_occurrences::workspace_occurrence_indexes_from_documents(
                            &state,
                            None,
                        );
                    },
                );
            }));
            let _ = std::fs::remove_dir_all(workspace_root);
            assert!(
                mutation.is_err(),
                "dropping the extractor workspace scope must stale-serve and make the shadow oracle RED",
            );
            Ok(())
        },
    )
}

#[test]
fn workspace_occurrence_shadow_mismatch_recovers_and_increments_the_production_counter()
-> TestResult {
    let WorkspaceOccurrenceReadSetFixtureV0 {
        mut state,
        workspace_root,
        workspace_uri,
        tokens_uri,
        ..
    } = workspace_occurrence_sampled_read_set_fixture(
        "shadow-recovery",
        WorkspaceOccurrenceServeArmV0::Style,
        WorkspaceOccurrenceSampleKeyV0::DependencyDropped,
    )?;
    crate::workspace_occurrence_cache::with_workspace_occurrence_shadow_recovery_for_test(|| {
        crate::workspace_occurrence_cache::with_workspace_occurrence_key_dependency_drop_for_test(
            || {
                crate::workspace_occurrences::workspace_occurrence_indexes_from_documents(
                    &state,
                    Some(workspace_uri.as_str()),
                );
                change_occurrence_style_document(
                    &mut state,
                    tokens_uri.as_str(),
                    2,
                    "$other: blue;\n",
                );
                *state.workspace_occurrence_index_memo_lock() = None;
                crate::workspace_occurrences::workspace_occurrence_indexes_from_documents(
                    &state,
                    Some(workspace_uri.as_str()),
                );
                assert_eq!(
                    state
                        .workspace_occurrence_index_memo_lock()
                        .as_ref()
                        .map(|memo| {
                            memo.shadow_mismatch_count
                                .load(std::sync::atomic::Ordering::Relaxed)
                        })
                        .unwrap_or(0),
                    1,
                    "production mismatch policy must count the repaired stale shard",
                );
            },
        );
    });
    let _ = std::fs::remove_dir_all(workspace_root);
    Ok(())
}

#[test]
#[ignore = "release-only timing receipt; run explicitly with --release --ignored"]
fn workspace_occurrence_release_timing_uses_comparable_corpus() -> TestResult {
    const SAMPLE_COUNT: usize = 7;
    const STYLE_DOCUMENT_COUNT: usize = 140;
    const SOURCE_DOCUMENT_COUNT: usize = 10;
    let mut cold_ms = Vec::with_capacity(SAMPLE_COUNT);
    let mut production_shadow_ms = Vec::with_capacity(SAMPLE_COUNT);
    let mut unrelated_edit_ms = Vec::with_capacity(SAMPLE_COUNT);
    let mut unrelated_edit_source_rebuilds = Vec::with_capacity(SAMPLE_COUNT);
    let mut full_shadow_ms = Vec::with_capacity(SAMPLE_COUNT);
    let mut production_verified_hits = Vec::with_capacity(SAMPLE_COUNT);
    let mut full_verified_hits = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let (mut state, workspace_root, workspace_uri, unrelated_uri) =
            workspace_occurrence_timing_fixture(
                format!("release-timing-{sample}").as_str(),
                STYLE_DOCUMENT_COUNT,
                SOURCE_DOCUMENT_COUNT,
            )?;

        let started = std::time::Instant::now();
        crate::workspace_occurrences::workspace_occurrence_indexes_from_documents(
            &state,
            Some(workspace_uri.as_str()),
        );
        cold_ms.push(started.elapsed().as_secs_f64() * 1_000.0);

        *state.workspace_occurrence_index_memo_lock() = None;
        crate::style_symbol_provider::reset_workspace_occurrence_extractor_counters_for_test();
        let started = std::time::Instant::now();
        crate::workspace_occurrences::workspace_occurrence_indexes_from_documents(
            &state,
            Some(workspace_uri.as_str()),
        );
        production_shadow_ms.push(started.elapsed().as_secs_f64() * 1_000.0);
        production_verified_hits.push(
            crate::style_symbol_provider::workspace_occurrence_shadow_verification_total_for_test(),
        );

        change_occurrence_style_document(
            &mut state,
            unrelated_uri.as_str(),
            2,
            ".renamed-unrelated { color: blue; }\n",
        );
        crate::style_symbol_provider::reset_workspace_occurrence_extractor_counters_for_test();
        let started = std::time::Instant::now();
        crate::workspace_occurrences::workspace_occurrence_indexes_from_documents(
            &state,
            Some(workspace_uri.as_str()),
        );
        unrelated_edit_ms.push(started.elapsed().as_secs_f64() * 1_000.0);
        unrelated_edit_source_rebuilds.push(
            (0..SOURCE_DOCUMENT_COUNT)
                .map(|ordinal| {
                    let uri = path_to_file_uri(
                        workspace_root
                            .join("src")
                            .join(format!("Usage{ordinal:03}.tsx"))
                            .as_path(),
                    );
                    crate::style_symbol_provider::workspace_occurrence_extractor_rebuild_count_for_test(
                        uri.as_str(),
                    )
                })
                .sum::<u64>(),
        );

        *state.workspace_occurrence_index_memo_lock() = None;
        crate::style_symbol_provider::reset_workspace_occurrence_extractor_counters_for_test();
        let started = std::time::Instant::now();
        crate::workspace_occurrence_cache::with_workspace_occurrence_shadow_all_for_test(|| {
            crate::workspace_occurrences::workspace_occurrence_indexes_from_documents(
                &state,
                Some(workspace_uri.as_str()),
            );
        });
        full_shadow_ms.push(started.elapsed().as_secs_f64() * 1_000.0);
        full_verified_hits.push(
            crate::style_symbol_provider::workspace_occurrence_shadow_verification_total_for_test(),
        );

        let _ = std::fs::remove_dir_all(workspace_root);
    }

    cold_ms.sort_by(f64::total_cmp);
    production_shadow_ms.sort_by(f64::total_cmp);
    unrelated_edit_ms.sort_by(f64::total_cmp);
    unrelated_edit_source_rebuilds.sort_unstable();
    full_shadow_ms.sort_by(f64::total_cmp);
    production_verified_hits.sort_unstable();
    full_verified_hits.sort_unstable();
    let middle = SAMPLE_COUNT / 2;
    eprintln!(
        "workspace-occurrence-release-timing documents={} style_documents={} source_documents={} samples={} production_shadow_rate=1/16 cold_ms_min={:.3} cold_ms_p50={:.3} cold_ms_max={:.3} production_shadow_ms_p50={:.3} unrelated_style_rename_ms_min={:.3} unrelated_style_rename_ms_p50={:.3} unrelated_style_rename_ms_max={:.3} unrelated_style_rename_source_rebuilds_p50={} full_shadow_ms_p50={:.3} production_verified_hits_p50={} full_verified_hits_p50={}",
        STYLE_DOCUMENT_COUNT + SOURCE_DOCUMENT_COUNT,
        STYLE_DOCUMENT_COUNT,
        SOURCE_DOCUMENT_COUNT,
        SAMPLE_COUNT,
        cold_ms[0],
        cold_ms[middle],
        cold_ms[SAMPLE_COUNT - 1],
        production_shadow_ms[middle],
        unrelated_edit_ms[0],
        unrelated_edit_ms[middle],
        unrelated_edit_ms[SAMPLE_COUNT - 1],
        unrelated_edit_source_rebuilds[middle],
        full_shadow_ms[middle],
        production_verified_hits[middle],
        full_verified_hits[middle],
    );
    assert!(production_verified_hits[middle] > 0);
    assert_eq!(
        unrelated_edit_source_rebuilds[middle], 0,
        "renaming an unrelated style must not rebuild any source occurrence entry",
    );
    assert_eq!(
        full_verified_hits[middle],
        (STYLE_DOCUMENT_COUNT + SOURCE_DOCUMENT_COUNT) as u64,
    );
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum WorkspaceOccurrenceServeArmV0 {
    Style,
    Source,
}

#[derive(Debug, Clone, Copy)]
enum WorkspaceOccurrenceSampleKeyV0 {
    Actual,
    DependencyDropped,
    WorkspaceScopeDropped,
}

fn workspace_occurrence_sampled_read_set_fixture(
    suffix: &str,
    arm: WorkspaceOccurrenceServeArmV0,
    sample_key: WorkspaceOccurrenceSampleKeyV0,
) -> Result<WorkspaceOccurrenceReadSetFixtureV0, Box<dyn std::error::Error>> {
    for attempt in 0..512 {
        let fixture = workspace_occurrence_read_set_fixture(
            format!("{suffix}-{}-{attempt}", std::process::id()).as_str(),
        )?;
        let target_uri = match arm {
            WorkspaceOccurrenceServeArmV0::Style => fixture.app_uri.as_str(),
            WorkspaceOccurrenceServeArmV0::Source => fixture.usage_uri.as_str(),
        };
        let document = fixture
            .state
            .document(target_uri)
            .ok_or("missing sampled occurrence document")?;
        let dependency_digest = match arm {
            WorkspaceOccurrenceServeArmV0::Style => {
                crate::style_symbol_provider::style_symbol_occurrence_read_set(
                    &fixture.state,
                    document,
                )
                .dependency_digest
            }
            WorkspaceOccurrenceServeArmV0::Source => {
                let definitions = style_selector_definitions_from_open_documents(
                    &fixture.state,
                    "",
                    Some(fixture.workspace_uri.as_str()),
                )
                .iter()
                .map(|(uri, definition)| {
                    query_style_selector_definition_for_matching(uri, definition)
                })
                .collect::<Vec<_>>();
                crate::workspace_occurrences::source_selector_occurrence_read_set_digest_for_test(
                    &fixture.state,
                    document,
                    &definitions,
                )
            }
        };
        let resolution_inputs = resolution_inputs_for_workspace_uri(
            &fixture.state,
            document.workspace_folder_uri.as_deref(),
        );
        let key = crate::workspace_occurrence_cache::workspace_occurrence_shard_key_for_test(
            document.workspace_folder_uri.as_deref(),
            match sample_key {
                WorkspaceOccurrenceSampleKeyV0::WorkspaceScopeDropped => None,
                WorkspaceOccurrenceSampleKeyV0::Actual
                | WorkspaceOccurrenceSampleKeyV0::DependencyDropped => {
                    Some(fixture.workspace_uri.as_str())
                }
            },
            document.uri.as_str(),
            document.language_id.as_str(),
            document.text_hash.as_str(),
            match sample_key {
                WorkspaceOccurrenceSampleKeyV0::DependencyDropped => None,
                WorkspaceOccurrenceSampleKeyV0::Actual
                | WorkspaceOccurrenceSampleKeyV0::WorkspaceScopeDropped => {
                    dependency_digest.as_deref()
                }
            },
            &resolution_inputs,
        );
        if key.as_deref().is_some_and(
            crate::workspace_occurrence_cache::workspace_occurrence_shard_should_shadow,
        ) {
            return Ok(fixture);
        }
        let _ = std::fs::remove_dir_all(fixture.workspace_root);
    }
    Err("failed to derive a production-sampled workspace occurrence fixture".into())
}

struct WorkspaceOccurrenceReadSetFixtureV0 {
    state: LspShellState,
    workspace_root: PathBuf,
    workspace_uri: String,
    app_uri: String,
    usage_uri: String,
    tokens_uri: String,
    unrelated_uri: String,
}

fn workspace_occurrence_read_set_fixture(
    suffix: &str,
) -> Result<WorkspaceOccurrenceReadSetFixtureV0, Box<dyn std::error::Error>> {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-workspace-occurrence-{suffix}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&workspace_root);
    std::fs::create_dir_all(workspace_root.join("src"))?;
    let workspace_uri = path_to_file_uri(workspace_root.as_path());
    let app_uri = path_to_file_uri(workspace_root.join("src/App.module.scss").as_path());
    let usage_uri = path_to_file_uri(workspace_root.join("src/Usage.tsx").as_path());
    let tokens_uri = path_to_file_uri(workspace_root.join("src/_tokens.scss").as_path());
    let unrelated_uri =
        path_to_file_uri(workspace_root.join("src/Unrelated.module.scss").as_path());
    let app_text = "@use \"./tokens\" as tokens;\n@use \"./missing\" as missing;\n:root { --theme: red; }\n.root { color: tokens.$brand; border-color: missing.$tone; background: var(--theme); }\n";
    let usage_text = "import styles from './App.module.scss';\nvoid styles.root;\n";
    let tokens_text = "$brand: red;\n";
    let unrelated_text = ".unrelated { color: red; }\n";
    std::fs::write(workspace_root.join("src/App.module.scss"), app_text)?;
    std::fs::write(workspace_root.join("src/Usage.tsx"), usage_text)?;
    std::fs::write(workspace_root.join("src/_tokens.scss"), tokens_text)?;
    std::fs::write(
        workspace_root.join("src/Unrelated.module.scss"),
        unrelated_text,
    )?;
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [{"uri": workspace_uri, "name": "occurrence-read-set"}],
            },
        }),
    );
    open_occurrence_style_document(&mut state, app_uri.as_str(), app_text);
    open_occurrence_source_document(&mut state, usage_uri.as_str(), usage_text);
    open_occurrence_style_document(&mut state, tokens_uri.as_str(), tokens_text);
    open_occurrence_style_document(&mut state, unrelated_uri.as_str(), unrelated_text);
    Ok(WorkspaceOccurrenceReadSetFixtureV0 {
        state,
        workspace_root,
        workspace_uri,
        app_uri,
        usage_uri,
        tokens_uri,
        unrelated_uri,
    })
}

fn workspace_occurrence_timing_fixture(
    suffix: &str,
    style_document_count: usize,
    source_document_count: usize,
) -> Result<(LspShellState, PathBuf, String, String), Box<dyn std::error::Error>> {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-workspace-occurrence-{suffix}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&workspace_root);
    std::fs::create_dir_all(workspace_root.join("src"))?;
    let workspace_uri = path_to_file_uri(workspace_root.as_path());
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [{"uri": workspace_uri, "name": "occurrence-timing"}],
            },
        }),
    );

    let unrelated_path = workspace_root
        .join("src")
        .join(format!("Style{:03}.module.scss", style_document_count - 1));
    let unrelated_uri = path_to_file_uri(unrelated_path.as_path());
    for ordinal in 0..style_document_count {
        let path = workspace_root
            .join("src")
            .join(format!("Style{ordinal:03}.module.scss"));
        let uri = path_to_file_uri(path.as_path());
        let text = if ordinal == 0 {
            "$tone: rgb(1, 3, 7);\n.item000 { color: $tone; }\n".to_string()
        } else {
            format!(
                "@use \"./Style000.module.scss\" as base;\n.item{ordinal:03} {{ color: base.$tone; }}\n",
            )
        };
        std::fs::write(&path, &text)?;
        open_occurrence_style_document(&mut state, uri.as_str(), text.as_str());
    }
    for ordinal in 0..source_document_count {
        let path = workspace_root
            .join("src")
            .join(format!("Usage{ordinal:03}.tsx"));
        let uri = path_to_file_uri(path.as_path());
        let style_ordinal = ordinal % style_document_count;
        let text = format!(
            "import styles from './Style{style_ordinal:03}.module.scss';\nvoid styles.item{style_ordinal:03};\n",
        );
        std::fs::write(&path, &text)?;
        open_occurrence_source_document(&mut state, uri.as_str(), text.as_str());
    }
    Ok((state, workspace_root, workspace_uri, unrelated_uri))
}

fn open_occurrence_source_document(state: &mut LspShellState, uri: &str, text: &str) {
    handle_lsp_message(
        state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "languageId": "typescriptreact",
                    "version": 1,
                    "text": text,
                },
            },
        }),
    );
}

fn open_occurrence_style_document(state: &mut LspShellState, uri: &str, text: &str) {
    handle_lsp_message(
        state,
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

fn change_occurrence_style_document(
    state: &mut LspShellState,
    uri: &str,
    version: i64,
    text: &str,
) {
    handle_lsp_message(
        state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {"uri": uri, "version": version},
                "contentChanges": [{"text": text}],
            },
        }),
    );
}
