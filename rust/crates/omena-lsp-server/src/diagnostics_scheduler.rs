use crate::{
    LspOptimizingTierFeedback, LspShellState, OPTIMIZING_DIAGNOSTICS_DELAY_MS, ScheduledLspOutput,
    is_resolution_config_document_uri, is_style_document_uri,
    protocol::{file_uri_equivalent, file_uri_to_path},
    resolution_inputs_for_workspace_uri, resolve_document_diagnostics_for_uri,
    resolve_source_diagnostics_for_uri, resolve_workspace_folder_uri, workspace_folder_compatible,
};
use omena_query::{
    resolve_omena_query_style_uri_for_specifier_with_resolution_inputs,
    summarize_omena_query_analyzed_graph, summarize_omena_query_sass_module_sources,
};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{BTreeSet, VecDeque};
use std::fs;

/// rfcs#61 FIX-1: per peer-recompute walk, the maximum number of style files whose
/// import edges are expanded while deciding whether an open style document
/// (transitively) imports the changed one. Mirrors the workspace-index budgeting
/// philosophy so a pathological import graph cannot stall the loop.
const STYLE_PEER_DISK_WALK_MAX_FILES: usize = 64;

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
            "refreshOpenStyleImporterDiagnosticsForStyleChanges",
            "dedupeWatchedStyleDiagnostics",
            "refreshSourceDiagnosticsForResolutionConfigChanges",
            "refreshOpenDocumentsOnConfigurationChange",
            "refreshOpenDocumentsAfterWorkspaceIndexing",
            "publishBaselineDiagnosticsBeforeOptimizingDiagnostics",
            "deferOptimizingDiagnosticsOnRustPath",
            "coalesceStaleOptimizingDiagnosticsByDocument",
            "tierUpHotStyleDiagnosticsIntoAnalyzedGraphFeedback",
        ],
        request_path_policy: vec![
            "noNodeDiagnosticsSchedulerOnRustLspPath",
            "diagnosticsNotificationsStayRustOwned",
            "closedDocumentsPublishEmptyDiagnostics",
            "baselineTierUsesFastFactsV0ForStyleDocuments",
            "optimizingTierUsesAnalyzedGraphV0ForStyleDocuments",
            "optimizingDiagnosticsUseRustScheduledOutputDelay",
            "delayedOptimizingDiagnosticsUseLatestDocumentGeneration",
            "baselineDiagnosticsConsumeOptimizingTierFeedback",
            "hoverCompletionProvidersConsumeOptimizingTierFeedback",
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
    state: &mut LspShellState,
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
    state: &mut LspShellState,
    uri: &str,
    is_close: bool,
) -> Vec<ScheduledLspOutput> {
    let mut outputs = if is_close {
        vec![publish_immediate_diagnostics_output(uri, json!([]))]
    } else {
        let diagnostics = resolve_document_diagnostics_for_uri(state, uri);
        publish_tiered_diagnostics_notifications(state, uri, diagnostics)
    };

    if is_style_document_uri(uri) {
        for source_uri in source_uris_for_text_style_change_diagnostics(state, uri) {
            let diagnostics = resolve_source_diagnostics_for_uri(state, source_uri.as_str());
            outputs.extend(publish_tiered_diagnostics_notifications(
                state,
                source_uri.as_str(),
                diagnostics,
            ));
        }
        for peer_uri in style_uris_for_style_peer_change_diagnostics(state, uri) {
            let diagnostics = resolve_document_diagnostics_for_uri(state, peer_uri.as_str());
            outputs.extend(publish_tiered_diagnostics_notifications(
                state,
                peer_uri.as_str(),
                diagnostics,
            ));
        }
    }

    outputs
}

fn diagnostics_for_watched_files(
    state: &mut LspShellState,
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
        for peer_uri in style_uris_for_style_peer_change_diagnostics(state, uri.as_str()) {
            document_uris_to_refresh.insert(peer_uri);
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
        let diagnostics = resolve_document_diagnostics_for_uri(state, document_uri.as_str());
        outputs.extend(publish_tiered_diagnostics_notifications(
            state,
            document_uri.as_str(),
            diagnostics,
        ));
    }
    outputs
}

fn diagnostics_for_open_documents(state: &mut LspShellState) -> Vec<ScheduledLspOutput> {
    let mut outputs = Vec::new();
    for uri in open_document_uris_for_diagnostics(state) {
        let diagnostics = resolve_document_diagnostics_for_uri(state, uri.as_str());
        outputs.extend(publish_tiered_diagnostics_notifications(
            state,
            uri.as_str(),
            diagnostics,
        ));
    }
    outputs
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

/// rfcs#61 FIX-1: open style documents whose transitive `@use`/`@forward`/`@import`
/// closure reaches `changed_style_uri`. The closure is resolved over DISK with the
/// same specifier resolver navigation uses (alias-aware, existence-checked), NOT the
/// in-memory open-document graph — an intermediate partial that is not open (or fell
/// outside the indexing budget) must not break the importer chain, otherwise a stale
/// diagnostic on the importer survives edits to its dependency.
fn style_uris_for_style_peer_change_diagnostics(
    state: &LspShellState,
    changed_style_uri: &str,
) -> Vec<String> {
    state
        .open_document_uris
        .iter()
        .filter(|uri| is_style_document_uri(uri.as_str()))
        .filter(|uri| !file_uri_equivalent(uri.as_str(), changed_style_uri))
        .filter(|uri| state.document(uri.as_str()).is_some())
        .filter(|uri| style_disk_import_closure_reaches(state, uri.as_str(), changed_style_uri))
        .cloned()
        .collect()
}

/// Walks `from_uri`'s Sass import closure breadth-first, reading not-open
/// intermediates from disk, and reports whether `needle_uri` is reachable.
fn style_disk_import_closure_reaches(
    state: &LspShellState,
    from_uri: &str,
    needle_uri: &str,
) -> bool {
    let workspace_folder_uri = state
        .document(from_uri)
        .and_then(|document| document.workspace_folder_uri.clone())
        .or_else(|| resolve_workspace_folder_uri(state, from_uri));
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, workspace_folder_uri.as_deref());
    let mut visited = BTreeSet::from([from_uri.to_string()]);
    let mut queue = VecDeque::from([from_uri.to_string()]);
    let mut remaining_files = STYLE_PEER_DISK_WALK_MAX_FILES;

    while let Some(uri) = queue.pop_front() {
        if remaining_files == 0 {
            return false;
        }
        remaining_files -= 1;
        let text = match state.document(uri.as_str()) {
            Some(document) => document.text.clone(),
            None => {
                let Some(path) = file_uri_to_path(uri.as_str()) else {
                    continue;
                };
                let Ok(text) = fs::read_to_string(path) else {
                    continue;
                };
                text
            }
        };
        let Some(sources) = summarize_omena_query_sass_module_sources(uri.as_str(), text.as_str())
        else {
            continue;
        };
        let specifiers = sources
            .module_use_edges
            .iter()
            .map(|edge| edge.source.clone())
            .chain(sources.module_forward_sources.iter().cloned());
        for specifier in specifiers {
            let Some(resolved) = resolve_omena_query_style_uri_for_specifier_with_resolution_inputs(
                uri.as_str(),
                workspace_folder_uri.as_deref(),
                specifier.as_str(),
                &resolution_inputs,
            ) else {
                continue;
            };
            if file_uri_equivalent(resolved.as_str(), needle_uri) {
                return true;
            }
            if visited.insert(resolved.clone()) {
                queue.push_back(resolved);
            }
        }
    }
    false
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
    baseline_feedback_evidence: Option<&'static str>,
}

fn publish_tiered_diagnostics_notifications(
    state: &mut LspShellState,
    uri: &str,
    diagnostics: Value,
) -> Vec<ScheduledLspOutput> {
    let Some(diagnostics) = diagnostics.as_array() else {
        return vec![publish_immediate_diagnostics_output(uri, diagnostics)];
    };
    record_diagnostics_schedule(state, uri);
    prewarm_optimizing_tier_feedback_for_hot_style_document(state, uri);
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
                tier_plan.baseline_feedback_evidence,
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
            annotate_diagnostic_pipeline_tier(
                diagnostic.clone(),
                tier,
                evidence,
                tier_plan.baseline_feedback_evidence,
            )
        })
        .collect::<Vec<_>>();

    let mut outputs = vec![publish_immediate_diagnostics_output(
        uri,
        json!(baseline_diagnostics),
    )];
    if full_diagnostics != baseline_diagnostics {
        outputs.push(ScheduledLspOutput::delayed_coalesced(
            publish_diagnostics_notification(uri, json!(full_diagnostics)),
            OPTIMIZING_DIAGNOSTICS_DELAY_MS,
            diagnostics_coalesce_key(uri),
        ));
    }
    outputs
}

fn publish_immediate_diagnostics_output(uri: &str, diagnostics: Value) -> ScheduledLspOutput {
    ScheduledLspOutput::immediate_coalesced(
        publish_diagnostics_notification(uri, diagnostics),
        diagnostics_coalesce_key(uri),
    )
}

fn diagnostics_coalesce_key(uri: &str) -> String {
    format!("textDocument/publishDiagnostics:{uri}")
}

fn record_diagnostics_schedule(state: &mut LspShellState, uri: &str) {
    if let Some(document) = state.document_mut(uri) {
        document.diagnostics_schedule_count = document.diagnostics_schedule_count.saturating_add(1);
    }
}

fn prewarm_optimizing_tier_feedback_for_hot_style_document(state: &mut LspShellState, uri: &str) {
    if !is_style_document_uri(uri) {
        return;
    }
    let Some(document) = state.document_mut(uri) else {
        return;
    };
    if document.diagnostics_schedule_count < 2 {
        return;
    }
    if document
        .optimizing_tier_feedback
        .as_ref()
        .is_some_and(|feedback| feedback.document_version == document.version)
    {
        return;
    }
    let analyzed_graph =
        summarize_omena_query_analyzed_graph(document.uri.as_str(), document.text.as_str());
    document.optimizing_tier_feedback = Some(LspOptimizingTierFeedback {
        schema_version: "0",
        product: "omena-lsp-server.optimizing-tier-feedback",
        document_version: document.version,
        policy: "hotStyleDiagnosticsPrewarm",
        consumer: "diagnosticsPipelineTierPlanAndProviderRequests",
        analyzed_graph,
    });
}

fn diagnostics_pipeline_tier_plan_for_uri(
    state: &LspShellState,
    uri: &str,
) -> DiagnosticsPipelineTierPlan {
    if is_style_document_uri(uri)
        && let Some(document) = state.document(uri)
    {
        // The version-keyed prewarm feedback already embeds the fast facts, so a cache
        // hit costs zero style-fact collections and a miss runs the analysis exactly
        // once for both tier evidences. The hit path may only consume input-independent
        // fields (the tier labels are constants); consuming data fields (node_count,
        // selector_count, ...) here would make correctness depend on the feedback
        // invalidation set staying exhaustive — re-audit it first.
        let cached_feedback = document
            .optimizing_tier_feedback
            .as_ref()
            .filter(|feedback| feedback.document_version == document.version);
        let (baseline_evidence, optimizing_evidence) = match cached_feedback {
            Some(feedback) => (
                feedback.analyzed_graph.fast_facts.tier,
                feedback.analyzed_graph.tier,
            ),
            None => {
                let analyzed_graph = summarize_omena_query_analyzed_graph(
                    document.uri.as_str(),
                    document.text.as_str(),
                );
                (analyzed_graph.fast_facts.tier, analyzed_graph.tier)
            }
        };
        return DiagnosticsPipelineTierPlan {
            baseline_evidence,
            optimizing_evidence,
            baseline_feedback_evidence: cached_feedback.map(|_| "analyzedGraphV0HotStylePrewarm"),
        };
    }

    DiagnosticsPipelineTierPlan {
        baseline_evidence: "sourceSyntaxIndexV0",
        optimizing_evidence: "workspaceSourceDiagnosticsV0",
        baseline_feedback_evidence: None,
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
    baseline_feedback_evidence: Option<&'static str>,
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
        if tier == DiagnosticsPipelineTier::Baseline
            && let Some(feedback_evidence) = baseline_feedback_evidence
        {
            data.insert("pipelineTierFeedback".to_string(), json!(feedback_evidence));
        }
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

    #[test]
    fn hot_style_diagnostics_prewarm_optimizing_feedback_for_baseline() -> Result<(), &'static str>
    {
        let uri = "file:///workspace-a/src/App.module.scss";
        let mut state = LspShellState::default();
        let first_outputs = crate::handle_lsp_message_scheduled_outputs(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": uri,
                        "languageId": "scss",
                        "version": 1,
                        "text": ":root { --brand: red; }\n.btn { width: var(--missing); color: red; color: blue; }",
                    },
                },
            }),
        );
        assert_eq!(first_outputs.len(), 2);
        assert!(
            first_outputs[0]
                .value
                .pointer("/params/diagnostics/0/data/pipelineTierFeedback")
                .is_none()
        );

        let second_outputs = crate::handle_lsp_message_scheduled_outputs(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didChange",
                "params": {
                    "textDocument": {
                        "uri": uri,
                        "version": 2,
                    },
                    "contentChanges": [{
                        "text": ":root { --brand: red; }\n.btn { width: var(--missing); color: red; color: blue; }\n.card { color: green; }",
                    }],
                },
            }),
        );
        assert_eq!(second_outputs.len(), 2);
        assert_eq!(
            diagnostic_data_value(
                second_outputs[0].value.pointer("/params/diagnostics"),
                "missingCustomProperty",
                "pipelineTierFeedback",
            ),
            Some(&json!("analyzedGraphV0HotStylePrewarm")),
        );
        assert_eq!(
            diagnostic_data_value(
                second_outputs[1].value.pointer("/params/diagnostics"),
                "missingCustomProperty",
                "pipelineTierFeedback",
            ),
            Some(&json!("analyzedGraphV0HotStylePrewarm")),
        );

        let document = state.document(uri).ok_or("style document stays open")?;
        assert_eq!(document.diagnostics_schedule_count, 2);
        let feedback = document
            .optimizing_tier_feedback
            .as_ref()
            .ok_or("hot style diagnostics should prewarm optimizing feedback")?;
        assert_eq!(
            feedback.product,
            "omena-lsp-server.optimizing-tier-feedback"
        );
        assert_eq!(feedback.document_version, 2);
        assert_eq!(feedback.policy, "hotStyleDiagnosticsPrewarm");
        assert_eq!(
            feedback.consumer,
            "diagnosticsPipelineTierPlanAndProviderRequests"
        );
        assert_eq!(feedback.analyzed_graph.tier, "analyzedGraphV0");
        assert_eq!(feedback.analyzed_graph.fast_facts.selector_count, 2);

        let hover_response = crate::handle_lsp_message(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "textDocument/hover",
                "params": {
                    "textDocument": {
                        "uri": uri,
                    },
                    "position": {
                        "line": 2,
                        "character": 2,
                    },
                },
            }),
        );
        assert_eq!(
            hover_response
                .as_ref()
                .and_then(|value| value.pointer("/result/data/providerTierFeedback/provider")),
            Some(&json!("textDocument/hover")),
        );
        assert_eq!(
            hover_response
                .as_ref()
                .and_then(|value| value.pointer("/result/data/providerTierFeedback/feedback")),
            Some(&json!("analyzedGraphV0HotStylePrewarm")),
        );

        let completion_response = crate::handle_lsp_message(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "id": 4,
                "method": "textDocument/completion",
                "params": {
                    "textDocument": {
                        "uri": uri,
                    },
                    "position": {
                        "line": 2,
                        "character": 0,
                    },
                },
            }),
        );
        let completion_items = completion_response
            .as_ref()
            .and_then(|value| value.pointer("/result/items"))
            .and_then(Value::as_array)
            .ok_or("completion items should be present")?;
        let card_completion = completion_items
            .iter()
            .find(|item| item.pointer("/label") == Some(&json!(".card")))
            .ok_or("card selector completion should be present")?;
        assert_eq!(
            card_completion.pointer("/data/providerTierFeedback/provider"),
            Some(&json!("textDocument/completion")),
        );
        assert_eq!(
            card_completion.pointer("/data/providerTierFeedback/feedback"),
            Some(&json!("analyzedGraphV0HotStylePrewarm")),
        );
        Ok(())
    }

    fn diagnostic_data_value<'a>(
        diagnostics: Option<&'a Value>,
        code: &str,
        key: &str,
    ) -> Option<&'a Value> {
        diagnostics
            .and_then(Value::as_array)?
            .iter()
            .find(|diagnostic| diagnostic.pointer("/code") == Some(&json!(code)))?
            .pointer(format!("/data/{key}").as_str())
    }
}
