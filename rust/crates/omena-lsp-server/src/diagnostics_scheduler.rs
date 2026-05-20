use crate::{
    LspShellState, is_style_document_uri, resolve_document_diagnostics_for_uri,
    resolve_source_diagnostics_for_uri, resolve_workspace_folder_uri, workspace_folder_compatible,
};
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
            "refreshOpenDocumentsOnConfigurationChange",
            "refreshOpenDocumentsAfterWorkspaceIndexing",
        ],
        request_path_policy: vec![
            "noNodeDiagnosticsSchedulerOnRustLspPath",
            "diagnosticsNotificationsStayRustOwned",
            "closedDocumentsPublishEmptyDiagnostics",
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
) -> Vec<Value> {
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
) -> Vec<Value> {
    let mut outputs = vec![publish_diagnostics_notification(
        uri,
        if is_close {
            json!([])
        } else {
            resolve_document_diagnostics_for_uri(state, uri)
        },
    )];

    if is_style_document_uri(uri) {
        for source_uri in source_uris_for_text_style_change_diagnostics(state, uri) {
            outputs.push(publish_diagnostics_notification(
                source_uri.as_str(),
                resolve_source_diagnostics_for_uri(state, source_uri.as_str()),
            ));
        }
    }

    outputs
}

fn diagnostics_for_watched_files(state: &LspShellState, uris: Vec<String>) -> Vec<Value> {
    let mut outputs = Vec::new();
    let mut style_uris_to_refresh = BTreeSet::new();
    let mut source_uris_to_refresh = BTreeSet::new();
    for uri in uris
        .into_iter()
        .filter(|uri| is_style_document_uri(uri.as_str()))
    {
        style_uris_to_refresh.insert(uri);
    }
    for uri in style_uris_to_refresh {
        outputs.push(publish_diagnostics_notification(
            uri.as_str(),
            resolve_document_diagnostics_for_uri(state, uri.as_str()),
        ));
        for source_uri in source_uris_for_style_change_diagnostics(state, uri.as_str()) {
            source_uris_to_refresh.insert(source_uri);
        }
    }
    for source_uri in source_uris_to_refresh {
        outputs.push(publish_diagnostics_notification(
            source_uri.as_str(),
            resolve_source_diagnostics_for_uri(state, source_uri.as_str()),
        ));
    }
    outputs
}

fn diagnostics_for_open_documents(state: &LspShellState) -> Vec<Value> {
    open_document_uris_for_diagnostics(state)
        .into_iter()
        .map(|uri| {
            publish_diagnostics_notification(
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
        .filter(|uri| state.documents.contains_key(uri.as_str()))
        .cloned()
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
}
