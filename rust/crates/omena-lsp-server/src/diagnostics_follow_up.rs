use crate::diagnostics_scheduler;
use crate::lsp_output::{LspDeferredDiagnosticsDispatchV0, ScheduledLspOutput};
use crate::protocol::is_style_document_uri;
use crate::{LspDocumentOrigin, LspShellState};

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
    let effects = diagnostics_scheduler::run_diagnostics_schedule_effects(
        state,
        diagnostics_scheduler::DiagnosticsScheduleEvent::WatchedFiles { uris },
    );
    LspDiagnosticsFollowUpEffectsV0 {
        outputs: effects.outputs,
        deferred_diagnostics: effects.deferred_diagnostics,
    }
}
