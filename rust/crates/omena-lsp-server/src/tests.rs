use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn fixture_parent<'a>(
    path: &'a Path,
    context: &'static str,
) -> Result<&'a Path, Box<dyn std::error::Error>> {
    path.parent()
        .ok_or_else(|| std::io::Error::other(context).into())
}

fn fixture_find(
    source: &str,
    needle: &str,
    context: &'static str,
) -> Result<usize, Box<dyn std::error::Error>> {
    source
        .find(needle)
        .ok_or_else(|| std::io::Error::other(context).into())
}

fn assert_source_binding_target(state: &LspShellState, source_uri: &str, expected_style_uri: &str) {
    let imported_style_bindings = state
        .document(source_uri)
        .map(|document| document.source_syntax_index.imported_style_bindings.clone());
    assert_eq!(
        imported_style_bindings.as_deref(),
        Some(
            [ImportedStyleBinding {
                binding: "styles".to_string(),
                style_uri: expected_style_uri.to_string(),
            }]
            .as_slice()
        ),
    );
}

fn assert_definition_response_single_target(response: &Option<Value>, expected_uri: &str) {
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/uri")),
        Some(&json!(expected_uri)),
    );
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/1/uri")),
        None,
    );
}

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

#[test]
fn resolves_style_hover_candidates_from_opened_style_documents() {
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
                        "uri": "file:///workspace-a",
                        "name": "workspace-a",
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
                    "uri": "file:///workspace-a/src/App.tsx",
                    "languageId": "typescriptreact",
                    "version": 1,
                    "text": "import styles from \"./App.module.scss\";\nconst view = <div className={styles.root} />;",
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
                    "uri": "file:///workspace-a/src/App.module.scss",
                    "languageId": "scss",
                    "version": 1,
                    "text": ".root { color: var(--brand); }\n.theme { --brand: red; }",
                },
            },
        }),
    );

    let response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": STYLE_HOVER_CANDIDATES_REQUEST,
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
            .and_then(|value| value.pointer("/result/product")),
        Some(&json!("omena-lsp-server.style-hover-candidates")),
    );
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/candidateCount")),
        Some(&json!(1)),
    );
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/candidates/0/name")),
        Some(&json!("root")),
    );
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/candidates/0/range")),
        Some(&json!({
            "start": {
                "line": 0,
                "character": 1,
            },
            "end": {
                "line": 0,
                "character": 5,
            },
        })),
    );
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/workspaceFolderUri")),
        Some(&json!("file:///workspace-a")),
    );

    let custom_property_ref_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": STYLE_HOVER_CANDIDATES_REQUEST,
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "position": {
                    "line": 0,
                    "character": 21,
                },
            },
        }),
    );
    assert_eq!(
        custom_property_ref_response
            .as_ref()
            .and_then(|value| value.pointer("/result/candidates/0/kind")),
        Some(&json!("customPropertyReference")),
    );
    assert_eq!(
        custom_property_ref_response
            .as_ref()
            .and_then(|value| value.pointer("/result/candidates/0/range")),
        Some(&json!({
            "start": {
                "line": 0,
                "character": 19,
            },
            "end": {
                "line": 0,
                "character": 26,
            },
        })),
    );

    let custom_property_decl_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": STYLE_HOVER_CANDIDATES_REQUEST,
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "position": {
                    "line": 1,
                    "character": 11,
                },
            },
        }),
    );
    assert_eq!(
        custom_property_decl_response
            .as_ref()
            .and_then(|value| value.pointer("/result/candidates/0/kind")),
        Some(&json!("customPropertyDeclaration")),
    );
    assert_eq!(
        custom_property_decl_response
            .as_ref()
            .and_then(|value| value.pointer("/result/candidates/0/range")),
        Some(&json!({
            "start": {
                "line": 1,
                "character": 9,
            },
            "end": {
                "line": 1,
                "character": 16,
            },
        })),
    );

    let hover_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 5,
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
        hover_response
            .as_ref()
            .and_then(|value| value.pointer("/result/contents/kind")),
        Some(&json!("markdown")),
    );
    assert_eq!(
        hover_response
            .as_ref()
            .and_then(|value| value.pointer("/result/range")),
        Some(&json!({
            "start": {
                "line": 0,
                "character": 1,
            },
            "end": {
                "line": 0,
                "character": 5,
            },
        })),
    );

    let definition_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "position": {
                    "line": 0,
                    "character": 21,
                },
            },
        }),
    );
    assert_eq!(
        definition_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/uri")),
        Some(&json!("file:///workspace-a/src/App.module.scss")),
    );
    assert_eq!(
        definition_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/range")),
        Some(&json!({
            "start": {
                "line": 1,
                "character": 9,
            },
            "end": {
                "line": 1,
                "character": 16,
            },
        })),
    );

    let references_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "textDocument/references",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "position": {
                    "line": 0,
                    "character": 21,
                },
                "context": {
                    "includeDeclaration": true,
                },
            },
        }),
    );
    assert_eq!(
        references_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/range")),
        Some(&json!({
            "start": {
                "line": 0,
                "character": 19,
            },
            "end": {
                "line": 0,
                "character": 26,
            },
        })),
    );
    assert_eq!(
        references_response
            .as_ref()
            .and_then(|value| value.pointer("/result/1/range")),
        Some(&json!({
            "start": {
                "line": 1,
                "character": 9,
            },
            "end": {
                "line": 1,
                "character": 16,
            },
        })),
    );

    let completion_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 8,
            "method": "textDocument/completion",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "position": {
                    "line": 0,
                    "character": 20,
                },
            },
        }),
    );
    assert_eq!(
        completion_response
            .as_ref()
            .and_then(|value| value.pointer("/result/isIncomplete")),
        Some(&json!(false)),
    );
    assert_eq!(
        completion_response
            .as_ref()
            .and_then(|value| value.pointer("/result/items/0/label")),
        Some(&json!("--brand")),
    );
    assert_eq!(
        completion_response
            .as_ref()
            .and_then(|value| value.pointer("/result/items/0/data/rankingSource")),
        Some(&json!("sameFileSourceOrderCascade")),
    );
    assert_eq!(
        completion_response
            .as_ref()
            .and_then(|value| value.pointer("/result/items"))
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(1),
    );

    let prepare_rename_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 9,
            "method": "textDocument/prepareRename",
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
        prepare_rename_response
            .as_ref()
            .and_then(|value| value.pointer("/result/placeholder")),
        Some(&json!("root")),
    );

    let rename_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 10,
            "method": "textDocument/rename",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "position": {
                    "line": 0,
                    "character": 21,
                },
                "newName": "--accent",
            },
        }),
    );
    assert_eq!(
        rename_response.as_ref().and_then(|value| value
            .pointer("/result/changes/file:~1~1~1workspace-a~1src~1App.module.scss/0/newText")),
        Some(&json!("--accent")),
    );
    assert_eq!(
        rename_response.as_ref().and_then(|value| value
            .pointer("/result/changes/file:~1~1~1workspace-a~1src~1App.module.scss/1/newText")),
        Some(&json!("--accent")),
    );

    let code_lens_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 11,
            "method": "textDocument/codeLens",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
            },
        }),
    );
    assert_eq!(
        code_lens_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/command/title")),
        Some(&json!("1 reference")),
    );
    assert_eq!(
        code_lens_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/command/command")),
        Some(&json!("editor.action.showReferences")),
    );
    assert_eq!(
        code_lens_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/command/arguments/2/0/range")),
        Some(&json!({
            "start": {
                "line": 1,
                "character": 36,
            },
            "end": {
                "line": 1,
                "character": 40,
            },
        })),
    );
}

#[test]
fn resolves_query_owned_cascade_and_context_requests_from_opened_style_documents() {
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
                    "text": ":root { --surface: white; }\n:root { --surface: black; }\n@layer components {\n  @container card (min-width: 20rem) {\n    @scope (.button) {\n      .button { color: var(--surface); }\n    }\n  }\n}\n",
                },
            },
        }),
    );

    let cascade_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": CASCADE_AT_POSITION_REQUEST,
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "position": {
                    "line": 5,
                    "character": 28,
                },
            },
        }),
    );
    assert_eq!(
        cascade_response
            .as_ref()
            .and_then(|value| value.pointer("/result/product")),
        Some(&json!("omena-query.read-cascade-at-position")),
    );
    assert_eq!(
        cascade_response
            .as_ref()
            .and_then(|value| value.pointer("/result/status")),
        Some(&json!("resolved")),
    );
    assert_eq!(
        cascade_response
            .as_ref()
            .and_then(|value| value.pointer("/result/referenceName")),
        Some(&json!("--surface")),
    );
    assert_eq!(
        cascade_response
            .as_ref()
            .and_then(|value| value.pointer("/result/cascadeEngine")),
        Some(&json!("omena-cascade")),
    );

    let context_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": STYLE_CONTEXT_INDEX_REQUEST,
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
            },
        }),
    );
    assert_eq!(
        context_response
            .as_ref()
            .and_then(|value| value.pointer("/result/product")),
        Some(&json!("omena-query.style-context-index")),
    );
    assert_eq!(
        context_response
            .as_ref()
            .and_then(|value| value.pointer("/result/contextIndexSource")),
        Some(&json!("omena-semantic.style-context-index")),
    );
    assert_eq!(
        context_response
            .as_ref()
            .and_then(|value| value.pointer("/result/contextIndex/layerIndex/namedLayerCount")),
        Some(&json!(1)),
    );
    assert_eq!(
        context_response.as_ref().and_then(|value| {
            value.pointer("/result/contextIndex/containerIndex/namedContainerCount")
        }),
        Some(&json!(1)),
    );
    assert_eq!(
        context_response
            .as_ref()
            .and_then(|value| value.pointer("/result/contextIndex/scopeIndex/scopes"))
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(1),
    );
    assert_eq!(
        context_response
            .as_ref()
            .and_then(|value| value.pointer("/result/contextIndex/scopeIndex/scopedSelectorCount")),
        Some(&json!(1)),
    );
}

#[test]
fn resolves_sass_internal_symbols_through_wildcard_import_targets() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-sass-symbols-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let source_style_path = workspace_path.join("src/Card.module.scss");
    let target_style_path = workspace_path.join("src/shared/_utils.scss");
    fs::create_dir_all(fixture_parent(
        target_style_path.as_path(),
        "target style fixture path has parent directory",
    )?)?;
    fs::create_dir_all(fixture_parent(
        source_style_path.as_path(),
        "source style fixture path has parent directory",
    )?)?;
    fs::write(
        workspace_path.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "$shared/*": ["src/shared/*"]
    }
  }
}"#,
    )?;
    let source_text = "@import \"$shared/utils\";\n.title {\n  @include defign_typography20;\n  border-top: 1px solid $defign_gray200;\n}\n";
    let target_text = "$defign_gray200: #eee;\n@mixin defign_typography20 { font-size: 20px; }\n";
    fs::write(source_style_path.as_path(), source_text)?;
    fs::write(target_style_path.as_path(), target_text)?;

    let workspace_uri = path_to_file_uri(workspace_path.as_path());
    let source_uri = path_to_file_uri(source_style_path.as_path());
    let target_uri = path_to_file_uri(target_style_path.as_path());
    let mixin_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "defign_typography20",
            "source fixture contains mixin include",
        )?,
    );
    let variable_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "$defign_gray200",
            "source fixture contains variable reference",
        )? + 1,
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
                        "name": "workspace-a",
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
                    "languageId": "scss",
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
                    "uri": target_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": target_text,
                },
            },
        }),
    );

    let mixin_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": mixin_position,
            },
        }),
    );
    assert_eq!(
        mixin_definition
            .as_ref()
            .and_then(|value| value.pointer("/result/0/uri")),
        Some(&json!(target_uri)),
    );
    assert_eq!(
        mixin_definition
            .as_ref()
            .and_then(|value| value.pointer("/result/0/range")),
        Some(&json!({
            "start": {
                "line": 1,
                "character": 7,
            },
            "end": {
                "line": 1,
                "character": 26,
            },
        })),
    );

    let variable_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": variable_position,
            },
        }),
    );
    assert_eq!(
        variable_definition
            .as_ref()
            .and_then(|value| value.pointer("/result/0/uri")),
        Some(&json!(target_uri)),
    );
    assert_eq!(
        variable_definition
            .as_ref()
            .and_then(|value| value.pointer("/result/0/range")),
        Some(&json!({
            "start": {
                "line": 0,
                "character": 1,
            },
            "end": {
                "line": 0,
                "character": 15,
            },
        })),
    );

    let variable_hover = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "textDocument/hover",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": variable_position,
            },
        }),
    );
    assert_eq!(
        variable_hover
            .as_ref()
            .and_then(|value| value.pointer("/result/contents/kind")),
        Some(&json!("markdown")),
    );
    assert_eq!(
        variable_hover
            .as_ref()
            .and_then(|value| value.pointer("/result/range")),
        Some(&json!({
            "start": {
                "line": 3,
                "character": 24,
            },
            "end": {
                "line": 3,
                "character": 39,
            },
        })),
    );
    let variable_hover_text = variable_hover
        .as_ref()
        .and_then(|value| value.pointer("/result/contents/value"))
        .and_then(Value::as_str)
        .ok_or_else(|| std::io::Error::other("variable hover should render markdown"))?;
    assert!(variable_hover_text.contains("Value: `#eee`"));
    assert!(variable_hover_text.contains("$defign_gray200: #eee;"));

    let mixin_hover = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "textDocument/hover",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": mixin_position,
            },
        }),
    );
    let mixin_hover_text = mixin_hover
        .as_ref()
        .and_then(|value| value.pointer("/result/contents/value"))
        .and_then(Value::as_str)
        .ok_or_else(|| std::io::Error::other("mixin hover should render markdown"))?;
    assert!(mixin_hover_text.contains("@mixin defign_typography20"));
    assert!(mixin_hover_text.contains("font-size: 20px;"));

    let _ = fs::remove_dir_all(workspace_path.as_path());
    Ok(())
}

#[test]
fn resolves_sass_namespace_symbols_through_forwarded_alias_targets() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-sass-forward-symbols-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let source_path = workspace_path.join("src").join("App.module.scss");
    let index_path = workspace_path
        .join("src")
        .join("shared")
        .join("_index.scss");
    let tokens_path = workspace_path
        .join("src")
        .join("shared")
        .join("_tokens.scss");
    fs::create_dir_all(fixture_parent(
        tokens_path.as_path(),
        "tokens fixture path has parent directory",
    )?)?;
    fs::write(
        workspace_path.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "$shared/*": ["src/shared/*"]
    }
  }
}"#,
    )?;
    fs::write(index_path.as_path(), r#"@forward "./tokens";"#)?;
    let target_text = r#"$gap: 1rem;
@mixin raised { box-shadow: none; }
@function tone($value) { @return $value; }
"#;
    fs::write(tokens_path.as_path(), target_text)?;
    let source_text = r#"@use "$shared/index" as tokens;
.button {
  color: tokens.$gap;
  @include tokens.raised;
  border-color: tokens.tone(tokens.$gap);
}
"#;
    fs::write(source_path.as_path(), source_text)?;

    let workspace_uri = path_to_file_uri(workspace_path.as_path());
    let source_uri = path_to_file_uri(source_path.as_path());
    let index_uri = path_to_file_uri(index_path.as_path());
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
                        "name": "workspace-a",
                    },
                ],
            },
        }),
    );
    for (uri, text) in [
        (source_uri.as_str(), source_text),
        (index_uri.as_str(), r#"@forward "./tokens";"#),
        (tokens_uri.as_str(), target_text),
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

    let gap_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "$gap",
            "source fixture contains namespaced variable",
        )?,
    );
    let gap_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": gap_position,
            },
        }),
    );
    assert_eq!(
        gap_definition
            .as_ref()
            .and_then(|value| value.pointer("/result/0/uri")),
        Some(&json!(tokens_uri)),
    );

    let mixin_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "raised",
            "source fixture contains namespaced mixin",
        )?,
    );
    let mixin_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": mixin_position,
            },
        }),
    );
    assert_eq!(
        mixin_definition
            .as_ref()
            .and_then(|value| value.pointer("/result/0/uri")),
        Some(&json!(tokens_uri)),
    );

    let function_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "tone",
            "source fixture contains namespaced function",
        )?,
    );
    let function_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": function_position,
            },
        }),
    );
    assert_eq!(
        function_definition
            .as_ref()
            .and_then(|value| value.pointer("/result/0/uri")),
        Some(&json!(tokens_uri)),
    );

    let _ = fs::remove_dir_all(workspace_path.as_path());
    Ok(())
}

#[test]
fn resolves_classnames_bind_source_definition_from_opened_documents() {
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
                        "uri": "file:///workspace-a",
                        "name": "workspace-a",
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
                    "uri": "file:///workspace-a/src/App.tsx",
                    "languageId": "typescriptreact",
                    "version": 1,
                    "text": "import bind from \"classnames/bind\";\nimport styles from \"./styles.module.scss\";\nconst cx = bind.bind(styles);\nexport const className = cx(\"root\");",
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
                    "uri": "file:///workspace-a/src/styles.module.scss",
                    "languageId": "scss",
                    "version": 1,
                    "text": ".root { display: block; }",
                },
            },
        }),
    );

    let definition_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.tsx",
                },
                "position": {
                    "line": 3,
                    "character": 30,
                },
            },
        }),
    );

    assert_eq!(
        definition_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/uri")),
        Some(&json!("file:///workspace-a/src/styles.module.scss")),
    );
    assert_eq!(
        definition_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/range")),
        Some(&json!({
            "start": {
                "line": 0,
                "character": 1,
            },
            "end": {
                "line": 0,
                "character": 5,
            },
        })),
    );
}

#[test]
fn projects_tsgo_type_facts_for_typed_cx_identifiers_and_template_holes() -> TestResult {
    let source_uri = "file:///workspace-a/src/App.tsx";
    let style_uri = "file:///workspace-a/src/App.module.scss";
    let source_text = r#"import bind from "classnames/bind";
import styles from "./App.module.scss";
const cx = bind.bind(styles);
interface BadgeProps { size: "medium" | "small"; fontSize?: 10 | 12; }
export function Badge({ size, fontSize }: BadgeProps) {
  return <span className={cx(size, `font-size-${fontSize}`)} />;
}"#;
    let style_text = ".medium { color: red; }\n.small { color: blue; }\n.font-size-10 { font-size: 10px; }\n.font-size-12 { font-size: 12px; }";

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
                        "uri": "file:///workspace-a",
                        "name": "workspace-a",
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
                    "text": style_text,
                },
            },
        }),
    );

    let type_fact_targets = state
        .document(source_uri)
        .ok_or_else(|| std::io::Error::other("source document should be indexed"))?
        .source_syntax_index
        .type_fact_targets
        .clone();
    let size_target = type_fact_targets
        .iter()
        .find(|target| &source_text[target.byte_span.start..target.byte_span.end] == "size")
        .ok_or_else(|| std::io::Error::other("size type fact target should exist"))?;
    let font_size_target = type_fact_targets
        .iter()
        .find(|target| &source_text[target.byte_span.start..target.byte_span.end] == "fontSize")
        .ok_or_else(|| std::io::Error::other("fontSize type fact target should exist"))?;
    apply_source_type_fact_results_to_document(
        &mut state,
        source_uri,
        &[
            TsgoTypeFactResultEntryV0 {
                file_path: "/workspace-a/src/App.tsx".to_string(),
                expression_id: size_target.expression_id.clone(),
                resolved_type: TsgoResolvedTypeV0 {
                    kind: "union",
                    values: vec!["medium".to_string(), "small".to_string()],
                },
            },
            TsgoTypeFactResultEntryV0 {
                file_path: "/workspace-a/src/App.tsx".to_string(),
                expression_id: font_size_target.expression_id.clone(),
                resolved_type: TsgoResolvedTypeV0 {
                    kind: "union",
                    values: vec!["10".to_string(), "12".to_string()],
                },
            },
        ],
    );

    let size_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": parser_position_for_byte_offset(source_text, size_target.byte_span.start),
            },
        }),
    );
    let size_results = size_definition
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("size definition should return results"))?;
    assert_eq!(size_results.len(), 2);
    assert_eq!(size_results[0].get("uri"), Some(&json!(style_uri)));
    assert_eq!(size_results[1].get("uri"), Some(&json!(style_uri)));

    let font_size_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": parser_position_for_byte_offset(source_text, font_size_target.byte_span.start),
            },
        }),
    );
    let font_size_results = font_size_definition
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("fontSize definition should return results"))?;
    assert_eq!(font_size_results.len(), 2);

    let size_hover = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "textDocument/hover",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": parser_position_for_byte_offset(source_text, size_target.byte_span.start),
            },
        }),
    );
    let hover_text = size_hover
        .as_ref()
        .and_then(|value| value.pointer("/result/contents/value"))
        .and_then(Value::as_str)
        .ok_or_else(|| std::io::Error::other("size hover should render markdown"))?;
    assert!(hover_text.contains("`.medium`"));
    assert!(hover_text.contains("`.small`"));

    let size_references = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "textDocument/references",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": parser_position_for_byte_offset(source_text, size_target.byte_span.start),
                "context": {
                    "includeDeclaration": true,
                },
            },
        }),
    );
    let reference_results = size_references
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("size references should return results"))?;
    assert!(
        reference_results
            .iter()
            .any(|location| location.get("uri") == Some(&json!(style_uri)))
    );
    assert!(
        reference_results
            .iter()
            .any(|location| location.get("uri") == Some(&json!(source_uri)))
    );

    let size_prepare_rename = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "textDocument/prepareRename",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": parser_position_for_byte_offset(source_text, size_target.byte_span.start),
            },
        }),
    );
    let placeholder = size_prepare_rename
        .as_ref()
        .and_then(|value| value.pointer("/result/placeholder"))
        .and_then(Value::as_str)
        .ok_or_else(|| std::io::Error::other("size prepareRename should use CSS selector path"))?;
    assert!(matches!(placeholder, "medium" | "small"));

    let size_rename = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "textDocument/rename",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": parser_position_for_byte_offset(source_text, size_target.byte_span.start),
                "newName": "large",
            },
        }),
    );
    let style_edits = size_rename
        .as_ref()
        .and_then(|value| value.pointer("/result/changes"))
        .and_then(|changes| changes.get(style_uri))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("size rename should produce style edits"))?;
    assert!(!style_edits.is_empty());
    Ok(())
}

#[test]
fn indexes_sass_map_prefix_include_generated_selectors_for_source_prefixes() -> TestResult {
    let source_uri = "file:///workspace-a/src/App.tsx";
    let style_uri = "file:///workspace-a/src/App.module.scss";
    let source_text = r#"import bind from "classnames/bind";
import styles from "./App.module.scss";
const cx = bind.bind(styles);
export const view = <span className={cx(color && `color-${color}`)} />;
"#;
    let style_text = r#"@include setAllStyle(
  ("green": #0f0, "blue": #00f),
  background-color,
  ".primary.fill",
  $prefix: "color"
);
"#;
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
                        "uri": "file:///workspace-a",
                        "name": "workspace-a",
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
                    "text": style_text,
                },
            },
        }),
    );

    let color_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "color-${color}",
            "source fixture contains color template prefix",
        )?,
    );
    let definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": color_position,
            },
        }),
    );
    let results = definition
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("color prefix definition should return results"))?;
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].get("uri"), Some(&json!(style_uri)));
    assert_eq!(results[1].get("uri"), Some(&json!(style_uri)));
    Ok(())
}

#[test]
fn narrows_source_completion_candidates_by_property_access_prefix() -> TestResult {
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
                        "uri": "file:///workspace-a",
                        "name": "workspace-a",
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
                    "uri": "file:///workspace-a/src/App.tsx",
                    "languageId": "typescriptreact",
                    "version": 1,
                    "text": "import styles from \"./App.module.scss\";\nconst view = styles.ro;",
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
                    "uri": "file:///workspace-a/src/App.module.scss",
                    "languageId": "scss",
                    "version": 1,
                    "text": ".root { display: block; }\n.row { display: flex; }\n.active { color: red; }",
                },
            },
        }),
    );

    let completion_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/completion",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.tsx",
                },
                "position": {
                    "line": 1,
                    "character": 22,
                },
            },
        }),
    );

    let items = completion_response
        .as_ref()
        .and_then(|value| value.pointer("/result/items"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("completion response should contain items"))?;
    let labels: Vec<String> = items
        .iter()
        .filter_map(|item| {
            item.get("label")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        })
        .collect();
    assert_eq!(labels, vec!["root".to_string(), "row".to_string()]);
    assert_eq!(
        items
            .first()
            .and_then(|item| item.pointer("/data/rankingSource")),
        Some(&json!("targetAndPrefixNarrowing")),
    );
    assert!(
        items
            .first()
            .and_then(|item| item.get("sortText"))
            .and_then(Value::as_str)
            .is_some_and(|sort_text| sort_text.starts_with("10-00-00-"))
    );
    Ok(())
}

#[test]
fn ranks_source_completion_with_value_domain_projection() -> TestResult {
    let source_uri = "file:///workspace-a/src/App.tsx";
    let style_uri = "file:///workspace-a/src/App.module.scss";
    let source_text = r#"import bind from "classnames/bind";
import styles from "./App.module.scss";
const cx = bind.bind(styles);
const view = <div className={cx(`item--${variant}`)} />;
"#;
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
                        "uri": "file:///workspace-a",
                        "name": "workspace-a",
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
                    "text": ".item--large {}\n.item--primary {}\n.item--secondary {}\n.item--muted {}\n",
                },
            },
        }),
    );

    let expression_id = state
        .document(source_uri)
        .and_then(|document| document.source_syntax_index.type_fact_targets.first())
        .map(|target| target.expression_id.clone())
        .ok_or_else(|| std::io::Error::other("expected a source type-fact target"))?;
    apply_source_type_fact_results_to_document(
        &mut state,
        source_uri,
        &[TsgoTypeFactResultEntryV0 {
            file_path: "/workspace-a/src/App.tsx".to_string(),
            expression_id,
            resolved_type: TsgoResolvedTypeV0 {
                kind: "union",
                values: vec!["primary".to_string(), "secondary".to_string()],
            },
        }],
    );

    let position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "variant}`",
            "source fixture contains dynamic selector expression",
        )?,
    );
    let completion_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/completion",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": position,
            },
        }),
    );

    let items = completion_response
        .as_ref()
        .and_then(|value| value.pointer("/result/items"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("completion response should contain items"))?;
    let labels = items
        .iter()
        .filter_map(|item| item.get("label").and_then(Value::as_str))
        .collect::<Vec<_>>();
    assert_eq!(
        labels,
        vec![
            "item--primary",
            "item--secondary",
            "item--large",
            "item--muted"
        ]
    );
    assert_eq!(
        items
            .first()
            .and_then(|item| item.pointer("/data/rankingSource")),
        Some(&json!("valueDomainSelectorProjection")),
    );
    assert!(
        items
            .first()
            .and_then(|item| item.get("sortText"))
            .and_then(Value::as_str)
            .is_some_and(|sort_text| sort_text.starts_with("00-00-"))
    );
    Ok(())
}

#[test]
fn resolves_classnames_bind_source_definition_through_tsconfig_path_alias() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-path-alias-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let target_style_path = workspace_path
        .join("src")
        .join("domain")
        .join("components")
        .join("some-component.module.scss");
    fs::create_dir_all(fixture_parent(
        target_style_path.as_path(),
        "target style fixture path has parent directory",
    )?)?;
    fs::write(
        workspace_path.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "$domain/*": ["src/domain/*"]
    }
  }
}"#,
    )?;
    fs::write(target_style_path.as_path(), ".article { display: block; }")?;

    let workspace_uri = path_to_file_uri(workspace_path.as_path());
    let source_uri = path_to_file_uri(workspace_path.join("src/App.tsx").as_path());
    let target_style_uri = path_to_file_uri(target_style_path.as_path());
    let unrelated_style_uri =
        path_to_file_uri(workspace_path.join("src/other.module.scss").as_path());

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
                        "name": "workspace-a",
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
                    "text": "import bind from \"classnames/bind\";\nimport styles from \"$domain/components/some-component.module.scss\";\nconst cx = bind.bind(styles);\nexport const className = cx(\"article\");",
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
                    "uri": target_style_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": ".article { display: block; }",
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
                    "uri": unrelated_style_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": ".article { color: red; }",
                },
            },
        }),
    );

    let source_index = state
        .document(source_uri.as_str())
        .map(|document| document.source_syntax_index.clone());
    assert_eq!(
        source_index
            .as_ref()
            .map(|index| index.imported_style_bindings.as_slice()),
        Some(
            [ImportedStyleBinding {
                binding: "styles".to_string(),
                style_uri: target_style_uri.clone(),
            }]
            .as_slice()
        ),
    );

    let definition_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": {
                    "line": 3,
                    "character": 31,
                },
            },
        }),
    );

    assert_eq!(
        definition_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/uri")),
        Some(&json!(target_style_uri)),
    );
    assert_eq!(
        definition_response
            .as_ref()
            .and_then(|value| value.pointer("/result/1/uri")),
        None,
    );

    let _ = fs::remove_dir_all(workspace_path.as_path());
    Ok(())
}

#[test]
fn resolves_classnames_bind_source_definition_through_tsconfig_extends_alias() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-path-alias-extends-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let target_style_path = workspace_path
        .join("src")
        .join("shared")
        .join("some-component.module.scss");
    let config_dir = workspace_path.join("config");
    fs::create_dir_all(fixture_parent(
        target_style_path.as_path(),
        "target style fixture path has parent directory",
    )?)?;
    fs::create_dir_all(config_dir.as_path())?;
    fs::write(
        config_dir.join("base.json"),
        r#"{"compilerOptions":{"baseUrl":"..","paths":{"$shared/*":["src/shared/*"]}}}"#,
    )?;
    fs::write(
        workspace_path.join("tsconfig.json"),
        r#"{"extends":"./config/base"}"#,
    )?;
    fs::write(target_style_path.as_path(), ".article { display: block; }")?;

    let workspace_uri = path_to_file_uri(workspace_path.as_path());
    let source_uri = path_to_file_uri(workspace_path.join("src/App.tsx").as_path());
    let target_style_uri = path_to_file_uri(target_style_path.as_path());

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
                        "name": "workspace-a",
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
                    "text": "import bind from \"classnames/bind\";\nimport styles from \"$shared/some-component.module.scss\";\nconst cx = bind.bind(styles);\nexport const className = cx(\"article\");",
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
                    "uri": target_style_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": ".article { display: block; }",
                },
            },
        }),
    );

    assert_source_binding_target(&state, source_uri.as_str(), target_style_uri.as_str());
    let definition_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": {
                    "line": 3,
                    "character": 31,
                },
            },
        }),
    );
    assert_definition_response_single_target(&definition_response, target_style_uri.as_str());

    let _ = fs::remove_dir_all(workspace_path.as_path());
    Ok(())
}

#[test]
fn resolves_classnames_bind_source_definition_through_vite_bundler_alias() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-bundler-alias-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let target_style_path = workspace_path
        .join("src")
        .join("styles")
        .join("some-component.module.scss");
    fs::create_dir_all(fixture_parent(
        target_style_path.as_path(),
        "target style fixture path has parent directory",
    )?)?;
    fs::write(
        workspace_path.join("vite.config.ts"),
        r#"export default { resolve: { alias: { "@styles": "./src/styles" } } };"#,
    )?;
    fs::write(target_style_path.as_path(), ".article { display: block; }")?;

    let workspace_uri = path_to_file_uri(workspace_path.as_path());
    let source_uri = path_to_file_uri(workspace_path.join("src/App.tsx").as_path());
    let target_style_uri = path_to_file_uri(target_style_path.as_path());
    let unrelated_style_uri =
        path_to_file_uri(workspace_path.join("src/other.module.scss").as_path());

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
                        "name": "workspace-a",
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
                    "text": "import bind from \"classnames/bind\";\nimport styles from \"@styles/some-component.module.scss\";\nconst cx = bind.bind(styles);\nexport const className = cx(\"article\");",
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
                    "uri": target_style_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": ".article { display: block; }",
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
                    "uri": unrelated_style_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": ".article { color: red; }",
                },
            },
        }),
    );

    let source_index = state
        .document(source_uri.as_str())
        .map(|document| document.source_syntax_index.clone());
    assert_eq!(
        source_index
            .as_ref()
            .map(|index| index.imported_style_bindings.as_slice()),
        Some(
            [ImportedStyleBinding {
                binding: "styles".to_string(),
                style_uri: target_style_uri.clone(),
            }]
            .as_slice()
        ),
    );

    let definition_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": {
                    "line": 3,
                    "character": 31,
                },
            },
        }),
    );

    assert_eq!(
        definition_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/uri")),
        Some(&json!(target_style_uri)),
    );
    assert_eq!(
        definition_response
            .as_ref()
            .and_then(|value| value.pointer("/result/1/uri")),
        None,
    );

    let _ = fs::remove_dir_all(workspace_path.as_path());
    Ok(())
}

#[test]
fn resolves_classnames_bind_source_definition_through_webpack_bundler_alias() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-webpack-alias-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let target_style_path = workspace_path
        .join("src")
        .join("theme")
        .join("deep")
        .join("some-component.module.scss");
    let specific_style_path = workspace_path
        .join("src")
        .join("specific")
        .join("some-component.module.scss");
    fs::create_dir_all(fixture_parent(
        target_style_path.as_path(),
        "target style fixture path has parent directory",
    )?)?;
    fs::create_dir_all(fixture_parent(
        specific_style_path.as_path(),
        "specific style fixture path has parent directory",
    )?)?;
    fs::write(
        workspace_path.join("webpack.config.js"),
        r#"module.exports = { resolve: { alias: [{ find: "@theme", replacement: "./src/theme" }, { find: "@theme/deep", replacement: "./src/specific" }] } };"#,
    )?;
    fs::write(target_style_path.as_path(), ".article { display: block; }")?;
    fs::write(
        specific_style_path.as_path(),
        ".article { color: hotpink; }",
    )?;

    let workspace_uri = path_to_file_uri(workspace_path.as_path());
    let source_uri = path_to_file_uri(workspace_path.join("src/App.tsx").as_path());
    let target_style_uri = path_to_file_uri(target_style_path.as_path());
    let specific_style_uri = path_to_file_uri(specific_style_path.as_path());
    let unrelated_style_uri =
        path_to_file_uri(workspace_path.join("src/other.module.scss").as_path());

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
                        "name": "workspace-a",
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
                    "text": "import bind from \"classnames/bind\";\nimport styles from \"@theme/deep/some-component.module.scss\";\nconst cx = bind.bind(styles);\nexport const className = cx(\"article\");",
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
                    "uri": target_style_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": ".article { display: block; }",
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
                    "uri": specific_style_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": ".article { color: hotpink; }",
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
                    "uri": unrelated_style_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": ".article { color: red; }",
                },
            },
        }),
    );

    assert_source_binding_target(&state, source_uri.as_str(), target_style_uri.as_str());
    let definition_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": {
                    "line": 3,
                    "character": 31,
                },
            },
        }),
    );

    assert_definition_response_single_target(&definition_response, target_style_uri.as_str());
    let _ = fs::remove_dir_all(workspace_path.as_path());
    Ok(())
}

#[test]
fn refreshes_source_bindings_after_bundler_config_watch_change() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-bundler-alias-refresh-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let old_style_path = workspace_path
        .join("src")
        .join("old")
        .join("some-component.module.scss");
    let new_style_path = workspace_path
        .join("src")
        .join("new")
        .join("some-component.module.scss");
    let config_path = workspace_path.join("vite.config.ts");
    fs::create_dir_all(fixture_parent(
        old_style_path.as_path(),
        "old style fixture path has parent directory",
    )?)?;
    fs::create_dir_all(fixture_parent(
        new_style_path.as_path(),
        "new style fixture path has parent directory",
    )?)?;
    fs::write(
        config_path.as_path(),
        r#"export default { resolve: { alias: { "@styles": "./src/old" } } };"#,
    )?;
    fs::write(old_style_path.as_path(), ".article { display: block; }")?;
    fs::write(new_style_path.as_path(), ".article { color: green; }")?;

    let workspace_uri = path_to_file_uri(workspace_path.as_path());
    let source_uri = path_to_file_uri(workspace_path.join("src/App.tsx").as_path());
    let old_style_uri = path_to_file_uri(old_style_path.as_path());
    let new_style_uri = path_to_file_uri(new_style_path.as_path());
    let config_uri = path_to_file_uri(config_path.as_path());

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
                        "name": "workspace-a",
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
                    "text": "import bind from \"classnames/bind\";\nimport styles from \"@styles/some-component.module.scss\";\nconst cx = bind.bind(styles);\nexport const className = cx(\"article\");",
                },
            },
        }),
    );
    for (uri, text) in [
        (old_style_uri.as_str(), ".article { display: block; }"),
        (new_style_uri.as_str(), ".article { color: green; }"),
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

    assert_source_binding_target(&state, source_uri.as_str(), old_style_uri.as_str());

    fs::write(
        config_path.as_path(),
        r#"export default { resolve: { alias: { "@styles": "./src/new" } } };"#,
    )?;
    let outputs = handle_lsp_message_outputs(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeWatchedFiles",
            "params": {
                "changes": [
                    {
                        "uri": config_uri,
                        "type": 2,
                    },
                ],
            },
        }),
    );

    assert_source_binding_target(&state, source_uri.as_str(), new_style_uri.as_str());
    assert!(
        outputs
            .iter()
            .any(|output| output.pointer("/params/uri") == Some(&json!(source_uri)))
    );

    let definition_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": {
                    "line": 3,
                    "character": 31,
                },
            },
        }),
    );

    assert_eq!(
        definition_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/uri")),
        Some(&json!(new_style_uri)),
    );
    assert_eq!(
        definition_response
            .as_ref()
            .and_then(|value| value.pointer("/result/1/uri")),
        None,
    );

    let _ = fs::remove_dir_all(workspace_path.as_path());
    Ok(())
}

#[test]
fn keeps_sass_resolution_on_cached_bundler_config_until_watch_change() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-bundler-alias-request-cache-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let source_style_path = workspace_path.join("src/App.module.scss");
    let old_style_path = workspace_path.join("src/old/_tokens.scss");
    let new_style_path = workspace_path.join("src/new/_tokens.scss");
    let config_path = workspace_path.join("vite.config.ts");
    fs::create_dir_all(fixture_parent(
        source_style_path.as_path(),
        "source style fixture path has parent directory",
    )?)?;
    fs::create_dir_all(fixture_parent(
        old_style_path.as_path(),
        "old style fixture path has parent directory",
    )?)?;
    fs::create_dir_all(fixture_parent(
        new_style_path.as_path(),
        "new style fixture path has parent directory",
    )?)?;
    fs::write(
        config_path.as_path(),
        r#"export default { resolve: { alias: { "@styles": "./src/old" } } };"#,
    )?;
    let source_text = r#"@use "@styles/tokens" as tokens;
.button { color: tokens.$brand; }
"#;
    fs::write(source_style_path.as_path(), source_text)?;
    fs::write(old_style_path.as_path(), "$brand: red;\n")?;
    fs::write(new_style_path.as_path(), "$brand: green;\n")?;

    let workspace_uri = path_to_file_uri(workspace_path.as_path());
    let source_uri = path_to_file_uri(source_style_path.as_path());
    let old_style_uri = path_to_file_uri(old_style_path.as_path());
    let new_style_uri = path_to_file_uri(new_style_path.as_path());
    let config_uri = path_to_file_uri(config_path.as_path());
    let brand_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "$brand",
            "source fixture contains Sass variable reference",
        )? + 1,
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
                        "name": "workspace-a",
                    },
                ],
            },
        }),
    );
    for (uri, text) in [
        (source_uri.as_str(), source_text),
        (old_style_uri.as_str(), "$brand: red;\n"),
        (new_style_uri.as_str(), "$brand: green;\n"),
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

    fs::write(
        config_path.as_path(),
        r#"export default { resolve: { alias: { "@styles": "./src/new" } } };"#,
    )?;
    let cached_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": brand_position,
            },
        }),
    );
    assert_definition_response_single_target(&cached_definition, old_style_uri.as_str());

    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeWatchedFiles",
            "params": {
                "changes": [
                    {
                        "uri": config_uri,
                        "type": 2,
                    },
                ],
            },
        }),
    );
    let refreshed_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": brand_position,
            },
        }),
    );
    assert_definition_response_single_target(&refreshed_definition, new_style_uri.as_str());

    let _ = fs::remove_dir_all(workspace_path.as_path());
    Ok(())
}

#[test]
fn refreshes_source_bindings_after_tsconfig_watch_change() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-tsconfig-alias-refresh-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let old_style_path = workspace_path
        .join("src")
        .join("old")
        .join("some-component.module.scss");
    let new_style_path = workspace_path
        .join("src")
        .join("new")
        .join("some-component.module.scss");
    let config_path = workspace_path.join("tsconfig.json");
    fs::create_dir_all(fixture_parent(
        old_style_path.as_path(),
        "old style fixture path has parent directory",
    )?)?;
    fs::create_dir_all(fixture_parent(
        new_style_path.as_path(),
        "new style fixture path has parent directory",
    )?)?;
    fs::write(
        config_path.as_path(),
        r#"{"compilerOptions":{"baseUrl":".","paths":{"$styles/*":["src/old/*"]}}}"#,
    )?;
    fs::write(old_style_path.as_path(), ".article { display: block; }")?;
    fs::write(new_style_path.as_path(), ".article { color: green; }")?;

    let workspace_uri = path_to_file_uri(workspace_path.as_path());
    let source_uri = path_to_file_uri(workspace_path.join("src/App.tsx").as_path());
    let old_style_uri = path_to_file_uri(old_style_path.as_path());
    let new_style_uri = path_to_file_uri(new_style_path.as_path());
    let config_uri = path_to_file_uri(config_path.as_path());

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
                        "name": "workspace-a",
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
                    "text": "import bind from \"classnames/bind\";\nimport styles from \"$styles/some-component.module.scss\";\nconst cx = bind.bind(styles);\nexport const className = cx(\"article\");",
                },
            },
        }),
    );
    for (uri, text) in [
        (old_style_uri.as_str(), ".article { display: block; }"),
        (new_style_uri.as_str(), ".article { color: green; }"),
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

    assert_source_binding_target(&state, source_uri.as_str(), old_style_uri.as_str());

    fs::write(
        config_path.as_path(),
        r#"{"compilerOptions":{"baseUrl":".","paths":{"$styles/*":["src/new/*"]}}}"#,
    )?;
    let outputs = handle_lsp_message_outputs(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeWatchedFiles",
            "params": {
                "changes": [
                    {
                        "uri": config_uri,
                        "type": 2,
                    },
                ],
            },
        }),
    );

    assert_source_binding_target(&state, source_uri.as_str(), new_style_uri.as_str());
    assert!(
        outputs
            .iter()
            .any(|output| output.pointer("/params/uri") == Some(&json!(source_uri)))
    );

    let definition_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": {
                    "line": 3,
                    "character": 31,
                },
            },
        }),
    );
    assert_definition_response_single_target(&definition_response, new_style_uri.as_str());

    let _ = fs::remove_dir_all(workspace_path.as_path());
    Ok(())
}

#[test]
fn source_hover_renders_unopened_target_style_rule_from_disk() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-disk-style-hover-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let src_dir = workspace_path.join("src");
    let source_path = src_dir.join("App.tsx");
    let style_path = src_dir.join("App.module.scss");
    fs::create_dir_all(src_dir.as_path())?;
    fs::write(style_path.as_path(), ".foo { color: red; }\n")?;

    let workspace_uri = path_to_file_uri(workspace_path.as_path());
    let source_uri = path_to_file_uri(source_path.as_path());
    let source_text = "import bind from \"classnames/bind\";\nimport styles from \"./App.module.scss\";\nconst cx = bind.bind(styles);\nexport const view = <div className={cx(\"foo\")} />;";
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
                        "name": "workspace",
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

    let hover_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/hover",
            "params": {
                "textDocument": {
                    "uri": path_to_file_uri(source_path.as_path()),
                },
                "position": parser_position_for_byte_offset(
                    source_text,
                    fixture_find(source_text, "\"foo\"", "source fixture contains foo")? + 1,
                ),
            },
        }),
    );
    let hover_text = hover_response
        .as_ref()
        .and_then(|value| value.pointer("/result/contents/value"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    assert!(
        hover_text.contains("color: red"),
        "hover text: {hover_text}"
    );

    let _ = fs::remove_dir_all(workspace_path.as_path());
    Ok(())
}

#[test]
fn sass_symbol_hover_renders_unopened_target_definition_from_disk() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-disk-sass-hover-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let src_dir = workspace_path.join("src");
    let source_style_path = src_dir.join("App.module.scss");
    let token_style_path = src_dir.join("_tokens.scss");
    fs::create_dir_all(src_dir.as_path())?;
    fs::write(token_style_path.as_path(), "$brand: #fff;\n")?;

    let workspace_uri = path_to_file_uri(workspace_path.as_path());
    let source_style_uri = path_to_file_uri(source_style_path.as_path());
    let source_style_text = "@use \"./tokens\" as *;\n.foo { color: $brand; }\n";
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
                        "name": "workspace",
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
                    "uri": source_style_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": source_style_text,
                },
            },
        }),
    );

    let hover_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/hover",
            "params": {
                "textDocument": {
                    "uri": path_to_file_uri(source_style_path.as_path()),
                },
                "position": parser_position_for_byte_offset(
                    source_style_text,
                    fixture_find(
                        source_style_text,
                        "$brand",
                        "style fixture contains brand variable",
                    )? + 1,
                ),
            },
        }),
    );
    let hover_text = hover_response
        .as_ref()
        .and_then(|value| value.pointer("/result/contents/value"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    assert!(hover_text.contains("$brand: #fff"));

    let _ = fs::remove_dir_all(workspace_path.as_path());
    Ok(())
}

#[test]
fn resolves_classnames_bind_dynamic_source_expressions() -> TestResult {
    let source_text = r#"import bind from "classnames/bind";
import styles from "./styles.module.scss";
const cx = bind.bind(styles);
const tone = "item--primary";
const icon = { glyph: "item__icon" };
const prefix = "item--";
export const view = <div className={cx(tone, icon.glyph, `item--${variant}`, { "item--danger": danger, item__label: true }, ok && "item--ok", active ? "item--on" : "item--off", prefix + state)} />;
"#;
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
                        "uri": "file:///workspace-a",
                        "name": "workspace-a",
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
                    "uri": "file:///workspace-a/src/App.tsx",
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
                    "uri": "file:///workspace-a/src/styles.module.scss",
                    "languageId": "scss",
                    "version": 1,
                    "text": ".item--primary {}\n.item__icon {}\n.item--large {}\n.item--danger {}\n.item__label {}\n.item--ok {}\n.item--on {}\n.item--off {}\n.item--muted {}\n",
                },
            },
        }),
    );

    let tone_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "tone,",
            "source fixture contains tone reference",
        )?,
    );
    let tone_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.tsx",
                },
                "position": tone_position,
            },
        }),
    );
    assert_eq!(
        tone_definition
            .as_ref()
            .and_then(|value| value.pointer("/result/0/range")),
        Some(&json!({
            "start": {
                "line": 0,
                "character": 1,
            },
            "end": {
                "line": 0,
                "character": 14,
            },
        })),
    );

    let icon_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "icon.glyph",
            "source fixture contains object property reference",
        )?,
    );
    let icon_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.tsx",
                },
                "position": icon_position,
            },
        }),
    );
    assert_eq!(
        icon_definition
            .as_ref()
            .and_then(|value| value.pointer("/result/0/range")),
        Some(&json!({
            "start": {
                "line": 1,
                "character": 1,
            },
            "end": {
                "line": 1,
                "character": 11,
            },
        })),
    );

    let template_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "`item--",
            "source fixture contains template prefix reference",
        )? + 1,
    );
    let template_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.tsx",
                },
                "position": template_position,
            },
        }),
    );
    let template_targets = template_definition
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(
        template_targets
            .iter()
            .any(|target| target.pointer("/range/start/line") == Some(&json!(2)))
    );
    assert!(
        !template_targets
            .iter()
            .any(|target| target.pointer("/range/start/line") == Some(&json!(1)))
    );

    let object_key_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "item__label",
            "source fixture contains object key reference",
        )?,
    );
    let object_key_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.tsx",
                },
                "position": object_key_position,
            },
        }),
    );
    assert_eq!(
        object_key_definition
            .as_ref()
            .and_then(|value| value.pointer("/result/0/range/start/line")),
        Some(&json!(4)),
    );
    Ok(())
}

#[test]
fn resolves_source_references_from_asi_imports_without_panicking() {
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
                        "uri": "file:///workspace-a",
                        "name": "workspace-a",
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
                    "uri": "file:///workspace-a/src/App.tsx",
                    "languageId": "typescriptreact",
                    "version": 1,
                    "text": "import {WidgetA, WidgetB} from \"@repo/widgets\"\nimport styles from \"./styles.module.scss\"\nconst view = <div className={styles.root} />",
                },
            },
        }),
    );
    let source_index = state
        .document("file:///workspace-a/src/App.tsx")
        .map(|document| document.source_syntax_index.clone());
    assert_eq!(
        source_index
            .as_ref()
            .map(|index| index.imported_style_bindings.as_slice()),
        Some(
            [ImportedStyleBinding {
                binding: "styles".to_string(),
                style_uri: "file:///workspace-a/src/styles.module.scss".to_string(),
            }]
            .as_slice()
        ),
    );

    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/styles.module.scss",
                    "languageId": "scss",
                    "version": 1,
                    "text": ".root { display: block; }",
                },
            },
        }),
    );

    let definition_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.tsx",
                },
                "position": {
                    "line": 2,
                    "character": 37,
                },
            },
        }),
    );

    assert_eq!(
        definition_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/uri")),
        Some(&json!("file:///workspace-a/src/styles.module.scss")),
    );
    assert_eq!(
        definition_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/range")),
        Some(&json!({
            "start": {
                "line": 0,
                "character": 1,
            },
            "end": {
                "line": 0,
                "character": 5,
            },
        })),
    );
}

#[test]
fn opens_source_with_multibyte_escaped_strings_without_panicking() {
    let source_text = r#"import bind from "classnames/bind";
import styles from "./styles.module.scss";
const cx = bind.bind(styles);
const label = "\비";
export const view = <div className={cx("root", label && `상태-${tone}`)} />;
"#;
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
                        "uri": "file:///workspace-a",
                        "name": "workspace-a",
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
                    "uri": "file:///workspace-a/src/App.tsx",
                    "languageId": "typescriptreact",
                    "version": 1,
                    "text": source_text,
                },
            },
        }),
    );

    let source_index = state
        .document("file:///workspace-a/src/App.tsx")
        .map(|document| document.source_syntax_index.clone());
    assert!(
        source_index
            .as_ref()
            .is_some_and(|index| !index.selector_references.is_empty())
    );
}

#[test]
fn workspace_folder_compatibility_normalizes_percent_encoded_route_groups() {
    assert!(workspace_folder_uri_equivalent(
        "file:///workspace/app/(marketing)",
        "file:///workspace/app/%28marketing%29",
    ));
}

#[test]
fn path_to_file_uri_percent_encodes_route_group_paths() {
    let uri = path_to_file_uri(Path::new("/workspace/app/(marketing)/Card.module.scss"));

    assert_eq!(
        uri,
        "file:///workspace/app/%28marketing%29/Card.module.scss"
    );
    assert!(file_uri_equivalent(
        uri.as_str(),
        "file:///workspace/app/(marketing)/Card.module.scss",
    ));
}

#[cfg(unix)]
#[test]
fn document_map_uses_canonical_identity_for_symlinked_document_paths() -> TestResult {
    let root = std::env::temp_dir().join(format!(
        "omena_lsp_symlinked_document_identity_{}_{}",
        std::process::id(),
        current_time_millis()
    ));
    let real_src = root.join("real/src");
    let link_src = root.join("linked-src");
    let real_style = real_src.join("Button.module.scss");
    let linked_style = link_src.join("Button.module.scss");
    fs::create_dir_all(real_src.as_path())?;
    fs::write(real_style.as_path(), ".button { color: red; }")?;
    std::os::unix::fs::symlink(real_src.as_path(), link_src.as_path())?;

    let real_uri = raw_test_file_uri(real_style.as_path());
    let linked_uri = raw_test_file_uri(linked_style.as_path());
    assert!(file_uri_equivalent(real_uri.as_str(), linked_uri.as_str()));

    let mut state = LspShellState::default();
    for (uri, text) in [
        (linked_uri.as_str(), ".button { color: red; }"),
        (real_uri.as_str(), ".button { color: blue; }"),
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

    assert_eq!(state.document_count(), 1);
    assert_eq!(state.open_document_uris.len(), 1);
    assert!(state.document(real_uri.as_str()).is_some());
    assert!(state.document(linked_uri.as_str()).is_some());
    assert_eq!(state.snapshot().documents.len(), 1);
    let _ = fs::remove_dir_all(root.as_path());
    Ok(())
}

#[cfg(unix)]
#[test]
fn resolves_sass_definition_through_symlinked_package_canonical_identity() -> TestResult {
    let root = std::env::temp_dir().join(format!(
        "omena_lsp_symlinked_package_identity_{}_{}",
        std::process::id(),
        current_time_millis()
    ));
    let source = root.join("src/App.module.scss");
    let real_package = root.join(".pnpm/@design+tokens@1.0.0/node_modules/@design/tokens");
    let linked_scope = root.join("node_modules/@design");
    let linked_package = linked_scope.join("tokens");
    let real_style = real_package.join("src/index.scss");
    let linked_style = linked_package.join("src/index.scss");
    fs::create_dir_all(fixture_parent(source.as_path(), "source parent")?)?;
    fs::create_dir_all(fixture_parent(real_style.as_path(), "style parent")?)?;
    fs::create_dir_all(linked_scope.as_path())?;
    fs::write(
        real_package.join("package.json"),
        r#"{"sass":"src/index.scss"}"#,
    )?;
    let source_text = r#"@use "@design/tokens" as tokens;
.button { color: tokens.$brand; }
"#;
    let target_text = "$brand: #fff;\n";
    fs::write(source.as_path(), source_text)?;
    fs::write(real_style.as_path(), target_text)?;
    std::os::unix::fs::symlink(real_package.as_path(), linked_package.as_path())?;

    let workspace_uri = raw_test_file_uri(root.as_path());
    let source_uri = raw_test_file_uri(source.as_path());
    let linked_style_uri = raw_test_file_uri(linked_style.as_path());
    let real_style_uri = raw_test_file_uri(real_style.as_path());
    let brand_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "$brand",
            "source fixture contains Sass variable reference",
        )? + 1,
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
                        "name": "workspace",
                    },
                ],
            },
        }),
    );
    for (uri, text) in [
        (source_uri.as_str(), source_text),
        (linked_style_uri.as_str(), target_text),
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

    let definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": brand_position,
            },
        }),
    );
    let targets = definition
        .as_ref()
        .and_then(|value| value.get("result"))
        .and_then(Value::as_array)
        .ok_or_else(|| {
            std::io::Error::other(format!("expected definition array, got {definition:?}"))
        })?;

    assert!(targets.iter().any(|target| {
        target
            .get("uri")
            .and_then(Value::as_str)
            .is_some_and(|uri| file_uri_equivalent(uri, real_style_uri.as_str()))
    }));
    let _ = fs::remove_dir_all(root.as_path());
    Ok(())
}

#[test]
fn resolves_sass_definition_after_package_manifest_watch_change() -> TestResult {
    let root = std::env::temp_dir().join(format!(
        "omena_lsp_package_manifest_refresh_{}_{}",
        std::process::id(),
        current_time_millis()
    ));
    let source = root.join("src/App.module.scss");
    let package_root = root.join("node_modules/@design/tokens");
    let old_style = package_root.join("old.scss");
    let new_style = package_root.join("new.scss");
    let package_json = package_root.join("package.json");
    fs::create_dir_all(fixture_parent(source.as_path(), "source parent")?)?;
    fs::create_dir_all(fixture_parent(old_style.as_path(), "style parent")?)?;
    fs::write(package_json.as_path(), r#"{"sass":"old.scss"}"#)?;
    let source_text = r#"@use "@design/tokens" as tokens;
.button { color: tokens.$brand; }
"#;
    fs::write(source.as_path(), source_text)?;
    fs::write(old_style.as_path(), "$brand: red;\n")?;
    fs::write(new_style.as_path(), "$brand: green;\n")?;

    let workspace_uri = path_to_file_uri(root.as_path());
    let source_uri = path_to_file_uri(source.as_path());
    let old_style_uri = path_to_file_uri(old_style.as_path());
    let new_style_uri = path_to_file_uri(new_style.as_path());
    let package_json_uri = path_to_file_uri(package_json.as_path());
    let brand_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "$brand",
            "source fixture contains Sass variable reference",
        )? + 1,
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
                        "name": "workspace",
                    },
                ],
            },
        }),
    );
    for (uri, text) in [
        (source_uri.as_str(), source_text),
        (old_style_uri.as_str(), "$brand: red;\n"),
        (new_style_uri.as_str(), "$brand: green;\n"),
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

    let initial_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": brand_position,
            },
        }),
    );
    assert_definition_response_single_target(&initial_definition, old_style_uri.as_str());

    fs::write(package_json.as_path(), r#"{"sass":"new.scss"}"#)?;
    let cached_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": brand_position,
            },
        }),
    );
    assert_definition_response_single_target(&cached_definition, old_style_uri.as_str());

    let outputs = handle_lsp_message_outputs(
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
    assert!(
        outputs
            .iter()
            .any(|output| output.pointer("/params/uri") == Some(&json!(source_uri)))
    );

    let updated_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": brand_position,
            },
        }),
    );
    assert_definition_response_single_target(&updated_definition, new_style_uri.as_str());

    let _ = fs::remove_dir_all(root.as_path());
    Ok(())
}

#[test]
fn resolves_sass_definition_with_configured_package_manifest_path() -> TestResult {
    let root = std::env::temp_dir().join(format!(
        "omena_lsp_package_manifest_setting_{}_{}",
        std::process::id(),
        current_time_millis()
    ));
    let source = root.join("src/App.module.scss");
    let package_root = root.join("node_modules/@design/tokens");
    let override_style = package_root.join("override.scss");
    let override_manifest = package_root.join("package.lsp.json");
    fs::create_dir_all(fixture_parent(source.as_path(), "source parent")?)?;
    fs::create_dir_all(package_root.as_path())?;
    let source_text = r#"@use "pkg:@design/tokens" as tokens;
.button { color: tokens.$brand; }
"#;
    fs::write(source.as_path(), source_text)?;
    fs::write(override_style.as_path(), "$brand: green;\n")?;
    fs::write(override_manifest.as_path(), r#"{"sass":"./override.scss"}"#)?;

    let workspace_uri = path_to_file_uri(root.as_path());
    let source_uri = path_to_file_uri(source.as_path());
    let override_style_uri = path_to_file_uri(override_style.as_path());
    let override_manifest_path = override_manifest.to_string_lossy().to_string();
    let brand_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "$brand",
            "source fixture contains Sass variable reference",
        )? + 1,
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
                        "name": "workspace",
                    },
                ],
            },
        }),
    );
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeConfiguration",
            "params": {
                "settings": {
                    "cssModuleExplainer": {
                        "resolution": {
                            "packageManifestPaths": [override_manifest_path],
                        },
                    },
                },
            },
        }),
    );
    assert!(
        state
            .snapshot()
            .resolution
            .package_manifest_paths
            .iter()
            .any(|path| path.ends_with("node_modules/@design/tokens/package.lsp.json"))
    );

    for (uri, text) in [
        (source_uri.as_str(), source_text),
        (override_style_uri.as_str(), "$brand: green;\n"),
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

    let definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": brand_position,
            },
        }),
    );
    assert_definition_response_single_target(&definition, override_style_uri.as_str());

    let _ = fs::remove_dir_all(root.as_path());
    Ok(())
}

fn raw_test_file_uri(path: &Path) -> String {
    format!("file://{}", path.to_string_lossy())
}

#[test]
fn codelens_keeps_references_when_workspace_owner_uri_encoding_differs() {
    let workspace_uri = "file:///workspace/(group-a)";
    let encoded_workspace_uri = "file:///workspace/%28group-a%29";
    let source_uri = "file:///workspace/%28group-a%29/src/App.tsx";
    let style_uri = "file:///workspace/(group-a)/src/App.module.scss";
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
                        "name": "group-a",
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
                    "text": "import bind from \"classnames/bind\";\nimport styles from \"./App.module.scss\";\nconst cx = bind.bind(styles);\nexport const view = <div className={cx(\"foo\")} />;",
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
                    "text": ".foo { color: red; }",
                },
            },
        }),
    );
    if let Some(document) = state.documents.get_mut(source_uri) {
        document.workspace_folder_uri = Some(encoded_workspace_uri.to_string());
    }

    let code_lens_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/codeLens",
            "params": {
                "textDocument": {
                    "uri": style_uri,
                },
            },
        }),
    );
    assert_eq!(
        code_lens_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/command/title")),
        Some(&json!("1 reference")),
    );
}

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
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer(
                "/result/0/edit/changes/file:~1~1~1workspace-a~1src~1App.module.scss/0/newText"
            )),
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
fn resolves_style_extract_code_actions_from_omena_query() {
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
                    "text": ".button { color: #ff0000; margin: 1rem; }",
                },
            },
        }),
    );

    let code_action_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "textDocument/codeAction",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "range": {
                    "start": {
                        "line": 0,
                        "character": 17,
                    },
                    "end": {
                        "line": 0,
                        "character": 24,
                    },
                },
                "context": {
                    "diagnostics": [],
                },
            },
        }),
    );

    assert_eq!(
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/title")),
        Some(&json!("Extract CSS custom property '--extracted-color'")),
    );
    assert_eq!(
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer("/result/1/title")),
        Some(&json!("Extract @value 'extractedColor'")),
    );
    assert_eq!(
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer(
                "/result/0/edit/changes/file:~1~1~1workspace-a~1src~1App.module.scss/0/newText"
            )),
        Some(&json!(":root {\n  --extracted-color: #ff0000;\n}\n\n")),
    );
    assert_eq!(
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer(
                "/result/0/edit/changes/file:~1~1~1workspace-a~1src~1App.module.scss/1/newText"
            )),
        Some(&json!("var(--extracted-color)")),
    );
    assert_eq!(
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/data/source")),
        Some(&json!("omenaQueryStyleExtractCodeActions")),
    );
}

#[test]
fn resolves_style_inline_code_actions_from_omena_query() {
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
                    "text": ".button {\n  composes: base;\n  color: red;\n}\n.base {\n  color: blue;\n  margin: 1rem;\n}",
                },
            },
        }),
    );

    let code_action_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "textDocument/codeAction",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "range": {
                    "start": {
                        "line": 1,
                        "character": 12,
                    },
                    "end": {
                        "line": 1,
                        "character": 16,
                    },
                },
                "context": {
                    "diagnostics": [],
                },
            },
        }),
    );

    assert_eq!(
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/title")),
        Some(&json!("Inline composed class 'base'")),
    );
    assert_eq!(
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/kind")),
        Some(&json!("refactor.inline")),
    );
    assert_eq!(
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer(
                "/result/0/edit/changes/file:~1~1~1workspace-a~1src~1App.module.scss/0/newText"
            )),
        Some(&json!("color: blue;\n  margin: 1rem;")),
    );
    assert_eq!(
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/data/source")),
        Some(&json!("omenaQueryStyleInlineCodeActions")),
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
fn tracks_workspace_folder_changes() {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-added-workspace-{}",
        std::process::id()
    ));
    let src_dir = workspace_root.join("src");
    let style_path = src_dir.join("Added.module.scss");
    let _ = std::fs::remove_dir_all(&workspace_root);
    let create_dir_result = std::fs::create_dir_all(&src_dir);
    assert!(
        create_dir_result.is_ok(),
        "create added-workspace fixture directory: {:?}",
        create_dir_result.err(),
    );
    let write_result = std::fs::write(&style_path, ".added { color: red; }");
    assert!(
        write_result.is_ok(),
        "write added-workspace style fixture: {:?}",
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
                        "uri": "file:///workspace-a",
                        "name": "workspace-a",
                    },
                ],
            },
        }),
    );
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeWorkspaceFolders",
            "params": {
                "event": {
                    "removed": [
                        {
                            "uri": "file:///workspace-a",
                            "name": "workspace-a",
                        },
                    ],
                    "added": [
                        {
                            "uri": workspace_uri,
                            "name": "workspace-b",
                        },
                    ],
                },
            },
        }),
    );

    assert_eq!(state.workspace_folder_count(), 1);
    assert!(state.workspace_folder("file:///workspace-a").is_none());
    assert!(state.workspace_folder(workspace_uri.as_str()).is_some());
    let indexed = state
        .document(style_uri.as_str())
        .and_then(|document| document.style_summary.as_ref());
    assert_eq!(
        indexed.map(|summary| summary.selector_names.clone()),
        Some(vec!["added".to_string()]),
    );
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeWorkspaceFolders",
            "params": {
                "event": {
                    "removed": [
                        {
                            "uri": workspace_uri,
                            "name": "workspace-b",
                        },
                    ],
                    "added": [],
                },
            },
        }),
    );
    assert!(state.workspace_folder(workspace_uri.as_str()).is_none());
    assert!(state.document(style_uri.as_str()).is_none());
    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn retargets_indexed_style_documents_after_nested_workspace_removal() {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-nested-workspace-retarget-{}",
        std::process::id()
    ));
    let nested_root = workspace_root.join("packages").join("app");
    let src_dir = nested_root.join("src");
    let style_path = src_dir.join("Nested.module.scss");
    let _ = std::fs::remove_dir_all(&workspace_root);
    let create_dir_result = std::fs::create_dir_all(&src_dir);
    assert!(
        create_dir_result.is_ok(),
        "create nested-workspace fixture directory: {:?}",
        create_dir_result.err(),
    );
    let write_result = std::fs::write(&style_path, ".nested { color: red; }");
    assert!(
        write_result.is_ok(),
        "write nested-workspace style fixture: {:?}",
        write_result.err(),
    );

    let workspace_uri = format!("file://{}", workspace_root.display());
    let nested_workspace_uri = format!("file://{}", nested_root.display());
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
                        "name": "workspace",
                    },
                    {
                        "uri": nested_workspace_uri,
                        "name": "app",
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

    let indexed = state.document(style_uri.as_str());
    assert_eq!(
        indexed.and_then(|document| document.workspace_folder_uri.as_deref()),
        Some(nested_workspace_uri.as_str()),
    );
    assert_eq!(
        indexed
            .and_then(|document| document.style_summary.as_ref())
            .map(|summary| summary.selector_names.clone()),
        Some(vec!["nested".to_string()]),
    );

    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeWorkspaceFolders",
            "params": {
                "event": {
                    "removed": [
                        {
                            "uri": nested_workspace_uri,
                            "name": "app",
                        },
                    ],
                    "added": [],
                },
            },
        }),
    );

    let retargeted = state.document(style_uri.as_str());
    assert_eq!(
        retargeted.and_then(|document| document.workspace_folder_uri.as_deref()),
        Some(workspace_uri.as_str()),
    );
    assert_eq!(
        retargeted
            .and_then(|document| document.style_summary.as_ref())
            .map(|summary| summary.selector_names.clone()),
        Some(vec!["nested".to_string()]),
    );

    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeWorkspaceFolders",
            "params": {
                "event": {
                    "removed": [
                        {
                            "uri": workspace_uri,
                            "name": "workspace",
                        },
                    ],
                    "added": [],
                },
            },
        }),
    );
    assert!(state.document(style_uri.as_str()).is_none());
    let _ = std::fs::remove_dir_all(&workspace_root);
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

#[test]
fn assigns_document_workspace_folder_by_longest_uri_prefix() {
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
                        "uri": "file:///workspace",
                        "name": "workspace",
                    },
                    {
                        "uri": "file:///workspace/packages/app",
                        "name": "app",
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
                    "uri": "file:///workspace/packages/app/src/App.tsx",
                    "languageId": "typescriptreact",
                    "version": 1,
                    "text": "export const App = () => null;",
                },
            },
        }),
    );

    assert_eq!(
        state
            .document("file:///workspace/packages/app/src/App.tsx")
            .and_then(|document| document.workspace_folder_uri.as_deref()),
        Some("file:///workspace/packages/app"),
    );

    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeWorkspaceFolders",
            "params": {
                "event": {
                    "removed": [
                        {
                            "uri": "file:///workspace/packages/app",
                            "name": "app",
                        },
                    ],
                    "added": [],
                },
            },
        }),
    );

    assert_eq!(
        state
            .document("file:///workspace/packages/app/src/App.tsx")
            .and_then(|document| document.workspace_folder_uri.as_deref()),
        Some("file:///workspace"),
    );
}
