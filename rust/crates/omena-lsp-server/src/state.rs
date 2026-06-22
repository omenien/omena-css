use crate::disk_cache::DiskDiagnosticsCacheSessionV0;
use crate::workspace_runtime_registry::WorkspaceRuntimeRegistry;
use omena_incremental::IncrementalCancellationRegistryV0;
use omena_query::{
    AnalyzedGraphV0, OmenaQueryExternalSifInputV0, OmenaQuerySourceSelectorOccurrenceIndexV0,
    OmenaQuerySourceSyntaxIndexV0 as SourceSyntaxIndex, OmenaQueryStyleCascadeNarrowingSubstrateV0,
    OmenaQueryStylePackageManifestV0, OmenaQueryStyleResolutionInputsV0,
    OmenaQueryStyleSelectorDefinitionV0, OmenaQueryStyleSourceInputV0,
    OmenaWorkspaceOccurrenceFamilyV0, OmenaWorkspaceOccurrenceIndexV0,
    OmenaWorkspaceOccurrenceKindV0, OmenaWorkspaceOccurrenceRoleV0, ParserPositionV0,
    ParserRangeV0,
};
use omena_tsgo_client::{TsgoTypeFactResultEntryV0, TsgoWorkspaceProcessPoolV0};
use serde::Serialize;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LspDocumentOrigin {
    Local,
    Foreign,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspTextDocumentState {
    pub uri: String,
    #[serde(skip)]
    pub origin: LspDocumentOrigin,
    pub workspace_folder_uri: Option<String>,
    pub language_id: String,
    pub version: i64,
    pub text: String,
    #[serde(skip)]
    pub(crate) text_hash: String,
    pub style_summary: Option<LspStyleDocumentSummary>,
    pub diagnostics_schedule_count: usize,
    pub optimizing_tier_feedback: Option<LspOptimizingTierFeedback>,
    #[serde(skip)]
    pub style_candidates: Vec<LspStyleHoverCandidate>,
    #[serde(skip)]
    pub(crate) source_syntax_index: SourceSyntaxIndex,
    #[serde(skip)]
    pub(crate) has_unresolved_style_import: bool,
    #[serde(skip)]
    pub source_selector_candidates: Vec<LspStyleHoverCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspOptimizingTierFeedback {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub document_version: i64,
    pub policy: &'static str,
    pub consumer: &'static str,
    pub analyzed_graph: AnalyzedGraphV0,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct LspFileId(u32);

#[derive(Debug, Clone, Default)]
pub(crate) struct LspFileIdentityInterner {
    next_id: u32,
    ids_by_storage_uri: BTreeMap<String, LspFileId>,
    storage_uris_by_id: BTreeMap<LspFileId, String>,
}

impl LspFileIdentityInterner {
    fn intern_uri(&mut self, uri: &str) -> (LspFileId, String) {
        let storage_uri = Self::storage_uri(uri);
        if let Some(file_id) = self.ids_by_storage_uri.get(storage_uri.as_str()) {
            return (*file_id, storage_uri);
        }
        let file_id = LspFileId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.ids_by_storage_uri.insert(storage_uri.clone(), file_id);
        self.storage_uris_by_id.insert(file_id, storage_uri.clone());
        (file_id, storage_uri)
    }

    fn file_id_for_uri(&self, uri: &str) -> Option<LspFileId> {
        let storage_uri = Self::storage_uri(uri);
        self.ids_by_storage_uri.get(storage_uri.as_str()).copied()
    }

    pub(crate) fn storage_uri_for_file_id(&self, file_id: LspFileId) -> Option<&str> {
        self.storage_uris_by_id.get(&file_id).map(String::as_str)
    }

    fn storage_uri(uri: &str) -> String {
        crate::protocol::canonical_file_uri(uri).unwrap_or_else(|| uri.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspShellStateSnapshot {
    pub shutdown_requested: bool,
    pub should_exit: bool,
    pub features: LspFeatureSettings,
    pub diagnostics: LspDiagnosticSettings,
    pub resolution: LspResolutionSettings,
    pub cancelled_request_count: usize,
    pub workspace_style_index_exhausted_count: usize,
    pub workspace_index_pending_file_count: usize,
    pub external_sif_lock_read_count: usize,
    pub external_sif_bridge_generation_count: usize,
    pub document_count: usize,
    pub workspace_folder_count: usize,
    pub configuration_change_count: usize,
    pub watched_file_event_count: usize,
    pub cached_workspace_resolution_input_count: usize,
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
    pub deep_analysis: bool,
}

impl Default for LspDiagnosticSettings {
    fn default() -> Self {
        Self {
            severity: 2,
            deep_analysis: false,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspResolutionSettings {
    pub package_manifest_paths: Vec<String>,
    #[serde(skip)]
    pub package_manifests: Vec<OmenaQueryStylePackageManifestV0>,
    #[serde(skip)]
    pub workspace_style_resolution_inputs: BTreeMap<String, OmenaQueryStyleResolutionInputsV0>,
    /// External Sass-module SIF artifacts sourced from workspace locks and bridge generation.
    /// The diagnostics path runs in Auto mode, so source-available edges stay local while
    /// SIF-backed and unresolved foreign edges are classified per import edge.
    #[serde(skip)]
    pub external_sifs: Vec<OmenaQueryExternalSifInputV0>,
    #[serde(skip)]
    pub(crate) bridge_external_sif_urls: BTreeSet<String>,
}

/// Workspace-revision memo for the cascade-narrowing substrate (rfcs#63 E-ii).
/// Self-validating: the key is the exact narrowing input set (ordered style sources +
/// package manifests + external SIFs + resolution mappings), so any document
/// open/close/edit, disk reload, or resolution-config change misses by comparison and
/// rebuilds — there is no eviction site to keep in sync.
#[derive(Debug)]
pub(crate) struct LspCascadeNarrowingSubstrateMemo {
    pub(crate) style_sources: Vec<OmenaQueryStyleSourceInputV0>,
    pub(crate) package_manifests: Vec<OmenaQueryStylePackageManifestV0>,
    pub(crate) external_sifs: Vec<OmenaQueryExternalSifInputV0>,
    pub(crate) resolution_inputs: OmenaQueryStyleResolutionInputsV0,
    pub(crate) substrate: Arc<OmenaQueryStyleCascadeNarrowingSubstrateV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LspSourceSelectorOccurrenceDocumentKey {
    pub(crate) uri: String,
    pub(crate) workspace_folder_uri: Option<String>,
    pub(crate) language_id: String,
    pub(crate) version: i64,
    pub(crate) text_hash: String,
}

#[derive(Debug, Clone)]
pub(crate) struct LspWorkspaceOccurrenceIndexMemo {
    pub(crate) workspace_folder_uri: Option<String>,
    pub(crate) source_document_keys: Vec<LspSourceSelectorOccurrenceDocumentKey>,
    pub(crate) style_document_keys: Vec<LspSourceSelectorOccurrenceDocumentKey>,
    pub(crate) definitions: Vec<OmenaQueryStyleSelectorDefinitionV0>,
    pub(crate) source_selector_index: Arc<OmenaQuerySourceSelectorOccurrenceIndexV0>,
    pub(crate) workspace_index: Arc<OmenaWorkspaceOccurrenceIndexV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LspStyleSymbolOccurrenceV0 {
    pub(crate) moniker: String,
    pub(crate) uri: String,
    pub(crate) kind: OmenaWorkspaceOccurrenceKindV0,
    pub(crate) family: OmenaWorkspaceOccurrenceFamilyV0,
    pub(crate) name: String,
    pub(crate) range: ParserRangeV0,
    pub(crate) role: OmenaWorkspaceOccurrenceRoleV0,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) namespace: Option<String>,
}

#[derive(Debug, Default)]
pub struct LspShellState {
    pub shutdown_requested: bool,
    pub should_exit: bool,
    pub(crate) features: LspFeatureSettings,
    pub(crate) diagnostics: LspDiagnosticSettings,
    pub(crate) resolution: LspResolutionSettings,
    pub(crate) cancelled_request_ids: IncrementalCancellationRegistryV0,
    pub(crate) workspace_style_index_exhausted_count: usize,
    pub(crate) workspace_index_pending_file_count: usize,
    pub(crate) external_sif_lock_read_count: usize,
    pub(crate) external_sif_bridge_generation_count: usize,
    pub(crate) external_sif_refresh_deferred: bool,
    pub(crate) external_sif_refresh_dirty: bool,
    pub(crate) external_sif_refresh_revision: u64,
    pub(crate) workspace_index_revision: u64,
    pub(crate) configuration_change_count: usize,
    /// RFC 0009 Pillar A (rfcs#67, slice A-min): documents are `Arc` entries so a
    /// query snapshot clones pointers instead of the corpus; mutation paths go
    /// through `document_mut`/`insert_document`, which copy-on-write via
    /// `Arc::make_mut`/`Arc::new` (a worker holding a snapshot of a document
    /// forces at most a one-document deep clone on that document's next edit).
    pub(crate) file_identity: LspFileIdentityInterner,
    pub(crate) documents: BTreeMap<LspFileId, Arc<LspTextDocumentState>>,
    pub(crate) open_document_uris: BTreeSet<LspFileId>,
    pub(crate) workspace_runtime_registry: WorkspaceRuntimeRegistry,
    pub(crate) tsgo_workspace_process_pool: TsgoWorkspaceProcessPoolV0,
    pub(crate) watched_file_changes: Vec<LspWatchedFileChangeState>,
    pub(crate) client_supports_work_done_progress: bool,
    pub(crate) next_server_progress_request_id: u64,
    pub(crate) pending_server_progress_request_tokens: BTreeMap<String, String>,
    /// Shared (not per-state) since RFC 0009 Pillar A: the loop and dispatched
    /// query snapshots reuse ONE memo slot so a substrate built on either side is
    /// visible to both. The memo is self-validating by exact input compare, so
    /// last-writer-wins is safe; lock only to compare and to store — never across
    /// the substrate collection (see `cascade_narrowing_substrate_for_style_sources`).
    pub(crate) cascade_narrowing_substrate_memo:
        Arc<Mutex<Option<LspCascadeNarrowingSubstrateMemo>>>,
    pub(crate) workspace_occurrence_index_memo: RefCell<Option<LspWorkspaceOccurrenceIndexMemo>>,
    pub(crate) source_type_fact_cache: BTreeMap<String, Vec<TsgoTypeFactResultEntryV0>>,
    /// RFC 0009 Pillar C (rfcs#66): fail-soft write breaker for the disk
    /// diagnostics shard cache. Interior mutability because the write-behind
    /// runs on the immutable resolve path; owned by the single loop thread.
    pub(crate) disk_diagnostics_cache_session: RefCell<DiskDiagnosticsCacheSessionV0>,
    /// RFC 0009 Pillar B (rfcs#65): the long-lived salsa-memoized
    /// style-diagnostics host. Owned by the loop thread; the host diff-syncs
    /// its inputs on every resolve, so it never serves a stale revision.
    #[cfg(feature = "salsa-style-diagnostics")]
    pub(crate) style_memo_host: RefCell<Option<omena_query::OmenaQueryStyleMemoHostV0>>,
}

impl LspShellState {
    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    pub fn workspace_folder_count(&self) -> usize {
        self.workspace_runtime_registry.len()
    }

    pub(crate) fn allocate_work_done_progress_request(&mut self) -> (String, String) {
        self.next_server_progress_request_id += 1;
        let id = format!(
            "omena-work-done-progress-create-{}",
            self.next_server_progress_request_id
        );
        let token = format!(
            "omena-workspace-index-{}",
            self.next_server_progress_request_id
        );
        self.pending_server_progress_request_tokens
            .insert(id.clone(), token.clone());
        (id, token)
    }

    pub(crate) fn take_server_progress_response(&mut self, id: &str) -> bool {
        self.pending_server_progress_request_tokens
            .remove(id)
            .is_some()
    }

    pub fn document(&self, uri: &str) -> Option<&LspTextDocumentState> {
        let file_id = self.file_identity.file_id_for_uri(uri)?;
        self.documents.get(&file_id).map(Arc::as_ref)
    }

    #[cfg(feature = "test-support")]
    pub fn evict_document_for_test(&mut self, uri: &str) -> Option<LspTextDocumentState> {
        self.remove_document_uri(uri)
    }

    #[cfg(feature = "test-support")]
    pub fn clear_workspace_occurrence_index_memo_for_test(&self) {
        *self.workspace_occurrence_index_memo.borrow_mut() = None;
    }

    pub(crate) fn document_mut(&mut self, uri: &str) -> Option<&mut LspTextDocumentState> {
        let file_id = self.file_identity.file_id_for_uri(uri)?;
        let document = self.documents.get_mut(&file_id)?;
        if document.origin == LspDocumentOrigin::Foreign {
            return None;
        }
        Some(Arc::make_mut(document))
    }

    #[cfg(test)]
    pub(crate) fn document_storage_uri(uri: &str) -> String {
        LspFileIdentityInterner::storage_uri(uri)
    }

    #[cfg(test)]
    pub(crate) fn document_file_id(&self, uri: &str) -> Option<LspFileId> {
        self.file_identity.file_id_for_uri(uri)
    }

    pub(crate) fn document_storage_uri_for_file_id(&self, file_id: LspFileId) -> Option<&str> {
        self.file_identity.storage_uri_for_file_id(file_id)
    }

    pub(crate) fn document_for_file_id(&self, file_id: LspFileId) -> Option<&LspTextDocumentState> {
        self.documents.get(&file_id).map(Arc::as_ref)
    }

    pub(crate) fn insert_open_document_uri(&mut self, uri: &str) -> String {
        let (file_id, storage_uri) = self.file_identity.intern_uri(uri);
        self.open_document_uris.insert(file_id);
        storage_uri
    }

    pub(crate) fn remove_open_document_uri(&mut self, uri: &str) {
        if let Some(file_id) = self.file_identity.file_id_for_uri(uri) {
            self.open_document_uris.remove(&file_id);
        }
    }

    pub(crate) fn has_open_document_uri(&self, uri: &str) -> bool {
        self.file_identity
            .file_id_for_uri(uri)
            .is_some_and(|file_id| self.open_document_uris.contains(&file_id))
    }

    pub(crate) fn insert_document(&mut self, uri: &str, document: LspTextDocumentState) {
        let (file_id, _) = self.file_identity.intern_uri(uri);
        self.documents.insert(file_id, Arc::new(document));
    }

    pub(crate) fn remove_document_uri(&mut self, uri: &str) -> Option<LspTextDocumentState> {
        let file_id = self.file_identity.file_id_for_uri(uri)?;
        self.documents.remove(&file_id).map(Arc::unwrap_or_clone)
    }

    pub(crate) fn contains_document_uri(&self, uri: &str) -> bool {
        self.document(uri).is_some()
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
            resolution: self.resolution.clone(),
            cancelled_request_count: self.cancelled_request_ids.len(),
            workspace_style_index_exhausted_count: self.workspace_style_index_exhausted_count,
            workspace_index_pending_file_count: self.workspace_index_pending_file_count,
            external_sif_lock_read_count: self.external_sif_lock_read_count,
            external_sif_bridge_generation_count: self.external_sif_bridge_generation_count,
            document_count: self.document_count(),
            workspace_folder_count: self.workspace_folder_count(),
            configuration_change_count: self.configuration_change_count,
            watched_file_event_count: self.watched_file_changes.len(),
            cached_workspace_resolution_input_count: self
                .resolution
                .workspace_style_resolution_inputs
                .len(),
            documents: self
                .documents
                .values()
                .map(|document| (**document).clone())
                .collect(),
            workspace_folders: self.workspace_runtime_registry.folder_snapshots(),
            watched_file_changes: self.watched_file_changes.clone(),
        }
    }

    pub(crate) fn cascade_narrowing_substrate_memo_lock(
        &self,
    ) -> MutexGuard<'_, Option<LspCascadeNarrowingSubstrateMemo>> {
        self.cascade_narrowing_substrate_memo
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }

    /// RFC 0009 Pillar A (rfcs#67, slice A-min): build the copy-on-write read
    /// model for the dispatched query lane. Called on the loop thread at
    /// dispatch time; cost is O(documents) `Arc` pointer clones plus plain
    /// clones of the small settings/registry values — never a corpus deep clone.
    pub fn query_snapshot(&self) -> LspQuerySnapshotV0 {
        LspQuerySnapshotV0 {
            state: LspShellState {
                features: self.features.clone(),
                diagnostics: self.diagnostics.clone(),
                resolution: self.resolution.clone(),
                file_identity: self.file_identity.clone(),
                documents: self.documents.clone(),
                open_document_uris: self.open_document_uris.clone(),
                workspace_runtime_registry: self.workspace_runtime_registry.clone(),
                cascade_narrowing_substrate_memo: Arc::clone(
                    &self.cascade_narrowing_substrate_memo,
                ),
                ..LspShellState::default()
            },
        }
    }
}

/// RFC 0009 Pillar A (rfcs#67, slice A-min): immutable-enough read model for the
/// dispatched query lane (`textDocument/hover` + `textDocument/definition`).
///
/// Internally this is a partial [`LspShellState`] so the existing resolver chain
/// (`resolve_lsp_hover`/`resolve_lsp_definition` and every `&LspShellState`
/// callee under them) runs against it unchanged. Carried fields: the documents
/// map (`Arc` entries — pointer clones), `open_document_uris`, `features`,
/// `diagnostics` + `resolution` settings, `workspace_runtime_registry`, and the
/// SHARED cascade-narrowing memo handle. Loop-owned machinery is deliberately
/// left at `Default` and must stay loop-side: the tsgo process pool (touched
/// only on didOpen/didChange/configuration mutation paths), the disk-cache
/// breaker session and the salsa style-memo host (touched only on the
/// style-diagnostics resolve path, `resolve_style_diagnostics`), and the
/// cancellation registry (taken on the loop before dispatch). The
/// hover/definition resolver chain reads none of them, so a worker holding this
/// snapshot can never contend with a loop-side salsa `set_*`.
#[derive(Debug)]
pub struct LspQuerySnapshotV0 {
    pub(crate) state: LspShellState,
}

impl LspQuerySnapshotV0 {
    pub(crate) fn shell_state(&self) -> &LspShellState {
        &self.state
    }
}

// The dispatched query lane moves snapshots onto the worker thread; keep that
// property checked at compile time independent of the worker code shape.
const _: () = {
    const fn assert_send<T: Send>() {}
    assert_send::<LspQuerySnapshotV0>();
};
