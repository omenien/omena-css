use super::*;

#[test]
fn handles_minimal_lsp_lifecycle_requests() {
    let mut state = LspShellState::default();
    let initialize = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": "file:///workspace-a",
                        "name": "workspace-a",
                    },
                ],
            },
        }),
    );

    assert_eq!(
        initialize.as_ref().and_then(|value| value.get("id")),
        Some(&json!(1))
    );
    assert_eq!(
        initialize
            .as_ref()
            .and_then(|value| value.pointer("/result/capabilities/textDocumentSync")),
        Some(&json!(2)),
    );
    assert!(!state.shutdown_requested);
    assert_eq!(state.workspace_folder_count(), 1);
    assert_eq!(
        state
            .workspace_folder("file:///workspace-a")
            .map(|folder| folder.name.as_str()),
        Some("workspace-a"),
    );

    let runtime_probe = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": RUNTIME_LOOP_PROBE_REQUEST,
        }),
    );
    assert_eq!(
        runtime_probe.as_ref().and_then(|value| value.get("id")),
        Some(&json!(2)),
    );
    assert!(
        runtime_probe
            .as_ref()
            .and_then(|value| value.pointer("/result/now"))
            .and_then(Value::as_u64)
            .is_some(),
    );

    let shutdown = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "shutdown",
        }),
    );
    assert_eq!(
        shutdown.as_ref().and_then(|value| value.get("result")),
        Some(&Value::Null)
    );
    assert!(state.shutdown_requested);

    let exit = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "exit",
        }),
    );
    assert!(exit.is_none());
    assert!(state.should_exit);
}

#[test]
fn work_done_progress_uses_client_capability_and_sinks_create_response()
-> Result<(), Box<dyn std::error::Error>> {
    let mut state = LspShellState::default();
    let initialize_outputs = handle_lsp_message_outputs(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "capabilities": {
                    "window": {
                        "workDoneProgress": true,
                    },
                },
                "workspaceFolders": [
                    {
                        "uri": "file:///workspace-a",
                        "name": "workspace-a",
                    },
                ],
            },
        }),
    );
    assert_eq!(initialize_outputs.len(), 1);

    let (initialized_outputs, mut initialized_jobs) =
        match handle_lsp_message_scheduled_outputs_or_dispatch(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "method": "initialized",
                "params": {},
            }),
        ) {
            LspLoopTurnV0::OutputsAndDeferredDiagnostics {
                outputs,
                workspace_index_jobs,
                ..
            } => (
                outputs
                    .into_iter()
                    .map(ScheduledLspOutput::into_value)
                    .collect::<Vec<_>>(),
                workspace_index_jobs,
            ),
            other => {
                return Err(std::io::Error::other(format!(
                    "initialized should schedule background index: {other:?}"
                ))
                .into());
            }
        };
    let create_id = assert_work_done_progress_begin(&initialized_outputs)
        .ok_or_else(|| std::io::Error::other("missing initialized progress begin"))?;
    let initialized_job = initialized_jobs.pop().ok_or_else(|| {
        std::io::Error::other("initialized should enqueue one workspace index job")
    })?;
    let initialized_result = collect_background_workspace_index(initialized_job);
    apply_background_workspace_index_result(&mut state, initialized_result.clone());
    let initialized_end = workspace_index_progress_end_output(&initialized_result)
        .ok_or_else(|| std::io::Error::other("workspace index result should end progress"))?
        .into_value();
    assert_work_done_progress_end(&initialized_end);
    assert!(
        handle_lsp_message(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "id": create_id,
                "result": null,
            }),
        )
        .is_none(),
        "client response to a server-created workDoneProgress request must be consumed"
    );

    let (workspace_change_outputs, mut workspace_change_jobs) =
        match handle_lsp_message_scheduled_outputs_or_dispatch(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "method": "workspace/didChangeWorkspaceFolders",
                "params": {
                    "event": {
                        "removed": [],
                        "added": [
                            {
                                "uri": "file:///workspace-b",
                                "name": "workspace-b",
                            },
                        ],
                    },
                },
            }),
        ) {
            LspLoopTurnV0::OutputsAndDeferredDiagnostics {
                outputs,
                workspace_index_jobs,
                ..
            } => (
                outputs
                    .into_iter()
                    .map(ScheduledLspOutput::into_value)
                    .collect::<Vec<_>>(),
                workspace_index_jobs,
            ),
            other => {
                return Err(std::io::Error::other(format!(
                    "workspace folder change should schedule background index: {other:?}"
                ))
                .into());
            }
        };
    let workspace_change_create_id = assert_work_done_progress_begin(&workspace_change_outputs)
        .ok_or_else(|| std::io::Error::other("missing workspace-change progress begin"))?;
    let workspace_change_job = workspace_change_jobs.pop().ok_or_else(|| {
        std::io::Error::other("workspace folder change should enqueue one workspace index job")
    })?;
    let workspace_change_result = collect_background_workspace_index(workspace_change_job);
    apply_background_workspace_index_result(&mut state, workspace_change_result.clone());
    let workspace_change_end = workspace_index_progress_end_output(&workspace_change_result)
        .ok_or_else(|| std::io::Error::other("workspace folder index result should end progress"))?
        .into_value();
    assert_work_done_progress_end(&workspace_change_end);
    assert!(
        handle_lsp_message(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "id": workspace_change_create_id,
                "result": null,
            }),
        )
        .is_none(),
        "workspace-folder progress create responses must also be consumed"
    );

    let unknown_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": "not-server-initiated",
            "result": null,
        }),
    );
    assert_eq!(
        unknown_response
            .as_ref()
            .and_then(|value| value.pointer("/error/code")),
        Some(&json!(-32600)),
        "untracked id-only client messages should keep the existing invalid-request fallback"
    );
    Ok(())
}

#[test]
fn work_done_progress_is_silent_without_client_capability() -> Result<(), Box<dyn std::error::Error>>
{
    let mut state = LspShellState::default();
    let _ = handle_lsp_message_outputs(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": "file:///workspace-a",
                        "name": "workspace-a",
                    },
                ],
            },
        }),
    );

    let initialized_outputs = match handle_lsp_message_scheduled_outputs_or_dispatch(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {},
        }),
    ) {
        LspLoopTurnV0::OutputsAndDeferredDiagnostics { outputs, .. } => outputs
            .into_iter()
            .map(ScheduledLspOutput::into_value)
            .collect::<Vec<_>>(),
        other => {
            return Err(std::io::Error::other(format!(
                "initialized should schedule background index: {other:?}"
            ))
            .into());
        }
    };
    assert!(
        initialized_outputs.iter().all(|output| output.get("method")
            != Some(&json!("window/workDoneProgress/create"))
            && output.get("method") != Some(&json!("$/progress"))),
        "workDoneProgress must be gated by the client capability"
    );
    Ok(())
}

fn assert_work_done_progress_begin(outputs: &[Value]) -> Option<String> {
    let create = outputs
        .iter()
        .find(|output| output.get("method") == Some(&json!("window/workDoneProgress/create")));
    assert!(
        create.is_some(),
        "missing workDoneProgress create request: {outputs:?}"
    );
    let create = create?;
    let id = create.get("id").and_then(Value::as_str);
    assert!(
        id.is_some(),
        "workDoneProgress create request must use a string id: {create:?}"
    );
    let token = create.pointer("/params/token").and_then(Value::as_str);
    assert!(
        token.is_some(),
        "workDoneProgress create request must carry a token: {create:?}"
    );
    let token = token?;
    let progress = outputs
        .iter()
        .filter(|output| output.get("method") == Some(&json!("$/progress")))
        .collect::<Vec<_>>();
    assert_eq!(
        progress.len(),
        1,
        "enqueue output must contain exactly one Begin progress notification"
    );
    assert_eq!(
        progress[0].pointer("/params/token").and_then(Value::as_str),
        Some(token)
    );
    assert_eq!(
        progress[0]
            .pointer("/params/value/kind")
            .and_then(Value::as_str),
        Some("begin")
    );
    assert!(
        progress[0].pointer("/params/value/percentage").is_none(),
        "workspace-index progress has no stable denominator and must not report percentage"
    );
    id.map(str::to_string)
}

fn assert_work_done_progress_end(output: &Value) {
    assert_eq!(output.get("method"), Some(&json!("$/progress")));
    assert_eq!(
        output.pointer("/params/value/kind").and_then(Value::as_str),
        Some("end")
    );
    assert!(
        output.pointer("/params/value/percentage").is_none(),
        "End notifications must omit percentage"
    );
}

#[test]
fn work_done_progress_end_reports_background_continuation_count()
-> Result<(), Box<dyn std::error::Error>> {
    let exhausted_result = LspWorkspaceIndexResultV0 {
        revision: 1,
        progress_token: Some("workspace-index-token".to_string()),
        documents: Vec::new(),
        pending_file_uris: vec![
            "file:///workspace/src/A.module.scss".to_string(),
            "file:///workspace/src/B.module.scss".to_string(),
        ],
        indexed_count: 512,
        pending_file_count: 2,
        exhausted: true,
    };
    let exhausted_output = workspace_index_progress_end_output(&exhausted_result)
        .ok_or_else(|| std::io::Error::other("progress token should produce an End notification"))?
        .into_value();
    assert_eq!(
        exhausted_output
            .pointer("/params/value/message")
            .and_then(Value::as_str),
        Some("Workspace index updated; continuing with 2 remaining files in the background")
    );

    let completed_result = LspWorkspaceIndexResultV0 {
        progress_token: Some("workspace-index-token".to_string()),
        pending_file_uris: Vec::new(),
        pending_file_count: 0,
        exhausted: false,
        ..exhausted_result
    };
    let completed_output = workspace_index_progress_end_output(&completed_result)
        .ok_or_else(|| std::io::Error::other("progress token should produce an End notification"))?
        .into_value();
    assert_eq!(
        completed_output
            .pointer("/params/value/message")
            .and_then(Value::as_str),
        Some("Workspace index updated")
    );
    Ok(())
}

#[test]
fn reports_unknown_requests_without_panicking() {
    let mut state = LspShellState::default();
    let response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": "unknown-1",
            "method": "workspace/symbol",
        }),
    );

    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/error/code")),
        Some(&json!(-32601)),
    );
}
