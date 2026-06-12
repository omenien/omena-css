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
//! The host owns the database on the LSP loop thread: all `set_*` happen
//! there (the salsa pending-write contract), so later pillars (A/F) can hand
//! `StorageHandle`-pinned read views to workers without re-architecting this
//! layer. `salsa::DatabaseImpl` cannot be rebuilt from a `StorageHandle`
//! (no public from-storage constructor), which is why the database is a
//! local `#[salsa::db]` struct — pinned by the rfcs#64 spike.

use super::*;
use salsa::Setter;
use std::collections::BTreeMap;

/// The long-lived analysis database for the memoized style-diagnostics layer.
#[salsa::db]
#[derive(Clone, Default)]
pub struct OmenaQueryStyleMemoDatabaseV0 {
    storage: salsa::Storage<Self>,
}

#[salsa::db]
impl salsa::Database for OmenaQueryStyleMemoDatabaseV0 {}

impl OmenaQueryStyleMemoDatabaseV0 {
    pub fn new() -> Self {
        Self::default()
    }

    /// A `Send` handle for fixed-revision read views (Pillars A/F): create it
    /// on the owner thread, move it across, rebuild a view via `from_handle`.
    pub fn handle(&self) -> salsa::StorageHandle<Self> {
        self.storage.clone().into_zalsa_handle()
    }

    pub fn from_handle(handle: salsa::StorageHandle<Self>) -> Self {
        Self {
            storage: handle.into_storage(),
        }
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

/// The memoized workspace diagnostics query. Mirrors the LSP's call shape
/// exactly: `classname_transform` is `None` and the external mode is derived
/// from SIF presence, byte-identical to `resolve_style_diagnostics_for_uri`.
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
    let external_mode = if external_sifs.is_empty() {
        OmenaQueryExternalModuleModeV0::Ignored
    } else {
        OmenaQueryExternalModuleModeV0::Sif
    };
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs(
        target.style_path(db).as_str(),
        corpus.as_slice(),
        workspace.source_documents(db).as_slice(),
        workspace.package_manifests(db).as_slice(),
        None,
        external_mode,
        external_sifs.as_slice(),
        workspace.resolution_inputs(db),
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
}

impl std::fmt::Debug for OmenaQueryStyleMemoHostV0 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OmenaQueryStyleMemoHostV0")
            .field("known_file_count", &self.files_by_path.len())
            .field("workspace_initialized", &self.workspace.is_some())
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
        }
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
            let external_mode = if external_sifs.is_empty() {
                OmenaQueryExternalModuleModeV0::Ignored
            } else {
                OmenaQueryExternalModuleModeV0::Sif
            };
            return summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs(
                target_style_path,
                style_sources,
                source_documents,
                package_manifests,
                None,
                external_mode,
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
