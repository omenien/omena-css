use super::*;

#[test]
fn cancels_queued_requests_before_provider_work() {
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": CANCEL_REQUEST_METHOD,
            "params": {
                "id": "hover-1",
            },
        }),
    );
    assert_eq!(state.snapshot().cancelled_request_count, 1);

    let response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": "hover-1",
            "method": "textDocument/hover",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "position": {
                    "line": 0,
                    "character": 2,
                },
            },
        }),
    );

    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/error/code")),
        Some(&json!(REQUEST_CANCELLED_ERROR_CODE)),
    );
    assert_eq!(state.snapshot().cancelled_request_count, 0);
}

#[test]
fn bounds_late_cancel_request_cache() {
    let mut state = LspShellState::default();
    for id in 0..=omena_incremental::DEFAULT_INCREMENTAL_CANCELLATION_LIMIT {
        handle_lsp_message(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "method": CANCEL_REQUEST_METHOD,
                "params": {
                    "id": id,
                },
            }),
        );
    }

    assert_eq!(state.snapshot().cancelled_request_count, 1);
}
