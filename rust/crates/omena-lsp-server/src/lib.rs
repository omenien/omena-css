mod diagnostics_scheduler;
mod query_reuse;
mod workspace_runtime_registry;

use diagnostics_scheduler::{
    RustDiagnosticsSchedulerBoundaryV0, diagnostics_schedule_event, run_diagnostics_schedule,
    rust_diagnostics_scheduler_contract,
};
use engine_style_parser::{
    ParserByteSpanV0, ParserPositionV0, ParserRangeV0, StyleLanguage, parse_style_module,
    summarize_css_modules_intermediate,
};
use omena_incremental::IncrementalCancellationRegistryV0;
use omena_tsgo_client::{OmenaTsgoClientBoundarySummaryV0, summarize_omena_tsgo_client_boundary};
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
        candidate_owner: "omena-lsp-server/sourceSyntaxIndex",
        style_definition_owner: "engine-style-parser/selectorDefinitionFacts",
        type_fact_owner: "omena-tsgo-client",
        request_path_policy: vec![
            "noNodeWorkspaceTypeResolverOnSourceProviderPath",
            "buildSourceSyntaxIndexOnDocumentChange",
            "dedupeTargetAwareSourceCandidates",
            "consumeParserCanonicalSelectorFacts",
            "consumeParserSelectorDefinitionFacts",
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct SourceSyntaxIndex {
    imported_style_bindings: Vec<ImportedStyleBinding>,
    class_string_literals: Vec<ParserByteSpanV0>,
    style_property_accesses: Vec<SourceStylePropertyAccessFact>,
    selector_references: Vec<SourceSelectorReferenceFact>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceStylePropertyAccessFact {
    byte_span: ParserByteSpanV0,
    target_style_uri: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceSelectorReferenceFact {
    byte_span: ParserByteSpanV0,
    target_style_uri: Option<String>,
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
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
                    "source": "openedSourceDocumentIndex",
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
            locations_by_name
                .entry(candidate.name)
                .or_default()
                .push(json!({
                    "uri": document.uri.as_str(),
                    "range": candidate.range,
                }));
        }
    }
    for locations in locations_by_name.values_mut() {
        locations.sort_by_key(location_sort_key);
        locations
            .dedup_by(|left, right| location_identity_key(left) == location_identity_key(right));
    }
    locations_by_name
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
    let Some(candidate) = source_selector_candidate_at_position(state, document, position) else {
        return Value::Null;
    };
    let definition = style_selector_definitions_for_source_candidate(
        state,
        &candidate,
        document.workspace_folder_uri.as_deref(),
    )
    .into_iter()
    .next();
    let value = definition
        .as_ref()
        .and_then(|(uri, definition)| {
            state.document(uri).map(|style_document| {
                render_style_hover_candidate_markdown(
                    uri.as_str(),
                    style_document.text.as_str(),
                    definition,
                )
            })
        })
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
    let Some(candidate) = source_selector_candidate_at_position(state, document, position) else {
        return Value::Null;
    };
    let definitions = style_selector_definitions_for_source_candidate(
        state,
        &candidate,
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
    let Some(candidate) = source_selector_candidate_at_position(state, document, position) else {
        return Value::Null;
    };
    let include_declaration = include_declaration_from_params(params);
    let mut locations = Vec::new();
    if include_declaration {
        locations.extend(
            style_selector_definitions_for_source_candidate(
                state,
                &candidate,
                document.workspace_folder_uri.as_deref(),
            )
            .into_iter()
            .map(|(uri, definition)| json!({ "uri": uri, "range": definition.range })),
        );
    }
    locations.extend(selector_reference_locations_from_open_documents(
        state,
        candidate.name.as_str(),
        document.workspace_folder_uri.as_deref(),
        candidate.target_style_uri.as_deref(),
    ));
    locations.sort_by_key(location_sort_key);

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
    let Some(target_style_uri) = source_completion_target_style_uri_at_position(document, position)
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

fn source_completion_target_style_uri_at_position(
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Option<Option<String>> {
    let offset = byte_offset_for_parser_position(document.text.as_str(), position)?;
    if document
        .source_syntax_index
        .class_string_literals
        .iter()
        .any(|span| offset >= span.start && offset <= span.end)
    {
        return Some(None);
    }
    styles_property_access_completion_target_style_uri(document, offset)
}

fn styles_property_access_completion_target_style_uri(
    document: &LspTextDocumentState,
    offset: usize,
) -> Option<Option<String>> {
    document
        .source_syntax_index
        .style_property_accesses
        .iter()
        .find(|access| offset >= access.byte_span.start && offset <= access.byte_span.end)
        .map(|access| access.target_style_uri.clone())
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
    collect_source_selector_reference_candidates(state, document)
        .into_iter()
        .find(|candidate| parser_range_contains_position(&candidate.range, position))
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
    let definitions = style_selector_definitions_from_open_documents(
        state,
        "",
        document.workspace_folder_uri.as_deref(),
    );
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
        collect_source_class_reference_candidates(document)
            .into_iter()
            .partition(|candidate| {
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
        definition.name == candidate.name
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
        .map(|reference| {
            source_reference_candidate(
                document,
                reference.byte_span,
                reference.target_style_uri.clone(),
            )
        })
        .collect();
    candidates.sort();
    candidates.dedup();
    candidates
}

fn build_source_syntax_index(document: &LspTextDocumentState) -> SourceSyntaxIndex {
    if is_style_document_uri(document.uri.as_str()) {
        return SourceSyntaxIndex::default();
    }

    let source = document.text.as_str();
    let imports = collect_source_imports(document);
    let imported_style_targets = imported_style_targets(imports.imported_style_bindings.as_slice());
    let property_access_targets =
        property_access_style_targets(imports.imported_style_bindings.as_slice());
    let classnames_bind_targets = collect_classnames_bind_utility_bindings(
        source,
        imported_style_targets.as_slice(),
        imports.classnames_bind_bindings.as_slice(),
    );

    let mut index = SourceSyntaxIndex {
        imported_style_bindings: imports.imported_style_bindings,
        class_string_literals: collect_class_name_string_literal_spans(source),
        style_property_accesses: collect_style_property_access_facts(
            source,
            property_access_targets.as_slice(),
        ),
        selector_references: Vec::new(),
    };

    for span in &index.class_string_literals {
        push_string_literal_selector_references(
            source,
            *span,
            None,
            &mut index.selector_references,
        );
    }
    for access in &index.style_property_accesses {
        index.selector_references.push(SourceSelectorReferenceFact {
            byte_span: access.byte_span,
            target_style_uri: access.target_style_uri.clone(),
        });
    }
    for binding in classnames_bind_targets {
        collect_string_literal_call_reference_facts(
            source,
            binding.binding.as_str(),
            Some(binding.style_uri.as_str()),
            &mut index.selector_references,
        );
    }
    canonicalize_source_selector_references(&mut index.selector_references);

    index
}

fn canonicalize_source_selector_references(references: &mut Vec<SourceSelectorReferenceFact>) {
    let mut targets_by_span: BTreeMap<(usize, usize), BTreeSet<Option<String>>> = BTreeMap::new();
    for reference in references.iter() {
        targets_by_span
            .entry((reference.byte_span.start, reference.byte_span.end))
            .or_default()
            .insert(reference.target_style_uri.clone());
    }

    let mut canonical = Vec::new();
    for ((start, end), targets) in targets_by_span {
        let has_targeted_reference = targets.iter().any(Option::is_some);
        for target_style_uri in targets {
            if has_targeted_reference && target_style_uri.is_none() {
                continue;
            }
            canonical.push(SourceSelectorReferenceFact {
                byte_span: ParserByteSpanV0 { start, end },
                target_style_uri,
            });
        }
    }
    *references = canonical;
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
    let mut cursor = 0usize;
    while let Some(identifier) = next_code_identifier(source, cursor) {
        cursor = identifier.end;
        if identifier.text != "import" {
            continue;
        }
        if let Some(import) = parse_source_import_declaration(document, identifier.end) {
            if import.specifier == "classnames/bind" {
                imports.classnames_bind_bindings.push(import.binding);
            } else if StyleLanguage::from_module_path(import.specifier.as_str()).is_some()
                && let Some(style_uri) =
                    style_uri_for_import_specifier(document.uri.as_str(), import.specifier.as_str())
            {
                imports.imported_style_bindings.push(ImportedStyleBinding {
                    binding: import.binding,
                    style_uri,
                });
            }
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceImportDeclaration {
    binding: String,
    specifier: String,
}

fn parse_source_import_declaration(
    document: &LspTextDocumentState,
    after_import: usize,
) -> Option<SourceImportDeclaration> {
    let source = document.text.as_str();
    let mut cursor = skip_js_trivia(source, after_import);
    match source.as_bytes().get(cursor).copied()? {
        b'(' | b'\'' | b'"' => return None,
        _ => {}
    }

    let clause_start = cursor;
    let mut clause_end = None;
    let mut specifier = None;
    while cursor < source.len() {
        cursor = skip_js_trivia(source, cursor);
        let Some(byte) = source.as_bytes().get(cursor).copied() else {
            break;
        };
        if matches!(byte, b'\'' | b'"') {
            if clause_end.is_some()
                && let Some((literal_start, literal_end, _)) =
                    js_string_literal_span(source, cursor, source.len())
            {
                specifier = source.get(literal_start..literal_end).map(str::to_string);
            }
            break;
        }
        if byte == b';' {
            break;
        }
        if byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$') {
            let (identifier, identifier_end) = read_js_identifier(source, cursor)?;
            if identifier == "from" && clause_end.is_none() {
                clause_end = Some(cursor);
            }
            cursor = identifier_end;
            continue;
        }
        cursor += 1;
    }

    let clause = source.get(clause_start..clause_end?)?;
    Some(SourceImportDeclaration {
        binding: import_binding_from_clause(clause)?.to_string(),
        specifier: specifier?,
    })
}

fn import_binding_from_clause(clause: &str) -> Option<&str> {
    let clause = clause.trim();
    if clause.is_empty() || clause.starts_with('{') {
        return None;
    }
    if let Some(namespace_clause) = clause.strip_prefix('*') {
        let namespace_clause = namespace_clause.trim_start();
        let namespace_clause = namespace_clause.strip_prefix("as")?.trim_start();
        let (binding, _) = read_js_identifier(namespace_clause, 0)?;
        return Some(binding);
    }

    let default_clause = clause.split(',').next()?.trim();
    let (binding, _) = read_js_identifier(default_clause, 0)?;
    if binding == "type" {
        return None;
    }
    Some(binding)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceStyleBindingTarget {
    binding: String,
    target_style_uri: Option<String>,
}

fn imported_style_targets(bindings: &[ImportedStyleBinding]) -> Vec<SourceStyleBindingTarget> {
    bindings
        .iter()
        .map(|binding| SourceStyleBindingTarget {
            binding: binding.binding.clone(),
            target_style_uri: Some(binding.style_uri.clone()),
        })
        .collect()
}

fn property_access_style_targets(
    bindings: &[ImportedStyleBinding],
) -> Vec<SourceStyleBindingTarget> {
    let imported = imported_style_targets(bindings);
    if imported.is_empty() {
        vec![SourceStyleBindingTarget {
            binding: "styles".to_string(),
            target_style_uri: None,
        }]
    } else {
        imported
    }
}

fn collect_class_name_string_literal_spans(source: &str) -> Vec<ParserByteSpanV0> {
    let mut spans = Vec::new();
    let mut cursor = 0usize;
    while let Some(identifier) = next_code_identifier(source, cursor) {
        cursor = identifier.end;
        if identifier.text != "className" {
            continue;
        }
        let equals_offset = skip_js_trivia(source, identifier.end);
        if source.as_bytes().get(equals_offset) != Some(&b'=') {
            continue;
        }
        let value_offset = skip_js_trivia(source, equals_offset + 1);
        match source.as_bytes().get(value_offset).copied() {
            Some(b'\'' | b'"' | b'`') => {
                if let Some((literal_start, literal_end, next_offset)) =
                    js_string_literal_span(source, value_offset, source.len())
                {
                    spans.push(ParserByteSpanV0 {
                        start: literal_start,
                        end: literal_end,
                    });
                    cursor = next_offset;
                }
            }
            Some(b'{') => {
                let expression_start = value_offset + 1;
                if let Some(expression_end) = jsx_expression_end(source, expression_start) {
                    spans.extend(collect_js_string_literal_spans(
                        source,
                        expression_start,
                        expression_end,
                    ));
                    cursor = (expression_end + 1).min(source.len());
                }
            }
            _ => {}
        }
    }
    spans
}

fn collect_js_string_literal_spans(
    source: &str,
    start: usize,
    end: usize,
) -> Vec<ParserByteSpanV0> {
    let mut spans = Vec::new();
    let mut cursor = start;
    while cursor < end {
        cursor = skip_js_trivia_until(source, cursor, end);
        let Some(byte) = source.as_bytes().get(cursor).copied() else {
            break;
        };
        if matches!(byte, b'\'' | b'"')
            && let Some((literal_start, literal_end, next_offset)) =
                js_string_literal_span(source, cursor, end)
        {
            spans.push(ParserByteSpanV0 {
                start: literal_start,
                end: literal_end,
            });
            cursor = next_offset;
            continue;
        }
        cursor += 1;
    }
    spans
}

fn collect_style_property_access_facts(
    source: &str,
    targets: &[SourceStyleBindingTarget],
) -> Vec<SourceStylePropertyAccessFact> {
    let mut facts = Vec::new();
    let mut cursor = 0usize;
    while let Some(identifier) = next_code_identifier(source, cursor) {
        cursor = identifier.end;
        let Some(target) = targets
            .iter()
            .find(|target| target.binding == identifier.text)
        else {
            continue;
        };
        let member_offset = skip_js_trivia(source, identifier.end);
        if source.as_bytes().get(member_offset) == Some(&b'.') {
            let start = member_offset + 1;
            let end = read_css_identifier_end(source, start);
            if end > start {
                facts.push(SourceStylePropertyAccessFact {
                    byte_span: ParserByteSpanV0 { start, end },
                    target_style_uri: target.target_style_uri.clone(),
                });
                cursor = end;
            }
            continue;
        }
        if source.as_bytes().get(member_offset) == Some(&b'[')
            && let Some((literal_start, literal_end, bracket_end)) =
                bracket_string_literal_access(source, member_offset)
        {
            if literal_end > literal_start
                && source[literal_start..literal_end]
                    .chars()
                    .all(is_css_identifier_continue)
            {
                facts.push(SourceStylePropertyAccessFact {
                    byte_span: ParserByteSpanV0 {
                        start: literal_start,
                        end: literal_end,
                    },
                    target_style_uri: target.target_style_uri.clone(),
                });
            }
            cursor = bracket_end.min(source.len());
        }
    }
    facts
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClassnamesBindUtilityBinding {
    binding: String,
    style_uri: String,
}

fn collect_classnames_bind_utility_bindings(
    source: &str,
    style_targets: &[SourceStyleBindingTarget],
    classnames_bind_imports: &[String],
) -> Vec<ClassnamesBindUtilityBinding> {
    if style_targets.is_empty() || classnames_bind_imports.is_empty() {
        return Vec::new();
    }
    let mut bindings = Vec::new();
    let mut cursor = 0usize;
    while let Some(keyword) = next_code_identifier(source, cursor) {
        cursor = keyword.end;
        if !matches!(keyword.text, "const" | "let" | "var") {
            continue;
        }
        if let Some((binding, next_offset)) = parse_classnames_bind_utility_binding(
            source,
            keyword.end,
            style_targets,
            classnames_bind_imports,
        ) {
            bindings.push(binding);
            cursor = next_offset;
        }
    }
    bindings.sort_by(|left, right| {
        left.binding
            .cmp(&right.binding)
            .then_with(|| left.style_uri.cmp(&right.style_uri))
    });
    bindings
        .dedup_by(|left, right| left.binding == right.binding && left.style_uri == right.style_uri);
    bindings
}

fn parse_classnames_bind_utility_binding(
    source: &str,
    after_keyword: usize,
    style_targets: &[SourceStyleBindingTarget],
    classnames_bind_imports: &[String],
) -> Option<(ClassnamesBindUtilityBinding, usize)> {
    let binding_start = skip_js_trivia(source, after_keyword);
    let (binding, binding_end) = read_js_identifier(source, binding_start)?;
    let equals_offset = skip_js_trivia(source, binding_end);
    if source.as_bytes().get(equals_offset) != Some(&b'=') {
        return None;
    }
    let callee_start = skip_js_trivia(source, equals_offset + 1);
    let (callee, callee_end) = read_js_identifier(source, callee_start)?;
    if !classnames_bind_imports
        .iter()
        .any(|import_binding| import_binding == callee)
    {
        return None;
    }
    let dot_offset = skip_js_trivia(source, callee_end);
    if source.as_bytes().get(dot_offset) != Some(&b'.') {
        return None;
    }
    let (property, property_end) = read_js_identifier(source, dot_offset + 1)?;
    if property != "bind" {
        return None;
    }
    let open_paren = skip_js_trivia(source, property_end);
    if source.as_bytes().get(open_paren) != Some(&b'(') {
        return None;
    }
    let style_arg_start = skip_js_trivia(source, open_paren + 1);
    let (style_binding_name, style_binding_end) = read_js_identifier(source, style_arg_start)?;
    let style_uri = style_targets
        .iter()
        .find(|style_binding| style_binding.binding == style_binding_name)?
        .target_style_uri
        .clone();
    let style_uri = style_uri?;

    Some((
        ClassnamesBindUtilityBinding {
            binding: binding.to_string(),
            style_uri,
        },
        style_binding_end,
    ))
}

fn collect_string_literal_call_reference_facts(
    source: &str,
    binding: &str,
    target_style_uri: Option<&str>,
    references: &mut Vec<SourceSelectorReferenceFact>,
) {
    let mut cursor = 0usize;
    while let Some(identifier) = next_code_identifier(source, cursor) {
        cursor = identifier.end;
        if identifier.text != binding {
            continue;
        }
        let open_paren = skip_js_trivia(source, identifier.end);
        if source.as_bytes().get(open_paren) != Some(&b'(') {
            continue;
        }
        let call_end = js_call_end(source, open_paren).unwrap_or(source.len());
        for literal_span in collect_js_string_literal_spans(source, open_paren + 1, call_end) {
            push_string_literal_selector_references(
                source,
                literal_span,
                target_style_uri.map(ToString::to_string),
                references,
            );
        }
        cursor = call_end.saturating_add(1).min(source.len());
    }
}

fn js_call_end(source: &str, open_paren: usize) -> Option<usize> {
    if source.as_bytes().get(open_paren) != Some(&b'(') {
        return None;
    }
    let mut cursor = open_paren + 1;
    let mut depth = 1usize;
    while cursor < source.len() {
        match source.as_bytes().get(cursor).copied()? {
            b'\'' | b'"' | b'`' => {
                cursor = skip_js_string_literal(source, cursor, source.len())?;
            }
            b'(' => {
                depth += 1;
                cursor += 1;
            }
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(cursor);
                }
                cursor += 1;
            }
            _ => {
                cursor += 1;
            }
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ImportedStyleBinding {
    binding: String,
    style_uri: String,
}

fn push_string_literal_selector_references(
    source: &str,
    literal_span: ParserByteSpanV0,
    target_style_uri: Option<String>,
    references: &mut Vec<SourceSelectorReferenceFact>,
) {
    for span in class_token_byte_spans(source, literal_span.start, literal_span.end) {
        references.push(SourceSelectorReferenceFact {
            byte_span: span,
            target_style_uri: target_style_uri.clone(),
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CodeIdentifier<'a> {
    text: &'a str,
    end: usize,
}

fn next_code_identifier(source: &str, mut cursor: usize) -> Option<CodeIdentifier<'_>> {
    while cursor < source.len() {
        cursor = skip_js_trivia(source, cursor);
        let byte = source.as_bytes().get(cursor).copied()?;
        if matches!(byte, b'\'' | b'"' | b'`') {
            cursor = skip_js_string_literal(source, cursor, source.len()).unwrap_or(source.len());
            continue;
        }
        if byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$') {
            let (text, end) = read_js_identifier(source, cursor)?;
            return Some(CodeIdentifier { text, end });
        }
        cursor += 1;
    }
    None
}

fn skip_js_trivia(source: &str, cursor: usize) -> usize {
    skip_js_trivia_until(source, cursor, source.len())
}

fn skip_js_trivia_until(source: &str, mut cursor: usize, limit: usize) -> usize {
    loop {
        cursor = skip_ascii_whitespace_until(source, cursor, limit);
        if source.as_bytes().get(cursor) == Some(&b'/') {
            match source.as_bytes().get(cursor + 1).copied() {
                Some(b'/') => {
                    cursor = skip_js_line_comment(source, cursor + 2, limit);
                    continue;
                }
                Some(b'*') => {
                    cursor = skip_js_block_comment(source, cursor + 2, limit);
                    continue;
                }
                _ => {}
            }
        }
        return cursor;
    }
}

fn skip_ascii_whitespace_until(source: &str, mut offset: usize, limit: usize) -> usize {
    while offset < limit
        && source
            .as_bytes()
            .get(offset)
            .is_some_and(u8::is_ascii_whitespace)
    {
        offset += 1;
    }
    offset
}

fn skip_js_line_comment(source: &str, mut cursor: usize, limit: usize) -> usize {
    while cursor < limit {
        if source.as_bytes().get(cursor) == Some(&b'\n') {
            return cursor + 1;
        }
        cursor += 1;
    }
    limit
}

fn skip_js_block_comment(source: &str, mut cursor: usize, limit: usize) -> usize {
    while cursor + 1 < limit {
        if source.as_bytes().get(cursor) == Some(&b'*')
            && source.as_bytes().get(cursor + 1) == Some(&b'/')
        {
            return cursor + 2;
        }
        cursor += 1;
    }
    limit
}

fn style_uri_for_import_specifier(source_uri: &str, specifier: &str) -> Option<String> {
    if !specifier.starts_with('.') {
        return None;
    }
    let source_path = file_uri_to_path(source_uri)?;
    let imported_path = normalize_path(source_path.parent()?.join(specifier));
    Some(path_to_file_uri(imported_path.as_path()))
}

fn source_reference_candidate(
    document: &LspTextDocumentState,
    byte_span: ParserByteSpanV0,
    target_style_uri: Option<String>,
) -> LspStyleHoverCandidate {
    LspStyleHoverCandidate {
        kind: "sourceSelectorReference",
        name: document.text[byte_span.start..byte_span.end].to_string(),
        range: parser_range_for_byte_span(document.text.as_str(), byte_span),
        source: "openedSourceDocumentIndex",
        target_style_uri,
    }
}

fn class_token_byte_spans(
    source: &str,
    literal_start: usize,
    literal_end: usize,
) -> Vec<ParserByteSpanV0> {
    let mut spans = Vec::new();
    let mut token_start: Option<usize> = None;
    for (relative_index, ch) in source[literal_start..literal_end].char_indices() {
        let index = literal_start + relative_index;
        if ch.is_ascii_whitespace() {
            if let Some(start) = token_start.take() {
                push_class_token_span(source, start, index, &mut spans);
            }
        } else if token_start.is_none() {
            token_start = Some(index);
        }
    }
    if let Some(start) = token_start {
        push_class_token_span(source, start, literal_end, &mut spans);
    }
    spans
}

fn push_class_token_span(
    source: &str,
    start: usize,
    end: usize,
    spans: &mut Vec<ParserByteSpanV0>,
) {
    if start < end && source[start..end].chars().all(is_css_identifier_continue) {
        spans.push(ParserByteSpanV0 { start, end });
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

fn jsx_expression_end(source: &str, start: usize) -> Option<usize> {
    let mut cursor = start;
    let mut nested_braces = 0usize;
    while cursor < source.len() {
        match source.as_bytes().get(cursor).copied()? {
            b'\'' | b'"' | b'`' => {
                cursor = skip_js_string_literal(source, cursor, source.len())?;
            }
            b'{' => {
                nested_braces += 1;
                cursor += 1;
            }
            b'}' => {
                if nested_braces == 0 {
                    return Some(cursor);
                }
                nested_braces -= 1;
                cursor += 1;
            }
            _ => {
                cursor += 1;
            }
        }
    }
    None
}

fn js_string_literal_span(
    source: &str,
    quote_offset: usize,
    limit: usize,
) -> Option<(usize, usize, usize)> {
    let quote = source.as_bytes().get(quote_offset).copied()?;
    if !matches!(quote, b'\'' | b'"' | b'`') {
        return None;
    }
    let literal_start = quote_offset + 1;
    let next_offset = skip_js_string_literal(source, quote_offset, limit)?;
    Some((literal_start, next_offset - 1, next_offset))
}

fn skip_js_string_literal(source: &str, quote_offset: usize, limit: usize) -> Option<usize> {
    let quote = source.as_bytes().get(quote_offset).copied()?;
    let mut cursor = quote_offset + 1;
    while cursor < limit {
        let byte = source.as_bytes().get(cursor).copied()?;
        if byte == b'\\' {
            cursor = (cursor + 2).min(limit);
            continue;
        }
        if byte == quote {
            return Some(cursor + 1);
        }
        cursor += 1;
    }
    None
}

fn bracket_string_literal_access(
    source: &str,
    bracket_offset: usize,
) -> Option<(usize, usize, usize)> {
    if source.as_bytes().get(bracket_offset) != Some(&b'[') {
        return None;
    }
    let quote_offset = skip_ascii_whitespace(source, bracket_offset + 1);
    let quote = source.as_bytes().get(quote_offset).copied()?;
    if !matches!(quote, b'\'' | b'"') {
        return None;
    }
    let (literal_start, literal_end, literal_next) =
        js_string_literal_span(source, quote_offset, source.len())?;
    if literal_next > source.len() {
        return None;
    }
    let closing_bracket = skip_ascii_whitespace(source, literal_end + 1);
    if source.as_bytes().get(closing_bracket) != Some(&b']') {
        return None;
    }
    Some((literal_start, literal_end, closing_bracket + 1))
}

fn read_css_identifier_end(source: &str, start: usize) -> usize {
    let mut end = start;
    for (relative_index, ch) in source[start..].char_indices() {
        if !is_css_identifier_continue(ch) {
            break;
        }
        end = start + relative_index + ch.len_utf8();
    }
    end
}

fn read_js_identifier(source: &str, start: usize) -> Option<(&str, usize)> {
    let first = source[start..].chars().next()?;
    if !is_js_identifier_start(first) {
        return None;
    }
    let mut end = start + first.len_utf8();
    let scan_start = end;
    for (relative_index, ch) in source[scan_start..].char_indices() {
        if !is_js_identifier_continue(ch) {
            break;
        }
        end = scan_start + relative_index + ch.len_utf8();
    }
    Some((&source[start..end], end))
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

fn style_selector_definitions_for_source_candidate(
    state: &LspShellState,
    candidate: &LspStyleHoverCandidate,
    workspace_folder_uri: Option<&str>,
) -> Vec<(String, LspStyleHoverCandidate)> {
    style_selector_definitions_from_open_documents(
        state,
        candidate.name.as_str(),
        workspace_folder_uri,
    )
    .into_iter()
    .filter(|(uri, _)| {
        candidate
            .target_style_uri
            .as_deref()
            .is_none_or(|target_uri| target_uri == uri)
    })
    .collect()
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
        (Some(left), Some(right)) => left == right,
        _ => true,
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
        _ => candidate.name.clone(),
    }
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
        cursor += 1;
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
                });
            }
        }
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

fn is_js_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || matches!(ch, '_' | '$')
}

fn is_js_identifier_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '$')
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
