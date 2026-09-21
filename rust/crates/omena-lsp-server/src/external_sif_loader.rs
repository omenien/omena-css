use crate::protocol::{file_uri_to_path, is_style_document_uri, normalize_path};
use crate::tide::{
    TideDisownCauseV0, TideFootprintStampV0, TideFootprintV0, TideGateInputsV0, TideInputKindV0,
    TideLaneConfigV0, TideRepublishDemandV0, TideSifDemandV0,
};
use crate::{LspShellState, LspTextDocumentState};
use omena_query::{
    OmenaQueryBridgeExternalSifTrustedResolutionV1, OmenaQueryExternalSifInputV0,
    OmenaQueryExternalSifStorageV0, OmenaQueryExternalSifTrustV1,
    OmenaQueryStyleResolutionInputsV0, OmenaQueryStyleSourceInputV0,
    resolve_omena_query_bridge_external_sifs_for_style_sources_with_cache_storage_and_trust,
    resolve_omena_query_bridge_external_sifs_for_style_sources_with_trust,
};
use omena_sif::OMENA_SIF_SHARD_VERDICT_DIR_V1;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct LspExternalSifRefreshDocumentV0 {
    pub uri: String,
    pub workspace_folder_uri: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct LspExternalSifRefreshJobV0 {
    pub stamp: TideFootprintStampV0,
    pub generation: u64,
    pub lockfiles: Vec<PathBuf>,
    pub documents: Vec<LspExternalSifRefreshDocumentV0>,
    pub package_manifests: Vec<omena_query::OmenaQueryStylePackageManifestV0>,
    pub resolution_inputs_by_workspace_uri:
        std::collections::BTreeMap<String, OmenaQueryStyleResolutionInputsV0>,
}

#[derive(Debug, Clone, Default)]
pub struct LspExternalSifRefreshCacheStorageV0 {
    by_workspace_uri: BTreeMap<String, OmenaQueryExternalSifStorageV0>,
    by_document_uri: BTreeMap<String, OmenaQueryExternalSifStorageV0>,
}

#[derive(Debug, Clone)]
pub struct LspExternalSifRefreshResultV0 {
    pub stamp: TideFootprintStampV0,
    pub generation: u64,
    pub external_sifs: Vec<OmenaQueryExternalSifInputV0>,
    pub bridge_external_sif_urls: BTreeSet<String>,
    /// Wire-compatibility sentinel for the retired automatic workspace-lock
    /// reader. Automatic LSP collection never reads a lock, so this is always
    /// zero; absence is verified by the product behavior and source census,
    /// not inferred from a synthetic observation set.
    pub lock_read_count: usize,
    pub bridge_generation_count: usize,
    pub trust_records: Vec<OmenaQueryExternalSifTrustV1>,
    pub resolution_edges: Vec<omena_query::OmenaQueryExternalSifResolutionEdgeV0>,
}

/// The SIF job's declared input footprint (rfcs#111 §4.1). DocumentText is
/// deliberately absent: text edits reach SIF resolution through the
/// bridge-source delta path, which deposits a NEW demand instead of staling
/// the in-flight job — an unfootprinted clock would discard in-flight work
/// on every keystroke (the review BLOCKER).
pub(crate) const EXTERNAL_SIF_FOOTPRINT: TideFootprintV0 = TideFootprintV0::of(&[
    TideInputKindV0::DocumentSet,
    TideInputKindV0::LockfileFingerprint,
    TideInputKindV0::PackageManifest,
    TideInputKindV0::ResolutionSettings,
    TideInputKindV0::ExternalSifSource,
]);

pub(crate) fn external_sif_source_uri_is_dependency(state: &LspShellState, uri: &str) -> bool {
    let file_id = state.known_document_file_id(uri);
    state
        .resolution
        .external_sifs
        .iter()
        .map(|input| input.sif.canonical_url.as_str())
        .chain(
            state
                .resolution
                .bridge_external_sif_urls
                .iter()
                .map(String::as_str)
                .filter(|target| target.starts_with("file://")),
        )
        .any(|target| {
            // The existing bridge target set also records attempted reads and
            // file URLs behind package-canonical SIFs. These are invalidation
            // dependencies only; matching them never admits SIF/trust claims.
            target == uri
                || file_id.is_some_and(|id| state.known_document_file_id(target) == Some(id))
        })
}

/// SettleGated lanes flush on frontier passage alone: the courtesy layer is
/// pinned open, so the aging bound is never consulted.
pub(crate) const TIDE_SETTLE_LANE_CONFIG: TideLaneConfigV0 = TideLaneConfigV0 {
    aging_bound_ticks: u64::MAX,
};

pub(crate) fn refresh_external_sifs_for_state(state: &mut LspShellState) {
    if state.external_sif_refresh_deferred {
        crate::loop_trace!("sif-demand reason=state-refresh");
        let tick = state.tide_tick;
        state
            .tide_sif_lane
            .deposit(TideSifDemandV0::refresh(), tick);
        return;
    }
    refresh_external_sifs_for_state_immediate(state);
}

fn refresh_external_sifs_for_state_immediate(state: &mut LspShellState) {
    // Workspace lock bytes are attacker-writable automatic inputs, so the LSP
    // product path does not read them. Only independently regenerated local
    // bridge bytes can enter the automatic external-SIF state.
    let bridge_result = resolve_in_process_external_sifs_for_lsp(state, &BTreeSet::new());
    state.external_sif_bridge_generation_count = state
        .external_sif_bridge_generation_count
        .saturating_add(bridge_result.resolution.generation_count);
    let external_sifs = bridge_result.resolution.external_sifs;

    let trust_records = external_sif_trust_record_map(bridge_result.trust_records);
    if state.resolution.external_sifs != external_sifs
        || state.resolution.external_sif_trust_records != trust_records
        || state.resolution.external_sif_resolution_edges != bridge_result.resolution_edges
    {
        state.invalidate_sdk_snapshots_before_owner_mutation();
        state.resolution.external_sifs = external_sifs;
        state.resolution.external_sif_trust_records = trust_records;
        state.resolution.external_sif_resolution_edges = bridge_result.resolution_edges;
        invalidate_external_sif_dependents(state);
    }
    state.resolution.bridge_external_sif_urls =
        bridge_result.resolution.bridge_urls.into_iter().collect();
}

pub(crate) fn refresh_external_sifs_for_bridge_source_delta(
    state: &mut LspShellState,
    affected_document_uris: &[String],
    previous_sources: &[String],
    next_sources: &[String],
) {
    if state.external_sif_refresh_deferred {
        // Mirror the immediate arm's early return: an unchanged bridge-source
        // set has nothing to refresh, so it must not mark the deferred job
        // dirty (previously every no-op admit wave scheduled a full external
        // SIF re-resolution and raced the in-flight one's revision).
        let previous_set = previous_sources.iter().collect::<BTreeSet<_>>();
        let next_set = next_sources.iter().collect::<BTreeSet<_>>();
        if previous_set == next_set {
            crate::loop_trace!(
                "sif-demand SKIPPED reason=bridge-delta-equal len={}",
                next_set.len()
            );
            return;
        }
        crate::loop_trace!(
            "sif-demand reason=bridge-delta prev={} next={}",
            previous_sources.len(),
            next_sources.len()
        );
        // A genuine bridge-topology change is a corpus-input mutation: it
        // stales any in-flight SIF job (footprint member) and deposits the
        // demand whose tide will re-resolve against the new topology.
        state.tide_ledger.advance(&[TideInputKindV0::DocumentSet]);
        let affected_file_ids = affected_document_uris
            .iter()
            .filter_map(|uri| state.document_file_id(uri))
            .collect::<Vec<_>>();
        state.tide_reopen_republish_window(TideDisownCauseV0::for_file_ids(
            TideInputKindV0::DocumentSet,
            affected_file_ids,
        ));
        let tick = state.tide_tick;
        state
            .tide_sif_lane
            .deposit(TideSifDemandV0::refresh(), tick);
        return;
    }
    let previous_sources = previous_sources.iter().cloned().collect::<BTreeSet<_>>();
    let next_sources = next_sources.iter().cloned().collect::<BTreeSet<_>>();
    if previous_sources == next_sources {
        return;
    }
    // A changed import topology requires the same contextual document admission
    // as a full refresh. Updating context-free URL aliases cannot retain which
    // importer selected a reused target. Equal topology still cuts off above.
    refresh_external_sifs_for_state_immediate(state);
}

pub(crate) fn bridge_sources_for_style_uris(
    state: &LspShellState,
    style_uris: &[String],
) -> Vec<String> {
    let mut sources = BTreeSet::new();
    for uri in style_uris {
        let Some(document) = state.document(uri.as_str()) else {
            continue;
        };
        collect_bridge_sources_from_style_document(document, &mut sources);
    }
    sources.into_iter().collect()
}

pub fn enable_deferred_external_sif_refresh(state: &mut LspShellState) {
    state.external_sif_refresh_deferred = true;
    let tick = state.tide_tick;
    state
        .tide_sif_lane
        .deposit(TideSifDemandV0::refresh(), tick);
}

pub fn prepare_deferred_external_sif_refresh_job(
    state: &mut LspShellState,
) -> Option<LspExternalSifRefreshJobV0> {
    if !state.external_sif_refresh_deferred {
        return None;
    }
    // Settle gate: the correctness layer is the index frontier — no flush
    // while an index chain still has pending files. The lane enforces one
    // in-flight tide, so a second prepare during a running job is a no-op.
    let inputs = TideGateInputsV0 {
        frontier_passed: state.workspace_index_pending_file_count == 0,
        idle: true,
    };
    let flush = state
        .tide_sif_lane
        .try_flush(inputs, state.tide_tick, &TIDE_SETTLE_LANE_CONFIG)?;
    crate::loop_trace!(
        "sif-job-prepared gen={} epoch={} docs={}",
        flush.generation,
        state.tide_ledger.epoch(),
        state
            .documents
            .values()
            .filter(|d| is_style_document_uri(d.uri.as_str()))
            .count()
    );
    Some(LspExternalSifRefreshJobV0 {
        stamp: state.tide_ledger.stamp(EXTERNAL_SIF_FOOTPRINT),
        generation: flush.generation,
        lockfiles: workspace_lockfiles(state),
        documents: state
            .documents
            .values()
            .map(AsRef::as_ref)
            .filter(|document| is_style_document_uri(document.uri.as_str()))
            .map(|document| LspExternalSifRefreshDocumentV0 {
                uri: document.uri.clone(),
                workspace_folder_uri: document.workspace_folder_uri.clone(),
                text: document.text.clone(),
            })
            .collect(),
        package_manifests: state.resolution.package_manifests.clone(),
        resolution_inputs_by_workspace_uri: state
            .resolution
            .workspace_style_resolution_inputs
            .clone(),
    })
}

pub fn prepare_deferred_external_sif_refresh_cache_storage(
    state: &LspShellState,
) -> LspExternalSifRefreshCacheStorageV0 {
    let mut by_workspace_uri = BTreeMap::new();
    for folder in state.workspace_runtime_registry.folder_snapshots() {
        if let Some(storage) =
            bridge_cache_storage_for_workspace_uri(state, Some(folder.uri.as_str()))
        {
            by_workspace_uri.insert(folder.uri, storage);
        }
    }
    let by_document_uri = state
        .documents
        .values()
        .map(AsRef::as_ref)
        .filter(|document| {
            document.workspace_folder_uri.is_none() && is_style_document_uri(document.uri.as_str())
        })
        .filter_map(|document| {
            bridge_cache_storage_for_document(state, None, document.uri.as_str())
                .map(|storage| (document.uri.clone(), storage))
        })
        .collect();
    LspExternalSifRefreshCacheStorageV0 {
        by_workspace_uri,
        by_document_uri,
    }
}

pub fn collect_deferred_external_sif_refresh(
    job: LspExternalSifRefreshJobV0,
) -> LspExternalSifRefreshResultV0 {
    collect_deferred_external_sif_refresh_with_cache_storage(
        job,
        LspExternalSifRefreshCacheStorageV0::default(),
    )
}

pub fn collect_deferred_external_sif_refresh_with_cache_storage(
    job: LspExternalSifRefreshJobV0,
    cache_storage: LspExternalSifRefreshCacheStorageV0,
) -> LspExternalSifRefreshResultV0 {
    let LspExternalSifRefreshJobV0 {
        stamp,
        generation,
        lockfiles: excluded_lockfiles,
        documents,
        package_manifests,
        resolution_inputs_by_workspace_uri,
    } = job;
    drop(excluded_lockfiles);
    let bridge_result = resolve_external_sifs_for_refresh_documents(
        documents.as_slice(),
        &[],
        package_manifests.as_slice(),
        &resolution_inputs_by_workspace_uri,
        Some(&cache_storage),
    );
    let external_sifs = bridge_result.resolution.external_sifs;

    LspExternalSifRefreshResultV0 {
        stamp,
        generation,
        external_sifs,
        bridge_external_sif_urls: bridge_result.resolution.bridge_urls.into_iter().collect(),
        lock_read_count: 0,
        bridge_generation_count: bridge_result.resolution.generation_count,
        trust_records: bridge_result.trust_records,
        resolution_edges: bridge_result.resolution_edges,
    }
}

pub fn apply_deferred_external_sif_refresh_result(
    state: &mut LspShellState,
    result: LspExternalSifRefreshResultV0,
) -> bool {
    if !state.tide_ledger.is_current(&result.stamp) {
        crate::loop_trace!(
            "sif-apply DISCARDED gen={} stamp_epoch={} ledger_epoch={}",
            result.generation,
            result.stamp.epoch,
            state.tide_ledger.epoch()
        );
        // The staling mutation also deposited a fresh demand (every advance
        // site deposits), so completing the disowned tide re-arms the gate.
        state.tide_sif_lane.tide_completed(result.generation);
        return false;
    }
    state.external_sif_bridge_generation_count = state
        .external_sif_bridge_generation_count
        .saturating_add(result.bridge_generation_count);
    let trust_records = external_sif_trust_record_map(result.trust_records);
    let changed = state.resolution.external_sifs != result.external_sifs
        || state.resolution.external_sif_trust_records != trust_records
        || state.resolution.external_sif_resolution_edges != result.resolution_edges;
    crate::loop_trace!(
        "sif-apply gen={} changed={} sifs {}->{}",
        result.generation,
        changed,
        state.resolution.external_sifs.len(),
        result.external_sifs.len()
    );
    if changed {
        state.invalidate_sdk_snapshots_before_owner_mutation();
        // Cone seeding (rfcs#111 demand lattice): the republish owed by a
        // SIF delta is the set of files that import a CHANGED fact, not the
        // workspace. Computed BEFORE the swap so the old set is diffable.
        let mut demand =
            republish_demand_for_external_sif_delta(state, result.external_sifs.as_slice());
        use crate::tide::TideDemandJoinV0;
        demand.join(republish_demand_for_external_sif_resolution_delta(
            state,
            &result.resolution_edges,
        ));
        state.resolution.external_sifs = result.external_sifs;
        state.resolution.external_sif_trust_records = trust_records;
        state.resolution.external_sif_resolution_edges = result.resolution_edges;
        invalidate_external_sif_dependents(state);
        // Output cutoff (rfcs#111 §4.1): only a CHANGED SIF set owes the
        // workspace republish; an Eq result blocks downstream entirely.
        state.tide_reopen_republish_window(TideDisownCauseV0::for_republish_demand(
            TideInputKindV0::DocumentSet,
            demand.clone(),
        ));
        let tick = state.tide_tick;
        state.tide_republish_lane.deposit(demand, tick);
    }
    state.resolution.bridge_external_sif_urls = result.bridge_external_sif_urls;
    state.tide_sif_lane.tide_completed(result.generation);
    changed
}

/// Mapping-only changes owe diagnostics even if the unordered SIF set is equal.
/// Document origins have existing owner IDs; unattributable transitive contexts
/// conservatively use the existing All demand, never a new identity issuer.
fn republish_demand_for_external_sif_resolution_delta(
    state: &LspShellState,
    next: &[omena_query::OmenaQueryExternalSifResolutionEdgeV0],
) -> TideRepublishDemandV0 {
    use omena_query::OmenaQueryExternalSifImportOriginV0 as Origin;
    let previous = state
        .resolution
        .external_sif_resolution_edges
        .iter()
        .collect::<BTreeSet<_>>();
    let next = next.iter().collect::<BTreeSet<_>>();
    let mut seeds = BTreeSet::new();
    for edge in previous.symmetric_difference(&next) {
        let Origin::Document { style_path } = &edge.importer else {
            return TideRepublishDemandV0::All;
        };
        let Some(id) = state.known_document_file_id(style_path) else {
            return TideRepublishDemandV0::All;
        };
        seeds.insert(id);
    }
    TideRepublishDemandV0::cone(seeds)
}

/// The republish demand a SIF delta deposits: `Cone(importers of every
/// changed url)` when the loop's reverse-dependency index can attribute
/// EVERY changed fact, `All` otherwise — a cold start has no index yet
/// (everything owes its first publish anyway), and an unattributable url
/// must widen rather than guess. Seeds are direct importers; the flush
/// takes their reverse closure against the then-current graph.
#[cfg(feature = "salsa-style-diagnostics")]
pub(crate) fn republish_demand_for_external_sif_delta(
    state: &LspShellState,
    next_external_sifs: &[OmenaQueryExternalSifInputV0],
) -> TideRepublishDemandV0 {
    let previous: BTreeMap<&str, &OmenaQueryExternalSifInputV0> = state
        .resolution
        .external_sifs
        .iter()
        .map(|input| (input.canonical_url.as_str(), input))
        .collect();
    let next: BTreeMap<&str, &OmenaQueryExternalSifInputV0> = next_external_sifs
        .iter()
        .map(|input| (input.canonical_url.as_str(), input))
        .collect();
    let mut changed_urls = BTreeSet::new();
    for (url, input) in &next {
        if previous.get(url).is_none_or(|prev| prev != input) {
            changed_urls.insert(*url);
        }
    }
    for url in previous.keys() {
        if !next.contains_key(url) {
            changed_urls.insert(*url);
        }
    }
    if changed_urls.is_empty() {
        return TideRepublishDemandV0::None;
    }
    let memo_slot = state.reverse_dependency_index_memo.borrow();
    let Some(memo) = memo_slot.as_ref() else {
        crate::loop_trace!(
            "republish-demand all: {} changed sif urls, no reverse index",
            changed_urls.len()
        );
        return TideRepublishDemandV0::All;
    };
    // Freshness gate: a memo that predates the latest corpus-shaping input
    // marks may hold a rev-set that is PRESENT but stale (a just-added
    // importer missing from it) — presence alone cannot widen, so the epoch
    // comparison does. Widen, never guess.
    let corpus_input_mark = state
        .tide_ledger
        .mark(TideInputKindV0::DocumentText)
        .max(state.tide_ledger.mark(TideInputKindV0::DocumentSet));
    if memo.ledger_epoch < corpus_input_mark {
        crate::loop_trace!(
            "republish-demand all: reverse index stale (memo epoch {} < corpus mark {})",
            memo.ledger_epoch,
            corpus_input_mark
        );
        return TideRepublishDemandV0::All;
    }
    let mut seeds: BTreeSet<crate::LspFileId> = BTreeSet::new();
    for url in &changed_urls {
        // A fact can appear as an edge target under its alias key or its
        // resolved canonical url; consult both, from whichever side of the
        // delta knows the entry.
        let resolved_alias = next
            .get(url)
            .or_else(|| previous.get(url))
            .map(|input| input.sif.canonical_url.as_str());
        let dependents = memo
            .index
            .rev
            .get(*url)
            .or_else(|| resolved_alias.and_then(|alias| memo.index.rev.get(alias)));
        let Some(dependents) = dependents else {
            crate::loop_trace!("republish-demand all: unattributed sif url {url}");
            return TideRepublishDemandV0::All;
        };
        for dependent in dependents {
            let Some(file_id) = state.document_file_id(dependent) else {
                crate::loop_trace!(
                    "republish-demand all: dependent has no admitted file id {dependent}"
                );
                return TideRepublishDemandV0::All;
            };
            seeds.insert(file_id);
        }
    }
    crate::loop_trace!(
        "republish-demand cone seeds={} changed_urls={}",
        seeds.len(),
        changed_urls.len()
    );
    TideRepublishDemandV0::cone(seeds)
}

#[cfg(not(feature = "salsa-style-diagnostics"))]
pub(crate) fn republish_demand_for_external_sif_delta(
    _state: &LspShellState,
    _next_external_sifs: &[OmenaQueryExternalSifInputV0],
) -> TideRepublishDemandV0 {
    TideRepublishDemandV0::All
}

fn workspace_lockfiles(state: &LspShellState) -> Vec<PathBuf> {
    let mut lockfiles = BTreeSet::new();
    for folder in state.workspace_runtime_registry.folder_snapshots() {
        let Some(root) = file_uri_to_path(folder.uri.as_str()).map(normalize_path) else {
            continue;
        };
        if let Some(lockfile) = discover_omena_lockfile_for_workspace_root(root.as_path()) {
            lockfiles.insert(lockfile);
        }
    }
    lockfiles.into_iter().collect()
}

fn discover_omena_lockfile_for_workspace_root(root: &Path) -> Option<PathBuf> {
    let mut current = Some(root);
    while let Some(directory) = current {
        let candidate = directory.join("omena.lock");
        if candidate.exists() {
            return Some(normalize_path(candidate));
        }
        current = directory.parent();
    }
    None
}

pub(crate) fn external_sif_trust_record_map(
    records: Vec<OmenaQueryExternalSifTrustV1>,
) -> BTreeMap<String, OmenaQueryExternalSifTrustV1> {
    records
        .into_iter()
        .map(|record| (record.canonical_url.clone(), record))
        .collect()
}

fn deduplicate_external_sif_trust_records(records: &mut Vec<OmenaQueryExternalSifTrustV1>) {
    *records = external_sif_trust_record_map(std::mem::take(records))
        .into_values()
        .collect();
}

fn resolve_in_process_external_sifs_for_lsp(
    state: &LspShellState,
    _existing_covered: &BTreeSet<String>,
) -> OmenaQueryBridgeExternalSifTrustedResolutionV1 {
    // Raw import aliases belong to their originating document. Reusing another
    // document's alias can suppress admission of a different resolved target.
    let mut combined = OmenaQueryBridgeExternalSifTrustedResolutionV1::default();
    let mut bridge_urls = BTreeSet::new();

    for document in state.documents.values().map(AsRef::as_ref) {
        if !is_style_document_uri(document.uri.as_str()) {
            continue;
        }
        let source = OmenaQueryStyleSourceInputV0 {
            style_path: document.uri.clone(),
            style_source: document.text.clone(),
        };
        let resolution_inputs =
            resolution_inputs_for_document(state, document.workspace_folder_uri.as_deref());
        let cache_storage = bridge_cache_storage_for_document(
            state,
            document.workspace_folder_uri.as_deref(),
            document.uri.as_str(),
        );
        let result = if let Some(cache_storage) = cache_storage.as_ref() {
            resolve_omena_query_bridge_external_sifs_for_style_sources_with_cache_storage_and_trust(
                std::slice::from_ref(&source),
                &[],
                &resolution_inputs,
                cache_storage,
            )
        } else {
            resolve_omena_query_bridge_external_sifs_for_style_sources_with_trust(
                std::slice::from_ref(&source),
                &[],
                &resolution_inputs,
            )
        };
        combined.resolution.generation_count = combined
            .resolution
            .generation_count
            .saturating_add(result.resolution.generation_count);
        bridge_urls.extend(result.resolution.bridge_urls);
        combined.trust_records.extend(result.trust_records);
        combined.resolution_edges.extend(result.resolution_edges);
        for external_sif in result.resolution.external_sifs {
            combined.resolution.external_sifs.push(external_sif);
        }
    }

    combined.resolution.bridge_urls = bridge_urls.into_iter().collect();
    deduplicate_external_sif_trust_records(&mut combined.trust_records);
    combined.resolution_edges.sort();
    combined.resolution_edges.dedup();
    combined
}

fn resolve_external_sifs_for_refresh_documents(
    documents: &[LspExternalSifRefreshDocumentV0],
    _existing_external_sifs: &[OmenaQueryExternalSifInputV0],
    package_manifests: &[omena_query::OmenaQueryStylePackageManifestV0],
    resolution_inputs_by_workspace_uri: &std::collections::BTreeMap<
        String,
        OmenaQueryStyleResolutionInputsV0,
    >,
    cache_storage: Option<&LspExternalSifRefreshCacheStorageV0>,
) -> OmenaQueryBridgeExternalSifTrustedResolutionV1 {
    let mut combined = OmenaQueryBridgeExternalSifTrustedResolutionV1::default();
    let mut bridge_urls = BTreeSet::new();

    for document in documents {
        let source = OmenaQueryStyleSourceInputV0 {
            style_path: document.uri.clone(),
            style_source: document.text.clone(),
        };
        let resolution_inputs = document
            .workspace_folder_uri
            .as_deref()
            .and_then(|uri| resolution_inputs_by_workspace_uri.get(uri))
            .cloned()
            .unwrap_or_else(|| OmenaQueryStyleResolutionInputsV0 {
                package_manifests: package_manifests.to_vec(),
                ..OmenaQueryStyleResolutionInputsV0::default()
            });
        let document_cache_storage = cache_storage.and_then(|storage| {
            if let Some(workspace_uri) = document.workspace_folder_uri.as_ref() {
                storage.by_workspace_uri.get(workspace_uri)
            } else {
                storage.by_document_uri.get(document.uri.as_str())
            }
        });
        let result = if let Some(document_cache_storage) = document_cache_storage {
            resolve_omena_query_bridge_external_sifs_for_style_sources_with_cache_storage_and_trust(
                std::slice::from_ref(&source),
                &[],
                &resolution_inputs,
                document_cache_storage,
            )
        } else {
            resolve_omena_query_bridge_external_sifs_for_style_sources_with_trust(
                std::slice::from_ref(&source),
                &[],
                &resolution_inputs,
            )
        };
        combined.resolution.generation_count = combined
            .resolution
            .generation_count
            .saturating_add(result.resolution.generation_count);
        bridge_urls.extend(result.resolution.bridge_urls);
        combined.trust_records.extend(result.trust_records);
        combined.resolution_edges.extend(result.resolution_edges);
        for external_sif in result.resolution.external_sifs {
            combined.resolution.external_sifs.push(external_sif);
        }
    }

    combined.resolution.bridge_urls = bridge_urls.into_iter().collect();
    deduplicate_external_sif_trust_records(&mut combined.trust_records);
    combined.resolution_edges.sort();
    combined.resolution_edges.dedup();
    combined
}

fn bridge_storage_for_resolved_workspace_root(
    workspace_cache_root: Option<PathBuf>,
    workspace_identity: &str,
    workspace_root: &Path,
) -> OmenaQueryExternalSifStorageV0 {
    OmenaQueryExternalSifStorageV0::from_optional_workspace_cache_root_and_verdict_dir(
        workspace_cache_root,
        workspace_identity,
        workspace_root
            .join(".cache")
            .join("omena")
            .join(OMENA_SIF_SHARD_VERDICT_DIR_V1),
    )
}

fn bridge_cache_storage_for_workspace_uri(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
) -> Option<OmenaQueryExternalSifStorageV0> {
    let workspace_folder_uri = workspace_folder_uri?;
    let workspace_root = file_uri_to_path(workspace_folder_uri)?;
    let workspace_cache_root = crate::cache_root::resolved_bridge_workspace_cache_root(
        &state.resolution.cache_storage,
        workspace_folder_uri,
        workspace_root.as_path(),
    );
    Some(bridge_storage_for_resolved_workspace_root(
        workspace_cache_root,
        workspace_folder_uri,
        workspace_root.as_path(),
    ))
}

pub(crate) fn bridge_cache_storage_for_document(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    document_uri: &str,
) -> Option<OmenaQueryExternalSifStorageV0> {
    if let Some(workspace_folder_uri) = workspace_folder_uri {
        return bridge_cache_storage_for_workspace_uri(state, Some(workspace_folder_uri));
    }
    let document_path = file_uri_to_path(document_uri)?;
    let document_root = document_path.parent()?;
    let workspace_identity = document_root.to_string_lossy();
    let workspace_cache_root = crate::cache_root::resolved_bridge_workspace_cache_root(
        &state.resolution.cache_storage,
        workspace_identity.as_ref(),
        document_root,
    );
    Some(bridge_storage_for_resolved_workspace_root(
        workspace_cache_root,
        workspace_identity.as_ref(),
        document_root,
    ))
}

fn resolution_inputs_for_document(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
) -> OmenaQueryStyleResolutionInputsV0 {
    workspace_folder_uri
        .and_then(|uri| {
            state
                .resolution
                .workspace_style_resolution_inputs
                .get(uri)
                .cloned()
        })
        .unwrap_or_else(|| OmenaQueryStyleResolutionInputsV0 {
            package_manifests: state.resolution.package_manifests.clone(),
            ..OmenaQueryStyleResolutionInputsV0::default()
        })
}

fn collect_bridge_sources_from_style_document(
    document: &LspTextDocumentState,
    sources: &mut BTreeSet<String>,
) {
    let Some(summary) = document.style_summary.as_ref() else {
        return;
    };
    let edge_sources = summary
        .sass_module_use_sources
        .iter()
        .map(String::as_str)
        .chain(
            summary
                .sass_module_forward_sources
                .iter()
                .map(String::as_str),
        );
    for edge_source in edge_sources {
        if edge_source.starts_with("file://") {
            sources.insert(edge_source.to_string());
        }
    }
}

fn invalidate_external_sif_dependents(state: &mut LspShellState) {
    *state.workspace_occurrence_index_memo_lock() = None;
    if let Ok(mut memo) = state.cascade_narrowing_substrate_memo.lock() {
        *memo = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_storage_retains_importing_root_verdict_when_cache_is_unavailable() {
        let first = Path::new("/workspace/a");
        let second = Path::new("/workspace/b");
        for (identity, root) in [
            ("file:///workspace/a", first),
            ("file:///workspace/b", second),
        ] {
            let expected_verdict = root
                .join(".cache/omena")
                .join(OMENA_SIF_SHARD_VERDICT_DIR_V1);
            let disabled = bridge_storage_for_resolved_workspace_root(None, identity, root);
            assert_eq!(disabled.workspace_cache_root(), None);
            assert_eq!(
                disabled.recorded_verdict_dir(),
                Some(expected_verdict.as_path())
            );
            let configured_cache = PathBuf::from("/configured/editor/cache");
            let enabled = bridge_storage_for_resolved_workspace_root(
                Some(configured_cache.clone()),
                identity,
                root,
            );
            assert_eq!(
                enabled.workspace_cache_root(),
                Some(configured_cache.as_path())
            );
            assert_eq!(
                enabled.recorded_verdict_dir(),
                disabled.recorded_verdict_dir()
            );
        }
    }
}
