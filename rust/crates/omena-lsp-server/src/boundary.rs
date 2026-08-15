use crate::diagnostics_scheduler::{
    RustDiagnosticsSchedulerBoundaryV0, rust_diagnostics_scheduler_contract,
};
use crate::disk_cache::{DiskDiagnosticsCacheBoundaryV0, disk_diagnostics_cache_contract};
use crate::query_reuse::{RustQueryReuseBoundaryV0, rust_query_reuse_contract};
use crate::workspace_runtime_registry::{
    WorkspaceRuntimeRegistryBoundaryV0, workspace_runtime_registry_contract,
};
use crate::{
    CANCEL_REQUEST_METHOD, CASCADE_AT_POSITION_REQUEST, CLEAR_CACHES_REQUEST,
    EXPLAIN_HOVER_TRACE_REQUEST, EXPLAIN_REQUEST, NODE_TEXT_DOCUMENT_SYNC_KIND,
    STYLE_CONTEXT_INDEX_REQUEST,
};
use omena_tsgo_client::{OmenaTsgoClientBoundarySummaryV0, summarize_omena_tsgo_client_boundary};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum CacheStorageRungV0 {
    InitializationOptions,
    Environment,
    Platform,
    Workspace,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum CacheWriteSurfaceKindV0 {
    LspWorkspaceCache,
    BridgeExternalSifCache,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheWriteSurfaceV0 {
    pub root_kind: CacheWriteSurfaceKindV0,
    pub resolved_rung: CacheStorageRungV0,
    pub root_shape: &'static str,
    pub cache_directories: Vec<&'static str>,
}

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
    /// Every owned cache root the LSP process may write, including writes
    /// performed by the bridge below the query layer. The typed rung keeps
    /// editor, environment, platform, workspace, and disabled resolution
    /// distinguishable without weakening the `neverFetch` network invariant.
    pub disk_write_surfaces: Vec<CacheWriteSurfaceV0>,
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
            "dispatchedRequestCancellationAtCompletionBoundary",
            "noMidComputationCancellationClaim",
            "workerQueriesUseSnapshotReadView",
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
    let lsp_rung = current_cache_storage_rung();
    let bridge_rung = current_bridge_cache_storage_rung();
    LspTrustBoundaryV0 {
        product: "omena-lsp-server.trust-boundary",
        network_access: "neverFetch",
        verification_owner: "omena-cli.lock-provenance",
        request_path_policy: vec![
            "analysisTimeUsesLocalWorkspaceOnly",
            "lockAndSifEvidenceReadFromDisk",
            "attestationVerificationOwnedByCli",
            // Workspace locks are digest-checked hints only. They cannot
            // suppress the locally regenerated bridge plane or establish
            // editor-visible Sass semantics by themselves.
            "workspaceLockSifHintsRequireDigestAndLocalBridgeRegeneration",
            // The CLI records immutable canonical-url + SIF-hash verdicts and
            // content-addressed bundles. Bridge verifies them offline; LSP
            // never treats a lock as trust authority or uses the network.
            "recordedShardVerdictsVerifiedOfflineWithoutNetworkAuthority",
            "noRegistryFetchOnLspRequestPath",
            "noTransparencyLogLookupOnLspRequestPath",
            // Cache roots may be editor- or platform-owned after resolution;
            // the durable invariant is containment by this declared set, not
            // physical placement below the opened repository.
            "cacheWritesConfinedToDeclaredOwnedRootsNeverNetwork",
        ],
        forbidden_runtime_capabilities: vec![
            "registryHttpClient",
            "sigstoreBundleVerifier",
            "transparencyLogClient",
            "socketNetworkIo",
        ],
        disk_write_surfaces: declared_cache_write_surfaces_for_rungs(lsp_rung, bridge_rung),
    }
}

pub(crate) fn current_cache_storage_rung() -> CacheStorageRungV0 {
    let config = crate::cache_root::LspCacheStorageConfigV0::standalone(None);
    crate::cache_root::process_cache_roots(
        &config,
        "<workspaceIdentity>",
        std::path::Path::new("<workspaceFolder>"),
    )
    .source
    .into()
}

fn current_bridge_cache_storage_rung() -> CacheStorageRungV0 {
    let has_environment_override = [
        crate::cache_root::OMENA_CACHE_DIR_ENV,
        crate::cache_root::OMENA_GLOBAL_CACHE_DIR_ENV,
    ]
    .into_iter()
    .any(|name| std::env::var_os(name).is_some_and(|value| !value.is_empty()));
    if has_environment_override {
        CacheStorageRungV0::Environment
    } else {
        CacheStorageRungV0::Platform
    }
}

pub(crate) fn declared_cache_write_surfaces_for_rungs(
    lsp_rung: CacheStorageRungV0,
    bridge_rung: CacheStorageRungV0,
) -> Vec<CacheWriteSurfaceV0> {
    let mut surfaces = Vec::new();
    if let Some(root_shape) = cache_root_shape(CacheWriteSurfaceKindV0::LspWorkspaceCache, lsp_rung)
    {
        surfaces.push(CacheWriteSurfaceV0 {
            root_kind: CacheWriteSurfaceKindV0::LspWorkspaceCache,
            resolved_rung: lsp_rung,
            root_shape,
            cache_directories: vec![
                "diagnostics-cache-v1",
                "source-document-index-v1",
                "source-type-fact-cache-v1",
                "workspace-occurrence-shards-v2",
            ],
        });
    }
    if let Some(root_shape) =
        cache_root_shape(CacheWriteSurfaceKindV0::BridgeExternalSifCache, bridge_rung)
    {
        surfaces.push(CacheWriteSurfaceV0 {
            root_kind: CacheWriteSurfaceKindV0::BridgeExternalSifCache,
            resolved_rung: bridge_rung,
            root_shape,
            cache_directories: vec!["external-sif-v0"],
        });
    }
    surfaces
}

fn cache_root_shape(
    kind: CacheWriteSurfaceKindV0,
    rung: CacheStorageRungV0,
) -> Option<&'static str> {
    match (kind, rung) {
        (CacheWriteSurfaceKindV0::LspWorkspaceCache, CacheStorageRungV0::InitializationOptions) => {
            Some("<initializationWorkspaceStorage>/omena/workspaces/<workspaceIdentityHash>/**")
        }
        (CacheWriteSurfaceKindV0::LspWorkspaceCache, CacheStorageRungV0::Environment) => {
            Some("<environmentCacheDir>/omena/workspaces/<workspaceIdentityHash>/**")
        }
        (CacheWriteSurfaceKindV0::LspWorkspaceCache, CacheStorageRungV0::Platform) => {
            Some("<platformCacheHome>/omena/workspaces/<workspaceIdentityHash>/**")
        }
        (CacheWriteSurfaceKindV0::LspWorkspaceCache, CacheStorageRungV0::Workspace) => {
            Some("<workspaceFolder>/.cache/omena/**")
        }
        (
            CacheWriteSurfaceKindV0::BridgeExternalSifCache,
            CacheStorageRungV0::InitializationOptions,
        ) => {
            Some("<initializationGlobalStorage>/omena/workspaces/<bridgeWorkspaceIdentityHash>/**")
        }
        (CacheWriteSurfaceKindV0::BridgeExternalSifCache, CacheStorageRungV0::Environment) => {
            Some("<environmentCacheDir>/omena/workspaces/<bridgeWorkspaceIdentityHash>/**")
        }
        (CacheWriteSurfaceKindV0::BridgeExternalSifCache, CacheStorageRungV0::Platform) => {
            Some("<platformCacheHome>/omena/workspaces/<bridgeWorkspaceIdentityHash>/**")
        }
        (CacheWriteSurfaceKindV0::BridgeExternalSifCache, CacheStorageRungV0::Workspace) => {
            Some("<bridgeWorkspaceRoot>/.cache/omena/**")
        }
        (_, CacheStorageRungV0::Disabled) => None,
    }
}

pub(crate) fn declared_disk_diagnostics_storage_locations() -> Vec<CacheWriteSurfaceV0> {
    declared_cache_write_surfaces_for_rungs(
        current_cache_storage_rung(),
        CacheStorageRungV0::Disabled,
    )
    .into_iter()
    .filter(|surface| surface.root_kind == CacheWriteSurfaceKindV0::LspWorkspaceCache)
    .map(|mut surface| {
        surface.cache_directories = vec!["diagnostics-cache-v1"];
        surface
    })
    .collect()
}

impl From<crate::cache_root::CacheRootSourceV0> for CacheStorageRungV0 {
    fn from(source: crate::cache_root::CacheRootSourceV0) -> Self {
        match source {
            crate::cache_root::CacheRootSourceV0::InitializationOptions => {
                Self::InitializationOptions
            }
            crate::cache_root::CacheRootSourceV0::Environment => Self::Environment,
            crate::cache_root::CacheRootSourceV0::Platform => Self::Platform,
            crate::cache_root::CacheRootSourceV0::Workspace => Self::Workspace,
            crate::cache_root::CacheRootSourceV0::Disabled => Self::Disabled,
        }
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
        split_repository: env!("CARGO_PKG_REPOSITORY"),
        cargo_install_command: concat!(
            "cargo install omena-lsp-server --version ",
            env!("CARGO_PKG_VERSION")
        ),
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
            "prepareEditorStorageRoots",
            "passStorageInitializationOptions",
            "declareStaticDocumentSelector",
            "startLanguageClient",
            "registerStaticFileWatchers",
            "requestServerOwnedCacheClear",
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
            "resolveAndClearDeclaredOwnedCachePaths",
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
        runtime_handler(CLEAR_CACHES_REQUEST),
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

#[cfg(test)]
mod cache_storage_boundary_tests {
    use super::*;

    #[test]
    fn declared_cache_surface_table_is_typed_and_exhaustive_by_rung() {
        let rows = [
            (
                CacheStorageRungV0::InitializationOptions,
                "<initializationWorkspaceStorage>/omena/workspaces/<workspaceIdentityHash>/**",
                "<initializationGlobalStorage>/omena/workspaces/<bridgeWorkspaceIdentityHash>/**",
            ),
            (
                CacheStorageRungV0::Environment,
                "<environmentCacheDir>/omena/workspaces/<workspaceIdentityHash>/**",
                "<environmentCacheDir>/omena/workspaces/<bridgeWorkspaceIdentityHash>/**",
            ),
            (
                CacheStorageRungV0::Platform,
                "<platformCacheHome>/omena/workspaces/<workspaceIdentityHash>/**",
                "<platformCacheHome>/omena/workspaces/<bridgeWorkspaceIdentityHash>/**",
            ),
            (
                CacheStorageRungV0::Workspace,
                "<workspaceFolder>/.cache/omena/**",
                "<bridgeWorkspaceRoot>/.cache/omena/**",
            ),
        ];

        for (rung, expected_lsp_root, expected_bridge_root) in rows {
            let surfaces = declared_cache_write_surfaces_for_rungs(rung, rung);
            assert_eq!(surfaces.len(), 2, "rung={rung:?}");
            assert_eq!(
                surfaces[0],
                CacheWriteSurfaceV0 {
                    root_kind: CacheWriteSurfaceKindV0::LspWorkspaceCache,
                    resolved_rung: rung,
                    root_shape: expected_lsp_root,
                    cache_directories: vec![
                        "diagnostics-cache-v1",
                        "source-document-index-v1",
                        "source-type-fact-cache-v1",
                        "workspace-occurrence-shards-v2",
                    ],
                }
            );
            assert_eq!(
                surfaces[1],
                CacheWriteSurfaceV0 {
                    root_kind: CacheWriteSurfaceKindV0::BridgeExternalSifCache,
                    resolved_rung: rung,
                    root_shape: expected_bridge_root,
                    cache_directories: vec!["external-sif-v0"],
                }
            );
        }

        assert!(
            declared_cache_write_surfaces_for_rungs(
                CacheStorageRungV0::Disabled,
                CacheStorageRungV0::Disabled,
            )
            .is_empty(),
            "disabled resolution must declare no writable cache surface"
        );

        let split = declared_cache_write_surfaces_for_rungs(
            CacheStorageRungV0::Platform,
            CacheStorageRungV0::Workspace,
        );
        assert_eq!(split[0].resolved_rung, CacheStorageRungV0::Platform);
        assert_eq!(split[1].resolved_rung, CacheStorageRungV0::Workspace);
    }
}
