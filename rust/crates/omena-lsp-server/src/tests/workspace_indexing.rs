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
    let memo_after_references = state
        .source_selector_occurrence_index_memo
        .borrow()
        .as_ref()
        .map(|memo| std::sync::Arc::clone(&memo.index))
        .ok_or_else(|| {
            std::io::Error::other("references should populate source occurrence memo")
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
        .source_selector_occurrence_index_memo
        .borrow()
        .as_ref()
        .map(|memo| std::sync::Arc::clone(&memo.index))
        .ok_or_else(|| std::io::Error::other("rename should retain source occurrence memo"))?;
    assert!(
        std::sync::Arc::ptr_eq(&memo_after_references, &memo_after_rename),
        "rename should reuse the source occurrence index produced for references"
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
    let _ = std::fs::remove_dir_all(&workspace_root);
    std::fs::create_dir_all(&src_dir)?;
    let source_text = "import styles from \"./Button.module.scss\";\nconst view = <div className={styles.root} />;";
    std::fs::write(&style_path, ".root { color: red; }")?;
    std::fs::write(&source_path, source_text)?;

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
