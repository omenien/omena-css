use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::{Arc, RwLock},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    IncrementalRevisionV0, OmenaError, OmenaErrorClassV0, OmenaErrorContextV0,
    OmenaErrorRecoverabilityV0, OmenaErrorSeverityV0, OmenaQueryExternalSifInputV0,
    OmenaQueryExternalSifTrustV1, OmenaQuerySourceDocumentInputV0,
    OmenaQueryStylePackageManifestV0, OmenaQueryStyleResolutionInputsV0,
    OmenaQueryStyleSourceInputV0, OmenaWorkspaceInputCommitmentV0, OmenaWorkspaceSnapshotBindingV0,
    OmenaWorkspaceSnapshotIdV0,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct OmenaWorkspaceSnapshotSettingsV0 {
    pub diagnostic_severity: u8,
    pub deep_analysis: bool,
    pub definition: bool,
    pub hover: bool,
    pub completion: bool,
    pub references: bool,
    pub rename: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_content_digest: Option<String>,
}

impl Default for OmenaWorkspaceSnapshotSettingsV0 {
    fn default() -> Self {
        Self {
            diagnostic_severity: 2,
            deep_analysis: false,
            definition: true,
            hover: true,
            completion: true,
            references: true,
            rename: true,
            config_content_digest: None,
        }
    }
}

/// Project already-admitted contextual edges and full target facts onto this
/// root's document corpus. Export performs no filesystem/resolver confirmation.
pub fn select_omena_workspace_snapshot_external_sifs_v0(
    styles: &[OmenaQueryStyleSourceInputV0],
    _resolution: &OmenaQueryStyleResolutionInputsV0,
    external_sifs: &[OmenaQueryExternalSifInputV0],
    _trust_records: &BTreeMap<String, OmenaQueryExternalSifTrustV1>,
    resolution_edges: &[crate::OmenaQueryExternalSifResolutionEdgeV0],
) -> Result<
    (
        Vec<OmenaQueryExternalSifInputV0>,
        BTreeMap<String, OmenaQueryExternalSifTrustV1>,
        Vec<crate::OmenaQueryExternalSifResolutionEdgeV0>,
    ),
    OmenaError,
> {
    use crate::OmenaQueryExternalSifImportOriginV0 as Origin;
    let mut pending = VecDeque::new();
    for style in styles {
        if let Some(summary) =
            crate::summarize_omena_query_sass_module_sources(&style.style_path, &style.style_source)
        {
            for source in summary
                .module_use_edges
                .into_iter()
                .map(|edge| edge.source)
                .chain(summary.module_forward_sources)
            {
                pending.push_back((
                    Origin::Document {
                        style_path: style.style_path.clone(),
                    },
                    source,
                ));
            }
        }
    }
    let mut selected = BTreeMap::new();
    let mut visited = BTreeSet::new();
    let mut selected_trust = BTreeMap::new();
    let mut selected_edges = BTreeSet::new();
    while let Some((origin, specifier)) = pending.pop_front() {
        let mut matches = resolution_edges
            .iter()
            .filter(|edge| edge.importer == origin && edge.specifier == specifier);
        let Some(edge) = matches.next() else { continue };
        if matches.any(|other| other != edge) {
            return Err(snapshot_error(
                "admitted import context resolves to conflicting targets",
                "workspace.snapshot-sif-admission",
            ));
        }
        let target = external_sifs
            .iter()
            .find(|input| {
                input.sif.canonical_url == edge.sif_canonical_url
                    && omena_sif::compute_omena_sif_artifact_hash_v1(&input.sif)
                        .is_ok_and(|hash| hash.as_str() == edge.sif_artifact_hash)
            })
            .ok_or_else(|| {
                snapshot_error(
                    "admitted import edge has no matching full SIF artifact",
                    "workspace.snapshot-sif-admission",
                )
            })?;
        let direct = matches!(origin, Origin::Document { .. });
        let alias = if direct || (!specifier.starts_with('.') && !specifier.starts_with("file://"))
        {
            specifier
        } else {
            target.sif.canonical_url.clone()
        };
        selected.insert(
            (
                alias.clone(),
                target.sif.canonical_url.clone(),
                edge.sif_artifact_hash.clone(),
            ),
            OmenaQueryExternalSifInputV0 {
                admitted_resolution_edges: Vec::new(),
                canonical_url: alias,
                sif: target.sif.clone(),
            },
        );
        selected_edges.insert(edge.clone());
        if edge.trust.canonical_url != target.sif.canonical_url {
            return Err(snapshot_error(
                "admitted verdict does not belong to the selected SIF artifact",
                "workspace.snapshot-sif-admission",
            ));
        }
        selected_trust.insert(contextual_trust_record_key(edge)?, edge.trust.clone());
        let child_origin = Origin::Sif {
            initiating_document: origin.initiating_document().map(ToOwned::to_owned),
            resolved_style_url: edge.resolved_style_url.clone(),
            artifact_hash: edge.sif_artifact_hash.clone(),
        };
        if visited.insert(child_origin.clone()) {
            for dependency in target
                .sif
                .exports
                .forwards
                .iter()
                .map(|entry| &entry.canonical_url)
                .chain(
                    target
                        .sif
                        .dependencies
                        .iter()
                        .map(|entry| &entry.canonical_url),
                )
            {
                pending.push_back((child_origin.clone(), dependency.clone()));
            }
        }
    }
    let mut targets_by_alias: BTreeMap<String, BTreeSet<(String, String)>> = BTreeMap::new();
    for (alias, target, artifact_hash) in selected.keys() {
        targets_by_alias
            .entry(alias.clone())
            .or_default()
            .insert((target.clone(), artifact_hash.clone()));
    }
    let mut unambiguous = BTreeMap::new();
    for ((alias, target, artifact_hash), mut input) in selected {
        if targets_by_alias[&alias].len() > 1 {
            input.canonical_url = target.clone();
        }
        input.admitted_resolution_edges = selected_edges
            .iter()
            .filter(|edge| {
                edge.sif_canonical_url == input.sif.canonical_url
                    && omena_sif::compute_omena_sif_artifact_hash_v1(&input.sif)
                        .is_ok_and(|hash| hash.as_str() == edge.sif_artifact_hash)
            })
            .cloned()
            .collect();
        unambiguous.insert((input.canonical_url.clone(), target, artifact_hash), input);
    }
    Ok((
        unambiguous.into_values().collect(),
        selected_trust,
        selected_edges.into_iter().collect(),
    ))
}

// This key serializes already-admitted provenance; it neither issues identity
// nor combines verdicts. Equal canonical artifacts in different contexts remain
// distinct entries in the existing commitment frame.
fn contextual_trust_record_key(
    edge: &crate::OmenaQueryExternalSifResolutionEdgeV0,
) -> Result<String, OmenaError> {
    serde_json::to_string(&(
        &edge.importer,
        &edge.specifier,
        &edge.resolved_style_url,
        &edge.sif_canonical_url,
        &edge.sif_artifact_hash,
    ))
    .map_err(|_| {
        snapshot_error(
            "admitted trust context cannot be encoded",
            "workspace.snapshot-sif-admission",
        )
    })
}

/// Inputs are borrowed from their existing owner. Manifest and resolver list
/// order is preserved because repeated entries can have first-match semantics.
#[derive(Debug, Clone, Copy)]
pub struct OmenaWorkspaceSnapshotInputsV0<'a> {
    pub workspace_root: &'a str,
    pub style_sources: &'a [OmenaQueryStyleSourceInputV0],
    pub source_documents: &'a [OmenaQuerySourceDocumentInputV0],
    pub source_language_ids: &'a BTreeMap<String, String>,
    pub source_provider_inputs: &'a BTreeMap<String, crate::OmenaWorkspaceSourceProviderInputsV0>,
    pub package_manifests: &'a [OmenaQueryStylePackageManifestV0],
    pub external_sifs: &'a [OmenaQueryExternalSifInputV0],
    pub external_sif_resolution_edges: &'a [crate::OmenaQueryExternalSifResolutionEdgeV0],
    pub external_sif_trust_records: &'a BTreeMap<String, OmenaQueryExternalSifTrustV1>,
    pub resolution_inputs: &'a OmenaQueryStyleResolutionInputsV0,
    pub settings: &'a OmenaWorkspaceSnapshotSettingsV0,
    pub source_corpus_complete: bool,
}

impl<'a> OmenaWorkspaceSnapshotInputsV0<'a> {
    pub fn input_commitment(&self) -> Result<OmenaWorkspaceInputCommitmentV0, OmenaError> {
        if self.workspace_root.trim().is_empty() {
            return Err(snapshot_error(
                "workspace root must not be empty",
                "workspace.empty-root",
            ));
        }
        let mut attached = BTreeSet::new();
        for input in self.external_sifs {
            let hash = omena_sif::compute_omena_sif_artifact_hash_v1(&input.sif).map_err(|_| {
                snapshot_error(
                    "SIF artifact integrity cannot be computed",
                    "workspace.snapshot-sif-admission",
                )
            })?;
            for edge in &input.admitted_resolution_edges {
                if edge.sif_canonical_url != input.sif.canonical_url
                    || edge.sif_artifact_hash != hash.as_str()
                    || edge.trust.canonical_url != input.sif.canonical_url
                {
                    return Err(snapshot_error(
                        "consumer context does not match its admitted SIF artifact",
                        "workspace.snapshot-sif-admission",
                    ));
                }
                attached.insert(edge);
            }
        }
        if attached != self.external_sif_resolution_edges.iter().collect() {
            return Err(snapshot_error(
                "consumer context differs from the committed admission edges",
                "workspace.snapshot-sif-admission",
            ));
        }
        if !attached.is_empty() {
            let mut contexts = BTreeMap::new();
            for &edge in &attached {
                if contexts
                    .insert((&edge.importer, edge.specifier.as_str()), edge)
                    .is_some_and(|previous| previous != edge)
                {
                    return Err(snapshot_error(
                        "one admitted import context contains conflicting targets or verdicts",
                        "workspace.snapshot-sif-admission",
                    ));
                }
            }
            let contextual_trust = attached
                .iter()
                .map(|edge| Ok((contextual_trust_record_key(edge)?, edge.trust.clone())))
                .collect::<Result<BTreeMap<_, _>, OmenaError>>()?;
            if &contextual_trust != self.external_sif_trust_records {
                return Err(snapshot_error(
                    "selected trust records differ from the actual contextual admission verdicts",
                    "workspace.snapshot-sif-admission",
                ));
            }
        }
        let mut hasher = Sha256::new();
        hasher.update(b"omena.workspace-snapshot-inputs.v0\0");
        append_input(&mut hasher, "workspaceRoot", &self.workspace_root)?;
        append_input(&mut hasher, "styleSources", &self.style_sources)?;
        append_input(&mut hasher, "sourceDocuments", &self.source_documents)?;
        append_input(&mut hasher, "sourceLanguageIds", self.source_language_ids)?;
        append_input(
            &mut hasher,
            "sourceProviderInputs",
            self.source_provider_inputs,
        )?;
        append_input(&mut hasher, "packageManifests", &self.package_manifests)?;
        append_input(&mut hasher, "externalSifs", &self.external_sifs)?;
        append_input(
            &mut hasher,
            "externalSifResolutionEdges",
            &self.external_sif_resolution_edges,
        )?;
        append_input(
            &mut hasher,
            "externalSifTrustRecords",
            self.external_sif_trust_records,
        )?;
        append_input(&mut hasher, "resolutionInputs", self.resolution_inputs)?;
        append_input(&mut hasher, "settings", self.settings)?;
        append_input(
            &mut hasher,
            "sourceCorpusComplete",
            &self.source_corpus_complete,
        )?;
        Ok(OmenaWorkspaceInputCommitmentV0::from_sha256(
            hasher.finalize().into(),
        ))
    }

    fn bind(
        self,
        binding: &OmenaWorkspaceSnapshotBindingV0,
    ) -> Result<OmenaWorkspaceSnapshotReadViewV0<'a>, OmenaError> {
        if binding.workspace_root() != self.workspace_root
            || binding.input_commitment() != &self.input_commitment()?
        {
            return Err(snapshot_error(
                "snapshot binding does not match the actual workspace inputs",
                "workspace.snapshot-binding-mismatch",
            ));
        }
        Ok(OmenaWorkspaceSnapshotReadViewV0 {
            binding: binding.clone(),
            inputs: self,
            owner: None,
        })
    }
}

pub struct OmenaWorkspaceSnapshotReadViewV0<'a> {
    binding: OmenaWorkspaceSnapshotBindingV0,
    inputs: OmenaWorkspaceSnapshotInputsV0<'a>,
    owner: Option<OmenaWorkspaceSnapshotReaderV0>,
}

impl OmenaWorkspaceSnapshotReadViewV0<'_> {
    pub fn binding(&self) -> &OmenaWorkspaceSnapshotBindingV0 {
        &self.binding
    }

    pub fn owner(&self) -> Option<&OmenaWorkspaceSnapshotReaderV0> {
        self.owner.as_ref()
    }

    pub fn document_bytes(&self, path: &str) -> Option<&[u8]> {
        self.inputs
            .style_sources
            .iter()
            .find(|document| document.style_path == path)
            .map(|document| document.style_source.as_bytes())
            .or_else(|| {
                self.inputs
                    .source_documents
                    .iter()
                    .find(|document| document.source_path == path)
                    .map(|document| document.source_source.as_bytes())
            })
    }
}

/// Unique local publication authority. Cloning a reader never clones this
/// capability. An imported portable claim alone is not destination write authority.
#[derive(Debug, Default)]
pub struct OmenaWorkspaceSnapshotPublisherV0 {
    reader: OmenaWorkspaceSnapshotReaderV0,
    pending_generation: Option<u64>,
}

#[derive(Debug, Clone, Default)]
pub struct OmenaWorkspaceSnapshotReaderV0 {
    current: Arc<RwLock<WorkspaceSnapshotPublication>>,
}

#[derive(Debug, Default)]
struct WorkspaceSnapshotPublication {
    binding: Option<OmenaWorkspaceSnapshotBindingV0>,
    readable: bool,
    generation: u64,
    imported_origin: bool,
}

impl OmenaWorkspaceSnapshotPublisherV0 {
    pub fn reader(&self) -> OmenaWorkspaceSnapshotReaderV0 {
        self.reader.clone()
    }

    /// Start an owner change only from its actual current binding. While the
    /// unique owner retains this generation, all readers and commit guards refuse
    /// the old view. Only this owner can complete the publication.
    pub fn begin_mutation(
        &mut self,
        expected: &OmenaWorkspaceSnapshotBindingV0,
    ) -> Result<(), OmenaError> {
        let mut current = self.reader.current.write().map_err(|_| poisoned_owner())?;
        if self.pending_generation.is_some()
            || !current.readable
            || current.binding.as_ref() != Some(expected)
        {
            return Err(stale_snapshot());
        }
        if current
            .binding
            .as_ref()
            .is_some_and(|binding| binding.snapshot_id().value == u64::MAX)
        {
            return Err(revision_exhausted());
        }
        let generation = current
            .generation
            .checked_add(1)
            .ok_or_else(revision_exhausted)?;
        current.readable = false;
        current.generation = generation;
        self.pending_generation = Some(generation);
        Ok(())
    }

    /// Called before returning a mutable borrow of an existing owner's inputs.
    /// Repeated invalidations share the same unfinished generation. Poison is
    /// recovered only to revoke readability; publication will still refuse it.
    pub fn invalidate_before_owner_mutation(&mut self) {
        if self.pending_generation.is_some() {
            return;
        }
        let mut current = self
            .reader
            .current
            .write()
            .unwrap_or_else(|error| error.into_inner());
        if current.binding.is_none() {
            return;
        }
        current.readable = false;
        if let Some(generation) = current.generation.checked_add(1) {
            current.generation = generation;
            self.pending_generation = Some(generation);
        }
    }

    pub fn publish(
        &mut self,
        inputs: OmenaWorkspaceSnapshotInputsV0<'_>,
        revision_hint: OmenaWorkspaceSnapshotIdV0,
    ) -> Result<OmenaWorkspaceSnapshotBindingV0, OmenaError> {
        let commitment = inputs.input_commitment()?;
        let mut current = self.reader.current.write().map_err(|_| poisoned_owner())?;
        if let Some(previous) = current.binding.as_ref() {
            let unchanged = previous.workspace_root() == inputs.workspace_root
                && previous.input_commitment() == &commitment;
            match self.pending_generation {
                Some(generation) if generation == current.generation && !current.readable => {}
                None if current.readable && unchanged => return Ok(previous.clone()),
                _ => return Err(stale_snapshot()),
            }
        }
        let revision = match current.binding.as_ref() {
            Some(previous)
                if previous.workspace_root() == inputs.workspace_root
                    && previous.input_commitment() == &commitment =>
            {
                previous.snapshot_id().value
            }
            Some(previous) => previous
                .snapshot_id()
                .value
                .checked_add(1)
                .ok_or_else(revision_exhausted)?
                .max(revision_hint.value),
            None => revision_hint.value.max(1),
        };
        let binding = OmenaWorkspaceSnapshotBindingV0::new(
            inputs.workspace_root,
            OmenaWorkspaceSnapshotIdV0::from_revision(IncrementalRevisionV0 { value: revision }),
            commitment,
        );
        current.binding = Some(binding.clone());
        current.readable = true;
        self.pending_generation = None;
        Ok(binding)
    }

    /// A receiving host may initialize a new local owner from admitted immutable
    /// inputs. This does not connect it to a remote owner's later mutations.
    pub fn import(
        &mut self,
        binding: &OmenaWorkspaceSnapshotBindingV0,
        actual_inputs: OmenaWorkspaceSnapshotInputsV0<'_>,
    ) -> Result<(), OmenaError> {
        actual_inputs.bind(binding)?;
        let mut current = self.reader.current.write().map_err(|_| poisoned_owner())?;
        match current.binding.as_ref() {
            Some(existing)
                if existing == binding && current.readable && self.pending_generation.is_none() =>
            {
                Ok(())
            }
            Some(_) => Err(snapshot_error(
                "an imported snapshot cannot replace or revive an existing owner binding",
                "workspace.snapshot-import-conflict",
            )),
            None => {
                current.binding = Some(binding.clone());
                current.imported_origin = true;
                current.readable = true;
                Ok(())
            }
        }
    }
}

impl OmenaWorkspaceSnapshotReaderV0 {
    pub fn read_view<'a>(
        &self,
        binding: &OmenaWorkspaceSnapshotBindingV0,
        actual_inputs: OmenaWorkspaceSnapshotInputsV0<'a>,
    ) -> Result<OmenaWorkspaceSnapshotReadViewV0<'a>, OmenaError> {
        self.with_current_binding(binding, || {
            let mut view = actual_inputs.bind(binding)?;
            view.owner = Some(self.clone());
            Ok(view)
        })?
    }

    pub fn with_current_binding<T>(
        &self,
        binding: &OmenaWorkspaceSnapshotBindingV0,
        read: impl FnOnce() -> T,
    ) -> Result<T, OmenaError> {
        let current = self.current.read().map_err(|_| poisoned_owner())?;
        if !current.readable || current.binding.as_ref() != Some(binding) {
            return Err(stale_snapshot());
        }
        Ok(read())
    }

    /// Only the actual native owner can guard destination writes. An imported
    /// claim stays read-only for file writes even after local in-memory mutation.
    pub fn with_current_write_binding<T>(
        &self,
        binding: &OmenaWorkspaceSnapshotBindingV0,
        write: impl FnOnce() -> T,
    ) -> Result<T, OmenaError> {
        let current = self.current.read().map_err(|_| poisoned_owner())?;
        if current.imported_origin {
            return Err(snapshot_error(
                "imported snapshot claims do not confer native destination write authority",
                "workspace.snapshot-write-owner-required",
            ));
        }
        if !current.readable || current.binding.as_ref() != Some(binding) {
            return Err(stale_snapshot());
        }
        Ok(write())
    }

    pub fn current_binding(&self) -> Result<Option<OmenaWorkspaceSnapshotBindingV0>, OmenaError> {
        self.current
            .read()
            .map(|current| current.readable.then(|| current.binding.clone()).flatten())
            .map_err(|_| poisoned_owner())
    }

    pub fn is_same_owner(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.current, &other.current)
    }
}

fn stale_snapshot() -> OmenaError {
    snapshot_error(
        "request or mutation does not match the current workspace owner binding",
        "workspace.snapshot-mismatch",
    )
}

fn revision_exhausted() -> OmenaError {
    snapshot_error(
        "workspace snapshot revision space is exhausted",
        "workspace.snapshot-revision-exhausted",
    )
}

/// Rebuild non-wire source facts once when a receiving host admits owned text.
/// No claimed source index or unresolved-import bit is deserialized or trusted.
/// Owners retain the returned documents and reuse their facts for later queries.
pub fn reconstruct_omena_workspace_snapshot_sources_v0(
    workspace_root: &str,
    sources: &[crate::OmenaWorkspaceSnapshotSourceV0],
    styles: &[OmenaQueryStyleSourceInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    utility_intelligence: &crate::OmenaQueryUtilityClassIntelligenceReportV0,
) -> Result<Vec<OmenaQuerySourceDocumentInputV0>, OmenaError> {
    let mut paths = std::collections::BTreeSet::new();
    let mut documents = Vec::with_capacity(sources.len());
    for source in sources {
        if !paths.insert(source.source_path.as_str()) {
            return Err(snapshot_error(
                "snapshot contains duplicate source paths",
                "workspace.duplicate-source-path",
            ));
        }
        let imports = crate::summarize_omena_query_source_import_declarations_for_source_language(
            &source.source_path,
            &source.source_source,
            Some(&source.language_id),
        );
        let mut resolutions = Vec::new();
        let mut unresolved = false;
        for import in imports.imports {
            if crate::StyleLanguage::from_module_path(&import.specifier).is_none() {
                continue;
            }
            match crate::resolve_omena_query_style_uri_for_specifier_with_resolution_inputs(
                &source.source_path,
                Some(workspace_root),
                &import.specifier,
                resolution_inputs,
            ) {
                Some(uri) => resolutions.push(import.style_resolution(&uri)),
                None => unresolved = true,
            }
        }
        resolutions.sort();
        resolutions.dedup();
        let build = crate::summarize_omena_query_source_syntax_index_for_source_language_with_type_fact_attempts(
            &source.source_path,
            &source.source_source,
            Some(&source.language_id),
            resolutions,
        );
        let mut index = build.source_syntax_index;
        crate::append_omena_query_utility_class_intelligence(
            &mut index,
            &source.source_source,
            utility_intelligence,
        );
        if let Some(provider) = &source.provider_inputs {
            crate::replay_omena_workspace_source_provider_v0(
                &source.source_path,
                &source.source_source,
                &mut index,
                &build.type_fact_attempts,
                styles,
                provider,
            )?;
        }
        documents.push(OmenaQuerySourceDocumentInputV0 {
            source_path: source.source_path.clone(),
            source_source: source.source_source.clone(),
            source_syntax_index: Some(index),
            has_unresolved_style_import: unresolved,
        });
    }
    documents.sort_by(|left, right| left.source_path.cmp(&right.source_path));
    Ok(documents)
}

fn append_input<T: Serialize + ?Sized>(
    hasher: &mut Sha256,
    family: &str,
    input: &T,
) -> Result<(), OmenaError> {
    let bytes = serde_json::to_vec(input).map_err(|error| {
        snapshot_error(
            format!("workspace input commitment encoding failed: {error}"),
            "workspace.snapshot-input-encoding",
        )
    })?;
    let family_length = u64::try_from(family.len()).map_err(|_| input_length_error())?;
    let input_length = u64::try_from(bytes.len()).map_err(|_| input_length_error())?;
    hasher.update(family_length.to_be_bytes());
    hasher.update(family.as_bytes());
    hasher.update(input_length.to_be_bytes());
    hasher.update(bytes);
    Ok(())
}

fn poisoned_owner() -> OmenaError {
    snapshot_error(
        "workspace snapshot owner lock was poisoned",
        "workspace.snapshot-owner-unavailable",
    )
}

fn input_length_error() -> OmenaError {
    snapshot_error(
        "workspace input exceeds the commitment encoding length",
        "workspace.snapshot-input-length",
    )
}

fn snapshot_error(message: impl Into<String>, code: &str) -> OmenaError {
    OmenaError::new(
        OmenaErrorClassV0::Workspace,
        message,
        OmenaErrorContextV0 {
            code: code.to_string(),
            severity: OmenaErrorSeverityV0::Error,
            recoverability: OmenaErrorRecoverabilityV0::Retry,
            evidence: Vec::new(),
        },
    )
}
