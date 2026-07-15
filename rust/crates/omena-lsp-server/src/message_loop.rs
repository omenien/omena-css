use crate::diagnostics_scheduler::{diagnostics_schedule_event, run_diagnostics_schedule_effects};
use crate::lsp_output::ScheduledLspOutput;
use crate::{
    CANCEL_REQUEST_METHOD, CASCADE_AT_POSITION_REQUEST, DEBUG_STATE_REQUEST,
    EXPLAIN_HOVER_TRACE_REQUEST, EXPLAIN_REQUEST, LspDeferredDiagnosticsDispatchV0,
    LspQuerySnapshotV0, LspShellState, LspWorkspaceIndexJobV0, LspWorkspaceIndexResultV0,
    REQUEST_CANCELLED_ERROR_CODE, RUNTIME_LOOP_PROBE_REQUEST, SDK_WORKFLOW_REQUEST,
    SOURCE_DIAGNOSTICS_REQUEST, STYLE_CONTEXT_INDEX_REQUEST, STYLE_DIAGNOSTICS_REQUEST,
    STYLE_HOVER_CANDIDATES_REQUEST, apply_diagnostic_settings, apply_feature_settings,
    apply_resolution_settings, current_node_lsp_capability_contract, did_change_text_document,
    did_change_watched_files, did_change_workspace_folders, did_close_text_document,
    did_open_text_document, index_workspace_style_files, initialize_workspace_folders,
    prepare_background_workspace_index_job, refresh_source_indexes_for_resolution_settings_change,
    resolve_cascade_at_position, resolve_lsp_code_actions, resolve_lsp_code_lens,
    resolve_lsp_completion, resolve_lsp_definition, resolve_lsp_explain, resolve_lsp_hover,
    resolve_lsp_hover_trace, resolve_lsp_prepare_rename, resolve_lsp_references,
    resolve_lsp_rename, resolve_source_diagnostics, resolve_style_context_index,
    resolve_style_diagnostics, resolve_style_hover_candidates,
};
use serde_json::{Value, json};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn handle_lsp_message(state: &mut LspShellState, message: Value) -> Option<Value> {
    let method = message.get("method").and_then(Value::as_str);
    let id = message.get("id").cloned();

    if method == Some(CANCEL_REQUEST_METHOD) && id.is_none() {
        cancel_lsp_request(state, message.get("params"));
        return None;
    }

    if let Some(request_id) = id.as_ref()
        && take_cancelled_request(state, request_id)
    {
        return Some(cancelled_request_response(request_id.clone()));
    }

    match (method, id) {
        (Some("initialize"), Some(request_id)) => {
            initialize_workspace_folders(state, message.get("params"));
            Some(json!({
                "jsonrpc": "2.0",
                "id": request_id,
                "result": {
                    "capabilities": current_node_lsp_capability_contract(),
                    "serverInfo": {
                        "name": "omena-css-rust",
                    },
                },
            }))
        }
        (Some("initialized"), None) => {
            index_workspace_style_files(state);
            None
        }
        (Some("textDocument/didOpen"), None) => {
            did_open_text_document(state, message.get("params"));
            None
        }
        (Some("textDocument/didChange"), None) => {
            did_change_text_document(state, message.get("params"));
            None
        }
        (Some("textDocument/didClose"), None) => {
            did_close_text_document(state, message.get("params"));
            None
        }
        (Some("workspace/didChangeWorkspaceFolders"), None) => {
            did_change_workspace_folders(state, message.get("params"), true);
            None
        }
        (Some("workspace/didChangeConfiguration"), None) => {
            did_change_configuration(state, message.get("params"));
            None
        }
        (Some("workspace/didChangeWatchedFiles"), None) => {
            did_change_watched_files(state, message.get("params"));
            None
        }
        (Some("textDocument/hover"), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": if state.features.hover { resolve_lsp_hover(state, message.get("params")) } else { Value::Null },
        })),
        (Some("textDocument/definition"), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": if state.features.definition { resolve_lsp_definition(state, message.get("params")) } else { Value::Null },
        })),
        (Some("textDocument/references"), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": if state.features.references { resolve_lsp_references(state, message.get("params")) } else { Value::Null },
        })),
        (Some("textDocument/completion"), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": if state.features.completion { resolve_lsp_completion(state, message.get("params")) } else { Value::Null },
        })),
        (Some("textDocument/codeAction"), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": resolve_lsp_code_actions(state, message.get("params")),
        })),
        (Some("textDocument/documentColor"), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": crate::color_provider::resolve_lsp_document_color(state, message.get("params")),
        })),
        (Some("textDocument/colorPresentation"), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": crate::color_provider::resolve_lsp_color_presentation(message.get("params")),
        })),
        (Some("textDocument/documentLink"), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": crate::document_links::resolve_lsp_document_links(state, message.get("params")),
        })),
        (Some("workspace/symbol"), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": crate::workspace_symbols::resolve_lsp_workspace_symbols(state, message.get("params")),
        })),
        (Some("textDocument/codeLens"), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": if state.features.references { resolve_lsp_code_lens(state, message.get("params")) } else { Value::Null },
        })),
        (Some("textDocument/prepareRename"), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": if state.features.rename { resolve_lsp_prepare_rename(state, message.get("params")) } else { Value::Null },
        })),
        (Some("textDocument/rename"), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": if state.features.rename { resolve_lsp_rename(state, message.get("params")) } else { Value::Null },
        })),
        (Some(DEBUG_STATE_REQUEST), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": state.snapshot(),
        })),
        (Some(RUNTIME_LOOP_PROBE_REQUEST), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": {
                "now": current_time_millis(),
            },
        })),
        (Some(STYLE_HOVER_CANDIDATES_REQUEST), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": resolve_style_hover_candidates(state, message.get("params")),
        })),
        (Some(STYLE_DIAGNOSTICS_REQUEST), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": resolve_style_diagnostics(state, message.get("params")),
        })),
        (Some(SOURCE_DIAGNOSTICS_REQUEST), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": resolve_source_diagnostics(state, message.get("params")),
        })),
        (Some(CASCADE_AT_POSITION_REQUEST), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": resolve_cascade_at_position(state, message.get("params")),
        })),
        (Some(STYLE_CONTEXT_INDEX_REQUEST), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": resolve_style_context_index(state, message.get("params")),
        })),
        (Some(EXPLAIN_HOVER_TRACE_REQUEST), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": resolve_lsp_hover_trace(state, message.get("params")),
        })),
        (Some(EXPLAIN_REQUEST), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": resolve_lsp_explain(state, message.get("params")),
        })),
        (Some(SDK_WORKFLOW_REQUEST), Some(request_id)) => {
            match crate::sdk_workflow::resolve_lsp_sdk_workflow(state, message.get("params")) {
                Ok(result) => Some(json!({
                    "jsonrpc": "2.0",
                    "id": request_id,
                    "result": result,
                })),
                Err(error) => Some(json!({
                    "jsonrpc": "2.0",
                    "id": request_id,
                    "error": {
                        "code": -32001,
                        "message": error.message.clone(),
                        "data": omena_query::OmenaSdkErrorEnvelopeV0 { error },
                    },
                })),
            }
        }
        (Some("shutdown"), Some(request_id)) => {
            state.shutdown_requested = true;
            Some(json!({
                "jsonrpc": "2.0",
                "id": request_id,
                "result": null,
            }))
        }
        (Some("exit"), None) => {
            state.should_exit = true;
            None
        }
        (Some(_), Some(request_id)) => Some(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "error": {
                "code": -32601,
                "message": "Method not found",
            },
        })),
        (Some(_), None) => None,
        (None, Some(request_id)) => {
            if take_server_progress_response(state, &request_id) {
                None
            } else {
                Some(json!({
                    "jsonrpc": "2.0",
                    "id": request_id,
                    "error": {
                        "code": -32600,
                        "message": "Invalid Request",
                    },
                }))
            }
        }
        (None, None) => None,
    }
}

fn did_change_configuration(state: &mut LspShellState, params: Option<&Value>) {
    state.configuration_change_count += 1;
    let Some(settings) = params
        .and_then(|value| value.get("settings"))
        .and_then(|value| value.get("omena"))
    else {
        return;
    };
    apply_feature_settings(state, settings.get("features"));
    if apply_diagnostic_settings(state, settings.get("diagnostics")) {
        // Changed diagnostics settings owe a workspace republish: published
        // diagnostics must re-render under the new severity / deep-analysis
        // settings instead of staying stale indefinitely (rfcs#111 §2, the
        // config-staleness fix). Settings-Eq changes cut off above.
        state
            .tide_ledger
            .advance(&[crate::tide::TideInputKindV0::DiagnosticSettings]);
        let tick = state.tide_tick;
        state
            .tide_republish_lane
            .deposit(crate::tide::TideRepublishDemandV0::All, tick);
    }
    if apply_resolution_settings(state, settings.get("resolution")) {
        state
            .tide_ledger
            .advance(&[crate::tide::TideInputKindV0::ResolutionSettings]);
        refresh_source_indexes_for_resolution_settings_change(state);
    }
}

fn cancel_lsp_request(state: &mut LspShellState, params: Option<&Value>) {
    let Some(id) = params.and_then(|value| value.get("id")) else {
        return;
    };
    if let Some(key) = request_id_key(id) {
        state.cancelled_request_ids.cancel(key);
    }
}

fn take_cancelled_request(state: &mut LspShellState, request_id: &Value) -> bool {
    request_id_key(request_id).is_some_and(|key| {
        state
            .cancelled_request_ids
            .take_cancelled_result(key.as_str())
            .is_err()
    })
}

fn take_server_progress_response(state: &mut LspShellState, request_id: &Value) -> bool {
    request_id
        .as_str()
        .is_some_and(|id| state.take_server_progress_response(id))
}

fn request_id_key(id: &Value) -> Option<String> {
    if let Some(value) = id.as_str() {
        return Some(format!("s:{value}"));
    }
    if id.is_number() {
        return Some(format!("n:{id}"));
    }
    None
}

fn cancelled_request_response(request_id: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": request_id,
        "error": {
            "code": REQUEST_CANCELLED_ERROR_CODE,
            "message": "Request cancelled",
        },
    })
}

pub fn handle_lsp_message_outputs(state: &mut LspShellState, message: Value) -> Vec<Value> {
    handle_lsp_message_scheduled_outputs(state, message)
        .into_iter()
        .map(ScheduledLspOutput::into_value)
        .collect()
}

/// One loop turn for the stdio server (RFC 0009 Pillar A, rfcs#67 slice A-min).
///
/// `Outputs` is the synchronous path — every notification and every mutating or
/// loop-owned request keeps its existing FIFO behaviour. `DispatchQuery` hands
/// the heaviest read-only request class (`textDocument/hover` and
/// `textDocument/definition`, the 75/95 of the runtime-loop burst) to a worker
/// together with a copy-on-write snapshot taken HERE on the loop thread, so the
/// loop turn for that class collapses to read + O(documents) pointer clones +
/// channel send.
#[derive(Debug)]
pub enum LspLoopTurnV0 {
    Outputs(Vec<ScheduledLspOutput>),
    OutputsAndDeferredDiagnostics {
        outputs: Vec<ScheduledLspOutput>,
        deferred_diagnostics: Vec<LspDeferredDiagnosticsDispatchV0>,
        workspace_index_jobs: Vec<LspWorkspaceIndexJobV0>,
    },
    // Boxed: a dispatch carries the whole snapshot (settings + documents map),
    // which would otherwise dominate the enum size for the common Outputs turn.
    DispatchQuery(Box<LspQueryDispatchV0>),
}

/// A dispatched hover/definition request: the request message paired with the
/// loop-consistent snapshot it must be answered from.
#[derive(Debug)]
pub struct LspQueryDispatchV0 {
    pub snapshot: LspQuerySnapshotV0,
    pub message: Value,
}

/// Loop-side turn handler for the stdio server. Mirrors
/// [`handle_lsp_message_scheduled_outputs`] except that dispatchable query
/// requests are returned as jobs instead of being resolved inline.
///
/// The `$/cancelRequest` gate stays loop-side: a request already cancelled when
/// it arrives is answered with `REQUEST_CANCELLED_ERROR_CODE` here and never
/// dispatched (the take happens exactly once — the dispatch path does not call
/// [`handle_lsp_message`], so there is no double-take). A `$/cancelRequest` for
/// a request that was ALREADY dispatched is a documented no-op in this slice:
/// the response is still computed and sent, which the LSP allows; in-flight
/// cancellation tokens are a follow-up slice.
pub fn handle_lsp_message_scheduled_outputs_or_dispatch(
    state: &mut LspShellState,
    message: Value,
) -> LspLoopTurnV0 {
    if let Some(request_id) = dispatchable_query_request_id(&message) {
        if take_cancelled_request(state, &request_id) {
            return LspLoopTurnV0::Outputs(vec![ScheduledLspOutput::immediate(
                cancelled_request_response(request_id),
            )]);
        }
        // documentColor on a document with no variable-reference candidate
        // answers [] right here: the editor merges ALL color providers
        // before painting swatches, so routing a trivially-empty answer
        // through the worker queue would gate the built-in literal chips
        // on our round-trip. The check reads the precomputed per-document
        // candidates only.
        if message.get("method").and_then(Value::as_str) == Some("textDocument/documentColor")
            && !crate::color_provider::document_has_color_reference_candidates(
                state,
                message.get("params"),
            )
        {
            return LspLoopTurnV0::Outputs(vec![ScheduledLspOutput::immediate(json!({
                "jsonrpc": "2.0",
                "id": request_id,
                "result": json!([]),
            }))]);
        }
        return LspLoopTurnV0::DispatchQuery(Box::new(LspQueryDispatchV0 {
            snapshot: state.query_snapshot(),
            message,
        }));
    }
    let effects = handle_lsp_message_scheduled_effects_with_deferral(state, message, true, true);
    if effects.deferred_diagnostics.is_empty() && effects.workspace_index_jobs.is_empty() {
        LspLoopTurnV0::Outputs(effects.outputs)
    } else {
        LspLoopTurnV0::OutputsAndDeferredDiagnostics {
            outputs: effects.outputs,
            deferred_diagnostics: effects.deferred_diagnostics,
            workspace_index_jobs: effects.workspace_index_jobs,
        }
    }
}

/// The dispatched request class: hover/definition REQUESTS (an `id` is
/// required — without one there is nothing to respond to). Notifications named
/// like these methods fall through to the synchronous path unchanged.
/// JSON-RPC internal-error response for a dispatched query whose resolver
/// panicked on the worker: the request still gets exactly one response (a
/// silent drop would hang the client), and the worker survives to serve the
/// rest of its queue.
pub fn dispatched_query_internal_error_response(dispatch: &LspQueryDispatchV0) -> Option<Value> {
    let request_id = dispatchable_query_request_id(&dispatch.message)?;
    Some(json!({
        "jsonrpc": "2.0",
        "id": request_id,
        "error": {
            "code": -32603,
            "message": "internal error while resolving the dispatched query",
        },
    }))
}

fn dispatchable_query_request_id(message: &Value) -> Option<Value> {
    let method = message.get("method").and_then(Value::as_str)?;
    if method != "textDocument/hover"
        && method != "textDocument/definition"
        && method != "textDocument/documentColor"
        && method != "textDocument/documentLink"
        && method != "workspace/symbol"
        && method != "textDocument/codeLens"
    {
        return None;
    }
    message.get("id").cloned()
}

/// Lane routing for dispatched queries: HEAVY dispatches (codeLens, whose
/// occurrence-index rebuild is a whole-corpus scan, and the internal
/// substrate warmups) run on their own worker so an interactive hover /
/// definition / documentColor never queues behind a multi-second compute.
/// Head-of-line blocking on the single lane was measured live: a
/// documentColor answered 3ms of work 20 SECONDS after dispatch because a
/// codeLens rebuild held the worker.
pub fn dispatched_query_is_heavy(dispatch: &LspQueryDispatchV0) -> bool {
    matches!(
        dispatch.message.get("method").and_then(Value::as_str),
        Some("textDocument/codeLens") | Some(HOVER_SUBSTRATE_WARMUP_METHOD)
    )
}

/// Worker-side resolution of a dispatched query request. Mirrors the
/// synchronous `handle_lsp_message` arms exactly, including the feature gating
/// (evaluated against the snapshot, i.e. the settings in force at dispatch
/// time). Returns the complete JSON-RPC response; `None` only for messages that
/// were never dispatchable (defensive — the loop only dispatches
/// hover/definition requests).
/// Internal dispatch (no client-visible response): resolve one hover
/// against the snapshot purely to POPULATE the Arc-shared hover memos
/// (cascade-narrowing substrate, resolver identity index) right after a
/// republish tide settles — the user's FIRST hover then lands warm instead
/// of paying the substrate build interactively.
pub const HOVER_SUBSTRATE_WARMUP_METHOD: &str = "omena/internalWarmHoverSubstrate";

/// The post-settle warmup dispatch: the first OPEN style document's first
/// hover candidate. One document suffices — the substrate the build warms
/// is workspace-scoped, not per-document.
pub fn hover_substrate_warmup_dispatch(state: &LspShellState) -> Option<Box<LspQueryDispatchV0>> {
    let document = state
        .open_document_uris
        .iter()
        .filter_map(|file_id| state.document_for_file_id(*file_id))
        .find(|document| {
            crate::protocol::is_style_document_uri(document.uri.as_str())
                && !document.style_candidates.is_empty()
        })?;
    let candidate = document.style_candidates.first()?;
    let message = json!({
        "jsonrpc": "2.0",
        "method": HOVER_SUBSTRATE_WARMUP_METHOD,
        "params": {
            "textDocument": { "uri": document.uri },
            "position": candidate.range.start,
        },
    });
    Some(Box::new(LspQueryDispatchV0 {
        snapshot: state.query_snapshot(),
        message,
    }))
}

pub fn resolve_dispatched_query_response(dispatch: &LspQueryDispatchV0) -> Option<Value> {
    if dispatch.message.get("method").and_then(Value::as_str) == Some(HOVER_SUBSTRATE_WARMUP_METHOD)
    {
        let started = std::time::Instant::now();
        let state = dispatch.snapshot.shell_state();
        let _ = resolve_lsp_hover(state, dispatch.message.get("params"));
        crate::loop_trace!(
            "hover-warmup done took_ms={}",
            started.elapsed().as_millis()
        );
        return None;
    }
    let request_id = dispatchable_query_request_id(&dispatch.message)?;
    let method = dispatch.message.get("method").and_then(Value::as_str)?;
    let params = dispatch.message.get("params");
    let state = dispatch.snapshot.shell_state();
    let result = match method {
        "textDocument/hover" => {
            if state.features.hover {
                resolve_lsp_hover(state, params)
            } else {
                Value::Null
            }
        }
        "textDocument/definition" => {
            if state.features.definition {
                resolve_lsp_definition(state, params)
            } else {
                Value::Null
            }
        }
        // Off-loop by design: specifier resolution may probe disk.
        "textDocument/documentLink" => {
            crate::document_links::resolve_lsp_document_links(state, params)
        }
        // Read-only sweep over precomputed per-document candidates.
        "workspace/symbol" => {
            crate::workspace_symbols::resolve_lsp_workspace_symbols(state, params)
        }
        // Off-loop by design: the resolution walk can read unadmitted
        // dependency chains from disk, which must never block the loop.
        "textDocument/documentColor" => {
            let started = std::time::Instant::now();
            let result = crate::color_provider::resolve_lsp_document_color(state, params);
            crate::loop_trace!(
                "document-color dispatched took_ms={}",
                started.elapsed().as_millis()
            );
            result
        }
        // Off-loop by design: a memo miss rebuilds the workspace occurrence
        // index — a whole-corpus scan measured in the tens of seconds on a
        // real workspace, which used to land on the loop. The memo is
        // Arc-shared with the loop and the other workers, so one rebuild
        // serves every consumer until the document keys move.
        "textDocument/codeLens" => {
            if state.features.references {
                let started = std::time::Instant::now();
                let result = resolve_lsp_code_lens(state, params);
                crate::loop_trace!(
                    "code-lens dispatched took_ms={}",
                    started.elapsed().as_millis()
                );
                result
            } else {
                Value::Null
            }
        }
        _ => return None,
    };
    Some(json!({
        "jsonrpc": "2.0",
        "id": request_id,
        "result": result,
    }))
}

pub fn handle_lsp_message_scheduled_outputs(
    state: &mut LspShellState,
    message: Value,
) -> Vec<ScheduledLspOutput> {
    handle_lsp_message_scheduled_effects_with_deferral(state, message, false, false).outputs
}

fn handle_lsp_message_scheduled_effects_with_deferral(
    state: &mut LspShellState,
    message: Value,
    enable_deferred_style_diagnostics: bool,
    enable_background_workspace_index: bool,
) -> LspScheduledEffectsV0 {
    let method = message
        .get("method")
        .and_then(Value::as_str)
        .map(str::to_string);
    let document_uri = message
        .get("params")
        .and_then(|value| value.get("textDocument"))
        .and_then(|value| value.get("uri"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let watched_file_uris = watched_file_uris_from_message(&message);
    // Computed BEFORE the handler mutates state: for a didOpen, the state
    // still holds the index-admitted document, so byte-identical text is
    // detectable here and nowhere later.
    let content_changed = if method.as_deref() == Some("textDocument/didOpen") {
        let incoming_text = message
            .pointer("/params/textDocument/text")
            .and_then(Value::as_str);
        match (
            document_uri.as_deref().and_then(|uri| state.document(uri)),
            incoming_text,
        ) {
            (Some(existing), Some(text)) => existing.text != text,
            _ => true,
        }
    } else {
        true
    };
    let diagnostics_event = diagnostics_schedule_event(
        method.as_deref(),
        document_uri,
        watched_file_uris,
        content_changed,
    );
    let mut effects = LspScheduledEffectsV0::default();

    let response = if enable_background_workspace_index {
        handle_lsp_message_for_background_workspace_index(state, &message, &mut effects)
    } else {
        handle_lsp_message(state, message)
    };

    if let Some(response) = response {
        effects
            .outputs
            .push(ScheduledLspOutput::immediate(response));
    }

    if let Some(event) = diagnostics_event {
        let diagnostics_effects = if enable_deferred_style_diagnostics {
            run_diagnostics_schedule_effects(state, event)
        } else {
            crate::diagnostics_scheduler::DiagnosticsScheduleEffectsV0::from_outputs(
                crate::diagnostics_scheduler::run_diagnostics_schedule(state, event),
            )
        };
        effects.outputs.extend(diagnostics_effects.outputs);
        effects
            .deferred_diagnostics
            .extend(diagnostics_effects.deferred_diagnostics);
    }

    effects
}

#[derive(Debug, Default)]
pub struct LspScheduledEffectsV0 {
    pub outputs: Vec<ScheduledLspOutput>,
    pub deferred_diagnostics: Vec<LspDeferredDiagnosticsDispatchV0>,
    pub workspace_index_jobs: Vec<LspWorkspaceIndexJobV0>,
}

impl From<Vec<ScheduledLspOutput>> for LspScheduledEffectsV0 {
    fn from(outputs: Vec<ScheduledLspOutput>) -> Self {
        Self {
            outputs,
            deferred_diagnostics: Vec::new(),
            workspace_index_jobs: Vec::new(),
        }
    }
}

fn handle_lsp_message_for_background_workspace_index(
    state: &mut LspShellState,
    message: &Value,
    effects: &mut LspScheduledEffectsV0,
) -> Option<Value> {
    match message.get("method").and_then(Value::as_str) {
        Some("initialized") if message.get("id").is_none() => {
            let mut job = prepare_background_workspace_index_job(state);
            effects
                .outputs
                .extend(workspace_index_progress_begin_outputs(state, &mut job));
            effects.workspace_index_jobs.push(job);
            None
        }
        Some("workspace/didChangeWorkspaceFolders") if message.get("id").is_none() => {
            let added_workspace_folder =
                did_change_workspace_folders(state, message.get("params"), false);
            if added_workspace_folder {
                let mut job = prepare_background_workspace_index_job(state);
                effects
                    .outputs
                    .extend(workspace_index_progress_begin_outputs(state, &mut job));
                effects.workspace_index_jobs.push(job);
            }
            None
        }
        _ => handle_lsp_message(state, message.clone()),
    }
}

fn workspace_index_progress_begin_outputs(
    state: &mut LspShellState,
    job: &mut LspWorkspaceIndexJobV0,
) -> Vec<ScheduledLspOutput> {
    if !state.client_supports_work_done_progress {
        return Vec::new();
    }

    let (id, token) = state.allocate_work_done_progress_request();
    job.progress_token = Some(token.clone());
    vec![
        ScheduledLspOutput::immediate(json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "window/workDoneProgress/create",
            "params": {
                "token": token,
            },
        })),
        ScheduledLspOutput::immediate(json!({
            "jsonrpc": "2.0",
            "method": "$/progress",
            "params": {
                "token": token,
                "value": {
                    "kind": "begin",
                    "title": "Omena CSS workspace index",
                    "message": "Scanning workspace files",
                },
            },
        })),
    ]
}

pub fn workspace_index_progress_end_output(
    result: &LspWorkspaceIndexResultV0,
) -> Option<ScheduledLspOutput> {
    let token = result.progress_token.as_deref()?;
    let message = if result.exhausted && result.pending_file_count > 0 {
        format!(
            "Workspace index updated; continuing with {} remaining files in the background",
            result.pending_file_count
        )
    } else {
        "Workspace index updated".to_string()
    };
    Some(ScheduledLspOutput::immediate(json!({
        "jsonrpc": "2.0",
        "method": "$/progress",
        "params": {
            "token": token,
            "value": {
                "kind": "end",
                "message": message,
            },
        },
    })))
}

pub(crate) fn current_time_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis())
}

fn watched_file_uris_from_message(message: &Value) -> Vec<String> {
    message
        .get("params")
        .and_then(|value| value.get("changes"))
        .and_then(Value::as_array)
        .map(|changes| {
            changes
                .iter()
                .filter_map(|change| change.get("uri").and_then(Value::as_str))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}
