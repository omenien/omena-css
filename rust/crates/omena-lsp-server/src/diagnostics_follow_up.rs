use crate::diagnostics_scheduler;
use crate::lsp_output::{LspDeferredDiagnosticsDispatchV0, ScheduledLspOutput};
use crate::protocol::is_style_document_uri;
use crate::tide::{TideGateInputsV0, TideLaneConfigV0};
use crate::{LspDocumentOrigin, LspExternalSifRefreshResultV0, LspShellState};

/// The workspace-republish lane: the M3 executor passes real idleness for
/// the courtesy layer; aging (~10s at the 5ms tick) overrides courtesy so a
/// busy editor still converges. The correctness layer is never overridden.
pub(crate) const TIDE_REPUBLISH_LANE_CONFIG: TideLaneConfigV0 = TideLaneConfigV0 {
    aging_bound_ticks: 2_000,
};

#[cfg(test)]
pub(crate) mod warmup_wave_count_probe {
    use std::cell::Cell;

    thread_local! {
        static FOLLOW_UP_WAVE_COUNT: Cell<usize> = const { Cell::new(0) };
    }

    pub(crate) fn record() {
        FOLLOW_UP_WAVE_COUNT.with(|count| count.set(count.get().saturating_add(1)));
    }

    pub(crate) fn reset() {
        FOLLOW_UP_WAVE_COUNT.with(|count| count.set(0));
    }

    pub(crate) fn read() -> usize {
        FOLLOW_UP_WAVE_COUNT.with(Cell::get)
    }
}

#[derive(Debug, Default)]
pub struct LspDiagnosticsFollowUpEffectsV0 {
    pub outputs: Vec<ScheduledLspOutput>,
    pub deferred_diagnostics: Vec<LspDeferredDiagnosticsDispatchV0>,
}

pub fn external_sif_refresh_follow_up_diagnostics_effects(
    state: &mut LspShellState,
) -> LspDiagnosticsFollowUpEffectsV0 {
    let uris = state
        .documents
        .values()
        .filter(|document| {
            document.origin == LspDocumentOrigin::Local
                && is_style_document_uri(document.uri.as_str())
        })
        .map(|document| document.uri.clone())
        .collect::<Vec<_>>();
    if uris.is_empty() {
        return LspDiagnosticsFollowUpEffectsV0::default();
    }
    #[cfg(test)]
    warmup_wave_count_probe::record();
    crate::loop_trace!(
        "sif-follow-up fired for {} style docs (deferred)",
        uris.len()
    );
    let effects = diagnostics_scheduler::run_diagnostics_schedule_effects(
        state,
        diagnostics_scheduler::DiagnosticsScheduleEvent::WatchedFiles { uris },
    );
    LspDiagnosticsFollowUpEffectsV0 {
        outputs: effects.outputs,
        deferred_diagnostics: effects.deferred_diagnostics,
    }
}

pub fn apply_external_sif_refresh_result_follow_up_diagnostics_effects(
    state: &mut LspShellState,
    result: LspExternalSifRefreshResultV0,
) -> LspDiagnosticsFollowUpEffectsV0 {
    crate::apply_deferred_external_sif_refresh_result(state, result);
    tide_workspace_republish_flush_effects(state)
}

/// The republish frontier: the SIF lane is fully settled — no undrained
/// demand, no in-flight tide — and no index chain is mid-flight. A republish
/// computed before that point would present a corpus state that is invalid
/// as a final (rfcs#111 P2).
pub(crate) fn workspace_republish_frontier_passed(state: &LspShellState) -> bool {
    !state.tide_sif_lane.has_demand()
        && !state.tide_sif_lane.in_flight()
        && state.workspace_index_pending_file_count == 0
}

/// Evaluate the workspace-republish settle gate; on flush, run the follow-up
/// executor. M2 keeps the per-file deferred executor verbatim and completes
/// the tide at flush time — the executor swap to the off-loop generation-
/// checked wave is exclusively M3 (rfcs#111 §12).
pub fn tide_workspace_republish_flush_effects(
    state: &mut LspShellState,
) -> LspDiagnosticsFollowUpEffectsV0 {
    let inputs = TideGateInputsV0 {
        frontier_passed: workspace_republish_frontier_passed(state),
        idle: true,
    };
    let Some(flush) =
        state
            .tide_republish_lane
            .try_flush(inputs, state.tide_tick, &TIDE_REPUBLISH_LANE_CONFIG)
    else {
        return LspDiagnosticsFollowUpEffectsV0::default();
    };
    let effects = external_sif_refresh_follow_up_diagnostics_effects(state);
    state.tide_republish_lane.tide_completed(flush.generation);
    effects
}
