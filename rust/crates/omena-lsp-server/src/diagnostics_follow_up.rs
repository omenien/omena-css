use crate::diagnostics_scheduler;
use crate::lsp_output::{LspDeferredDiagnosticsDispatchV0, ScheduledLspOutput};
use crate::protocol::is_style_document_uri;
use crate::tide::{TideGateInputsV0, TideLaneConfigV0, TideRepublishDemandV0};
use crate::{LspDocumentOrigin, LspExternalSifRefreshResultV0, LspShellState};
use std::collections::BTreeSet;

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
    // The reference arm for runtime-loop probes and tests: full-workspace
    // coverage through the SAME target resolution the demand-shaped flush
    // uses, so the two can never drift apart.
    let uris = tide_republish_target_uris(state, &TideRepublishDemandV0::All);
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

/// Resolve a flushed republish demand into concrete target uris — open
/// documents first (the user is looking at them), then the rest in
/// canonical order. `All` covers every admitted local style document;
/// `Cone(seeds)` covers the seeds plus their reverse-dependency closure
/// against the committed graph AT FLUSH TIME. A cone that cannot be
/// resolved (no committed scope yet) widens to the workspace — never the
/// other direction.
pub(crate) fn tide_republish_target_uris(
    state: &LspShellState,
    demand: &TideRepublishDemandV0,
) -> Vec<String> {
    let cone = match demand {
        TideRepublishDemandV0::None => return Vec::new(),
        TideRepublishDemandV0::All => None,
        TideRepublishDemandV0::Cone(seeds) => match republish_cone_paths(state, seeds) {
            Some(cone) => Some(cone),
            None => {
                crate::loop_trace!(
                    "republish-cone widened to all: {} seeds, no committed scope",
                    seeds.len()
                );
                None
            }
        },
    };
    let mut open = Vec::new();
    let mut unopened = Vec::new();
    for document in state.documents.values() {
        if document.origin != LspDocumentOrigin::Local
            || !is_style_document_uri(document.uri.as_str())
        {
            continue;
        }
        if let Some(cone) = cone.as_ref()
            && !diagnostics_scheduler::file_uri_set_contains_equivalent(cone, document.uri.as_str())
        {
            continue;
        }
        if state.has_open_document_uri(document.uri.as_str()) {
            open.push(document.uri.clone());
        } else {
            unopened.push(document.uri.clone());
        }
    }
    open.extend(unopened);
    open
}

/// The seeds' reverse-dependency closure (seeds included) over the
/// committed cross-file summary, or `None` when no committed scope exists.
fn republish_cone_paths(
    state: &LspShellState,
    seeds: &BTreeSet<String>,
) -> Option<BTreeSet<String>> {
    diagnostics_scheduler::with_reverse_dependency_index(state, |index| {
        let mut cone =
            diagnostics_scheduler::reverse_dependency_closure_for_lsp_paths(index, seeds);
        cone.extend(seeds.iter().cloned());
        cone
    })
}

/// Evaluate the workspace-republish settle gate; on flush, run the follow-up
/// executor over the flushed demand's targets. M2 keeps the per-file
/// deferred executor verbatim and completes the tide at flush time — the
/// executor swap to the off-loop generation-checked wave is exclusively M3
/// (rfcs#111 §12).
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
    let uris = tide_republish_target_uris(state, &flush.demand);
    let effects = if uris.is_empty() {
        LspDiagnosticsFollowUpEffectsV0::default()
    } else {
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
    };
    state.tide_republish_lane.tide_completed(flush.generation);
    effects
}
