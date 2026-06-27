//! RFC 0009 Pillar B (rfcs#65), stage 1: the salsa-backed memoized
//! style-diagnostics query layer.
//!
//! The workspace diagnostics entry point is wrapped in a salsa tracked query
//! keyed by `(workspace revision, target file)`. Per-file texts are salsa
//! inputs, so an unchanged corpus revalidates instead of recomputing, and a
//! single-file edit re-runs only queries whose inputs actually changed —
//! under the RFC 0009 invariant that a memoized result is returned only when
//! it is byte-identical to a from-scratch evaluation. That invariant is
//! enforced by `omena-diff-test`'s cache-equivalence oracle, which diffs this
//! evaluator against the straight-line path over warm rounds and edit
//! sequences; this module must never merge changes that gate does not cover.
//!
//! The host owns the shared Omena salsa database on the LSP loop thread: all
//! `set_*` happen there (the salsa pending-write contract), and fixed-revision
//! `StorageHandle` read views are rebuilt on worker threads for parallel
//! diagnostics.

use super::*;
pub type OmenaQueryStyleMemoDatabaseV0 = OmenaSalsaDatabaseV0;
use salsa::Setter;
use std::collections::{BTreeMap, BTreeSet};

#[cfg(test)]
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

/// RFC 0009 Pillar F (rfcs#68): one loop-side sync, many worker-side read
/// views. Produced by
/// [`OmenaQueryStyleMemoHostV0::sync_workspace_for_parallel_resolve`] AFTER
/// every `set_*` for the wave has been applied; the embedded `handle` pins
/// that revision and is `Send`, so a parallel wave rebuilds per-worker views
/// via [`OmenaQueryStyleMemoDatabaseV0::from_handle`]. Every handle clone and
/// every rebuilt view MUST be dropped before the owning thread issues its
/// next `set_*` (the salsa pending-write contract) — a leaked view blocks
/// that write forever. The LSP wave joins inside one loop turn precisely to
/// guarantee the drop, and `omena-diff-test`'s
/// `parallelSalsaViewsVsFromScratchEquivalence` gate drives this bundle
/// through edit phases that would deadlock on a leak.
pub struct OmenaQueryStyleParallelResolveSyncV0 {
    /// Fixed-revision database handle: clone per worker, drop with the wave.
    pub handle: salsa::StorageHandle<OmenaQueryStyleMemoDatabaseV0>,
    /// The synced workspace input entity (`Copy` salsa id).
    pub workspace: OmenaQueryStyleWorkspaceInputV0,
    /// `(style_path, file input entity)` for every corpus member, in corpus
    /// order, so callers map targets onto input ids without re-entering the
    /// host.
    pub files: Vec<(String, OmenaQueryStyleFileInputV0)>,
}

pub struct OmenaQueryStyleRevisionSelectorV0 {
    revision: IncrementalRevisionV0,
    db: OmenaQueryStyleMemoDatabaseV0,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    files_by_path: BTreeMap<String, OmenaQueryStyleFileInputV0>,
}

impl std::fmt::Debug for OmenaQueryStyleRevisionSelectorV0 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OmenaQueryStyleRevisionSelectorV0")
            .field("revision", &self.revision)
            .field("file_count", &self.files_by_path.len())
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
        let target = self.files_by_path.get(target_style_path).copied()?;
        memo_workspace_style_diagnostics(&self.db, self.workspace, target)
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

/// RFC 0009 Pillar F (rfcs#68): the tracked workspace diagnostics query,
/// callable from a fixed-revision read view rebuilt via `from_handle` on a
/// worker thread. Byte-identity with the host entry point holds by
/// construction (same tracked function, same revision); the parallel arm of
/// omena-diff-test's cache-equivalence oracle stands as the merge gate.
pub fn resolve_memo_workspace_style_diagnostics_from_view(
    db: &OmenaQueryStyleMemoDatabaseV0,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    target: OmenaQueryStyleFileInputV0,
) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
    memo_workspace_style_diagnostics(db, workspace, target)
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
    #[cfg(test)]
    style_fact_entry_probe::record(file.style_path(db));
    collect_omena_query_style_fact_entry(file.style_path(db), file.style_source(db))
}

/// The memoized workspace diagnostics query. Mirrors the LSP's call shape
/// exactly: `classname_transform` is `None` and the external mode is derived
/// from SIF presence, byte-identical to `resolve_style_diagnostics_for_uri`.
/// Reads the workspace-keyed substrate query so the target-independent
/// resolution is shared across targets.
#[salsa::tracked(returns(clone))]
fn memo_workspace_style_diagnostics(
    db: &dyn salsa::Database,
    workspace: OmenaQueryStyleWorkspaceInputV0,
    target: OmenaQueryStyleFileInputV0,
) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
    let corpus = workspace
        .files(db)
        .iter()
        .map(|file| OmenaQueryStyleSourceInputV0 {
            style_path: file.style_path(db).clone(),
            style_source: file.style_source(db).clone(),
        })
        .collect::<Vec<_>>();
    let external_sifs = workspace.external_sifs(db);
    let substrate = memo_workspace_diagnostics_substrate(db, workspace);
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs_and_suppression_mode_with_substrate(
        target.style_path(db).as_str(),
        corpus.as_slice(),
        workspace.source_documents(db).as_slice(),
        workspace.package_manifests(db).as_slice(),
        None,
        OmenaQueryExternalModuleModeV0::Auto,
        external_sifs.as_slice(),
        workspace.resolution_inputs(db),
        OmenaQueryDiagnosticSuppressionModeV0::Apply,
        substrate,
    )
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
    workspace: Option<OmenaQueryStyleWorkspaceInputV0>,
    committed_revision: IncrementalRevisionV0,
}

impl std::fmt::Debug for OmenaQueryStyleMemoHostV0 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OmenaQueryStyleMemoHostV0")
            .field("known_file_count", &self.files_by_path.len())
            .field("workspace_initialized", &self.workspace.is_some())
            .field("committed_revision", &self.committed_revision)
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
            workspace: None,
            committed_revision: IncrementalRevisionV0 { value: 0 },
        }
    }

    pub fn committed_revision(&self) -> IncrementalRevisionV0 {
        self.committed_revision
    }

    /// Sync the in-hand inputs into the database (diff-only) and run the
    /// memoized workspace diagnostics for `target_style_path`. Returns `None`
    /// exactly when the straight-line entry point would (target not in the
    /// corpus / no hover candidates).
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
        let workspace = self.sync_workspace(
            style_sources,
            source_documents,
            package_manifests,
            external_sifs,
            resolution_inputs,
        );
        let target = self.files_by_path.get(target_style_path).copied();
        // The straight-line path returns None for a target outside the corpus;
        // mirror that without touching the database.
        let target = target.filter(|_| {
            style_sources
                .iter()
                .any(|source| source.style_path == target_style_path)
        })?;
        memo_workspace_style_diagnostics(&self.db, workspace, target)
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
        // All `set_*` happen here, on the calling (owner) thread, BEFORE the
        // handle is created — the pending-write contract for the wave.
        let workspace = self.sync_workspace(
            style_sources,
            source_documents,
            package_manifests,
            external_sifs,
            resolution_inputs,
        );
        let files = style_sources
            .iter()
            .filter_map(|source| {
                self.files_by_path
                    .get(source.style_path.as_str())
                    .map(|file| (source.style_path.clone(), *file))
            })
            .collect::<Vec<_>>();
        Some(OmenaQueryStyleParallelResolveSyncV0 {
            handle: self.db.handle(),
            workspace,
            files,
        })
    }

    fn commit_workspace_transaction(
        &mut self,
        transaction: OmenaQueryStyleWorkspaceTransactionV0,
    ) -> Result<
        OmenaQueryStyleWorkspaceTransactionCommitV0,
        OmenaQueryStyleWorkspaceTransactionErrorV0,
    > {
        validate_workspace_transaction(&transaction)?;
        let changed_style_paths = self.changed_style_paths_for_transaction(&transaction);
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
        let selector = build_revision_selector(
            self.committed_revision,
            transaction.style_sources.as_slice(),
            transaction.source_documents.as_slice(),
            transaction.package_manifests.as_slice(),
            transaction.external_sifs.as_slice(),
            &transaction.resolution_inputs,
        );
        Ok(OmenaQueryStyleWorkspaceTransactionCommitV0 {
            revision: self.committed_revision,
            workspace,
            files,
            changed_style_paths,
            selector,
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
        workspace: _,
        committed_revision: _,
    } = host;
    OmenaQueryStyleRevisionSelectorV0 {
        revision,
        db,
        workspace,
        files_by_path,
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

        style_fact_entry_probe::reset();
        {
            let _ = memo_workspace_diagnostics_substrate(&host.db, commit.workspace);
        }
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
        let edited_commit = transaction
            .commit_revision(&mut host)
            .map_err(|_| "registered edit transaction must commit")?;
        assert_eq!(edited_commit.revision, IncrementalRevisionV0 { value: 2 });
        assert_eq!(
            edited_commit.changed_style_paths,
            set_of(["/workspace/src/App.module.scss"]),
            "editing one registered style file must report only that file as the transaction delta",
        );

        style_fact_entry_probe::reset();
        {
            let _ = memo_workspace_diagnostics_substrate(&host.db, edited_commit.workspace);
        }
        assert_eq!(
            style_fact_entry_probe::read(),
            set_of(["/workspace/src/App.module.scss"]),
            "transaction commit must preserve the per-file salsa firewall",
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
        let view_results = std::thread::scope(|scope| {
            let workers = sync
                .files
                .iter()
                .map(|(style_path, file)| {
                    let handle = sync.handle.clone();
                    let file = *file;
                    let style_path = style_path.clone();
                    scope.spawn(move || {
                        let db = OmenaQueryStyleMemoDatabaseV0::from_handle(handle);
                        let summary = resolve_memo_workspace_style_diagnostics_from_view(
                            &db, workspace, file,
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
    fn workspace_substrate_recompute_set_is_size_invariant() {
        assert_changed_file_recompute_set(parallel_probe_corpus());
        assert_changed_file_recompute_set(doubled_parallel_probe_corpus());
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

    /// RFC 0009 Pillar F handle-scope regression: once every view and handle
    /// clone from a parallel wave is dropped, the next `set_*` MUST proceed.
    /// A leaked view would block the write forever, so the post-wave edit
    /// resolve runs on a watchdog thread and the test fails (instead of
    /// hanging) when it does not complete.
    #[test]
    fn post_wave_edit_writes_proceed_after_views_drop() -> Result<(), &'static str> {
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
        std::thread::scope(|scope| {
            for (_, file) in sync.files.iter() {
                let handle = sync.handle.clone();
                let file = *file;
                scope.spawn(move || {
                    let db = OmenaQueryStyleMemoDatabaseV0::from_handle(handle);
                    let _ =
                        resolve_memo_workspace_style_diagnostics_from_view(&db, workspace, file);
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
            .map_err(|_| {
                "post-wave set_* deadlocked: a parallel view or handle clone leaked past the wave"
            })?;
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
