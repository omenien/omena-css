//! Worker-side deferred diagnostics compute: turn a dispatched snapshot
//! into the full publish notification, carrying the reverse-dependency
//! refresh the selector build produced as a byproduct. Split from
//! `style_diagnostics` so the serial-arm pipeline file stays within the
//! god-file ceiling; the render tail (`finish_style_diagnostics_value`)
//! stays there and is shared byte-for-byte.

use super::*;
use crate::style_diagnostics::owned_style_diagnostics_render_inputs_for_uri;

impl LspOwnedStyleDiagnosticsRenderInputsV0 {
    pub(crate) fn borrowed(&self) -> LspStyleDiagnosticsRenderInputsV0<'_> {
        LspStyleDiagnosticsRenderInputsV0 {
            document_uri: self.document_uri.as_str(),
            document_text: self.document_text.as_str(),
            query_candidates: self.query_candidates.as_slice(),
            snapshot_id: self.snapshot_id,
            deep_analysis: self.deep_analysis,
            configured_severity: self.configured_severity,
        }
    }
}

pub fn resolve_deferred_diagnostics_notification(
    host: &mut omena_query::OmenaQueryStyleMemoHostV0,
    dispatch: &LspDeferredDiagnosticsDispatchV0,
) -> Value {
    resolve_deferred_diagnostics_notification_with_reverse_refresh(host, dispatch).0
}

/// The worker-side deferred compute, additionally returning the
/// reverse-dependency refresh its selector build produced — the loop
/// applies it from the completion channel, which is what keeps the
/// loop-side memo fresh WITHOUT the loop ever building a selector.
pub fn resolve_deferred_diagnostics_notification_with_reverse_refresh(
    host: &mut omena_query::OmenaQueryStyleMemoHostV0,
    dispatch: &LspDeferredDiagnosticsDispatchV0,
) -> (
    Value,
    Option<crate::lsp_output::LspReverseDependencyRefreshV0>,
) {
    let mut reverse_refresh = None;
    let diagnostics = match &dispatch.render_inputs {
        DeferredDiagnosticsRenderInputsV0::StyleSnapshot(snapshot) => {
            let Some(inputs) = owned_style_diagnostics_render_inputs_for_uri(
                snapshot.shell_state(),
                &dispatch.uri,
            ) else {
                return (
                    diagnostics_scheduler::deferred_full_diagnostics_notification(
                        dispatch.uri.as_str(),
                        json!([]),
                        dispatch.tier_plan,
                    ),
                    None,
                );
            };
            let (workspace_summary, committed_cross_file_summary, snapshot_id) = host
                .workspace_style_diagnostics_with_selector(
                    inputs.document_uri.as_str(),
                    inputs.style_sources.as_slice(),
                    inputs.source_documents.as_slice(),
                    inputs.package_manifests.as_slice(),
                    inputs.external_sifs.as_slice(),
                    &inputs.resolution_inputs,
                )
                .map(|resolved| {
                    let snapshot_id = resolved.snapshot_id();
                    reverse_refresh = Some(crate::lsp_output::LspReverseDependencyRefreshV0 {
                        revision: resolved.selector.revision().value,
                        ledger_epoch: dispatch.ledger_epoch,
                        summary: resolved.selector.workspace_cross_file_summary().clone(),
                    });
                    (
                        Some(resolved.diagnostics),
                        Some(resolved.selector.workspace_cross_file_summary().clone()),
                        Some(snapshot_id),
                    )
                })
                .unwrap_or((None, None, None));
            let render_inputs = LspStyleDiagnosticsRenderInputsV0 {
                snapshot_id: dispatch.workspace_snapshot_id.or(snapshot_id),
                ..inputs.borrowed()
            };
            crate::style_diagnostics::finish_style_diagnostics_value(
                &render_inputs,
                workspace_summary,
                committed_cross_file_summary.as_ref(),
            )
        }
        DeferredDiagnosticsRenderInputsV0::Source(inputs) => {
            finish_source_diagnostics_value(&inputs.borrowed())
        }
    };
    (
        diagnostics_scheduler::deferred_full_diagnostics_notification(
            dispatch.uri.as_str(),
            diagnostics,
            dispatch.tier_plan,
        ),
        reverse_refresh,
    )
}
