//! RFC 0009 Pillar A (rfcs#67, slice A-min): deterministic contracts for the
//! dispatched query lane — copy-on-write snapshot isolation, worker/synchronous
//! handler equivalence, the shared cascade-narrowing memo, and the loop-side
//! cancellation gate. The end-to-end dispatcher stress test (burst of
//! didChange + hover/definition through `run_stdio_server`) lives in the
//! binary's test module (`src/bin/omena-lsp-server.rs`).

use super::*;
use std::sync::Arc;

const APP_STYLE_URI: &str = "file:///workspace-q/src/App.module.scss";
const THEME_STYLE_URI: &str = "file:///workspace-q/src/_theme.scss";

fn open_query_dispatch_workspace(state: &mut LspShellState) {
    handle_lsp_message(
        state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": "file:///workspace-q",
                        "name": "workspace-q",
                    },
                ],
            },
        }),
    );
    for (uri, text) in [
        (
            APP_STYLE_URI,
            "@use \"./theme\";\n.btn { color: red; }\n.btn { color: green; }",
        ),
        (THEME_STYLE_URI, ".btn { color: blue; }"),
    ] {
        handle_lsp_message(
            state,
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": uri,
                        "languageId": "scss",
                        "version": 1,
                        "text": text,
                    },
                },
            }),
        );
    }
}

fn hover_btn_request(id: u64) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "textDocument/hover",
        "params": {
            "textDocument": {
                "uri": APP_STYLE_URI,
            },
            "position": {
                "line": 1,
                "character": 2,
            },
        },
    })
}

fn change_theme_color(state: &mut LspShellState, version: i64, text: &str) {
    handle_lsp_message(
        state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {
                    "uri": THEME_STYLE_URI,
                    "version": version,
                },
                "contentChanges": [
                    {
                        "text": text,
                    },
                ],
            },
        }),
    );
}

fn dispatched_hover_markdown(
    snapshot: &LspQuerySnapshotV0,
    id: u64,
) -> Result<String, std::io::Error> {
    let dispatch = LspQueryDispatchV0 {
        snapshot: snapshot_clone_for_test(snapshot),
        message: hover_btn_request(id),
    };
    resolve_dispatched_query_response(&dispatch)
        .as_ref()
        .and_then(|value| value.pointer("/result/contents/value"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| std::io::Error::other("dispatched hover should render markdown"))
}

// `LspQuerySnapshotV0` is intentionally not `Clone` in production (a dispatch
// owns its snapshot); tests re-derive an identical view through the same
// pointer-clone constructor the loop uses.
fn snapshot_clone_for_test(snapshot: &LspQuerySnapshotV0) -> LspQuerySnapshotV0 {
    snapshot.shell_state().query_snapshot()
}

#[test]
fn query_snapshot_clones_document_pointers_and_isolates_later_edits() -> TestResult {
    let mut state = LspShellState::default();
    open_query_dispatch_workspace(&mut state);

    let snapshot = state.query_snapshot();
    for (file_id, document) in &state.documents {
        let snapshot_document = snapshot
            .shell_state()
            .documents
            .get(file_id)
            .ok_or_else(|| std::io::Error::other("snapshot must carry every document"))?;
        assert!(
            Arc::ptr_eq(document, snapshot_document),
            "snapshot must clone document pointers, not the corpus: {}",
            document.uri
        );
    }

    change_theme_color(&mut state, 2, ".btn { color: purple; }");

    // The edit copy-on-writes ONLY the edited document; the untouched document
    // still shares storage with the snapshot.
    let theme_storage_uri = LspShellState::document_storage_uri(THEME_STYLE_URI);
    let app_storage_uri = LspShellState::document_storage_uri(APP_STYLE_URI);
    let theme_file_id = state
        .document_file_id(THEME_STYLE_URI)
        .ok_or_else(|| std::io::Error::other("theme document must be interned"))?;
    let app_file_id = state
        .document_file_id(APP_STYLE_URI)
        .ok_or_else(|| std::io::Error::other("app document must be interned"))?;
    assert!(
        !Arc::ptr_eq(
            &state.documents[&theme_file_id],
            &snapshot.shell_state().documents[&theme_file_id],
        ),
        "edited document must be copy-on-write detached from the snapshot: {theme_storage_uri}"
    );
    assert!(
        Arc::ptr_eq(
            &state.documents[&app_file_id],
            &snapshot.shell_state().documents[&app_file_id],
        ),
        "untouched document must still share storage with the snapshot: {app_storage_uri}"
    );

    // The stale snapshot keeps answering from its dispatch-time corpus (LSP
    // allows this; clients re-request after edits), while a fresh snapshot sees
    // the edit.
    let stale_markdown = dispatched_hover_markdown(&snapshot, 11)?;
    assert!(
        stale_markdown.contains("`blue`") && !stale_markdown.contains("`purple`"),
        "stale snapshot must answer from its dispatch-time corpus: {stale_markdown}"
    );
    let fresh_markdown = dispatched_hover_markdown(&state.query_snapshot(), 12)?;
    assert!(
        fresh_markdown.contains("`purple`") && !fresh_markdown.contains("`blue`"),
        "fresh snapshot must reflect the edit: {fresh_markdown}"
    );
    Ok(())
}

#[test]
fn dispatched_query_response_on_worker_thread_matches_synchronous_handler() -> TestResult {
    let mut state = LspShellState::default();
    open_query_dispatch_workspace(&mut state);

    let dispatch = LspQueryDispatchV0 {
        snapshot: state.query_snapshot(),
        message: hover_btn_request(21),
    };
    let worker_response = std::thread::spawn(move || resolve_dispatched_query_response(&dispatch))
        .join()
        .map_err(|_| std::io::Error::other("query worker test thread panicked"))?;
    let synchronous_response = handle_lsp_message(&mut state, hover_btn_request(21));

    assert_eq!(
        worker_response, synchronous_response,
        "worker resolution from the snapshot must match the synchronous handler"
    );
    assert!(
        worker_response.is_some(),
        "hover request must produce a response"
    );
    Ok(())
}

#[test]
fn dispatched_query_shares_cascade_narrowing_memo_with_loop() -> TestResult {
    let mut state = LspShellState::default();
    open_query_dispatch_workspace(&mut state);

    let snapshot = state.query_snapshot();
    assert!(
        Arc::ptr_eq(
            &state.cascade_narrowing_substrate_memo,
            &snapshot.shell_state().cascade_narrowing_substrate_memo,
        ),
        "snapshot must share the memo handle, not copy it"
    );
    assert!(
        state.cascade_narrowing_substrate_memo_lock().is_none(),
        "no hover ran yet, the memo must be empty"
    );

    let _ = dispatched_hover_markdown(&snapshot, 31)?;
    assert!(
        state.cascade_narrowing_substrate_memo_lock().is_some(),
        "a substrate built on the dispatched lane must be visible to the loop"
    );
    Ok(())
}

#[test]
fn dispatch_gate_classifies_requests_and_honors_loop_side_cancellation() -> TestResult {
    let mut state = LspShellState::default();
    open_query_dispatch_workspace(&mut state);

    // A hover/definition REQUEST is dispatched with a snapshot.
    let LspLoopTurnV0::DispatchQuery(dispatch) =
        handle_lsp_message_scheduled_outputs_or_dispatch(&mut state, hover_btn_request(41))
    else {
        return Err(
            std::io::Error::other("hover request must dispatch, not resolve inline").into(),
        );
    };
    assert_eq!(
        dispatch.message.get("id"),
        Some(&json!(41)),
        "dispatch must carry the original request"
    );

    // A request already cancelled when it arrives is answered on the loop and
    // never dispatched.
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "$/cancelRequest",
            "params": {
                "id": 42,
            },
        }),
    );
    let LspLoopTurnV0::Outputs(outputs) =
        handle_lsp_message_scheduled_outputs_or_dispatch(&mut state, hover_btn_request(42))
    else {
        return Err(std::io::Error::other("pre-cancelled request must never be dispatched").into());
    };
    let codes = outputs
        .iter()
        .map(|output| output.value.pointer("/error/code").cloned())
        .collect::<Vec<_>>();
    assert_eq!(
        codes,
        vec![Some(json!(REQUEST_CANCELLED_ERROR_CODE))],
        "pre-cancelled request must get the cancelled error on the loop"
    );

    // A notification spelled like a dispatchable method (no id) stays on the
    // synchronous path: nothing to respond to.
    let notification = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/hover",
        "params": {
            "textDocument": {
                "uri": APP_STYLE_URI,
            },
            "position": {
                "line": 1,
                "character": 2,
            },
        },
    });
    let LspLoopTurnV0::Outputs(outputs) =
        handle_lsp_message_scheduled_outputs_or_dispatch(&mut state, notification)
    else {
        return Err(std::io::Error::other("a notification must never be dispatched").into());
    };
    assert_eq!(outputs, Vec::new());
    Ok(())
}
