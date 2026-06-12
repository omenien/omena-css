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
fn work_done_progress_uses_client_capability_and_sinks_create_response() {
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

    let initialized_outputs = handle_lsp_message_outputs(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {},
        }),
    );
    let Some(create_id) = assert_work_done_progress_triplet(&initialized_outputs) else {
        return;
    };
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

    let workspace_change_outputs = handle_lsp_message_outputs(
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
    );
    let Some(workspace_change_create_id) =
        assert_work_done_progress_triplet(&workspace_change_outputs)
    else {
        return;
    };
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
}

#[test]
fn work_done_progress_is_silent_without_client_capability() {
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

    let initialized_outputs = handle_lsp_message_outputs(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {},
        }),
    );
    assert!(
        initialized_outputs.iter().all(|output| output.get("method")
            != Some(&json!("window/workDoneProgress/create"))
            && output.get("method") != Some(&json!("$/progress"))),
        "workDoneProgress must be gated by the client capability"
    );
}

fn assert_work_done_progress_triplet(outputs: &[Value]) -> Option<String> {
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
        2,
        "progress output must be exactly Begin + End"
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
    assert_eq!(
        progress[1].pointer("/params/token").and_then(Value::as_str),
        Some(token)
    );
    assert_eq!(
        progress[1]
            .pointer("/params/value/kind")
            .and_then(Value::as_str),
        Some("end")
    );
    assert!(
        progress[1].pointer("/params/value/percentage").is_none(),
        "End notifications must also omit percentage"
    );
    id.map(str::to_string)
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
