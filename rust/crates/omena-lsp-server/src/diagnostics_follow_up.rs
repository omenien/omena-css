use crate::diagnostics_scheduler;
use crate::lsp_output::{LspDeferredDiagnosticsDispatchV0, ScheduledLspOutput};
use crate::protocol::is_style_document_uri;
use crate::{LspDocumentOrigin, LspExternalSifRefreshResultV0, LspShellState};

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
    if crate::apply_deferred_external_sif_refresh_result(state, result) {
        external_sif_refresh_follow_up_diagnostics_effects(state)
    } else {
        LspDiagnosticsFollowUpEffectsV0::default()
    }
}
