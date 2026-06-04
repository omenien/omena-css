use crate::{
    LspShellState, OPTIMIZING_DIAGNOSTICS_DELAY_MS, ScheduledLspOutput,
    is_resolution_config_document_uri, is_style_document_uri, resolve_document_diagnostics_for_uri,
    resolve_source_diagnostics_for_uri, resolve_workspace_folder_uri, workspace_folder_compatible,
};
use omena_query::{summarize_omena_query_analyzed_graph, summarize_omena_query_fast_facts};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RustDiagnosticsSchedulerBoundaryV0 {
    pub product: &'static str,
    pub owner: &'static str,
    pub scheduling_model: &'static str,
    pub event_policy: Vec<&'static str>,
    pub request_path_policy: Vec<&'static str>,
}

pub fn rust_diagnostics_scheduler_contract() -> RustDiagnosticsSchedulerBoundaryV0 {
    RustDiagnosticsSchedulerBoundaryV0 {
        product: "omena-lsp-server.diagnostics-scheduler",
        owner: "omena-lsp-server/diagnosticsScheduler",
        scheduling_model: "deterministicNotificationPlanner",
        event_policy: vec![
            "publishOnOpenChangeClose",
            "refreshSourceDiagnosticsForStyleChanges",
            "dedupeWatchedStyleDiagnostics",
            "refreshSourceDiagnosticsForResolutionConfigChanges",
            "refreshOpenDocumentsOnConfigurationChange",
            "refreshOpenDocumentsAfterWorkspaceIndexing",
            "publishBaselineDiagnosticsBeforeOptimizingDiagnostics",
            "deferOptimizingDiagnosticsOnRustPath",
        ],
        request_path_policy: vec![
            "noNodeDiagnosticsSchedulerOnRustLspPath",
            "diagnosticsNotificationsStayRustOwned",
            "closedDocumentsPublishEmptyDiagnostics",
            "baselineTierUsesFastFactsV0ForStyleDocuments",
            "optimizingTierUsesAnalyzedGraphV0ForStyleDocuments",
            "optimizingDiagnosticsUseRustScheduledOutputDelay",
        ],
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DiagnosticsScheduleEvent {
    TextDocument { uri: String, is_close: bool },
    WatchedFiles { uris: Vec<String> },
    ConfigurationChanged,
    Initialized,
}

pub(crate) fn diagnostics_schedule_event(
    method: Option<&str>,
    document_uri: Option<String>,
    watched_file_uris: Vec<String>,
) -> Option<DiagnosticsScheduleEvent> {
    match method {
        Some("textDocument/didOpen" | "textDocument/didChange" | "textDocument/didClose") => {
            document_uri.map(|uri| DiagnosticsScheduleEvent::TextDocument {
                uri,
                is_close: method == Some("textDocument/didClose"),
            })
        }
        Some("workspace/didChangeWatchedFiles") => Some(DiagnosticsScheduleEvent::WatchedFiles {
            uris: watched_file_uris,
        }),
        Some("workspace/didChangeConfiguration") => {
            Some(DiagnosticsScheduleEvent::ConfigurationChanged)
        }
        Some("initialized") => Some(DiagnosticsScheduleEvent::Initialized),
        _ => None,
    }
}

pub(crate) fn run_diagnostics_schedule(
    state: &LspShellState,
    event: DiagnosticsScheduleEvent,
) -> Vec<ScheduledLspOutput> {
    match event {
        DiagnosticsScheduleEvent::TextDocument { uri, is_close } => {
            diagnostics_for_text_document_event(state, uri.as_str(), is_close)
        }
        DiagnosticsScheduleEvent::WatchedFiles { uris } => {
            diagnostics_for_watched_files(state, uris)
        }
        DiagnosticsScheduleEvent::ConfigurationChanged | DiagnosticsScheduleEvent::Initialized => {
            diagnostics_for_open_documents(state)
        }
    }
}

fn diagnostics_for_text_document_event(
    state: &LspShellState,
    uri: &str,
    is_close: bool,
) -> Vec<ScheduledLspOutput> {
    let mut outputs = if is_close {
        vec![ScheduledLspOutput::immediate(
            publish_diagnostics_notification(uri, json!([])),
        )]
    } else {
        publish_tiered_diagnostics_notifications(
            state,
            uri,
            resolve_document_diagnostics_for_uri(state, uri),
        )
    };

    if is_style_document_uri(uri) {
        for source_uri in source_uris_for_text_style_change_diagnostics(state, uri) {
            outputs.extend(publish_tiered_diagnostics_notifications(
                state,
                source_uri.as_str(),
                resolve_source_diagnostics_for_uri(state, source_uri.as_str()),
            ));
        }
    }

    outputs
}

fn diagnostics_for_watched_files(
    state: &LspShellState,
    uris: Vec<String>,
) -> Vec<ScheduledLspOutput> {
    let mut outputs = Vec::new();
    let mut style_uris_to_refresh = BTreeSet::new();
    let mut config_uris_to_refresh = BTreeSet::new();
    let mut document_uris_to_refresh = BTreeSet::new();
    for uri in uris {
        if is_style_document_uri(uri.as_str()) {
            style_uris_to_refresh.insert(uri);
        } else if is_resolution_config_document_uri(uri.as_str()) {
            config_uris_to_refresh.insert(uri);
        }
    }
    for uri in style_uris_to_refresh {
        document_uris_to_refresh.insert(uri.clone());
        for source_uri in source_uris_for_style_change_diagnostics(state, uri.as_str()) {
            document_uris_to_refresh.insert(source_uri);
        }
    }
    for uri in config_uris_to_refresh {
        for document_uri in
            document_uris_for_resolution_config_change_diagnostics(state, uri.as_str())
        {
            document_uris_to_refresh.insert(document_uri);
        }
    }
    for document_uri in document_uris_to_refresh {
        outputs.extend(publish_tiered_diagnostics_notifications(
            state,
            document_uri.as_str(),
            resolve_document_diagnostics_for_uri(state, document_uri.as_str()),
        ));
    }
    outputs
}

fn diagnostics_for_open_documents(state: &LspShellState) -> Vec<ScheduledLspOutput> {
    open_document_uris_for_diagnostics(state)
        .into_iter()
        .flat_map(|uri| {
            publish_tiered_diagnostics_notifications(
                state,
                uri.as_str(),
                resolve_document_diagnostics_for_uri(state, uri.as_str()),
            )
        })
        .collect()
}

fn open_document_uris_for_diagnostics(state: &LspShellState) -> Vec<String> {
    state
        .open_document_uris
        .iter()
        .filter_map(|uri| {
            state
                .document(uri.as_str())
                .map(|document| document.uri.clone())
        })
        .collect()
}

fn source_uris_for_text_style_change_diagnostics(
    state: &LspShellState,
    style_uri: &str,
) -> Vec<String> {
    state
        .documents
        .values()
        .filter(|document| !is_style_document_uri(document.uri.as_str()))
        .filter(|document| {
            state.document(style_uri).is_none_or(|style_document| {
                workspace_folder_compatible(
                    style_document.workspace_folder_uri.as_deref(),
                    document,
                )
            })
        })
        .map(|document| document.uri.clone())
        .collect()
}

fn source_uris_for_style_change_diagnostics(state: &LspShellState, style_uri: &str) -> Vec<String> {
    let workspace_folder_uri = state
        .document(style_uri)
        .and_then(|document| document.workspace_folder_uri.clone())
        .or_else(|| resolve_workspace_folder_uri(state, style_uri));
    state
        .documents
        .values()
        .filter(|document| !is_style_document_uri(document.uri.as_str()))
        .filter(|document| {
            workspace_folder_uri.as_deref().is_none_or(|workspace_uri| {
                workspace_folder_compatible(Some(workspace_uri), document)
            })
        })
        .map(|document| document.uri.clone())
        .collect()
}

fn document_uris_for_resolution_config_change_diagnostics(
    state: &LspShellState,
    config_uri: &str,
) -> Vec<String> {
    let workspace_folder_uri = resolve_workspace_folder_uri(state, config_uri);
    state
        .documents
        .values()
        .filter(|document| {
            workspace_folder_uri.as_deref().is_none_or(|workspace_uri| {
                workspace_folder_compatible(Some(workspace_uri), document)
            })
        })
        .map(|document| document.uri.clone())
        .collect()
}

fn publish_diagnostics_notification(uri: &str, diagnostics: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {
            "uri": uri,
            "diagnostics": diagnostics,
        },
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiagnosticsPipelineTier {
    Baseline,
    Optimizing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DiagnosticsPipelineTierPlan {
    baseline_evidence: &'static str,
    optimizing_evidence: &'static str,
}

fn publish_tiered_diagnostics_notifications(
    state: &LspShellState,
    uri: &str,
    diagnostics: Value,
) -> Vec<ScheduledLspOutput> {
    let Some(diagnostics) = diagnostics.as_array() else {
        return vec![ScheduledLspOutput::immediate(
            publish_diagnostics_notification(uri, diagnostics),
        )];
    };
    let tier_plan = diagnostics_pipeline_tier_plan_for_uri(state, uri);
    let baseline_diagnostics = diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic_pipeline_tier(diagnostic) == DiagnosticsPipelineTier::Baseline
        })
        .map(|diagnostic| {
            annotate_diagnostic_pipeline_tier(
                diagnostic.clone(),
                DiagnosticsPipelineTier::Baseline,
                tier_plan.baseline_evidence,
            )
        })
        .collect::<Vec<_>>();
    let full_diagnostics = diagnostics
        .iter()
        .map(|diagnostic| {
            let tier = diagnostic_pipeline_tier(diagnostic);
            let evidence = match tier {
                DiagnosticsPipelineTier::Baseline => tier_plan.baseline_evidence,
                DiagnosticsPipelineTier::Optimizing => tier_plan.optimizing_evidence,
            };
            annotate_diagnostic_pipeline_tier(diagnostic.clone(), tier, evidence)
        })
        .collect::<Vec<_>>();

    let mut outputs = vec![ScheduledLspOutput::immediate(
        publish_diagnostics_notification(uri, json!(baseline_diagnostics)),
    )];
    if full_diagnostics != baseline_diagnostics {
        outputs.push(ScheduledLspOutput::delayed(
            publish_diagnostics_notification(uri, json!(full_diagnostics)),
            OPTIMIZING_DIAGNOSTICS_DELAY_MS,
        ));
    }
    outputs
}

fn diagnostics_pipeline_tier_plan_for_uri(
    state: &LspShellState,
    uri: &str,
) -> DiagnosticsPipelineTierPlan {
    if is_style_document_uri(uri)
        && let Some(document) = state.document(uri)
    {
        let fast_facts =
            summarize_omena_query_fast_facts(document.uri.as_str(), document.text.as_str());
        let analyzed_graph =
            summarize_omena_query_analyzed_graph(document.uri.as_str(), document.text.as_str());
        return DiagnosticsPipelineTierPlan {
            baseline_evidence: fast_facts.tier,
            optimizing_evidence: analyzed_graph.tier,
        };
    }

    DiagnosticsPipelineTierPlan {
        baseline_evidence: "sourceSyntaxIndexV0",
        optimizing_evidence: "workspaceSourceDiagnosticsV0",
    }
}

fn diagnostic_pipeline_tier(diagnostic: &Value) -> DiagnosticsPipelineTier {
    match diagnostic
        .get("code")
        .and_then(Value::as_str)
        .unwrap_or_default()
    {
        "missingStaticClass"
        | "missingTemplatePrefix"
        | "missingResolvedClassValues"
        | "missingResolvedClassDomain"
        | "missingSelector"
        | "missingModule"
        | "missingCustomProperty" => DiagnosticsPipelineTier::Baseline,
        _ => DiagnosticsPipelineTier::Optimizing,
    }
}

fn annotate_diagnostic_pipeline_tier(
    mut diagnostic: Value,
    tier: DiagnosticsPipelineTier,
    tier_evidence: &'static str,
) -> Value {
    let Some(diagnostic) = diagnostic.as_object_mut() else {
        return diagnostic;
    };
    let data = diagnostic
        .entry("data")
        .or_insert_with(|| json!({}))
        .as_object_mut();
    if let Some(data) = data {
        data.insert(
            "pipelineTier".to_string(),
            json!(match tier {
                DiagnosticsPipelineTier::Baseline => "baseline",
                DiagnosticsPipelineTier::Optimizing => "optimizing",
            }),
        );
        data.insert("pipelineTierEvidence".to_string(), json!(tier_evidence));
    }
    Value::Object(diagnostic.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_lsp_events_to_diagnostics_schedule_events() {
        assert_eq!(
            diagnostics_schedule_event(
                Some("textDocument/didChange"),
                Some("file:///repo/src/App.tsx".to_string()),
                Vec::new(),
            ),
            Some(DiagnosticsScheduleEvent::TextDocument {
                uri: "file:///repo/src/App.tsx".to_string(),
                is_close: false,
            }),
        );
        assert_eq!(
            diagnostics_schedule_event(
                Some("textDocument/didClose"),
                Some("file:///repo/src/App.tsx".to_string()),
                Vec::new(),
            ),
            Some(DiagnosticsScheduleEvent::TextDocument {
                uri: "file:///repo/src/App.tsx".to_string(),
                is_close: true,
            }),
        );
        assert_eq!(
            diagnostics_schedule_event(
                Some("workspace/didChangeWatchedFiles"),
                None,
                vec!["file:///repo/src/App.module.scss".to_string()],
            ),
            Some(DiagnosticsScheduleEvent::WatchedFiles {
                uris: vec!["file:///repo/src/App.module.scss".to_string()],
            }),
        );
    }

    #[test]
    fn publishes_baseline_before_optimizing_diagnostics() {
        let mut state = LspShellState::default();
        let outputs = crate::handle_lsp_message_scheduled_outputs(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": "file:///workspace-a/src/App.module.scss",
                        "languageId": "scss",
                        "version": 1,
                        "text": ":root { --brand: red; }\n.btn { width: var(--missing); color: red; color: blue; }",
                    },
                },
            }),
        );

        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs[0].delay_millis, None);
        assert_eq!(
            outputs[1].delay_millis,
            Some(OPTIMIZING_DIAGNOSTICS_DELAY_MS)
        );
        assert_eq!(
            outputs[0].value.pointer("/params/diagnostics/0/code"),
            Some(&json!("missingCustomProperty")),
        );
        assert_eq!(
            outputs[0]
                .value
                .pointer("/params/diagnostics/0/data/pipelineTier"),
            Some(&json!("baseline")),
        );
        assert_eq!(
            outputs[0]
                .value
                .pointer("/params/diagnostics/0/data/pipelineTierEvidence"),
            Some(&json!("fastFactsV0")),
        );
        assert!(
            outputs[0]
                .value
                .pointer("/params/diagnostics")
                .and_then(Value::as_array)
                .is_some_and(|diagnostics| diagnostics
                    .iter()
                    .all(|diagnostic| diagnostic.pointer("/code")
                        != Some(&json!("unreachableDeclaration"))))
        );

        assert!(
            outputs[1]
                .value
                .pointer("/params/diagnostics")
                .and_then(Value::as_array)
                .is_some_and(|full_diagnostics| full_diagnostics.iter().any(
                    |diagnostic| diagnostic.pointer("/code")
                        == Some(&json!("missingCustomProperty"))
                        && diagnostic.pointer("/data/pipelineTier") == Some(&json!("baseline"))
                ))
        );
        assert!(
            outputs[1]
                .value
                .pointer("/params/diagnostics")
                .and_then(Value::as_array)
                .is_some_and(|full_diagnostics| full_diagnostics.iter().any(
                    |diagnostic| diagnostic.pointer("/code")
                        == Some(&json!("unreachableDeclaration"))
                        && diagnostic.pointer("/data/pipelineTier") == Some(&json!("optimizing"))
                        && diagnostic.pointer("/data/pipelineTierEvidence")
                            == Some(&json!("analyzedGraphV0"))
                ))
        );
    }
}
