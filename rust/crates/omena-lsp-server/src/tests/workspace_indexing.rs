use super::*;

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
                property_name: "color".to_string(),
                value: Some("red".to_string()),
                target_style_uri: Some(style_uri.clone()),
                cascade_tier: "authorInlineStyle",
                static_value: true,
            },
        ],
        selector_references: vec![SourceSelectorReferenceFact {
            byte_span: selector_span,
            selector_name: Some("cachedRoot".to_string()),
            match_kind: SourceSelectorReferenceMatchKind::Exact,
            target_style_uri: Some(style_uri.clone()),
        }],
        type_fact_targets: Vec::new(),
        class_value_universes: vec![omena_query::OmenaQuerySourceClassValueUniverseEntryV0 {
            plugin_id: "cva-recipe-domain",
            domain: "cva-recipe",
            owner_name: "buttonRecipe".to_string(),
            class_names: vec!["button_primary".to_string()],
            axes: vec![omena_query::OmenaQuerySourceClassValueUniverseAxisV0 {
                axis_name: "intent".to_string(),
                values: vec!["primary".to_string()],
            }],
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
    };
    let text_hash = crate::source_document_cache::source_document_text_hash(source_text);
    crate::source_document_cache::store_source_document_index_sidecar(
        Some(workspace_uri.as_str()),
        source_uri.as_str(),
        "typescriptreact",
        text_hash.as_str(),
        &resolution_inputs,
        &cached_index,
        false,
    );
    let sidecar_path =
        crate::source_document_cache::source_document_index_sidecar_file_path_for_test(
            Some(workspace_uri.as_str()),
            source_uri.as_str(),
            "typescriptreact",
            text_hash.as_str(),
            &resolution_inputs,
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
        state.workspace_occurrence_index_memo.borrow().is_some(),
        "references should populate the workspace occurrence memo"
    );
    let document_keys =
        source_selector_occurrence_document_keys(&state, Some(workspace_uri.as_str()));
    let sidecar_path =
        crate::source_occurrence_cache::source_occurrence_sidecar_file_path_for_test(
            &state,
            Some(workspace_uri.as_str()),
            document_keys.as_slice(),
        )
        .ok_or_else(|| std::io::Error::other("source occurrence sidecar path should resolve"))?;
    assert!(
        sidecar_path.exists(),
        "references should persist the source occurrence sidecar: {sidecar_path:?}"
    );
    *state.workspace_occurrence_index_memo.borrow_mut() = None;
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
        .workspace_occurrence_index_memo
        .borrow()
        .as_ref()
        .map(|memo| std::sync::Arc::clone(&memo.source_selector_index))
        .ok_or_else(|| {
            std::io::Error::other("cached references should populate source occurrence memo")
        })?;

    let rename_response = handle_lsp_message(
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
    );
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
        .workspace_occurrence_index_memo
        .borrow()
        .as_ref()
        .map(|memo| std::sync::Arc::clone(&memo.source_selector_index))
        .ok_or_else(|| std::io::Error::other("rename should retain source occurrence memo"))?;
    assert!(
        std::sync::Arc::ptr_eq(&memo_after_cached_references, &memo_after_rename),
        "rename should reuse the source occurrence index rehydrated for references"
    );
    let _ = std::fs::remove_dir_all(&workspace_root);
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
    let _ = std::fs::remove_dir_all(&workspace_root);
    std::fs::create_dir_all(&src_dir)?;
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
    let new_uri = style_uris
        .get(3)
        .ok_or_else(|| std::io::Error::other("missing new style uri"))?;
    let result = LspWorkspaceIndexResultV0 {
        revision: state.workspace_index_revision,
        progress_token: None,
        documents: vec![lsp_text_document_state(
            new_uri.clone(),
            Some(workspace_uri.clone()),
            "scss".to_string(),
            0,
            ".new { color: blue; }".to_string(),
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

    let _ = std::fs::remove_dir_all(&workspace_root);
    Ok(())
}

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
        },
    );
    assert_eq!(
        omena_query::read_style_fact_entry_probe_for_test(),
        std::collections::BTreeSet::from([theme_uri.clone()]),
        "background-indexed style edits must recompute only the changed style fact",
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
    let revision_before = state.external_sif_refresh_revision;
    assert!(apply_background_workspace_index_result(&mut state, result));
    let refresh_revision_delta = state
        .external_sif_refresh_revision
        .saturating_sub(revision_before);
    let job = prepare_deferred_external_sif_refresh_job(&mut state).ok_or_else(|| {
        std::io::Error::other("workspace-index result did not schedule external SIF refresh")
    })?;
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
        published_uris.contains(&style_uri),
        "style open should publish diagnostics for the opened style document: {published_uris:?}"
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
                        "text": ".root { color: green; }",
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
            },
            SourceSelectorReferenceFact {
                byte_span: ParserByteSpanV0 {
                    start: primary_start,
                    end: primary_start + "primary".len(),
                },
                selector_name: Some("buttonPrimary".to_string()),
                match_kind: SourceSelectorReferenceMatchKind::Exact,
                target_style_uri: Some(style_uri.clone()),
            },
            SourceSelectorReferenceFact {
                byte_span: ParserByteSpanV0 {
                    start: lost_start,
                    end: lost_start + "lost".len(),
                },
                selector_name: Some("lost".to_string()),
                match_kind: SourceSelectorReferenceMatchKind::Prefix,
                target_style_uri: Some(style_uri.clone()),
            },
            SourceSelectorReferenceFact {
                byte_span: ParserByteSpanV0 {
                    start: empty_start,
                    end: empty_start + "empty".len(),
                },
                selector_name: Some("emptyGhost".to_string()),
                match_kind: SourceSelectorReferenceMatchKind::Prefix,
                target_style_uri: Some(style_uri.clone()),
            },
        ],
        type_fact_targets: Vec::new(),
        class_value_universes: Vec::new(),
        domain_class_references: Vec::new(),
    };
    let text_hash = crate::source_document_cache::source_document_text_hash(source_text);
    crate::source_document_cache::store_source_document_index_sidecar(
        Some(workspace_uri.as_str()),
        source_uri.as_str(),
        "typescriptreact",
        text_hash.as_str(),
        &resolution_inputs,
        &cached_index,
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
        }],
        type_fact_targets: Vec::new(),
        class_value_universes: Vec::new(),
        domain_class_references: Vec::new(),
    };
    let text_hash = crate::source_document_cache::source_document_text_hash(source_text);
    crate::source_document_cache::store_source_document_index_sidecar(
        Some(workspace_uri.as_str()),
        source_uri.as_str(),
        "typescriptreact",
        text_hash.as_str(),
        &resolution_inputs,
        &cached_index,
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
        state.workspace_occurrence_index_memo.borrow().is_some(),
        "custom property definition should populate the workspace occurrence memo"
    );
    let document_keys = style_symbol_occurrence_document_keys(&state, Some(workspace_uri.as_str()));
    let sidecar_path =
        crate::style_symbol_occurrence_cache::style_symbol_occurrence_sidecar_file_path_for_test(
            &state,
            Some(workspace_uri.as_str()),
            document_keys.as_slice(),
        )
        .ok_or_else(|| {
            std::io::Error::other("style symbol occurrence sidecar path should resolve")
        })?;
    assert!(
        sidecar_path.exists(),
        "custom property lookup should persist the style symbol occurrence sidecar: {sidecar_path:?}"
    );
    *state.workspace_occurrence_index_memo.borrow_mut() = None;
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
        "style symbol sidecar should rehydrate custom property definitions without rescanning the declaring style candidates: {cached_definition_response:?}"
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
        state.workspace_occurrence_index_memo.borrow().is_some(),
        "Sass references should populate the workspace occurrence memo"
    );
    let document_keys = style_symbol_occurrence_document_keys(&state, Some(workspace_uri.as_str()));
    let sidecar_path =
        crate::style_symbol_occurrence_cache::style_symbol_occurrence_sidecar_file_path_for_test(
            &state,
            Some(workspace_uri.as_str()),
            document_keys.as_slice(),
        )
        .ok_or_else(|| {
            std::io::Error::other("style symbol occurrence sidecar path should resolve")
        })?;
    assert!(
        sidecar_path.exists(),
        "Sass reference lookup should persist the style symbol occurrence sidecar: {sidecar_path:?}"
    );
    *state.workspace_occurrence_index_memo.borrow_mut() = None;
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
        "style symbol sidecar should rehydrate the first Sass consumer without rescanning style candidates: {cached_references_response:?}"
    );
    assert!(
        cached_reference_locations.iter().any(|location| location
            .get("uri")
            .and_then(Value::as_str)
            .is_some_and(|uri| file_uri_equivalent(uri, other_uri.as_str()))),
        "style symbol sidecar should rehydrate the second Sass consumer without rescanning style candidates: {cached_references_response:?}"
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
