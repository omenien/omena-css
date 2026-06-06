use crate::workspace_runtime_registry::WorkspaceRuntimeRegistry;
use omena_incremental::IncrementalCancellationRegistryV0;
use omena_query::{
    AnalyzedGraphV0, OmenaQueryExternalSifInputV0,
    OmenaQuerySourceSyntaxIndexV0 as SourceSyntaxIndex, OmenaQueryStylePackageManifestV0,
    OmenaQueryStyleResolutionInputsV0, ParserPositionV0, ParserRangeV0,
};
use omena_tsgo_client::TsgoWorkspaceProcessPoolV0;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspTextDocumentState {
    pub uri: String,
    pub workspace_folder_uri: Option<String>,
    pub language_id: String,
    pub version: i64,
    pub text: String,
    pub style_summary: Option<LspStyleDocumentSummary>,
    pub diagnostics_schedule_count: usize,
    pub optimizing_tier_feedback: Option<LspOptimizingTierFeedback>,
    #[serde(skip)]
    pub style_candidates: Vec<LspStyleHoverCandidate>,
    #[serde(skip)]
    pub(crate) source_syntax_index: SourceSyntaxIndex,
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
    /// External Sass-module SIF artifacts sourced from the lock/bridge (#32/#33). When non-empty
    /// the style-diagnostics path runs in `ExternalModuleModeV0::Sif`, which surfaces the
    /// boundary lattice and the `@omena-strict:` sigil (#35); empty preserves the legacy
    /// `Ignored` behaviour.
    #[serde(skip)]
    pub external_sifs: Vec<OmenaQueryExternalSifInputV0>,
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
    pub(crate) configuration_change_count: usize,
    pub(crate) documents: BTreeMap<String, LspTextDocumentState>,
    pub(crate) open_document_uris: BTreeSet<String>,
    pub(crate) workspace_runtime_registry: WorkspaceRuntimeRegistry,
    pub(crate) tsgo_workspace_process_pool: TsgoWorkspaceProcessPoolV0,
    pub(crate) watched_file_changes: Vec<LspWatchedFileChangeState>,
}

impl LspShellState {
    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    pub fn workspace_folder_count(&self) -> usize {
        self.workspace_runtime_registry.len()
    }

    pub fn document(&self, uri: &str) -> Option<&LspTextDocumentState> {
        let storage_uri = Self::document_storage_uri(uri);
        self.documents.get(storage_uri.as_str()).or_else(|| {
            self.documents
                .iter()
                .find(|(document_uri, _)| crate::protocol::file_uri_equivalent(document_uri, uri))
                .map(|(_, document)| document)
        })
    }

    pub(crate) fn document_mut(&mut self, uri: &str) -> Option<&mut LspTextDocumentState> {
        let storage_uri = Self::document_storage_uri(uri);
        if self.documents.contains_key(storage_uri.as_str()) {
            return self.documents.get_mut(storage_uri.as_str());
        }
        let equivalent_uri = self
            .documents
            .keys()
            .find(|document_uri| crate::protocol::file_uri_equivalent(document_uri, uri))
            .cloned();
        equivalent_uri.and_then(|document_uri| self.documents.get_mut(document_uri.as_str()))
    }

    pub(crate) fn document_storage_uri(uri: &str) -> String {
        crate::protocol::canonical_file_uri(uri).unwrap_or_else(|| uri.to_string())
    }

    pub(crate) fn insert_open_document_uri(&mut self, uri: &str) -> String {
        let storage_uri = Self::document_storage_uri(uri);
        self.remove_open_document_uri(uri);
        self.open_document_uris.insert(storage_uri.clone());
        storage_uri
    }

    pub(crate) fn remove_open_document_uri(&mut self, uri: &str) {
        let storage_uri = Self::document_storage_uri(uri);
        self.open_document_uris.remove(storage_uri.as_str());
        let equivalent_uris = self
            .open_document_uris
            .iter()
            .filter(|candidate| crate::protocol::file_uri_equivalent(candidate, uri))
            .cloned()
            .collect::<Vec<_>>();
        for candidate in equivalent_uris {
            self.open_document_uris.remove(candidate.as_str());
        }
    }

    pub(crate) fn has_open_document_uri(&self, uri: &str) -> bool {
        let storage_uri = Self::document_storage_uri(uri);
        self.open_document_uris.contains(storage_uri.as_str())
            || self
                .open_document_uris
                .iter()
                .any(|candidate| crate::protocol::file_uri_equivalent(candidate, uri))
    }

    pub(crate) fn insert_document(&mut self, uri: &str, document: LspTextDocumentState) {
        let storage_uri = Self::document_storage_uri(uri);
        self.remove_document_uri(uri);
        self.documents.insert(storage_uri, document);
    }

    pub(crate) fn remove_document_uri(&mut self, uri: &str) -> Option<LspTextDocumentState> {
        let storage_uri = Self::document_storage_uri(uri);
        let removed = self.documents.remove(storage_uri.as_str());
        let equivalent_uris = self
            .documents
            .keys()
            .filter(|candidate| crate::protocol::file_uri_equivalent(candidate, uri))
            .cloned()
            .collect::<Vec<_>>();
        for candidate in equivalent_uris {
            self.documents.remove(candidate.as_str());
        }
        removed
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
            document_count: self.document_count(),
            workspace_folder_count: self.workspace_folder_count(),
            configuration_change_count: self.configuration_change_count,
            watched_file_event_count: self.watched_file_changes.len(),
            cached_workspace_resolution_input_count: self
                .resolution
                .workspace_style_resolution_inputs
                .len(),
            documents: self.documents.values().cloned().collect(),
            workspace_folders: self.workspace_runtime_registry.folder_snapshots(),
            watched_file_changes: self.watched_file_changes.clone(),
        }
    }
}
