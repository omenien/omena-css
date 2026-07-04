//! RFC 0009 Pillar B (rfcs#65), stage 1: the salsa-backed memoized
//! style-diagnostics query layer.
//!
//! The workspace diagnostics entry point commits one selector graph per
//! workspace revision. Per-file texts are salsa inputs, so an unchanged corpus
//! revalidates instead of recomputing, and a single-file edit re-runs only
//! queries whose inputs actually changed. Byte-identity with the straight-line
//! evaluator is guarded by `omena-diff-test`'s cache-equivalence oracle over
//! warm rounds and edit sequences.
//!
//! The host owns the shared Omena salsa database on the LSP loop thread: all
//! `set_*` happen there (the salsa pending-write contract), and fixed-revision
//! `StorageHandle` read views are rebuilt on worker threads for parallel
//! diagnostics.

use super::cross_file_summary::{
    summarize_omena_query_cross_file_summary_from_module_interfaces,
    summarize_omena_query_workspace_cross_file_summary_from_module_interfaces,
    summarize_omena_query_workspace_cross_file_summary_from_style_summary,
};
use super::diagnostics::{
    OmenaQueryExternalSifResolutionContext,
    collect_omena_query_workspace_diagnostics_substrate_from_committed_graph,
    promote_sif_backed_external_edges,
    summarize_omena_query_sass_module_resolution_identity_diagnostics_for_workspace_from_resolution,
};
use super::*;
pub type OmenaQueryStyleMemoDatabaseV0 = OmenaSalsaDatabaseV0;
use salsa::Setter;
use std::collections::{BTreeMap, BTreeSet};

#[cfg(any(test, feature = "test-support"))]
mod style_fact_entry_probe {
    use std::cell::RefCell;
    use std::collections::BTreeSet;

    thread_local! {
        static RUN_PATHS: RefCell<BTreeSet<String>> = const { RefCell::new(BTreeSet::new()) };
    }

    pub(super) fn record(style_path: &str) {
        RUN_PATHS.with(|paths| {
            paths.borrow_mut().insert(style_path.to_string());
        });
    }

    pub(super) fn reset() {
        RUN_PATHS.with(|paths| paths.borrow_mut().clear());
    }

    pub(super) fn read() -> BTreeSet<String> {
        RUN_PATHS.with(|paths| paths.borrow().clone())
    }
}

#[cfg(any(test, feature = "test-support"))]
mod module_interface_projection_probe {
    use std::cell::RefCell;
    use std::collections::BTreeSet;

    thread_local! {
        static RUN_PATHS: RefCell<BTreeSet<String>> = const { RefCell::new(BTreeSet::new()) };
    }

    pub(super) fn record(style_path: &str) {
        RUN_PATHS.with(|paths| {
            paths.borrow_mut().insert(style_path.to_string());
        });
    }

    pub(super) fn reset() {
        RUN_PATHS.with(|paths| paths.borrow_mut().clear());
    }

    pub(super) fn read() -> BTreeSet<String> {
        RUN_PATHS.with(|paths| paths.borrow().clone())
    }
}

#[cfg(any(test, feature = "test-support"))]
mod css_modules_import_edge_resolution_probe {
    use std::cell::RefCell;
    use std::collections::BTreeSet;

    thread_local! {
        static RUN_PATHS: RefCell<BTreeSet<String>> = const { RefCell::new(BTreeSet::new()) };
    }

    pub(super) fn record(style_path: &str) {
        RUN_PATHS.with(|paths| {
            paths.borrow_mut().insert(style_path.to_string());
        });
    }

    pub(super) fn reset() {
        RUN_PATHS.with(|paths| paths.borrow_mut().clear());
    }

    pub(super) fn read() -> BTreeSet<String> {
        RUN_PATHS.with(|paths| paths.borrow().clone())
    }
}

#[cfg(any(test, feature = "test-support"))]
mod sass_module_edge_resolution_probe {
    use std::cell::RefCell;
    use std::collections::BTreeSet;

    thread_local! {
        static RUN_PATHS: RefCell<BTreeSet<String>> = const { RefCell::new(BTreeSet::new()) };
    }

    pub(super) fn record(style_path: &str) {
        RUN_PATHS.with(|paths| {
            paths.borrow_mut().insert(style_path.to_string());
        });
    }

    pub(super) fn reset() {
        RUN_PATHS.with(|paths| paths.borrow_mut().clear());
    }

    pub(super) fn read() -> BTreeSet<String> {
        RUN_PATHS.with(|paths| paths.borrow().clone())
    }
}

#[cfg(feature = "test-support")]
pub fn reset_style_fact_entry_probe_for_test() {
    style_fact_entry_probe::reset();
}

#[cfg(feature = "test-support")]
pub fn read_style_fact_entry_probe_for_test() -> BTreeSet<String> {
    style_fact_entry_probe::read()
}

#[cfg(feature = "test-support")]
pub fn reset_module_interface_projection_probe_for_test() {
    module_interface_projection_probe::reset();
}

#[cfg(feature = "test-support")]
pub fn read_module_interface_projection_probe_for_test() -> BTreeSet<String> {
    module_interface_projection_probe::read()
}

#[cfg(any(test, feature = "test-support"))]
pub fn reset_css_modules_import_edge_resolution_probe_for_test() {
    css_modules_import_edge_resolution_probe::reset();
}

#[cfg(any(test, feature = "test-support"))]
pub fn read_css_modules_import_edge_resolution_probe_for_test() -> BTreeSet<String> {
    css_modules_import_edge_resolution_probe::read()
}

#[cfg(any(test, feature = "test-support"))]
pub fn reset_sass_module_edge_resolution_probe_for_test() {
    sass_module_edge_resolution_probe::reset();
}

#[cfg(any(test, feature = "test-support"))]
pub fn read_sass_module_edge_resolution_probe_for_test() -> BTreeSet<String> {
    sass_module_edge_resolution_probe::read()
}

#[cfg(any(test, feature = "test-support"))]
thread_local! {
    static COMMITTED_STYLE_SEMANTIC_GRAPH_COMPUTES: std::cell::Cell<u64> =
        const { std::cell::Cell::new(0) };
    static CSS_MODULES_CROSS_FILE_RESOLUTION_COMPUTES: std::cell::Cell<u64> =
        const { std::cell::Cell::new(0) };
}

#[cfg(any(test, feature = "test-support"))]
pub fn reset_committed_style_semantic_graph_compute_count_for_test() {
    COMMITTED_STYLE_SEMANTIC_GRAPH_COMPUTES.with(|count| count.set(0));
}

#[cfg(any(test, feature = "test-support"))]
pub fn read_committed_style_semantic_graph_compute_count_for_test() -> u64 {
    COMMITTED_STYLE_SEMANTIC_GRAPH_COMPUTES.with(|count| count.get())
}

#[cfg(any(test, feature = "test-support"))]
fn record_committed_style_semantic_graph_compute_for_test() {
    COMMITTED_STYLE_SEMANTIC_GRAPH_COMPUTES.with(|count| {
        count.set(count.get() + 1);
    });
}

#[cfg(any(test, feature = "test-support"))]
pub fn reset_css_modules_cross_file_resolution_compute_count_for_test() {
    CSS_MODULES_CROSS_FILE_RESOLUTION_COMPUTES.with(|count| count.set(0));
}

#[cfg(any(test, feature = "test-support"))]
pub fn read_css_modules_cross_file_resolution_compute_count_for_test() -> u64 {
    CSS_MODULES_CROSS_FILE_RESOLUTION_COMPUTES.with(|count| count.get())
}

#[cfg(any(test, feature = "test-support"))]
fn record_css_modules_cross_file_resolution_compute_for_test() {
    CSS_MODULES_CROSS_FILE_RESOLUTION_COMPUTES.with(|count| {
        count.set(count.get() + 1);
    });
}

/// One style file of the open-document corpus.
#[salsa::input]
pub struct OmenaQueryStyleFileInputV0 {
    #[returns(ref)]
    pub style_path: String,
    #[returns(ref)]
    pub style_source: String,
}

/// The full narrowing-input set the workspace diagnostics entry point reads.
/// Plain-data fields are set wholesale when they change; `files` carries the
/// per-file entities so an edit bumps only the changed file's input.
#[salsa::input]
pub struct OmenaQueryStyleWorkspaceInputV0 {
    #[returns(ref)]
    pub files: Vec<OmenaQueryStyleFileInputV0>,
    #[returns(ref)]
    pub source_documents: Vec<OmenaQuerySourceDocumentInputV0>,
    #[returns(ref)]
    pub package_manifests: Vec<OmenaQueryStylePackageManifestV0>,
    #[returns(ref)]
    pub external_sifs: Vec<OmenaQueryExternalSifInputV0>,
    #[returns(ref)]
    pub resolution_inputs: OmenaQueryStyleResolutionInputsV0,
}

/// One committed selector, many worker-side read views. Produced by
/// [`OmenaQueryStyleMemoHostV0::sync_workspace_for_parallel_resolve`] after the
/// host commits the wave and builds an independent selector read database; the
/// embedded `handle` pins that selector snapshot and is `Send`, so a parallel
/// wave rebuilds per-worker views via
/// [`OmenaQueryStyleMemoDatabaseV0::from_handle`].
pub struct OmenaQueryStyleParallelResolveSyncV0 {
    /// Fixed-revision database handle: clone per worker, drop with the wave.
    pub handle: salsa::StorageHandle<OmenaQueryStyleMemoDatabaseV0>,
    /// The synced workspace input entity (`Copy` salsa id).
    pub workspace: OmenaQueryStyleWorkspaceInputV0,
    /// `(style_path, file input entity)` for every corpus member, in corpus
    /// order, so callers map targets onto input ids without re-entering the
    /// host.
    pub files: Vec<(String, OmenaQueryStyleFileInputV0)>,
    pub committed_graph: OmenaQueryCommittedStyleSemanticGraphV0,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OmenaQueryCommittedStyleSemanticGraphV0 {
    style_fact_entries: Vec<OmenaQueryStyleFactEntry>,
    pub style_cross_file_summary: OmenaQueryCrossFileSummaryV0,
    pub cross_file_summary: OmenaQueryCrossFileSummaryV0,
    pub css_modules_resolution: OmenaQueryCssModulesCrossFileResolutionV0,
    pub sass_module_resolution: OmenaQuerySassModuleCrossFileResolutionV0,
    pub sass_module_resolution_without_manifests: OmenaQuerySassModuleCrossFileResolutionV0,
    pub sass_module_resolution_without_path_mappings: OmenaQuerySassModuleCrossFileResolutionV0,
    pub sass_module_resolution_with_external_sifs: OmenaQuerySassModuleCrossFileResolutionV0,
}

pub struct OmenaQueryStyleRevisionSelectorV0 {
    revision: IncrementalRevisionV0,
    db: OmenaQueryStyleMemoDatabaseV0,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    files: Vec<(String, OmenaQueryStyleFileInputV0)>,
    files_by_path: BTreeMap<String, OmenaQueryStyleFileInputV0>,
    committed_graph: OmenaQueryCommittedStyleSemanticGraphV0,
}

impl std::fmt::Debug for OmenaQueryStyleRevisionSelectorV0 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OmenaQueryStyleRevisionSelectorV0")
            .field("revision", &self.revision)
            .field("file_count", &self.files.len())
            .finish()
    }
}

impl OmenaQueryStyleRevisionSelectorV0 {
    pub fn revision(&self) -> IncrementalRevisionV0 {
        self.revision
    }

    pub fn workspace_style_diagnostics(
        &self,
        target_style_path: &str,
    ) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
        self.workspace_style_diagnostics_with_external_mode(
            target_style_path,
            OmenaQueryExternalModuleModeV0::Auto,
        )
    }

    pub fn workspace_style_diagnostics_with_external_mode(
        &self,
        target_style_path: &str,
        external_mode: OmenaQueryExternalModuleModeV0,
    ) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
        self.workspace_style_diagnostics_with_external_mode_and_suppression_mode(
            target_style_path,
            external_mode,
            OmenaQueryDiagnosticSuppressionModeV0::Apply,
        )
    }

    pub fn workspace_style_diagnostics_with_external_mode_and_suppression_mode(
        &self,
        target_style_path: &str,
        external_mode: OmenaQueryExternalModuleModeV0,
        suppression_mode: OmenaQueryDiagnosticSuppressionModeV0,
    ) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
        let target = self.files_by_path.get(target_style_path).copied()?;
        resolve_committed_workspace_style_diagnostics_from_view_with_external_mode_and_suppression_mode(
            &self.db,
            self.workspace,
            target,
            &self.committed_graph,
            external_mode,
            suppression_mode,
        )
    }

    pub fn committed_style_semantic_graph(&self) -> &OmenaQueryCommittedStyleSemanticGraphV0 {
        &self.committed_graph
    }

    pub fn workspace_cross_file_summary(&self) -> &OmenaQueryCrossFileSummaryV0 {
        &self.committed_graph.cross_file_summary
    }

    pub fn css_modules_cross_file_resolution(&self) -> &OmenaQueryCssModulesCrossFileResolutionV0 {
        &self.committed_graph.css_modules_resolution
    }

    pub fn sass_module_cross_file_resolution(&self) -> &OmenaQuerySassModuleCrossFileResolutionV0 {
        &self.committed_graph.sass_module_resolution
    }

    pub fn sass_module_resolution_identity_diagnostics_for_workspace(
        &self,
        target_style_path: &str,
    ) -> Vec<OmenaQueryStyleDiagnosticV0> {
        let style_sources = self
            .files
            .iter()
            .map(|(style_path, file)| OmenaQueryStyleSourceInputV0 {
                style_path: style_path.clone(),
                style_source: file.style_source(&self.db).clone(),
            })
            .collect::<Vec<_>>();
        summarize_omena_query_sass_module_resolution_identity_diagnostics_for_workspace_from_resolution(
            target_style_path,
            style_sources.as_slice(),
            &self.committed_graph.sass_module_resolution,
        )
    }

    pub fn style_cascade_narrowing_substrate(&self) -> OmenaQueryStyleCascadeNarrowingSubstrateV0 {
        let entries = self
            .committed_graph
            .style_fact_entries
            .iter()
            .map(|entry| StyleCascadeNarrowingSubstrateEntry {
                style_path: entry.style_path.clone(),
                facts: entry.facts.clone(),
                declarations: cascade_checker::collect_query_checker_cascade_declarations(
                    entry.style_source.as_str(),
                ),
            })
            .collect();
        OmenaQueryStyleCascadeNarrowingSubstrateV0 {
            entries,
            resolution: self
                .committed_graph
                .sass_module_resolution_with_external_sifs
                .clone(),
        }
    }

    pub fn style_completion_for_workspace_file(
        &self,
        target_style_path: &str,
        position: ParserPositionV0,
    ) -> OmenaQueryCompletionAtPositionV0 {
        let style_sources = self
            .files
            .iter()
            .map(|(style_path, file)| OmenaQueryStyleSourceInputV0 {
                style_path: style_path.clone(),
                style_source: file.style_source(&self.db).clone(),
            })
            .collect::<Vec<_>>();
        let substrate = self.style_cascade_narrowing_substrate();
        summarize_omena_query_style_completion_for_workspace_file_with_substrate(
            target_style_path,
            style_sources.as_slice(),
            self.workspace.package_manifests(&self.db).as_slice(),
            self.workspace.external_sifs(&self.db).as_slice(),
            self.workspace.resolution_inputs(&self.db),
            &substrate,
            position,
        )
    }

    pub fn style_semantic_graph_batch(
        &self,
        input: &EngineInputV2,
        package_manifests: &[OmenaQueryStylePackageManifestV0],
    ) -> OmenaQueryStyleSemanticGraphBatchOutputV0 {
        let style_sources = self
            .files
            .iter()
            .map(|(style_path, file)| OmenaQueryStyleSourceInputV0 {
                style_path: style_path.clone(),
                style_source: file.style_source(&self.db).clone(),
            })
            .collect::<Vec<_>>();
        summarize_omena_query_style_semantic_graph_batch_from_committed_parts(
            style_sources.as_slice(),
            self.committed_graph.style_fact_entries.as_slice(),
            input,
            package_manifests,
            self.committed_graph.style_cross_file_summary.clone(),
            self.committed_graph.css_modules_resolution.clone(),
            self.committed_graph.sass_module_resolution.clone(),
        )
    }

    pub fn into_parallel_resolve_sync(self) -> OmenaQueryStyleParallelResolveSyncV0 {
        OmenaQueryStyleParallelResolveSyncV0 {
            handle: self.db.handle(),
            workspace: self.workspace,
            files: self.files,
            committed_graph: self.committed_graph,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OmenaQueryStyleWorkspaceTransactionErrorV0 {
    DuplicateStylePath { style_path: String },
    UnregisteredStylePath { style_path: String },
}

pub struct OmenaQueryStyleWorkspaceTransactionCommitV0 {
    pub revision: IncrementalRevisionV0,
    pub workspace: OmenaQueryStyleWorkspaceInputV0,
    pub files: Vec<(String, OmenaQueryStyleFileInputV0)>,
    pub changed_style_paths: BTreeSet<String>,
    pub selector: OmenaQueryStyleRevisionSelectorV0,
}

struct OmenaQueryStyleWorkspaceTransactionCoreCommitV0 {
    revision: IncrementalRevisionV0,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    files: Vec<(String, OmenaQueryStyleFileInputV0)>,
    changed_style_paths: BTreeSet<String>,
    style_sources: Vec<OmenaQueryStyleSourceInputV0>,
    source_documents: Vec<OmenaQuerySourceDocumentInputV0>,
    package_manifests: Vec<OmenaQueryStylePackageManifestV0>,
    external_sifs: Vec<OmenaQueryExternalSifInputV0>,
    resolution_inputs: OmenaQueryStyleResolutionInputsV0,
    committed_graph: OmenaQueryCommittedStyleSemanticGraphV0,
}

pub struct OmenaQueryStyleDiagnosticsWithSelectorV0 {
    pub diagnostics: OmenaQueryStyleDiagnosticsForFileV0,
    pub selector: OmenaQueryStyleRevisionSelectorV0,
}

/// Loop-owned transaction over the memo host's registered workspace files.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OmenaQueryStyleWorkspaceTransactionV0 {
    registered_style_paths: BTreeSet<String>,
    style_sources: Vec<OmenaQueryStyleSourceInputV0>,
    source_documents: Vec<OmenaQuerySourceDocumentInputV0>,
    package_manifests: Vec<OmenaQueryStylePackageManifestV0>,
    external_sifs: Vec<OmenaQueryExternalSifInputV0>,
    resolution_inputs: OmenaQueryStyleResolutionInputsV0,
}

impl OmenaQueryStyleWorkspaceTransactionV0 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_style_file(&mut self, style_path: impl Into<String>) -> &mut Self {
        self.registered_style_paths.insert(style_path.into());
        self
    }

    pub fn register_style_paths<I, S>(&mut self, style_paths: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for style_path in style_paths {
            self.register_style_file(style_path);
        }
        self
    }

    pub fn register_style_sources(
        &mut self,
        style_sources: &[OmenaQueryStyleSourceInputV0],
    ) -> &mut Self {
        for source in style_sources {
            self.register_style_file(source.style_path.clone());
        }
        self
    }

    pub fn set_workspace_inputs(
        &mut self,
        style_sources: &[OmenaQueryStyleSourceInputV0],
        source_documents: &[OmenaQuerySourceDocumentInputV0],
        package_manifests: &[OmenaQueryStylePackageManifestV0],
        external_sifs: &[OmenaQueryExternalSifInputV0],
        resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    ) -> &mut Self {
        self.style_sources = style_sources.to_vec();
        self.source_documents = source_documents.to_vec();
        self.package_manifests = package_manifests.to_vec();
        self.external_sifs = external_sifs.to_vec();
        self.resolution_inputs = resolution_inputs.clone();
        self
    }

    pub fn commit_revision(
        self,
        host: &mut OmenaQueryStyleMemoHostV0,
    ) -> Result<
        OmenaQueryStyleWorkspaceTransactionCommitV0,
        OmenaQueryStyleWorkspaceTransactionErrorV0,
    > {
        host.commit_workspace_transaction(self)
    }
}

pub fn resolve_committed_workspace_style_diagnostics_from_view(
    db: &OmenaQueryStyleMemoDatabaseV0,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    target: OmenaQueryStyleFileInputV0,
    committed_graph: &OmenaQueryCommittedStyleSemanticGraphV0,
) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
    resolve_committed_workspace_style_diagnostics_from_view_with_external_mode(
        db,
        workspace,
        target,
        committed_graph,
        OmenaQueryExternalModuleModeV0::Auto,
    )
}

pub fn resolve_committed_workspace_style_diagnostics_from_view_with_identity_index(
    db: &OmenaQueryStyleMemoDatabaseV0,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    target: OmenaQueryStyleFileInputV0,
    committed_graph: &OmenaQueryCommittedStyleSemanticGraphV0,
    resolver_identity_index: &OmenaResolverStyleModuleConfirmationIdentityIndexV0,
) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
    resolve_committed_workspace_style_diagnostics_from_view_with_external_mode_and_suppression_mode_and_identity_index(
        db,
        workspace,
        target,
        committed_graph,
        OmenaQueryExternalModuleModeV0::Auto,
        OmenaQueryDiagnosticSuppressionModeV0::Apply,
        Some(resolver_identity_index),
    )
}

/// Target-INDEPENDENT per-wave state (rfcs#111, the first C1 slice): the
/// corpus snapshot and the diagnostics substrate — all 137-entry fact/
/// resolution clones — hoisted out of the per-target resolve. Opaque to
/// callers; build once per wave, share behind an `Arc`, and resolve each
/// target through the `_and_wave_substrate` variant below. Byte-identical
/// to the per-target build by construction (same collector, same inputs).
pub struct OmenaQueryCommittedWaveSubstrateV0 {
    corpus: Vec<OmenaQueryStyleSourceInputV0>,
    substrate: OmenaQueryWorkspaceDiagnosticsSubstrateV0,
    shared_passes: crate::style::diagnostics::OmenaQueryWorkspaceSharedPassProductsV0,
}

pub fn prepare_committed_workspace_wave_substrate(
    db: &OmenaQueryStyleMemoDatabaseV0,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    committed_graph: &OmenaQueryCommittedStyleSemanticGraphV0,
    resolver_identity_index: Option<&OmenaResolverStyleModuleConfirmationIdentityIndexV0>,
) -> OmenaQueryCommittedWaveSubstrateV0 {
    let corpus = workspace
        .files(db)
        .iter()
        .map(|file| OmenaQueryStyleSourceInputV0 {
            style_path: file.style_path(db).clone(),
            style_source: file.style_source(db).clone(),
        })
        .collect::<Vec<_>>();
    let substrate = collect_omena_query_workspace_diagnostics_substrate_from_committed_graph(
        committed_graph.style_fact_entries.clone(),
        &committed_graph.css_modules_resolution,
        &committed_graph.sass_module_resolution,
        &committed_graph.sass_module_resolution_without_manifests,
        &committed_graph.sass_module_resolution_without_path_mappings,
        &committed_graph.sass_module_resolution_with_external_sifs,
    );
    // rfcs#111 C1 slice 2: the target-independent pass cores, computed once
    // per wave. Arguments mirror the per-target dispatch exactly — the
    // committed arm passes classname_transform = None (as the per-target
    // wrapper does) and the same resolution inputs and identity index the
    // targets will resolve with.
    let source_documents = workspace.source_documents(db);
    let package_manifests = workspace.package_manifests(db);
    let resolution_inputs = workspace.resolution_inputs(db);
    let shared_passes = crate::style::diagnostics::OmenaQueryWorkspaceSharedPassProductsV0 {
        unused_selector: crate::style::diagnostics::collect_omena_query_unused_selector_shared(
            &substrate.style_fact_entries,
            source_documents.as_slice(),
            package_manifests.as_slice(),
            None,
            resolution_inputs.bundler_path_mappings.as_slice(),
            resolution_inputs.tsconfig_path_mappings.as_slice(),
            resolution_inputs.disk_style_path_identities.as_slice(),
            resolver_identity_index,
        ),
        inline_style_overrides_by_style: Some(
            crate::style::diagnostics::collect_omena_query_inline_style_runtime_overrides_by_style(
                corpus.as_slice(),
                source_documents.as_slice(),
                resolution_inputs,
                resolver_identity_index,
            ),
        ),
        #[cfg(feature = "hypergraph-ifds")]
        cross_file_scc_report: Some(
            crate::style::diagnostics::collect_omena_query_unified_cross_file_scc_report_shared(
                corpus.as_slice(),
                source_documents.as_slice(),
                package_manifests.as_slice(),
                &substrate,
            ),
        ),
    };
    OmenaQueryCommittedWaveSubstrateV0 {
        corpus,
        substrate,
        shared_passes,
    }
}

pub fn resolve_committed_workspace_style_diagnostics_from_view_with_identity_index_and_wave_substrate(
    db: &OmenaQueryStyleMemoDatabaseV0,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    target: OmenaQueryStyleFileInputV0,
    wave_substrate: &OmenaQueryCommittedWaveSubstrateV0,
    resolver_identity_index: &OmenaResolverStyleModuleConfirmationIdentityIndexV0,
) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
    let target_style_path = target.style_path(db);
    let source_documents = workspace.source_documents(db);
    let package_manifests = workspace.package_manifests(db);
    let external_sifs = workspace.external_sifs(db);
    let resolution_inputs = workspace.resolution_inputs(db);
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs_and_suppression_mode_with_substrate_and_shared(
        target_style_path.as_str(),
        wave_substrate.corpus.as_slice(),
        source_documents.as_slice(),
        package_manifests.as_slice(),
        None,
        OmenaQueryExternalModuleModeV0::Auto,
        external_sifs.as_slice(),
        resolution_inputs,
        OmenaQueryDiagnosticSuppressionModeV0::Apply,
        &wave_substrate.substrate,
        Some(resolver_identity_index),
        Some(&wave_substrate.shared_passes),
    )
}

pub fn resolve_committed_workspace_style_diagnostics_from_view_with_external_mode(
    db: &OmenaQueryStyleMemoDatabaseV0,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    target: OmenaQueryStyleFileInputV0,
    committed_graph: &OmenaQueryCommittedStyleSemanticGraphV0,
    external_mode: OmenaQueryExternalModuleModeV0,
) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
    resolve_committed_workspace_style_diagnostics_from_view_with_external_mode_and_suppression_mode(
        db,
        workspace,
        target,
        committed_graph,
        external_mode,
        OmenaQueryDiagnosticSuppressionModeV0::Apply,
    )
}

pub fn resolve_committed_workspace_style_diagnostics_from_view_with_external_mode_and_suppression_mode(
    db: &OmenaQueryStyleMemoDatabaseV0,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    target: OmenaQueryStyleFileInputV0,
    committed_graph: &OmenaQueryCommittedStyleSemanticGraphV0,
    external_mode: OmenaQueryExternalModuleModeV0,
    suppression_mode: OmenaQueryDiagnosticSuppressionModeV0,
) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
    resolve_committed_workspace_style_diagnostics_from_view_with_external_mode_and_suppression_mode_and_identity_index(
        db,
        workspace,
        target,
        committed_graph,
        external_mode,
        suppression_mode,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn resolve_committed_workspace_style_diagnostics_from_view_with_external_mode_and_suppression_mode_and_identity_index(
    db: &OmenaQueryStyleMemoDatabaseV0,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    target: OmenaQueryStyleFileInputV0,
    committed_graph: &OmenaQueryCommittedStyleSemanticGraphV0,
    external_mode: OmenaQueryExternalModuleModeV0,
    suppression_mode: OmenaQueryDiagnosticSuppressionModeV0,
    resolver_identity_index: Option<&OmenaResolverStyleModuleConfirmationIdentityIndexV0>,
) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
    let target_style_path = target.style_path(db);
    let corpus = workspace
        .files(db)
        .iter()
        .map(|file| OmenaQueryStyleSourceInputV0 {
            style_path: file.style_path(db).clone(),
            style_source: file.style_source(db).clone(),
        })
        .collect::<Vec<_>>();
    let source_documents = workspace.source_documents(db);
    let package_manifests = workspace.package_manifests(db);
    let external_sifs = workspace.external_sifs(db);
    let resolution_inputs = workspace.resolution_inputs(db);
    let substrate = collect_omena_query_workspace_diagnostics_substrate_from_committed_graph(
        committed_graph.style_fact_entries.clone(),
        &committed_graph.css_modules_resolution,
        &committed_graph.sass_module_resolution,
        &committed_graph.sass_module_resolution_without_manifests,
        &committed_graph.sass_module_resolution_without_path_mappings,
        &committed_graph.sass_module_resolution_with_external_sifs,
    );
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs_and_suppression_mode_with_substrate(
        target_style_path.as_str(),
        corpus.as_slice(),
        source_documents.as_slice(),
        package_manifests.as_slice(),
        None,
        external_mode,
        external_sifs.as_slice(),
        resolution_inputs,
        suppression_mode,
        &substrate,
        resolver_identity_index,
    )
}

/// RFC 0009 Pillar B (#65) SLICE-3: the target-INDEPENDENT diagnostics substrate, hoisted into its
/// own workspace-keyed tracked query so N open targets share ONE substrate build per revision
/// instead of rebuilding it per `(workspace, target)`. `returns(ref)` hands the per-target query a
/// borrow, so the entries + resolution variants are not cloned per target. The arguments mirror the
/// monolith wrapper's inline build exactly, so the substrate is byte-identical either way.
#[salsa::tracked(returns(ref))]
fn memo_workspace_diagnostics_substrate(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> OmenaQueryWorkspaceDiagnosticsSubstrateV0 {
    let style_fact_entries = workspace
        .files(db)
        .iter()
        .map(|file| memo_style_fact_entry(db, *file))
        .collect::<Vec<_>>();
    let resolution_inputs = workspace.resolution_inputs(db);
    collect_omena_query_workspace_diagnostics_substrate_from_entries(
        style_fact_entries,
        workspace.package_manifests(db).as_slice(),
        workspace.external_sifs(db).as_slice(),
        resolution_inputs.bundler_path_mappings.as_slice(),
        resolution_inputs.tsconfig_path_mappings.as_slice(),
    )
}

#[salsa::tracked(returns(clone))]
fn memo_style_fact_entry(
    db: &dyn salsa::Database,
    file: OmenaQueryStyleFileInputV0,
) -> OmenaQueryStyleFactEntry {
    #[cfg(any(test, feature = "test-support"))]
    style_fact_entry_probe::record(file.style_path(db));
    collect_omena_query_style_fact_entry(file.style_path(db), file.style_source(db))
}

#[salsa::tracked(returns(clone))]
pub fn memo_module_interface_projection(
    db: &dyn salsa::Database,
    file: OmenaQueryStyleFileInputV0,
) -> OmenaQueryModuleInterfaceProjectionV0 {
    #[cfg(any(test, feature = "test-support"))]
    module_interface_projection_probe::record(file.style_path(db));
    module_interface_projection_for_query(&memo_style_fact_entry(db, file))
}

#[salsa::tracked(returns(clone))]
pub fn memo_css_modules_cross_file_resolution_from_module_interfaces(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> OmenaQueryCssModulesCrossFileResolutionV0 {
    #[cfg(any(test, feature = "test-support"))]
    record_css_modules_cross_file_resolution_compute_for_test();
    let module_interfaces = module_interfaces_for_workspace(db, workspace);
    let edges = style_paths_for_workspace(db, workspace)
        .into_iter()
        .flat_map(|style_path| {
            memo_css_modules_import_edge_resolutions_for_origin_from_module_interfaces(
                db, workspace, style_path,
            )
        })
        .collect::<Vec<_>>();
    summarize_css_modules_cross_file_resolution_from_module_interfaces_and_import_edges(
        module_interfaces.as_slice(),
        workspace.package_manifests(db).as_slice(),
        edges,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OmenaQueryModuleDependencySurfaceV0 {
    style_path: String,
    style_dependency_sources: Vec<String>,
}

#[salsa::tracked(returns(clone))]
fn memo_module_dependency_surface(
    db: &dyn salsa::Database,
    file: OmenaQueryStyleFileInputV0,
) -> OmenaQueryModuleDependencySurfaceV0 {
    let entry = memo_style_fact_entry(db, file);
    OmenaQueryModuleDependencySurfaceV0 {
        style_path: entry.style_path,
        style_dependency_sources: collect_style_module_dependency_sources_from_facts(&entry.facts),
    }
}

#[salsa::tracked(returns(clone))]
fn memo_css_modules_import_edge_resolutions_for_origin_from_module_interfaces(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    origin_style_path: String,
) -> Vec<OmenaQueryCssModulesImportEdgeResolutionV0> {
    #[cfg(any(test, feature = "test-support"))]
    css_modules_import_edge_resolution_probe::record(origin_style_path.as_str());
    let Some(origin_file) = file_for_style_path(db, workspace, origin_style_path.as_str()) else {
        return Vec::new();
    };
    let package_manifests = workspace.package_manifests(db);
    let origin = memo_module_interface_projection(db, origin_file);
    let available_style_paths = style_paths_for_workspace(db, workspace);
    let available_style_path_refs = available_style_paths
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let style_import_edges = style_import_reachability_edges_from_dependency_surfaces(
        module_dependency_surfaces_for_workspace(db, workspace).as_slice(),
        &available_style_path_refs,
        package_manifests.as_slice(),
    );
    let target_interfaces = origin
        .style_dependency_sources
        .iter()
        .filter_map(|source| {
            resolve_style_module_source(
                origin.style_path.as_str(),
                source,
                &available_style_path_refs,
                package_manifests.as_slice(),
            )
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter_map(|style_path| file_for_style_path(db, workspace, style_path.as_str()))
        .map(|file| memo_module_interface_projection(db, file))
        .collect::<Vec<_>>();
    summarize_css_modules_import_edge_resolutions_for_module_interface(
        &origin,
        target_interfaces.as_slice(),
        &available_style_path_refs,
        style_import_edges.as_slice(),
        package_manifests.as_slice(),
    )
}

#[salsa::tracked(returns(clone))]
pub fn memo_sass_module_cross_file_resolution_from_module_interfaces(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> OmenaQuerySassModuleCrossFileResolutionV0 {
    #[cfg(any(test, feature = "test-support"))]
    record_sass_module_resolution_internal_compute_for_test();
    let module_interfaces = module_interfaces_for_workspace(db, workspace);
    let edges = style_paths_for_workspace(db, workspace)
        .into_iter()
        .flat_map(|style_path| {
            memo_sass_module_edge_resolutions_for_origin_from_module_interfaces(
                db, workspace, style_path,
            )
        })
        .collect::<Vec<_>>();
    let configurable_names_by_path = sass_configurable_names_by_path_for_workspace(db, workspace);
    summarize_sass_module_cross_file_resolution_from_module_interfaces_and_edges(
        module_interfaces.as_slice(),
        edges,
        &configurable_names_by_path,
    )
}

#[salsa::tracked(returns(clone))]
fn memo_sass_configurable_variable_names_from_module_interface(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    style_path: String,
) -> BTreeSet<String> {
    let available_style_paths = style_paths_for_workspace(db, workspace);
    let available_style_path_refs = available_style_paths
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let package_manifests = workspace.package_manifests(db);
    let resolution_inputs = workspace.resolution_inputs(db);
    let mut visiting = BTreeSet::new();
    sass_configurable_variable_names_for_module_interface_tracked(
        db,
        workspace,
        style_path.as_str(),
        &available_style_path_refs,
        package_manifests.as_slice(),
        resolution_inputs.bundler_path_mappings.as_slice(),
        resolution_inputs.tsconfig_path_mappings.as_slice(),
        &mut visiting,
    )
}

#[salsa::tracked(returns(clone))]
fn memo_sass_configurable_variable_names_without_manifests_from_module_interface(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    style_path: String,
) -> BTreeSet<String> {
    let available_style_paths = style_paths_for_workspace(db, workspace);
    let available_style_path_refs = available_style_paths
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let resolution_inputs = workspace.resolution_inputs(db);
    let mut visiting = BTreeSet::new();
    sass_configurable_variable_names_for_module_interface_tracked(
        db,
        workspace,
        style_path.as_str(),
        &available_style_path_refs,
        &[],
        resolution_inputs.bundler_path_mappings.as_slice(),
        resolution_inputs.tsconfig_path_mappings.as_slice(),
        &mut visiting,
    )
}

#[salsa::tracked(returns(clone))]
fn memo_sass_configurable_variable_names_without_path_mappings_from_module_interface(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    style_path: String,
) -> BTreeSet<String> {
    let available_style_paths = style_paths_for_workspace(db, workspace);
    let available_style_path_refs = available_style_paths
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let package_manifests = workspace.package_manifests(db);
    let mut visiting = BTreeSet::new();
    sass_configurable_variable_names_for_module_interface_tracked(
        db,
        workspace,
        style_path.as_str(),
        &available_style_path_refs,
        package_manifests.as_slice(),
        &[],
        &[],
        &mut visiting,
    )
}

#[salsa::tracked(returns(clone))]
fn memo_sass_module_edge_resolutions_for_origin_from_module_interfaces(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    origin_style_path: String,
) -> Vec<OmenaQuerySassModuleEdgeResolutionV0> {
    #[cfg(any(test, feature = "test-support"))]
    sass_module_edge_resolution_probe::record(origin_style_path.as_str());
    let Some(origin_file) = file_for_style_path(db, workspace, origin_style_path.as_str()) else {
        return Vec::new();
    };
    let package_manifests = workspace.package_manifests(db);
    let resolution_inputs = workspace.resolution_inputs(db);
    let origin = memo_module_interface_projection(db, origin_file);
    let available_style_paths = style_paths_for_workspace(db, workspace);
    let available_style_path_refs = available_style_paths
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let resolver_available_style_paths = resolver_style_paths_for_workspace(db, workspace);
    let resolver_available_style_path_refs = resolver_available_style_paths
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    summarize_sass_module_edge_resolutions_for_module_interface(
        &origin,
        &available_style_path_refs,
        &resolver_available_style_path_refs,
        package_manifests.as_slice(),
        resolution_inputs.bundler_path_mappings.as_slice(),
        resolution_inputs.tsconfig_path_mappings.as_slice(),
        |target_style_path| {
            memo_sass_configurable_variable_names_from_module_interface(
                db,
                workspace,
                target_style_path.to_string(),
            )
        },
    )
}

#[salsa::tracked(returns(clone))]
fn memo_sass_module_edge_resolutions_without_manifests_for_origin_from_module_interfaces(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    origin_style_path: String,
) -> Vec<OmenaQuerySassModuleEdgeResolutionV0> {
    #[cfg(any(test, feature = "test-support"))]
    sass_module_edge_resolution_probe::record(origin_style_path.as_str());
    let Some(origin_file) = file_for_style_path(db, workspace, origin_style_path.as_str()) else {
        return Vec::new();
    };
    let resolution_inputs = workspace.resolution_inputs(db);
    let origin = memo_module_interface_projection(db, origin_file);
    let available_style_paths = style_paths_for_workspace(db, workspace);
    let available_style_path_refs = available_style_paths
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let resolver_available_style_paths = resolver_style_paths_for_workspace(db, workspace);
    let resolver_available_style_path_refs = resolver_available_style_paths
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    summarize_sass_module_edge_resolutions_for_module_interface(
        &origin,
        &available_style_path_refs,
        &resolver_available_style_path_refs,
        &[],
        resolution_inputs.bundler_path_mappings.as_slice(),
        resolution_inputs.tsconfig_path_mappings.as_slice(),
        |target_style_path| {
            memo_sass_configurable_variable_names_without_manifests_from_module_interface(
                db,
                workspace,
                target_style_path.to_string(),
            )
        },
    )
}

#[salsa::tracked(returns(clone))]
fn memo_sass_module_edge_resolutions_without_path_mappings_for_origin_from_module_interfaces(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    origin_style_path: String,
) -> Vec<OmenaQuerySassModuleEdgeResolutionV0> {
    #[cfg(any(test, feature = "test-support"))]
    sass_module_edge_resolution_probe::record(origin_style_path.as_str());
    let Some(origin_file) = file_for_style_path(db, workspace, origin_style_path.as_str()) else {
        return Vec::new();
    };
    let package_manifests = workspace.package_manifests(db);
    let origin = memo_module_interface_projection(db, origin_file);
    let available_style_paths = style_paths_for_workspace(db, workspace);
    let available_style_path_refs = available_style_paths
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let resolver_available_style_paths = resolver_style_paths_for_workspace(db, workspace);
    let resolver_available_style_path_refs = resolver_available_style_paths
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    summarize_sass_module_edge_resolutions_for_module_interface(
        &origin,
        &available_style_path_refs,
        &resolver_available_style_path_refs,
        package_manifests.as_slice(),
        &[],
        &[],
        |target_style_path| {
            memo_sass_configurable_variable_names_without_path_mappings_from_module_interface(
                db,
                workspace,
                target_style_path.to_string(),
            )
        },
    )
}

#[salsa::tracked(returns(clone))]
pub fn memo_sass_module_cross_file_resolution_without_manifests_from_module_interfaces(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> OmenaQuerySassModuleCrossFileResolutionV0 {
    #[cfg(any(test, feature = "test-support"))]
    record_sass_module_resolution_internal_compute_for_test();
    let module_interfaces = module_interfaces_for_workspace(db, workspace);
    let edges = style_paths_for_workspace(db, workspace)
        .into_iter()
        .flat_map(|style_path| {
            memo_sass_module_edge_resolutions_without_manifests_for_origin_from_module_interfaces(
                db, workspace, style_path,
            )
        })
        .collect::<Vec<_>>();
    let configurable_names_by_path =
        sass_configurable_names_without_manifests_by_path_for_workspace(db, workspace);
    summarize_sass_module_cross_file_resolution_from_module_interfaces_and_edges(
        module_interfaces.as_slice(),
        edges,
        &configurable_names_by_path,
    )
}

#[salsa::tracked(returns(clone))]
pub fn memo_sass_module_cross_file_resolution_without_path_mappings_from_module_interfaces(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> OmenaQuerySassModuleCrossFileResolutionV0 {
    #[cfg(any(test, feature = "test-support"))]
    record_sass_module_resolution_internal_compute_for_test();
    let module_interfaces = module_interfaces_for_workspace(db, workspace);
    let edges = style_paths_for_workspace(db, workspace)
        .into_iter()
        .flat_map(|style_path| {
            memo_sass_module_edge_resolutions_without_path_mappings_for_origin_from_module_interfaces(
                db, workspace, style_path,
            )
        })
        .collect::<Vec<_>>();
    let configurable_names_by_path =
        sass_configurable_names_without_path_mappings_by_path_for_workspace(db, workspace);
    summarize_sass_module_cross_file_resolution_from_module_interfaces_and_edges(
        module_interfaces.as_slice(),
        edges,
        &configurable_names_by_path,
    )
}

#[salsa::tracked(returns(clone))]
pub fn memo_sass_module_cross_file_resolution_with_external_sifs_from_module_interfaces(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> OmenaQuerySassModuleCrossFileResolutionV0 {
    let mut resolution =
        memo_sass_module_cross_file_resolution_from_module_interfaces(db, workspace);
    let resolution_inputs = workspace.resolution_inputs(db);
    promote_sif_backed_external_edges(
        &mut resolution,
        OmenaQueryExternalSifResolutionContext {
            package_manifests: workspace.package_manifests(db).as_slice(),
            bundler_path_mappings: resolution_inputs.bundler_path_mappings.as_slice(),
            tsconfig_path_mappings: resolution_inputs.tsconfig_path_mappings.as_slice(),
            external_sifs: workspace.external_sifs(db).as_slice(),
        },
    );
    resolution
}

#[salsa::tracked(returns(clone))]
fn memo_style_cross_file_summary_from_module_interfaces(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> OmenaQueryCrossFileSummaryV0 {
    let module_interfaces = module_interfaces_for_workspace(db, workspace);
    let css_modules_resolution =
        memo_css_modules_cross_file_resolution_from_module_interfaces(db, workspace);
    let sass_module_resolution =
        memo_sass_module_cross_file_resolution_from_module_interfaces(db, workspace);
    summarize_omena_query_cross_file_summary_from_module_interfaces(
        module_interfaces.as_slice(),
        &css_modules_resolution,
        &sass_module_resolution,
    )
}

#[salsa::tracked(returns(clone))]
fn memo_workspace_cross_file_summary_from_module_interfaces(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> OmenaQueryCrossFileSummaryV0 {
    let module_interfaces = module_interfaces_for_workspace(db, workspace);
    let style_cross_file_summary =
        memo_style_cross_file_summary_from_module_interfaces(db, workspace);
    summarize_omena_query_workspace_cross_file_summary_from_module_interfaces(
        module_interfaces.as_slice(),
        workspace.source_documents(db).as_slice(),
        workspace.package_manifests(db).as_slice(),
        style_cross_file_summary,
    )
}

#[salsa::tracked(returns(clone))]
fn memo_committed_style_semantic_graph_from_module_interfaces(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> OmenaQueryCommittedStyleSemanticGraphV0 {
    let style_fact_entries = style_fact_entries_for_workspace(db, workspace);
    let css_modules_resolution =
        memo_css_modules_cross_file_resolution_from_module_interfaces(db, workspace);
    let sass_module_resolution =
        memo_sass_module_cross_file_resolution_from_module_interfaces(db, workspace);
    let sass_module_resolution_without_manifests =
        memo_sass_module_cross_file_resolution_without_manifests_from_module_interfaces(
            db, workspace,
        );
    let sass_module_resolution_without_path_mappings =
        memo_sass_module_cross_file_resolution_without_path_mappings_from_module_interfaces(
            db, workspace,
        );
    let sass_module_resolution_with_external_sifs =
        memo_sass_module_cross_file_resolution_with_external_sifs_from_module_interfaces(
            db, workspace,
        );
    let style_cross_file_summary =
        memo_style_cross_file_summary_from_module_interfaces(db, workspace);
    let cross_file_summary =
        memo_workspace_cross_file_summary_from_module_interfaces(db, workspace);
    OmenaQueryCommittedStyleSemanticGraphV0 {
        style_fact_entries,
        style_cross_file_summary,
        cross_file_summary,
        css_modules_resolution,
        sass_module_resolution,
        sass_module_resolution_without_manifests,
        sass_module_resolution_without_path_mappings,
        sass_module_resolution_with_external_sifs,
    }
}

fn module_interfaces_for_workspace(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> Vec<OmenaQueryModuleInterfaceProjectionV0> {
    let mut module_interfaces = workspace
        .files(db)
        .iter()
        .map(|file| memo_module_interface_projection(db, *file))
        .collect::<Vec<_>>();
    module_interfaces.sort_by(|left, right| left.style_path.cmp(&right.style_path));
    module_interfaces
}

fn style_fact_entries_for_workspace(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> Vec<OmenaQueryStyleFactEntry> {
    let mut style_fact_entries = workspace
        .files(db)
        .iter()
        .map(|file| memo_style_fact_entry(db, *file))
        .collect::<Vec<_>>();
    style_fact_entries.sort_by(|left, right| left.style_path.cmp(&right.style_path));
    style_fact_entries
}

fn style_paths_for_workspace(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> Vec<String> {
    let mut style_paths = workspace
        .files(db)
        .iter()
        .map(|file| file.style_path(db).clone())
        .collect::<Vec<_>>();
    style_paths.sort();
    style_paths
}

fn resolver_style_paths_for_workspace(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> BTreeSet<String> {
    workspace
        .files(db)
        .iter()
        .flat_map(|file| {
            let style_path = file.style_path(db).clone();
            [style_path.clone(), resolver_style_path(style_path.as_str())]
        })
        .collect()
}

fn file_for_style_path(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    style_path: &str,
) -> Option<OmenaQueryStyleFileInputV0> {
    workspace
        .files(db)
        .iter()
        .copied()
        .find(|file| file.style_path(db) == style_path)
}

fn module_dependency_surfaces_for_workspace(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> Vec<OmenaQueryModuleDependencySurfaceV0> {
    let mut surfaces = workspace
        .files(db)
        .iter()
        .map(|file| memo_module_dependency_surface(db, *file))
        .collect::<Vec<_>>();
    surfaces.sort_by(|left, right| left.style_path.cmp(&right.style_path));
    surfaces
}

fn style_import_reachability_edges_from_dependency_surfaces(
    dependency_surfaces: &[OmenaQueryModuleDependencySurfaceV0],
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<omena_semantic::StyleImportReachabilityEdgeFactV0> {
    let mut edges = Vec::new();
    for surface in dependency_surfaces {
        let targets = surface
            .style_dependency_sources
            .iter()
            .filter_map(|source| {
                resolve_style_module_source(
                    surface.style_path.as_str(),
                    source,
                    available_style_paths,
                    package_manifests,
                )
            })
            .collect::<BTreeSet<_>>();
        for target in targets {
            edges.push(omena_semantic::StyleImportReachabilityEdgeFactV0 {
                from_style_path: surface.style_path.clone(),
                target_style_path: target,
            });
        }
    }
    edges
}

fn sass_configurable_names_by_path_for_workspace(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> BTreeMap<String, BTreeSet<String>> {
    style_paths_for_workspace(db, workspace)
        .into_iter()
        .map(|style_path| {
            let names = memo_sass_configurable_variable_names_from_module_interface(
                db,
                workspace,
                style_path.clone(),
            );
            (style_path, names)
        })
        .collect()
}

fn sass_configurable_names_without_manifests_by_path_for_workspace(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> BTreeMap<String, BTreeSet<String>> {
    style_paths_for_workspace(db, workspace)
        .into_iter()
        .map(|style_path| {
            let names =
                memo_sass_configurable_variable_names_without_manifests_from_module_interface(
                    db,
                    workspace,
                    style_path.clone(),
                );
            (style_path, names)
        })
        .collect()
}

fn sass_configurable_names_without_path_mappings_by_path_for_workspace(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> BTreeMap<String, BTreeSet<String>> {
    style_paths_for_workspace(db, workspace)
        .into_iter()
        .map(|style_path| {
            let names =
                memo_sass_configurable_variable_names_without_path_mappings_from_module_interface(
                    db,
                    workspace,
                    style_path.clone(),
                );
            (style_path, names)
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn sass_configurable_variable_names_for_module_interface_tracked(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    style_path: &str,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
    visiting: &mut BTreeSet<String>,
) -> BTreeSet<String> {
    if !visiting.insert(style_path.to_string()) {
        return BTreeSet::new();
    }
    let Some(file) = file_for_style_path(db, workspace, style_path) else {
        visiting.remove(style_path);
        return BTreeSet::new();
    };
    let projection = memo_module_interface_projection(db, file);
    let projection_style_path = projection.style_path.clone();
    let mut names = projection.sass_module_configurable_variable_names.clone();
    let forward_edges = projection
        .sass_module_edges
        .iter()
        .filter(|edge| edge.kind == "sassForward")
        .cloned()
        .enumerate()
        .collect::<Vec<_>>();
    for (forward_rule_ordinal, edge) in forward_edges {
        let Some(resolved) = resolve_style_module_source(
            projection_style_path.as_str(),
            edge.source.as_str(),
            available_style_paths,
            package_manifests,
        )
        .or_else(|| {
            let resolver_package_manifests = package_manifests
                .iter()
                .map(|manifest| OmenaResolverStylePackageManifestV0 {
                    package_json_path: manifest.package_json_path.clone(),
                    package_json_source: manifest.package_json_source.clone(),
                })
                .collect::<Vec<_>>();
            let load_path_roots = collect_load_path_roots(available_style_paths);
            let load_path_root_refs = load_path_roots
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>();
            summarize_omena_resolver_style_module_resolution_with_load_path_roots(
                resolver_style_path(projection_style_path.as_str()).as_str(),
                edge.source.as_str(),
                available_style_paths,
                &resolver_package_manifests,
                bundler_path_mappings,
                tsconfig_path_mappings,
                &load_path_root_refs,
            )
            .resolved_style_path
        }) else {
            continue;
        };
        let Some(resolved) =
            canonical_available_style_path(resolved.as_str(), available_style_paths)
        else {
            continue;
        };
        let child_names = sass_configurable_variable_names_for_module_interface_tracked(
            db,
            workspace,
            resolved.as_str(),
            available_style_paths,
            package_manifests,
            bundler_path_mappings,
            tsconfig_path_mappings,
            visiting,
        );
        let non_default_forward_overrides = sass_module_forward_variable_overrides_from_interface(
            &projection,
            forward_rule_ordinal,
        )
        .into_iter()
        .filter_map(|(name, override_entry)| (!override_entry.is_default).then_some(name))
        .collect::<BTreeSet<_>>();
        let child_names = child_names
            .into_iter()
            .filter(|name| !non_default_forward_overrides.contains(name))
            .collect::<BTreeSet<_>>();
        names.extend(
            omena_semantic::filter_sass_forward_configurable_variable_names(
                child_names,
                edge.forward_prefix.as_deref(),
                edge.visibility_filter_kind,
                &edge.visibility_filter_names,
            ),
        );
    }
    visiting.remove(style_path);
    names
}

#[allow(dead_code)]
fn style_sources_for_workspace(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> Vec<OmenaQueryStyleSourceInputV0> {
    let mut style_sources = workspace
        .files(db)
        .iter()
        .map(|file| OmenaQueryStyleSourceInputV0 {
            style_path: file.style_path(db).clone(),
            style_source: file.style_source(db).clone(),
        })
        .collect::<Vec<_>>();
    style_sources.sort_by(|left, right| left.style_path.cmp(&right.style_path));
    style_sources
}

/// Owner of the memo database plus the input mirror. The sync discipline is
/// the same self-validating shape as the cascade-narrowing memo (rfcs#63
/// E-ii): every call compares the in-hand inputs against what the database
/// holds and applies `set_*` only for actual differences — there is no event
/// eviction list to keep in sync, so a stale memo cannot be served. File
/// entities persist per path, so re-adding an unchanged file (or switching
/// workspace folders back and forth) keeps its memos green.
pub struct OmenaQueryStyleMemoHostV0 {
    db: OmenaQueryStyleMemoDatabaseV0,
    files_by_path: BTreeMap<String, OmenaQueryStyleFileInputV0>,
    registered_style_paths: BTreeSet<String>,
    workspace: Option<OmenaQueryStyleWorkspaceInputV0>,
    committed_revision: IncrementalRevisionV0,
    committed_graph: Option<OmenaQueryCommittedStyleSemanticGraphV0>,
}

impl std::fmt::Debug for OmenaQueryStyleMemoHostV0 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OmenaQueryStyleMemoHostV0")
            .field("known_file_count", &self.files_by_path.len())
            .field(
                "registered_style_path_count",
                &self.registered_style_paths.len(),
            )
            .field("workspace_initialized", &self.workspace.is_some())
            .field("committed_revision", &self.committed_revision)
            .field(
                "committed_graph_initialized",
                &self.committed_graph.is_some(),
            )
            .finish()
    }
}

impl Default for OmenaQueryStyleMemoHostV0 {
    fn default() -> Self {
        Self::new()
    }
}

impl OmenaQueryStyleMemoHostV0 {
    pub fn new() -> Self {
        Self {
            db: OmenaQueryStyleMemoDatabaseV0::new(),
            files_by_path: BTreeMap::new(),
            registered_style_paths: BTreeSet::new(),
            workspace: None,
            committed_revision: IncrementalRevisionV0 { value: 0 },
            committed_graph: None,
        }
    }

    pub fn committed_revision(&self) -> IncrementalRevisionV0 {
        self.committed_revision
    }

    pub fn register_style_paths<I, S>(&mut self, style_paths: I) -> usize
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let before = self.registered_style_paths.len();
        self.registered_style_paths
            .extend(style_paths.into_iter().map(Into::into));
        self.registered_style_paths.len().saturating_sub(before)
    }

    pub fn registered_style_path_count(&self) -> usize {
        self.registered_style_paths.len()
    }

    /// Sync the in-hand inputs into the database (diff-only), commit a graph,
    /// and run diagnostics for `target_style_path` through that committed
    /// graph. Returns `None` exactly when the straight-line entry point would
    /// (target not in the corpus / no hover candidates).
    ///
    /// A corpus with DUPLICATE `style_path` entries cannot be mirrored as
    /// one input entity per path without diverging from the straight-line
    /// first-match/full-slice semantics, so that (LSP-unreachable) shape
    /// bypasses the memo and evaluates straight-line — byte-identical by
    /// construction, just unmemoized.
    #[allow(clippy::too_many_arguments)]
    pub fn workspace_style_diagnostics(
        &mut self,
        target_style_path: &str,
        style_sources: &[OmenaQueryStyleSourceInputV0],
        source_documents: &[OmenaQuerySourceDocumentInputV0],
        package_manifests: &[OmenaQueryStylePackageManifestV0],
        external_sifs: &[OmenaQueryExternalSifInputV0],
        resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    ) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
        let mut seen_paths = std::collections::BTreeSet::new();
        if style_sources
            .iter()
            .any(|source| !seen_paths.insert(source.style_path.as_str()))
        {
            return summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs(
                target_style_path,
                style_sources,
                source_documents,
                package_manifests,
                None,
                OmenaQueryExternalModuleModeV0::Auto,
                external_sifs,
                resolution_inputs,
            );
        }
        self.register_style_paths(style_sources.iter().map(|source| source.style_path.clone()));
        let workspace = self.sync_workspace(
            style_sources,
            source_documents,
            package_manifests,
            external_sifs,
            resolution_inputs,
        );
        let substrate = memo_workspace_diagnostics_substrate(&self.db, workspace);
        summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs_and_suppression_mode_with_substrate(
            target_style_path,
            style_sources,
            source_documents,
            package_manifests,
            None,
            OmenaQueryExternalModuleModeV0::Auto,
            external_sifs,
            resolution_inputs,
            OmenaQueryDiagnosticSuppressionModeV0::Apply,
            substrate,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn workspace_revision_selector(
        &mut self,
        style_sources: &[OmenaQueryStyleSourceInputV0],
        source_documents: &[OmenaQuerySourceDocumentInputV0],
        package_manifests: &[OmenaQueryStylePackageManifestV0],
        external_sifs: &[OmenaQueryExternalSifInputV0],
        resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    ) -> Option<OmenaQueryStyleRevisionSelectorV0> {
        let mut seen_paths = std::collections::BTreeSet::new();
        if style_sources
            .iter()
            .any(|source| !seen_paths.insert(source.style_path.as_str()))
        {
            return None;
        }
        self.register_style_paths(style_sources.iter().map(|source| source.style_path.clone()));
        let mut transaction = OmenaQueryStyleWorkspaceTransactionV0::new();
        transaction
            .register_style_paths(self.registered_style_paths.iter().cloned())
            .set_workspace_inputs(
                style_sources,
                source_documents,
                package_manifests,
                external_sifs,
                resolution_inputs,
            );
        let commit = transaction.commit_revision(self).ok()?;
        Some(commit.selector)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn workspace_style_diagnostics_with_selector(
        &mut self,
        target_style_path: &str,
        style_sources: &[OmenaQueryStyleSourceInputV0],
        source_documents: &[OmenaQuerySourceDocumentInputV0],
        package_manifests: &[OmenaQueryStylePackageManifestV0],
        external_sifs: &[OmenaQueryExternalSifInputV0],
        resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    ) -> Option<OmenaQueryStyleDiagnosticsWithSelectorV0> {
        let selector = self.workspace_revision_selector(
            style_sources,
            source_documents,
            package_manifests,
            external_sifs,
            resolution_inputs,
        )?;
        let diagnostics = selector.workspace_style_diagnostics(target_style_path)?;
        Some(OmenaQueryStyleDiagnosticsWithSelectorV0 {
            diagnostics,
            selector,
        })
    }

    /// RFC 0009 Pillar F (rfcs#68): run the SAME diff-only sync as
    /// [`Self::workspace_style_diagnostics`] — loop-side, before any handle
    /// exists — and hand back a fixed-revision view bundle for a parallel
    /// fan-out. Returns `None` for a corpus with duplicate `style_path`
    /// entries, exactly where the memoized entry point bypasses to the
    /// straight-line arm; the caller must fall back to its serial path.
    pub fn sync_workspace_for_parallel_resolve(
        &mut self,
        style_sources: &[OmenaQueryStyleSourceInputV0],
        source_documents: &[OmenaQuerySourceDocumentInputV0],
        package_manifests: &[OmenaQueryStylePackageManifestV0],
        external_sifs: &[OmenaQueryExternalSifInputV0],
        resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    ) -> Option<OmenaQueryStyleParallelResolveSyncV0> {
        let mut seen_paths = std::collections::BTreeSet::new();
        if style_sources
            .iter()
            .any(|source| !seen_paths.insert(source.style_path.as_str()))
        {
            return None;
        }
        self.register_style_paths(style_sources.iter().map(|source| source.style_path.clone()));
        let mut transaction = OmenaQueryStyleWorkspaceTransactionV0::new();
        transaction
            .register_style_paths(self.registered_style_paths.iter().cloned())
            .set_workspace_inputs(
                style_sources,
                source_documents,
                package_manifests,
                external_sifs,
                resolution_inputs,
            );
        let commit = transaction.commit_revision(self).ok()?;
        Some(commit.selector.into_parallel_resolve_sync())
    }

    fn commit_workspace_transaction(
        &mut self,
        transaction: OmenaQueryStyleWorkspaceTransactionV0,
    ) -> Result<
        OmenaQueryStyleWorkspaceTransactionCommitV0,
        OmenaQueryStyleWorkspaceTransactionErrorV0,
    > {
        let commit = self.commit_workspace_transaction_core(transaction)?;
        let selector = build_revision_selector(
            commit.revision,
            commit.style_sources.as_slice(),
            commit.source_documents.as_slice(),
            commit.package_manifests.as_slice(),
            commit.external_sifs.as_slice(),
            &commit.resolution_inputs,
            commit.committed_graph,
        );
        Ok(OmenaQueryStyleWorkspaceTransactionCommitV0 {
            revision: commit.revision,
            workspace: commit.workspace,
            files: commit.files,
            changed_style_paths: commit.changed_style_paths,
            selector,
        })
    }

    fn commit_workspace_transaction_core(
        &mut self,
        transaction: OmenaQueryStyleWorkspaceTransactionV0,
    ) -> Result<
        OmenaQueryStyleWorkspaceTransactionCoreCommitV0,
        OmenaQueryStyleWorkspaceTransactionErrorV0,
    > {
        validate_workspace_transaction(&transaction)?;
        let changed_style_paths = self.changed_style_paths_for_transaction(&transaction);
        if changed_style_paths.is_empty()
            && let (Some(workspace), Some(committed_graph)) =
                (self.workspace, self.committed_graph.clone())
        {
            let files = transaction
                .style_sources
                .iter()
                .filter_map(|source| {
                    self.files_by_path
                        .get(source.style_path.as_str())
                        .map(|file| (source.style_path.clone(), *file))
                })
                .collect::<Vec<_>>();
            return Ok(OmenaQueryStyleWorkspaceTransactionCoreCommitV0 {
                revision: self.committed_revision,
                workspace,
                files,
                changed_style_paths,
                style_sources: transaction.style_sources,
                source_documents: transaction.source_documents,
                package_manifests: transaction.package_manifests,
                external_sifs: transaction.external_sifs,
                resolution_inputs: transaction.resolution_inputs,
                committed_graph,
            });
        }
        let workspace = self.sync_workspace(
            transaction.style_sources.as_slice(),
            transaction.source_documents.as_slice(),
            transaction.package_manifests.as_slice(),
            transaction.external_sifs.as_slice(),
            &transaction.resolution_inputs,
        );
        let files = transaction
            .style_sources
            .iter()
            .filter_map(|source| {
                self.files_by_path
                    .get(source.style_path.as_str())
                    .map(|file| (source.style_path.clone(), *file))
            })
            .collect::<Vec<_>>();
        self.committed_revision = IncrementalRevisionV0 {
            value: self.committed_revision.value + 1,
        };
        let committed_graph = build_committed_style_semantic_graph(
            &self.db,
            workspace,
            transaction.source_documents.as_slice(),
            transaction.package_manifests.as_slice(),
            transaction.external_sifs.as_slice(),
            &transaction.resolution_inputs,
        );
        self.committed_graph = Some(committed_graph.clone());
        Ok(OmenaQueryStyleWorkspaceTransactionCoreCommitV0 {
            revision: self.committed_revision,
            workspace,
            files,
            changed_style_paths,
            style_sources: transaction.style_sources,
            source_documents: transaction.source_documents,
            package_manifests: transaction.package_manifests,
            external_sifs: transaction.external_sifs,
            resolution_inputs: transaction.resolution_inputs,
            committed_graph,
        })
    }

    fn changed_style_paths_for_transaction(
        &self,
        transaction: &OmenaQueryStyleWorkspaceTransactionV0,
    ) -> BTreeSet<String> {
        let mut changed = BTreeSet::new();
        let incoming_paths = transaction
            .style_sources
            .iter()
            .map(|source| source.style_path.as_str())
            .collect::<BTreeSet<_>>();
        let Some(workspace) = self.workspace else {
            changed.extend(
                transaction
                    .style_sources
                    .iter()
                    .map(|source| source.style_path.clone()),
            );
            return changed;
        };

        for source in &transaction.style_sources {
            match self.files_by_path.get(source.style_path.as_str()) {
                Some(file) if file.style_source(&self.db) == &source.style_source => {}
                _ => {
                    changed.insert(source.style_path.clone());
                }
            }
        }
        for file in workspace.files(&self.db) {
            let style_path = file.style_path(&self.db);
            if !incoming_paths.contains(style_path.as_str()) {
                changed.insert(style_path.clone());
            }
        }

        let global_inputs_changed = workspace.source_documents(&self.db).as_slice()
            != transaction.source_documents.as_slice()
            || workspace.package_manifests(&self.db).as_slice()
                != transaction.package_manifests.as_slice()
            || workspace.external_sifs(&self.db).as_slice() != transaction.external_sifs.as_slice()
            || workspace.resolution_inputs(&self.db) != &transaction.resolution_inputs;
        if global_inputs_changed {
            changed.extend(
                transaction
                    .style_sources
                    .iter()
                    .map(|source| source.style_path.clone()),
            );
        }
        changed
    }

    fn sync_workspace(
        &mut self,
        style_sources: &[OmenaQueryStyleSourceInputV0],
        source_documents: &[OmenaQuerySourceDocumentInputV0],
        package_manifests: &[OmenaQueryStylePackageManifestV0],
        external_sifs: &[OmenaQueryExternalSifInputV0],
        resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    ) -> OmenaQueryStyleWorkspaceInputV0 {
        let files = style_sources
            .iter()
            .map(
                |source| match self.files_by_path.get(source.style_path.as_str()) {
                    Some(file) => {
                        if file.style_source(&self.db) != &source.style_source {
                            file.set_style_source(&mut self.db)
                                .to(source.style_source.clone());
                        }
                        *file
                    }
                    None => {
                        let file = OmenaQueryStyleFileInputV0::new(
                            &self.db,
                            source.style_path.clone(),
                            source.style_source.clone(),
                        );
                        self.files_by_path.insert(source.style_path.clone(), file);
                        file
                    }
                },
            )
            .collect::<Vec<_>>();

        match self.workspace {
            Some(workspace) => {
                if workspace.files(&self.db) != &files {
                    workspace.set_files(&mut self.db).to(files);
                }
                if workspace.source_documents(&self.db).as_slice() != source_documents {
                    workspace
                        .set_source_documents(&mut self.db)
                        .to(source_documents.to_vec());
                }
                if workspace.package_manifests(&self.db).as_slice() != package_manifests {
                    workspace
                        .set_package_manifests(&mut self.db)
                        .to(package_manifests.to_vec());
                }
                if workspace.external_sifs(&self.db).as_slice() != external_sifs {
                    workspace
                        .set_external_sifs(&mut self.db)
                        .to(external_sifs.to_vec());
                }
                if workspace.resolution_inputs(&self.db) != resolution_inputs {
                    workspace
                        .set_resolution_inputs(&mut self.db)
                        .to(resolution_inputs.clone());
                }
                workspace
            }
            None => {
                let workspace = OmenaQueryStyleWorkspaceInputV0::new(
                    &self.db,
                    files,
                    source_documents.to_vec(),
                    package_manifests.to_vec(),
                    external_sifs.to_vec(),
                    resolution_inputs.clone(),
                );
                self.workspace = Some(workspace);
                workspace
            }
        }
    }
}

fn build_revision_selector(
    revision: IncrementalRevisionV0,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    external_sifs: &[OmenaQueryExternalSifInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    committed_graph: OmenaQueryCommittedStyleSemanticGraphV0,
) -> OmenaQueryStyleRevisionSelectorV0 {
    let mut host = OmenaQueryStyleMemoHostV0::new();
    let workspace = host.sync_workspace(
        style_sources,
        source_documents,
        package_manifests,
        external_sifs,
        resolution_inputs,
    );
    let OmenaQueryStyleMemoHostV0 {
        db,
        files_by_path,
        registered_style_paths: _,
        workspace: _,
        committed_revision: _,
        committed_graph: _,
    } = host;
    let files = style_sources
        .iter()
        .filter_map(|source| {
            files_by_path
                .get(source.style_path.as_str())
                .map(|file| (source.style_path.clone(), *file))
        })
        .collect();
    OmenaQueryStyleRevisionSelectorV0 {
        revision,
        db,
        workspace,
        files,
        files_by_path,
        committed_graph,
    }
}

fn build_committed_style_semantic_graph(
    db: &OmenaQueryStyleMemoDatabaseV0,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    _source_documents: &[OmenaQuerySourceDocumentInputV0],
    _package_manifests: &[OmenaQueryStylePackageManifestV0],
    _external_sifs: &[OmenaQueryExternalSifInputV0],
    _resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> OmenaQueryCommittedStyleSemanticGraphV0 {
    #[cfg(any(test, feature = "test-support"))]
    record_committed_style_semantic_graph_compute_for_test();

    memo_committed_style_semantic_graph_from_module_interfaces(db, workspace)
}

#[allow(dead_code)]
pub(crate) fn build_committed_style_semantic_graph_monolith(
    db: &OmenaQueryStyleMemoDatabaseV0,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    external_sifs: &[OmenaQueryExternalSifInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> OmenaQueryCommittedStyleSemanticGraphV0 {
    let style_sources = style_sources_for_workspace(db, workspace);
    let style_fact_entries = style_fact_entries_for_workspace(db, workspace);
    let css_modules_resolution =
        summarize_css_modules_cross_file_resolution(&style_fact_entries, package_manifests);
    let sass_module_resolution = summarize_sass_module_cross_file_resolution(
        &style_fact_entries,
        package_manifests,
        resolution_inputs.bundler_path_mappings.as_slice(),
        resolution_inputs.tsconfig_path_mappings.as_slice(),
    );
    let sass_module_resolution_without_manifests = summarize_sass_module_cross_file_resolution(
        &style_fact_entries,
        &[],
        resolution_inputs.bundler_path_mappings.as_slice(),
        resolution_inputs.tsconfig_path_mappings.as_slice(),
    );
    let sass_module_resolution_without_path_mappings = summarize_sass_module_cross_file_resolution(
        &style_fact_entries,
        package_manifests,
        &[],
        &[],
    );
    let mut sass_module_resolution_with_external_sifs = sass_module_resolution.clone();
    promote_sif_backed_external_edges(
        &mut sass_module_resolution_with_external_sifs,
        OmenaQueryExternalSifResolutionContext {
            package_manifests,
            bundler_path_mappings: resolution_inputs.bundler_path_mappings.as_slice(),
            tsconfig_path_mappings: resolution_inputs.tsconfig_path_mappings.as_slice(),
            external_sifs,
        },
    );
    let style_cross_file_summary = summarize_omena_query_cross_file_summary(
        style_fact_entries.as_slice(),
        &css_modules_resolution,
        &sass_module_resolution,
    );
    let cross_file_summary = summarize_omena_query_workspace_cross_file_summary_from_style_summary(
        style_sources.as_slice(),
        source_documents,
        package_manifests,
        style_cross_file_summary.clone(),
    );
    OmenaQueryCommittedStyleSemanticGraphV0 {
        style_fact_entries,
        style_cross_file_summary,
        cross_file_summary,
        css_modules_resolution,
        sass_module_resolution,
        sass_module_resolution_without_manifests,
        sass_module_resolution_without_path_mappings,
        sass_module_resolution_with_external_sifs,
    }
}

fn validate_workspace_transaction(
    transaction: &OmenaQueryStyleWorkspaceTransactionV0,
) -> Result<(), OmenaQueryStyleWorkspaceTransactionErrorV0> {
    let mut seen_paths = BTreeSet::new();
    for source in &transaction.style_sources {
        if !seen_paths.insert(source.style_path.as_str()) {
            return Err(
                OmenaQueryStyleWorkspaceTransactionErrorV0::DuplicateStylePath {
                    style_path: source.style_path.clone(),
                },
            );
        }
        if !transaction
            .registered_style_paths
            .contains(source.style_path.as_str())
        {
            return Err(
                OmenaQueryStyleWorkspaceTransactionErrorV0::UnregisteredStylePath {
                    style_path: source.style_path.clone(),
                },
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use omena_evidence_graph::{
        EvidenceDemandEdgeV0, EvidenceNodeKeyV0, EvidenceNodeSeedV0, GuaranteeKindV0,
        build_salsa_demand_evidence_graph_v0,
    };
    use std::collections::BTreeSet;

    fn parallel_probe_corpus() -> Vec<OmenaQueryStyleSourceInputV0> {
        vec![
            OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace/src/App.module.scss".to_string(),
                style_source: "@use \"./theme\";\n.app { color: red; }\n".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace/src/_theme.scss".to_string(),
                style_source: ":root { --tone: green; }\n.btn { color: var(--tone); }\n"
                    .to_string(),
            },
        ]
    }

    fn doubled_parallel_probe_corpus() -> Vec<OmenaQueryStyleSourceInputV0> {
        let mut corpus = parallel_probe_corpus();
        corpus.extend([
            OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace/src/Card.module.scss".to_string(),
                style_source: ".card { display: grid; }\n.card__title { color: navy; }\n"
                    .to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace/src/Tokens.module.css".to_string(),
                style_source: ":root { --space: 8px; }\n.stack { gap: var(--space); }\n"
                    .to_string(),
            },
        ]);
        corpus
    }

    fn css_modules_resolution_probe_corpus() -> Vec<OmenaQueryStyleSourceInputV0> {
        vec![
            OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace/src/base.module.css".to_string(),
                style_source: ".base { color: red; }\n".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace/src/tokens.module.css".to_string(),
                style_source: "@value primary: #fff; :export { exported: primary; }\n".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace/src/Card.module.css".to_string(),
                style_source: r#"@value primary as brand from "./tokens.module.css";
:import("./tokens.module.css") { imported: exported; }
:export { forwarded: imported; }
.card { composes: base from "./base.module.css"; color: brand; background: white; }
"#
                .to_string(),
            },
        ]
    }

    fn sass_module_resolution_probe_corpus() -> Vec<OmenaQueryStyleSourceInputV0> {
        vec![
            OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace/src/tokens.scss".to_string(),
                style_source: "$brand: red !default;\n.token { color: $brand; }\n".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace/src/theme.scss".to_string(),
                style_source: r#"@forward "./tokens.scss" with ($brand: blue !default);
.theme { color: blue; }
"#
                .to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace/src/app.scss".to_string(),
                style_source: r#"@use "./theme.scss" as theme;
.app { color: theme.$brand; background: white; }
"#
                .to_string(),
            },
        ]
    }

    fn set_of(paths: impl IntoIterator<Item = &'static str>) -> BTreeSet<String> {
        paths.into_iter().map(str::to_string).collect()
    }

    #[test]
    fn workspace_transaction_commit_revision_increases_and_preserves_per_file_firewall()
    -> Result<(), &'static str> {
        let corpus = parallel_probe_corpus();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();

        let mut transaction = OmenaQueryStyleWorkspaceTransactionV0::new();
        transaction
            .register_style_sources(corpus.as_slice())
            .set_workspace_inputs(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        style_fact_entry_probe::reset();
        let commit = transaction
            .commit_revision(&mut host)
            .map_err(|_| "registered transaction must commit")?;
        assert_eq!(commit.revision, IncrementalRevisionV0 { value: 1 });
        assert_eq!(
            commit.changed_style_paths,
            set_of([
                "/workspace/src/App.module.scss",
                "/workspace/src/_theme.scss",
            ]),
            "initial transaction registers every style file as changed",
        );

        assert_eq!(
            style_fact_entry_probe::read(),
            set_of([
                "/workspace/src/App.module.scss",
                "/workspace/src/_theme.scss",
            ]),
            "initial committed revision must collect every file fact entry",
        );

        let mut edited_corpus = corpus.clone();
        edited_corpus[0]
            .style_source
            .push_str("\n.app__icon { color: blue; }\n");
        let mut transaction = OmenaQueryStyleWorkspaceTransactionV0::new();
        transaction
            .register_style_sources(edited_corpus.as_slice())
            .set_workspace_inputs(edited_corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        style_fact_entry_probe::reset();
        let edited_commit = transaction
            .commit_revision(&mut host)
            .map_err(|_| "registered edit transaction must commit")?;
        assert_eq!(edited_commit.revision, IncrementalRevisionV0 { value: 2 });
        assert_eq!(
            edited_commit.changed_style_paths,
            set_of(["/workspace/src/App.module.scss"]),
            "editing one registered style file must report only that file as the transaction delta",
        );

        assert_eq!(
            style_fact_entry_probe::read(),
            set_of(["/workspace/src/App.module.scss"]),
            "transaction commit must preserve the per-file salsa firewall",
        );
        Ok(())
    }

    #[test]
    fn workspace_transaction_reuses_revision_and_graph_when_inputs_are_unchanged()
    -> Result<(), &'static str> {
        let corpus = parallel_probe_corpus();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();

        let mut transaction = OmenaQueryStyleWorkspaceTransactionV0::new();
        transaction
            .register_style_sources(corpus.as_slice())
            .set_workspace_inputs(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        let initial_commit = transaction
            .commit_revision(&mut host)
            .map_err(|_| "initial registered transaction must commit")?;
        assert_eq!(initial_commit.revision, IncrementalRevisionV0 { value: 1 });
        assert_eq!(
            host.committed_revision(),
            IncrementalRevisionV0 { value: 1 }
        );

        reset_committed_style_semantic_graph_compute_count_for_test();
        style_fact_entry_probe::reset();
        let mut unchanged_transaction = OmenaQueryStyleWorkspaceTransactionV0::new();
        unchanged_transaction
            .register_style_sources(corpus.as_slice())
            .set_workspace_inputs(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        let unchanged_commit = unchanged_transaction
            .commit_revision(&mut host)
            .map_err(|_| "unchanged registered transaction must commit")?;

        assert_eq!(
            unchanged_commit.revision, initial_commit.revision,
            "unchanged transactions must keep the committed workspace revision pinned",
        );
        assert!(
            unchanged_commit.changed_style_paths.is_empty(),
            "unchanged transactions should not report a recompute delta",
        );
        assert_eq!(
            read_committed_style_semantic_graph_compute_count_for_test(),
            0,
            "unchanged transactions must reuse the graph committed at the existing revision",
        );
        assert!(
            style_fact_entry_probe::read().is_empty(),
            "unchanged transactions must not re-run per-file fact collection",
        );
        Ok(())
    }

    #[test]
    fn workspace_transaction_rejects_unregistered_style_file_without_revision_bump()
    -> Result<(), &'static str> {
        let corpus = parallel_probe_corpus();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let before = host.committed_revision();

        let mut transaction = OmenaQueryStyleWorkspaceTransactionV0::new();
        transaction
            .register_style_file("/workspace/src/App.module.scss")
            .set_workspace_inputs(corpus.as_slice(), &[], &[], &[], &resolution_inputs);

        let Err(error) = transaction.commit_revision(&mut host) else {
            return Err("unregistered workspace file must reject the transaction");
        };
        assert_eq!(
            error,
            OmenaQueryStyleWorkspaceTransactionErrorV0::UnregisteredStylePath {
                style_path: "/workspace/src/_theme.scss".to_string(),
            },
            "a transaction must fail closed when a workspace file was not registered",
        );
        assert_eq!(
            host.committed_revision(),
            before,
            "failed transactions must not bump the committed revision",
        );
        assert!(
            host.workspace.is_none(),
            "failed transactions must not initialize or mutate the workspace mirror",
        );
        Ok(())
    }

    #[test]
    fn revision_selector_reads_committed_snapshot_after_later_commit() -> Result<(), &'static str> {
        let corpus = parallel_probe_corpus();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();

        let mut transaction = OmenaQueryStyleWorkspaceTransactionV0::new();
        transaction
            .register_style_sources(corpus.as_slice())
            .set_workspace_inputs(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        let commit = transaction
            .commit_revision(&mut host)
            .map_err(|_| "registered transaction must commit")?;
        assert_eq!(
            commit.selector.revision(),
            IncrementalRevisionV0 { value: 1 }
        );
        let selector = commit.selector;
        let initial_json = serde_json::to_string(
            &selector.workspace_style_diagnostics("/workspace/src/App.module.scss"),
        )
        .map_err(|_| "initial selector diagnostics must serialize")?;

        let mut edited_corpus = corpus.clone();
        edited_corpus[0].style_source =
            format!("@use \"./missing\";\n{}", edited_corpus[0].style_source);
        let mut transaction = OmenaQueryStyleWorkspaceTransactionV0::new();
        transaction
            .register_style_sources(edited_corpus.as_slice())
            .set_workspace_inputs(edited_corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        let edited_commit = transaction
            .commit_revision(&mut host)
            .map_err(|_| "registered edit transaction must commit")?;

        let old_selector_json = serde_json::to_string(
            &selector.workspace_style_diagnostics("/workspace/src/App.module.scss"),
        )
        .map_err(|_| "old selector diagnostics must serialize")?;
        assert_eq!(
            old_selector_json, initial_json,
            "a selector pinned to an earlier commit must not observe a later commit",
        );

        let fresh_json = serde_json::to_string(
            &edited_commit
                .selector
                .workspace_style_diagnostics("/workspace/src/App.module.scss"),
        )
        .map_err(|_| "fresh selector diagnostics must serialize")?;
        assert_ne!(
            fresh_json, initial_json,
            "a fresh selector for the edited commit must observe the changed diagnostics",
        );
        assert_eq!(
            edited_commit.selector.revision(),
            IncrementalRevisionV0 { value: 2 },
        );
        Ok(())
    }

    #[test]
    fn revision_selector_committed_graph_matches_direct_paths_without_direct_recompute()
    -> Result<(), &'static str> {
        let corpus = parallel_probe_corpus();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let direct_summary =
            summarize_omena_query_workspace_cross_file_summary(corpus.as_slice(), &[], &[]);
        let direct_sass = summarize_omena_query_sass_module_cross_file_resolution_for_workspace(
            corpus.as_slice(),
            &[],
            &[],
            &[],
        );
        reset_workspace_cross_file_summary_direct_recompute_count_for_test();
        reset_sass_module_resolution_direct_recompute_count_for_test();
        reset_committed_style_semantic_graph_compute_count_for_test();

        let mut host = OmenaQueryStyleMemoHostV0::new();
        let mut transaction = OmenaQueryStyleWorkspaceTransactionV0::new();
        transaction
            .register_style_sources(corpus.as_slice())
            .set_workspace_inputs(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        let commit = transaction
            .commit_revision(&mut host)
            .map_err(|_| "registered transaction must commit")?;

        assert_eq!(
            commit.selector.workspace_cross_file_summary(),
            &direct_summary
        );
        assert_eq!(
            commit.selector.sass_module_cross_file_resolution(),
            &direct_sass
        );
        reset_sass_module_resolution_internal_compute_count_for_test();
        let _ = commit
            .selector
            .workspace_style_diagnostics("/workspace/src/App.module.scss");
        let _ = commit.selector.committed_style_semantic_graph();
        let _ = commit.selector.workspace_cross_file_summary();
        let _ = commit.selector.css_modules_cross_file_resolution();
        let _ = commit.selector.sass_module_cross_file_resolution();
        let _ = commit.selector.workspace_cross_file_summary();
        let _ = commit.selector.sass_module_cross_file_resolution();
        assert_eq!(
            read_committed_style_semantic_graph_compute_count_for_test(),
            1,
            "selector graph lookup must reuse the graph computed at transaction commit",
        );
        assert_eq!(
            read_workspace_cross_file_summary_direct_recompute_count_for_test(),
            0,
            "selector graph lookup must not call the direct workspace summary API",
        );
        assert_eq!(
            read_sass_module_resolution_direct_recompute_count_for_test(),
            0,
            "selector graph lookup must not call the direct Sass module resolution API",
        );
        assert_eq!(
            read_sass_module_resolution_internal_compute_count_for_test(),
            0,
            "selector diagnostics lookup must reuse committed Sass resolution variants",
        );
        Ok(())
    }

    #[test]
    fn revision_selector_sass_identity_diagnostics_reuses_committed_graph()
    -> Result<(), &'static str> {
        let corpus = vec![
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/tokens.scss".to_string(),
                style_source: "$brand: blue !default;".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/theme-red.scss".to_string(),
                style_source: r#"@forward "./tokens" with ($brand: red);"#.to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/theme-blue.scss".to_string(),
                style_source: r#"@forward "./tokens" with ($brand: blue);"#.to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/App.module.scss".to_string(),
                style_source:
                    r#"@use "./theme-red" as redTheme; @use "./theme-blue" as blueTheme;"#
                        .to_string(),
            },
        ];
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let direct_diagnostics =
            summarize_omena_query_sass_module_resolution_identity_diagnostics_for_workspace(
                "/tmp/App.module.scss",
                corpus.as_slice(),
                &[],
                &resolution_inputs,
            );
        assert!(
            direct_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "sassModuleConfigurationConflict"),
            "fixture must exercise Sass module identity diagnostics",
        );

        let mut host = OmenaQueryStyleMemoHostV0::new();
        let mut transaction = OmenaQueryStyleWorkspaceTransactionV0::new();
        transaction
            .register_style_sources(corpus.as_slice())
            .set_workspace_inputs(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        let commit = transaction
            .commit_revision(&mut host)
            .map_err(|_| "registered transaction must commit")?;

        reset_sass_module_resolution_direct_recompute_count_for_test();
        reset_sass_module_resolution_internal_compute_count_for_test();
        let first = commit
            .selector
            .sass_module_resolution_identity_diagnostics_for_workspace("/tmp/App.module.scss");
        let second = commit
            .selector
            .sass_module_resolution_identity_diagnostics_for_workspace("/tmp/App.module.scss");
        assert_eq!(first, direct_diagnostics);
        assert_eq!(second, direct_diagnostics);
        assert_eq!(
            read_sass_module_resolution_direct_recompute_count_for_test(),
            0,
            "selector Sass identity diagnostics must not call the direct workspace API",
        );
        assert_eq!(
            read_sass_module_resolution_internal_compute_count_for_test(),
            0,
            "selector Sass identity diagnostics must reuse the committed Sass resolution",
        );
        Ok(())
    }

    #[test]
    fn public_sass_identity_diagnostics_uses_committed_graph() -> Result<(), &'static str> {
        let corpus = vec![
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/tokens.scss".to_string(),
                style_source: "$brand: blue !default;".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/theme-red.scss".to_string(),
                style_source: r#"@forward "./tokens" with ($brand: red);"#.to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/theme-blue.scss".to_string(),
                style_source: r#"@forward "./tokens" with ($brand: blue);"#.to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/App.module.scss".to_string(),
                style_source:
                    r#"@use "./theme-red" as redTheme; @use "./theme-blue" as blueTheme;"#
                        .to_string(),
            },
        ];
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        reset_sass_module_resolution_direct_recompute_count_for_test();
        reset_committed_style_semantic_graph_compute_count_for_test();

        let diagnostics =
            summarize_omena_query_sass_module_resolution_identity_diagnostics_for_workspace(
                "/tmp/App.module.scss",
                corpus.as_slice(),
                &[],
                &resolution_inputs,
            );

        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "sassModuleConfigurationConflict"),
            "fixture must exercise Sass module identity diagnostics",
        );
        assert_eq!(
            read_committed_style_semantic_graph_compute_count_for_test(),
            1,
            "public Sass identity diagnostics should commit one selector graph",
        );
        assert_eq!(
            read_sass_module_resolution_direct_recompute_count_for_test(),
            0,
            "public Sass identity diagnostics should avoid the direct Sass workspace API on registered inputs",
        );
        Ok(())
    }

    #[test]
    fn workspace_style_diagnostics_direct_path_skips_committed_graph_compute()
    -> Result<(), &'static str> {
        let mut corpus = parallel_probe_corpus();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();

        reset_committed_style_semantic_graph_compute_count_for_test();
        let first = host.workspace_style_diagnostics(
            "/workspace/src/App.module.scss",
            corpus.as_slice(),
            &[],
            &[],
            &[],
            &resolution_inputs,
        );
        assert!(first.is_some(), "diagnostics fixture must resolve");

        corpus[0]
            .style_source
            .push_str("\n.directPathProbe { color: currentColor; }\n");
        let second = host.workspace_style_diagnostics(
            "/workspace/src/App.module.scss",
            corpus.as_slice(),
            &[],
            &[],
            &[],
            &resolution_inputs,
        );
        assert!(second.is_some(), "edited diagnostics fixture must resolve");
        assert_eq!(
            read_committed_style_semantic_graph_compute_count_for_test(),
            0,
            "diagnostics-only hot path must use the tracked diagnostics substrate, not the full committed graph",
        );
        Ok(())
    }

    #[test]
    fn revision_selector_cascade_narrowing_substrate_reuses_committed_graph()
    -> Result<(), &'static str> {
        let corpus = parallel_probe_corpus();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let direct_substrate =
            collect_omena_query_style_cascade_narrowing_substrate_with_external_sifs(
                corpus.as_slice(),
                &[],
                &[],
                &resolution_inputs,
            );

        let mut host = OmenaQueryStyleMemoHostV0::new();
        let mut transaction = OmenaQueryStyleWorkspaceTransactionV0::new();
        transaction
            .register_style_sources(corpus.as_slice())
            .set_workspace_inputs(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        let commit = transaction
            .commit_revision(&mut host)
            .map_err(|_| "registered transaction must commit")?;

        reset_sass_module_resolution_internal_compute_count_for_test();
        let first_substrate = commit.selector.style_cascade_narrowing_substrate();
        let second_substrate = commit.selector.style_cascade_narrowing_substrate();
        assert_eq!(first_substrate, direct_substrate);
        assert_eq!(second_substrate, direct_substrate);
        assert_eq!(
            read_sass_module_resolution_internal_compute_count_for_test(),
            0,
            "selector substrate lookup must reuse the Sass resolution committed with the graph",
        );
        Ok(())
    }

    #[test]
    fn revision_selector_style_completion_reuses_committed_graph() -> Result<(), &'static str> {
        let corpus = parallel_probe_corpus();
        let position = ParserPositionV0 {
            line: 1,
            character: 1,
        };
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let direct_completion =
            summarize_omena_query_style_completion_for_workspace_file_with_resolution_inputs(
                "/workspace/src/App.module.scss",
                corpus.as_slice(),
                &[],
                &[],
                &resolution_inputs,
                position,
            );

        let mut host = OmenaQueryStyleMemoHostV0::new();
        let mut transaction = OmenaQueryStyleWorkspaceTransactionV0::new();
        transaction
            .register_style_sources(corpus.as_slice())
            .set_workspace_inputs(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        let commit = transaction
            .commit_revision(&mut host)
            .map_err(|_| "registered transaction must commit")?;

        reset_sass_module_resolution_internal_compute_count_for_test();
        let first_completion = commit
            .selector
            .style_completion_for_workspace_file("/workspace/src/App.module.scss", position);
        let second_completion = commit
            .selector
            .style_completion_for_workspace_file("/workspace/src/App.module.scss", position);
        let direct_json = serde_json::to_value(&direct_completion)
            .map_err(|_| "direct completion must serialize")?;
        assert_eq!(
            serde_json::to_value(&first_completion)
                .map_err(|_| "selector completion must serialize")?,
            direct_json,
        );
        assert_eq!(
            serde_json::to_value(&second_completion)
                .map_err(|_| "selector completion must serialize")?,
            direct_json,
        );
        assert_eq!(
            read_sass_module_resolution_internal_compute_count_for_test(),
            0,
            "selector style completion must reuse the Sass resolution committed with the graph",
        );
        Ok(())
    }

    #[test]
    fn revision_selector_style_semantic_graph_batch_reuses_committed_graph()
    -> Result<(), &'static str> {
        let corpus = parallel_probe_corpus();
        let input = EngineInputV2 {
            version: "2".to_string(),
            sources: Vec::new(),
            styles: Vec::new(),
            type_facts: Vec::new(),
        };
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let direct_batch =
            summarize_omena_query_style_semantic_graph_batch_from_sources_with_package_manifests(
                corpus
                    .iter()
                    .map(|source| (source.style_path.as_str(), source.style_source.as_str())),
                &input,
                &[],
            );

        let mut host = OmenaQueryStyleMemoHostV0::new();
        let mut transaction = OmenaQueryStyleWorkspaceTransactionV0::new();
        transaction
            .register_style_sources(corpus.as_slice())
            .set_workspace_inputs(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        let commit = transaction
            .commit_revision(&mut host)
            .map_err(|_| "registered transaction must commit")?;

        reset_sass_module_resolution_internal_compute_count_for_test();
        let selector_batch = commit.selector.style_semantic_graph_batch(&input, &[]);
        assert_eq!(
            &selector_batch.cross_file_summary,
            &commit
                .selector
                .committed_style_semantic_graph()
                .style_cross_file_summary,
            "selector semantic graph batch should read the style-only summary committed with the graph",
        );
        assert_ne!(
            &selector_batch.cross_file_summary,
            commit.selector.workspace_cross_file_summary(),
            "semantic graph batch must not substitute the workspace style+source summary for its style-only summary",
        );
        assert_eq!(
            serde_json::to_value(&selector_batch).map_err(|_| "selector batch must serialize")?,
            serde_json::to_value(&direct_batch).map_err(|_| "direct batch must serialize")?,
        );
        assert_eq!(
            read_sass_module_resolution_internal_compute_count_for_test(),
            0,
            "selector semantic graph batch must reuse the Sass resolution committed with the graph",
        );
        Ok(())
    }

    #[test]
    fn parallel_resolve_views_match_the_host_entry_point() -> Result<(), &'static str> {
        let corpus = parallel_probe_corpus();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();

        let sync = host
            .sync_workspace_for_parallel_resolve(
                corpus.as_slice(),
                &[],
                &[],
                &[],
                &resolution_inputs,
            )
            .ok_or("duplicate-free corpus must sync for parallel resolve")?;
        let workspace = sync.workspace;
        let committed_graph = sync.committed_graph.clone();
        let view_results = std::thread::scope(|scope| {
            let workers = sync
                .files
                .iter()
                .map(|(style_path, file)| {
                    let handle = sync.handle.clone();
                    let committed_graph = committed_graph.clone();
                    let file = *file;
                    let style_path = style_path.clone();
                    scope.spawn(move || {
                        let db = OmenaQueryStyleMemoDatabaseV0::from_handle(handle);
                        let summary = resolve_committed_workspace_style_diagnostics_from_view(
                            &db,
                            workspace,
                            file,
                            &committed_graph,
                        );
                        (style_path, serde_json::to_string(&summary).ok())
                    })
                })
                .collect::<Vec<_>>();
            workers
                .into_iter()
                .map(|worker| worker.join().map_err(|_| "parallel view worker panicked"))
                .collect::<Result<Vec<_>, _>>()
        })?;
        drop(sync);

        for (style_path, view_json) in view_results {
            let host_summary = host.workspace_style_diagnostics(
                style_path.as_str(),
                corpus.as_slice(),
                &[],
                &[],
                &[],
                &resolution_inputs,
            );
            assert_eq!(
                view_json,
                serde_json::to_string(&host_summary).ok(),
                "fixed-revision view diagnostics must be byte-identical to the host entry point for {style_path}",
            );
        }
        Ok(())
    }

    #[test]
    fn duplicate_path_corpus_refuses_a_parallel_resolve_sync() {
        let mut corpus = parallel_probe_corpus();
        corpus.push(OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/App.module.scss".to_string(),
            style_source: ".dup { color: blue; }".to_string(),
        });
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();
        assert!(
            host.sync_workspace_for_parallel_resolve(
                corpus.as_slice(),
                &[],
                &[],
                &[],
                &resolution_inputs,
            )
            .is_none(),
            "a duplicate style_path corpus must bypass to the caller's serial arm",
        );
    }

    #[test]
    fn workspace_substrate_recomputes_only_changed_file_facts() {
        let corpus = parallel_probe_corpus();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let workspace = host.sync_workspace(corpus.as_slice(), &[], &[], &[], &resolution_inputs);

        style_fact_entry_probe::reset();
        {
            let _ = memo_workspace_diagnostics_substrate(&host.db, workspace);
        }
        assert_eq!(
            style_fact_entry_probe::read(),
            set_of([
                "/workspace/src/App.module.scss",
                "/workspace/src/_theme.scss",
            ]),
            "initial substrate build must collect facts for every style input",
        );

        style_fact_entry_probe::reset();
        {
            let _ = memo_workspace_diagnostics_substrate(&host.db, workspace);
        }
        assert_eq!(
            style_fact_entry_probe::read(),
            BTreeSet::new(),
            "unchanged workspace substrate must reuse per-file fact entries",
        );

        let mut edited_corpus = corpus.clone();
        edited_corpus[0]
            .style_source
            .push_str("\n.app__icon { color: blue; }\n");
        let edited_workspace =
            host.sync_workspace(edited_corpus.as_slice(), &[], &[], &[], &resolution_inputs);

        style_fact_entry_probe::reset();
        {
            let _ = memo_workspace_diagnostics_substrate(&host.db, edited_workspace);
        }
        assert_eq!(
            style_fact_entry_probe::read(),
            set_of(["/workspace/src/App.module.scss"]),
            "editing one file must not dirty unchanged file fact entries",
        );
    }

    #[test]
    fn module_interface_projection_preserves_body_only_edits() {
        let mut corpus = vec![OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/Card.module.scss".to_string(),
            style_source: ".card { color: red; }\n".to_string(),
        }];
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let workspace = host.sync_workspace(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        let file = workspace.files(&host.db)[0];
        let initial_fact = memo_style_fact_entry(&host.db, file);
        let initial_projection = memo_module_interface_projection(&host.db, file);

        corpus[0].style_source = ".card { color: blue; }\n".to_string();
        let edited_workspace =
            host.sync_workspace(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        let edited_file = edited_workspace.files(&host.db)[0];

        module_interface_projection_probe::reset();
        let edited_fact = memo_style_fact_entry(&host.db, edited_file);
        let edited_projection = memo_module_interface_projection(&host.db, edited_file);

        assert_ne!(
            initial_fact, edited_fact,
            "the body edit must change the underlying source-bearing fact entry",
        );
        assert_eq!(
            initial_projection, edited_projection,
            "body-only declarations must not change the cross-module interface projection",
        );
        assert_eq!(
            module_interface_projection_probe::read(),
            set_of(["/workspace/src/Card.module.scss"]),
            "the interface query should re-run only for the edited file",
        );
    }

    #[test]
    fn module_interface_projection_exposes_cross_boundary_surface() {
        let corpus = vec![OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/Card.module.scss".to_string(),
            style_source: r#"
@use "./theme" as theme;
@forward "./tokens" show $tone;
@value primary: #fff;
@value shadow as localShadow from "./tokens.module.css";
:import("./tokens.module.css") { imported: primary; }
:export { exported: primary; }
.card { composes: base utility from "./base.module.css"; color: primary; }
"#
            .to_string(),
        }];
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let workspace = host.sync_workspace(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        let file = workspace.files(&host.db)[0];
        let projection = memo_module_interface_projection(&host.db, file);
        let css_facts = &projection.css_modules_style_facts;

        assert_eq!(projection.style_path, "/workspace/src/Card.module.scss");
        assert!(css_facts.class_selector_names.contains(&"card".to_string()));
        assert!(
            css_facts
                .css_module_value_definition_names
                .contains(&"primary".to_string())
        );
        assert!(css_facts.css_module_value_import_edges.iter().any(|edge| {
            edge.local_name == "localShadow" && edge.import_source == "./tokens.module.css"
        }));
        assert!(css_facts.css_module_composes_edges.iter().any(|edge| {
            edge.owner_selector_names.contains(&"card".to_string())
                && edge.target_names.contains(&"base".to_string())
                && edge.import_source.as_deref() == Some("./base.module.css")
        }));
        assert!(
            css_facts
                .icss_export_names
                .contains(&"exported".to_string())
        );
        assert!(css_facts.icss_import_edges.iter().any(|edge| {
            edge.local_name == "imported" && edge.import_source == "./tokens.module.css"
        }));
        assert!(
            projection
                .sass_module_edges
                .iter()
                .any(|edge| { edge.kind == "sassUse" && edge.source == "./theme" })
        );
        assert!(
            projection
                .sass_module_edges
                .iter()
                .any(|edge| { edge.kind == "sassForward" && edge.source == "./tokens" })
        );
        assert!(
            projection
                .style_dependency_sources
                .contains(&"./base.module.css".to_string())
        );
        assert!(
            projection
                .style_dependency_sources
                .contains(&"./tokens.module.css".to_string())
        );
        assert!(
            projection
                .style_dependency_sources
                .contains(&"./theme".to_string())
        );
    }

    #[test]
    fn css_modules_resolution_from_module_interfaces_matches_fact_entry_resolution() {
        let corpus = css_modules_resolution_probe_corpus();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let workspace = host.sync_workspace(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        let style_fact_entries = workspace
            .files(&host.db)
            .iter()
            .map(|file| memo_style_fact_entry(&host.db, *file))
            .collect::<Vec<_>>();

        let direct = summarize_css_modules_cross_file_resolution(&style_fact_entries, &[]);
        let tracked =
            memo_css_modules_cross_file_resolution_from_module_interfaces(&host.db, workspace);

        assert_eq!(
            tracked, direct,
            "interface-fed CSS Modules resolution must match the fact-entry adapter",
        );
    }

    #[test]
    fn css_modules_resolution_backdates_after_module_interface_preserving_edit() {
        let mut corpus = css_modules_resolution_probe_corpus();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let workspace = host.sync_workspace(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        let edited_file = workspace.files(&host.db)[2];
        let initial_projection = memo_module_interface_projection(&host.db, edited_file);

        reset_css_modules_cross_file_resolution_compute_count_for_test();
        let initial_resolution =
            memo_css_modules_cross_file_resolution_from_module_interfaces(&host.db, workspace);
        assert_eq!(
            read_css_modules_cross_file_resolution_compute_count_for_test(),
            1
        );

        corpus[2].style_source = r#"@value primary as brand from "./tokens.module.css";
:import("./tokens.module.css") { imported: exported; }
:export { forwarded: imported; }
.card { composes: base from "./base.module.css"; color: brand; background: black; }
"#
        .to_string();
        let edited_workspace =
            host.sync_workspace(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        let edited_file = edited_workspace.files(&host.db)[2];
        let edited_projection = memo_module_interface_projection(&host.db, edited_file);
        assert_eq!(
            initial_projection, edited_projection,
            "the edited declaration value must not change the module interface",
        );

        reset_css_modules_cross_file_resolution_compute_count_for_test();
        let edited_resolution = memo_css_modules_cross_file_resolution_from_module_interfaces(
            &host.db,
            edited_workspace,
        );

        assert_eq!(edited_resolution, initial_resolution);
        assert_eq!(
            read_css_modules_cross_file_resolution_compute_count_for_test(),
            0,
            "interface-stable edits must not re-run CSS Modules cross-file resolution",
        );
    }

    #[test]
    fn css_modules_import_edges_from_module_interface_projection_recompute_only_import_dependents()
    -> Result<(), &'static str> {
        let mut corpus = css_modules_resolution_probe_corpus();
        corpus.push(OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/Unused.module.css".to_string(),
            style_source: ".unused { color: gray; }\n".to_string(),
        });
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let workspace = host.sync_workspace(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        let _ = memo_css_modules_cross_file_resolution_from_module_interfaces(&host.db, workspace);

        corpus[0].style_source = ".renamed { color: red; }\n".to_string();
        let edited_workspace =
            host.sync_workspace(corpus.as_slice(), &[], &[], &[], &resolution_inputs);

        reset_css_modules_import_edge_resolution_probe_for_test();
        let edited_resolution = memo_css_modules_cross_file_resolution_from_module_interfaces(
            &host.db,
            edited_workspace,
        );

        assert_eq!(
            read_css_modules_import_edge_resolution_probe_for_test(),
            set_of([
                "/workspace/src/base.module.css",
                "/workspace/src/Card.module.css",
            ]),
            "CSS Modules import-edge recomputation must stay scoped to the edited module and its importer",
        );
        let edge = edited_resolution
            .edges
            .iter()
            .find(|edge| {
                edge.from_style_path == "/workspace/src/Card.module.css"
                    && edge.import_kind == "composes"
                    && edge.source == "./base.module.css"
            })
            .ok_or("the Card CSS Modules composes edge should still be present")?;
        assert_eq!(
            edge.resolved_style_path.as_deref(),
            Some("/workspace/src/base.module.css")
        );
        assert_eq!(edge.status, "resolvedSourceNoNameMatch");
        assert!(edge.matched_names.is_empty());
        assert_eq!(edge.imported_names, vec!["base".to_string()]);
        assert_eq!(edge.exported_names, vec!["renamed".to_string()]);
        Ok(())
    }

    #[test]
    fn sass_module_resolution_from_module_interfaces_matches_fact_entry_resolution() {
        let corpus = sass_module_resolution_probe_corpus();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let workspace = host.sync_workspace(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        let style_fact_entries = workspace
            .files(&host.db)
            .iter()
            .map(|file| memo_style_fact_entry(&host.db, *file))
            .collect::<Vec<_>>();

        let direct =
            summarize_sass_module_cross_file_resolution(&style_fact_entries, &[], &[], &[]);
        let tracked =
            memo_sass_module_cross_file_resolution_from_module_interfaces(&host.db, workspace);

        assert_eq!(
            tracked, direct,
            "interface-fed Sass module resolution must match the fact-entry adapter",
        );
    }

    #[test]
    fn sass_module_resolution_backdates_after_module_interface_preserving_edit() {
        let mut corpus = sass_module_resolution_probe_corpus();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let workspace = host.sync_workspace(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        let edited_file = workspace.files(&host.db)[2];
        let initial_projection = memo_module_interface_projection(&host.db, edited_file);

        reset_sass_module_resolution_internal_compute_count_for_test();
        let initial_resolution =
            memo_sass_module_cross_file_resolution_from_module_interfaces(&host.db, workspace);
        assert_eq!(
            read_sass_module_resolution_internal_compute_count_for_test(),
            1
        );

        corpus[2].style_source = r#"@use "./theme.scss" as theme;
.app { color: theme.$brand; background: black; }
"#
        .to_string();
        let edited_workspace =
            host.sync_workspace(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        let edited_file = edited_workspace.files(&host.db)[2];
        let edited_projection = memo_module_interface_projection(&host.db, edited_file);
        assert_eq!(
            initial_projection, edited_projection,
            "the edited declaration value must not change the module interface",
        );

        reset_sass_module_resolution_internal_compute_count_for_test();
        let edited_resolution = memo_sass_module_cross_file_resolution_from_module_interfaces(
            &host.db,
            edited_workspace,
        );

        assert_eq!(edited_resolution, initial_resolution);
        assert_eq!(
            read_sass_module_resolution_internal_compute_count_for_test(),
            0,
            "interface-stable edits must not re-run Sass module resolution",
        );
    }

    #[test]
    fn sass_module_edges_from_module_interface_projection_recompute_only_config_dependents()
    -> Result<(), &'static str> {
        let mut corpus = vec![
            OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace/src/tokens.scss".to_string(),
                style_source: "$brand: red !default;\n.token { color: $brand; }\n".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace/src/app.scss".to_string(),
                style_source: r#"@use "./tokens.scss" with ($brand: blue);
.app { color: tokens.$brand; }
"#
                .to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace/src/unused.scss".to_string(),
                style_source: ".unused { color: gray; }\n".to_string(),
            },
        ];
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let workspace = host.sync_workspace(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        let _ = memo_sass_module_cross_file_resolution_from_module_interfaces(&host.db, workspace);
        let _ = memo_sass_module_cross_file_resolution_without_manifests_from_module_interfaces(
            &host.db, workspace,
        );
        let _ = memo_sass_module_cross_file_resolution_without_path_mappings_from_module_interfaces(
            &host.db, workspace,
        );

        corpus[0].style_source = "$tone: red !default;\n.token { color: $tone; }\n".to_string();
        let edited_workspace =
            host.sync_workspace(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        let expected_recompute_set =
            set_of(["/workspace/src/app.scss", "/workspace/src/tokens.scss"]);
        let assert_configured_edge =
            |resolution: &OmenaQuerySassModuleCrossFileResolutionV0| -> Result<(), &'static str> {
                let edge = resolution
                    .edges
                    .iter()
                    .find(|edge| {
                        edge.from_style_path == "/workspace/src/app.scss"
                            && edge.edge_kind == "sassUse"
                            && edge.source == "./tokens.scss"
                    })
                    .ok_or("the configured Sass use edge should still be present")?;
                assert_eq!(
                    edge.resolved_style_path.as_deref(),
                    Some("/workspace/src/tokens.scss")
                );
                assert_eq!(edge.status, "resolved");
                assert_eq!(edge.configuration_variable_count, 1);
                assert_eq!(
                    edge.invalid_configuration_variable_names,
                    vec!["brand".to_string()]
                );
                Ok(())
            };

        reset_sass_module_edge_resolution_probe_for_test();
        let edited_resolution = memo_sass_module_cross_file_resolution_from_module_interfaces(
            &host.db,
            edited_workspace,
        );

        assert_eq!(
            read_sass_module_edge_resolution_probe_for_test(),
            expected_recompute_set,
            "Sass edge recomputation must stay scoped to the edited module and its configured importer",
        );
        assert_configured_edge(&edited_resolution)?;

        reset_sass_module_edge_resolution_probe_for_test();
        let without_manifests =
            memo_sass_module_cross_file_resolution_without_manifests_from_module_interfaces(
                &host.db,
                edited_workspace,
            );
        assert_eq!(
            read_sass_module_edge_resolution_probe_for_test(),
            expected_recompute_set,
            "manifest-independent Sass edge recomputation must stay scoped to the edited module and its configured importer",
        );
        assert_configured_edge(&without_manifests)?;

        reset_sass_module_edge_resolution_probe_for_test();
        let without_path_mappings =
            memo_sass_module_cross_file_resolution_without_path_mappings_from_module_interfaces(
                &host.db,
                edited_workspace,
            );
        assert_eq!(
            read_sass_module_edge_resolution_probe_for_test(),
            expected_recompute_set,
            "path-mapping-independent Sass edge recomputation must stay scoped to the edited module and its configured importer",
        );
        assert_configured_edge(&without_path_mappings)?;
        Ok(())
    }

    #[test]
    fn committed_style_semantic_graph_from_module_interface_projection_matches_monolith() {
        let corpus = sass_module_resolution_probe_corpus();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let workspace = host.sync_workspace(corpus.as_slice(), &[], &[], &[], &resolution_inputs);

        let decomposed =
            memo_committed_style_semantic_graph_from_module_interfaces(&host.db, workspace);
        let monolith = build_committed_style_semantic_graph_monolith(
            &host.db,
            workspace,
            &[],
            &[],
            &[],
            &resolution_inputs,
        );

        assert_eq!(
            decomposed, monolith,
            "interface-fed committed graph must match the retained monolith",
        );
    }

    #[test]
    fn committed_style_semantic_graph_backdates_module_interface_stable_cross_file_layers() {
        let mut corpus = sass_module_resolution_probe_corpus();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let workspace = host.sync_workspace(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        let edited_file = workspace.files(&host.db)[2];
        let initial_fact = memo_style_fact_entry(&host.db, edited_file);
        let initial_projection = memo_module_interface_projection(&host.db, edited_file);
        let _ = memo_committed_style_semantic_graph_from_module_interfaces(&host.db, workspace);

        corpus[2].style_source = r#"@use "./theme.scss" as theme;
.app { color: theme.$brand; background: black; }
"#
        .to_string();
        let edited_workspace =
            host.sync_workspace(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        let edited_file = edited_workspace.files(&host.db)[2];
        let edited_fact = memo_style_fact_entry(&host.db, edited_file);
        let edited_projection = memo_module_interface_projection(&host.db, edited_file);
        assert_ne!(
            initial_fact, edited_fact,
            "the body edit must change the underlying style fact entry",
        );
        assert_eq!(
            initial_projection, edited_projection,
            "the body edit must preserve the module interface",
        );

        reset_css_modules_cross_file_resolution_compute_count_for_test();
        reset_sass_module_resolution_internal_compute_count_for_test();
        reset_workspace_cross_file_summary_internal_compute_count_for_test();
        let _ =
            memo_committed_style_semantic_graph_from_module_interfaces(&host.db, edited_workspace);

        assert_eq!(
            read_css_modules_cross_file_resolution_compute_count_for_test(),
            0,
            "interface-stable edits must not re-run CSS Modules resolution",
        );
        assert_eq!(
            read_sass_module_resolution_internal_compute_count_for_test(),
            0,
            "interface-stable edits must not re-run Sass module resolution variants",
        );
        assert_eq!(
            read_workspace_cross_file_summary_internal_compute_count_for_test(),
            0,
            "interface-stable edits must not re-run cross-file summary layers",
        );
    }

    #[test]
    fn committed_style_semantic_graph_from_module_interface_projection_is_order_independent() {
        let corpus = sass_module_resolution_probe_corpus();
        let reversed = corpus.iter().cloned().rev().collect::<Vec<_>>();
        let rotated = [corpus[1].clone(), corpus[2].clone(), corpus[0].clone()].to_vec();
        let baseline = committed_graph_from_corpus_order(corpus.as_slice());

        assert_eq!(
            committed_graph_from_corpus_order(reversed.as_slice()),
            baseline,
            "reversed file order must not change the committed graph",
        );
        assert_eq!(
            committed_graph_from_corpus_order(rotated.as_slice()),
            baseline,
            "rotated file order must not change the committed graph",
        );
    }

    #[test]
    fn workspace_substrate_recompute_set_is_size_invariant() {
        assert_changed_file_recompute_set(parallel_probe_corpus());
        assert_changed_file_recompute_set(doubled_parallel_probe_corpus());
    }

    #[test]
    fn evidence_graph_keys_changed_nodes_on_salsa_demand_edges() -> Result<(), &'static str> {
        let mut corpus = doubled_parallel_probe_corpus();
        let edited_path = corpus[0].style_path.clone();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let workspace = host.sync_workspace(corpus.as_slice(), &[], &[], &[], &resolution_inputs);

        style_fact_entry_probe::reset();
        {
            let _ = memo_workspace_diagnostics_substrate(&host.db, workspace);
        }

        corpus[0]
            .style_source
            .push_str("\n.app__icon { color: blue; }\n");
        let edited_workspace =
            host.sync_workspace(corpus.as_slice(), &[], &[], &[], &resolution_inputs);

        style_fact_entry_probe::reset();
        {
            let _ = memo_workspace_diagnostics_substrate(&host.db, edited_workspace);
        }
        let demand_paths = style_fact_entry_probe::read();
        assert_eq!(
            demand_paths,
            BTreeSet::from([edited_path.clone()]),
            "the salsa firewall must provide the inherited demand-edge precondition",
        );

        let all_node_seeds = corpus.iter().map(|source| {
            EvidenceNodeSeedV0::new(
                EvidenceNodeKeyV0::new("memo_style_fact_entry", source.style_path.as_str()),
                Vec::new(),
                GuaranteeKindV0::for_label_less_family(),
            )
        });
        let demand_edges = demand_paths.iter().map(|style_path| {
            EvidenceDemandEdgeV0::new(
                "memo_workspace_diagnostics_substrate",
                EvidenceNodeKeyV0::new("memo_style_fact_entry", style_path.as_str()),
                "salsa-demand-read",
            )
        });
        let graph = build_salsa_demand_evidence_graph_v0(all_node_seeds, demand_edges)
            .map_err(|_| "salsa demand edges must target known workspace style nodes")?;

        assert_eq!(
            graph.node_input_identities(),
            BTreeSet::from([edited_path.clone()]),
            "the evidence graph must key changed nodes on demand edges, not the full workspace list",
        );
        assert_eq!(
            graph.edge_input_identities(),
            BTreeSet::from([edited_path]),
            "the evidence graph must expose only the changed salsa demand edge",
        );
        Ok(())
    }

    fn committed_graph_from_corpus_order(
        corpus: &[OmenaQueryStyleSourceInputV0],
    ) -> OmenaQueryCommittedStyleSemanticGraphV0 {
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let workspace = host.sync_workspace(corpus, &[], &[], &[], &resolution_inputs);
        memo_committed_style_semantic_graph_from_module_interfaces(&host.db, workspace)
    }

    fn assert_changed_file_recompute_set(mut corpus: Vec<OmenaQueryStyleSourceInputV0>) {
        let edited_path = corpus[0].style_path.clone();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let workspace = host.sync_workspace(corpus.as_slice(), &[], &[], &[], &resolution_inputs);

        style_fact_entry_probe::reset();
        {
            let _ = memo_workspace_diagnostics_substrate(&host.db, workspace);
        }

        corpus[0]
            .style_source
            .push_str("\n.app__icon { color: blue; }\n");
        let edited_workspace =
            host.sync_workspace(corpus.as_slice(), &[], &[], &[], &resolution_inputs);

        style_fact_entry_probe::reset();
        {
            let _ = memo_workspace_diagnostics_substrate(&host.db, edited_workspace);
        }

        assert_eq!(
            style_fact_entry_probe::read(),
            BTreeSet::from([edited_path]),
            "editing one file must re-run exactly that file's fact entry regardless of corpus size",
        );
    }

    /// Parallel read bundles are selector-owned snapshots, not live host
    /// handles; a post-wave edit must proceed even after a parallel read.
    #[test]
    fn post_wave_edit_writes_proceed_after_selector_reads() -> Result<(), &'static str> {
        let corpus = parallel_probe_corpus();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();

        let sync = host
            .sync_workspace_for_parallel_resolve(
                corpus.as_slice(),
                &[],
                &[],
                &[],
                &resolution_inputs,
            )
            .ok_or("duplicate-free corpus must sync for parallel resolve")?;
        let workspace = sync.workspace;
        let committed_graph = sync.committed_graph.clone();
        std::thread::scope(|scope| {
            for (_, file) in sync.files.iter() {
                let handle = sync.handle.clone();
                let committed_graph = committed_graph.clone();
                let file = *file;
                scope.spawn(move || {
                    let db = OmenaQueryStyleMemoDatabaseV0::from_handle(handle);
                    let _ = resolve_committed_workspace_style_diagnostics_from_view(
                        &db,
                        workspace,
                        file,
                        &committed_graph,
                    );
                });
            }
        });
        drop(sync);

        let mut edited_corpus = corpus.clone();
        let edited_entry = edited_corpus.first_mut().ok_or("non-empty probe corpus")?;
        edited_entry
            .style_source
            .push_str("\n.after-wave { @extend %missing-after-wave; }\n");
        let (sender, receiver) = std::sync::mpsc::channel();
        let writer = std::thread::spawn(move || {
            let edited = host.workspace_style_diagnostics(
                "/workspace/src/App.module.scss",
                edited_corpus.as_slice(),
                &[],
                &[],
                &[],
                &resolution_inputs,
            );
            sender.send(serde_json::to_string(&edited).ok()).ok();
        });
        let edited_json = receiver
            .recv_timeout(std::time::Duration::from_secs(30))
            .map_err(|_| "post-wave set_* did not complete after selector-backed reads")?;
        writer
            .join()
            .map_err(|_| "post-wave edit resolve panicked")?;
        assert!(
            edited_json.is_some(),
            "post-wave edit resolve must serialize",
        );
        Ok(())
    }
}
