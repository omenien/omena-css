use crate::diagnostics_scheduler::{
    RustDiagnosticsSchedulerBoundaryV0, rust_diagnostics_scheduler_contract,
};
use crate::disk_cache::{DiskDiagnosticsCacheBoundaryV0, disk_diagnostics_cache_contract};
use crate::query_reuse::{RustQueryReuseBoundaryV0, rust_query_reuse_contract};
use crate::workspace_runtime_registry::{
    WorkspaceRuntimeRegistryBoundaryV0, workspace_runtime_registry_contract,
};
use crate::{
    CANCEL_REQUEST_METHOD, CASCADE_AT_POSITION_REQUEST, EXPLAIN_HOVER_TRACE_REQUEST,
    EXPLAIN_REQUEST, NODE_TEXT_DOCUMENT_SYNC_KIND, STYLE_CONTEXT_INDEX_REQUEST,
};
use omena_tsgo_client::{OmenaTsgoClientBoundarySummaryV0, summarize_omena_tsgo_client_boundary};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaLspServerBoundarySummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub server_name: &'static str,
    pub migration_status: &'static str,
    pub transport_contract: &'static str,
    pub trust_boundary: LspTrustBoundaryV0,
    pub capabilities: OmenaLspServerCapabilitiesV0,
    pub handler_surfaces: Vec<LspHandlerSurfaceV0>,
    pub migration_phases: Vec<LspMigrationPhaseV0>,
    pub blocking_work_policy: Vec<&'static str>,
    pub tsgo_client_boundary: OmenaTsgoClientBoundarySummaryV0,
    pub source_provider_adapter: SourceProviderDirectRustAdapterV0,
    pub workspace_runtime_registry: WorkspaceRuntimeRegistryBoundaryV0,
    pub diagnostics_scheduler: RustDiagnosticsSchedulerBoundaryV0,
    pub query_reuse: RustQueryReuseBoundaryV0,
    pub disk_diagnostics_cache: DiskDiagnosticsCacheBoundaryV0,
    pub thin_client_endpoint: ThinClientEndpointV0,
    pub multi_editor_distribution: MultiEditorDistributionV0,
    pub node_parity_contracts: Vec<&'static str>,
    pub next_decoupling_targets: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspTrustBoundaryV0 {
    pub product: &'static str,
    pub network_access: &'static str,
    pub verification_owner: &'static str,
    pub request_path_policy: Vec<&'static str>,
    pub forbidden_runtime_capabilities: Vec<&'static str>,
    /// Declared local-disk write surfaces. The LSP historically wrote NO
    /// files; the disk diagnostics cache (RFC 0009 Pillar C, rfcs#66) is the
    /// first declared write surface. Local workspace disk only — this does
    /// not weaken the `neverFetch` network invariant.
    pub disk_write_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaLspServerCapabilitiesV0 {
    pub text_document_sync: u8,
    pub definition_provider: bool,
    pub hover_provider: bool,
    pub color_provider: bool,
    pub completion_provider: CompletionProviderCapabilityV0,
    pub code_action_provider: CodeActionProviderCapabilityV0,
    pub references_provider: bool,
    pub code_lens_provider: ResolveProviderCapabilityV0,
    pub document_link_provider: ResolveProviderCapabilityV0,
    pub workspace_symbol_provider: bool,
    pub rename_provider: RenameProviderCapabilityV0,
    pub workspace: WorkspaceCapabilityV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionProviderCapabilityV0 {
    pub trigger_characters: Vec<&'static str>,
    pub resolve_provider: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeActionProviderCapabilityV0 {
    pub code_action_kinds: Vec<&'static str>,
    pub resolve_provider: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveProviderCapabilityV0 {
    pub resolve_provider: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameProviderCapabilityV0 {
    pub prepare_provider: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceCapabilityV0 {
    pub workspace_folders: WorkspaceFoldersCapabilityV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFoldersCapabilityV0 {
    pub supported: bool,
    pub change_notifications: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspHandlerSurfaceV0 {
    pub method: &'static str,
    pub node_owner: &'static str,
    pub rust_owner_target: &'static str,
    pub migration_state: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspMigrationPhaseV0 {
    pub phase: &'static str,
    pub goal: &'static str,
    pub exit_gate: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThinClientEndpointV0 {
    pub product: &'static str,
    pub endpoint_name: &'static str,
    pub transport_contract: &'static str,
    pub command_owner: &'static str,
    pub standalone_package: &'static str,
    pub split_repository: &'static str,
    pub cargo_install_command: &'static str,
    pub node_fallback_allowed: bool,
    pub file_watcher_globs: Vec<&'static str>,
    pub host_responsibilities: Vec<&'static str>,
    pub rust_responsibilities: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiEditorDistributionV0 {
    pub product: &'static str,
    pub owner: &'static str,
    pub distribution_model: &'static str,
    pub supported_editors: Vec<&'static str>,
    pub install_surfaces: Vec<&'static str>,
    pub documentation: Vec<&'static str>,
    pub endpoint_policy: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceProviderDirectRustAdapterV0 {
    pub product: &'static str,
    pub candidate_owner: &'static str,
    pub style_definition_owner: &'static str,
    pub type_fact_owner: &'static str,
    pub request_path_policy: Vec<&'static str>,
    pub provider_surfaces: Vec<&'static str>,
}

pub fn summarize_omena_lsp_server_boundary() -> OmenaLspServerBoundarySummaryV0 {
    OmenaLspServerBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-lsp-server.boundary",
        server_name: "omena-css",
        migration_status: "rustStable",
        transport_contract: "LSP stdio or IPC JSON-RPC",
        trust_boundary: lsp_trust_boundary_contract(),
        capabilities: current_node_lsp_capability_contract(),
        handler_surfaces: lsp_handler_surfaces(),
        migration_phases: lsp_migration_phases(),
        blocking_work_policy: vec![
            "noFullWorkspaceProgramOnRequestPath",
            "queuedRequestCancellationBeforeProviderWork",
            "tsgoProviderCancellationTokenBoundary",
            "backgroundIndexAndTypeFactWarmup",
            "staleOrUnresolvableFastReturn",
        ],
        tsgo_client_boundary: summarize_omena_tsgo_client_boundary(),
        source_provider_adapter: source_provider_direct_rust_adapter_contract(),
        workspace_runtime_registry: workspace_runtime_registry_contract(),
        diagnostics_scheduler: rust_diagnostics_scheduler_contract(),
        query_reuse: rust_query_reuse_contract(),
        disk_diagnostics_cache: disk_diagnostics_cache_contract(),
        thin_client_endpoint: thin_client_endpoint_contract(),
        multi_editor_distribution: multi_editor_distribution_contract(),
        node_parity_contracts: vec![
            "initializeCapabilities",
            "textDocumentSync",
            "workspaceFolders",
            "dynamicFileWatchers",
            "diagnosticsPush",
            "codeLensRefresh",
        ],
        next_decoupling_targets: vec![],
    }
}

pub fn lsp_trust_boundary_contract() -> LspTrustBoundaryV0 {
    LspTrustBoundaryV0 {
        product: "omena-lsp-server.trust-boundary",
        network_access: "neverFetch",
        verification_owner: "omena-cli.lock-provenance",
        request_path_policy: vec![
            "analysisTimeUsesLocalWorkspaceOnly",
            "lockAndSifEvidenceReadFromDisk",
            "attestationVerificationOwnedByCli",
            "noRegistryFetchOnLspRequestPath",
            "noTransparencyLogLookupOnLspRequestPath",
            "diskDiagnosticsCacheLocalWorkspaceWritesOnly",
        ],
        forbidden_runtime_capabilities: vec![
            "registryHttpClient",
            "sigstoreBundleVerifier",
            "transparencyLogClient",
            "socketNetworkIo",
        ],
        disk_write_surfaces: vec!["<workspaceFolder>/.cache/omena/**"],
    }
}

pub fn source_provider_direct_rust_adapter_contract() -> SourceProviderDirectRustAdapterV0 {
    SourceProviderDirectRustAdapterV0 {
        product: "omena-lsp-server.source-provider-direct-rust-adapter",
        candidate_owner: "omena-query/sourceSyntaxIndex",
        style_definition_owner: "omena-query/styleHoverCandidates",
        type_fact_owner: "omena-tsgo-client",
        request_path_policy: vec![
            "noNodeWorkspaceTypeResolverOnSourceProviderPath",
            "buildQuerySourceSyntaxIndexOnDocumentChange",
            "dedupeTargetAwareSourceCandidates",
            "consumeQueryStyleHoverCandidates",
            "consumeQuerySassModuleSources",
            "consumeConfiguredPackageManifestPaths",
            "consumeTsgoTypeFactsForTypedCxProjection",
            "consumeSassPartialEvaluatorGeneratedSelectors",
            "useOpenedDocumentIndexesBeforeWorkspaceFallback",
            "unresolvedCandidatesRemainFastDiagnostics",
        ],
        provider_surfaces: vec![
            "textDocument/hover",
            "textDocument/definition",
            "textDocument/references",
            "textDocument/completion",
            "textDocument/publishDiagnostics",
            CASCADE_AT_POSITION_REQUEST,
            STYLE_CONTEXT_INDEX_REQUEST,
            EXPLAIN_HOVER_TRACE_REQUEST,
            EXPLAIN_REQUEST,
        ],
    }
}

pub fn thin_client_endpoint_contract() -> ThinClientEndpointV0 {
    ThinClientEndpointV0 {
        product: "omena-lsp-server.thin-client-endpoint",
        endpoint_name: "omena-css.thin-client-runtime-endpoint",
        transport_contract: "LSP stdio JSON-RPC",
        command_owner: "dist/bin/<platform>-<arch>/omena-lsp-server",
        standalone_package: "omena-lsp-server",
        split_repository: "https://github.com/omenien/omena-lsp-server",
        cargo_install_command: "cargo install omena-lsp-server --version 0.1.5",
        node_fallback_allowed: false,
        file_watcher_globs: vec![
            "**/*.module.{scss,css,less}",
            "**/*.{ts,tsx,js,jsx,mts,cts,mjs,cjs,d.ts,vue,html,svelte,astro,md,mdx,liquid,twig,njk,nunjucks,hbs,handlebars,erb,ejs,html.eex,heex}",
            "**/tsconfig*.json",
            "**/jsconfig*.json",
            "**/package.json",
            "**/vite.config.{ts,mts,cts,js,mjs,cjs}",
            "**/webpack.config.{ts,mts,cts,js,mjs,cjs}",
        ],
        host_responsibilities: vec![
            "resolvePackagedRustBinary",
            "resolveStandaloneRustCommand",
            "buildThinClientServerOptions",
            "declareStaticDocumentSelector",
            "startLanguageClient",
            "registerStaticFileWatchers",
            "translateShowReferencesArguments",
            "renderHoverTracePanel",
            "surfaceStartupErrors",
        ],
        rust_responsibilities: vec![
            "ownLspLifecycle",
            "ownWorkspaceState",
            "ownDiagnosticsScheduling",
            "ownProviderExecution",
            "ownTsgoClientLifecycle",
        ],
    }
}

pub fn multi_editor_distribution_contract() -> MultiEditorDistributionV0 {
    MultiEditorDistributionV0 {
        product: "omena-lsp-server.multi-editor-distribution",
        owner: "omena-lsp-server/distribution",
        distribution_model: "standaloneRustLspServerWithThinEditorHosts",
        supported_editors: vec!["vscode", "neovim", "zed"],
        install_surfaces: vec![
            "vsixBundledDistBinary",
            "cargoInstallOmenaLspServer",
            "repoLocalDistBin",
        ],
        documentation: vec![
            "client/src/extension.ts",
            "docs/clients/neovim.md",
            "docs/clients/zed.md",
        ],
        endpoint_policy: vec![
            "standaloneRustServerIsPrimaryMultiEditorEndpoint",
            "nodeLspServerIsNotPrimaryEndpoint",
            "editorClientsDoNotImplementProviderSemantics",
            "editorsMayRunBesideNativeTypeScriptServer",
        ],
    }
}

pub fn current_node_lsp_capability_contract() -> OmenaLspServerCapabilitiesV0 {
    OmenaLspServerCapabilitiesV0 {
        text_document_sync: NODE_TEXT_DOCUMENT_SYNC_KIND,
        definition_provider: true,
        hover_provider: true,
        color_provider: true,
        completion_provider: CompletionProviderCapabilityV0 {
            trigger_characters: vec!["'", "\"", "`", ",", ".", "$", "@", "-"],
            resolve_provider: false,
        },
        code_action_provider: CodeActionProviderCapabilityV0 {
            code_action_kinds: vec!["quickfix", "refactor.extract", "refactor.inline"],
            resolve_provider: false,
        },
        references_provider: true,
        code_lens_provider: ResolveProviderCapabilityV0 {
            resolve_provider: false,
        },
        document_link_provider: ResolveProviderCapabilityV0 {
            resolve_provider: false,
        },
        workspace_symbol_provider: true,
        rename_provider: RenameProviderCapabilityV0 {
            prepare_provider: true,
        },
        workspace: WorkspaceCapabilityV0 {
            workspace_folders: WorkspaceFoldersCapabilityV0 {
                supported: true,
                change_notifications: true,
            },
        },
    }
}

pub fn lsp_handler_surfaces() -> Vec<LspHandlerSurfaceV0> {
    vec![
        style_provider_handler("textDocument/definition"),
        style_provider_handler("textDocument/hover"),
        style_provider_handler("textDocument/completion"),
        style_provider_handler("textDocument/codeAction"),
        style_provider_handler("textDocument/references"),
        style_provider_handler("textDocument/codeLens"),
        style_provider_handler("textDocument/documentColor"),
        style_provider_handler("textDocument/colorPresentation"),
        style_provider_handler("textDocument/documentLink"),
        style_provider_handler("workspace/symbol"),
        style_provider_handler("textDocument/prepareRename"),
        style_provider_handler("textDocument/rename"),
        runtime_handler("initialized"),
        runtime_handler("textDocument/didOpen"),
        runtime_handler("textDocument/didChange"),
        runtime_handler("textDocument/didClose"),
        runtime_handler("workspace/didChangeWatchedFiles"),
        runtime_handler("workspace/didChangeConfiguration"),
        runtime_handler("workspace/didChangeWorkspaceFolders"),
        diagnostics_handler("textDocument/publishDiagnostics"),
        query_inspection_handler(CASCADE_AT_POSITION_REQUEST),
        query_inspection_handler(STYLE_CONTEXT_INDEX_REQUEST),
        query_inspection_handler(EXPLAIN_HOVER_TRACE_REQUEST),
        query_inspection_handler(EXPLAIN_REQUEST),
        runtime_handler(CANCEL_REQUEST_METHOD),
    ]
}

fn style_provider_handler(method: &'static str) -> LspHandlerSurfaceV0 {
    LspHandlerSurfaceV0 {
        method,
        node_owner: "server/lsp-server/src/providers",
        rust_owner_target: "omena-lsp-server/providers/style-source",
        migration_state: "providerParity",
    }
}

fn runtime_handler(method: &'static str) -> LspHandlerSurfaceV0 {
    LspHandlerSurfaceV0 {
        method,
        node_owner: "server/lsp-server/src/handler-registration.ts",
        rust_owner_target: "omena-lsp-server/runtime",
        migration_state: "implemented",
    }
}

fn diagnostics_handler(method: &'static str) -> LspHandlerSurfaceV0 {
    LspHandlerSurfaceV0 {
        method,
        node_owner: "server/lsp-server/src/diagnostics-scheduler.ts",
        rust_owner_target: "omena-lsp-server/diagnostics",
        migration_state: "implemented",
    }
}

fn query_inspection_handler(method: &'static str) -> LspHandlerSurfaceV0 {
    LspHandlerSurfaceV0 {
        method,
        node_owner: "server/lsp-server/src/query-inspection",
        rust_owner_target: "omena-lsp-server/query-inspection",
        migration_state: "implemented",
    }
}

pub fn lsp_migration_phases() -> Vec<LspMigrationPhaseV0> {
    vec![
        LspMigrationPhaseV0 {
            phase: "phase-0-boundary",
            goal: "declare Rust LSP capability and handler parity with the Node server",
            exit_gate: "rust/omena-lsp-server/boundary",
        },
        LspMigrationPhaseV0 {
            phase: "phase-1-shell",
            goal: "own initialize, shutdown, text sync, workspace folders, and watcher state in Rust",
            exit_gate: "rust/omena-lsp-server/runtime-loop",
        },
        LspMigrationPhaseV0 {
            phase: "phase-2-style-providers",
            goal: "serve style-side hover, definition, references, diagnostics, and code lens from Rust",
            exit_gate: "rust/omena-lsp-server/provider-parity",
        },
        LspMigrationPhaseV0 {
            phase: "phase-3-source-providers",
            goal: "replace Node WorkspaceTypeResolver hot path with a long-lived tsgo client and Rust query runtime",
            exit_gate: "rust/omena-tsgo-client/boundary",
        },
        LspMigrationPhaseV0 {
            phase: "phase-4-thin-client",
            goal: "shrink the VS Code extension to UI commands and Rust LSP process orchestration",
            exit_gate: "rust/omena-lsp-server/thin-client-boundary",
        },
    ]
}
