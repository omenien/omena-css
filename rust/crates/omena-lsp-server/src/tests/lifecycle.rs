use super::*;

#[test]
fn declares_current_node_lsp_capability_contract() {
    let capabilities = current_node_lsp_capability_contract();

    assert_eq!(capabilities.text_document_sync, 2);
    assert!(capabilities.definition_provider);
    assert!(capabilities.hover_provider);
    assert!(capabilities.references_provider);
    assert_eq!(
        capabilities.completion_provider.trigger_characters,
        vec!["'", "\"", "`", ",", ".", "$", "@", "-"],
    );
    assert_eq!(
        capabilities.code_action_provider.code_action_kinds,
        vec!["quickfix", "refactor.extract", "refactor.inline"]
    );
    assert!(capabilities.rename_provider.prepare_provider);
    assert!(capabilities.workspace.workspace_folders.supported);
    assert!(
        capabilities
            .workspace
            .workspace_folders
            .change_notifications
    );
}

#[test]
fn declares_migration_blocking_work_policy() {
    let summary = summarize_omena_lsp_server_boundary();

    assert_eq!(summary.product, "omena-lsp-server.boundary");
    assert!(
        summary
            .blocking_work_policy
            .contains(&"noFullWorkspaceProgramOnRequestPath")
    );
    assert!(
        summary
            .blocking_work_policy
            .contains(&"queuedRequestCancellationBeforeProviderWork")
    );
    assert!(
        summary
            .blocking_work_policy
            .contains(&"tsgoProviderCancellationTokenBoundary")
    );
    assert!(
        !summary
            .next_decoupling_targets
            .contains(&"tsgoJsonRpcProviderImplementation")
    );
    assert!(
        summary
            .tsgo_client_boundary
            .ready_surfaces
            .contains(&"jsonRpcTypeFactProviderImplementation")
    );
    assert!(
        !summary
            .next_decoupling_targets
            .contains(&"thinVsCodeClientHost")
    );
    assert!(
        !summary
            .next_decoupling_targets
            .contains(&"multiEditorDistribution")
    );
    assert!(
        summary
            .migration_phases
            .iter()
            .any(|phase| phase.phase == "phase-4-thin-client")
    );
    assert_eq!(
        summary.thin_client_endpoint.product,
        "omena-lsp-server.thin-client-endpoint"
    );
    assert!(!summary.thin_client_endpoint.node_fallback_allowed);
    assert!(
        summary
            .thin_client_endpoint
            .host_responsibilities
            .contains(&"buildThinClientServerOptions")
    );
    assert!(
        summary
            .thin_client_endpoint
            .rust_responsibilities
            .contains(&"ownTsgoClientLifecycle")
    );
    assert_eq!(
        summary.multi_editor_distribution.product,
        "omena-lsp-server.multi-editor-distribution"
    );
    assert!(
        summary
            .multi_editor_distribution
            .supported_editors
            .contains(&"neovim")
    );
    assert!(
        summary
            .multi_editor_distribution
            .endpoint_policy
            .contains(&"nodeLspServerIsNotPrimaryEndpoint")
    );
    assert!(
        summary
            .handler_surfaces
            .iter()
            .any(|surface| surface.method == "textDocument/hover"),
    );
    assert!(
        summary
            .handler_surfaces
            .iter()
            .any(|surface| surface.method == CANCEL_REQUEST_METHOD),
    );
    assert_eq!(
        summary.source_provider_adapter.product,
        "omena-lsp-server.source-provider-direct-rust-adapter"
    );
    assert!(
        summary
            .source_provider_adapter
            .request_path_policy
            .contains(&"noNodeWorkspaceTypeResolverOnSourceProviderPath")
    );
    assert!(
        summary
            .source_provider_adapter
            .provider_surfaces
            .contains(&"textDocument/definition")
    );
}

#[test]
fn handles_minimal_lsp_lifecycle_requests() {
    let mut state = LspShellState::default();
    let initialize = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": "file:///workspace-a",
                        "name": "workspace-a",
                    },
                ],
            },
        }),
    );

    assert_eq!(
        initialize.as_ref().and_then(|value| value.get("id")),
        Some(&json!(1))
    );
    assert_eq!(
        initialize
            .as_ref()
            .and_then(|value| value.pointer("/result/capabilities/textDocumentSync")),
        Some(&json!(2)),
    );
    assert!(!state.shutdown_requested);
    assert_eq!(state.workspace_folder_count(), 1);
    assert_eq!(
        state
            .workspace_folder("file:///workspace-a")
            .map(|folder| folder.name.as_str()),
        Some("workspace-a"),
    );

    let runtime_probe = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": RUNTIME_LOOP_PROBE_REQUEST,
        }),
    );
    assert_eq!(
        runtime_probe.as_ref().and_then(|value| value.get("id")),
        Some(&json!(2)),
    );
    assert!(
        runtime_probe
            .as_ref()
            .and_then(|value| value.pointer("/result/now"))
            .and_then(Value::as_u64)
            .is_some(),
    );

    let shutdown = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "shutdown",
        }),
    );
    assert_eq!(
        shutdown.as_ref().and_then(|value| value.get("result")),
        Some(&Value::Null)
    );
    assert!(state.shutdown_requested);

    let exit = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "exit",
        }),
    );
    assert!(exit.is_none());
    assert!(state.should_exit);
}

#[test]
fn reports_unknown_requests_without_panicking() {
    let mut state = LspShellState::default();
    let response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": "unknown-1",
            "method": "workspace/symbol",
        }),
    );

    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/error/code")),
        Some(&json!(-32601)),
    );
}

#[test]
fn cancels_queued_requests_before_provider_work() {
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": CANCEL_REQUEST_METHOD,
            "params": {
                "id": "hover-1",
            },
        }),
    );
    assert_eq!(state.snapshot().cancelled_request_count, 1);

    let response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": "hover-1",
            "method": "textDocument/hover",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "position": {
                    "line": 0,
                    "character": 2,
                },
            },
        }),
    );

    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/error/code")),
        Some(&json!(REQUEST_CANCELLED_ERROR_CODE)),
    );
    assert_eq!(state.snapshot().cancelled_request_count, 0);
}

#[test]
fn bounds_late_cancel_request_cache() {
    let mut state = LspShellState::default();
    for id in 0..=omena_incremental::DEFAULT_INCREMENTAL_CANCELLATION_LIMIT {
        handle_lsp_message(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "method": CANCEL_REQUEST_METHOD,
                "params": {
                    "id": id,
                },
            }),
        );
    }

    assert_eq!(state.snapshot().cancelled_request_count, 1);
}

#[test]
fn honors_feature_configuration_toggles() {
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
                    "text": ".root { color: red; }",
                },
            },
        }),
    );

    let enabled_hover = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "textDocument/hover",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "position": {
                    "line": 0,
                    "character": 2,
                },
            },
        }),
    );
    assert_eq!(
        enabled_hover
            .as_ref()
            .and_then(|value| value.pointer("/result/contents/kind")),
        Some(&json!("markdown")),
    );

    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeConfiguration",
            "params": {
                "settings": {
                    "cssModuleExplainer": {
                        "features": {
                            "hover": false,
                        },
                        "diagnostics": {
                            "severity": "error",
                        },
                    },
                },
            },
        }),
    );

    let disabled_hover = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/hover",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "position": {
                    "line": 0,
                    "character": 2,
                },
            },
        }),
    );
    assert_eq!(
        disabled_hover
            .as_ref()
            .and_then(|value| value.get("result")),
        Some(&Value::Null),
    );
    assert!(!state.snapshot().features.hover);
    assert_eq!(state.snapshot().diagnostics.severity, 1);
}

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
