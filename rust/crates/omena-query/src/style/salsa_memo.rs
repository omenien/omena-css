//! Salsa-backed memoized style-diagnostics query layer.
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
    OmenaQueryExternalSifResolutionContext, OmenaQueryUnusedSelectorSharedV0,
    collect_omena_query_workspace_diagnostics_substrate_from_committed_graph,
    promote_sif_backed_external_edges,
    summarize_omena_query_sass_module_resolution_identity_diagnostics_for_workspace_from_resolution,
};
use super::*;
use omena_syntax::ident::{AuthoredPropertyTextV0, PropertyNameV0, property_names_same};
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

#[cfg(test)]
mod memo_style_cascade_projection_probe {
    use std::cell::Cell;

    thread_local! {
        static RUN_COUNT: Cell<u64> = const { Cell::new(0) };
    }

    pub(super) fn record() {
        RUN_COUNT.with(|count| count.set(count.get().saturating_add(1)));
    }

    pub(super) fn reset() {
        RUN_COUNT.with(|count| count.set(0));
    }

    pub(super) fn count() -> u64 {
        RUN_COUNT.with(Cell::get)
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

#[cfg(test)]
mod package_manifest_projection_probe {
    use std::cell::RefCell;
    use std::collections::BTreeSet;

    thread_local! {
        static RUN_PATHS: RefCell<BTreeSet<String>> = const { RefCell::new(BTreeSet::new()) };
    }

    pub(super) fn record(package_json_path: &str) {
        RUN_PATHS.with(|paths| {
            paths.borrow_mut().insert(package_json_path.to_string());
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
#[allow(dead_code)]
mod source_workspace_projection_probe {
    use std::cell::RefCell;
    use std::collections::BTreeSet;

    thread_local! {
        static RUN_PATHS: RefCell<BTreeSet<String>> = const { RefCell::new(BTreeSet::new()) };
    }

    pub(super) fn record(source_path: &str) {
        RUN_PATHS.with(|paths| {
            paths.borrow_mut().insert(source_path.to_string());
        });
    }

    pub(super) fn reset() {
        RUN_PATHS.with(|paths| paths.borrow_mut().clear());
    }

    pub(super) fn read() -> BTreeSet<String> {
        RUN_PATHS.with(|paths| paths.borrow().clone())
    }
}

#[cfg(test)]
mod source_workspace_query_probe {
    use std::cell::Cell;

    thread_local! {
        static UNUSED_SELECTOR_COMPUTES: Cell<u64> = const { Cell::new(0) };
        static CROSS_FILE_SUMMARY_COMPUTES: Cell<u64> = const { Cell::new(0) };
    }

    pub(super) fn record_unused_selector() {
        UNUSED_SELECTOR_COMPUTES.with(|count| count.set(count.get().saturating_add(1)));
    }

    pub(super) fn record_cross_file_summary() {
        CROSS_FILE_SUMMARY_COMPUTES.with(|count| count.set(count.get().saturating_add(1)));
    }

    pub(super) fn reset() {
        UNUSED_SELECTOR_COMPUTES.with(|count| count.set(0));
        CROSS_FILE_SUMMARY_COMPUTES.with(|count| count.set(0));
    }

    pub(super) fn read() -> (u64, u64) {
        (
            UNUSED_SELECTOR_COMPUTES.with(Cell::get),
            CROSS_FILE_SUMMARY_COMPUTES.with(Cell::get),
        )
    }
}

#[cfg(test)]
mod source_element_parent_chain_probe {
    use std::cell::RefCell;
    use std::collections::BTreeSet;

    thread_local! {
        static RUN_PATHS: RefCell<BTreeSet<String>> = const { RefCell::new(BTreeSet::new()) };
    }

    pub(super) fn record(source_path: &str) {
        RUN_PATHS.with(|paths| {
            paths.borrow_mut().insert(source_path.to_string());
        });
    }

    pub(super) fn reset() {
        RUN_PATHS.with(|paths| paths.borrow_mut().clear());
    }

    pub(super) fn read() -> BTreeSet<String> {
        RUN_PATHS.with(|paths| paths.borrow().clone())
    }
}

#[cfg(test)]
pub fn reset_source_element_parent_chain_run_paths_for_test() {
    source_element_parent_chain_probe::reset();
}

#[cfg(test)]
pub fn read_source_element_parent_chain_run_paths_for_test() -> BTreeSet<String> {
    source_element_parent_chain_probe::read()
}

#[cfg(test)]
mod source_element_computed_value_probe {
    use std::cell::Cell;

    thread_local! {
        static COMPUTE_COUNT: Cell<u64> = const { Cell::new(0) };
    }

    pub(super) fn record() {
        COMPUTE_COUNT.with(|count| count.set(count.get() + 1));
    }

    pub(super) fn reset() {
        COMPUTE_COUNT.with(|count| count.set(0));
    }

    pub(super) fn read() -> u64 {
        COMPUTE_COUNT.with(Cell::get)
    }
}

#[cfg(test)]
pub fn reset_source_element_computed_value_compute_count_for_test() {
    source_element_computed_value_probe::reset();
}

#[cfg(test)]
pub fn read_source_element_computed_value_compute_count_for_test() -> u64 {
    source_element_computed_value_probe::read()
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

#[cfg(feature = "test-support")]
#[allow(dead_code)]
pub fn reset_source_workspace_projection_probe_for_test() {
    source_workspace_projection_probe::reset();
}

#[cfg(feature = "test-support")]
#[allow(dead_code)]
pub fn read_source_workspace_projection_probe_for_test() -> BTreeSet<String> {
    source_workspace_projection_probe::read()
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

/// One source file in the workspace projection. Keeping text and precomputed
/// syntax facts on path-stable input entities lets ancestry queries depend on
/// only the files traversed by a parent chain.
#[salsa::input]
pub struct OmenaQuerySourceFileInputV0 {
    #[returns(ref)]
    pub source_path: String,
    #[returns(ref)]
    pub source_source: String,
    #[returns(ref)]
    pub source_syntax_index: Option<OmenaQuerySourceSyntaxIndexV0>,
    pub has_unresolved_style_import: bool,
}

/// One path-stable package manifest. The workspace keeps stable entity ids so
/// changing one manifest source invalidates only that manifest's projection.
#[salsa::input]
#[doc(hidden)]
pub struct OmenaQueryStylePackageManifestInputV0 {
    #[returns(ref)]
    package_json_path: String,
    #[returns(ref)]
    package_json_source: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaQueryElementComputedValueStatusV0 {
    Resolved,
    IncompleteParentChain,
    MissingElement,
    DynamicDeclaration,
    UnsupportedStaticValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryElementComputedValueV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub target: omena_cascade::ElementIdentityV0,
    pub property: String,
    pub status: OmenaQueryElementComputedValueStatusV0,
    pub parent_chain: omena_cascade::ElementParentChainV0,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub computed_value: Option<omena_cascade::CascadeComputedValueResultV0>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceElementDeclarationProjectionV0 {
    declarations: Vec<omena_cascade::CascadeDeclaration>,
    status: OmenaQueryElementComputedValueStatusV0,
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
    pub source_files: Vec<OmenaQuerySourceFileInputV0>,
    #[returns(ref)]
    pub package_manifests: Vec<OmenaQueryStylePackageManifestV0>,
    #[returns(ref)]
    pub external_sifs: Vec<OmenaQueryExternalSifInputV0>,
    #[returns(ref)]
    pub resolution_inputs: OmenaQueryStyleResolutionInputsV0,
    #[default]
    #[returns(ref)]
    package_manifest_inputs: Vec<OmenaQueryStylePackageManifestInputV0>,
    #[default]
    #[returns(ref)]
    resolution_package_manifest_inputs: Vec<OmenaQueryStylePackageManifestInputV0>,
    #[default]
    #[returns(ref)]
    resolution_tsconfig_path_mappings: Vec<OmenaQueryTsconfigPathMappingV0>,
    #[default]
    #[returns(ref)]
    resolution_bundler_path_mappings: Vec<OmenaQueryBundlerPathAliasMappingV0>,
    #[default]
    #[returns(ref)]
    resolution_disk_style_path_identities: Vec<OmenaQueryStyleModuleDiskCandidateIdentityV0>,
    #[default]
    #[returns(ref)]
    resolution_external_sif_cache_fingerprint: Option<String>,
    #[default]
    #[returns(copy)]
    granular_inputs_initialized: bool,
}

/// One committed selector, many worker-side read views. Produced by
/// [`OmenaQueryStyleMemoHostV0::sync_workspace_for_parallel_resolve`] after the
/// host commits the wave and builds an independent selector read database; the
/// embedded `handle` pins that selector snapshot and is `Send`, so a parallel
/// wave rebuilds per-worker views via
/// [`OmenaQueryStyleMemoDatabaseV0::from_handle`].
pub struct OmenaQueryStyleParallelResolveSyncV0 {
    pub revision: IncrementalRevisionV0,
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
    cascade_declarations_by_style:
        BTreeMap<String, Vec<cascade_checker::QueryCheckerCascadeDeclaration>>,
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
    changed_module_interface_paths: BTreeSet<String>,
    committed_graph: OmenaQueryCommittedStyleSemanticGraphV0,
    resolver_identity_index: Option<OmenaResolverStyleModuleConfirmationIdentityIndexV0>,
    source_corpus_complete: bool,
    unused_selector_shared: std::sync::OnceLock<Option<OmenaQueryUnusedSelectorSharedV0>>,
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

    pub fn snapshot_id(&self) -> OmenaWorkspaceSnapshotIdV0 {
        OmenaWorkspaceSnapshotIdV0::from_revision(self.revision)
    }

    pub fn changed_module_interface_paths(&self) -> &BTreeSet<String> {
        &self.changed_module_interface_paths
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
        let unused_selector_shared = self.unused_selector_shared.get_or_init(|| {
            let resolution_inputs = style_resolution_inputs_for_workspace(&self.db, self.workspace);
            let package_manifests = package_manifests_for_workspace(&self.db, self.workspace);
            crate::style::diagnostics::collect_omena_query_unused_selector_shared(
                self.committed_graph.style_fact_entries.as_slice(),
                self.workspace.source_documents(&self.db).as_slice(),
                package_manifests.as_slice(),
                None,
                resolution_inputs.bundler_path_mappings.as_slice(),
                resolution_inputs.tsconfig_path_mappings.as_slice(),
                resolution_inputs.disk_style_path_identities.as_slice(),
                self.resolver_identity_index.as_ref(),
                self.source_corpus_complete,
            )
        });
        resolve_committed_workspace_style_diagnostics_from_view_with_external_mode_and_suppression_mode_and_precomputed_unused_selector_and_identity_index(
            &self.db,
            self.workspace,
            target,
            &self.committed_graph,
            external_mode,
            suppression_mode,
            None,
            self.resolver_identity_index.as_ref(),
            Some(unused_selector_shared),
            self.source_corpus_complete,
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
                declarations: self
                    .committed_graph
                    .cascade_declarations_by_style
                    .get(entry.style_path.as_str())
                    .cloned()
                    .unwrap_or_default(),
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
        let package_manifests = package_manifests_for_workspace(&self.db, self.workspace);
        let resolution_inputs = style_resolution_inputs_for_workspace(&self.db, self.workspace);
        summarize_omena_query_style_completion_for_workspace_file_with_substrate(
            target_style_path,
            style_sources.as_slice(),
            package_manifests.as_slice(),
            self.workspace.external_sifs(&self.db).as_slice(),
            &resolution_inputs,
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
            input,
            package_manifests,
            &style_resolution_inputs_for_workspace(&self.db, self.workspace),
            OmenaQueryStyleSemanticGraphCommittedParts {
                style_fact_entries: self.committed_graph.style_fact_entries.as_slice(),
                cross_file_summary: self.committed_graph.style_cross_file_summary.clone(),
                css_modules_resolution: self.committed_graph.css_modules_resolution.clone(),
                sass_module_resolution: self.committed_graph.sass_module_resolution.clone(),
            },
        )
    }

    pub fn into_parallel_resolve_sync(self) -> OmenaQueryStyleParallelResolveSyncV0 {
        OmenaQueryStyleParallelResolveSyncV0 {
            revision: self.revision,
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
    pub changed_module_interface_paths: BTreeSet<String>,
    pub selector: OmenaQueryStyleRevisionSelectorV0,
}

struct OmenaQueryStyleWorkspaceTransactionCoreCommitV0 {
    revision: IncrementalRevisionV0,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    files: Vec<(String, OmenaQueryStyleFileInputV0)>,
    changed_style_paths: BTreeSet<String>,
    changed_module_interface_paths: BTreeSet<String>,
    style_sources: Vec<OmenaQueryStyleSourceInputV0>,
    source_documents: Vec<OmenaQuerySourceDocumentInputV0>,
    package_manifests: Vec<OmenaQueryStylePackageManifestV0>,
    external_sifs: Vec<OmenaQueryExternalSifInputV0>,
    resolution_inputs: OmenaQueryStyleResolutionInputsV0,
    resolver_identity_index: Option<OmenaResolverStyleModuleConfirmationIdentityIndexV0>,
    source_corpus_complete: bool,
    committed_graph: OmenaQueryCommittedStyleSemanticGraphV0,
}

struct OmenaQueryStyleRevisionSelectorBuildInputV0<'a> {
    revision: IncrementalRevisionV0,
    style_sources: &'a [OmenaQueryStyleSourceInputV0],
    source_documents: &'a [OmenaQuerySourceDocumentInputV0],
    package_manifests: &'a [OmenaQueryStylePackageManifestV0],
    external_sifs: &'a [OmenaQueryExternalSifInputV0],
    resolution_inputs: &'a OmenaQueryStyleResolutionInputsV0,
    resolver_identity_index: &'a Option<OmenaResolverStyleModuleConfirmationIdentityIndexV0>,
    source_corpus_complete: bool,
    committed_graph: OmenaQueryCommittedStyleSemanticGraphV0,
    changed_module_interface_paths: BTreeSet<String>,
}

pub struct OmenaQueryStyleDiagnosticsWithSelectorV0 {
    pub diagnostics: OmenaQueryStyleDiagnosticsForFileV0,
    pub selector: OmenaQueryStyleRevisionSelectorV0,
    pub snapshot_id: OmenaWorkspaceSnapshotIdV0,
}

impl OmenaQueryStyleDiagnosticsWithSelectorV0 {
    pub fn snapshot_id(&self) -> OmenaWorkspaceSnapshotIdV0 {
        self.snapshot_id
    }
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
    resolver_identity_index: Option<OmenaResolverStyleModuleConfirmationIdentityIndexV0>,
    source_corpus_complete: bool,
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

    fn set_resolver_identity_index(
        &mut self,
        resolver_identity_index: &OmenaResolverStyleModuleConfirmationIdentityIndexV0,
    ) -> &mut Self {
        self.resolver_identity_index = Some(resolver_identity_index.clone());
        self
    }

    fn mark_source_corpus_complete(&mut self) -> &mut Self {
        self.source_corpus_complete = true;
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
        None,
        Some(resolver_identity_index),
    )
}

/// Target-independent per-wave state: the corpus snapshot and diagnostics
/// substrate are hoisted out of the per-target resolve. Opaque to
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
    // Compute target-independent pass cores once per wave. Arguments mirror
    // the per-target dispatch exactly: the
    // committed arm passes classname_transform = None (as the per-target
    // wrapper does) and the same resolution inputs and identity index the
    // targets will resolve with.
    let source_documents = workspace.source_documents(db);
    let package_manifests = package_manifests_for_workspace(db, workspace);
    let resolution_inputs = style_resolution_inputs_for_workspace(db, workspace);
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
            false,
        ),
        inline_style_overrides_by_style: Some(
            crate::style::diagnostics::collect_omena_query_inline_style_runtime_overrides_by_style(
                corpus.as_slice(),
                source_documents.as_slice(),
                &resolution_inputs,
                resolver_identity_index,
            ),
        ),
        #[cfg(feature = "hypergraph-monotone-fact-propagation")]
        cross_file_scc_report: Some(
            crate::style::diagnostics::collect_omena_query_unified_cross_file_scc_report_shared(
                corpus.as_slice(),
                source_documents.as_slice(),
                package_manifests.as_slice(),
                &resolution_inputs,
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
    let package_manifests = package_manifests_for_workspace(db, workspace);
    let external_sifs = workspace.external_sifs(db);
    let resolution_inputs = style_resolution_inputs_for_workspace(db, workspace);
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs_and_suppression_mode_with_substrate_and_shared(
        target_style_path.as_str(),
        wave_substrate.corpus.as_slice(),
        source_documents.as_slice(),
        package_manifests.as_slice(),
        None,
        OmenaQueryExternalModuleModeV0::Auto,
        external_sifs.as_slice(),
        &resolution_inputs,
        OmenaQueryDiagnosticSuppressionModeV0::Apply,
        &wave_substrate.substrate,
        Some(resolver_identity_index),
        false,
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
    classname_transform: Option<&str>,
    resolver_identity_index: Option<&OmenaResolverStyleModuleConfirmationIdentityIndexV0>,
) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
    resolve_committed_workspace_style_diagnostics_from_view_with_external_mode_and_suppression_mode_and_precomputed_unused_selector_and_identity_index(
        db,
        workspace,
        target,
        committed_graph,
        external_mode,
        suppression_mode,
        classname_transform,
        resolver_identity_index,
        None,
        false,
    )
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
fn resolve_committed_workspace_style_diagnostics_from_view_with_external_mode_and_suppression_mode_and_precomputed_unused_selector(
    db: &OmenaQueryStyleMemoDatabaseV0,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    target: OmenaQueryStyleFileInputV0,
    committed_graph: &OmenaQueryCommittedStyleSemanticGraphV0,
    external_mode: OmenaQueryExternalModuleModeV0,
    suppression_mode: OmenaQueryDiagnosticSuppressionModeV0,
    precomputed_unused_selector: Option<&Option<OmenaQueryUnusedSelectorSharedV0>>,
    source_corpus_complete: bool,
) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
    resolve_committed_workspace_style_diagnostics_from_view_with_external_mode_and_suppression_mode_and_precomputed_unused_selector_and_identity_index(
        db,
        workspace,
        target,
        committed_graph,
        external_mode,
        suppression_mode,
        None,
        None,
        precomputed_unused_selector,
        source_corpus_complete,
    )
}

#[allow(clippy::too_many_arguments)]
fn resolve_committed_workspace_style_diagnostics_from_view_with_external_mode_and_suppression_mode_and_precomputed_unused_selector_and_identity_index(
    db: &OmenaQueryStyleMemoDatabaseV0,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    target: OmenaQueryStyleFileInputV0,
    committed_graph: &OmenaQueryCommittedStyleSemanticGraphV0,
    external_mode: OmenaQueryExternalModuleModeV0,
    suppression_mode: OmenaQueryDiagnosticSuppressionModeV0,
    classname_transform: Option<&str>,
    resolver_identity_index: Option<&OmenaResolverStyleModuleConfirmationIdentityIndexV0>,
    precomputed_unused_selector: Option<&Option<OmenaQueryUnusedSelectorSharedV0>>,
    source_corpus_complete: bool,
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
    let package_manifests = package_manifests_for_workspace(db, workspace);
    let external_sifs = workspace.external_sifs(db);
    let resolution_inputs = style_resolution_inputs_for_workspace(db, workspace);
    let substrate = collect_omena_query_workspace_diagnostics_substrate_from_committed_graph(
        committed_graph.style_fact_entries.clone(),
        &committed_graph.css_modules_resolution,
        &committed_graph.sass_module_resolution,
        &committed_graph.sass_module_resolution_without_manifests,
        &committed_graph.sass_module_resolution_without_path_mappings,
        &committed_graph.sass_module_resolution_with_external_sifs,
    );
    // The default committed path freezes source-selector attribution with the
    // workspace revision. Non-default naming or identity semantics retain the
    // direct collector because either axis changes the shared result.
    let precomputed_unused_selector = precomputed_unused_selector.or_else(|| {
        (classname_transform.is_none() && resolver_identity_index.is_none())
            .then(|| memo_workspace_unused_selector_shared(db, workspace, source_corpus_complete))
    });
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs_and_suppression_mode_with_substrate_and_shared(
        target_style_path.as_str(),
        corpus.as_slice(),
        source_documents.as_slice(),
        package_manifests.as_slice(),
        classname_transform,
        external_mode,
        external_sifs.as_slice(),
        &resolution_inputs,
        suppression_mode,
        &substrate,
        resolver_identity_index,
        source_corpus_complete,
        precomputed_unused_selector,
    )
}

/// Cache source-selector attribution once for every workspace revision.
///
/// Resolution mappings and disk identities are read through the workspace
/// input so Salsa invalidates this result with the same revision that owns the
/// source documents. Unchanged file facts remain memoized across revisions.
#[salsa::tracked(returns(clone))]
fn memo_source_workspace_projection(
    db: &dyn salsa::Database,
    file: OmenaQuerySourceFileInputV0,
) -> OmenaQuerySourceDocumentInputV0 {
    #[cfg(any(test, feature = "test-support"))]
    source_workspace_projection_probe::record(file.source_path(db));
    let source_syntax_index = file.source_syntax_index(db).clone();
    let source_source =
        source_workspace_projected_text(file.source_source(db), source_syntax_index.as_ref());
    OmenaQuerySourceDocumentInputV0 {
        source_path: file.source_path(db).clone(),
        source_source,
        source_syntax_index,
        has_unresolved_style_import: *file.has_unresolved_style_import(db),
    }
}

fn source_workspace_projected_text(
    source: &str,
    source_syntax_index: Option<&OmenaQuerySourceSyntaxIndexV0>,
) -> String {
    let Some(index) = source_syntax_index else {
        return source.to_string();
    };
    let index_is_consumable = !index.imported_style_bindings.is_empty()
        || index
            .selector_references
            .iter()
            .any(|reference| reference.target_style_uri.is_some());
    if !index_is_consumable {
        return source.to_string();
    }
    let mut spans = index
        .selector_references
        .iter()
        .filter(|reference| reference.selector_name.is_none())
        .map(|reference| reference.byte_span)
        .collect::<Vec<_>>();
    spans.extend(index.style_property_accesses.iter().filter_map(|access| {
        (!index.selector_references.iter().any(|reference| {
            reference.byte_span == access.byte_span
                && reference.target_style_uri == access.target_style_uri
                && reference.selector_name.is_some()
        }))
        .then_some(access.byte_span)
    }));
    if spans.is_empty() {
        return String::new();
    }
    let mut projected = vec![b' '; source.len()];
    for span in spans {
        let Some(fragment) = source.as_bytes().get(span.start..span.end) else {
            return source.to_string();
        };
        projected[span.start..span.end].copy_from_slice(fragment);
    }
    String::from_utf8(projected).unwrap_or_else(|_| source.to_string())
}

fn source_workspace_projections(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> Vec<OmenaQuerySourceDocumentInputV0> {
    workspace
        .source_files(db)
        .iter()
        .map(|file| memo_source_workspace_projection(db, *file))
        .collect()
}

#[salsa::tracked(returns(ref))]
fn memo_workspace_unused_selector_shared(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    source_corpus_complete: bool,
) -> Option<OmenaQueryUnusedSelectorSharedV0> {
    #[cfg(test)]
    source_workspace_query_probe::record_unused_selector();
    let style_fact_entries = workspace
        .files(db)
        .iter()
        .map(|file| memo_style_fact_entry(db, *file))
        .collect::<Vec<_>>();
    crate::style::diagnostics::collect_omena_query_unused_selector_shared(
        style_fact_entries.as_slice(),
        source_workspace_projections(db, workspace).as_slice(),
        package_manifests_for_workspace(db, workspace).as_slice(),
        None,
        resolution_bundler_path_mappings_for_workspace(db, workspace),
        resolution_tsconfig_path_mappings_for_workspace(db, workspace),
        resolution_disk_style_path_identities_for_workspace(db, workspace),
        None,
        source_corpus_complete,
    )
}

/// Target-independent diagnostics substrate hoisted into a workspace-keyed
/// tracked query so N open targets share one substrate build per revision
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
    collect_omena_query_workspace_diagnostics_substrate_from_entries(
        style_fact_entries,
        package_manifests_for_workspace(db, workspace).as_slice(),
        workspace.external_sifs(db).as_slice(),
        resolution_bundler_path_mappings_for_workspace(db, workspace),
        resolution_tsconfig_path_mappings_for_workspace(db, workspace),
    )
}

#[salsa::tracked(returns(clone))]
fn memo_style_fact_entry(
    db: &dyn salsa::Database,
    file: OmenaQueryStyleFileInputV0,
) -> OmenaQueryStyleFactEntry {
    #[cfg(any(test, feature = "test-support"))]
    style_fact_entry_probe::record(file.style_path(db));
    let style_path = file.style_path(db);
    let style_source = file.style_source(db);
    let dialect = omena_parser_dialect_for_style_path(style_path);
    let parsed = parse_omena_query_omena_parser_style_source(style_source, dialect);
    let (raw_facts, icss_export_values) =
        collect_omena_query_style_facts_with_icss_values_from_parse(style_source, &parsed);
    collect_omena_query_style_fact_entry_from_raw(
        style_path,
        style_source,
        dialect,
        raw_facts,
        icss_export_values,
    )
    .with_parser_materialization(parsed)
}

#[salsa::tracked(returns(clone))]
fn memo_style_cascade_declarations(
    db: &dyn salsa::Database,
    file: OmenaQueryStyleFileInputV0,
) -> Vec<cascade_checker::QueryCheckerCascadeDeclaration> {
    #[cfg(test)]
    memo_style_cascade_projection_probe::record();
    let fact_entry = memo_style_fact_entry(db, file);
    let Some(parsed) = fact_entry.parser_materialization() else {
        return cascade_checker::collect_query_checker_cascade_declarations(file.style_source(db));
    };
    let (syntax_facts, context_index) =
        omena_semantic::collect_parser_declaration_syntax_and_style_context_from_parse(
            file.style_source(db),
            parsed,
        );
    cascade_checker::collect_query_checker_cascade_declarations_from_syntax_and_context(
        syntax_facts,
        &context_index,
    )
}

#[salsa::tracked(returns(clone))]
pub fn memo_module_interface_projection(
    db: &dyn salsa::Database,
    file: OmenaQueryStyleFileInputV0,
) -> OmenaQueryModuleInterfaceProjectionV0 {
    memo_module_interface_change_projection(db, file).module_interface
}

fn package_manifests_for_workspace(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> Vec<OmenaQueryStylePackageManifestV0> {
    if !workspace.granular_inputs_initialized(db) {
        return workspace.package_manifests(db).clone();
    }
    workspace
        .package_manifest_inputs(db)
        .iter()
        .map(|manifest| memo_package_manifest_projection(db, *manifest))
        .collect()
}

#[salsa::tracked(returns(clone))]
fn memo_package_manifest_projection(
    db: &dyn salsa::Database,
    manifest: OmenaQueryStylePackageManifestInputV0,
) -> OmenaQueryStylePackageManifestV0 {
    #[cfg(test)]
    package_manifest_projection_probe::record(manifest.package_json_path(db));
    OmenaQueryStylePackageManifestV0 {
        package_json_path: manifest.package_json_path(db).clone(),
        package_json_source: manifest.package_json_source(db).clone(),
    }
}

fn resolution_package_manifests_for_workspace(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> Vec<OmenaQueryStylePackageManifestV0> {
    if !workspace.granular_inputs_initialized(db) {
        return workspace.resolution_inputs(db).package_manifests.clone();
    }
    workspace
        .resolution_package_manifest_inputs(db)
        .iter()
        .map(|manifest| memo_package_manifest_projection(db, *manifest))
        .collect()
}

/// Resolution consumers in the memo graph do not read the external-SIF cache
/// freshness token. Reconstructing only the four fields they use prevents a
/// cache-freshness change from invalidating semantic resolution queries.
fn style_resolution_inputs_for_workspace(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> OmenaQueryStyleResolutionInputsV0 {
    if !workspace.granular_inputs_initialized(db) {
        let mut resolution_inputs = workspace.resolution_inputs(db).clone();
        resolution_inputs.external_sif_cache_fingerprint = None;
        return resolution_inputs;
    }
    OmenaQueryStyleResolutionInputsV0 {
        package_manifests: resolution_package_manifests_for_workspace(db, workspace),
        tsconfig_path_mappings: workspace.resolution_tsconfig_path_mappings(db).clone(),
        bundler_path_mappings: workspace.resolution_bundler_path_mappings(db).clone(),
        disk_style_path_identities: workspace.resolution_disk_style_path_identities(db).clone(),
        external_sif_cache_fingerprint: None,
    }
}

fn resolution_tsconfig_path_mappings_for_workspace(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> &[OmenaQueryTsconfigPathMappingV0] {
    if workspace.granular_inputs_initialized(db) {
        workspace.resolution_tsconfig_path_mappings(db).as_slice()
    } else {
        workspace
            .resolution_inputs(db)
            .tsconfig_path_mappings
            .as_slice()
    }
}

fn resolution_bundler_path_mappings_for_workspace(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> &[OmenaQueryBundlerPathAliasMappingV0] {
    if workspace.granular_inputs_initialized(db) {
        workspace.resolution_bundler_path_mappings(db).as_slice()
    } else {
        workspace
            .resolution_inputs(db)
            .bundler_path_mappings
            .as_slice()
    }
}

fn resolution_disk_style_path_identities_for_workspace(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> &[OmenaQueryStyleModuleDiskCandidateIdentityV0] {
    if workspace.granular_inputs_initialized(db) {
        workspace
            .resolution_disk_style_path_identities(db)
            .as_slice()
    } else {
        workspace
            .resolution_inputs(db)
            .disk_style_path_identities
            .as_slice()
    }
}

#[salsa::tracked(returns(clone))]
fn memo_module_interface_change_projection(
    db: &dyn salsa::Database,
    file: OmenaQueryStyleFileInputV0,
) -> OmenaQueryModuleInterfaceChangeProjectionV0 {
    #[cfg(any(test, feature = "test-support"))]
    module_interface_projection_probe::record(file.style_path(db));
    module_interface_change_projection_for_query(&memo_style_fact_entry(db, file))
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
        package_manifests_for_workspace(db, workspace).as_slice(),
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
    let package_manifests = package_manifests_for_workspace(db, workspace);
    let resolution_inputs = style_resolution_inputs_for_workspace(db, workspace);
    let origin = memo_module_interface_projection(db, origin_file);
    let available_style_paths = style_paths_for_workspace(db, workspace);
    let available_style_path_refs = available_style_paths
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let style_import_edges = memo_style_import_reachability_edges(db, workspace);
    let target_interfaces = origin
        .style_dependency_sources
        .iter()
        .filter_map(|source| {
            resolve_style_module_source_with_resolution_inputs_and_identity_index(
                origin.style_path.as_str(),
                source,
                &available_style_path_refs,
                package_manifests.as_slice(),
                &resolution_inputs,
                None,
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
        &resolution_inputs,
        None,
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
    let package_manifests = package_manifests_for_workspace(db, workspace);
    let mut visiting = BTreeSet::new();
    sass_configurable_variable_names_for_module_interface_tracked(
        db,
        workspace,
        style_path.as_str(),
        &available_style_path_refs,
        package_manifests.as_slice(),
        resolution_bundler_path_mappings_for_workspace(db, workspace),
        resolution_tsconfig_path_mappings_for_workspace(db, workspace),
        resolution_disk_style_path_identities_for_workspace(db, workspace),
        None,
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
    let mut visiting = BTreeSet::new();
    sass_configurable_variable_names_for_module_interface_tracked(
        db,
        workspace,
        style_path.as_str(),
        &available_style_path_refs,
        &[],
        resolution_bundler_path_mappings_for_workspace(db, workspace),
        resolution_tsconfig_path_mappings_for_workspace(db, workspace),
        resolution_disk_style_path_identities_for_workspace(db, workspace),
        None,
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
    let package_manifests = package_manifests_for_workspace(db, workspace);
    let mut visiting = BTreeSet::new();
    sass_configurable_variable_names_for_module_interface_tracked(
        db,
        workspace,
        style_path.as_str(),
        &available_style_path_refs,
        package_manifests.as_slice(),
        &[],
        &[],
        &[],
        None,
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
    let package_manifests = package_manifests_for_workspace(db, workspace);
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
        resolution_bundler_path_mappings_for_workspace(db, workspace),
        resolution_tsconfig_path_mappings_for_workspace(db, workspace),
        None,
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
        resolution_bundler_path_mappings_for_workspace(db, workspace),
        resolution_tsconfig_path_mappings_for_workspace(db, workspace),
        None,
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
    let package_manifests = package_manifests_for_workspace(db, workspace);
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
        None,
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
    promote_sif_backed_external_edges(
        &mut resolution,
        OmenaQueryExternalSifResolutionContext {
            package_manifests: package_manifests_for_workspace(db, workspace).as_slice(),
            bundler_path_mappings: resolution_bundler_path_mappings_for_workspace(db, workspace),
            tsconfig_path_mappings: resolution_tsconfig_path_mappings_for_workspace(db, workspace),
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
    #[cfg(test)]
    source_workspace_query_probe::record_cross_file_summary();
    let module_interfaces = module_interfaces_for_workspace(db, workspace);
    let style_cross_file_summary =
        memo_style_cross_file_summary_from_module_interfaces(db, workspace);
    summarize_omena_query_workspace_cross_file_summary_from_module_interfaces(
        module_interfaces.as_slice(),
        source_workspace_projections(db, workspace).as_slice(),
        package_manifests_for_workspace(db, workspace).as_slice(),
        style_cross_file_summary,
        &style_resolution_inputs_for_workspace(db, workspace),
    )
}

#[salsa::tracked(returns(clone))]
fn memo_committed_style_semantic_graph_from_module_interfaces(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> OmenaQueryCommittedStyleSemanticGraphV0 {
    let style_fact_entries = style_fact_entries_for_workspace(db, workspace);
    let cascade_declarations_by_style = workspace
        .files(db)
        .iter()
        .map(|file| {
            (
                file.style_path(db).clone(),
                memo_style_cascade_declarations(db, *file),
            )
        })
        .collect();
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
        cascade_declarations_by_style,
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

/// Workspace-level accessories hoisted into tracked queries (regression fix:
/// the per-origin edge-resolution queries recomputed these — including the
/// FULL workspace import-edge resolution — once per origin, which explodes
/// on corpora with unresolvable specifiers where candidate generation cannot
/// early-return. One execution per revision; `Eq` outputs backdate, so the
/// module-interface firewall semantics are preserved.
#[salsa::tracked(returns(ref))]
fn memo_style_paths_for_workspace(
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

#[salsa::tracked(returns(ref))]
fn memo_resolver_style_paths_for_workspace(
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

#[salsa::tracked(returns(ref))]
fn memo_file_by_style_path(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> BTreeMap<String, OmenaQueryStyleFileInputV0> {
    workspace
        .files(db)
        .iter()
        .map(|file| (file.style_path(db).clone(), *file))
        .collect()
}

#[salsa::tracked(returns(ref))]
fn memo_style_import_reachability_edges(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> Vec<omena_semantic::StyleImportReachabilityEdgeFactV0> {
    let available_style_paths = memo_style_paths_for_workspace(db, workspace)
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    style_import_reachability_edges_from_dependency_surfaces(
        module_dependency_surfaces_for_workspace(db, workspace).as_slice(),
        &available_style_paths,
        package_manifests_for_workspace(db, workspace).as_slice(),
        &style_resolution_inputs_for_workspace(db, workspace),
        None,
    )
}

fn style_paths_for_workspace(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> Vec<String> {
    memo_style_paths_for_workspace(db, workspace).clone()
}

fn resolver_style_paths_for_workspace(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> BTreeSet<String> {
    memo_resolver_style_paths_for_workspace(db, workspace).clone()
}

fn file_for_style_path(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    style_path: &str,
) -> Option<OmenaQueryStyleFileInputV0> {
    memo_file_by_style_path(db, workspace)
        .get(style_path)
        .copied()
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
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    resolver_identity_index: Option<&OmenaResolverStyleModuleConfirmationIdentityIndexV0>,
) -> Vec<omena_semantic::StyleImportReachabilityEdgeFactV0> {
    let mut edges = Vec::new();
    for surface in dependency_surfaces {
        let targets = surface
            .style_dependency_sources
            .iter()
            .filter_map(|source| {
                resolve_style_module_source_with_resolution_inputs_and_identity_index(
                    surface.style_path.as_str(),
                    source,
                    available_style_paths,
                    package_manifests,
                    resolution_inputs,
                    resolver_identity_index,
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
    disk_style_path_identities: &[OmenaResolverStyleModuleDiskCandidateIdentityV0],
    resolver_identity_index: Option<&OmenaResolverStyleModuleConfirmationIdentityIndexV0>,
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
        let Some(resolved) = resolve_style_module_source_with_path_mappings_and_identity_index(
            projection_style_path.as_str(),
            edge.source.as_str(),
            available_style_paths,
            package_manifests,
            bundler_path_mappings,
            tsconfig_path_mappings,
            disk_style_path_identities,
            resolver_identity_index,
        ) else {
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
            disk_style_path_identities,
            resolver_identity_index,
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
/// self-validating: every call compares the in-hand inputs against what the database
/// holds and applies `set_*` only for actual differences — there is no event
/// eviction list to keep in sync, so a stale memo cannot be served. File
/// entities persist per path, so re-adding an unchanged file (or switching
/// workspace folders back and forth) keeps its memos green.
pub struct OmenaQueryStyleMemoHostV0 {
    db: OmenaQueryStyleMemoDatabaseV0,
    files_by_path: BTreeMap<String, OmenaQueryStyleFileInputV0>,
    source_files_by_path: BTreeMap<String, OmenaQuerySourceFileInputV0>,
    package_manifest_inputs_by_identity:
        BTreeMap<(String, usize), OmenaQueryStylePackageManifestInputV0>,
    resolution_package_manifest_inputs_by_identity:
        BTreeMap<(String, usize), OmenaQueryStylePackageManifestInputV0>,
    registered_style_paths: BTreeSet<String>,
    workspace: Option<OmenaQueryStyleWorkspaceInputV0>,
    committed_revision: IncrementalRevisionV0,
    committed_graph: Option<OmenaQueryCommittedStyleSemanticGraphV0>,
    committed_module_interface_changed_paths: BTreeSet<String>,
    resolver_identity_index: Option<OmenaResolverStyleModuleConfirmationIdentityIndexV0>,
    source_corpus_complete: bool,
}

impl std::fmt::Debug for OmenaQueryStyleMemoHostV0 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OmenaQueryStyleMemoHostV0")
            .field("known_file_count", &self.files_by_path.len())
            .field("known_source_file_count", &self.source_files_by_path.len())
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

fn sync_package_manifest_inputs(
    db: &mut OmenaQueryStyleMemoDatabaseV0,
    inputs_by_identity: &mut BTreeMap<(String, usize), OmenaQueryStylePackageManifestInputV0>,
    manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<OmenaQueryStylePackageManifestInputV0> {
    let mut occurrence_by_path = BTreeMap::<String, usize>::new();
    manifests
        .iter()
        .map(|manifest| {
            let occurrence = occurrence_by_path
                .entry(manifest.package_json_path.clone())
                .or_default();
            let identity = (manifest.package_json_path.clone(), *occurrence);
            *occurrence += 1;
            match inputs_by_identity.get(&identity).copied() {
                Some(input) => {
                    if input.package_json_source(db) != &manifest.package_json_source {
                        input
                            .set_package_json_source(db)
                            .to(manifest.package_json_source.clone());
                    }
                    input
                }
                None => {
                    let input = OmenaQueryStylePackageManifestInputV0::new(
                        db,
                        manifest.package_json_path.clone(),
                        manifest.package_json_source.clone(),
                    );
                    inputs_by_identity.insert(identity, input);
                    input
                }
            }
        })
        .collect()
}

impl OmenaQueryStyleMemoHostV0 {
    pub fn new() -> Self {
        Self {
            db: OmenaQueryStyleMemoDatabaseV0::new(),
            files_by_path: BTreeMap::new(),
            source_files_by_path: BTreeMap::new(),
            package_manifest_inputs_by_identity: BTreeMap::new(),
            resolution_package_manifest_inputs_by_identity: BTreeMap::new(),
            registered_style_paths: BTreeSet::new(),
            workspace: None,
            committed_revision: IncrementalRevisionV0 { value: 0 },
            committed_graph: None,
            committed_module_interface_changed_paths: BTreeSet::new(),
            resolver_identity_index: None,
            source_corpus_complete: false,
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

    pub fn source_element_parent_chain(
        &mut self,
        source_documents: &[OmenaQuerySourceDocumentInputV0],
        target: omena_cascade::ElementIdentityV0,
    ) -> omena_cascade::ElementParentChainV0 {
        let workspace = self.sync_source_workspace(source_documents);
        memo_source_element_parent_chain(&self.db, workspace, target)
    }

    pub fn source_scope_proximity(
        &mut self,
        source_documents: &[OmenaQuerySourceDocumentInputV0],
        target: omena_cascade::ElementIdentityV0,
        scope_root_selector: impl Into<String>,
    ) -> omena_cascade::ScopeProximityV0 {
        let workspace = self.sync_source_workspace(source_documents);
        memo_source_scope_proximity(&self.db, workspace, target, scope_root_selector.into())
    }

    pub fn source_element_computed_value(
        &mut self,
        source_documents: &[OmenaQuerySourceDocumentInputV0],
        target: omena_cascade::ElementIdentityV0,
        property: impl Into<String>,
    ) -> OmenaQueryElementComputedValueV0 {
        let workspace = self.sync_source_workspace(source_documents);
        memo_source_element_computed_value(&self.db, workspace, target, property.into())
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
        self.workspace_revision_selector_with_source_corpus_completeness(
            style_sources,
            source_documents,
            package_manifests,
            external_sifs,
            resolution_inputs,
            None,
            false,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn workspace_revision_selector_with_identity_index(
        &mut self,
        style_sources: &[OmenaQueryStyleSourceInputV0],
        source_documents: &[OmenaQuerySourceDocumentInputV0],
        package_manifests: &[OmenaQueryStylePackageManifestV0],
        external_sifs: &[OmenaQueryExternalSifInputV0],
        resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
        resolver_identity_index: &OmenaResolverStyleModuleConfirmationIdentityIndexV0,
    ) -> Option<OmenaQueryStyleRevisionSelectorV0> {
        self.workspace_revision_selector_with_source_corpus_completeness(
            style_sources,
            source_documents,
            package_manifests,
            external_sifs,
            resolution_inputs,
            Some(resolver_identity_index),
            false,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn workspace_revision_selector_with_complete_source_corpus(
        &mut self,
        style_sources: &[OmenaQueryStyleSourceInputV0],
        source_documents: &[OmenaQuerySourceDocumentInputV0],
        package_manifests: &[OmenaQueryStylePackageManifestV0],
        external_sifs: &[OmenaQueryExternalSifInputV0],
        resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    ) -> Option<OmenaQueryStyleRevisionSelectorV0> {
        self.workspace_revision_selector_with_source_corpus_completeness(
            style_sources,
            source_documents,
            package_manifests,
            external_sifs,
            resolution_inputs,
            None,
            true,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn workspace_revision_selector_with_source_corpus_completeness(
        &mut self,
        style_sources: &[OmenaQueryStyleSourceInputV0],
        source_documents: &[OmenaQuerySourceDocumentInputV0],
        package_manifests: &[OmenaQueryStylePackageManifestV0],
        external_sifs: &[OmenaQueryExternalSifInputV0],
        resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
        resolver_identity_index: Option<&OmenaResolverStyleModuleConfirmationIdentityIndexV0>,
        source_corpus_complete: bool,
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
        if let Some(resolver_identity_index) = resolver_identity_index {
            transaction.set_resolver_identity_index(resolver_identity_index);
        }
        if source_corpus_complete {
            transaction.mark_source_corpus_complete();
        }
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
            snapshot_id: selector.snapshot_id(),
            selector,
        })
    }

    /// Run the same diff-only sync as [`Self::workspace_style_diagnostics`]
    /// on the loop thread, before any handle
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
        let selector = build_revision_selector(OmenaQueryStyleRevisionSelectorBuildInputV0 {
            revision: commit.revision,
            style_sources: commit.style_sources.as_slice(),
            source_documents: commit.source_documents.as_slice(),
            package_manifests: commit.package_manifests.as_slice(),
            external_sifs: commit.external_sifs.as_slice(),
            resolution_inputs: &commit.resolution_inputs,
            resolver_identity_index: &commit.resolver_identity_index,
            source_corpus_complete: commit.source_corpus_complete,
            committed_graph: commit.committed_graph,
            changed_module_interface_paths: commit.changed_module_interface_paths.clone(),
        });
        Ok(OmenaQueryStyleWorkspaceTransactionCommitV0 {
            revision: commit.revision,
            workspace: commit.workspace,
            files: commit.files,
            changed_style_paths: commit.changed_style_paths,
            changed_module_interface_paths: commit.changed_module_interface_paths,
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
        let workspace_inputs_changed = self.workspace_inputs_changed_for_transaction(&transaction);
        if changed_style_paths.is_empty()
            && !workspace_inputs_changed
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
                changed_module_interface_paths: self
                    .committed_module_interface_changed_paths
                    .clone(),
                style_sources: transaction.style_sources,
                source_documents: transaction.source_documents,
                package_manifests: transaction.package_manifests,
                external_sifs: transaction.external_sifs,
                resolution_inputs: transaction.resolution_inputs,
                resolver_identity_index: transaction.resolver_identity_index,
                source_corpus_complete: transaction.source_corpus_complete,
                committed_graph,
            });
        }
        let previous_module_interfaces = self
            .workspace
            .map(|workspace| {
                self.module_interface_projections_for_workspace_paths(
                    workspace,
                    &changed_style_paths,
                )
            })
            .unwrap_or_default();
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
        let current_module_interfaces =
            self.module_interface_projections_for_workspace_paths(workspace, &changed_style_paths);
        let changed_module_interface_paths = changed_style_paths
            .iter()
            .filter(|style_path| {
                previous_module_interfaces.get(style_path.as_str())
                    != current_module_interfaces.get(style_path.as_str())
            })
            .cloned()
            .collect::<BTreeSet<_>>();
        self.committed_revision = IncrementalRevisionV0 {
            value: self.committed_revision.value + 1,
        };
        let committed_graph = match transaction.resolver_identity_index.as_ref() {
            Some(resolver_identity_index) => {
                build_committed_style_semantic_graph_with_identity_index(
                    &self.db,
                    workspace,
                    transaction.source_documents.as_slice(),
                    transaction.package_manifests.as_slice(),
                    transaction.external_sifs.as_slice(),
                    &transaction.resolution_inputs,
                    resolver_identity_index,
                )
            }
            None => build_committed_style_semantic_graph(
                &self.db,
                workspace,
                transaction.source_documents.as_slice(),
                transaction.package_manifests.as_slice(),
                transaction.external_sifs.as_slice(),
                &transaction.resolution_inputs,
            ),
        };
        self.committed_graph = Some(committed_graph.clone());
        self.committed_module_interface_changed_paths = changed_module_interface_paths.clone();
        self.resolver_identity_index = transaction.resolver_identity_index.clone();
        self.source_corpus_complete = transaction.source_corpus_complete;
        Ok(OmenaQueryStyleWorkspaceTransactionCoreCommitV0 {
            revision: self.committed_revision,
            workspace,
            files,
            changed_style_paths,
            changed_module_interface_paths,
            style_sources: transaction.style_sources,
            source_documents: transaction.source_documents,
            package_manifests: transaction.package_manifests,
            external_sifs: transaction.external_sifs,
            resolution_inputs: transaction.resolution_inputs,
            resolver_identity_index: transaction.resolver_identity_index,
            source_corpus_complete: transaction.source_corpus_complete,
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

        changed
    }

    fn workspace_inputs_changed_for_transaction(
        &self,
        transaction: &OmenaQueryStyleWorkspaceTransactionV0,
    ) -> bool {
        let Some(workspace) = self.workspace else {
            return true;
        };
        workspace.source_documents(&self.db).as_slice() != transaction.source_documents.as_slice()
            || workspace.package_manifests(&self.db).as_slice()
                != transaction.package_manifests.as_slice()
            || workspace.external_sifs(&self.db).as_slice() != transaction.external_sifs.as_slice()
            || workspace.resolution_inputs(&self.db) != &transaction.resolution_inputs
            || self.resolver_identity_index != transaction.resolver_identity_index
            || self.source_corpus_complete != transaction.source_corpus_complete
    }

    fn module_interface_projections_for_workspace_paths(
        &self,
        workspace: OmenaQueryStyleWorkspaceInputV0,
        style_paths: &BTreeSet<String>,
    ) -> BTreeMap<String, OmenaQueryModuleInterfaceChangeProjectionV0> {
        workspace
            .files(&self.db)
            .iter()
            .filter_map(|file| {
                let style_path = file.style_path(&self.db);
                style_paths.contains(style_path).then(|| {
                    (
                        style_path.clone(),
                        memo_module_interface_change_projection(&self.db, *file),
                    )
                })
            })
            .collect()
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
        let source_files = self.sync_source_file_inputs(source_documents);
        let package_manifest_inputs = sync_package_manifest_inputs(
            &mut self.db,
            &mut self.package_manifest_inputs_by_identity,
            package_manifests,
        );
        let resolution_package_manifest_inputs = sync_package_manifest_inputs(
            &mut self.db,
            &mut self.resolution_package_manifest_inputs_by_identity,
            resolution_inputs.package_manifests.as_slice(),
        );

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
                if workspace.source_files(&self.db) != &source_files {
                    workspace.set_source_files(&mut self.db).to(source_files);
                }
                if workspace.package_manifests(&self.db).as_slice() != package_manifests {
                    workspace
                        .set_package_manifests(&mut self.db)
                        .to(package_manifests.to_vec());
                }
                if workspace.package_manifest_inputs(&self.db) != &package_manifest_inputs {
                    workspace
                        .set_package_manifest_inputs(&mut self.db)
                        .to(package_manifest_inputs);
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
                if workspace.resolution_package_manifest_inputs(&self.db)
                    != &resolution_package_manifest_inputs
                {
                    workspace
                        .set_resolution_package_manifest_inputs(&mut self.db)
                        .to(resolution_package_manifest_inputs);
                }
                if workspace.resolution_tsconfig_path_mappings(&self.db)
                    != &resolution_inputs.tsconfig_path_mappings
                {
                    workspace
                        .set_resolution_tsconfig_path_mappings(&mut self.db)
                        .to(resolution_inputs.tsconfig_path_mappings.clone());
                }
                if workspace.resolution_bundler_path_mappings(&self.db)
                    != &resolution_inputs.bundler_path_mappings
                {
                    workspace
                        .set_resolution_bundler_path_mappings(&mut self.db)
                        .to(resolution_inputs.bundler_path_mappings.clone());
                }
                if workspace.resolution_disk_style_path_identities(&self.db)
                    != &resolution_inputs.disk_style_path_identities
                {
                    workspace
                        .set_resolution_disk_style_path_identities(&mut self.db)
                        .to(resolution_inputs.disk_style_path_identities.clone());
                }
                if workspace.resolution_external_sif_cache_fingerprint(&self.db)
                    != &resolution_inputs.external_sif_cache_fingerprint
                {
                    workspace
                        .set_resolution_external_sif_cache_fingerprint(&mut self.db)
                        .to(resolution_inputs.external_sif_cache_fingerprint.clone());
                }
                if !workspace.granular_inputs_initialized(&self.db) {
                    workspace
                        .set_granular_inputs_initialized(&mut self.db)
                        .to(true);
                }
                workspace
            }
            None => {
                let workspace = OmenaQueryStyleWorkspaceInputV0::new(
                    &self.db,
                    files,
                    source_documents.to_vec(),
                    source_files,
                    package_manifests.to_vec(),
                    external_sifs.to_vec(),
                    resolution_inputs.clone(),
                );
                workspace
                    .set_package_manifest_inputs(&mut self.db)
                    .to(package_manifest_inputs);
                workspace
                    .set_resolution_package_manifest_inputs(&mut self.db)
                    .to(resolution_package_manifest_inputs);
                workspace
                    .set_resolution_tsconfig_path_mappings(&mut self.db)
                    .to(resolution_inputs.tsconfig_path_mappings.clone());
                workspace
                    .set_resolution_bundler_path_mappings(&mut self.db)
                    .to(resolution_inputs.bundler_path_mappings.clone());
                workspace
                    .set_resolution_disk_style_path_identities(&mut self.db)
                    .to(resolution_inputs.disk_style_path_identities.clone());
                workspace
                    .set_resolution_external_sif_cache_fingerprint(&mut self.db)
                    .to(resolution_inputs.external_sif_cache_fingerprint.clone());
                workspace
                    .set_granular_inputs_initialized(&mut self.db)
                    .to(true);
                self.workspace = Some(workspace);
                workspace
            }
        }
    }

    fn sync_source_file_inputs(
        &mut self,
        source_documents: &[OmenaQuerySourceDocumentInputV0],
    ) -> Vec<OmenaQuerySourceFileInputV0> {
        source_documents
            .iter()
            .map(
                |document| match self.source_files_by_path.get(document.source_path.as_str()) {
                    Some(file) => {
                        if file.source_source(&self.db) != &document.source_source {
                            file.set_source_source(&mut self.db)
                                .to(document.source_source.clone());
                        }
                        if file.source_syntax_index(&self.db) != &document.source_syntax_index {
                            file.set_source_syntax_index(&mut self.db)
                                .to(document.source_syntax_index.clone());
                        }
                        if *file.has_unresolved_style_import(&self.db)
                            != document.has_unresolved_style_import
                        {
                            file.set_has_unresolved_style_import(&mut self.db)
                                .to(document.has_unresolved_style_import);
                        }
                        *file
                    }
                    None => {
                        let file = OmenaQuerySourceFileInputV0::new(
                            &self.db,
                            document.source_path.clone(),
                            document.source_source.clone(),
                            document.source_syntax_index.clone(),
                            document.has_unresolved_style_import,
                        );
                        self.source_files_by_path
                            .insert(document.source_path.clone(), file);
                        file
                    }
                },
            )
            .collect()
    }

    fn sync_source_workspace(
        &mut self,
        source_documents: &[OmenaQuerySourceDocumentInputV0],
    ) -> OmenaQueryStyleWorkspaceInputV0 {
        let source_files = self.sync_source_file_inputs(source_documents);
        match self.workspace {
            Some(workspace) => {
                if workspace.source_documents(&self.db).as_slice() != source_documents {
                    workspace
                        .set_source_documents(&mut self.db)
                        .to(source_documents.to_vec());
                }
                if workspace.source_files(&self.db) != &source_files {
                    workspace.set_source_files(&mut self.db).to(source_files);
                }
                workspace
            }
            None => {
                let workspace = OmenaQueryStyleWorkspaceInputV0::new(
                    &self.db,
                    Vec::new(),
                    source_documents.to_vec(),
                    source_files,
                    Vec::new(),
                    Vec::new(),
                    OmenaQueryStyleResolutionInputsV0::default(),
                );
                self.workspace = Some(workspace);
                workspace
            }
        }
    }
}

fn build_revision_selector(
    input: OmenaQueryStyleRevisionSelectorBuildInputV0<'_>,
) -> OmenaQueryStyleRevisionSelectorV0 {
    let mut host = OmenaQueryStyleMemoHostV0::new();
    let workspace = host.sync_workspace(
        input.style_sources,
        input.source_documents,
        input.package_manifests,
        input.external_sifs,
        input.resolution_inputs,
    );
    let OmenaQueryStyleMemoHostV0 {
        db,
        files_by_path,
        source_files_by_path: _,
        package_manifest_inputs_by_identity: _,
        resolution_package_manifest_inputs_by_identity: _,
        registered_style_paths: _,
        workspace: _,
        committed_revision: _,
        committed_graph: _,
        committed_module_interface_changed_paths: _,
        resolver_identity_index: _,
        source_corpus_complete: _,
    } = host;
    let files = input
        .style_sources
        .iter()
        .filter_map(|source| {
            files_by_path
                .get(source.style_path.as_str())
                .map(|file| (source.style_path.clone(), *file))
        })
        .collect();
    OmenaQueryStyleRevisionSelectorV0 {
        revision: input.revision,
        db,
        workspace,
        files,
        files_by_path,
        changed_module_interface_paths: input.changed_module_interface_paths,
        committed_graph: input.committed_graph,
        resolver_identity_index: input.resolver_identity_index.clone(),
        source_corpus_complete: input.source_corpus_complete,
        unused_selector_shared: std::sync::OnceLock::new(),
    }
}

#[salsa::tracked(returns(ref))]
fn memo_source_file_by_path(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
) -> BTreeMap<String, OmenaQuerySourceFileInputV0> {
    workspace
        .source_files(db)
        .iter()
        .map(|file| (file.source_path(db).clone(), *file))
        .collect()
}

#[salsa::tracked(returns(clone))]
pub fn memo_source_element_parent_chain(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    target: omena_cascade::ElementIdentityV0,
) -> omena_cascade::ElementParentChainV0 {
    use omena_cascade::{ElementIdentityV0, ElementParentChainStatusV0, ElementParentChainV0};

    let files_by_path = memo_source_file_by_path(db, workspace);
    let mut current = target.clone();
    let mut ancestors = Vec::new();
    let mut visited = BTreeSet::from([target.clone()]);

    loop {
        let Some(file) = files_by_path.get(current.source_path.as_str()).copied() else {
            return ElementParentChainV0 {
                target,
                ancestors,
                status: ElementParentChainStatusV0::MissingSource,
            };
        };
        #[cfg(test)]
        source_element_parent_chain_probe::record(file.source_path(db));
        let owned_index;
        let index = if let Some(index) = file.source_syntax_index(db).as_ref() {
            index
        } else {
            owned_index = summarize_omena_query_source_syntax_index_for_source_language(
                file.source_path(db),
                file.source_source(db),
                None,
                Vec::new(),
            );
            &owned_index
        };
        let identity = OmenaQuerySourceElementIdentityFactV0 {
            source_path: current.source_path.clone(),
            byte_span: ParserByteSpanV0 {
                start: current.byte_start,
                end: current.byte_end,
            },
        };
        if !index
            .source_elements
            .iter()
            .any(|element| element.identity == identity)
        {
            return ElementParentChainV0 {
                target,
                ancestors,
                status: ElementParentChainStatusV0::MissingElement,
            };
        }
        let parents = index
            .element_parent_edges
            .iter()
            .filter(|edge| edge.child == identity)
            .map(|edge| ElementIdentityV0 {
                source_path: edge.parent.source_path.clone(),
                byte_start: edge.parent.byte_span.start,
                byte_end: edge.parent.byte_span.end,
            })
            .collect::<BTreeSet<_>>();
        let mut parents = parents.into_iter();
        let Some(parent) = parents.next() else {
            return ElementParentChainV0 {
                target,
                ancestors,
                status: ElementParentChainStatusV0::Complete,
            };
        };
        if parents.next().is_some() {
            return ElementParentChainV0 {
                target,
                ancestors,
                status: ElementParentChainStatusV0::AmbiguousParent,
            };
        }
        if !visited.insert(parent.clone()) {
            return ElementParentChainV0 {
                target,
                ancestors,
                status: ElementParentChainStatusV0::Cycle,
            };
        }
        ancestors.push(parent.clone());
        current = parent;
    }
}

#[salsa::tracked(returns(clone))]
pub fn memo_source_scope_proximity(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    target: omena_cascade::ElementIdentityV0,
    scope_root_selector: String,
) -> omena_cascade::ScopeProximityV0 {
    use omena_cascade::{
        ElementIdentityV0, ElementSignature, ScopeProximityStatusV0,
        scope_proximity_from_ancestor_signatures,
    };

    let parent_chain = memo_source_element_parent_chain(db, workspace, target.clone());
    let files_by_path = memo_source_file_by_path(db, workspace);
    let mut identities = Vec::with_capacity(parent_chain.ancestors.len().saturating_add(1));
    identities.push(target);
    identities.extend(parent_chain.ancestors.iter().cloned());
    let mut elements = Vec::<(ElementIdentityV0, ElementSignature)>::new();

    for identity in identities {
        let Some(file) = files_by_path.get(identity.source_path.as_str()).copied() else {
            return omena_cascade::ScopeProximityV0::unknown(
                ScopeProximityStatusV0::MissingElementSignature,
            );
        };
        let owned_index;
        let index = if let Some(index) = file.source_syntax_index(db).as_ref() {
            index
        } else {
            owned_index = summarize_omena_query_source_syntax_index_for_source_language(
                file.source_path(db),
                file.source_source(db),
                None,
                Vec::new(),
            );
            &owned_index
        };
        let Some(element) = index.source_elements.iter().find(|element| {
            element.identity.source_path == identity.source_path
                && element.identity.byte_span.start == identity.byte_start
                && element.identity.byte_span.end == identity.byte_end
        }) else {
            return omena_cascade::ScopeProximityV0::unknown(
                ScopeProximityStatusV0::MissingElementSignature,
            );
        };
        let mut signature = ElementSignature::concrete(
            element.intrinsic_tag_name.clone(),
            None::<String>,
            element.static_class_names.clone(),
        );
        signature.classes_are_exact = element.classes_are_exact;
        signature.attributes_are_exact = false;
        signature.pseudo_states_are_exact = false;
        signature.tag_is_exact = element.intrinsic_tag_name.is_some();
        signature.id_is_exact = false;
        elements.push((identity, signature));
    }

    scope_proximity_from_ancestor_signatures(
        scope_root_selector.as_str(),
        elements.as_slice(),
        parent_chain.is_complete(),
    )
}

#[salsa::tracked(returns(clone))]
pub fn memo_source_element_computed_value(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    target: omena_cascade::ElementIdentityV0,
    property: String,
) -> OmenaQueryElementComputedValueV0 {
    use omena_cascade::{
        CascadeComputedValueInputV0, CascadeStandardValueVerdictV0, CascadeValue,
        CustomPropertyEnv, compute_cascade_computed_value_with_standard_value_validator_v0,
    };
    use omena_query_checker_orchestrator::standard_property_value_verdict_v0;
    use omena_query_core::SpecStandardPropertyValueValidatorV0;

    #[cfg(test)]
    source_element_computed_value_probe::record();

    let parent_chain = memo_source_element_parent_chain(db, workspace, target.clone());
    if !parent_chain.is_complete() {
        return element_computed_value_report(
            target,
            property,
            parent_chain,
            OmenaQueryElementComputedValueStatusV0::IncompleteParentChain,
            None,
        );
    }

    let mut identities = parent_chain
        .ancestors
        .iter()
        .rev()
        .cloned()
        .collect::<Vec<_>>();
    identities.push(target.clone());
    let mut parent_computed_value = None::<CascadeValue>;
    let mut target_result = None;
    let mut custom_property_env = CustomPropertyEnv::new();

    for identity in identities {
        match source_element_static_custom_property_env(db, workspace, &identity) {
            Ok(local_custom_property_env) => custom_property_env.extend(local_custom_property_env),
            Err(status) => {
                return element_computed_value_report(target, property, parent_chain, status, None);
            }
        }
        custom_property_env =
            omena_cascade::resolve_custom_property_env_least_fixed_point(&custom_property_env);
        let projection = memo_source_element_static_declarations(
            db,
            workspace,
            identity.clone(),
            property.clone(),
        );
        if projection.status != OmenaQueryElementComputedValueStatusV0::Resolved {
            return element_computed_value_report(
                target,
                property,
                parent_chain,
                projection.status,
                None,
            );
        }
        let standard_property_value_verdicts = projection
            .declarations
            .iter()
            .filter(|declaration| declaration.property_key.as_custom().is_none())
            .map(|declaration| {
                let verdict = match &declaration.value {
                    CascadeValue::Literal(value) => {
                        standard_property_value_verdict_v0(declaration.property_key.as_str(), value)
                    }
                    _ => CascadeStandardValueVerdictV0::Unknown,
                };
                (declaration.id.clone(), verdict)
            })
            .collect();
        let result = compute_cascade_computed_value_with_standard_value_validator_v0(
            CascadeComputedValueInputV0 {
                property: property.clone(),
                declarations: projection.declarations,
                custom_property_env: custom_property_env.clone(),
                parent_computed_value,
                registered_custom_property: None,
                standard_property_value_verdicts,
            },
            &SpecStandardPropertyValueValidatorV0,
        );
        parent_computed_value = Some(result.value.clone());
        target_result = Some(result);
    }

    element_computed_value_report(
        target,
        property,
        parent_chain,
        OmenaQueryElementComputedValueStatusV0::Resolved,
        target_result,
    )
}

fn source_element_static_custom_property_env(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    identity: &omena_cascade::ElementIdentityV0,
) -> Result<omena_cascade::CustomPropertyEnv, OmenaQueryElementComputedValueStatusV0> {
    use omena_cascade::{CascadeValue, CustomPropertyEnv};
    use omena_query_transform_runner::parse_static_css_cascade_value;

    let files_by_path = memo_source_file_by_path(db, workspace);
    let Some(file) = files_by_path.get(identity.source_path.as_str()).copied() else {
        return Err(OmenaQueryElementComputedValueStatusV0::MissingElement);
    };
    let owned_index;
    let index = if let Some(index) = file.source_syntax_index(db).as_ref() {
        index
    } else {
        owned_index = summarize_omena_query_source_syntax_index_for_source_language(
            file.source_path(db),
            file.source_source(db),
            None,
            Vec::new(),
        );
        &owned_index
    };
    if !index.source_elements.iter().any(|element| {
        element.identity.source_path == identity.source_path
            && element.identity.byte_span.start == identity.byte_start
            && element.identity.byte_span.end == identity.byte_end
    }) {
        return Err(OmenaQueryElementComputedValueStatusV0::MissingElement);
    }

    let mut env = CustomPropertyEnv::new();
    for declaration in index
        .inline_style_declarations
        .iter()
        .filter(|declaration| {
            declaration.property_name.starts_with("--")
                && declaration.byte_span.start >= identity.byte_start
                && declaration.byte_span.end <= identity.byte_end
        })
    {
        let value = declaration
            .static_value
            .then_some(declaration.value.as_deref())
            .flatten()
            .and_then(source_inline_css_value)
            .filter(|_| !declaration.important_suffix_present())
            .and_then(|value| parse_static_css_cascade_value(value.as_str()))
            .unwrap_or(CascadeValue::Indeterminate);
        env.insert(
            PropertyNameV0::canonical_custom_key(&declaration.property_name),
            value,
        );
    }
    Ok(env)
}

#[salsa::tracked(returns(clone))]
fn memo_source_element_static_declarations(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    identity: omena_cascade::ElementIdentityV0,
    property: String,
) -> SourceElementDeclarationProjectionV0 {
    use omena_cascade::{
        CascadeDeclaration, CascadeKey, CascadeOriginV0, OpenWorldTieEvidence, Specificity,
        cascade_level_for_origin, normalized_layer_rank,
    };
    use omena_query_transform_runner::parse_static_css_cascade_value;

    let files_by_path = memo_source_file_by_path(db, workspace);
    let Some(file) = files_by_path.get(identity.source_path.as_str()).copied() else {
        return source_element_declaration_projection(
            OmenaQueryElementComputedValueStatusV0::MissingElement,
        );
    };
    #[cfg(test)]
    source_element_parent_chain_probe::record(file.source_path(db));
    let owned_index;
    let index = if let Some(index) = file.source_syntax_index(db).as_ref() {
        index
    } else {
        owned_index = summarize_omena_query_source_syntax_index_for_source_language(
            file.source_path(db),
            file.source_source(db),
            None,
            Vec::new(),
        );
        &owned_index
    };
    let element_exists = index.source_elements.iter().any(|element| {
        element.identity.source_path == identity.source_path
            && element.identity.byte_span.start == identity.byte_start
            && element.identity.byte_span.end == identity.byte_end
    });
    if !element_exists {
        return source_element_declaration_projection(
            OmenaQueryElementComputedValueStatusV0::MissingElement,
        );
    }

    let mut declarations = Vec::new();
    for declaration in index
        .inline_style_declarations
        .iter()
        .filter(|declaration| {
            property_names_same(&declaration.property_name, &property)
                && declaration.byte_span.start >= identity.byte_start
                && declaration.byte_span.end <= identity.byte_end
        })
    {
        if !declaration.static_value {
            return source_element_declaration_projection(
                OmenaQueryElementComputedValueStatusV0::DynamicDeclaration,
            );
        }
        let Some(value_source) = declaration.value.as_deref() else {
            return source_element_declaration_projection(
                OmenaQueryElementComputedValueStatusV0::DynamicDeclaration,
            );
        };
        let Some(css_value_source) = source_inline_css_value(value_source) else {
            return source_element_declaration_projection(
                OmenaQueryElementComputedValueStatusV0::UnsupportedStaticValue,
            );
        };
        if declaration.important_suffix_present() {
            return source_element_declaration_projection(
                OmenaQueryElementComputedValueStatusV0::UnsupportedStaticValue,
            );
        }
        let Some(value) = parse_static_css_cascade_value(css_value_source.as_str()) else {
            return source_element_declaration_projection(
                OmenaQueryElementComputedValueStatusV0::UnsupportedStaticValue,
            );
        };
        declarations.push(CascadeDeclaration {
            id: format!(
                "{}:{}:{}",
                identity.source_path, property, declaration.byte_span.start
            ),
            property: AuthoredPropertyTextV0::new(property.clone()),
            property_key: PropertyNameV0::from_authored(&property).canonical_key(),
            value,
            key: CascadeKey::new(
                cascade_level_for_origin(CascadeOriginV0::Inline, false),
                normalized_layer_rank(false, None),
                0,
                Specificity::ZERO,
                declaration.byte_span.start.min(u32::MAX as usize) as u32,
            ),
            open_world_tie_evidence: OpenWorldTieEvidence::NONE,
            specificity_exactness: omena_cascade::SpecificityExactnessV0::Exact,
        });
    }

    SourceElementDeclarationProjectionV0 {
        declarations,
        status: OmenaQueryElementComputedValueStatusV0::Resolved,
    }
}

fn source_element_declaration_projection(
    status: OmenaQueryElementComputedValueStatusV0,
) -> SourceElementDeclarationProjectionV0 {
    SourceElementDeclarationProjectionV0 {
        declarations: Vec::new(),
        status,
    }
}

fn source_inline_css_value(value_source: &str) -> Option<String> {
    let value_source = value_source.trim();
    let quoted = [('\'', '\''), ('"', '"'), ('`', '`')]
        .into_iter()
        .find(|(open, close)| value_source.starts_with(*open) && value_source.ends_with(*close));
    let Some((_, _)) = quoted else {
        return Some(value_source.to_string());
    };
    let inner = value_source.get(1..value_source.len().checked_sub(1)?)?;
    (!inner.contains(['\\', '$'])).then(|| inner.to_string())
}

fn element_computed_value_report(
    target: omena_cascade::ElementIdentityV0,
    property: String,
    parent_chain: omena_cascade::ElementParentChainV0,
    status: OmenaQueryElementComputedValueStatusV0,
    computed_value: Option<omena_cascade::CascadeComputedValueResultV0>,
) -> OmenaQueryElementComputedValueV0 {
    OmenaQueryElementComputedValueV0 {
        schema_version: "0",
        product: "omena-query.element-computed-value",
        target,
        property,
        status,
        parent_chain,
        computed_value,
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

#[allow(clippy::too_many_arguments)]
fn build_committed_style_semantic_graph_with_identity_index(
    db: &OmenaQueryStyleMemoDatabaseV0,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    _source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    external_sifs: &[OmenaQueryExternalSifInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    resolver_identity_index: &OmenaResolverStyleModuleConfirmationIdentityIndexV0,
) -> OmenaQueryCommittedStyleSemanticGraphV0 {
    #[cfg(any(test, feature = "test-support"))]
    record_committed_style_semantic_graph_compute_for_test();

    let style_fact_entries = style_fact_entries_for_workspace(db, workspace);
    let module_interfaces = module_interfaces_for_workspace(db, workspace);
    let cascade_declarations_by_style = workspace
        .files(db)
        .iter()
        .map(|file| {
            (
                file.style_path(db).clone(),
                memo_style_cascade_declarations(db, *file),
            )
        })
        .collect();
    let css_modules_resolution = css_modules_resolution_with_identity_index_from_module_interfaces(
        db,
        workspace,
        module_interfaces.as_slice(),
        package_manifests,
        resolution_inputs,
        resolver_identity_index,
    );
    let sass_module_resolution = sass_module_resolution_with_identity_index_from_module_interfaces(
        db,
        workspace,
        module_interfaces.as_slice(),
        package_manifests,
        resolution_inputs.bundler_path_mappings.as_slice(),
        resolution_inputs.tsconfig_path_mappings.as_slice(),
        resolution_inputs.disk_style_path_identities.as_slice(),
        resolver_identity_index,
    );
    let sass_module_resolution_without_manifests =
        sass_module_resolution_with_identity_index_from_module_interfaces(
            db,
            workspace,
            module_interfaces.as_slice(),
            &[],
            resolution_inputs.bundler_path_mappings.as_slice(),
            resolution_inputs.tsconfig_path_mappings.as_slice(),
            resolution_inputs.disk_style_path_identities.as_slice(),
            resolver_identity_index,
        );
    let sass_module_resolution_without_path_mappings =
        sass_module_resolution_with_identity_index_from_module_interfaces(
            db,
            workspace,
            module_interfaces.as_slice(),
            package_manifests,
            &[],
            &[],
            &[],
            resolver_identity_index,
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
    let style_cross_file_summary = summarize_omena_query_cross_file_summary_from_module_interfaces(
        module_interfaces.as_slice(),
        &css_modules_resolution,
        &sass_module_resolution,
    );
    let cross_file_summary =
        summarize_omena_query_workspace_cross_file_summary_from_module_interfaces(
            module_interfaces.as_slice(),
            source_workspace_projections(db, workspace).as_slice(),
            package_manifests,
            style_cross_file_summary.clone(),
            resolution_inputs,
        );
    OmenaQueryCommittedStyleSemanticGraphV0 {
        style_fact_entries,
        cascade_declarations_by_style,
        style_cross_file_summary,
        cross_file_summary,
        css_modules_resolution,
        sass_module_resolution,
        sass_module_resolution_without_manifests,
        sass_module_resolution_without_path_mappings,
        sass_module_resolution_with_external_sifs,
    }
}

fn css_modules_resolution_with_identity_index_from_module_interfaces(
    db: &OmenaQueryStyleMemoDatabaseV0,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    module_interfaces: &[OmenaQueryModuleInterfaceProjectionV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    resolver_identity_index: &OmenaResolverStyleModuleConfirmationIdentityIndexV0,
) -> OmenaQueryCssModulesCrossFileResolutionV0 {
    #[cfg(any(test, feature = "test-support"))]
    record_css_modules_cross_file_resolution_compute_for_test();
    let available_style_paths = style_paths_for_workspace(db, workspace);
    let available_style_path_refs = available_style_paths
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let style_import_edges = style_import_reachability_edges_from_dependency_surfaces(
        module_dependency_surfaces_for_workspace(db, workspace).as_slice(),
        &available_style_path_refs,
        package_manifests,
        resolution_inputs,
        Some(resolver_identity_index),
    );
    let interfaces_by_path = module_interfaces
        .iter()
        .map(|projection| (projection.style_path.as_str(), projection))
        .collect::<BTreeMap<_, _>>();
    let edges = module_interfaces
        .iter()
        .flat_map(|origin| {
            let target_interfaces = origin
                .style_dependency_sources
                .iter()
                .filter_map(|source| {
                    resolve_style_module_source_with_resolution_inputs_and_identity_index(
                        origin.style_path.as_str(),
                        source,
                        &available_style_path_refs,
                        package_manifests,
                        resolution_inputs,
                        Some(resolver_identity_index),
                    )
                })
                .collect::<BTreeSet<_>>()
                .into_iter()
                .filter_map(|style_path| interfaces_by_path.get(style_path.as_str()).copied())
                .cloned()
                .collect::<Vec<_>>();
            summarize_css_modules_import_edge_resolutions_for_module_interface(
                origin,
                target_interfaces.as_slice(),
                &available_style_path_refs,
                style_import_edges.as_slice(),
                package_manifests,
                resolution_inputs,
                Some(resolver_identity_index),
            )
        })
        .collect::<Vec<_>>();
    summarize_css_modules_cross_file_resolution_from_module_interfaces_and_pre_resolved_import_edges(
        module_interfaces,
        package_manifests,
        edges,
    )
}

#[allow(clippy::too_many_arguments)]
fn sass_module_resolution_with_identity_index_from_module_interfaces(
    db: &OmenaQueryStyleMemoDatabaseV0,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    module_interfaces: &[OmenaQueryModuleInterfaceProjectionV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
    disk_style_path_identities: &[OmenaResolverStyleModuleDiskCandidateIdentityV0],
    resolver_identity_index: &OmenaResolverStyleModuleConfirmationIdentityIndexV0,
) -> OmenaQuerySassModuleCrossFileResolutionV0 {
    #[cfg(any(test, feature = "test-support"))]
    record_sass_module_resolution_internal_compute_for_test();
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
    let configurable_names_by_path = available_style_paths
        .iter()
        .map(|style_path| {
            let mut visiting = BTreeSet::new();
            let names = sass_configurable_variable_names_for_module_interface_tracked(
                db,
                workspace,
                style_path.as_str(),
                &available_style_path_refs,
                package_manifests,
                bundler_path_mappings,
                tsconfig_path_mappings,
                disk_style_path_identities,
                Some(resolver_identity_index),
                &mut visiting,
            );
            (style_path.clone(), names)
        })
        .collect::<BTreeMap<_, _>>();
    let edges = module_interfaces
        .iter()
        .flat_map(|origin| {
            summarize_sass_module_edge_resolutions_for_module_interface(
                origin,
                &available_style_path_refs,
                &resolver_available_style_path_refs,
                package_manifests,
                bundler_path_mappings,
                tsconfig_path_mappings,
                Some(resolver_identity_index),
                |target_style_path| {
                    configurable_names_by_path
                        .get(target_style_path)
                        .cloned()
                        .unwrap_or_default()
                },
            )
        })
        .collect::<Vec<_>>();
    summarize_sass_module_cross_file_resolution_from_module_interfaces_and_edges(
        module_interfaces,
        edges,
        &configurable_names_by_path,
    )
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
        resolution_inputs,
    );
    let cascade_declarations_by_style = style_fact_entries
        .iter()
        .map(|entry| {
            (
                entry.style_path.clone(),
                cascade_checker::collect_query_checker_cascade_declarations(
                    entry.style_source.as_str(),
                ),
            )
        })
        .collect();
    OmenaQueryCommittedStyleSemanticGraphV0 {
        style_fact_entries,
        cascade_declarations_by_style,
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

    fn summarize_omena_query_source_syntax_index_for_source_language(
        source_path: &str,
        source: &str,
        source_language: Option<&str>,
        imported_style_bindings: Vec<OmenaQuerySourceImportedStyleBindingV0>,
        _classnames_bind_bindings: Vec<String>,
    ) -> OmenaQuerySourceSyntaxIndexV0 {
        let declarations = summarize_omena_query_source_import_declarations_for_source_language(
            source_path,
            source,
            source_language,
        );
        let resolutions = imported_style_bindings
            .into_iter()
            .flat_map(|binding| {
                declarations
                    .imports
                    .iter()
                    .filter(move |declaration| declaration.binding == binding.binding)
                    .map(move |declaration| {
                        declaration.style_resolution(binding.style_uri.as_str())
                    })
            })
            .collect();
        super::summarize_omena_query_source_syntax_index_for_source_language(
            source_path,
            source,
            source_language,
            resolutions,
        )
    }

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

    fn unused_selector_shared_probe_corpus() -> Vec<OmenaQueryStyleSourceInputV0> {
        [
            ("A.module.css", "usedA", "ghostA"),
            ("B.module.css", "usedB", "ghostB"),
            ("C.module.css", "usedC", "ghostC"),
        ]
        .into_iter()
        .map(|(file_name, used, unused)| OmenaQueryStyleSourceInputV0 {
            style_path: format!("/workspace/src/{file_name}"),
            style_source: format!(".{used} {{ color: red; }}\n.{unused} {{ color: blue; }}\n"),
        })
        .collect()
    }

    fn unused_selector_shared_probe_documents() -> Vec<OmenaQuerySourceDocumentInputV0> {
        vec![OmenaQuerySourceDocumentInputV0 {
            source_path: "/workspace/src/App.tsx".to_string(),
            source_source: r#"import a from "./A.module.css";
import b from "./B.module.css";
import c from "./C.module.css";
export const classes = [a.usedA, b.usedB, c.usedC];
"#
            .to_string(),
            source_syntax_index: None,
            has_unresolved_style_import: false,
        }]
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

    #[test]
    fn committed_css_modules_graph_resolves_alias_targets_and_reachability()
    -> Result<(), &'static str> {
        let corpus = vec![
            OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace/src/styles/base.module.css".to_string(),
                style_source: ".base { color: red; }".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace/src/Card.module.css".to_string(),
                style_source: ".card { composes: base from \"@styles/base.module.css\"; }"
                    .to_string(),
            },
        ];
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0 {
            tsconfig_path_mappings: vec![OmenaResolverTsconfigPathMappingV0 {
                base_path: "/workspace".to_string(),
                pattern: "@styles/*".to_string(),
                target_patterns: vec!["src/styles/*".to_string()],
            }],
            ..OmenaQueryStyleResolutionInputsV0::default()
        };
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let selector = host
            .workspace_revision_selector(corpus.as_slice(), &[], &[], &[], &resolution_inputs)
            .ok_or("alias corpus must commit a selector")?;
        let edge = selector
            .css_modules_cross_file_resolution()
            .edges
            .iter()
            .find(|edge| edge.import_kind == "composes")
            .ok_or("alias composes edge must be present")?;

        assert_eq!(
            edge.resolved_style_path.as_deref(),
            Some("/workspace/src/styles/base.module.css"),
        );
        assert_eq!(edge.matched_names, vec!["base"]);
        assert_eq!(edge.import_graph_distance, Some(1));
        Ok(())
    }

    #[test]
    fn committed_workspace_summary_matches_alias_aware_source_projection()
    -> Result<(), &'static str> {
        let style_path = "/workspace/src/styles/Button.module.scss";
        let corpus = vec![OmenaQueryStyleSourceInputV0 {
            style_path: style_path.to_string(),
            style_source: ".root { color: red; }".to_string(),
        }];
        let source_documents = vec![OmenaQuerySourceDocumentInputV0 {
            source_path: "/workspace/src/Button.tsx".to_string(),
            source_source: r#"import styles from "@styles/Button.module.scss";
const cls = styles.root;"#
                .to_string(),
            source_syntax_index: None,
            has_unresolved_style_import: false,
        }];
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0 {
            tsconfig_path_mappings: vec![OmenaResolverTsconfigPathMappingV0 {
                base_path: "/workspace".to_string(),
                pattern: "@styles/*".to_string(),
                target_patterns: vec!["src/styles/*".to_string()],
            }],
            ..OmenaQueryStyleResolutionInputsV0::default()
        };
        let source_summary =
            crate::summarize_omena_query_source_selector_reference_cross_file_summary_with_resolution_inputs(
                corpus.as_slice(),
                source_documents.as_slice(),
                &[],
                &resolution_inputs,
            );
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let selector = host
            .workspace_revision_selector(
                corpus.as_slice(),
                source_documents.as_slice(),
                &[],
                &[],
                &resolution_inputs,
            )
            .ok_or("alias source corpus must commit a selector")?;
        let workspace_summary = selector.workspace_cross_file_summary();
        let source_edges = workspace_summary
            .edges
            .iter()
            .filter(|edge| edge.edge_kind == "sourceSelectorReference")
            .cloned()
            .collect::<Vec<_>>();

        assert_eq!(source_summary.summary_edge_count, 1);
        assert_eq!(source_edges, source_summary.edges);
        assert_eq!(source_edges[0].target_path.as_deref(), Some(style_path));
        Ok(())
    }

    #[test]
    fn committed_sass_graph_propagates_configurable_names_through_alias_forwards()
    -> Result<(), &'static str> {
        let corpus = vec![
            OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace/src/styles/_tokens.scss".to_string(),
                style_source: "$brand: red !default;".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace/src/_theme.scss".to_string(),
                style_source: "@forward \"@styles/tokens\";".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace/src/app.scss".to_string(),
                style_source: "@use \"./theme\" with ($brand: blue);".to_string(),
            },
        ];
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0 {
            tsconfig_path_mappings: vec![OmenaResolverTsconfigPathMappingV0 {
                base_path: "/workspace".to_string(),
                pattern: "@styles/*".to_string(),
                target_patterns: vec!["src/styles/*".to_string()],
            }],
            ..OmenaQueryStyleResolutionInputsV0::default()
        };
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let selector = host
            .workspace_revision_selector(corpus.as_slice(), &[], &[], &[], &resolution_inputs)
            .ok_or("alias Sass corpus must commit a selector")?;
        let app_edge = selector
            .sass_module_cross_file_resolution()
            .edges
            .iter()
            .find(|edge| edge.from_style_path == "/workspace/src/app.scss")
            .ok_or("configured app edge must be present")?;

        assert_eq!(app_edge.status, "resolved");
        assert_eq!(app_edge.configuration_variable_count, 1);
        assert_eq!(
            app_edge.invalid_configuration_variable_names,
            Vec::<String>::new(),
        );
        Ok(())
    }

    fn set_of(paths: impl IntoIterator<Item = &'static str>) -> BTreeSet<String> {
        paths.into_iter().map(str::to_string).collect()
    }

    fn fixed_view_diagnostics_json(
        sync: &OmenaQueryStyleParallelResolveSyncV0,
        target_style_path: &str,
    ) -> Result<String, &'static str> {
        fixed_view_diagnostics_json_with_committed_graph(
            sync,
            &sync.committed_graph,
            target_style_path,
        )
    }

    fn fixed_view_diagnostics_json_with_committed_graph(
        sync: &OmenaQueryStyleParallelResolveSyncV0,
        committed_graph: &OmenaQueryCommittedStyleSemanticGraphV0,
        target_style_path: &str,
    ) -> Result<String, &'static str> {
        let (_, file) = sync
            .files
            .iter()
            .find(|(style_path, _)| style_path == target_style_path)
            .ok_or("target style path must be present in the fixed view")?;
        let db = OmenaQueryStyleMemoDatabaseV0::from_handle(sync.handle.clone());
        let summary = resolve_committed_workspace_style_diagnostics_from_view(
            &db,
            sync.workspace,
            *file,
            committed_graph,
        );
        serde_json::to_string(&summary).map_err(|_| "fixed view diagnostics must serialize")
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
            commit.changed_module_interface_paths, commit.changed_style_paths,
            "initial transaction exposes every style module interface as changed",
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
            edited_commit.changed_module_interface_paths,
            set_of(["/workspace/src/App.module.scss"]),
            "adding an exported selector must report the changed module interface",
        );

        assert_eq!(
            style_fact_entry_probe::read(),
            set_of(["/workspace/src/App.module.scss"]),
            "transaction commit must preserve the per-file salsa firewall",
        );
        Ok(())
    }

    #[test]
    fn workspace_transaction_reports_only_changed_module_interfaces() -> Result<(), &'static str> {
        let mut corpus = vec![OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/Card.module.scss".to_string(),
            style_source: ".card { color: red; }\n".to_string(),
        }];
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();

        let mut transaction = OmenaQueryStyleWorkspaceTransactionV0::new();
        transaction
            .register_style_sources(corpus.as_slice())
            .set_workspace_inputs(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        let initial_commit = transaction
            .commit_revision(&mut host)
            .map_err(|_| "initial registered transaction must commit")?;
        assert_eq!(
            initial_commit.changed_module_interface_paths,
            set_of(["/workspace/src/Card.module.scss"]),
        );

        corpus[0].style_source = ".card { color: blue; }\n".to_string();
        let mut transaction = OmenaQueryStyleWorkspaceTransactionV0::new();
        transaction
            .register_style_sources(corpus.as_slice())
            .set_workspace_inputs(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        let body_only_commit = transaction
            .commit_revision(&mut host)
            .map_err(|_| "body-only registered transaction must commit")?;
        assert_eq!(
            body_only_commit.changed_style_paths,
            set_of(["/workspace/src/Card.module.scss"]),
            "the source text changed",
        );
        assert!(
            body_only_commit.changed_module_interface_paths.is_empty(),
            "declaration-body edits must not publish downstream module-interface changes",
        );
        assert!(
            body_only_commit
                .selector
                .changed_module_interface_paths()
                .is_empty(),
            "selector snapshots expose the same module-interface delta as the commit",
        );

        corpus[0].style_source =
            ".card { color: blue; }\n.card__icon { color: currentColor; }\n".to_string();
        let mut transaction = OmenaQueryStyleWorkspaceTransactionV0::new();
        transaction
            .register_style_sources(corpus.as_slice())
            .set_workspace_inputs(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        let export_commit = transaction
            .commit_revision(&mut host)
            .map_err(|_| "export-affecting registered transaction must commit")?;
        assert_eq!(
            export_commit.changed_module_interface_paths,
            set_of(["/workspace/src/Card.module.scss"]),
            "selector-surface edits must publish downstream module-interface changes",
        );
        assert_eq!(
            export_commit.selector.changed_module_interface_paths(),
            &export_commit.changed_module_interface_paths,
        );
        Ok(())
    }

    #[test]
    fn workspace_transaction_reports_public_sass_member_changes() -> Result<(), &'static str> {
        let style_path = "/workspace/src/_tokens.scss";
        let mut corpus = vec![OmenaQueryStyleSourceInputV0 {
            style_path: style_path.to_string(),
            style_source: "$tone: red;\n".to_string(),
        }];
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();

        for source in ["$tone: red;\n", "$tone: blue;\n", "$accent: blue;\n"] {
            corpus[0].style_source = source.to_string();
            let mut transaction = OmenaQueryStyleWorkspaceTransactionV0::new();
            transaction
                .register_style_sources(corpus.as_slice())
                .set_workspace_inputs(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
            let commit = transaction
                .commit_revision(&mut host)
                .map_err(|_| "Sass module transaction must commit")?;

            match source {
                "$tone: blue;\n" => assert!(
                    commit.changed_module_interface_paths.is_empty(),
                    "changing a public variable value must preserve its member interface"
                ),
                _ => assert_eq!(
                    commit.changed_module_interface_paths,
                    set_of([style_path]),
                    "adding or removing a public Sass variable must change the module interface"
                ),
            }
        }
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
    fn workspace_transaction_reuses_file_facts_when_resolution_inputs_change()
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

        let changed_resolution_inputs = OmenaQueryStyleResolutionInputsV0 {
            external_sif_cache_fingerprint: Some("updated-cache".to_string()),
            ..OmenaQueryStyleResolutionInputsV0::default()
        };
        style_fact_entry_probe::reset();
        reset_committed_style_semantic_graph_compute_count_for_test();
        let mut changed_transaction = OmenaQueryStyleWorkspaceTransactionV0::new();
        changed_transaction
            .register_style_sources(corpus.as_slice())
            .set_workspace_inputs(corpus.as_slice(), &[], &[], &[], &changed_resolution_inputs);
        let changed_commit = changed_transaction
            .commit_revision(&mut host)
            .map_err(|_| "resolution input transaction must commit")?;

        assert_eq!(
            changed_commit.revision.value,
            initial_commit.revision.value + 1,
            "workspace-level resolution changes must advance the committed revision",
        );
        assert!(
            changed_commit.changed_style_paths.is_empty(),
            "workspace-level inputs must not masquerade as changed style source files",
        );
        assert_eq!(
            read_committed_style_semantic_graph_compute_count_for_test(),
            1,
            "workspace-level resolution changes must rebuild the committed graph",
        );
        assert!(
            style_fact_entry_probe::read().is_empty(),
            "workspace-level resolution changes must reuse unchanged per-file facts",
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
    fn workspace_style_diagnostics_with_selector_reports_snapshot_id() -> Result<(), &'static str> {
        let corpus = parallel_probe_corpus();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();

        let initial = host
            .workspace_style_diagnostics_with_selector(
                "/workspace/src/App.module.scss",
                corpus.as_slice(),
                &[],
                &[],
                &[],
                &resolution_inputs,
            )
            .ok_or("initial selector diagnostics must resolve")?;
        assert_eq!(
            initial.snapshot_id(),
            OmenaWorkspaceSnapshotIdV0::from_revision(IncrementalRevisionV0 { value: 1 }),
        );
        assert_eq!(initial.snapshot_id, initial.snapshot_id());
        assert_eq!(initial.snapshot_id(), initial.selector.snapshot_id());

        let unchanged = host
            .workspace_style_diagnostics_with_selector(
                "/workspace/src/App.module.scss",
                corpus.as_slice(),
                &[],
                &[],
                &[],
                &resolution_inputs,
            )
            .ok_or("unchanged selector diagnostics must resolve")?;
        assert_eq!(
            unchanged.snapshot_id(),
            initial.snapshot_id(),
            "unchanged inputs must keep the workspace snapshot id stable",
        );

        let mut edited_corpus = corpus.clone();
        edited_corpus[0]
            .style_source
            .push_str("\n.app__icon { color: blue; }\n");
        let edited = host
            .workspace_style_diagnostics_with_selector(
                "/workspace/src/App.module.scss",
                edited_corpus.as_slice(),
                &[],
                &[],
                &[],
                &resolution_inputs,
            )
            .ok_or("edited selector diagnostics must resolve")?;
        assert_eq!(
            edited.snapshot_id(),
            OmenaWorkspaceSnapshotIdV0::from_revision(IncrementalRevisionV0 { value: 2 }),
        );
        assert_ne!(edited.snapshot_id(), initial.snapshot_id());
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
        memo_style_cascade_projection_probe::reset();
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
        assert_eq!(
            memo_style_cascade_projection_probe::count(),
            0,
            "diagnostics-only hot path must not invoke the tracked cascade projection",
        );
        Ok(())
    }

    #[test]
    fn revision_selector_reuses_committed_resolution_and_per_file_cascade_declarations()
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
        cascade_declarations_collect_probe::reset();
        let first_substrate = commit.selector.style_cascade_narrowing_substrate();
        let second_substrate = commit.selector.style_cascade_narrowing_substrate();
        assert_eq!(first_substrate, direct_substrate);
        assert_eq!(second_substrate, direct_substrate);
        assert_eq!(
            read_sass_module_resolution_internal_compute_count_for_test(),
            0,
            "selector substrate lookup must reuse the Sass resolution committed with the graph",
        );
        assert_eq!(
            cascade_declarations_collect_probe::count(),
            0,
            "selector substrate lookups must reuse declarations committed through the per-file tracked queries",
        );
        Ok(())
    }

    #[test]
    fn style_fact_and_cascade_queries_share_one_parser_materialization() {
        let corpus = vec![OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/App.module.scss".to_string(),
            style_source: "@layer theme { .card { color: red; } }".to_string(),
        }];
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let workspace = host.sync_workspace(
            corpus.as_slice(),
            &[],
            &[],
            &[],
            &OmenaQueryStyleResolutionInputsV0::default(),
        );
        let file = workspace.files(&host.db)[0];

        let (_, instrumentation) = omena_parser::with_omena_parser_parse_instrumentation(|| {
            let _ = memo_style_fact_entry(&host.db, file);
            let _ = memo_style_cascade_declarations(&host.db, file);
        });

        assert_eq!(instrumentation.parse_invocation_count, 1);
    }

    #[test]
    fn parser_materialization_lives_with_the_memo_session_and_releases_with_it()
    -> Result<(), &'static str> {
        let corpus = vec![OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/App.module.scss".to_string(),
            style_source: ".card { color: red; }".to_string(),
        }];
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let workspace = host.sync_workspace(
            corpus.as_slice(),
            &[],
            &[],
            &[],
            &OmenaQueryStyleResolutionInputsV0::default(),
        );
        let file = workspace.files(&host.db)[0];
        let weak_parser = {
            let fact_entry = memo_style_fact_entry(&host.db, file);
            fact_entry
                .parser_materialization_weak()
                .ok_or("memoized fact entry must retain its parser materialization")?
        };

        assert!(
            weak_parser.upgrade().is_some(),
            "the session cache must retain parser state after the caller clone is dropped",
        );
        let hot_fact_entry = memo_style_fact_entry(&host.db, file);
        drop(hot_fact_entry);
        assert!(
            weak_parser.upgrade().is_some(),
            "a hot lookup must leave the parser sidecar owned by the live session",
        );

        drop(host);
        assert!(
            weak_parser.upgrade().is_none(),
            "dropping the memo host must release its parser sidecar",
        );
        Ok(())
    }

    #[test]
    fn revision_selector_recollects_cascade_declarations_for_only_the_edited_style_file()
    -> Result<(), &'static str> {
        let mut corpus = doubled_parallel_probe_corpus();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let initial = host
            .workspace_revision_selector(corpus.as_slice(), &[], &[], &[], &resolution_inputs)
            .ok_or("initial selector must commit")?;
        let _ = initial.style_cascade_narrowing_substrate();

        corpus[0]
            .style_source
            .push_str("\n.edited { color: currentColor; }\n");
        cascade_declarations_collect_probe::reset();
        let edited = host
            .workspace_revision_selector(corpus.as_slice(), &[], &[], &[], &resolution_inputs)
            .ok_or("edited selector must commit")?;
        let incremental = edited.style_cascade_narrowing_substrate();
        assert_eq!(
            cascade_declarations_collect_probe::count(),
            1,
            "a single-file edit must recollect exactly that file's cascade declarations",
        );
        cascade_declarations_collect_probe::reset();
        let direct = collect_omena_query_style_cascade_narrowing_substrate_with_external_sifs(
            corpus.as_slice(),
            &[],
            &[],
            &resolution_inputs,
        );

        assert_eq!(incremental, direct);
        assert_eq!(
            cascade_declarations_collect_probe::count(),
            corpus.len(),
            "the direct oracle must still exercise the full corpus",
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
    fn complete_empty_source_corpus_reaches_the_workspace_memo() -> Result<(), &'static str> {
        let corpus = vec![OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/App.module.css".to_string(),
            style_source: ".unused { color: red; }\n".to_string(),
        }];
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let expected =
            crate::summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs_and_complete_source_corpus(
                corpus[0].style_path.as_str(),
                corpus.as_slice(),
                &[],
                &[],
                None,
                OmenaQueryExternalModuleModeV0::Auto,
                &[],
                &resolution_inputs,
            );

        let mut host = OmenaQueryStyleMemoHostV0::new();
        let selector = host
            .workspace_revision_selector_with_complete_source_corpus(
                corpus.as_slice(),
                &[],
                &[],
                &[],
                &resolution_inputs,
            )
            .ok_or("complete source corpus must commit a selector")?;
        let target = selector
            .files_by_path
            .get(corpus[0].style_path.as_str())
            .copied()
            .ok_or("target style must exist")?;
        let actual =
            resolve_committed_workspace_style_diagnostics_from_view_with_external_mode_and_suppression_mode_and_precomputed_unused_selector(
                &selector.db,
                selector.workspace,
                target,
                &selector.committed_graph,
                OmenaQueryExternalModuleModeV0::Auto,
                OmenaQueryDiagnosticSuppressionModeV0::Apply,
                None,
                true,
            );

        assert_eq!(
            serde_json::to_string(&actual).ok(),
            serde_json::to_string(&expected).ok(),
            "memoized diagnostics must preserve a complete empty source corpus"
        );
        Ok(())
    }

    #[test]
    fn committed_unused_selector_summary_is_byte_identical_and_revision_scoped()
    -> Result<(), &'static str> {
        let corpus = unused_selector_shared_probe_corpus();
        let source_documents = unused_selector_shared_probe_documents();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let references = corpus
            .iter()
            .map(|target| {
                summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs_and_suppression_mode(
                    target.style_path.as_str(),
                    corpus.as_slice(),
                    source_documents.as_slice(),
                    &[],
                    None,
                    OmenaQueryExternalModuleModeV0::Auto,
                    &[],
                    &resolution_inputs,
                    OmenaQueryDiagnosticSuppressionModeV0::Apply,
                )
            })
            .map(|summary| {
                serde_json::to_string(&summary)
                    .map_err(|_| "reference diagnostics must serialize")
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut host = OmenaQueryStyleMemoHostV0::new();
        let selector = host
            .workspace_revision_selector(
                corpus.as_slice(),
                source_documents.as_slice(),
                &[],
                &[],
                &resolution_inputs,
            )
            .ok_or("workspace selector must commit")?;
        crate::style::diagnostics::reset_unused_selector_shared_walk_count_for_test();
        let actual = corpus
            .iter()
            .map(|target| selector.workspace_style_diagnostics(target.style_path.as_str()))
            .map(|summary| {
                serde_json::to_string(&summary).map_err(|_| "memo diagnostics must serialize")
            })
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(actual, references);
        assert_eq!(
            crate::style::diagnostics::read_unused_selector_shared_walk_count_for_test(),
            1
        );

        for target in &corpus {
            let _ = selector.workspace_style_diagnostics(target.style_path.as_str());
        }
        assert_eq!(
            crate::style::diagnostics::read_unused_selector_shared_walk_count_for_test(),
            1,
            "repeated reads of one revision must retain the shared result"
        );
        drop(selector);

        let mut edited_documents = source_documents;
        edited_documents[0].source_source = r#"import a from "./A.module.css";
import b from "./B.module.css";
import c from "./C.module.css";
export const classes = [a.ghostA, b.usedB, c.usedC];
"#
        .to_string();
        let edited_selector = host
            .workspace_revision_selector(
                corpus.as_slice(),
                edited_documents.as_slice(),
                &[],
                &[],
                &resolution_inputs,
            )
            .ok_or("edited workspace selector must commit")?;
        for target in &corpus {
            let _ = edited_selector.workspace_style_diagnostics(target.style_path.as_str());
        }
        assert_eq!(
            crate::style::diagnostics::read_unused_selector_shared_walk_count_for_test(),
            2,
            "a changed source revision must compute one new shared result"
        );
        Ok(())
    }

    #[test]
    fn non_default_unused_selector_axes_retain_target_local_resolution() -> Result<(), &'static str>
    {
        let corpus = unused_selector_shared_probe_corpus();
        let source_documents = unused_selector_shared_probe_documents();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let selector = host
            .workspace_revision_selector(
                corpus.as_slice(),
                source_documents.as_slice(),
                &[],
                &[],
                &resolution_inputs,
            )
            .ok_or("workspace selector must commit")?;
        let targets = selector
            .files
            .iter()
            .take(2)
            .map(|(_, target)| *target)
            .collect::<Vec<_>>();

        let transform_reference = targets
            .iter()
            .map(|target| {
                summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs_and_suppression_mode(
                    target.style_path(&selector.db).as_str(),
                    corpus.as_slice(),
                    source_documents.as_slice(),
                    &[],
                    Some("camelCase"),
                    OmenaQueryExternalModuleModeV0::Auto,
                    &[],
                    &resolution_inputs,
                    OmenaQueryDiagnosticSuppressionModeV0::Apply,
                )
            })
            .map(|summary| {
                serde_json::to_string(&summary)
                    .map_err(|_| "transform reference diagnostics must serialize")
            })
            .collect::<Result<Vec<_>, _>>()?;
        crate::style::diagnostics::reset_unused_selector_shared_walk_count_for_test();
        let transform_actual = targets
            .iter()
            .map(|target| {
                resolve_committed_workspace_style_diagnostics_from_view_with_external_mode_and_suppression_mode_and_identity_index(
                    &selector.db,
                    selector.workspace,
                    *target,
                    &selector.committed_graph,
                    OmenaQueryExternalModuleModeV0::Auto,
                    OmenaQueryDiagnosticSuppressionModeV0::Apply,
                    Some("camelCase"),
                    None,
                )
            })
            .map(|summary| {
                serde_json::to_string(&summary)
                    .map_err(|_| "transform committed diagnostics must serialize")
            })
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(transform_actual, transform_reference);
        assert_eq!(
            crate::style::diagnostics::read_unused_selector_shared_walk_count_for_test(),
            2,
            "a classname transform changes selector attribution and must bypass the default memo"
        );

        let identity_index = OmenaResolverStyleModuleConfirmationIdentityIndexV0 {
            available_by_identity: BTreeMap::new(),
            disk_by_identity: BTreeMap::new(),
        };
        let substrate = collect_omena_query_workspace_diagnostics_substrate_from_committed_graph(
            selector.committed_graph.style_fact_entries.clone(),
            &selector.committed_graph.css_modules_resolution,
            &selector.committed_graph.sass_module_resolution,
            &selector
                .committed_graph
                .sass_module_resolution_without_manifests,
            &selector
                .committed_graph
                .sass_module_resolution_without_path_mappings,
            &selector
                .committed_graph
                .sass_module_resolution_with_external_sifs,
        );
        let identity_reference = targets
            .iter()
            .map(|target| {
                summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs_and_suppression_mode_with_substrate(
                    target.style_path(&selector.db).as_str(),
                    corpus.as_slice(),
                    source_documents.as_slice(),
                    &[],
                    None,
                    OmenaQueryExternalModuleModeV0::Auto,
                    &[],
                    &resolution_inputs,
                    OmenaQueryDiagnosticSuppressionModeV0::Apply,
                    &substrate,
                    Some(&identity_index),
                )
            })
            .map(|summary| {
                serde_json::to_string(&summary)
                    .map_err(|_| "identity reference diagnostics must serialize")
            })
            .collect::<Result<Vec<_>, _>>()?;
        crate::style::diagnostics::reset_unused_selector_shared_walk_count_for_test();
        let identity_actual = targets
            .iter()
            .map(|target| {
                resolve_committed_workspace_style_diagnostics_from_view_with_external_mode_and_suppression_mode_and_identity_index(
                    &selector.db,
                    selector.workspace,
                    *target,
                    &selector.committed_graph,
                    OmenaQueryExternalModuleModeV0::Auto,
                    OmenaQueryDiagnosticSuppressionModeV0::Apply,
                    None,
                    Some(&identity_index),
                )
            })
            .map(|summary| {
                serde_json::to_string(&summary)
                    .map_err(|_| "identity committed diagnostics must serialize")
            })
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(identity_actual, identity_reference);
        assert_eq!(
            crate::style::diagnostics::read_unused_selector_shared_walk_count_for_test(),
            2,
            "an explicit resolver identity index must retain its target-local resolution path"
        );
        Ok(())
    }

    #[test]
    fn parallel_resolve_sync_records_committed_revision_watermark() -> Result<(), &'static str> {
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

        let mut reference_host = OmenaQueryStyleMemoHostV0::new();
        let reference_sync = reference_host
            .sync_workspace_for_parallel_resolve(
                corpus.as_slice(),
                &[],
                &[],
                &[],
                &resolution_inputs,
            )
            .ok_or("reference corpus must sync for parallel resolve")?;

        assert_eq!(
            sync.revision,
            IncrementalRevisionV0 { value: 1 },
            "fixed read bundles must carry the committed revision they were minted from",
        );
        assert_eq!(
            sync.revision, reference_sync.revision,
            "independent hosts should mint the same first committed revision for the same corpus",
        );
        assert_eq!(
            sync.committed_graph, reference_sync.committed_graph,
            "the fixed view graph must match an independently rebuilt graph at the same revision",
        );
        Ok(())
    }

    #[test]
    fn parallel_resolve_worker_db_stays_pinned_after_host_commit() -> Result<(), &'static str> {
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
        let initial_json = fixed_view_diagnostics_json(&sync, "/workspace/src/App.module.scss")?;

        let mut edited_corpus = corpus.clone();
        edited_corpus[0].style_source =
            format!("@use \"./missing\";\n{}", edited_corpus[0].style_source);
        let edited_sync = host
            .sync_workspace_for_parallel_resolve(
                edited_corpus.as_slice(),
                &[],
                &[],
                &[],
                &resolution_inputs,
            )
            .ok_or("edited corpus must sync for parallel resolve")?;
        assert_eq!(
            edited_sync.revision,
            IncrementalRevisionV0 { value: 2 },
            "the intervening host commit must advance the live revision",
        );

        let pinned_after_commit_json =
            fixed_view_diagnostics_json(&sync, "/workspace/src/App.module.scss")?;
        assert_eq!(
            pinned_after_commit_json, initial_json,
            "worker reads through a fixed handle must not observe later host commits",
        );

        let fresh_json =
            fixed_view_diagnostics_json(&edited_sync, "/workspace/src/App.module.scss")?;
        assert_ne!(
            fresh_json, initial_json,
            "a fresh fixed view for the edited commit must observe the changed diagnostics",
        );

        let leaked_handle_with_pinned_graph_json =
            fixed_view_diagnostics_json_with_committed_graph(
                &edited_sync,
                &sync.committed_graph,
                "/workspace/src/App.module.scss",
            )?;
        assert_ne!(
            leaked_handle_with_pinned_graph_json, initial_json,
            "the pinned-read witness must distinguish a newer handle even when the committed graph is fixed",
        );
        Ok(())
    }

    #[test]
    fn committed_graph_delete_commit_matches_fresh_reduced_corpus() -> Result<(), &'static str> {
        let corpus = css_modules_resolution_probe_corpus();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();

        let initial_selector = host
            .workspace_revision_selector(corpus.as_slice(), &[], &[], &[], &resolution_inputs)
            .ok_or("initial corpus must commit a selector")?;
        let mut reduced_corpus = corpus.clone();
        reduced_corpus.retain(|source| source.style_path != "/workspace/src/base.module.css");

        reset_committed_style_semantic_graph_compute_count_for_test();
        let delete_selector = host
            .workspace_revision_selector(
                reduced_corpus.as_slice(),
                &[],
                &[],
                &[],
                &resolution_inputs,
            )
            .ok_or("reduced corpus must commit a selector")?;
        let delete_compute_count = read_committed_style_semantic_graph_compute_count_for_test();

        let mut fresh_host = OmenaQueryStyleMemoHostV0::new();
        let fresh_selector = fresh_host
            .workspace_revision_selector(
                reduced_corpus.as_slice(),
                &[],
                &[],
                &[],
                &resolution_inputs,
            )
            .ok_or("fresh reduced corpus must commit a selector")?;

        assert_ne!(
            initial_selector.committed_style_semantic_graph(),
            delete_selector.committed_style_semantic_graph(),
            "removing an imported module must change the committed graph surface",
        );
        assert_eq!(
            delete_selector.committed_style_semantic_graph(),
            fresh_selector.committed_style_semantic_graph(),
            "delete commits must retract the removed file exactly like a fresh reduced build",
        );
        assert_eq!(
            delete_compute_count, 1,
            "the delete commit must record graph construction on the measured path",
        );
        assert_eq!(
            read_committed_style_semantic_graph_compute_count_for_test(),
            2,
            "the independent reduced-corpus reference must record its own graph construction",
        );
        Ok(())
    }

    fn assert_committed_graph_edit_records_construction(
        mut corpus: Vec<OmenaQueryStyleSourceInputV0>,
    ) -> Result<(), &'static str> {
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let initial_selector = host
            .workspace_revision_selector(corpus.as_slice(), &[], &[], &[], &resolution_inputs)
            .ok_or("initial corpus must commit a selector")?;

        corpus[0]
            .style_source
            .push_str("\n.committedGraphEdit { color: currentColor; }\n");
        reset_committed_style_semantic_graph_compute_count_for_test();
        let edited_selector = host
            .workspace_revision_selector(corpus.as_slice(), &[], &[], &[], &resolution_inputs)
            .ok_or("edited corpus must commit a selector")?;

        assert_ne!(
            initial_selector.committed_style_semantic_graph(),
            edited_selector.committed_style_semantic_graph(),
            "the edit fixture must change the committed graph surface",
        );
        assert_eq!(
            read_committed_style_semantic_graph_compute_count_for_test(),
            1,
            "the committed graph edit path must record graph construction once for the edit",
        );
        Ok(())
    }

    #[test]
    fn committed_graph_edit_path_records_construction_for_each_corpus_scale()
    -> Result<(), &'static str> {
        assert_committed_graph_edit_records_construction(parallel_probe_corpus())?;
        assert_committed_graph_edit_records_construction(doubled_parallel_probe_corpus())?;
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
    fn source_workspace_projection_cuts_off_selector_stable_tsx_body_edits() {
        let styles = source_workspace_projection_style_corpus();
        let mut documents = vec![source_workspace_projection_document(
            "/workspace/src/App.tsx",
            "card",
            "one",
        )];
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let workspace = host.sync_workspace(
            styles.as_slice(),
            documents.as_slice(),
            &[],
            &[],
            &resolution_inputs,
        );
        exercise_source_workspace_projection_consumers(&host.db, workspace);

        documents[0] =
            source_workspace_projection_document("/workspace/src/App.tsx", "card", "two");
        let edited_workspace = host.sync_workspace(
            styles.as_slice(),
            documents.as_slice(),
            &[],
            &[],
            &resolution_inputs,
        );
        source_workspace_projection_probe::reset();
        source_workspace_query_probe::reset();
        exercise_source_workspace_projection_consumers(&host.db, edited_workspace);

        assert_eq!(
            source_workspace_projection_probe::read(),
            set_of(["/workspace/src/App.tsx"]),
            "the edited file projection must run so zero workspace recomputes cannot be dead code",
        );
        assert_eq!(
            source_workspace_query_probe::read(),
            (0, 0),
            "a selector-stable body edit must cut off both workspace consumers",
        );
    }

    #[test]
    fn source_workspace_projection_recomputes_for_selector_reference_changes() {
        let styles = source_workspace_projection_style_corpus();
        let mut documents = vec![source_workspace_projection_document(
            "/workspace/src/App.tsx",
            "card",
            "one",
        )];
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let workspace = host.sync_workspace(
            styles.as_slice(),
            documents.as_slice(),
            &[],
            &[],
            &resolution_inputs,
        );
        exercise_source_workspace_projection_consumers(&host.db, workspace);

        documents[0] =
            source_workspace_projection_document("/workspace/src/App.tsx", "tile", "one");
        let edited_workspace = host.sync_workspace(
            styles.as_slice(),
            documents.as_slice(),
            &[],
            &[],
            &resolution_inputs,
        );
        source_workspace_projection_probe::reset();
        source_workspace_query_probe::reset();
        exercise_source_workspace_projection_consumers(&host.db, edited_workspace);

        assert_eq!(
            source_workspace_projection_probe::read(),
            set_of(["/workspace/src/App.tsx"]),
        );
        assert_eq!(
            source_workspace_query_probe::read(),
            (1, 1),
            "the negative control must recompute both consumers when a selector reference changes",
        );
    }

    #[test]
    fn source_workspace_projection_recompute_set_is_the_single_renamed_file() {
        let styles = source_workspace_projection_style_corpus();
        let mut documents = vec![
            source_workspace_projection_document("/workspace/src/App.tsx", "card", "one"),
            source_workspace_projection_document("/workspace/src/Peer.tsx", "tile", "one"),
        ];
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let workspace = host.sync_workspace(
            styles.as_slice(),
            documents.as_slice(),
            &[],
            &[],
            &resolution_inputs,
        );
        exercise_source_workspace_projection_consumers(&host.db, workspace);

        documents[1] =
            source_workspace_projection_document("/workspace/src/Peer.tsx", "card", "one");
        let edited_workspace = host.sync_workspace(
            styles.as_slice(),
            documents.as_slice(),
            &[],
            &[],
            &resolution_inputs,
        );
        source_workspace_projection_probe::reset();
        source_workspace_query_probe::reset();
        exercise_source_workspace_projection_consumers(&host.db, edited_workspace);

        assert_eq!(
            source_workspace_projection_probe::read(),
            set_of(["/workspace/src/Peer.tsx"]),
            "the recompute set must be the one source file whose selector reference changed",
        );
        assert_eq!(source_workspace_query_probe::read(), (1, 1));
    }

    fn source_workspace_projection_style_corpus() -> Vec<OmenaQueryStyleSourceInputV0> {
        vec![OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/App.module.scss".to_string(),
            style_source: ".card { color: red; }\n.tile { color: blue; }\n".to_string(),
        }]
    }

    fn source_workspace_projection_document(
        source_path: &str,
        selector_name: &str,
        body_value: &str,
    ) -> OmenaQuerySourceDocumentInputV0 {
        let source = format!(
            "import styles from './App.module.scss';\nexport const App = () => <div className={{styles.{selector_name}}} data-body=\"{body_value}\" />;\n"
        );
        let index = summarize_omena_query_source_syntax_index_for_source_language(
            source_path,
            source.as_str(),
            Some("typescriptreact"),
            vec![OmenaQuerySourceImportedStyleBindingV0 {
                binding: "styles".to_string(),
                style_uri: "/workspace/src/App.module.scss".to_string(),
            }],
            Vec::new(),
        );
        OmenaQuerySourceDocumentInputV0 {
            source_path: source_path.to_string(),
            source_source: source,
            source_syntax_index: Some(index),
            has_unresolved_style_import: false,
        }
    }

    fn exercise_source_workspace_projection_consumers(
        db: &OmenaQueryStyleMemoDatabaseV0,
        workspace: OmenaQueryStyleWorkspaceInputV0,
    ) {
        let _ = memo_workspace_unused_selector_shared(db, workspace, true);
        let _ = memo_workspace_cross_file_summary_from_module_interfaces(db, workspace);
    }

    #[test]
    #[ignore = "timing harness - run with --ignored --nocapture in release"]
    fn cold_build_timing_per_origin_vs_monolith() {
        for n in [50usize, 100, 150] {
            let corpus: Vec<OmenaQueryStyleSourceInputV0> = (0..n)
                .map(|i| OmenaQueryStyleSourceInputV0 {
                    style_path: format!("/workspace/src/F{i}.module.scss"),
                    style_source: format!(
                        "@use \"./F{}.module.scss\" as dep;\n@use \"@ext/tokens\" as t;\n@use \"$alias/mixins\" as m;\n.c{i} {{ color: red; }}\n.d{i} {{ composes: c{} from './F{}.module.scss'; }}\n.e{i} {{ composes: shared from '@theme/shared.module.css'; }}\n",
                        (i + 1) % n,
                        (i + 2) % n,
                        (i + 2) % n,
                    ),
                })
                .collect();
            let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();

            let mut host = OmenaQueryStyleMemoHostV0::new();
            let workspace =
                host.sync_workspace(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
            let started = std::time::Instant::now();
            let graph = build_committed_style_semantic_graph(
                &host.db,
                workspace,
                &[],
                &[],
                &[],
                &resolution_inputs,
            );
            let per_origin_ms = started.elapsed().as_millis();

            let mut host2 = OmenaQueryStyleMemoHostV0::new();
            let workspace2 =
                host2.sync_workspace(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
            let started2 = std::time::Instant::now();
            let graph2 = build_committed_style_semantic_graph_monolith(
                &host2.db,
                workspace2,
                &[],
                &[],
                &[],
                &resolution_inputs,
            );
            let monolith_ms = started2.elapsed().as_millis();
            assert_eq!(graph, graph2, "arms must stay byte-identical");
            println!(
                "N={n}: per-origin={per_origin_ms}ms monolith={monolith_ms}ms ratio={:.1}x",
                per_origin_ms as f64 / monolith_ms.max(1) as f64
            );
        }
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
    fn module_interface_projection_tracks_public_sass_members_by_namespace() {
        let initial = summarize_omena_query_module_interface_change_projection(
            "/workspace/src/_tokens.scss",
            r#"
$tone_value: red;
$_private-token: hidden;
@mixin paint_card($color) { color: $color; }
@mixin _private-mixin { color: red; }
@function scale_value($value) { @return $value * 2; }
@function -private-function() { @return 0; }
"#,
        );
        let body_only = summarize_omena_query_module_interface_change_projection(
            "/workspace/src/_tokens.scss",
            r#"
$tone-value: blue;
$_private-token: changed;
@mixin paint-card($color) { background: $color; }
@mixin _private-mixin { color: blue; }
@function scale-value($value) { @return $value * 3; }
@function -private-function() { @return 1; }
"#,
        );
        let removed = summarize_omena_query_module_interface_change_projection(
            "/workspace/src/_tokens.scss",
            "$accent: blue;\n",
        );

        assert_eq!(
            initial.sass_module_public_variable_names,
            set_of(["tone-value"])
        );
        assert_eq!(
            initial.sass_module_public_mixin_names,
            set_of(["paint-card"])
        );
        assert_eq!(
            initial.sass_module_public_function_names,
            set_of(["scale-value"])
        );
        assert_eq!(
            initial, body_only,
            "body edits and Sass hyphen/underscore spelling must preserve the public interface"
        );
        assert_ne!(
            initial, removed,
            "removing public Sass members must change the module interface projection"
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
        let initial_graph =
            memo_committed_style_semantic_graph_from_module_interfaces(&host.db, workspace);
        assert!(
            initial_graph
                .sass_module_resolution
                .graph_closure_edge_count
                > 0,
            "the backdating corpus must exercise a real cross-module closure",
        );

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
    fn workspace_cross_file_summary_counter_records_initial_graph_computation() {
        let corpus = sass_module_resolution_probe_corpus();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let workspace = host.sync_workspace(corpus.as_slice(), &[], &[], &[], &resolution_inputs);

        reset_workspace_cross_file_summary_internal_compute_count_for_test();
        let graph = memo_committed_style_semantic_graph_from_module_interfaces(&host.db, workspace);

        assert!(
            graph.cross_file_summary.summary_edge_count > 0,
            "the liveness corpus must produce an export-affecting cross-file summary",
        );
        assert!(
            read_workspace_cross_file_summary_internal_compute_count_for_test() > 0,
            "initial graph computation must record cross-file summary work",
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
    fn source_element_parent_chain_and_scope_proximity_cross_files_without_unrelated_reads() {
        let child_path = "/workspace/Child.tsx";
        let parent_path = "/workspace/Parent.tsx";
        let unrelated_path = "/workspace/Unrelated.tsx";
        let child_source = "export const Child = () => <span />;";
        let parent_source = "export const Parent = () => <main className=\"scope-root\" />;";
        let unrelated_source = "export const Unrelated = () => <aside />;";
        let mut child_index = summarize_omena_query_source_syntax_index_for_source_language(
            child_path,
            child_source,
            None,
            Vec::new(),
            Vec::new(),
        );
        let parent_index = summarize_omena_query_source_syntax_index_for_source_language(
            parent_path,
            parent_source,
            None,
            Vec::new(),
            Vec::new(),
        );
        let unrelated_index = summarize_omena_query_source_syntax_index_for_source_language(
            unrelated_path,
            unrelated_source,
            None,
            Vec::new(),
            Vec::new(),
        );
        let child = child_index.source_elements[0].identity.clone();
        let parent = parent_index.source_elements[0].identity.clone();
        child_index
            .element_parent_edges
            .push(OmenaQuerySourceElementParentFactV0 {
                child: child.clone(),
                parent: parent.clone(),
            });
        let mut documents = vec![
            OmenaQuerySourceDocumentInputV0 {
                source_path: child_path.to_string(),
                source_source: child_source.to_string(),
                source_syntax_index: Some(child_index),
                has_unresolved_style_import: false,
            },
            OmenaQuerySourceDocumentInputV0 {
                source_path: parent_path.to_string(),
                source_source: parent_source.to_string(),
                source_syntax_index: Some(parent_index),
                has_unresolved_style_import: false,
            },
            OmenaQuerySourceDocumentInputV0 {
                source_path: unrelated_path.to_string(),
                source_source: unrelated_source.to_string(),
                source_syntax_index: Some(unrelated_index),
                has_unresolved_style_import: false,
            },
        ];
        let target = omena_cascade::ElementIdentityV0 {
            source_path: child.source_path,
            byte_start: child.byte_span.start,
            byte_end: child.byte_span.end,
        };
        let expected_parent = omena_cascade::ElementIdentityV0 {
            source_path: parent.source_path,
            byte_start: parent.byte_span.start,
            byte_end: parent.byte_span.end,
        };
        let mut host = OmenaQueryStyleMemoHostV0::new();

        let initial = host.source_element_parent_chain(documents.as_slice(), target.clone());
        assert_eq!(initial.ancestors, vec![expected_parent]);
        assert!(initial.is_complete());
        assert_eq!(
            read_source_element_parent_chain_run_paths_for_test(),
            BTreeSet::from([child_path.to_string(), parent_path.to_string()]),
        );

        let proximity = host.source_scope_proximity(
            documents.as_slice(),
            initial.target.clone(),
            ".scope-root",
        );
        assert_eq!(
            proximity.status,
            omena_cascade::ScopeProximityStatusV0::Known
        );
        assert_eq!(proximity.distance, Some(1));

        documents[2].source_source.push_str("\n// unrelated edit\n");
        reset_source_element_parent_chain_run_paths_for_test();
        let after_unrelated_edit = host.source_element_parent_chain(documents.as_slice(), target);
        let proximity_after_unrelated_edit = host.source_scope_proximity(
            documents.as_slice(),
            initial.target.clone(),
            ".scope-root",
        );

        assert_eq!(after_unrelated_edit, initial);
        assert_eq!(proximity_after_unrelated_edit, proximity);
        assert!(
            read_source_element_parent_chain_run_paths_for_test().is_empty(),
            "an unrelated source edit must not invalidate a demand-shaped parent chain",
        );
    }

    #[test]
    fn source_element_computed_value_inherits_across_files_without_unrelated_reads() {
        use omena_cascade::{CascadeValue, ComputedCascadeValueStatusV0, ElementIdentityV0};

        let child_path = "/workspace/Child.tsx";
        let parent_path = "/workspace/Parent.tsx";
        let unrelated_path = "/workspace/Unrelated.tsx";
        let child_source = "export const Child = () => <span />;";
        let parent_source = "export const Parent = () => <main style={{ color: \"blue\", opacity: 0.5, fontFamily: theme.font }} />;";
        let unrelated_source = "export const Unrelated = () => <aside />;";
        let mut child_index = summarize_omena_query_source_syntax_index_for_source_language(
            child_path,
            child_source,
            None,
            Vec::new(),
            Vec::new(),
        );
        let parent_index = summarize_omena_query_source_syntax_index_for_source_language(
            parent_path,
            parent_source,
            None,
            Vec::new(),
            Vec::new(),
        );
        let unrelated_index = summarize_omena_query_source_syntax_index_for_source_language(
            unrelated_path,
            unrelated_source,
            None,
            Vec::new(),
            Vec::new(),
        );
        let child = child_index.source_elements[0].identity.clone();
        let parent = parent_index.source_elements[0].identity.clone();
        child_index
            .element_parent_edges
            .push(OmenaQuerySourceElementParentFactV0 {
                child: child.clone(),
                parent: parent.clone(),
            });
        let mut documents = vec![
            OmenaQuerySourceDocumentInputV0 {
                source_path: child_path.to_string(),
                source_source: child_source.to_string(),
                source_syntax_index: Some(child_index),
                has_unresolved_style_import: false,
            },
            OmenaQuerySourceDocumentInputV0 {
                source_path: parent_path.to_string(),
                source_source: parent_source.to_string(),
                source_syntax_index: Some(parent_index),
                has_unresolved_style_import: false,
            },
            OmenaQuerySourceDocumentInputV0 {
                source_path: unrelated_path.to_string(),
                source_source: unrelated_source.to_string(),
                source_syntax_index: Some(unrelated_index),
                has_unresolved_style_import: false,
            },
        ];
        let target = ElementIdentityV0 {
            source_path: child.source_path,
            byte_start: child.byte_span.start,
            byte_end: child.byte_span.end,
        };
        let mut host = OmenaQueryStyleMemoHostV0::new();

        let color =
            host.source_element_computed_value(documents.as_slice(), target.clone(), "color");
        assert_eq!(
            color.status,
            OmenaQueryElementComputedValueStatusV0::Resolved
        );
        assert_eq!(
            color
                .computed_value
                .as_ref()
                .map(|value| (&value.status, &value.value)),
            Some((
                &ComputedCascadeValueStatusV0::Inherited,
                &CascadeValue::Literal("blue".to_string()),
            ))
        );

        let opacity =
            host.source_element_computed_value(documents.as_slice(), target.clone(), "opacity");
        assert_eq!(
            opacity.computed_value.as_ref().map(|value| (
                &value.status,
                &value.value,
                value.inherited,
            )),
            Some((
                &ComputedCascadeValueStatusV0::Initial,
                &CascadeValue::Literal("1".to_string()),
                false,
            ))
        );

        let dynamic =
            host.source_element_computed_value(documents.as_slice(), target.clone(), "font-family");
        assert_eq!(
            dynamic.status,
            OmenaQueryElementComputedValueStatusV0::DynamicDeclaration
        );
        assert!(dynamic.computed_value.is_none());

        documents[2].source_source.push_str("\n// unrelated edit\n");
        reset_source_element_parent_chain_run_paths_for_test();
        reset_source_element_computed_value_compute_count_for_test();
        let color_after_unrelated_edit =
            host.source_element_computed_value(documents.as_slice(), target, "color");
        assert_eq!(color_after_unrelated_edit, color);
        assert_eq!(
            read_source_element_computed_value_compute_count_for_test(),
            0,
            "an unrelated source edit must not recompute inherited values",
        );
        assert!(
            read_source_element_parent_chain_run_paths_for_test().is_empty(),
            "an unrelated source edit must not invalidate inherited computed values",
        );
    }

    #[test]
    fn source_element_computed_value_rejects_jsx_inline_important_suffix() {
        use omena_cascade::ElementIdentityV0;

        let source_path = "/workspace/App.tsx";
        let source = r#"export const App = () => <main style={{ color: "blue !important" }} />;"#;
        let index = summarize_omena_query_source_syntax_index_for_source_language(
            source_path,
            source,
            None,
            Vec::new(),
            Vec::new(),
        );
        let element = index.source_elements[0].identity.clone();
        let documents = [OmenaQuerySourceDocumentInputV0 {
            source_path: source_path.to_string(),
            source_source: source.to_string(),
            source_syntax_index: Some(index),
            has_unresolved_style_import: false,
        }];
        let target = ElementIdentityV0 {
            source_path: element.source_path,
            byte_start: element.byte_span.start,
            byte_end: element.byte_span.end,
        };
        let mut host = OmenaQueryStyleMemoHostV0::new();

        let color = host.source_element_computed_value(documents.as_slice(), target, "color");

        assert_eq!(
            color.status,
            OmenaQueryElementComputedValueStatusV0::UnsupportedStaticValue
        );
        assert_eq!(color.computed_value, None);
    }

    #[test]
    fn source_element_computed_value_consumes_definite_standard_property_verdicts() {
        use omena_cascade::{ComputedCascadeValueStatusV0, ElementIdentityV0};

        let source_path = "/workspace/App.tsx";
        let source = r#"export const App = () => <main style={{ boxSizing: "inline-box" }} />;"#;
        let index = summarize_omena_query_source_syntax_index_for_source_language(
            source_path,
            source,
            None,
            Vec::new(),
            Vec::new(),
        );
        let element = index.source_elements[0].identity.clone();
        let documents = [OmenaQuerySourceDocumentInputV0 {
            source_path: source_path.to_string(),
            source_source: source.to_string(),
            source_syntax_index: Some(index),
            has_unresolved_style_import: false,
        }];
        let target = ElementIdentityV0 {
            source_path: element.source_path,
            byte_start: element.byte_span.start,
            byte_end: element.byte_span.end,
        };
        let mut host = OmenaQueryStyleMemoHostV0::new();

        let value = host.source_element_computed_value(documents.as_slice(), target, "box-sizing");

        assert_eq!(
            value
                .computed_value
                .as_ref()
                .map(|value| (&value.status, value.invalid_at_computed_value_time)),
            Some((
                &ComputedCascadeValueStatusV0::InvalidAtComputedValueTime,
                true,
            ))
        );
    }

    #[test]
    fn source_element_computed_value_revalidates_a_substituted_standard_value() {
        use omena_cascade::{ComputedCascadeValueStatusV0, ElementIdentityV0};

        for (custom_value, expected_status) in [
            ("red", ComputedCascadeValueStatusV0::Resolved),
            ("12px", ComputedCascadeValueStatusV0::Indeterminate),
        ] {
            let source_path = format!("/workspace/{custom_value}.tsx");
            let source = format!(
                r#"export const App = () => <main style={{{{ "--tone": "{custom_value}", color: "var(--tone)" }}}} />;"#
            );
            let index = summarize_omena_query_source_syntax_index_for_source_language(
                source_path.as_str(),
                source.as_str(),
                None,
                Vec::new(),
                Vec::new(),
            );
            let element = index.source_elements[0].identity.clone();
            let documents = [OmenaQuerySourceDocumentInputV0 {
                source_path: source_path.clone(),
                source_source: source,
                source_syntax_index: Some(index),
                has_unresolved_style_import: false,
            }];
            let target = ElementIdentityV0 {
                source_path: element.source_path,
                byte_start: element.byte_span.start,
                byte_end: element.byte_span.end,
            };
            let mut host = OmenaQueryStyleMemoHostV0::new();

            let color = host.source_element_computed_value(documents.as_slice(), target, "color");
            assert!(
                color.computed_value.is_some(),
                "the static declaration must reach computed-value resolution"
            );
            let Some(computed) = color.computed_value else {
                continue;
            };

            assert_eq!(computed.status, expected_status, "{custom_value}");
            assert!(
                computed
                    .derivation_steps
                    .contains(&"standardPropertySyntaxDeferredByVarReference")
            );
            if custom_value == "12px" {
                assert!(!computed.invalid_at_computed_value_time);
                assert!(
                    computed
                        .derivation_steps
                        .contains(&"postSubstitutionStandardPropertySyntaxIndeterminate")
                );
            } else {
                assert!(
                    computed
                        .derivation_steps
                        .contains(&"postSubstitutionStandardPropertySyntaxMatched")
                );
            }
        }
    }

    #[test]
    fn source_element_computed_value_definitely_rejects_substituted_literal_only_grammar() {
        use omena_cascade::{ComputedCascadeValueStatusV0, ElementIdentityV0};

        let source_path = "/workspace/box-sizing.tsx";
        let source = r#"export const App = () => <main style={{ "--mode": "inline-box", boxSizing: "var(--mode)" }} />;"#;
        let index = summarize_omena_query_source_syntax_index_for_source_language(
            source_path,
            source,
            None,
            Vec::new(),
            Vec::new(),
        );
        let element = index.source_elements[0].identity.clone();
        let documents = [OmenaQuerySourceDocumentInputV0 {
            source_path: source_path.to_string(),
            source_source: source.to_string(),
            source_syntax_index: Some(index),
            has_unresolved_style_import: false,
        }];
        let target = ElementIdentityV0 {
            source_path: element.source_path,
            byte_start: element.byte_span.start,
            byte_end: element.byte_span.end,
        };
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let value = host.source_element_computed_value(documents.as_slice(), target, "box-sizing");
        let computed = value.computed_value.as_ref();
        assert!(
            computed.is_some(),
            "the static declaration must reach computed-value resolution"
        );
        let Some(computed) = computed else {
            return;
        };

        assert_eq!(
            computed.status,
            ComputedCascadeValueStatusV0::InvalidAtComputedValueTime
        );
        assert!(computed.invalid_at_computed_value_time);
        assert!(
            computed
                .derivation_steps
                .contains(&"postSubstitutionStandardPropertySyntaxUnmatched")
        );
    }

    #[test]
    fn inherited_custom_property_values_do_not_rebind_to_child_overrides() {
        use omena_cascade::{CascadeValue, ComputedCascadeValueStatusV0, ElementIdentityV0};

        let child_path = "/workspace/Child.tsx";
        let parent_path = "/workspace/Parent.tsx";
        let child_source = r#"export const Child = () => <span style={{ "--base": "blue", color: "var(--tone)" }} />;"#;
        let parent_source = r#"export const Parent = () => <main style={{ "--tone": "var(--base)", "--base": "red" }} />;"#;
        let mut child_index = summarize_omena_query_source_syntax_index_for_source_language(
            child_path,
            child_source,
            None,
            Vec::new(),
            Vec::new(),
        );
        let parent_index = summarize_omena_query_source_syntax_index_for_source_language(
            parent_path,
            parent_source,
            None,
            Vec::new(),
            Vec::new(),
        );
        let child = child_index.source_elements[0].identity.clone();
        let parent = parent_index.source_elements[0].identity.clone();
        child_index
            .element_parent_edges
            .push(OmenaQuerySourceElementParentFactV0 {
                child: child.clone(),
                parent,
            });
        let documents = [
            OmenaQuerySourceDocumentInputV0 {
                source_path: child_path.to_string(),
                source_source: child_source.to_string(),
                source_syntax_index: Some(child_index),
                has_unresolved_style_import: false,
            },
            OmenaQuerySourceDocumentInputV0 {
                source_path: parent_path.to_string(),
                source_source: parent_source.to_string(),
                source_syntax_index: Some(parent_index),
                has_unresolved_style_import: false,
            },
        ];
        let target = ElementIdentityV0 {
            source_path: child.source_path,
            byte_start: child.byte_span.start,
            byte_end: child.byte_span.end,
        };
        let mut host = OmenaQueryStyleMemoHostV0::new();

        let color = host.source_element_computed_value(documents.as_slice(), target, "color");

        assert_eq!(
            color.computed_value.as_ref().map(|value| (
                &value.status,
                &value.value,
                value.invalid_at_computed_value_time,
            )),
            Some((
                &ComputedCascadeValueStatusV0::Resolved,
                &CascadeValue::Literal("red".to_string()),
                false,
            )),
            "the parent's computed --tone value must not rebind to the child's --base declaration",
        );
    }

    #[test]
    fn unrelated_dynamic_custom_properties_do_not_block_static_standard_values() {
        use omena_cascade::{CascadeValue, ComputedCascadeValueStatusV0, ElementIdentityV0};

        for (color_value, expected_status, expected_value) in [
            (
                "red",
                ComputedCascadeValueStatusV0::Resolved,
                CascadeValue::Literal("red".to_string()),
            ),
            (
                "var(--dynamic-tone)",
                ComputedCascadeValueStatusV0::Indeterminate,
                CascadeValue::Indeterminate,
            ),
        ] {
            let source_path = format!("/workspace/{expected_status:?}.tsx");
            let source = format!(
                r#"export const App = () => <main style={{{{ "--dynamic-tone": theme.color, color: "{color_value}" }}}} />;"#
            );
            let index = summarize_omena_query_source_syntax_index_for_source_language(
                source_path.as_str(),
                source.as_str(),
                None,
                Vec::new(),
                Vec::new(),
            );
            let element = index.source_elements[0].identity.clone();
            let documents = [OmenaQuerySourceDocumentInputV0 {
                source_path: source_path.clone(),
                source_source: source,
                source_syntax_index: Some(index),
                has_unresolved_style_import: false,
            }];
            let target = ElementIdentityV0 {
                source_path: element.source_path,
                byte_start: element.byte_span.start,
                byte_end: element.byte_span.end,
            };
            let mut host = OmenaQueryStyleMemoHostV0::new();

            let color = host.source_element_computed_value(documents.as_slice(), target, "color");

            assert_eq!(
                color
                    .computed_value
                    .as_ref()
                    .map(|value| (&value.status, &value.value)),
                Some((&expected_status, &expected_value)),
                "{color_value}",
            );
        }
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

    #[test]
    fn source_workspace_projection_records_length_changing_selector_stable_edit_limitation() {
        let styles = source_workspace_projection_style_corpus();
        let mut documents = vec![source_workspace_projection_document(
            "/workspace/src/App.tsx",
            "card",
            "one",
        )];
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let workspace = host.sync_workspace(
            styles.as_slice(),
            documents.as_slice(),
            &[],
            &[],
            &resolution_inputs,
        );
        exercise_source_workspace_projection_consumers(&host.db, workspace);

        documents[0] = source_workspace_projection_document(
            "/workspace/src/App.tsx",
            "card",
            "expanded-body-value",
        );
        let edited_workspace = host.sync_workspace(
            styles.as_slice(),
            documents.as_slice(),
            &[],
            &[],
            &resolution_inputs,
        );
        source_workspace_projection_probe::reset();
        source_workspace_query_probe::reset();
        exercise_source_workspace_projection_consumers(&host.db, edited_workspace);

        let projection_recompute_set = source_workspace_projection_probe::read();
        let consumer_recompute_counts = source_workspace_query_probe::read();
        eprintln!(
            "projectionRecomputeSet={projection_recompute_set:?} consumerRecomputeCounts={consumer_recompute_counts:?}"
        );
        assert_eq!(
            projection_recompute_set,
            set_of(["/workspace/src/App.tsx"]),
            "the length-changing edited file projection must execute"
        );
        assert_eq!(
            consumer_recompute_counts,
            (1, 1),
            "the current zero-recompute guarantee is limited to byte-length-preserving edits"
        );
    }

    #[test]
    fn package_manifest_projection_revalidates_only_the_changed_entity() {
        let corpus = parallel_probe_corpus();
        let manifests = vec![
            OmenaQueryStylePackageManifestV0 {
                package_json_path: "/workspace/packages/alpha/package.json".to_string(),
                package_json_source: r#"{"name":"alpha","style":"./alpha.css"}"#.to_string(),
            },
            OmenaQueryStylePackageManifestV0 {
                package_json_path: "/workspace/packages/beta/package.json".to_string(),
                package_json_source: r#"{"name":"beta","style":"./beta.css"}"#.to_string(),
            },
        ];
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let workspace = host.sync_workspace(
            corpus.as_slice(),
            &[],
            manifests.as_slice(),
            &[],
            &resolution_inputs,
        );
        let _ = memo_css_modules_cross_file_resolution_from_module_interfaces(&host.db, workspace);

        let mut edited_manifests = manifests;
        edited_manifests[1].package_json_source =
            r#"{"name":"beta","style":"./beta-updated.css"}"#.to_string();
        let workspace = host.sync_workspace(
            corpus.as_slice(),
            &[],
            edited_manifests.as_slice(),
            &[],
            &resolution_inputs,
        );
        package_manifest_projection_probe::reset();
        let _ = memo_css_modules_cross_file_resolution_from_module_interfaces(&host.db, workspace);

        assert_eq!(
            package_manifest_projection_probe::read(),
            set_of(["/workspace/packages/beta/package.json"]),
            "a same-field edit must not revalidate unchanged manifest projections"
        );
        let memo_graph = build_committed_style_semantic_graph(
            &host.db,
            workspace,
            &[],
            edited_manifests.as_slice(),
            &[],
            &resolution_inputs,
        );
        let straight_line_graph = build_committed_style_semantic_graph_monolith(
            &host.db,
            workspace,
            &[],
            edited_manifests.as_slice(),
            &[],
            &resolution_inputs,
        );
        assert_eq!(
            memo_graph, straight_line_graph,
            "entity-granular and straight-line graph bytes must remain identical"
        );
    }

    #[test]
    fn cache_freshness_field_does_not_revalidate_semantic_resolution() {
        let corpus = parallel_probe_corpus();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let workspace = host.sync_workspace(corpus.as_slice(), &[], &[], &[], &resolution_inputs);
        let baseline =
            memo_css_modules_cross_file_resolution_from_module_interfaces(&host.db, workspace);

        let changed_resolution_inputs = OmenaQueryStyleResolutionInputsV0 {
            external_sif_cache_fingerprint: Some("fresh-cache-generation".to_string()),
            ..resolution_inputs
        };
        let workspace =
            host.sync_workspace(corpus.as_slice(), &[], &[], &[], &changed_resolution_inputs);
        reset_css_modules_import_edge_resolution_probe_for_test();
        let after =
            memo_css_modules_cross_file_resolution_from_module_interfaces(&host.db, workspace);
        let recomputed_origins = read_css_modules_import_edge_resolution_probe_for_test();
        eprintln!("cacheFreshnessPerOriginRecomputeSet={recomputed_origins:?}");

        assert_eq!(
            recomputed_origins,
            BTreeSet::new(),
            "a cache-only field outside the semantic read set must not re-run either per-origin resolution query"
        );
        assert_eq!(after, baseline, "the narrowed read set must preserve bytes");
    }

    #[test]
    fn public_workspace_constructor_retains_legacy_input_semantics() {
        let db = OmenaQueryStyleMemoDatabaseV0::new();
        let corpus = parallel_probe_corpus();
        let files = corpus
            .iter()
            .map(|source| {
                OmenaQueryStyleFileInputV0::new(
                    &db,
                    source.style_path.clone(),
                    source.style_source.clone(),
                )
            })
            .collect::<Vec<_>>();
        let manifests = vec![OmenaQueryStylePackageManifestV0 {
            package_json_path: "/workspace/node_modules/theme/package.json".to_string(),
            package_json_source:
                r#"{"name":"theme","exports":{"./tokens":{"style":"./tokens.css"}}}"#.to_string(),
        }];
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0 {
            package_manifests: manifests.clone(),
            external_sif_cache_fingerprint: Some("cache-only".to_string()),
            ..OmenaQueryStyleResolutionInputsV0::default()
        };
        let workspace = OmenaQueryStyleWorkspaceInputV0::new(
            &db,
            files,
            Vec::new(),
            Vec::new(),
            manifests.clone(),
            Vec::new(),
            resolution_inputs.clone(),
        );

        let memo_graph = build_committed_style_semantic_graph(
            &db,
            workspace,
            &[],
            manifests.as_slice(),
            &[],
            &resolution_inputs,
        );
        let straight_line_graph = build_committed_style_semantic_graph_monolith(
            &db,
            workspace,
            &[],
            manifests.as_slice(),
            &[],
            &resolution_inputs,
        );
        assert_eq!(
            memo_graph, straight_line_graph,
            "defaulted granular fields must fall back to the constructor's legacy values"
        );
    }
}
