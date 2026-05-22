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
