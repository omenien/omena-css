//! RFC 0009 Pillar F (rfcs#68): the bounded parallel resolve wave for
//! memo-eligible style diagnostics targets.
//!
//! Shape of a wave, all within ONE loop turn: the scheduler hands over the
//! canonical-order URI list of a fan-out; loop-side we gather each eligible
//! style target's full input surface, serve exact-match disk-cache hits, and
//! group the remaining targets by byte-equality of that surface (mirroring
//! the memo host's own diff-sync compare). For a group of two or more
//! targets we run ONE `sync_workspace_for_parallel_resolve` (every salsa
//! `set_*` happens there, loop-side, before any handle exists), then fan the
//! targets across a bounded, wave-owned rayon pool — each worker rebuilds a
//! fixed-revision read view via `from_handle` and runs the tracked query
//! plus the worker-safe pure tail (`finish_style_diagnostics_value`). The
//! collect is order-preserving, the pool and every handle clone drop before
//! this module returns (the salsa pending-write contract: a leaked view
//! would block the next loop turn's `set_*` forever — nothing salsa-typed
//! can escape, the result map carries only rendered JSON plus loop-side
//! cache slots), and the scheduler then writes-behind and publishes in the
//! SAME canonical order as the serial arm, so the notification stream stays
//! byte-identical between the two arms.
//!
//! Everything else falls through to the existing serial code unchanged:
//! source documents, foreign-folder style targets (their surfaces differ),
//! duplicate-path corpora (the host refuses the sync), targets whose worker
//! panicked (the loop recomputes them serially and panics exactly where the
//! serial arm would), waves with at most one eligible target (the
//! runtime-loop probe's measured turn never pays for gathers or pool
//! spin-up), kill-switched sessions, and `--no-default-features` builds
//! (this module does not exist there).

use crate::disk_cache::{
    DiskDiagnosticsCacheSlotV0, disk_diagnostics_cache_slot_for_resolve,
    is_disk_diagnostics_cache_kill_switch_value,
};
use crate::{
    LspShellState, LspStyleDiagnosticsRenderInputsV0, finish_style_diagnostics_value,
    protocol::is_style_document_uri, query_adapter::query_style_hover_candidate_from_lsp,
    resolution_inputs_for_workspace_uri, source_documents_from_open_documents,
    style_hover_candidates_for_document, style_sources_from_open_documents,
};
use omena_query::{
    OmenaQuerySourceDocumentInputV0, OmenaQueryStyleHoverCandidateV0,
    OmenaQueryStyleMemoDatabaseV0, OmenaQueryStyleMemoHostV0, OmenaQueryStyleResolutionInputsV0,
    OmenaQueryStyleSourceInputV0, resolve_memo_workspace_style_diagnostics_from_view,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Waves with fewer eligible compute targets than this skip the pool (and,
/// below the same count of candidate URIs, skip even the per-target gather).
pub(crate) const PARALLEL_STYLE_WAVE_MIN_PARALLEL_TARGETS: usize = 2;
/// Thread budget cap: the binary already owns the loop, a query worker,
/// delayed-output writers and the tsgo process pool — the wave never takes
/// more than four threads, never fewer cores than targets warrant, and never
/// touches the global rayon pool.
const PARALLEL_STYLE_WAVE_MAX_THREADS: usize = 4;
/// Kill switch mirroring the disk-cache convention: `off` / `0` / `false`
/// route every wave back to the serial arm.
pub(crate) const PARALLEL_STYLE_DIAGNOSTICS_ENV_KILL_SWITCH: &str =
    "OMENA_LSP_PARALLEL_DIAGNOSTICS";

fn parallel_style_diagnostics_kill_switch_engaged() -> bool {
    std::env::var(PARALLEL_STYLE_DIAGNOSTICS_ENV_KILL_SWITCH)
        .is_ok_and(|value| is_disk_diagnostics_cache_kill_switch_value(value.as_str()))
}

/// One wave-resolved target, ready for the scheduler's loop-side pass. No
/// salsa type can ride along here — that is the type-level half of the
/// handle-scope invariant.
pub(crate) struct ResolvedParallelStyleTargetV0 {
    /// Rendered diagnostics JSON, byte-identical to the serial arm.
    pub(crate) diagnostics: Value,
    /// `Some` when the value was computed this wave (write-behind applies);
    /// `None` for an exact-match disk-cache hit — the serial arm returns
    /// before its write-behind on a hit, and the wave mirrors that.
    pub(crate) disk_cache_slot: Option<DiskDiagnosticsCacheSlotV0>,
}

/// The shared (group-key) portion of the input surface. Per-target gathers
/// are compared against the first eligible target's surface with the same
/// `PartialEq` the memo host's diff-sync uses; package manifests and
/// external SIFs are state-global within one wave, so they are shared
/// verbatim rather than keyed.
#[derive(PartialEq)]
struct ParallelStyleWaveSurfaceV0 {
    style_sources: Vec<OmenaQueryStyleSourceInputV0>,
    source_documents: Vec<OmenaQuerySourceDocumentInputV0>,
    resolution_inputs: OmenaQueryStyleResolutionInputsV0,
}

/// One compute-needing group member: the per-target inputs the worker owns
/// plus the loop-side cache slot for the write-behind after join.
struct ParallelStyleWaveTargetPlanV0 {
    /// Position in the scheduler's canonical URI list.
    index: usize,
    /// The document's own URI (compute identity, exactly what the serial
    /// resolve uses).
    target_uri: String,
    document_text: String,
    query_candidates: Vec<OmenaQueryStyleHoverCandidateV0>,
    disk_cache_slot: Option<DiskDiagnosticsCacheSlotV0>,
}

/// Resolve the memo-eligible style targets of `document_uris` on a bounded
/// parallel wave. Returns canonical-index-keyed results; any index absent
/// from the map (and every non-style URI) must be handled by the caller's
/// serial path. `min_parallel_targets` is the group-size knob — production
/// passes [`PARALLEL_STYLE_WAVE_MIN_PARALLEL_TARGETS`], tests pass
/// `usize::MAX` to force the serial arm for differential comparison.
pub(crate) fn resolved_parallel_style_wave_targets(
    state: &LspShellState,
    document_uris: &[String],
    min_parallel_targets: usize,
) -> BTreeMap<usize, ResolvedParallelStyleTargetV0> {
    let mut resolved = BTreeMap::new();
    let min_parallel_targets = min_parallel_targets.max(PARALLEL_STYLE_WAVE_MIN_PARALLEL_TARGETS);
    if parallel_style_diagnostics_kill_switch_engaged() {
        return resolved;
    }
    // Cheap candidate count before any per-target work: the single-target
    // turn (the runtime-loop probe's measured path) pays only this scan.
    let candidate_indices = document_uris
        .iter()
        .enumerate()
        .filter(|(_, uri)| is_style_document_uri(uri.as_str()))
        .filter(|(_, uri)| {
            state
                .document(uri.as_str())
                .is_some_and(|document| document.style_summary.is_some())
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if candidate_indices.len() < min_parallel_targets {
        return resolved;
    }

    let package_manifests = state.resolution.package_manifests.as_slice();
    let external_sifs = state.resolution.external_sifs.as_slice();
    let configured_severity = state.diagnostics.severity;
    let deep_analysis = state.diagnostics.deep_analysis;

    // Loop-side per-target gather + disk-cache load. The first eligible
    // target's surface becomes the group key; byte-unequal surfaces
    // (multi-root corpora, required-document append edges) fall through to
    // the serial arm — running them against a foreign synced revision would
    // be wrong, not just stale.
    let mut shared_surface: Option<Arc<ParallelStyleWaveSurfaceV0>> = None;
    let mut group: Vec<ParallelStyleWaveTargetPlanV0> = Vec::new();
    for index in candidate_indices {
        let uri = document_uris[index].as_str();
        let Some(document) = state.document(uri) else {
            continue;
        };
        let Some((_, candidates)) = style_hover_candidates_for_document(document) else {
            continue;
        };
        let surface = ParallelStyleWaveSurfaceV0 {
            style_sources: style_sources_from_open_documents(
                state,
                document.workspace_folder_uri.as_deref(),
                Some(document.uri.as_str()),
            ),
            source_documents: source_documents_from_open_documents(
                state,
                document.workspace_folder_uri.as_deref(),
            ),
            resolution_inputs: resolution_inputs_for_workspace_uri(
                state,
                document.workspace_folder_uri.as_deref(),
            ),
        };
        let surface = match shared_surface.as_ref() {
            None => {
                let surface = Arc::new(surface);
                shared_surface = Some(Arc::clone(&surface));
                surface
            }
            Some(shared) if **shared == surface => Arc::clone(shared),
            Some(_) => continue,
        };
        let disk_cache_slot = disk_diagnostics_cache_slot_for_resolve(
            state,
            document.workspace_folder_uri.as_deref(),
            document.uri.as_str(),
            surface.style_sources.as_slice(),
            surface.source_documents.as_slice(),
            external_sifs,
            &surface.resolution_inputs,
        );
        if let Some(cached_diagnostics) = disk_cache_slot.as_ref().and_then(|slot| slot.load()) {
            resolved.insert(
                index,
                ResolvedParallelStyleTargetV0 {
                    diagnostics: cached_diagnostics,
                    disk_cache_slot: None,
                },
            );
            continue;
        }
        group.push(ParallelStyleWaveTargetPlanV0 {
            index,
            target_uri: document.uri.clone(),
            document_text: document.text.clone(),
            query_candidates: candidates
                .iter()
                .map(query_style_hover_candidate_from_lsp)
                .collect(),
            disk_cache_slot,
        });
    }
    if group.len() < min_parallel_targets {
        return resolved;
    }
    let Some(shared_surface) = shared_surface else {
        return resolved;
    };

    // ONE host sync for the whole group: all set_* happen here, loop-side,
    // before the handle exists. A duplicate-path corpus refuses the sync and
    // the whole group falls back to the serial arm (which evaluates the
    // straight-line bypass, byte-identical by construction).
    let sync = {
        let mut host_slot = state.style_memo_host.borrow_mut();
        let host = host_slot.get_or_insert_with(OmenaQueryStyleMemoHostV0::new);
        host.sync_workspace_for_parallel_resolve(
            shared_surface.style_sources.as_slice(),
            shared_surface.source_documents.as_slice(),
            package_manifests,
            external_sifs,
            &shared_surface.resolution_inputs,
        )
    };
    let Some(sync) = sync else {
        return resolved;
    };
    let files_by_path = sync
        .files
        .iter()
        .map(|(style_path, file)| (style_path.as_str(), *file))
        .collect::<BTreeMap<_, _>>();
    // Pair each plan with its synced input entity; a target absent from the
    // corpus mirrors the host's target-outside-corpus `None` by falling back
    // to the serial arm (unreachable here — the gather appends the target).
    let pool_items = group
        .into_iter()
        .filter_map(|plan| {
            files_by_path
                .get(plan.target_uri.as_str())
                .copied()
                .map(|file| (plan, file))
        })
        .collect::<Vec<_>>();
    if pool_items.len() < min_parallel_targets {
        return resolved;
    }

    // The pool is wave-owned and bounded; built only on waves that fan out.
    let thread_count = std::thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(1)
        .min(PARALLEL_STYLE_WAVE_MAX_THREADS)
        .min(pool_items.len());
    let Ok(pool) = rayon::ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .build()
    else {
        return resolved;
    };
    let workspace = sync.workspace;
    let computed: Vec<Option<Value>> = pool.install(|| {
        pool_items
            .par_iter()
            .map_with(sync.handle.clone(), |handle, (plan, file)| {
                // A worker panic must not abort the wave: the target drops
                // out of the result map and the loop recomputes it serially,
                // panicking exactly where the serial arm would.
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let db = OmenaQueryStyleMemoDatabaseV0::from_handle(handle.clone());
                    let summary =
                        resolve_memo_workspace_style_diagnostics_from_view(&db, workspace, *file);
                    finish_style_diagnostics_value(
                        &LspStyleDiagnosticsRenderInputsV0 {
                            document_uri: plan.target_uri.as_str(),
                            document_text: plan.document_text.as_str(),
                            query_candidates: plan.query_candidates.as_slice(),
                            style_sources: shared_surface.style_sources.as_slice(),
                            source_documents: shared_surface.source_documents.as_slice(),
                            package_manifests,
                            deep_analysis,
                            configured_severity,
                        },
                        summary,
                    )
                }))
                .ok()
            })
            .collect()
    });
    // Order-preserving join happened above; drop the pool threads and the
    // handle before anything else — the next loop turn's set_* depends on it.
    drop(pool);
    drop(sync);

    for ((plan, _), diagnostics) in pool_items.into_iter().zip(computed) {
        if let Some(diagnostics) = diagnostics {
            resolved.insert(
                plan.index,
                ResolvedParallelStyleTargetV0 {
                    diagnostics,
                    disk_cache_slot: plan.disk_cache_slot,
                },
            );
        }
    }
    resolved
}
