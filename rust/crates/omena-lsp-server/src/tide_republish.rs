//! M3: the off-loop workspace-republish executor (rfcs#111 §8.5, §12 M3).
//!
//! `prepare` flushes the republish lane through its settle gate on the loop
//! and captures a copy-on-write query snapshot; `collect` runs the
//! abort-capable parallel wave against that snapshot on a worker thread —
//! the loop stays free; `apply` publishes loop-side in canonical order under
//! the loop's per-tick chunk budget, writing behind through the LOOP state's
//! disk-cache session (the snapshot carries a default session on purpose).
//! A settle-window reopen bumps the lane generation: the wave aborts at the
//! next item boundary and pending applies are dropped — their keys are
//! republished by the reopened window's tide, and the publication order key
//! forbids any stale overwrite.

use crate::diagnostics_follow_up::{
    TIDE_REPUBLISH_LANE_CONFIG, workspace_republish_frontier_passed,
};
use crate::disk_cache::DiskDiagnosticsCacheSlotV0;
use crate::lsp_output::ScheduledLspOutput;
use crate::protocol::is_style_document_uri;
use crate::state::LspQuerySnapshotV0;
use crate::tide::TideGateInputsV0;
use crate::{LspDocumentOrigin, LspShellState};
use serde_json::Value;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug)]
pub struct TideWorkspaceRepublishJobV0 {
    snapshot: LspQuerySnapshotV0,
    uris: Vec<String>,
    pub generation: u64,
    gen_watch: Arc<AtomicU64>,
}

#[derive(Debug)]
pub struct TideWorkspaceRepublishItemV0 {
    pub(crate) uri: String,
    pub(crate) diagnostics: Value,
    pub(crate) disk_cache_slot: Option<DiskDiagnosticsCacheSlotV0>,
}

#[derive(Debug)]
pub struct TideWorkspaceRepublishResultV0 {
    pub generation: u64,
    pub items: Vec<TideWorkspaceRepublishItemV0>,
    /// Wave-ineligible targets of THIS chunk. When the tide is still current
    /// at completion these fall back to the per-file deferred arm; a
    /// disowned tide drops them (the reopened window covers the corpus).
    pub uncovered_uris: Vec<String>,
    /// The last chunk of the tide: the loop completes the tide once this
    /// arrives and the apply queue drains.
    pub final_chunk: bool,
}

/// Gate evaluation + snapshot capture, on the loop. `idle` is the courtesy
/// input (no recent client message); aging overrides it after
/// [`TIDE_REPUBLISH_LANE_CONFIG`]'s bound, the frontier never.
pub fn prepare_tide_workspace_republish_job(
    state: &mut LspShellState,
    idle: bool,
) -> Option<TideWorkspaceRepublishJobV0> {
    if !state.external_sif_refresh_deferred {
        return None;
    }
    let inputs = TideGateInputsV0 {
        frontier_passed: workspace_republish_frontier_passed(state),
        idle,
    };
    let flush = state.tide_republish_lane.try_flush(
        inputs,
        state.tide_tick,
        &TIDE_REPUBLISH_LANE_CONFIG,
    )?;
    state
        .tide_republish_gen_watch
        .store(flush.generation, Ordering::Relaxed);
    // Open documents first — the user is looking at them — then the rest;
    // canonical order within each group. Chunked streaming below turns this
    // ordering into convergence latency for the open files.
    let mut uris: Vec<String> = Vec::new();
    let mut unopened: Vec<String> = Vec::new();
    for document in state.documents.values() {
        if document.origin != LspDocumentOrigin::Local
            || !is_style_document_uri(document.uri.as_str())
        {
            continue;
        }
        if state.has_open_document_uri(document.uri.as_str()) {
            uris.push(document.uri.clone());
        } else {
            unopened.push(document.uri.clone());
        }
    }
    uris.extend(unopened);
    if uris.is_empty() {
        state.tide_republish_lane.tide_completed(flush.generation);
        return None;
    }
    crate::loop_trace!(
        "republish-tide prepared gen={} targets={}",
        flush.generation,
        uris.len()
    );
    Some(TideWorkspaceRepublishJobV0 {
        snapshot: state.query_snapshot(),
        uris,
        generation: flush.generation,
        gen_watch: Arc::clone(&state.tide_republish_gen_watch),
    })
}

/// Worker-side compute, streaming (rfcs#111 §8.5): ONE shared-graph parallel
/// wave — one memo-host sync, one substrate, one condensation — with a
/// per-item sink that emits each target the moment its pool task finishes.
/// Open documents were ordered first by prepare, so they converge first.
/// The generation watch aborts disowned tides at item boundaries; the final
/// event carries the uncovered remainder for the fallback arm.
pub fn collect_tide_workspace_republish_streaming(
    job: TideWorkspaceRepublishJobV0,
    emit: &(dyn Fn(TideWorkspaceRepublishResultV0) -> bool + Sync),
) {
    let covered = std::sync::Mutex::new(std::collections::BTreeSet::<usize>::new());
    let sink =
        |index: usize,
         diagnostics: serde_json::Value,
         disk_cache_slot: Option<crate::disk_cache::DiskDiagnosticsCacheSlotV0>| {
            if let Ok(mut covered) = covered.lock() {
                covered.insert(index);
            }
            let Some(uri) = job.uris.get(index) else {
                return;
            };
            let _ = emit(TideWorkspaceRepublishResultV0 {
                generation: job.generation,
                items: vec![TideWorkspaceRepublishItemV0 {
                    uri: uri.clone(),
                    diagnostics,
                    disk_cache_slot,
                }],
                uncovered_uris: Vec::new(),
                final_chunk: false,
            });
        };
    let _ = crate::parallel_style_wave::resolved_parallel_style_wave_targets_with_abort_and_sink(
        job.snapshot.shell_state(),
        job.uris.as_slice(),
        crate::parallel_style_wave::PARALLEL_STYLE_WAVE_MIN_PARALLEL_TARGETS,
        Some((job.gen_watch.as_ref(), job.generation)),
        Some(&sink),
    );
    let covered = covered.into_inner().unwrap_or_default();
    let uncovered_uris = job
        .uris
        .iter()
        .enumerate()
        .filter(|(index, _)| !covered.contains(index))
        .map(|(_, uri)| uri.clone())
        .collect::<Vec<_>>();
    crate::loop_trace!(
        "republish-tide collected gen={} covered={} uncovered={}",
        job.generation,
        covered.len(),
        uncovered_uris.len()
    );
    let _ = emit(TideWorkspaceRepublishResultV0 {
        generation: job.generation,
        items: Vec::new(),
        uncovered_uris,
        final_chunk: true,
    });
}

/// Loop-side apply for ONE item — the caller pumps a bounded chunk per tick
/// (I4) and must have verified the tide generation is still current.
/// Write-behind runs through the loop state's real disk-cache session, then
/// the tiered publish emits in the same shape as every other arm.
pub fn apply_tide_workspace_republish_item(
    state: &mut LspShellState,
    item: TideWorkspaceRepublishItemV0,
) -> Vec<ScheduledLspOutput> {
    if let Some(slot) = item.disk_cache_slot.as_ref() {
        slot.store_write_behind(state, &item.diagnostics);
    }
    crate::diagnostics_scheduler::publish_tiered_diagnostics_notifications(
        state,
        item.uri.as_str(),
        item.diagnostics,
    )
}

/// Completion: re-arm the lane; when the tide is still current, uncovered
/// targets re-enter the per-file deferred arm so no key is silently skipped.
pub fn complete_tide_workspace_republish(
    state: &mut LspShellState,
    generation: u64,
    uncovered_uris: Vec<String>,
) -> crate::LspDiagnosticsFollowUpEffectsV0 {
    let current = state.tide_republish_lane.generation() == generation;
    state.tide_republish_lane.tide_completed(generation);
    if !current || uncovered_uris.is_empty() {
        return crate::LspDiagnosticsFollowUpEffectsV0::default();
    }
    crate::loop_trace!(
        "republish-tide leftovers gen={} n={}",
        generation,
        uncovered_uris.len()
    );
    let effects = crate::diagnostics_scheduler::run_diagnostics_schedule_effects(
        state,
        crate::diagnostics_scheduler::DiagnosticsScheduleEvent::WatchedFiles {
            uris: uncovered_uris,
        },
    );
    crate::LspDiagnosticsFollowUpEffectsV0 {
        outputs: effects.outputs,
        deferred_diagnostics: effects.deferred_diagnostics,
    }
}
