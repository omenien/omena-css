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
            .contains(&"dispatchedRequestCancellationAtCompletionBoundary")
    );
    assert!(
        summary
            .blocking_work_policy
            .contains(&"noMidComputationCancellationClaim")
    );
    assert!(
        summary
            .blocking_work_policy
            .contains(&"workerQueriesUseSnapshotReadView")
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
