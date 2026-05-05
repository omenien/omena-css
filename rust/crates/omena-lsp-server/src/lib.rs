mod diagnostics_scheduler;
mod query_reuse;
mod workspace_runtime_registry;

use diagnostics_scheduler::{
    RustDiagnosticsSchedulerBoundaryV0, diagnostics_schedule_event, run_diagnostics_schedule,
    rust_diagnostics_scheduler_contract,
};
use engine_style_parser::{
    AtRuleKind, ParserByteSpanV0, ParserPositionV0, ParserRangeV0, StyleLanguage,
    SyntaxNodePayload, parse_style_module, summarize_css_modules_intermediate,
};
use omena_bridge::{
    SourceImportedStyleBindingV0 as ImportedStyleBinding,
    SourceSelectorReferenceFactV0 as SourceSelectorReferenceFact,
    SourceSelectorReferenceMatchKindV0 as SourceSelectorReferenceMatchKind,
    SourceSyntaxIndexV0 as SourceSyntaxIndex, SourceTypeFactTargetV0 as SourceTypeFactTarget,
    canonicalize_source_selector_references, summarize_omena_bridge_source_syntax_index,
};
use omena_incremental::IncrementalCancellationRegistryV0;
use omena_tsgo_client::{
    OmenaTsgoClientBoundarySummaryV0, TsgoJsonRpcTypeFactProviderV0, TsgoResolvedTypeV0,
    TsgoTypeFactRequestV0, TsgoTypeFactResultEntryV0, TsgoTypeFactTargetV0,
    TsgoWorkspaceProcessPoolV0, build_tsgo_process_command, summarize_omena_tsgo_client_boundary,
};
use query_reuse::{
    RustQueryReuseBoundaryV0, refresh_document_reusable_indexes, rust_query_reuse_contract,
};
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use workspace_runtime_registry::{
    WorkspaceRuntimeRegistry, WorkspaceRuntimeRegistryBoundaryV0,
    workspace_runtime_registry_contract,
};

pub const NODE_TEXT_DOCUMENT_SYNC_KIND: u8 = 2;
pub const DEBUG_STATE_REQUEST: &str = "cssModuleExplainer/rustLspState";
pub const RUNTIME_LOOP_PROBE_REQUEST: &str = "cssModuleExplainer/runtimeLoopProbe";
pub const STYLE_HOVER_CANDIDATES_REQUEST: &str = "cssModuleExplainer/rustStyleHoverCandidates";
pub const STYLE_DIAGNOSTICS_REQUEST: &str = "cssModuleExplainer/rustStyleDiagnostics";
pub const SOURCE_DIAGNOSTICS_REQUEST: &str = "cssModuleExplainer/rustSourceDiagnostics";
const CANCEL_REQUEST_METHOD: &str = "$/cancelRequest";
const REQUEST_CANCELLED_ERROR_CODE: i32 = -32800;
const WORKSPACE_STYLE_INDEX_LIMIT: usize = 512;
const WORKSPACE_STYLE_INDEX_DIR_LIMIT: usize = 2048;
const WORKSPACE_STYLE_INDEX_TIME_BUDGET_MS: u128 = 50;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaLspServerBoundarySummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub server_name: &'static str,
    pub migration_status: &'static str,
    pub transport_contract: &'static str,
    pub capabilities: OmenaLspServerCapabilitiesV0,
    pub handler_surfaces: Vec<LspHandlerSurfaceV0>,
    pub migration_phases: Vec<LspMigrationPhaseV0>,
    pub blocking_work_policy: Vec<&'static str>,
    pub tsgo_client_boundary: OmenaTsgoClientBoundarySummaryV0,
    pub source_provider_adapter: SourceProviderDirectRustAdapterV0,
    pub workspace_runtime_registry: WorkspaceRuntimeRegistryBoundaryV0,
    pub diagnostics_scheduler: RustDiagnosticsSchedulerBoundaryV0,
    pub query_reuse: RustQueryReuseBoundaryV0,
    pub thin_client_endpoint: ThinClientEndpointV0,
    pub multi_editor_distribution: MultiEditorDistributionV0,
    pub node_parity_contracts: Vec<&'static str>,
    pub next_decoupling_targets: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaLspServerCapabilitiesV0 {
    pub text_document_sync: u8,
    pub definition_provider: bool,
    pub hover_provider: bool,
    pub completion_provider: CompletionProviderCapabilityV0,
    pub code_action_provider: CodeActionProviderCapabilityV0,
    pub references_provider: bool,
    pub code_lens_provider: ResolveProviderCapabilityV0,
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
        server_name: "css-module-explainer",
        migration_status: "thinClient",
        transport_contract: "LSP stdio or IPC JSON-RPC",
        capabilities: current_node_lsp_capability_contract(),
        handler_surfaces: lsp_handler_surfaces(),
        migration_phases: lsp_migration_phases(),
        blocking_work_policy: vec![
            "noFullWorkspaceProgramOnRequestPath",
            "cooperativeCancellationBeforeProviderWork",
            "backgroundIndexAndTypeFactWarmup",
            "staleOrUnresolvableFastReturn",
        ],
        tsgo_client_boundary: summarize_omena_tsgo_client_boundary(),
        source_provider_adapter: source_provider_direct_rust_adapter_contract(),
        workspace_runtime_registry: workspace_runtime_registry_contract(),
        diagnostics_scheduler: rust_diagnostics_scheduler_contract(),
        query_reuse: rust_query_reuse_contract(),
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

pub fn source_provider_direct_rust_adapter_contract() -> SourceProviderDirectRustAdapterV0 {
    SourceProviderDirectRustAdapterV0 {
        product: "omena-lsp-server.source-provider-direct-rust-adapter",
        candidate_owner: "omena-bridge/sourceSyntaxIndex",
        style_definition_owner: "engine-style-parser/selectorDefinitionFacts",
        type_fact_owner: "omena-tsgo-client",
        request_path_policy: vec![
            "noNodeWorkspaceTypeResolverOnSourceProviderPath",
            "buildBridgeSourceSyntaxIndexOnDocumentChange",
            "dedupeTargetAwareSourceCandidates",
            "consumeParserCanonicalSelectorFacts",
            "consumeParserSelectorDefinitionFacts",
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
        ],
    }
}

pub fn thin_client_endpoint_contract() -> ThinClientEndpointV0 {
    ThinClientEndpointV0 {
        product: "omena-lsp-server.thin-client-endpoint",
        endpoint_name: "css-module-explainer.thin-client-runtime-endpoint",
        transport_contract: "LSP stdio JSON-RPC",
        command_owner: "dist/bin/<platform>-<arch>/omena-lsp-server",
        standalone_package: "omena-lsp-server",
        split_repository: "https://github.com/omenien/omena-lsp-server",
        cargo_install_command: "cargo install omena-lsp-server --version 0.1.3",
        node_fallback_allowed: false,
        file_watcher_globs: vec![
            "**/*.module.{scss,css,less}",
            "**/*.{ts,tsx,js,jsx,mts,cts,mjs,cjs,d.ts}",
            "**/tsconfig*.json",
            "**/jsconfig*.json",
        ],
        host_responsibilities: vec![
            "resolvePackagedRustBinary",
            "resolveStandaloneRustCommand",
            "buildThinClientServerOptions",
            "declareStaticDocumentSelector",
            "startLanguageClient",
            "registerStaticFileWatchers",
            "translateShowReferencesArguments",
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
        completion_provider: CompletionProviderCapabilityV0 {
            trigger_characters: vec!["'", "\"", "`", ",", ".", "$", "@", "-"],
            resolve_provider: false,
        },
        code_action_provider: CodeActionProviderCapabilityV0 {
            code_action_kinds: vec!["quickfix"],
            resolve_provider: false,
        },
        references_provider: true,
        code_lens_provider: ResolveProviderCapabilityV0 {
            resolve_provider: false,
        },
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspTextDocumentState {
    pub uri: String,
    pub workspace_folder_uri: Option<String>,
    pub language_id: String,
    pub version: i64,
    pub text: String,
    pub style_summary: Option<LspStyleDocumentSummary>,
    #[serde(skip)]
    pub style_candidates: Vec<LspStyleHoverCandidate>,
    #[serde(skip)]
    source_syntax_index: SourceSyntaxIndex,
    #[serde(skip)]
    pub source_selector_candidates: Vec<LspStyleHoverCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspStyleDocumentSummary {
    pub language: &'static str,
    pub selector_names: Vec<String>,
    pub custom_property_decl_names: Vec<String>,
    pub custom_property_ref_names: Vec<String>,
    pub sass_module_use_sources: Vec<String>,
    pub sass_module_forward_sources: Vec<String>,
    pub diagnostic_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspStyleHoverCandidatesResult {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub document_uri: String,
    pub workspace_folder_uri: Option<String>,
    pub language: Option<&'static str>,
    pub query_position: Option<ParserPositionV0>,
    pub candidate_count: usize,
    pub candidates: Vec<LspStyleHoverCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspStyleHoverCandidate {
    pub kind: &'static str,
    pub name: String,
    pub range: ParserRangeV0,
    pub source: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_style_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceProviderCandidateResolution {
    matched: Vec<LspStyleHoverCandidate>,
    unresolved: Vec<LspStyleHoverCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspWorkspaceFolderState {
    pub uri: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspWatchedFileChangeState {
    pub uri: String,
    pub change_type: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspShellStateSnapshot {
    pub shutdown_requested: bool,
    pub should_exit: bool,
    pub features: LspFeatureSettings,
    pub diagnostics: LspDiagnosticSettings,
    pub cancelled_request_count: usize,
    pub workspace_style_index_exhausted_count: usize,
    pub document_count: usize,
    pub workspace_folder_count: usize,
    pub configuration_change_count: usize,
    pub watched_file_event_count: usize,
    pub documents: Vec<LspTextDocumentState>,
    pub workspace_folders: Vec<LspWorkspaceFolderState>,
    pub watched_file_changes: Vec<LspWatchedFileChangeState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspFeatureSettings {
    pub definition: bool,
    pub hover: bool,
    pub completion: bool,
    pub references: bool,
    pub rename: bool,
}

impl Default for LspFeatureSettings {
    fn default() -> Self {
        Self {
            definition: true,
            hover: true,
            completion: true,
            references: true,
            rename: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspDiagnosticSettings {
    pub severity: u8,
}

impl Default for LspDiagnosticSettings {
    fn default() -> Self {
        Self { severity: 2 }
    }
}

#[derive(Debug, Default)]
pub struct LspShellState {
    pub shutdown_requested: bool,
    pub should_exit: bool,
    features: LspFeatureSettings,
    diagnostics: LspDiagnosticSettings,
    cancelled_request_ids: IncrementalCancellationRegistryV0,
    workspace_style_index_exhausted_count: usize,
    configuration_change_count: usize,
    documents: BTreeMap<String, LspTextDocumentState>,
    open_document_uris: BTreeSet<String>,
    workspace_runtime_registry: WorkspaceRuntimeRegistry,
    tsgo_workspace_process_pool: TsgoWorkspaceProcessPoolV0,
    watched_file_changes: Vec<LspWatchedFileChangeState>,
}

impl LspShellState {
    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    pub fn workspace_folder_count(&self) -> usize {
        self.workspace_runtime_registry.len()
    }

    pub fn document(&self, uri: &str) -> Option<&LspTextDocumentState> {
        self.documents.get(uri)
    }

    pub fn workspace_folder(&self, uri: &str) -> Option<&LspWorkspaceFolderState> {
        self.workspace_runtime_registry.get(uri)
    }

    pub fn snapshot(&self) -> LspShellStateSnapshot {
        LspShellStateSnapshot {
            shutdown_requested: self.shutdown_requested,
            should_exit: self.should_exit,
            features: self.features.clone(),
            diagnostics: self.diagnostics.clone(),
            cancelled_request_count: self.cancelled_request_ids.len(),
            workspace_style_index_exhausted_count: self.workspace_style_index_exhausted_count,
            document_count: self.document_count(),
            workspace_folder_count: self.workspace_folder_count(),
            configuration_change_count: self.configuration_change_count,
            watched_file_event_count: self.watched_file_changes.len(),
            documents: self.documents.values().cloned().collect(),
            workspace_folders: self.workspace_runtime_registry.folder_snapshots(),
            watched_file_changes: self.watched_file_changes.clone(),
        }
    }
}

pub fn handle_lsp_message(state: &mut LspShellState, message: Value) -> Option<Value> {
    let method = message.get("method").and_then(Value::as_str);
    let id = message.get("id").cloned();

    if method == Some(CANCEL_REQUEST_METHOD) && id.is_none() {
        cancel_lsp_request(state, message.get("params"));
        return None;
    }

    if let Some(request_id) = id.as_ref()
        && take_cancelled_request(state, request_id)
    {
        return Some(cancelled_request_response(request_id.clone()));
    }

    match (method, id) {
        (Some("initialize"), Some(request_id)) => {
            initialize_workspace_folders(state, message.get("params"));
            Some(json!({
                "jsonrpc": "2.0",
                "id": request_id,
                "result": {
                    "capabilities": current_node_lsp_capability_contract(),
                    "serverInfo": {
                        "name": "css-module-explainer-rust",
                    },
                },
            }))
        }
        (Some("initialized"), None) => {
            index_workspace_style_files(state);
            None
        }
        (Some("textDocument/didOpen"), None) => {
            did_open_text_document(state, message.get("params"));
            None
        }
        (Some("textDocument/didChange"), None) => {
            did_change_text_document(state, message.get("params"));
            None
        }
        (Some("textDocument/didClose"), None) => {
            did_close_text_document(state, message.get("params"));
            None
        }
        (Some("workspace/didChangeWorkspaceFolders"), None) => {
            did_change_workspace_folders(state, message.get("params"));
            None
        }
        (Some("workspace/didChangeConfiguration"), None) => {
            did_change_configuration(state, message.get("params"));
            None
        }
        (Some("workspace/didChangeWatchedFiles"), None) => {
            did_change_watched_files(state, message.get("params"));
            None
        }
        (Some("textDocument/hover"), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": if state.features.hover { resolve_lsp_hover(state, message.get("params")) } else { Value::Null },
        })),
        (Some("textDocument/definition"), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": if state.features.definition { resolve_lsp_definition(state, message.get("params")) } else { Value::Null },
        })),
        (Some("textDocument/references"), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": if state.features.references { resolve_lsp_references(state, message.get("params")) } else { Value::Null },
        })),
        (Some("textDocument/completion"), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": if state.features.completion { resolve_lsp_completion(state, message.get("params")) } else { Value::Null },
        })),
        (Some("textDocument/codeAction"), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": resolve_lsp_code_actions(message.get("params")),
        })),
        (Some("textDocument/codeLens"), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": if state.features.references { resolve_lsp_code_lens(state, message.get("params")) } else { Value::Null },
        })),
        (Some("textDocument/prepareRename"), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": if state.features.rename { resolve_lsp_prepare_rename(state, message.get("params")) } else { Value::Null },
        })),
        (Some("textDocument/rename"), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": if state.features.rename { resolve_lsp_rename(state, message.get("params")) } else { Value::Null },
        })),
        (Some(DEBUG_STATE_REQUEST), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": state.snapshot(),
        })),
        (Some(RUNTIME_LOOP_PROBE_REQUEST), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": {
                "now": current_time_millis(),
            },
        })),
        (Some(STYLE_HOVER_CANDIDATES_REQUEST), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": resolve_style_hover_candidates(state, message.get("params")),
        })),
        (Some(STYLE_DIAGNOSTICS_REQUEST), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": resolve_style_diagnostics(state, message.get("params")),
        })),
        (Some(SOURCE_DIAGNOSTICS_REQUEST), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": resolve_source_diagnostics(state, message.get("params")),
        })),
        (Some("shutdown"), Some(request_id)) => {
            state.shutdown_requested = true;
            Some(json!({
                "jsonrpc": "2.0",
                "id": request_id,
                "result": null,
            }))
        }
        (Some("exit"), None) => {
            state.should_exit = true;
            None
        }
        (Some(_), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "error": {
                "code": -32601,
                "message": "Method not found",
            },
        })),
        (Some(_), None) => None,
        (None, Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "error": {
                "code": -32600,
                "message": "Invalid Request",
            },
        })),
        (None, None) => None,
    }
}

fn cancel_lsp_request(state: &mut LspShellState, params: Option<&Value>) {
    let Some(id) = params.and_then(|value| value.get("id")) else {
        return;
    };
    if let Some(key) = request_id_key(id) {
        state.cancelled_request_ids.cancel(key);
    }
}

fn take_cancelled_request(state: &mut LspShellState, request_id: &Value) -> bool {
    request_id_key(request_id)
        .is_some_and(|key| state.cancelled_request_ids.take_cancelled(key.as_str()))
}

fn request_id_key(id: &Value) -> Option<String> {
    if let Some(value) = id.as_str() {
        return Some(format!("s:{value}"));
    }
    if id.is_number() {
        return Some(format!("n:{id}"));
    }
    None
}

fn cancelled_request_response(request_id: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": request_id,
        "error": {
            "code": REQUEST_CANCELLED_ERROR_CODE,
            "message": "Request cancelled",
        },
    })
}

pub fn handle_lsp_message_outputs(state: &mut LspShellState, message: Value) -> Vec<Value> {
    let method = message
        .get("method")
        .and_then(Value::as_str)
        .map(str::to_string);
    let document_uri = message
        .get("params")
        .and_then(|value| value.get("textDocument"))
        .and_then(|value| value.get("uri"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let watched_file_uris = watched_file_uris_from_message(&message);
    let diagnostics_event =
        diagnostics_schedule_event(method.as_deref(), document_uri, watched_file_uris);
    let mut outputs = Vec::new();

    if let Some(response) = handle_lsp_message(state, message) {
        outputs.push(response);
    }

    if let Some(event) = diagnostics_event {
        outputs.extend(run_diagnostics_schedule(state, event));
    }

    outputs
}

fn current_time_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis())
}

fn watched_file_uris_from_message(message: &Value) -> Vec<String> {
    message
        .get("params")
        .and_then(|value| value.get("changes"))
        .and_then(Value::as_array)
        .map(|changes| {
            changes
                .iter()
                .filter_map(|change| change.get("uri").and_then(Value::as_str))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn initialize_workspace_folders(state: &mut LspShellState, params: Option<&Value>) {
    state.workspace_runtime_registry.clear();
    if let Some(folders) = params
        .and_then(|value| value.get("workspaceFolders"))
        .and_then(Value::as_array)
    {
        for folder in folders {
            insert_workspace_folder(state, folder);
        }
        return;
    }

    if let Some(root_uri) = params
        .and_then(|value| value.get("rootUri"))
        .and_then(Value::as_str)
    {
        state
            .workspace_runtime_registry
            .insert(root_uri.to_string(), root_uri.to_string());
    }
}

fn index_workspace_style_files(state: &mut LspShellState) {
    let mut budget = WorkspaceStyleIndexBudget::with_defaults();
    index_workspace_style_files_with_budget(state, &mut budget);
}

fn index_workspace_style_files_with_budget(
    state: &mut LspShellState,
    budget: &mut WorkspaceStyleIndexBudget,
) {
    let folders = state.workspace_runtime_registry.folder_snapshots();
    for folder in folders {
        if budget.should_stop() {
            break;
        }
        let Some(path) = file_uri_to_path(folder.uri.as_str()) else {
            continue;
        };
        index_workspace_style_files_from_dir(state, folder.uri.as_str(), path.as_path(), budget);
    }
    if budget.exhausted {
        state.workspace_style_index_exhausted_count += 1;
    }
}

fn index_workspace_style_files_from_dir(
    state: &mut LspShellState,
    workspace_folder_uri: &str,
    dir: &Path,
    budget: &mut WorkspaceStyleIndexBudget,
) {
    if budget.should_stop() || should_skip_workspace_index_dir(dir) {
        return;
    }
    budget.consume_dir();
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if budget.should_stop() {
            return;
        }
        let path = entry.path();
        if path.is_dir() {
            index_workspace_style_files_from_dir(
                state,
                workspace_folder_uri,
                path.as_path(),
                budget,
            );
            continue;
        }
        if !is_indexable_style_path(path.as_path()) {
            continue;
        }
        let uri = path_to_file_uri(path.as_path());
        if state.documents.contains_key(uri.as_str()) {
            continue;
        }
        let Ok(text) = fs::read_to_string(path.as_path()) else {
            continue;
        };
        state.documents.insert(
            uri.clone(),
            lsp_text_document_state(
                uri.clone(),
                Some(workspace_folder_uri.to_string()),
                StyleLanguage::from_module_path(uri.as_str())
                    .map(style_language_label)
                    .unwrap_or("unknown")
                    .to_string(),
                0,
                text,
            ),
        );
        budget.consume_style_file();
    }
}

struct WorkspaceStyleIndexBudget {
    remaining_style_files: usize,
    remaining_dirs: usize,
    started_at: Instant,
    time_budget_ms: u128,
    exhausted: bool,
}

impl WorkspaceStyleIndexBudget {
    fn with_defaults() -> Self {
        Self::with_limits(
            WORKSPACE_STYLE_INDEX_LIMIT,
            WORKSPACE_STYLE_INDEX_DIR_LIMIT,
            WORKSPACE_STYLE_INDEX_TIME_BUDGET_MS,
        )
    }

    fn with_limits(
        remaining_style_files: usize,
        remaining_dirs: usize,
        time_budget_ms: u128,
    ) -> Self {
        Self {
            remaining_style_files,
            remaining_dirs,
            started_at: Instant::now(),
            time_budget_ms,
            exhausted: false,
        }
    }

    fn should_stop(&mut self) -> bool {
        if self.remaining_style_files == 0
            || self.remaining_dirs == 0
            || self.started_at.elapsed().as_millis() >= self.time_budget_ms
        {
            self.exhausted = true;
            return true;
        }
        false
    }

    fn consume_dir(&mut self) {
        self.remaining_dirs = self.remaining_dirs.saturating_sub(1);
    }

    fn consume_style_file(&mut self) {
        self.remaining_style_files = self.remaining_style_files.saturating_sub(1);
    }
}

fn should_skip_workspace_index_dir(dir: &Path) -> bool {
    dir.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            matches!(
                name,
                ".cache"
                    | ".git"
                    | ".next"
                    | ".turbo"
                    | "build"
                    | "coverage"
                    | "dist"
                    | "node_modules"
                    | "out"
                    | "target"
            )
        })
}

fn is_indexable_style_path(path: &Path) -> bool {
    StyleLanguage::from_module_path(path.to_string_lossy().as_ref()).is_some()
}

fn path_to_file_uri(path: &Path) -> String {
    format!("file://{}", path.to_string_lossy())
}

fn normalize_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(_) | Component::RootDir | Component::Prefix(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    normalized
}

fn lsp_text_document_state(
    uri: String,
    workspace_folder_uri: Option<String>,
    language_id: String,
    version: i64,
    text: String,
) -> LspTextDocumentState {
    let mut document = LspTextDocumentState {
        uri,
        workspace_folder_uri,
        language_id,
        version,
        text,
        style_summary: None,
        style_candidates: Vec::new(),
        source_syntax_index: SourceSyntaxIndex::default(),
        source_selector_candidates: Vec::new(),
    };
    refresh_document_reusable_indexes(&mut document);
    document
}

fn did_open_text_document(state: &mut LspShellState, params: Option<&Value>) {
    let Some(document) = params.and_then(|value| value.get("textDocument")) else {
        return;
    };
    let Some(uri) = document.get("uri").and_then(Value::as_str) else {
        return;
    };

    state.open_document_uris.insert(uri.to_string());
    state.documents.insert(
        uri.to_string(),
        lsp_text_document_state(
            uri.to_string(),
            resolve_workspace_folder_uri(state, uri),
            document
                .get("languageId")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
            document.get("version").and_then(Value::as_i64).unwrap_or(0),
            document
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
        ),
    );
    refresh_source_type_fact_candidates_for_document(state, uri);
}

fn did_change_text_document(state: &mut LspShellState, params: Option<&Value>) {
    let Some(text_document) = params.and_then(|value| value.get("textDocument")) else {
        return;
    };
    let Some(uri) = text_document.get("uri").and_then(Value::as_str) else {
        return;
    };
    let Some(existing) = state.documents.get_mut(uri) else {
        return;
    };

    if let Some(version) = text_document.get("version").and_then(Value::as_i64) {
        existing.version = version;
    }
    let Some(changes) = params
        .and_then(|value| value.get("contentChanges"))
        .and_then(Value::as_array)
    else {
        return;
    };

    let mut text_changed = false;
    for change in changes {
        if apply_text_document_content_change(existing, change) {
            text_changed = true;
        }
    }
    if text_changed {
        refresh_document_reusable_indexes(existing);
    }
    if text_changed {
        refresh_source_type_fact_candidates_for_document(state, uri);
    }
}

fn refresh_source_type_fact_candidates_for_document(state: &mut LspShellState, uri: &str) {
    let Some(document) = state.document(uri) else {
        return;
    };
    if is_style_document_uri(document.uri.as_str()) {
        return;
    }
    let type_fact_targets = document.source_syntax_index.type_fact_targets.clone();
    if type_fact_targets.is_empty() {
        return;
    }
    let Some(request) = tsgo_type_fact_request_for_document(document, type_fact_targets.as_slice())
    else {
        return;
    };
    let Some(tsgo_command) = tsgo_process_command_for_workspace(request.workspace_root.as_str())
    else {
        return;
    };
    let config = omena_tsgo_client::TsgoWorkspaceProcessConfigV0 {
        workspace_root: request.workspace_root.clone(),
        command: tsgo_command,
    };
    if state
        .tsgo_workspace_process_pool
        .ensure_workspace_process(config)
        .is_err()
    {
        return;
    }

    let pool = std::mem::take(&mut state.tsgo_workspace_process_pool);
    let mut provider = TsgoJsonRpcTypeFactProviderV0::new(pool);
    let entries = provider.collect_type_facts(&request).ok();
    state.tsgo_workspace_process_pool = provider.into_transport();
    let Some(entries) = entries else {
        return;
    };
    apply_source_type_fact_results_to_document(state, uri, entries.as_slice());
}

fn tsgo_type_fact_request_for_document(
    document: &LspTextDocumentState,
    type_fact_targets: &[SourceTypeFactTarget],
) -> Option<TsgoTypeFactRequestV0> {
    let file_path = file_uri_to_path(document.uri.as_str())?;
    let workspace_root = document
        .workspace_folder_uri
        .as_deref()
        .and_then(file_uri_to_path)
        .or_else(|| file_path.parent().map(Path::to_path_buf))?;
    let config_path = find_tsconfig_for_workspace(workspace_root.as_path())?;
    let file_path = file_path.to_string_lossy().to_string();
    let targets = type_fact_targets
        .iter()
        .filter_map(|target| {
            let position = u32::try_from(target.byte_span.start).ok()?;
            Some(TsgoTypeFactTargetV0 {
                file_path: file_path.clone(),
                expression_id: target.expression_id.clone(),
                position,
            })
        })
        .collect::<Vec<_>>();
    if targets.is_empty() {
        return None;
    }
    Some(TsgoTypeFactRequestV0 {
        workspace_root: workspace_root.to_string_lossy().to_string(),
        config_path: config_path.to_string_lossy().to_string(),
        targets,
    })
}

fn apply_source_type_fact_results_to_document(
    state: &mut LspShellState,
    uri: &str,
    entries: &[TsgoTypeFactResultEntryV0],
) {
    let Some(document) = state.document(uri) else {
        return;
    };
    let mut references = document.source_syntax_index.selector_references.clone();
    let targets = document.source_syntax_index.type_fact_targets.clone();
    for target in targets {
        let Some(entry) = entries
            .iter()
            .find(|entry| entry.expression_id == target.expression_id)
        else {
            continue;
        };
        for selector_name in project_tsgo_type_fact_target(entry.resolved_type.clone(), &target) {
            push_selector_reference(
                target.byte_span,
                Some(selector_name),
                SourceSelectorReferenceMatchKind::Exact,
                target.target_style_uri.as_deref(),
                &mut references,
            );
        }
    }
    canonicalize_source_selector_references(&mut references);
    let Some(document) = state.documents.get_mut(uri) else {
        return;
    };
    document.source_syntax_index.selector_references = references;
    let source_syntax_index = document.source_syntax_index.clone();
    document.source_selector_candidates =
        source_selector_candidates_from_index(document, &source_syntax_index);
}

fn project_tsgo_type_fact_target(
    resolved_type: TsgoResolvedTypeV0,
    target: &SourceTypeFactTarget,
) -> Vec<String> {
    if resolved_type.kind != "union" {
        return Vec::new();
    }
    let mut names = resolved_type
        .values
        .into_iter()
        .filter(|value| value.chars().all(is_css_identifier_continue))
        .map(|value| format!("{}{}{}", target.prefix, value, target.suffix))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    names
}

fn push_selector_reference(
    byte_span: ParserByteSpanV0,
    selector_name: Option<String>,
    match_kind: SourceSelectorReferenceMatchKind,
    target_style_uri: Option<&str>,
    references: &mut Vec<SourceSelectorReferenceFact>,
) {
    references.push(SourceSelectorReferenceFact {
        byte_span,
        selector_name,
        match_kind,
        target_style_uri: target_style_uri.map(ToString::to_string),
    });
}

fn find_tsconfig_for_workspace(workspace_root: &Path) -> Option<PathBuf> {
    let mut current = Some(workspace_root);
    while let Some(dir) = current {
        for file_name in ["tsconfig.json", "jsconfig.json"] {
            let candidate = dir.join(file_name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
        current = dir.parent();
    }
    None
}

fn tsgo_process_command_for_workspace(
    workspace_root: &str,
) -> Option<omena_tsgo_client::TsgoProcessCommandV0> {
    let tsgo_path = resolve_tsgo_binary_path()?;
    Some(build_tsgo_process_command(
        tsgo_path.to_string_lossy().as_ref(),
        workspace_root,
        std::env::var("CME_TSGO_CHECKERS")
            .ok()
            .and_then(|value| value.parse::<usize>().ok()),
    ))
}

fn resolve_tsgo_binary_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("CME_TSGO_PATH")
        && !path.is_empty()
    {
        let path = PathBuf::from(path);
        if path.exists() {
            return Some(path);
        }
    }
    let binary_name = if cfg!(windows) { "tsgo.exe" } else { "tsgo" };
    let sibling = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join(binary_name)));
    if let Some(path) = sibling
        && path.exists()
    {
        return Some(path);
    }
    None
}

fn apply_text_document_content_change(document: &mut LspTextDocumentState, change: &Value) -> bool {
    let Some(next_text) = change.get("text").and_then(Value::as_str) else {
        return false;
    };
    let Some(range) = change.get("range").and_then(lsp_range_from_value) else {
        document.text = next_text.to_string();
        return true;
    };
    let Some(start_offset) = byte_offset_for_parser_position(document.text.as_str(), range.start)
    else {
        return false;
    };
    let Some(end_offset) = byte_offset_for_parser_position(document.text.as_str(), range.end)
    else {
        return false;
    };
    if start_offset > end_offset {
        return false;
    }
    document
        .text
        .replace_range(start_offset..end_offset, next_text);
    true
}

fn did_close_text_document(state: &mut LspShellState, params: Option<&Value>) {
    let Some(uri) = params
        .and_then(|value| value.get("textDocument"))
        .and_then(|value| value.get("uri"))
        .and_then(Value::as_str)
    else {
        return;
    };
    state.open_document_uris.remove(uri);
    if is_style_document_uri(uri) && reload_indexed_style_document_from_disk(state, uri) {
        return;
    }
    state.documents.remove(uri);
}

fn did_change_workspace_folders(state: &mut LspShellState, params: Option<&Value>) {
    let event = params.and_then(|value| value.get("event"));
    if let Some(removed) = event
        .and_then(|value| value.get("removed"))
        .and_then(Value::as_array)
    {
        for folder in removed {
            if let Some(uri) = folder.get("uri").and_then(Value::as_str) {
                state.workspace_runtime_registry.remove(uri);
                remove_indexed_documents_for_workspace(state, uri);
            }
        }
    }
    if let Some(added) = event
        .and_then(|value| value.get("added"))
        .and_then(Value::as_array)
    {
        for folder in added {
            insert_workspace_folder(state, folder);
        }
        index_workspace_style_files(state);
    }
    refresh_document_workspace_owners(state);
}

fn remove_indexed_documents_for_workspace(state: &mut LspShellState, workspace_uri: &str) {
    state.documents.retain(|uri, document| {
        state.open_document_uris.contains(uri)
            || document.workspace_folder_uri.as_deref() != Some(workspace_uri)
    });
}

fn did_change_configuration(state: &mut LspShellState, params: Option<&Value>) {
    state.configuration_change_count += 1;
    let Some(settings) = params
        .and_then(|value| value.get("settings"))
        .and_then(|value| value.get("cssModuleExplainer"))
    else {
        return;
    };
    apply_feature_settings(state, settings.get("features"));
    apply_diagnostic_settings(state, settings.get("diagnostics"));
}

fn apply_feature_settings(state: &mut LspShellState, features: Option<&Value>) {
    let Some(features) = features.and_then(Value::as_object) else {
        return;
    };
    if let Some(value) = features.get("definition").and_then(Value::as_bool) {
        state.features.definition = value;
    }
    if let Some(value) = features.get("hover").and_then(Value::as_bool) {
        state.features.hover = value;
    }
    if let Some(value) = features.get("completion").and_then(Value::as_bool) {
        state.features.completion = value;
    }
    if let Some(value) = features.get("references").and_then(Value::as_bool) {
        state.features.references = value;
    }
    if let Some(value) = features.get("rename").and_then(Value::as_bool) {
        state.features.rename = value;
    }
}

fn apply_diagnostic_settings(state: &mut LspShellState, diagnostics: Option<&Value>) {
    let Some(diagnostics) = diagnostics.and_then(Value::as_object) else {
        return;
    };
    if let Some(value) = diagnostics
        .get("severity")
        .and_then(Value::as_str)
        .and_then(diagnostic_severity_code)
    {
        state.diagnostics.severity = value;
    }
}

fn diagnostic_severity_code(value: &str) -> Option<u8> {
    match value {
        "error" => Some(1),
        "warning" => Some(2),
        "information" => Some(3),
        "hint" => Some(4),
        _ => None,
    }
}

fn did_change_watched_files(state: &mut LspShellState, params: Option<&Value>) {
    let Some(changes) = params
        .and_then(|value| value.get("changes"))
        .and_then(Value::as_array)
    else {
        return;
    };
    for change in changes {
        let Some(uri) = change.get("uri").and_then(Value::as_str) else {
            continue;
        };
        let change_type = change.get("type").and_then(Value::as_u64).unwrap_or(0);
        state.watched_file_changes.push(LspWatchedFileChangeState {
            uri: uri.to_string(),
            change_type,
        });
        apply_watched_file_change_to_index(state, uri, change_type);
    }
}

fn apply_watched_file_change_to_index(state: &mut LspShellState, uri: &str, change_type: u64) {
    if !is_style_document_uri(uri) {
        return;
    }
    if state.open_document_uris.contains(uri) {
        return;
    }
    if change_type == 3 {
        state.documents.remove(uri);
        return;
    }

    reload_indexed_style_document_from_disk(state, uri);
}

fn reload_indexed_style_document_from_disk(state: &mut LspShellState, uri: &str) -> bool {
    let Some(path) = file_uri_to_path(uri) else {
        return false;
    };
    let Ok(text) = fs::read_to_string(path) else {
        return false;
    };
    state.documents.insert(
        uri.to_string(),
        lsp_text_document_state(
            uri.to_string(),
            resolve_workspace_folder_uri(state, uri),
            StyleLanguage::from_module_path(uri)
                .map(style_language_label)
                .unwrap_or("unknown")
                .to_string(),
            0,
            text,
        ),
    );
    true
}

fn insert_workspace_folder(state: &mut LspShellState, folder: &Value) {
    let Some(uri) = folder.get("uri").and_then(Value::as_str) else {
        return;
    };
    state.workspace_runtime_registry.insert(
        uri.to_string(),
        folder
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or(uri)
            .to_string(),
    );
}

fn refresh_document_workspace_owners(state: &mut LspShellState) {
    let workspace_runtime_registry = state.workspace_runtime_registry.clone();
    for document in state.documents.values_mut() {
        document.workspace_folder_uri =
            workspace_runtime_registry.resolve_owner_uri(document.uri.as_str());
    }
}

fn resolve_workspace_folder_uri(state: &LspShellState, document_uri: &str) -> Option<String> {
    state
        .workspace_runtime_registry
        .resolve_owner_uri(document_uri)
}

fn summarize_style_document(uri: &str, text: Option<&str>) -> Option<LspStyleDocumentSummary> {
    let text = text?;
    let sheet = parse_style_module(uri, text)?;
    let index = summarize_css_modules_intermediate(&sheet);
    Some(LspStyleDocumentSummary {
        language: style_language_label(sheet.language),
        selector_names: index.selectors.names,
        custom_property_decl_names: index.custom_properties.decl_names,
        custom_property_ref_names: index.custom_properties.ref_names,
        sass_module_use_sources: index.sass.module_use_sources,
        sass_module_forward_sources: index.sass.module_forward_sources,
        diagnostic_count: sheet.diagnostics.len(),
    })
}

pub fn resolve_style_hover_candidates(
    state: &LspShellState,
    params: Option<&Value>,
) -> LspStyleHoverCandidatesResult {
    let document_uri = document_uri_from_params(params);
    let query_position = lsp_position_from_params(params);
    let Some(document) = state.document(&document_uri) else {
        return empty_style_hover_candidates_result(document_uri, None, query_position);
    };

    let Some((language, mut candidates)) = style_hover_candidates_for_document(document) else {
        return empty_style_hover_candidates_result(
            document_uri,
            document.workspace_folder_uri.clone(),
            query_position,
        );
    };

    if let Some(position) = query_position {
        candidates.retain(|candidate| parser_range_contains_position(&candidate.range, position));
    }

    LspStyleHoverCandidatesResult {
        schema_version: "0",
        product: "omena-lsp-server.style-hover-candidates",
        document_uri,
        workspace_folder_uri: document.workspace_folder_uri.clone(),
        language: Some(language),
        query_position,
        candidate_count: candidates.len(),
        candidates,
    }
}

fn style_hover_candidates_for_document(
    document: &LspTextDocumentState,
) -> Option<(&'static str, Vec<LspStyleHoverCandidate>)> {
    let summary = document.style_summary.as_ref()?;
    Some((summary.language, document.style_candidates.clone()))
}

fn style_text_for_uri(state: &LspShellState, uri: &str) -> Option<String> {
    state
        .document(uri)
        .map(|document| document.text.clone())
        .or_else(|| fs::read_to_string(file_uri_to_path(uri)?).ok())
}

fn style_hover_candidates_for_uri(
    state: &LspShellState,
    uri: &str,
) -> Option<(&'static str, Vec<LspStyleHoverCandidate>)> {
    if let Some(document) = state.document(uri) {
        return style_hover_candidates_for_document(document);
    }
    let text = style_text_for_uri(state, uri)?;
    collect_style_hover_candidates(uri, text.as_str())
}

fn resolve_lsp_definition(state: &LspShellState, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    let Some(position) = lsp_position_from_params(params) else {
        return Value::Null;
    };
    let Some(document) = state.document(&document_uri) else {
        return Value::Null;
    };
    if !is_style_document_uri(document.uri.as_str()) {
        return resolve_source_lsp_definition(state, document, position);
    }

    let Some((_, candidates)) = style_hover_candidates_for_document(document) else {
        return Value::Null;
    };
    let Some(candidate) = candidates
        .iter()
        .find(|candidate| parser_range_contains_position(&candidate.range, position))
    else {
        return Value::Null;
    };
    if is_sass_symbol_reference_kind(candidate.kind) {
        let definitions = sass_symbol_definitions_for_candidate(state, document, candidate);
        if definitions.is_empty() {
            return Value::Null;
        }
        return json!(
            definitions
                .into_iter()
                .map(|(uri, definition)| json!({ "uri": uri, "range": definition.range }))
                .collect::<Vec<_>>()
        );
    }
    let target = if candidate.kind == "customPropertyReference" {
        candidates
            .iter()
            .find(|target| {
                target.kind == "customPropertyDeclaration" && target.name == candidate.name
            })
            .unwrap_or(candidate)
    } else {
        candidate
    };

    json!([
        {
            "uri": document.uri.as_str(),
            "range": target.range,
        },
    ])
}

fn resolve_lsp_references(state: &LspShellState, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    let Some(position) = lsp_position_from_params(params) else {
        return Value::Null;
    };
    let Some(document) = state.document(&document_uri) else {
        return Value::Null;
    };
    if !is_style_document_uri(document.uri.as_str()) {
        return resolve_source_lsp_references(state, document, position, params);
    }

    let Some((_, candidates)) = style_hover_candidates_for_document(document) else {
        return Value::Null;
    };
    let Some(candidate) = candidates
        .iter()
        .find(|candidate| parser_range_contains_position(&candidate.range, position))
    else {
        return Value::Null;
    };
    let include_declaration = include_declaration_from_params(params);
    let mut locations: Vec<Value> = if candidate.kind.starts_with("customProperty") {
        candidates
            .iter()
            .filter(|target| {
                target.name == candidate.name
                    && (target.kind == "customPropertyReference"
                        || (include_declaration && target.kind == "customPropertyDeclaration"))
            })
            .map(|target| json!({ "uri": document.uri.as_str(), "range": target.range }))
            .collect()
    } else if is_sass_symbol_candidate_kind(candidate.kind) {
        let mut locations = Vec::new();
        if include_declaration {
            locations.extend(
                sass_symbol_definitions_for_candidate(state, document, candidate)
                    .into_iter()
                    .map(|(uri, definition)| json!({ "uri": uri, "range": definition.range })),
            );
        }
        locations.extend(
            candidates
                .iter()
                .filter(|target| sass_symbol_reference_matches(candidate, target))
                .map(|target| json!({ "uri": document.uri.as_str(), "range": target.range })),
        );
        locations
    } else if candidate.kind == "selector" {
        let mut locations = if include_declaration {
            vec![json!({ "uri": document.uri.as_str(), "range": candidate.range })]
        } else {
            Vec::new()
        };
        locations.extend(selector_reference_locations_from_open_documents(
            state,
            candidate.name.as_str(),
            document.workspace_folder_uri.as_deref(),
            Some(document.uri.as_str()),
        ));
        locations
    } else if include_declaration {
        vec![json!({ "uri": document.uri.as_str(), "range": candidate.range })]
    } else {
        Vec::new()
    };

    locations.sort_by_key(|location| {
        let line = location
            .pointer("/range/start/line")
            .and_then(Value::as_u64)
            .unwrap_or_default();
        let character = location
            .pointer("/range/start/character")
            .and_then(Value::as_u64)
            .unwrap_or_default();
        (line, character)
    });
    json!(locations)
}

fn resolve_lsp_completion(state: &LspShellState, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    let Some(document) = state.document(&document_uri) else {
        return Value::Null;
    };
    if !is_style_document_uri(document.uri.as_str()) {
        return resolve_source_lsp_completion(state, document, params);
    }

    let Some((_, candidates)) = style_hover_candidates_for_document(document) else {
        return Value::Null;
    };

    let mut emitted_labels = BTreeSet::new();
    let items: Vec<Value> = candidates
        .iter()
        .filter_map(|candidate| match candidate.kind {
            "selector" => Some((format!(".{}", candidate.name), 7, "CSS Module selector")),
            "customPropertyDeclaration" => {
                Some((candidate.name.clone(), 10, "CSS custom property"))
            }
            _ => None,
        })
        .filter(|(label, _, _)| emitted_labels.insert(label.clone()))
        .map(|(label, kind, detail)| {
            json!({
                "label": label,
                "kind": kind,
                "detail": detail,
                "data": {
                    "source": "openedStyleDocumentIndex",
                },
            })
        })
        .collect();

    json!({
        "isIncomplete": false,
        "items": items,
    })
}

fn resolve_style_diagnostics(state: &LspShellState, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    resolve_style_diagnostics_for_uri(state, document_uri.as_str())
}

fn resolve_source_diagnostics(state: &LspShellState, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    resolve_source_diagnostics_for_uri(state, document_uri.as_str())
}

fn resolve_document_diagnostics_for_uri(state: &LspShellState, document_uri: &str) -> Value {
    if is_style_document_uri(document_uri) {
        resolve_style_diagnostics_for_uri(state, document_uri)
    } else {
        resolve_source_diagnostics_for_uri(state, document_uri)
    }
}

fn resolve_style_diagnostics_for_uri(state: &LspShellState, document_uri: &str) -> Value {
    let Some(document) = state.document(document_uri) else {
        return json!([]);
    };
    let Some((_, candidates)) = style_hover_candidates_for_document(document) else {
        return json!([]);
    };

    let decl_names: BTreeSet<&str> = candidates
        .iter()
        .filter(|candidate| candidate.kind == "customPropertyDeclaration")
        .map(|candidate| candidate.name.as_str())
        .collect();
    if decl_names.is_empty() {
        return json!([]);
    }

    let insertion_range = end_of_document_range(document.text.as_str());
    let diagnostics: Vec<Value> = candidates
        .iter()
        .filter(|candidate| {
            candidate.kind == "customPropertyReference"
                && !decl_names.contains(candidate.name.as_str())
        })
        .map(|candidate| {
            json!({
                "range": candidate.range,
                "severity": state.diagnostics.severity,
                "source": "css-module-explainer",
                "message": format!(
                    "CSS custom property '{}' not found in indexed style tokens.",
                    candidate.name
                ),
                "data": {
                    "createCustomProperty": {
                        "uri": document.uri.as_str(),
                        "range": insertion_range,
                        "newText": format!("\n\n:root {{\n  {}: ;\n}}\n", candidate.name),
                        "propertyName": candidate.name.as_str(),
                    },
                },
            })
        })
        .collect();

    json!(diagnostics)
}

fn resolve_source_diagnostics_for_uri(state: &LspShellState, document_uri: &str) -> Value {
    let Some(document) = state.document(document_uri) else {
        return json!([]);
    };
    if is_style_document_uri(document.uri.as_str()) {
        return json!([]);
    }

    let diagnostics: Vec<Value> = resolve_source_provider_candidates(state, document)
        .unresolved
        .into_iter()
        .filter(|candidate| candidate.kind == "sourceSelectorReference")
        .filter_map(|candidate| {
            let (target_style_uri, target_style_document) = source_selector_diagnostic_target(
                state,
                &candidate,
                document.workspace_folder_uri.as_deref(),
            )?;
            let insertion_range = end_of_document_range(target_style_document.text.as_str());
            let has_existing_style_content = !target_style_document.text.trim().is_empty();
            Some(json!({
                "range": candidate.range,
                "severity": state.diagnostics.severity,
                "source": "css-module-explainer",
                "message": format!(
                    "CSS Module selector '.{}' not found in indexed style tokens.",
                    candidate.name
                ),
                "data": {
                    "createSelector": {
                        "uri": target_style_uri.as_str(),
                        "range": insertion_range,
                        "newText": if has_existing_style_content {
                            format!("\n\n.{} {{\n}}\n", candidate.name)
                        } else {
                            format!(".{} {{\n}}\n", candidate.name)
                        },
                        "selectorName": candidate.name.as_str(),
                    },
                },
            }))
        })
        .collect();

    json!(diagnostics)
}

fn source_selector_diagnostic_target<'a>(
    state: &'a LspShellState,
    candidate: &LspStyleHoverCandidate,
    workspace_folder_uri: Option<&str>,
) -> Option<(String, &'a LspTextDocumentState)> {
    if let Some(target_style_uri) = candidate.target_style_uri.as_deref() {
        let target_document = state.document(target_style_uri)?;
        if !is_style_document_uri(target_document.uri.as_str())
            || !workspace_folder_compatible(workspace_folder_uri, target_document)
        {
            return None;
        }
        return Some((target_style_uri.to_string(), target_document));
    }

    first_style_document_for_workspace(state, workspace_folder_uri)
}

fn resolve_lsp_code_actions(params: Option<&Value>) -> Value {
    let Some(diagnostics) = params
        .and_then(|value| value.get("context"))
        .and_then(|value| value.get("diagnostics"))
        .and_then(Value::as_array)
    else {
        return Value::Null;
    };

    let actions: Vec<Value> = diagnostics
        .iter()
        .enumerate()
        .filter_map(|(index, diagnostic)| {
            let payload = diagnostic
                .pointer("/data/createCustomProperty")
                .and_then(Value::as_object)?;
            let uri = payload.get("uri").and_then(Value::as_str)?;
            let range = payload.get("range")?;
            let new_text = payload.get("newText").and_then(Value::as_str)?;
            let property_name = payload.get("propertyName").and_then(Value::as_str)?;
            let mut changes = serde_json::Map::new();
            changes.insert(
                uri.to_string(),
                json!([
                    {
                        "range": range,
                        "newText": new_text,
                    },
                ]),
            );

            Some(json!({
                "title": format!("Add '{}' to {}", property_name, file_label_from_uri(uri)),
                "kind": "quickfix",
                "diagnostics": [diagnostic],
                "edit": {
                    "changes": Value::Object(changes),
                },
                "data": {
                    "source": "openedStyleDocumentIndex",
                    "diagnosticIndex": index,
                },
            }))
        })
        .chain(diagnostics.iter().enumerate().filter_map(|(index, diagnostic)| {
            let payload = diagnostic
                .pointer("/data/createSelector")
                .and_then(Value::as_object)?;
            let uri = payload.get("uri").and_then(Value::as_str)?;
            let range = payload.get("range")?;
            let new_text = payload.get("newText").and_then(Value::as_str)?;
            let selector_name = payload.get("selectorName").and_then(Value::as_str)?;
            let mut changes = serde_json::Map::new();
            changes.insert(
                uri.to_string(),
                json!([
                    {
                        "range": range,
                        "newText": new_text,
                    },
                ]),
            );

            Some(json!({
                "title": format!("Add '.{}' to {}", selector_name, file_label_from_uri(uri)),
                "kind": "quickfix",
                "diagnostics": [diagnostic],
                "edit": {
                    "changes": Value::Object(changes),
                },
                "data": {
                    "source": "omenaBridgeSourceSyntaxIndex",
                    "diagnosticIndex": index,
                },
            }))
        }))
        .collect();

    if actions.is_empty() {
        Value::Null
    } else {
        json!(actions)
    }
}

fn resolve_lsp_code_lens(state: &LspShellState, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    let Some(document) = state.document(document_uri.as_str()) else {
        return Value::Null;
    };
    let Some((_, candidates)) = style_hover_candidates_for_document(document) else {
        return Value::Null;
    };

    let mut lenses = Vec::new();
    let mut emitted_selectors = BTreeSet::new();
    let reference_locations_by_name = selector_reference_locations_by_name_from_open_documents(
        state,
        document.workspace_folder_uri.as_deref(),
        Some(document.uri.as_str()),
    );
    for candidate in candidates
        .iter()
        .filter(|candidate| candidate.kind == "selector")
    {
        if !emitted_selectors.insert(candidate.name.as_str()) {
            continue;
        }
        let locations = reference_locations_by_name
            .get(candidate.name.as_str())
            .cloned()
            .unwrap_or_default();
        if locations.is_empty() {
            continue;
        }
        let position = candidate.range.start;
        lenses.push(json!({
            "range": {
                "start": position,
                "end": position,
            },
            "command": {
                "title": reference_lens_title(locations.len()),
                "command": "editor.action.showReferences",
                "arguments": [
                    document.uri.as_str(),
                    position,
                    locations,
                ],
            },
        }));
    }
    lenses.sort_by_key(lsp_range_start_sort_key);

    if lenses.is_empty() {
        Value::Null
    } else {
        json!(lenses)
    }
}

fn selector_reference_locations_from_open_documents(
    state: &LspShellState,
    selector_name: &str,
    workspace_folder_uri: Option<&str>,
    target_style_uri: Option<&str>,
) -> Vec<Value> {
    selector_reference_locations_by_name_from_open_documents(
        state,
        workspace_folder_uri,
        target_style_uri,
    )
    .remove(selector_name)
    .unwrap_or_default()
}

fn selector_reference_locations_by_name_from_open_documents(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    target_style_uri: Option<&str>,
) -> BTreeMap<String, Vec<Value>> {
    let mut locations_by_name: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    let definitions =
        style_selector_definitions_from_open_documents(state, "", workspace_folder_uri);
    for document in state.documents.values() {
        if is_style_document_uri(document.uri.as_str()) {
            continue;
        }
        if !workspace_folder_compatible(workspace_folder_uri, document) {
            continue;
        }
        for candidate in collect_source_selector_reference_candidates(state, document) {
            if !source_candidate_matches_target_style(&candidate, target_style_uri) {
                continue;
            }
            for selector_name in source_candidate_selector_names(
                &candidate,
                definitions.as_slice(),
                target_style_uri,
            ) {
                locations_by_name
                    .entry(selector_name)
                    .or_default()
                    .push(json!({
                        "uri": document.uri.as_str(),
                        "range": candidate.range,
                    }));
            }
        }
    }
    for locations in locations_by_name.values_mut() {
        locations.sort_by_key(location_sort_key);
        locations
            .dedup_by(|left, right| location_identity_key(left) == location_identity_key(right));
    }
    locations_by_name
}

fn source_candidate_selector_names(
    candidate: &LspStyleHoverCandidate,
    definitions: &[(String, LspStyleHoverCandidate)],
    target_style_uri: Option<&str>,
) -> Vec<String> {
    if candidate.kind != "sourceSelectorPrefixReference" {
        return vec![candidate.name.clone()];
    }
    let mut names: Vec<String> = definitions
        .iter()
        .filter(|(uri, definition)| {
            source_candidate_matches_selector(candidate, definition.name.as_str())
                && candidate
                    .target_style_uri
                    .as_deref()
                    .or(target_style_uri)
                    .is_none_or(|target_uri| target_uri == uri)
        })
        .map(|(_, definition)| definition.name.clone())
        .collect();
    names.sort();
    names.dedup();
    names
}

fn source_candidate_matches_target_style(
    candidate: &LspStyleHoverCandidate,
    target_style_uri: Option<&str>,
) -> bool {
    target_style_uri.is_none_or(|target_uri| {
        candidate
            .target_style_uri
            .as_deref()
            .is_none_or(|candidate_target_uri| candidate_target_uri == target_uri)
    })
}

fn is_sass_symbol_candidate_kind(kind: &str) -> bool {
    sass_symbol_kind_from_candidate_kind(kind).is_some()
}

fn is_sass_symbol_reference_kind(kind: &str) -> bool {
    matches!(
        kind,
        "sassVariableReference"
            | "sassMixinInclude"
            | "sassFunctionCall"
            | "sassMixinReference"
            | "sassFunctionReference"
            | "sassSymbolReference"
    )
}

fn is_sass_symbol_declaration_kind(kind: &str) -> bool {
    matches!(
        kind,
        "sassVariableDeclaration"
            | "sassMixinDeclaration"
            | "sassFunctionDeclaration"
            | "sassSymbolDeclaration"
    )
}

fn sass_symbol_kind_from_candidate_kind(kind: &str) -> Option<&'static str> {
    match kind {
        "sassVariableDeclaration" | "sassVariableReference" => Some("variable"),
        "sassMixinDeclaration" | "sassMixinInclude" | "sassMixinReference" => Some("mixin"),
        "sassFunctionDeclaration" | "sassFunctionCall" | "sassFunctionReference" => {
            Some("function")
        }
        "sassSymbolDeclaration" | "sassSymbolReference" => Some("symbol"),
        _ => None,
    }
}

fn sass_symbol_definitions_for_candidate(
    state: &LspShellState,
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
) -> Vec<(String, LspStyleHoverCandidate)> {
    let Some(symbol_kind) = sass_symbol_kind_from_candidate_kind(candidate.kind) else {
        return Vec::new();
    };
    if is_sass_symbol_declaration_kind(candidate.kind) {
        return vec![(document.uri.clone(), candidate.clone())];
    }

    let mut definitions = if candidate.namespace.is_none() {
        sass_symbol_declarations_in_document(document, symbol_kind, candidate)
    } else {
        Vec::new()
    };
    if candidate.namespace.is_none() && !definitions.is_empty() {
        return definitions;
    }

    for target_uri in sass_module_target_uris_for_candidate(state, document, candidate) {
        definitions.extend(sass_symbol_declarations_for_uri(
            state,
            target_uri.as_str(),
            symbol_kind,
            candidate,
        ));
    }
    definitions.sort_by_key(|(uri, target)| {
        (
            uri.clone(),
            target.range.start.line,
            target.range.start.character,
        )
    });
    definitions.dedup_by(|left, right| {
        left.0 == right.0
            && left.1.kind == right.1.kind
            && left.1.name == right.1.name
            && left.1.range == right.1.range
    });
    definitions
}

fn sass_symbol_declarations_for_uri(
    state: &LspShellState,
    target_uri: &str,
    symbol_kind: &str,
    candidate: &LspStyleHoverCandidate,
) -> Vec<(String, LspStyleHoverCandidate)> {
    if let Some(target_document) = state.document(target_uri) {
        return sass_symbol_declarations_with_forwards(
            state,
            target_document,
            symbol_kind,
            candidate,
            &mut BTreeSet::new(),
        );
    }
    let Some((_, candidates)) = style_hover_candidates_for_uri(state, target_uri) else {
        return Vec::new();
    };
    candidates
        .into_iter()
        .filter(|target| {
            is_sass_symbol_declaration_kind(target.kind)
                && sass_symbol_kind_from_candidate_kind(target.kind) == Some(symbol_kind)
                && target.name == candidate.name
        })
        .map(|target| (target_uri.to_string(), target))
        .collect()
}

fn sass_symbol_declarations_in_document(
    document: &LspTextDocumentState,
    symbol_kind: &str,
    candidate: &LspStyleHoverCandidate,
) -> Vec<(String, LspStyleHoverCandidate)> {
    document
        .style_candidates
        .iter()
        .filter(|target| {
            is_sass_symbol_declaration_kind(target.kind)
                && sass_symbol_kind_from_candidate_kind(target.kind) == Some(symbol_kind)
                && target.name == candidate.name
        })
        .cloned()
        .map(|target| (document.uri.clone(), target))
        .collect()
}

fn sass_module_target_uris_for_candidate(
    state: &LspShellState,
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
) -> Vec<String> {
    let Some(sheet) = parse_style_module(document.uri.as_str(), document.text.as_str()) else {
        return Vec::new();
    };
    let index = summarize_css_modules_intermediate(&sheet);
    let mut uris = Vec::new();
    for edge in index.sass.module_use_edges.iter().filter(|edge| {
        if let Some(namespace) = candidate.namespace.as_deref() {
            edge.namespace.as_deref() == Some(namespace)
        } else {
            edge.namespace_kind == "wildcard"
        }
    }) {
        if edge.source.starts_with("sass:") {
            continue;
        }
        if let Some(uri) = style_uri_for_style_specifier(
            document.uri.as_str(),
            document.workspace_folder_uri.as_deref(),
            edge.source.as_str(),
        ) {
            uris.push(uri);
        }
    }
    for forward_source in &index.sass.module_forward_sources {
        if forward_source.starts_with("sass:") {
            continue;
        }
        if let Some(uri) = style_uri_for_style_specifier(
            document.uri.as_str(),
            document.workspace_folder_uri.as_deref(),
            forward_source.as_str(),
        ) {
            uris.push(uri.clone());
            if let Some(target_document) = state.document(uri.as_str()) {
                uris.extend(sass_forward_module_target_uris(
                    target_document,
                    &mut BTreeSet::new(),
                ));
            }
        }
    }
    uris.sort();
    uris.dedup();
    uris
}

fn sass_symbol_declarations_with_forwards(
    state: &LspShellState,
    document: &LspTextDocumentState,
    symbol_kind: &str,
    candidate: &LspStyleHoverCandidate,
    visited: &mut BTreeSet<String>,
) -> Vec<(String, LspStyleHoverCandidate)> {
    if !visited.insert(document.uri.clone()) {
        return Vec::new();
    }
    let mut definitions = sass_symbol_declarations_in_document(document, symbol_kind, candidate);
    let Some(sheet) = parse_style_module(document.uri.as_str(), document.text.as_str()) else {
        return definitions;
    };
    let index = summarize_css_modules_intermediate(&sheet);
    for forward_source in index.sass.module_forward_sources {
        if forward_source.starts_with("sass:") {
            continue;
        }
        let Some(uri) = style_uri_for_style_specifier(
            document.uri.as_str(),
            document.workspace_folder_uri.as_deref(),
            forward_source.as_str(),
        ) else {
            continue;
        };
        let Some(target_document) = state.document(uri.as_str()) else {
            continue;
        };
        definitions.extend(sass_symbol_declarations_with_forwards(
            state,
            target_document,
            symbol_kind,
            candidate,
            visited,
        ));
    }
    definitions
}

fn sass_forward_module_target_uris(
    document: &LspTextDocumentState,
    visited: &mut BTreeSet<String>,
) -> Vec<String> {
    if !visited.insert(document.uri.clone()) {
        return Vec::new();
    }
    let Some(sheet) = parse_style_module(document.uri.as_str(), document.text.as_str()) else {
        return Vec::new();
    };
    let index = summarize_css_modules_intermediate(&sheet);
    let mut uris = Vec::new();
    for forward_source in index.sass.module_forward_sources {
        if forward_source.starts_with("sass:") {
            continue;
        }
        if let Some(uri) = style_uri_for_style_specifier(
            document.uri.as_str(),
            document.workspace_folder_uri.as_deref(),
            forward_source.as_str(),
        ) {
            uris.push(uri.clone());
        }
    }
    uris.sort();
    uris.dedup();
    uris
}

fn sass_symbol_reference_matches(
    candidate: &LspStyleHoverCandidate,
    target: &LspStyleHoverCandidate,
) -> bool {
    is_sass_symbol_reference_kind(target.kind) && sass_symbol_target_matches(candidate, target)
}

fn sass_symbol_target_matches(
    candidate: &LspStyleHoverCandidate,
    target: &LspStyleHoverCandidate,
) -> bool {
    candidate.name == target.name
        && candidate.namespace == target.namespace
        && sass_symbol_kind_from_candidate_kind(candidate.kind)
            == sass_symbol_kind_from_candidate_kind(target.kind)
}

fn render_sass_symbol_label(candidate: &LspStyleHoverCandidate) -> String {
    let namespace_prefix = candidate
        .namespace
        .as_deref()
        .map(|namespace| format!("{namespace}."))
        .unwrap_or_default();
    match sass_symbol_kind_from_candidate_kind(candidate.kind) {
        Some("variable") => format!("{namespace_prefix}${}", candidate.name),
        Some("mixin") if is_sass_symbol_declaration_kind(candidate.kind) => {
            format!("@mixin {}", candidate.name)
        }
        Some("mixin") => format!("@include {namespace_prefix}{}", candidate.name),
        Some("function") => format!("{namespace_prefix}{}()", candidate.name),
        _ => candidate.name.clone(),
    }
}

fn reference_lens_title(count: usize) -> String {
    if count == 1 {
        "1 reference".to_string()
    } else {
        format!("{count} references")
    }
}

fn resolve_lsp_prepare_rename(state: &LspShellState, params: Option<&Value>) -> Value {
    if let Some((_, candidate)) = source_selector_candidate_for_params(state, params) {
        return json!({
            "range": candidate.range,
            "placeholder": candidate.name,
        });
    }

    let Some((_, candidate, _)) = style_candidates_for_params(state, params) else {
        return Value::Null;
    };

    json!({
        "range": candidate.range,
        "placeholder": rename_placeholder(&candidate),
    })
}

fn resolve_lsp_rename(state: &LspShellState, params: Option<&Value>) -> Value {
    let Some(new_name) = params
        .and_then(|value| value.get("newName"))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
    else {
        return Value::Null;
    };
    if let Some((document_uri, candidate)) = source_selector_candidate_for_params(state, params) {
        let workspace_folder_uri = state
            .document(document_uri.as_str())
            .and_then(|document| document.workspace_folder_uri.as_deref());
        return resolve_selector_rename(
            state,
            workspace_folder_uri,
            candidate.target_style_uri.as_deref(),
            candidate.name.as_str(),
            new_name,
        );
    }

    let Some((document_uri, candidate, candidates)) = style_candidates_for_params(state, params)
    else {
        return Value::Null;
    };

    if candidate.kind == "selector" {
        let workspace_folder_uri = state
            .document(document_uri.as_str())
            .and_then(|document| document.workspace_folder_uri.as_deref());
        return resolve_selector_rename(
            state,
            workspace_folder_uri,
            Some(document_uri.as_str()),
            candidate.name.as_str(),
            new_name,
        );
    }

    let replacement = match candidate.kind {
        "customPropertyReference" | "customPropertyDeclaration" => new_name.to_string(),
        _ => return Value::Null,
    };
    let mut targets: Vec<&LspStyleHoverCandidate> = candidates
        .iter()
        .filter(|target| rename_target_matches(&candidate, target))
        .collect();
    targets.sort_by_key(|target| {
        (
            target.range.start.line,
            target.range.start.character,
            target.range.end.line,
            target.range.end.character,
        )
    });
    let edits: Vec<Value> = targets
        .into_iter()
        .map(|target| {
            json!({
                "range": target.range,
                "newText": replacement,
            })
        })
        .collect();
    if edits.is_empty() {
        return Value::Null;
    }

    let mut changes = serde_json::Map::new();
    changes.insert(document_uri, json!(edits));
    json!({
        "changes": Value::Object(changes),
    })
}

fn style_candidates_for_params(
    state: &LspShellState,
    params: Option<&Value>,
) -> Option<(String, LspStyleHoverCandidate, Vec<LspStyleHoverCandidate>)> {
    let document_uri = document_uri_from_params(params);
    let position = lsp_position_from_params(params)?;
    let document = state.document(document_uri.as_str())?;
    let (_, candidates) = style_hover_candidates_for_document(document)?;
    let candidate = candidates
        .iter()
        .find(|candidate| parser_range_contains_position(&candidate.range, position))?
        .clone();
    Some((document_uri, candidate, candidates))
}

fn rename_placeholder(candidate: &LspStyleHoverCandidate) -> &str {
    candidate.name.as_str()
}

fn rename_target_matches(
    candidate: &LspStyleHoverCandidate,
    target: &LspStyleHoverCandidate,
) -> bool {
    match candidate.kind {
        "selector" => target.kind == "selector" && target.name == candidate.name,
        "customPropertyReference" | "customPropertyDeclaration" => {
            target.name == candidate.name && target.kind.starts_with("customProperty")
        }
        kind if is_sass_symbol_candidate_kind(kind) => {
            sass_symbol_target_matches(candidate, target)
        }
        _ => false,
    }
}

fn resolve_lsp_hover(state: &LspShellState, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    if let Some(document) = state.document(document_uri.as_str())
        && !is_style_document_uri(document.uri.as_str())
    {
        return resolve_source_lsp_hover(state, document, params);
    }

    let candidates = resolve_style_hover_candidates(state, params);
    let Some(candidate) = candidates.candidates.first() else {
        return Value::Null;
    };
    let Some(document) = state.document(document_uri.as_str()) else {
        return Value::Null;
    };
    if is_sass_symbol_reference_kind(candidate.kind)
        && let Some((target_uri, target)) =
            sass_symbol_definitions_for_candidate(state, document, candidate)
                .into_iter()
                .next()
        && let Some(target_text) = style_text_for_uri(state, target_uri.as_str())
    {
        return json!({
            "contents": {
                "kind": "markdown",
                "value": render_style_hover_candidate_markdown(
                    target_uri.as_str(),
                    target_text.as_str(),
                    &target,
                ),
            },
            "range": candidate.range,
        });
    }

    json!({
        "contents": {
            "kind": "markdown",
            "value": render_style_hover_candidate_markdown(
                document.uri.as_str(),
                document.text.as_str(),
                candidate,
            ),
        },
        "range": candidate.range,
    })
}

fn resolve_source_lsp_hover(
    state: &LspShellState,
    document: &LspTextDocumentState,
    params: Option<&Value>,
) -> Value {
    let Some(position) = lsp_position_from_params(params) else {
        return Value::Null;
    };
    let candidates = source_selector_candidates_at_position(state, document, position);
    let Some(candidate) = candidates.first() else {
        return Value::Null;
    };
    let definitions = style_selector_definitions_for_source_candidates(
        state,
        candidates.as_slice(),
        document.workspace_folder_uri.as_deref(),
    );
    let value = render_source_hover_definitions_markdown(state, definitions.as_slice())
        .unwrap_or_else(|| format!("**`.{}`**", candidate.name));

    json!({
        "contents": {
            "kind": "markdown",
            "value": value,
        },
        "range": candidate.range,
    })
}

fn resolve_source_lsp_definition(
    state: &LspShellState,
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Value {
    let candidates = source_selector_candidates_at_position(state, document, position);
    if candidates.is_empty() {
        return Value::Null;
    };
    let definitions = style_selector_definitions_for_source_candidates(
        state,
        candidates.as_slice(),
        document.workspace_folder_uri.as_deref(),
    );
    if definitions.is_empty() {
        return Value::Null;
    }

    json!(
        definitions
            .into_iter()
            .map(|(uri, definition)| json!({ "uri": uri, "range": definition.range }))
            .collect::<Vec<_>>()
    )
}

fn resolve_source_lsp_references(
    state: &LspShellState,
    document: &LspTextDocumentState,
    position: ParserPositionV0,
    params: Option<&Value>,
) -> Value {
    let candidates = source_selector_candidates_at_position(state, document, position);
    if candidates.is_empty() {
        return Value::Null;
    };
    let include_declaration = include_declaration_from_params(params);
    let mut locations = Vec::new();
    if include_declaration {
        locations.extend(
            style_selector_definitions_for_source_candidates(
                state,
                candidates.as_slice(),
                document.workspace_folder_uri.as_deref(),
            )
            .into_iter()
            .map(|(uri, definition)| json!({ "uri": uri, "range": definition.range })),
        );
    }
    for candidate in candidates {
        if candidate.kind == "sourceSelectorPrefixReference" {
            let definitions = style_selector_definitions_from_open_documents(
                state,
                "",
                document.workspace_folder_uri.as_deref(),
            );
            for selector_name in source_candidate_selector_names(
                &candidate,
                definitions.as_slice(),
                candidate.target_style_uri.as_deref(),
            ) {
                locations.extend(selector_reference_locations_from_open_documents(
                    state,
                    selector_name.as_str(),
                    document.workspace_folder_uri.as_deref(),
                    candidate.target_style_uri.as_deref(),
                ));
            }
        } else {
            locations.extend(selector_reference_locations_from_open_documents(
                state,
                candidate.name.as_str(),
                document.workspace_folder_uri.as_deref(),
                candidate.target_style_uri.as_deref(),
            ));
        }
    }
    locations.sort_by_key(location_sort_key);
    locations.dedup();

    if locations.is_empty() {
        Value::Null
    } else {
        json!(locations)
    }
}

fn resolve_source_lsp_completion(
    state: &LspShellState,
    document: &LspTextDocumentState,
    params: Option<&Value>,
) -> Value {
    let Some(position) = lsp_position_from_params(params) else {
        return Value::Null;
    };
    let Some((target_style_uri, value_prefix)) =
        source_completion_context_at_position(document, position)
    else {
        return Value::Null;
    };

    let labels: BTreeSet<String> = style_selector_definitions_from_open_documents(
        state,
        "",
        document.workspace_folder_uri.as_deref(),
    )
    .into_iter()
    .filter(|(uri, _)| {
        target_style_uri
            .as_deref()
            .is_none_or(|target_uri| target_uri == uri)
    })
    .map(|(_, definition)| definition.name)
    .filter(|label| {
        value_prefix
            .as_deref()
            .is_none_or(|prefix| label.starts_with(prefix))
    })
    .collect();
    let items: Vec<Value> = labels
        .into_iter()
        .map(|label| {
            json!({
                "label": label,
                "kind": 10,
                "detail": "CSS Module selector",
                "data": {
                    "source": "openedStyleDocumentIndex",
                },
            })
        })
        .collect();

    json!({
        "isIncomplete": false,
        "items": items,
    })
}

fn source_completion_context_at_position(
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Option<(Option<String>, Option<String>)> {
    let offset = byte_offset_for_parser_position(document.text.as_str(), position)?;
    if let Some(access) = document
        .source_syntax_index
        .style_property_accesses
        .iter()
        .find(|access| offset >= access.byte_span.start && offset <= access.byte_span.end)
    {
        return Some((
            access.target_style_uri.clone(),
            source_completion_prefix_for_terminal_offset(
                document.text.as_str(),
                access.byte_span,
                offset,
            ),
        ));
    }
    if let Some(reference) = document
        .source_syntax_index
        .selector_references
        .iter()
        .find(|reference| offset >= reference.byte_span.start && offset <= reference.byte_span.end)
    {
        return Some((
            reference.target_style_uri.clone(),
            source_completion_prefix_for_terminal_offset(
                document.text.as_str(),
                reference.byte_span,
                offset,
            ),
        ));
    }
    if document
        .source_syntax_index
        .class_string_literals
        .iter()
        .any(|span| offset >= span.start && offset <= span.end)
    {
        let span = document
            .source_syntax_index
            .class_string_literals
            .iter()
            .find(|span| offset >= span.start && offset <= span.end)
            .copied()?;
        return Some((
            None,
            source_completion_class_token_prefix_from_span(document.text.as_str(), span, offset),
        ));
    }
    None
}

fn source_completion_prefix_for_terminal_offset(
    source: &str,
    span: ParserByteSpanV0,
    offset: usize,
) -> Option<String> {
    (offset >= span.end).then(|| source_completion_prefix_from_span(source, span, offset))?
}

fn source_completion_prefix_from_span(
    source: &str,
    span: ParserByteSpanV0,
    offset: usize,
) -> Option<String> {
    let end = offset.min(span.end);
    if end < span.start {
        return None;
    }
    let prefix = source.get(span.start..end)?;
    if prefix.is_empty() {
        return None;
    }
    if prefix.chars().all(is_css_identifier_continue) {
        Some(prefix.to_string())
    } else {
        None
    }
}

fn source_completion_class_token_prefix_from_span(
    source: &str,
    span: ParserByteSpanV0,
    offset: usize,
) -> Option<String> {
    let end = offset.min(span.end);
    if end < span.start {
        return None;
    }
    let prefix = source.get(span.start..end)?;
    let token = prefix
        .rsplit(|ch: char| ch.is_ascii_whitespace())
        .next()
        .unwrap_or_default();
    if token.is_empty() {
        return None;
    }
    if token.chars().all(is_css_identifier_continue) {
        Some(token.to_string())
    } else {
        None
    }
}

fn source_selector_candidate_for_params(
    state: &LspShellState,
    params: Option<&Value>,
) -> Option<(String, LspStyleHoverCandidate)> {
    let document_uri = document_uri_from_params(params);
    let position = lsp_position_from_params(params)?;
    let document = state.document(document_uri.as_str())?;
    if is_style_document_uri(document.uri.as_str()) {
        return None;
    }
    source_selector_candidate_at_position(state, document, position)
        .map(|candidate| (document_uri, candidate))
}

fn source_selector_candidate_at_position(
    state: &LspShellState,
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Option<LspStyleHoverCandidate> {
    source_selector_candidates_at_position(state, document, position)
        .into_iter()
        .next()
}

fn source_selector_candidates_at_position(
    state: &LspShellState,
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Vec<LspStyleHoverCandidate> {
    collect_source_selector_reference_candidates(state, document)
        .into_iter()
        .filter(|candidate| parser_range_contains_position(&candidate.range, position))
        .collect()
}

fn collect_source_selector_reference_candidates(
    state: &LspShellState,
    document: &LspTextDocumentState,
) -> Vec<LspStyleHoverCandidate> {
    resolve_source_provider_candidates(state, document).matched
}

fn resolve_source_provider_candidates(
    state: &LspShellState,
    document: &LspTextDocumentState,
) -> SourceProviderCandidateResolution {
    let source_candidates = collect_source_class_reference_candidates(document);
    let mut definitions = style_selector_definitions_from_open_documents(
        state,
        "",
        document.workspace_folder_uri.as_deref(),
    );
    for candidate in &source_candidates {
        if let Some(target_uri) = candidate.target_style_uri.as_deref()
            && !definitions.iter().any(|(uri, _)| uri == target_uri)
        {
            definitions.extend(style_selector_definitions_from_uri(state, target_uri));
        }
    }
    let selector_names: BTreeSet<String> = definitions
        .iter()
        .map(|(_, definition)| definition.name.clone())
        .collect();
    if selector_names.is_empty() {
        return SourceProviderCandidateResolution {
            matched: Vec::new(),
            unresolved: Vec::new(),
        };
    }
    let (mut matched, mut unresolved): (Vec<_>, Vec<_>) =
        source_candidates.into_iter().partition(|candidate| {
            source_candidate_has_style_definition(candidate, definitions.as_slice())
        });
    matched.sort();
    unresolved.sort();
    SourceProviderCandidateResolution {
        matched,
        unresolved,
    }
}

fn source_candidate_has_style_definition(
    candidate: &LspStyleHoverCandidate,
    definitions: &[(String, LspStyleHoverCandidate)],
) -> bool {
    definitions.iter().any(|(uri, definition)| {
        source_candidate_matches_selector(candidate, definition.name.as_str())
            && candidate
                .target_style_uri
                .as_deref()
                .is_none_or(|target_uri| target_uri == uri)
    })
}

fn collect_source_class_reference_candidates(
    document: &LspTextDocumentState,
) -> Vec<LspStyleHoverCandidate> {
    document.source_selector_candidates.clone()
}

fn source_selector_candidates_from_index(
    document: &LspTextDocumentState,
    index: &SourceSyntaxIndex,
) -> Vec<LspStyleHoverCandidate> {
    let mut candidates: Vec<LspStyleHoverCandidate> = index
        .selector_references
        .iter()
        .map(|reference| source_reference_candidate(document, reference))
        .collect();
    candidates.sort();
    candidates.dedup();
    candidates
}

fn build_source_syntax_index(document: &LspTextDocumentState) -> SourceSyntaxIndex {
    if is_style_document_uri(document.uri.as_str()) {
        return SourceSyntaxIndex::default();
    }

    let imports = collect_source_imports(document);
    summarize_omena_bridge_source_syntax_index(
        document.text.as_str(),
        imports.imported_style_bindings,
        imports.classnames_bind_bindings,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceImportIndex {
    imported_style_bindings: Vec<ImportedStyleBinding>,
    classnames_bind_bindings: Vec<String>,
}

fn collect_source_imports(document: &LspTextDocumentState) -> SourceImportIndex {
    let source = document.text.as_str();
    let mut imports = SourceImportIndex {
        imported_style_bindings: Vec::new(),
        classnames_bind_bindings: Vec::new(),
    };
    let summary = omena_bridge::summarize_omena_bridge_source_import_declarations(source);
    for import in summary.imports {
        if import.specifier == "classnames/bind" {
            imports.classnames_bind_bindings.push(import.binding);
        } else if StyleLanguage::from_module_path(import.specifier.as_str()).is_some()
            && let Some(style_uri) =
                style_uri_for_import_specifier(document, import.specifier.as_str())
        {
            imports.imported_style_bindings.push(ImportedStyleBinding {
                binding: import.binding,
                style_uri,
            });
        }
    }
    imports
        .imported_style_bindings
        .sort_by(|left, right| left.binding.cmp(&right.binding));
    imports
        .imported_style_bindings
        .dedup_by(|left, right| left.binding == right.binding && left.style_uri == right.style_uri);
    imports.classnames_bind_bindings.sort();
    imports.classnames_bind_bindings.dedup();
    imports
}

fn style_uri_for_import_specifier(
    document: &LspTextDocumentState,
    specifier: &str,
) -> Option<String> {
    style_uri_for_style_specifier(
        document.uri.as_str(),
        document.workspace_folder_uri.as_deref(),
        specifier,
    )
}

fn style_uri_for_style_specifier(
    source_uri: &str,
    workspace_folder_uri: Option<&str>,
    specifier: &str,
) -> Option<String> {
    if specifier.starts_with('.') {
        let source_path = file_uri_to_path(source_uri)?;
        let imported_path = normalize_path(source_path.parent()?.join(specifier));
        return style_uri_for_style_candidate_base(imported_path.as_path());
    }

    style_uri_for_tsconfig_path_alias(workspace_folder_uri, specifier)
}

fn style_uri_for_tsconfig_path_alias(
    workspace_folder_uri: Option<&str>,
    specifier: &str,
) -> Option<String> {
    let workspace_path = file_uri_to_path(workspace_folder_uri?)?;
    for config_path in [
        workspace_path.join("tsconfig.json"),
        workspace_path.join("jsconfig.json"),
    ] {
        if let Some(style_uri) =
            style_uri_for_tsconfig_path_alias_config(config_path.as_path(), specifier)
        {
            return Some(style_uri);
        }
    }
    None
}

fn style_uri_for_tsconfig_path_alias_config(config_path: &Path, specifier: &str) -> Option<String> {
    let config_text = fs::read_to_string(config_path).ok()?;
    let config = serde_json::from_str::<Value>(config_text.as_str()).ok()?;
    let compiler_options = config.get("compilerOptions")?;
    let paths = compiler_options.get("paths")?.as_object()?;
    let config_dir = config_path.parent()?;
    let base_url = compiler_options
        .get("baseUrl")
        .and_then(Value::as_str)
        .unwrap_or(".");
    let base_path = normalize_path(config_dir.join(base_url));
    let mut candidates = Vec::new();

    for (pattern, targets) in paths {
        let Some((capture, score)) = tsconfig_path_pattern_match(pattern.as_str(), specifier)
        else {
            continue;
        };
        let Some(targets) = targets.as_array() else {
            continue;
        };
        for target in targets.iter().filter_map(Value::as_str) {
            let candidate_path =
                tsconfig_path_target_candidate(base_path.as_path(), target, capture.as_deref());
            for resolved_path in style_candidate_paths(candidate_path.as_path()) {
                candidates.push((score, resolved_path.exists(), resolved_path));
            }
        }
    }

    candidates.sort_by(|left, right| {
        right
            .1
            .cmp(&left.1)
            .then_with(|| right.0.cmp(&left.0))
            .then_with(|| left.2.cmp(&right.2))
    });
    candidates
        .into_iter()
        .map(|(_, _, path)| path_to_file_uri(path.as_path()))
        .next()
}

fn tsconfig_path_pattern_match(pattern: &str, specifier: &str) -> Option<(Option<String>, usize)> {
    let Some(star_index) = pattern.find('*') else {
        return (pattern == specifier).then_some((None, pattern.len()));
    };
    if pattern[star_index + 1..].contains('*') {
        return None;
    }
    let prefix = &pattern[..star_index];
    let suffix = &pattern[star_index + 1..];
    if !specifier.starts_with(prefix) || !specifier.ends_with(suffix) {
        return None;
    }
    let capture_end = specifier.len().saturating_sub(suffix.len());
    if capture_end < prefix.len() {
        return None;
    }
    Some((
        specifier.get(prefix.len()..capture_end).map(str::to_string),
        prefix.len() + suffix.len(),
    ))
}

fn tsconfig_path_target_candidate(
    base_path: &Path,
    target_pattern: &str,
    capture: Option<&str>,
) -> PathBuf {
    let target = if target_pattern.contains('*') {
        target_pattern.replace('*', capture.unwrap_or_default())
    } else {
        target_pattern.to_string()
    };
    let target_path = PathBuf::from(target);
    if target_path.is_absolute() {
        normalize_path(target_path)
    } else {
        normalize_path(base_path.join(target_path))
    }
}

fn style_uri_for_style_candidate_base(base_path: &Path) -> Option<String> {
    let candidates = style_candidate_paths(base_path);
    candidates
        .iter()
        .find(|path| path.exists())
        .or_else(|| candidates.first())
        .map(|path| path_to_file_uri(path.as_path()))
}

fn style_candidate_paths(base_path: &Path) -> Vec<PathBuf> {
    let normalized = normalize_path(base_path.to_path_buf());
    if is_indexable_style_path(normalized.as_path()) {
        return vec![normalized];
    }

    let mut candidates = Vec::new();
    for extension in ["scss", "sass", "css"] {
        candidates.push(normalized.with_extension(extension));
        if let Some(file_name) = normalized.file_name().and_then(|value| value.to_str()) {
            candidates.push(
                normalized
                    .with_file_name(format!("_{file_name}"))
                    .with_extension(extension),
            );
        }
        candidates.push(normalized.join(format!("index.{extension}")));
        candidates.push(normalized.join(format!("_index.{extension}")));
    }
    candidates.sort();
    candidates.dedup();
    candidates
        .into_iter()
        .filter(|path| is_indexable_style_path(path.as_path()))
        .collect()
}

fn char_boundary_floor(source: &str, index: usize) -> usize {
    let mut index = index.min(source.len());
    while index > 0 && !source.is_char_boundary(index) {
        index -= 1;
    }
    index
}

fn char_boundary_ceil(source: &str, index: usize) -> usize {
    let mut index = index.min(source.len());
    while index < source.len() && !source.is_char_boundary(index) {
        index += 1;
    }
    index
}

fn advance_js_scan_cursor(source: &str, cursor: usize, limit: usize) -> usize {
    let cursor = char_boundary_ceil(source, cursor);
    let limit = char_boundary_floor(source, limit);
    if cursor >= limit {
        return limit;
    }
    char_boundary_ceil(source, cursor + 1).min(limit)
}

fn advance_js_escaped_char(source: &str, slash_offset: usize, limit: usize) -> usize {
    let after_slash = advance_js_scan_cursor(source, slash_offset, limit);
    advance_js_scan_cursor(source, after_slash, limit)
}

fn matching_js_block_end(source: &str, open_offset: usize, open: u8, close: u8) -> Option<usize> {
    if source.as_bytes().get(open_offset) != Some(&open) {
        return None;
    }
    let mut cursor = advance_js_scan_cursor(source, open_offset, source.len());
    let mut depth = 1usize;
    while cursor < source.len() {
        match source.as_bytes().get(cursor).copied()? {
            b'\'' | b'"' | b'`' => {
                cursor = skip_js_string_literal(source, cursor, source.len())?;
            }
            byte if byte == open => {
                depth += 1;
                cursor = advance_js_scan_cursor(source, cursor, source.len());
            }
            byte if byte == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(cursor);
                }
                cursor = advance_js_scan_cursor(source, cursor, source.len());
            }
            _ => cursor = advance_js_scan_cursor(source, cursor, source.len()),
        }
    }
    None
}

fn split_top_level_js_segments(
    source: &str,
    start: usize,
    end: usize,
    delimiter: u8,
) -> Vec<(usize, usize)> {
    let mut segments = Vec::new();
    let end = char_boundary_floor(source, end);
    let mut segment_start = char_boundary_ceil(source, start).min(end);
    let mut cursor = segment_start;
    let mut depth = 0usize;
    while cursor < end {
        match source.as_bytes().get(cursor).copied() {
            Some(b'\'' | b'"' | b'`') => {
                cursor = skip_js_string_literal(source, cursor, end).unwrap_or(end);
            }
            Some(b'(' | b'[' | b'{') => {
                depth += 1;
                cursor = advance_js_scan_cursor(source, cursor, end);
            }
            Some(b')' | b']' | b'}') => {
                depth = depth.saturating_sub(1);
                cursor = advance_js_scan_cursor(source, cursor, end);
            }
            Some(byte) if byte == delimiter && depth == 0 => {
                segments.push((segment_start, cursor));
                cursor = advance_js_scan_cursor(source, cursor, end);
                segment_start = cursor;
            }
            Some(_) => cursor = advance_js_scan_cursor(source, cursor, end),
            None => break,
        }
    }
    if segment_start <= end {
        segments.push((segment_start, end));
    }
    segments
}

fn source_reference_candidate(
    document: &LspTextDocumentState,
    reference: &SourceSelectorReferenceFact,
) -> LspStyleHoverCandidate {
    let name = reference.selector_name.clone().unwrap_or_else(|| {
        document.text[reference.byte_span.start..reference.byte_span.end].to_string()
    });
    LspStyleHoverCandidate {
        kind: match reference.match_kind {
            SourceSelectorReferenceMatchKind::Exact => "sourceSelectorReference",
            SourceSelectorReferenceMatchKind::Prefix => "sourceSelectorPrefixReference",
        },
        name,
        range: parser_range_for_byte_span(document.text.as_str(), reference.byte_span),
        source: "omenaBridgeSourceSyntaxIndex",
        target_style_uri: reference.target_style_uri.clone(),
        namespace: None,
    }
}

fn skip_ascii_whitespace(source: &str, mut offset: usize) -> usize {
    while source
        .as_bytes()
        .get(offset)
        .is_some_and(u8::is_ascii_whitespace)
    {
        offset += 1;
    }
    offset
}

fn skip_js_string_literal(source: &str, quote_offset: usize, limit: usize) -> Option<usize> {
    let quote = source.as_bytes().get(quote_offset).copied()?;
    let limit = char_boundary_floor(source, limit);
    let mut cursor = quote_offset + 1;
    while cursor < limit {
        let byte = source.as_bytes().get(cursor).copied()?;
        if byte == b'\\' {
            cursor = advance_js_escaped_char(source, cursor, limit);
            continue;
        }
        if byte == quote {
            return Some(cursor + 1);
        }
        cursor = advance_js_scan_cursor(source, cursor, limit);
    }
    None
}

fn style_selector_definitions_from_open_documents(
    state: &LspShellState,
    selector_name: &str,
    workspace_folder_uri: Option<&str>,
) -> Vec<(String, LspStyleHoverCandidate)> {
    let mut definitions = Vec::new();
    for document in state.documents.values() {
        if !is_style_document_uri(document.uri.as_str())
            || !workspace_folder_compatible(workspace_folder_uri, document)
        {
            continue;
        }
        let Some((_, candidates)) = style_hover_candidates_for_document(document) else {
            continue;
        };
        definitions.extend(
            candidates
                .into_iter()
                .filter(|candidate| {
                    candidate.kind == "selector"
                        && (selector_name.is_empty() || candidate.name == selector_name)
                })
                .map(|candidate| (document.uri.clone(), candidate)),
        );
    }
    definitions.sort_by_key(|(uri, candidate)| {
        (
            uri.clone(),
            candidate.range.start.line,
            candidate.range.start.character,
        )
    });
    definitions
}

fn style_selector_definitions_from_uri(
    state: &LspShellState,
    uri: &str,
) -> Vec<(String, LspStyleHoverCandidate)> {
    style_hover_candidates_for_uri(state, uri)
        .map(|(_, candidates)| {
            candidates
                .into_iter()
                .filter(|candidate| candidate.kind == "selector")
                .map(|candidate| (uri.to_string(), candidate))
                .collect()
        })
        .unwrap_or_default()
}

fn style_selector_definitions_for_source_candidate(
    state: &LspShellState,
    candidate: &LspStyleHoverCandidate,
    workspace_folder_uri: Option<&str>,
) -> Vec<(String, LspStyleHoverCandidate)> {
    let mut definitions = style_selector_definitions_from_open_documents(
        state,
        source_candidate_definition_lookup_name(candidate),
        workspace_folder_uri,
    );
    if let Some(target_uri) = candidate.target_style_uri.as_deref()
        && !definitions.iter().any(|(uri, _)| uri == target_uri)
    {
        definitions.extend(style_selector_definitions_from_uri(state, target_uri));
    }
    definitions
        .into_iter()
        .filter(|(uri, _)| {
            candidate
                .target_style_uri
                .as_deref()
                .is_none_or(|target_uri| target_uri == uri)
        })
        .filter(|(_, definition)| {
            source_candidate_matches_selector(candidate, definition.name.as_str())
        })
        .collect()
}

fn style_selector_definitions_for_source_candidates(
    state: &LspShellState,
    candidates: &[LspStyleHoverCandidate],
    workspace_folder_uri: Option<&str>,
) -> Vec<(String, LspStyleHoverCandidate)> {
    let mut definitions = candidates
        .iter()
        .flat_map(|candidate| {
            style_selector_definitions_for_source_candidate(state, candidate, workspace_folder_uri)
        })
        .collect::<Vec<_>>();
    definitions.sort_by_key(|(uri, definition)| {
        (
            uri.clone(),
            definition.range.start.line,
            definition.range.start.character,
            definition.name.clone(),
        )
    });
    definitions.dedup_by(|left, right| {
        left.0 == right.0 && left.1.name == right.1.name && left.1.range == right.1.range
    });
    definitions
}

fn render_source_hover_definitions_markdown(
    state: &LspShellState,
    definitions: &[(String, LspStyleHoverCandidate)],
) -> Option<String> {
    let parts = definitions
        .iter()
        .filter_map(|(uri, definition)| {
            style_text_for_uri(state, uri).map(|text| {
                render_style_hover_candidate_markdown(uri.as_str(), text.as_str(), definition)
            })
        })
        .collect::<Vec<_>>();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n\n---\n\n"))
    }
}

fn source_candidate_definition_lookup_name(candidate: &LspStyleHoverCandidate) -> &str {
    if candidate.kind == "sourceSelectorPrefixReference" {
        ""
    } else {
        candidate.name.as_str()
    }
}

fn source_candidate_matches_selector(
    candidate: &LspStyleHoverCandidate,
    selector_name: &str,
) -> bool {
    if candidate.kind == "sourceSelectorPrefixReference" {
        selector_name.starts_with(candidate.name.as_str())
    } else {
        selector_name == candidate.name
    }
}

fn first_style_document_for_workspace<'a>(
    state: &'a LspShellState,
    workspace_folder_uri: Option<&str>,
) -> Option<(String, &'a LspTextDocumentState)> {
    state
        .documents
        .values()
        .filter(|document| is_style_document_uri(document.uri.as_str()))
        .filter(|document| workspace_folder_compatible(workspace_folder_uri, document))
        .map(|document| (document.uri.clone(), document))
        .next()
}

fn resolve_selector_rename(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    target_style_uri: Option<&str>,
    selector_name: &str,
    new_name: &str,
) -> Value {
    let replacement = new_name.trim_start_matches('.');
    if replacement.is_empty() {
        return Value::Null;
    }

    let mut changes: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for (uri, definition) in
        style_selector_definitions_from_open_documents(state, selector_name, workspace_folder_uri)
            .into_iter()
            .filter(|(uri, _)| target_style_uri.is_none_or(|target_uri| target_uri == uri))
    {
        changes.entry(uri).or_default().push(json!({
            "range": definition.range,
            "newText": replacement,
        }));
    }
    for document in state.documents.values() {
        if is_style_document_uri(document.uri.as_str()) {
            continue;
        }
        if !workspace_folder_compatible(workspace_folder_uri, document) {
            continue;
        }
        for candidate in collect_source_selector_reference_candidates(state, document)
            .into_iter()
            .filter(|candidate| candidate.name == selector_name)
            .filter(|candidate| source_candidate_matches_target_style(candidate, target_style_uri))
        {
            changes
                .entry(document.uri.clone())
                .or_default()
                .push(json!({
                    "range": candidate.range,
                    "newText": replacement,
                }));
        }
    }

    if changes.is_empty() {
        return Value::Null;
    }
    for edits in changes.values_mut() {
        edits.sort_by_key(|edit| {
            let line = edit
                .pointer("/range/start/line")
                .and_then(Value::as_u64)
                .unwrap_or_default();
            let character = edit
                .pointer("/range/start/character")
                .and_then(Value::as_u64)
                .unwrap_or_default();
            (line, character)
        });
    }

    let mut response_changes = serde_json::Map::new();
    for (uri, edits) in changes {
        response_changes.insert(uri, json!(edits));
    }
    json!({
        "changes": Value::Object(response_changes),
    })
}

fn workspace_folder_compatible(
    workspace_folder_uri: Option<&str>,
    document: &LspTextDocumentState,
) -> bool {
    match (
        workspace_folder_uri,
        document.workspace_folder_uri.as_deref(),
    ) {
        (Some(left), Some(right)) => workspace_folder_uri_equivalent(left, right),
        _ => true,
    }
}

fn workspace_folder_uri_equivalent(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    match (file_uri_to_path(left), file_uri_to_path(right)) {
        (Some(left_path), Some(right_path)) => {
            normalize_path(left_path) == normalize_path(right_path)
        }
        _ => false,
    }
}

fn is_style_document_uri(uri: &str) -> bool {
    StyleLanguage::from_module_path(uri).is_some()
}

fn file_uri_to_path(uri: &str) -> Option<PathBuf> {
    let raw_path = uri.strip_prefix("file://")?;
    Some(PathBuf::from(percent_decode_uri_path(raw_path)?))
}

fn percent_decode_uri_path(raw_path: &str) -> Option<String> {
    let bytes = raw_path.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = bytes.get(index + 1).and_then(|byte| hex_value(*byte))?;
            let low = bytes.get(index + 2).and_then(|byte| hex_value(*byte))?;
            decoded.push((high << 4) | low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn location_sort_key(location: &Value) -> (String, u64, u64) {
    let uri = location
        .get("uri")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let line = location
        .pointer("/range/start/line")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let character = location
        .pointer("/range/start/character")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    (uri, line, character)
}

fn location_identity_key(location: &Value) -> (String, u64, u64, u64, u64) {
    let uri = location
        .get("uri")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let start_line = location
        .pointer("/range/start/line")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let start_character = location
        .pointer("/range/start/character")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let end_line = location
        .pointer("/range/end/line")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let end_character = location
        .pointer("/range/end/character")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    (uri, start_line, start_character, end_line, end_character)
}

fn lsp_range_start_sort_key(value: &Value) -> (u64, u64) {
    let line = value
        .pointer("/range/start/line")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let character = value
        .pointer("/range/start/character")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    (line, character)
}

fn render_style_hover_candidate_markdown(
    document_uri: &str,
    source: &str,
    candidate: &LspStyleHoverCandidate,
) -> String {
    let location = format!(
        "{}:{}",
        file_label_from_uri(document_uri),
        candidate.range.start.line + 1
    );
    match candidate.kind {
        "selector" => {
            let snippet = rule_snippet_around_position(source, candidate.range.start)
                .unwrap_or_else(|| format!(".{} {{ ... }}", candidate.name));
            format!(
                "**`.{}`** - _{}_\n\n```scss\n{}\n```",
                candidate.name, location, snippet
            )
        }
        "customPropertyReference" => {
            let snippet =
                line_snippet_at_position(source, candidate.range.start).unwrap_or_default();
            format!(
                "**`var({})`** - _{}_\n\n```scss\n{}\n```",
                candidate.name, location, snippet
            )
        }
        "customPropertyDeclaration" => {
            let snippet =
                line_snippet_at_position(source, candidate.range.start).unwrap_or_default();
            format!(
                "**`{}`** - _{}_\n\n```scss\n{}\n```",
                candidate.name, location, snippet
            )
        }
        kind if is_sass_symbol_candidate_kind(kind) => {
            render_sass_symbol_hover_markdown(source, candidate, location.as_str())
        }
        _ => candidate.name.clone(),
    }
}

fn render_sass_symbol_hover_markdown(
    source: &str,
    candidate: &LspStyleHoverCandidate,
    location: &str,
) -> String {
    let label = render_sass_symbol_label(candidate);
    match sass_symbol_kind_from_candidate_kind(candidate.kind) {
        Some("variable") if is_sass_symbol_declaration_kind(candidate.kind) => {
            let snippet =
                line_snippet_at_position(source, candidate.range.start).unwrap_or_default();
            if let Some(value) = sass_variable_value_from_declaration_line(snippet.as_str()) {
                return format!(
                    "**`{}`** - _{}_\n\nValue: `{}`\n\n```scss\n{}\n```",
                    label, location, value, snippet
                );
            }
            format!(
                "**`{}`** - _{}_\n\n```scss\n{}\n```",
                label, location, snippet
            )
        }
        Some("mixin" | "function") if is_sass_symbol_declaration_kind(candidate.kind) => {
            let rendered = sass_callable_definition_render_parts(source, candidate.range.start)
                .or_else(|| {
                    line_snippet_at_position(source, candidate.range.start)
                        .map(|line| (label.clone(), line))
                })
                .unwrap_or_else(|| (label.clone(), String::new()));
            format!(
                "**`{}`** - _{}_\n\n```scss\n{}\n```",
                rendered.0, location, rendered.1
            )
        }
        _ => {
            let snippet =
                line_snippet_at_position(source, candidate.range.start).unwrap_or_default();
            format!(
                "**`{}`** - _{}_\n\n```scss\n{}\n```",
                label, location, snippet
            )
        }
    }
}

fn sass_variable_value_from_declaration_line(line: &str) -> Option<String> {
    let (_, value) = line.split_once(':')?;
    let value = value
        .trim()
        .trim_end_matches(';')
        .trim()
        .trim_end_matches("!default")
        .trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn sass_callable_definition_render_parts(
    source: &str,
    position: ParserPositionV0,
) -> Option<(String, String)> {
    let line_start = byte_offset_for_parser_position(
        source,
        ParserPositionV0 {
            line: position.line,
            character: 0,
        },
    )?;
    let open_brace = source[line_start..].find('{')? + line_start;
    let close_brace = matching_js_block_end(source, open_brace, b'{', b'}')?;
    let signature = source[line_start..open_brace].trim().to_string();
    let body = source[open_brace + 1..close_brace].trim();
    if signature.is_empty() || body.is_empty() {
        return None;
    }
    Some((signature, trim_hover_snippet(body)))
}

fn rule_snippet_around_position(source: &str, position: ParserPositionV0) -> Option<String> {
    let line_start = byte_offset_for_parser_position(
        source,
        ParserPositionV0 {
            line: position.line,
            character: 0,
        },
    )?;
    let open_brace = source[line_start..].find('{')? + line_start;
    let mut depth = 0usize;
    let mut cursor = open_brace;
    while cursor < source.len() {
        match source.as_bytes().get(cursor).copied()? {
            b'{' => depth += 1,
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let snippet = source[line_start..=cursor].trim();
                    return Some(trim_hover_snippet(snippet));
                }
            }
            _ => {}
        }
        cursor = advance_js_scan_cursor(source, cursor, source.len());
    }
    None
}

fn line_snippet_at_position(source: &str, position: ParserPositionV0) -> Option<String> {
    let line_start = byte_offset_for_parser_position(
        source,
        ParserPositionV0 {
            line: position.line,
            character: 0,
        },
    )?;
    let line_end = source[line_start..]
        .find('\n')
        .map(|offset| line_start + offset)
        .unwrap_or(source.len());
    Some(source[line_start..line_end].trim().to_string())
}

fn trim_hover_snippet(snippet: &str) -> String {
    const MAX_SNIPPET_LEN: usize = 1200;
    if snippet.len() <= MAX_SNIPPET_LEN {
        return snippet.to_string();
    }
    format!("{}...", snippet[..MAX_SNIPPET_LEN].trim_end())
}

fn include_declaration_from_params(params: Option<&Value>) -> bool {
    params
        .and_then(|value| value.get("context"))
        .and_then(|value| value.get("includeDeclaration"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn end_of_document_range(source: &str) -> ParserRangeV0 {
    let position = parser_position_for_byte_offset(source, source.len());
    ParserRangeV0 {
        start: position,
        end: position,
    }
}

fn file_label_from_uri(uri: &str) -> &str {
    uri.rsplit('/')
        .next()
        .filter(|label| !label.is_empty())
        .unwrap_or(uri)
}

fn document_uri_from_params(params: Option<&Value>) -> String {
    params
        .and_then(|value| value.get("textDocument"))
        .and_then(|value| value.get("uri"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn empty_style_hover_candidates_result(
    document_uri: String,
    workspace_folder_uri: Option<String>,
    query_position: Option<ParserPositionV0>,
) -> LspStyleHoverCandidatesResult {
    LspStyleHoverCandidatesResult {
        schema_version: "0",
        product: "omena-lsp-server.style-hover-candidates",
        document_uri,
        workspace_folder_uri,
        language: None,
        query_position,
        candidate_count: 0,
        candidates: Vec::new(),
    }
}

fn lsp_position_from_params(params: Option<&Value>) -> Option<ParserPositionV0> {
    let position = params.and_then(|value| value.get("position"))?;
    lsp_position_from_value(position)
}

fn lsp_range_from_value(value: &Value) -> Option<ParserRangeV0> {
    Some(ParserRangeV0 {
        start: lsp_position_from_value(value.get("start")?)?,
        end: lsp_position_from_value(value.get("end")?)?,
    })
}

fn lsp_position_from_value(position: &Value) -> Option<ParserPositionV0> {
    Some(ParserPositionV0 {
        line: position.get("line").and_then(Value::as_u64)? as usize,
        character: position.get("character").and_then(Value::as_u64)? as usize,
    })
}

fn collect_style_hover_candidates(
    uri: &str,
    text: &str,
) -> Option<(&'static str, Vec<LspStyleHoverCandidate>)> {
    let sheet = parse_style_module(uri, text)?;
    let index = summarize_css_modules_intermediate(&sheet);
    let mut seen = BTreeSet::new();
    let mut candidates = Vec::new();
    collect_style_selector_hover_candidates_from_parser_facts(
        index.selectors.definition_facts.as_slice(),
        &mut seen,
        &mut candidates,
    );
    collect_custom_property_hover_candidates(
        sheet.source.as_str(),
        index.custom_properties.decl_facts.as_slice(),
        index.custom_properties.ref_names.as_slice(),
        &mut seen,
        &mut candidates,
    );
    collect_sass_symbol_hover_candidates(
        index.sass.symbol_decl_facts.as_slice(),
        index.sass.selector_symbol_facts.as_slice(),
        &mut seen,
        &mut candidates,
    );
    collect_sass_partial_evaluator_selector_candidates(
        sheet.source.as_str(),
        sheet.nodes.as_slice(),
        &mut seen,
        &mut candidates,
    );
    candidates.sort();
    Some((style_language_label(sheet.language), candidates))
}

fn collect_style_selector_hover_candidates_from_parser_facts(
    definition_facts: &[engine_style_parser::ParserIndexSelectorDefinitionFactV0],
    seen: &mut BTreeSet<(usize, usize, String)>,
    candidates: &mut Vec<LspStyleHoverCandidate>,
) {
    for fact in definition_facts {
        if seen.insert((fact.byte_span.start, fact.byte_span.end, fact.name.clone())) {
            candidates.push(LspStyleHoverCandidate {
                kind: "selector",
                name: fact.name.clone(),
                range: fact.range,
                source: "engineStyleParserSelectorDefinitionFacts",
                target_style_uri: None,
                namespace: None,
            });
        }
    }
}

fn collect_custom_property_hover_candidates(
    source: &str,
    decl_facts: &[engine_style_parser::ParserIndexCustomPropertyDeclFactV0],
    ref_names: &[String],
    seen: &mut BTreeSet<(usize, usize, String)>,
    candidates: &mut Vec<LspStyleHoverCandidate>,
) {
    for fact in decl_facts {
        if seen.insert((fact.byte_span.start, fact.byte_span.end, fact.name.clone())) {
            candidates.push(LspStyleHoverCandidate {
                kind: "customPropertyDeclaration",
                name: fact.name.clone(),
                range: fact.range,
                source: "openedStyleDocumentIndex",
                target_style_uri: None,
                namespace: None,
            });
        }
    }

    for name in ref_names {
        for byte_span in custom_property_ref_byte_spans(source, name) {
            if seen.insert((byte_span.start, byte_span.end, name.clone())) {
                candidates.push(LspStyleHoverCandidate {
                    kind: "customPropertyReference",
                    name: name.clone(),
                    range: parser_range_for_byte_span(source, byte_span),
                    source: "openedStyleDocumentIndex",
                    target_style_uri: None,
                    namespace: None,
                });
            }
        }
    }
}

fn collect_sass_symbol_hover_candidates(
    decl_facts: &[engine_style_parser::ParserIndexSassSymbolDeclFactV0],
    ref_facts: &[engine_style_parser::ParserIndexSassSelectorSymbolFactV0],
    seen: &mut BTreeSet<(usize, usize, String)>,
    candidates: &mut Vec<LspStyleHoverCandidate>,
) {
    for fact in decl_facts {
        if seen.insert((
            fact.byte_span.start,
            fact.byte_span.end,
            format!("{}:{}", fact.symbol_kind, fact.name),
        )) {
            candidates.push(LspStyleHoverCandidate {
                kind: sass_symbol_declaration_candidate_kind(fact.symbol_kind),
                name: fact.name.clone(),
                range: fact.range,
                source: "engineStyleParserSassSymbolFacts",
                target_style_uri: None,
                namespace: None,
            });
        }
    }

    for fact in ref_facts {
        if seen.insert((
            fact.byte_span.start,
            fact.byte_span.end,
            format!(
                "{}:{}:{}",
                fact.symbol_kind,
                fact.namespace.as_deref().unwrap_or_default(),
                fact.name
            ),
        )) {
            candidates.push(LspStyleHoverCandidate {
                kind: sass_symbol_reference_candidate_kind(fact.symbol_kind, fact.role),
                name: fact.name.clone(),
                range: fact.range,
                source: "engineStyleParserSassSymbolFacts",
                target_style_uri: None,
                namespace: fact.namespace.clone(),
            });
        }
    }
}

fn collect_sass_partial_evaluator_selector_candidates(
    source: &str,
    nodes: &[engine_style_parser::SyntaxNode],
    seen: &mut BTreeSet<(usize, usize, String)>,
    candidates: &mut Vec<LspStyleHoverCandidate>,
) {
    for node in nodes {
        if let Some(SyntaxNodePayload::AtRule(at_rule)) = &node.payload
            && at_rule.kind == AtRuleKind::Include
        {
            let range_span = ParserByteSpanV0 {
                start: node.header_span.unwrap_or(node.span).start,
                end: node.header_span.unwrap_or(node.span).end,
            };
            for selector_name in infer_sass_include_generated_selector_names(&at_rule.params) {
                if seen.insert((range_span.start, range_span.end, selector_name.clone())) {
                    candidates.push(LspStyleHoverCandidate {
                        kind: "selector",
                        name: selector_name,
                        range: parser_range_for_byte_span(source, range_span),
                        source: "sassPartialEvaluatorGeneratedSelectors",
                        target_style_uri: None,
                        namespace: None,
                    });
                }
            }
        }
        collect_sass_partial_evaluator_selector_candidates(
            source,
            &node.children,
            seen,
            candidates,
        );
    }
}

fn infer_sass_include_generated_selector_names(params: &str) -> Vec<String> {
    let Some(prefix) = sass_named_argument_string_value(params, "prefix") else {
        return Vec::new();
    };
    if prefix.is_empty() || !prefix.chars().all(is_css_identifier_continue) {
        return Vec::new();
    }
    let mut selectors = sass_first_map_string_keys(params)
        .into_iter()
        .filter(|key| !key.is_empty() && key.chars().all(is_css_identifier_continue))
        .map(|key| format!("{prefix}-{key}"))
        .collect::<Vec<_>>();
    selectors.sort();
    selectors.dedup();
    selectors
}

fn sass_named_argument_string_value(params: &str, name: &str) -> Option<String> {
    let needle = format!("${name}");
    let mut cursor = 0usize;
    while let Some(relative_match) = params[cursor..].find(needle.as_str()) {
        let name_start = cursor + relative_match;
        let name_end = name_start + needle.len();
        if !sass_identifier_boundary(params, name_start, name_end) {
            cursor = name_end;
            continue;
        }
        let colon_offset = skip_ascii_whitespace(params, name_end);
        if params.as_bytes().get(colon_offset) != Some(&b':') {
            cursor = name_end;
            continue;
        }
        let value_start = skip_ascii_whitespace(params, colon_offset + 1);
        return sass_string_literal_value(params, value_start).map(|(value, _)| value);
    }
    None
}

fn sass_first_map_string_keys(params: &str) -> Vec<String> {
    let mut cursor = 0usize;
    while cursor < params.len() {
        let Some(open_relative) = params[cursor..].find('(') else {
            break;
        };
        let open = cursor + open_relative;
        let Some(close) = matching_js_block_end(params, open, b'(', b')') else {
            break;
        };
        let keys = sass_map_string_keys(params, open + 1, close);
        if !keys.is_empty() {
            return keys;
        }
        cursor = open + 1;
    }
    Vec::new()
}

fn sass_map_string_keys(params: &str, start: usize, end: usize) -> Vec<String> {
    split_top_level_js_segments(params, start, end, b',')
        .into_iter()
        .filter_map(|(entry_start, entry_end)| {
            let key_start = skip_ascii_whitespace(params, entry_start);
            let (key, key_end) = sass_string_literal_value(params, key_start)?;
            let colon_offset = skip_ascii_whitespace(params, key_end);
            (colon_offset < entry_end && params.as_bytes().get(colon_offset) == Some(&b':'))
                .then_some(key)
        })
        .collect()
}

fn sass_string_literal_value(source: &str, quote_offset: usize) -> Option<(String, usize)> {
    let quote = source.as_bytes().get(quote_offset).copied()?;
    if !matches!(quote, b'\'' | b'"') {
        return None;
    }
    let literal_end = skip_js_string_literal(source, quote_offset, source.len())?;
    let value_end = literal_end.saturating_sub(1);
    source
        .get(quote_offset + 1..value_end)
        .map(|value| (value.to_string(), literal_end))
}

fn sass_identifier_boundary(source: &str, start: usize, end: usize) -> bool {
    let before = source
        .get(..start)
        .and_then(|prefix| prefix.chars().next_back())
        .is_none_or(|ch| !is_css_identifier_continue(ch) && ch != '$');
    let after = source
        .get(end..)
        .and_then(|suffix| suffix.chars().next())
        .is_none_or(|ch| !is_css_identifier_continue(ch));
    before && after
}

fn sass_symbol_declaration_candidate_kind(symbol_kind: &str) -> &'static str {
    match symbol_kind {
        "variable" => "sassVariableDeclaration",
        "mixin" => "sassMixinDeclaration",
        "function" => "sassFunctionDeclaration",
        _ => "sassSymbolDeclaration",
    }
}

fn sass_symbol_reference_candidate_kind(symbol_kind: &str, role: &str) -> &'static str {
    match (symbol_kind, role) {
        ("variable", _) => "sassVariableReference",
        ("mixin", "include") => "sassMixinInclude",
        ("function", "call") => "sassFunctionCall",
        ("mixin", _) => "sassMixinReference",
        ("function", _) => "sassFunctionReference",
        _ => "sassSymbolReference",
    }
}

fn custom_property_ref_byte_spans(source: &str, name: &str) -> Vec<ParserByteSpanV0> {
    let mut spans = Vec::new();
    let mut search_offset = 0usize;

    while let Some(relative_match) = source[search_offset..].find(name) {
        let name_start = search_offset + relative_match;
        let name_end = name_start + name.len();
        if source[..name_start].trim_end().ends_with("var(")
            && is_selector_name_boundary(source, name_end)
        {
            spans.push(ParserByteSpanV0 {
                start: name_start,
                end: name_end,
            });
        }
        search_offset += relative_match + name.len();
    }

    spans
}

fn is_selector_name_boundary(source: &str, byte_offset: usize) -> bool {
    source[byte_offset..]
        .chars()
        .next()
        .is_none_or(|ch| !is_css_identifier_continue(ch))
}

fn is_css_identifier_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_')
}

fn parser_range_for_byte_span(source: &str, span: ParserByteSpanV0) -> ParserRangeV0 {
    ParserRangeV0 {
        start: parser_position_for_byte_offset(source, span.start),
        end: parser_position_for_byte_offset(source, span.end),
    }
}

fn parser_position_for_byte_offset(source: &str, offset: usize) -> ParserPositionV0 {
    let clamped_offset = offset.min(source.len());
    let mut line = 0usize;
    let mut character = 0usize;

    for (byte_index, ch) in source.char_indices() {
        if byte_index >= clamped_offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            character = 0;
        } else {
            character += ch.len_utf16();
        }
    }

    ParserPositionV0 { line, character }
}

fn byte_offset_for_parser_position(source: &str, position: ParserPositionV0) -> Option<usize> {
    let mut line = 0usize;
    let mut character = 0usize;

    for (byte_index, ch) in source.char_indices() {
        if line == position.line && character == position.character {
            return Some(byte_index);
        }
        if ch == '\n' {
            if line == position.line {
                return None;
            }
            line += 1;
            character = 0;
        } else {
            character += ch.len_utf16();
            if line == position.line && character > position.character {
                return None;
            }
        }
    }

    if line == position.line && character == position.character {
        Some(source.len())
    } else {
        None
    }
}

fn parser_range_contains_position(range: &ParserRangeV0, position: ParserPositionV0) -> bool {
    parser_position_is_after_or_equal(position, range.start)
        && parser_position_is_before(position, range.end)
}

fn parser_position_is_after_or_equal(position: ParserPositionV0, start: ParserPositionV0) -> bool {
    position.line > start.line
        || (position.line == start.line && position.character >= start.character)
}

fn parser_position_is_before(position: ParserPositionV0, end: ParserPositionV0) -> bool {
    position.line < end.line || (position.line == end.line && position.character < end.character)
}

fn style_language_label(language: StyleLanguage) -> &'static str {
    match language {
        StyleLanguage::Css => "css",
        StyleLanguage::Scss => "scss",
        StyleLanguage::Less => "less",
    }
}

#[cfg(test)]
mod tests {
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
            vec!["quickfix"]
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
                .and_then(|value| value.pointer("/result/items/1/label")),
            Some(&json!(".root")),
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
        let target_text =
            "$defign_gray200: #eee;\n@mixin defign_typography20 { font-size: 20px; }\n";
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
            .ok_or_else(|| {
                std::io::Error::other("color prefix definition should return results")
            })?;
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
}
