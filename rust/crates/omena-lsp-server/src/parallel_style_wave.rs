//! RFC 0009 Pillar F (rfcs#68): the bounded parallel resolve wave for
//! memo-eligible style diagnostics targets.
//!
//! Shape of a wave, all within ONE loop turn: the scheduler hands over the
//! canonical-order URI list of a fan-out; loop-side we gather each eligible
//! style target's full input surface, serve exact-match disk-cache hits, and
//! group the remaining targets by byte-equality of that surface (mirroring
//! the memo host's own diff-sync compare). For a group of two or more
//! targets we run ONE `sync_workspace_for_parallel_resolve`, which commits
//! the group and hands back a selector-backed read bundle. The targets then
//! fan across a bounded, wave-owned rayon pool — each worker rebuilds a
//! fixed-revision read view via `from_handle` and runs the tracked query plus
//! the worker-safe pure tail (`finish_style_diagnostics_value`). The collect
//! is order-preserving, nothing salsa-typed escapes, the result map carries
//! only rendered JSON plus loop-side cache slots, and the scheduler then
//! writes-behind and publishes in the SAME canonical order as the serial arm,
//! so the notification stream stays byte-identical between the two arms.
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
use crate::style_diagnostics::finish_style_diagnostics_value_with_shared_reachability;
use crate::{
    LspShellState, LspStyleDiagnosticsRenderInputsV0, protocol::is_style_document_uri,
    query_adapter::query_style_hover_candidate_from_lsp, resolution_inputs_for_workspace_uri,
    source_documents_from_open_documents, state::LspResolverIdentityIndexMemo,
    style_hover_candidates_for_document, style_sources_from_open_documents,
};
use omena_query::{
    OmenaQuerySourceDocumentInputV0, OmenaQueryStyleHoverCandidateV0,
    OmenaQueryStyleMemoDatabaseV0, OmenaQueryStyleMemoHostV0, OmenaQueryStyleResolutionInputsV0,
    OmenaQueryStyleSourceInputV0, OmenaResolverStyleModuleConfirmationIdentityIndexV0,
    build_omena_resolver_style_module_confirmation_identity_index,
    omena_resolver_style_identity_generation, prepare_committed_workspace_wave_substrate,
    resolve_committed_workspace_style_diagnostics_from_view_with_identity_index_and_wave_substrate,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
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
    /// Shadow-oracle mode only: the manifest-verified shard value this
    /// target ALSO computes against. The computed value is what gets
    /// served; the comparison is telemetry for read-set completeness.
    oracle_cached_diagnostics: Option<Value>,
}

fn resolver_identity_index_for_parallel_style_wave(
    state: &LspShellState,
    surface: &ParallelStyleWaveSurfaceV0,
) -> Arc<OmenaResolverStyleModuleConfirmationIdentityIndexV0> {
    let generation = omena_resolver_style_identity_generation();
    let available_style_paths = surface
        .style_sources
        .iter()
        .map(|source| source.style_path.clone())
        .collect::<Vec<_>>();
    let disk_style_path_identities = surface.resolution_inputs.disk_style_path_identities.clone();
    {
        let memo = state.resolver_identity_index_memo_lock();
        if let Some(memo) = memo.as_ref()
            && memo.generation == generation
            && memo.available_style_paths == available_style_paths
            && memo.disk_style_path_identities == disk_style_path_identities
        {
            return Arc::clone(&memo.index);
        }
    }

    let available_style_path_refs = available_style_paths
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let index = Arc::new(
        build_omena_resolver_style_module_confirmation_identity_index(
            &available_style_path_refs,
            disk_style_path_identities.as_slice(),
        ),
    );
    *state.resolver_identity_index_memo_lock() = Some(LspResolverIdentityIndexMemo {
        generation,
        available_style_paths,
        disk_style_path_identities,
        index: Arc::clone(&index),
    });
    index
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
    resolved_parallel_style_wave_targets_with_abort(
        state,
        document_uris,
        min_parallel_targets,
        None,
    )
}

/// Per-item sink: index into `document_uris`, the rendered diagnostics, and
/// the write-behind slot. Called from pool threads as each target finishes —
/// the streaming arm of the republish tide (rfcs#111 §8.5). Sink callers
/// still receive the full result map; the sink only adds early delivery.
pub(crate) type ParallelStyleWaveItemSinkV0<'sink> =
    &'sink (dyn Fn(usize, Value, Option<DiskDiagnosticsCacheSlotV0>) + Sync);

/// The abort-capable arm (rfcs#111 §9.4): when `abort` is `Some`, each pool
/// task first compares the shared generation watch against the tide's
/// generation and returns uncovered when the settle window has reopened —
/// item-boundary preemption, so a disowned tide stops burning the pool.
/// Uncovered targets are the CALLER's business: the scheduler arm falls back
/// to the serial resolve, the republish tide skips them.
pub(crate) fn resolved_parallel_style_wave_targets_with_abort(
    state: &LspShellState,
    document_uris: &[String],
    min_parallel_targets: usize,
    abort: Option<(&std::sync::atomic::AtomicU64, u64)>,
) -> BTreeMap<usize, ResolvedParallelStyleTargetV0> {
    resolved_parallel_style_wave_targets_with_abort_and_sink(
        state,
        document_uris,
        min_parallel_targets,
        abort,
        None,
    )
}

pub(crate) fn resolved_parallel_style_wave_targets_with_abort_and_sink(
    state: &LspShellState,
    document_uris: &[String],
    min_parallel_targets: usize,
    abort: Option<(&std::sync::atomic::AtomicU64, u64)>,
    on_item: Option<ParallelStyleWaveItemSinkV0<'_>>,
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
    let oracle_engaged = crate::disk_cache::disk_diagnostics_cache_oracle_engaged();
    let mut shared_surface: Option<Arc<ParallelStyleWaveSurfaceV0>> = None;
    let mut wave_cache_plan: Option<crate::disk_cache::DiskDiagnosticsCacheWavePlanV0> = None;
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
        match shared_surface.as_ref() {
            None => {
                let surface = Arc::new(surface);
                shared_surface = Some(Arc::clone(&surface));
                // The verification plan is per-wave work: ONE environment
                // fingerprint + ONE content-hash pass over the corpus,
                // shared by every target's load and store (stage 1 paid an
                // O(corpus) serialize+hash per target here).
                wave_cache_plan = crate::disk_cache::disk_diagnostics_cache_wave_plan_v1(
                    &crate::disk_cache::DiskDiagnosticsCacheEnvironmentComponentsV1 {
                        style_sources: surface.style_sources.as_slice(),
                        source_documents: surface.source_documents.as_slice(),
                        package_manifests,
                        external_sifs,
                        resolution_inputs: &surface.resolution_inputs,
                        severity: configured_severity,
                        deep_analysis,
                    },
                );
            }
            Some(shared) if **shared == surface => {}
            Some(_) => continue,
        }
        let disk_cache_slot = wave_cache_plan.as_ref().and_then(|plan| {
            disk_diagnostics_cache_slot_for_resolve(
                state,
                document.workspace_folder_uri.as_deref(),
                document.uri.as_str(),
                plan,
            )
        });
        let mut oracle_cached_diagnostics = None;
        if let Some(cached_diagnostics) = disk_cache_slot.as_ref().and_then(|slot| slot.load()) {
            if oracle_engaged {
                // Shadow oracle: keep the target in the compute group and
                // byte-compare after the join; serve the computed value.
                oracle_cached_diagnostics = Some(cached_diagnostics);
            } else {
                if let Some(sink) = on_item {
                    sink(index, cached_diagnostics.clone(), None);
                }
                resolved.insert(
                    index,
                    ResolvedParallelStyleTargetV0 {
                        diagnostics: cached_diagnostics,
                        disk_cache_slot: None,
                    },
                );
                continue;
            }
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
            oracle_cached_diagnostics,
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
    let committed_graph = std::sync::Arc::new(sync.committed_graph.clone());
    let resolver_identity_index =
        resolver_identity_index_for_parallel_style_wave(state, shared_surface.as_ref());
    // rfcs#111 C1 slices 1+2: hoist the target-INDEPENDENT work out of the
    // per-target loop — the corpus + diagnostics-substrate clones, the
    // shared pass cores (source-selector usage resolution, the cross-file
    // SCC report), and the hypergraph SCC condensation are identical across
    // every target of a wave, so they are built ONCE here and shared behind
    // Arcs. All arms are byte-identical to the per-target builds (same
    // collectors; parity gates in omena-streaming-ifds and the
    // wave-vs-serial oracle).
    let wave_substrate = {
        let db = OmenaQueryStyleMemoDatabaseV0::from_handle(sync.handle.clone());
        std::sync::Arc::new(prepare_committed_workspace_wave_substrate(
            &db,
            workspace,
            committed_graph.as_ref(),
            Some(resolver_identity_index.as_ref()),
        ))
    };
    let shared_reachability = std::sync::Arc::new(
        crate::streaming_ifds_diagnostics::shared_streaming_reachability_for_lsp(
            &committed_graph.cross_file_summary,
        ),
    );
    // The dependency index the verifying-trace manifests are read from: one
    // build over the committed summary's edges, shared across workers. Each
    // target records its declared read-set into its cache slot post-compute.
    let read_set_index = std::sync::Arc::new(omena_query::reverse_dependency_index_from_edges_v0(
        committed_graph.cross_file_summary.edges.as_slice(),
    ));
    let computed: Vec<Option<(Value, Option<DiskDiagnosticsCacheSlotV0>)>> = pool.install(|| {
        pool_items
            .par_iter()
            .map_with(
                (
                    sync.handle.clone(),
                    committed_graph.clone(),
                    resolver_identity_index.clone(),
                    wave_substrate.clone(),
                    shared_reachability.clone(),
                    read_set_index.clone(),
                ),
                |worker_state, (plan, file)| {
                    if let Some((watch, generation)) = abort
                        && watch.load(std::sync::atomic::Ordering::Relaxed) != generation
                    {
                        return None;
                    }
                    let handle = worker_state.0.clone();
                    let committed_graph = worker_state.1.clone();
                    let resolver_identity_index = worker_state.2.clone();
                    let wave_substrate = worker_state.3.clone();
                    let shared_reachability = worker_state.4.clone();
                    let read_set_index = worker_state.5.clone();
                    // A worker panic must not abort the wave: the target drops
                    // out of the result map and the loop recomputes it serially,
                    // panicking exactly where the serial arm would.
                    let value = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        let db = OmenaQueryStyleMemoDatabaseV0::from_handle(handle);
                        let summary =
                            resolve_committed_workspace_style_diagnostics_from_view_with_identity_index_and_wave_substrate(
                            &db,
                            workspace,
                            *file,
                            wave_substrate.as_ref(),
                            resolver_identity_index.as_ref(),
                        );
                        finish_style_diagnostics_value_with_shared_reachability(
                            &LspStyleDiagnosticsRenderInputsV0 {
                                document_uri: plan.target_uri.as_str(),
                                document_text: plan.document_text.as_str(),
                                query_candidates: plan.query_candidates.as_slice(),
                                deep_analysis,
                                configured_severity,
                            },
                            summary,
                            Some(&committed_graph.cross_file_summary),
                            Some(shared_reachability.as_ref()),
                        )
                    }))
                    .ok();
                    // Attach the declared read-set to the write-behind slot:
                    // the shard records exactly what this compute could read
                    // over the committed edge graph.
                    let slot_with_read_set = value.as_ref().and_then(|_| {
                        plan.disk_cache_slot.clone().map(|mut slot| {
                            slot.set_read_set_paths(
                                omena_query::diagnostics_read_set_for_target_v0(
                                    read_set_index.as_ref(),
                                    plan.target_uri.as_str(),
                                ),
                            );
                            slot
                        })
                    });
                    if let (Some(sink), Some(value)) = (on_item, value.as_ref()) {
                        sink(plan.index, value.clone(), slot_with_read_set.clone());
                    }
                    value.map(|value| (value, slot_with_read_set))
                },
            )
            .collect()
    });
    // Order-preserving join happened above; drop the pool threads and the
    // handle before anything else — the next loop turn's set_* depends on it.
    drop(pool);
    drop(sync);

    for ((plan, _), computed_target) in pool_items.into_iter().zip(computed) {
        if let Some((diagnostics, disk_cache_slot)) = computed_target {
            if let Some(cached) = plan.oracle_cached_diagnostics.as_ref() {
                crate::disk_cache::record_disk_diagnostics_cache_oracle_outcome(
                    plan.target_uri.as_str(),
                    cached == &diagnostics,
                );
            }
            resolved.insert(
                plan.index,
                ResolvedParallelStyleTargetV0 {
                    diagnostics,
                    disk_cache_slot,
                },
            );
        }
    }
    resolved
}
