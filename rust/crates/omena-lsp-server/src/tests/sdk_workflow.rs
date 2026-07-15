use super::*;

const WORKSPACE_ROOT: &str = "file:///workspace-a";
const STYLE_URI: &str = "file:///workspace-a/src/App.module.scss";

#[test]
fn sdk_workflows_read_the_open_document_snapshot() {
    let mut state = LspShellState::default();
    open_style(&mut state, 1, ".card { color: red; }");

    let snapshot = request(
        &mut state,
        1,
        "snapshot",
        json!({
            "workspaceRoot": WORKSPACE_ROOT,
        }),
    );
    let snapshot_id = snapshot.pointer("/result/snapshotId").cloned().unwrap();

    let diagnostics = request(
        &mut state,
        2,
        "diagnostics",
        json!({
            "snapshotId": snapshot_id,
            "stylePath": STYLE_URI,
            "styleSource": ".card { color: red; }",
        }),
    );
    assert_eq!(
        diagnostics.pointer("/result/summary/classSelectorCount"),
        Some(&json!(1)),
    );
    assert_eq!(
        diagnostics.pointer("/result/snapshotId"),
        snapshot.pointer("/result/snapshotId"),
    );

    let query = request(
        &mut state,
        3,
        "query",
        json!({
            "snapshotId": snapshot_id,
            "queryKind": "styleSummary",
            "input": { "stylePath": STYLE_URI },
        }),
    );
    assert_eq!(
        query.pointer("/result/payload/selectorNames/0"),
        Some(&json!("card")),
    );
}

#[test]
fn sdk_workflow_rejects_a_snapshot_after_document_change() {
    let mut state = LspShellState::default();
    open_style(&mut state, 1, ".card { color: red; }");
    let snapshot = request(
        &mut state,
        1,
        "snapshot",
        json!({
            "workspaceRoot": WORKSPACE_ROOT,
        }),
    );
    let snapshot_id = snapshot.pointer("/result/snapshotId").cloned().unwrap();

    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": { "uri": STYLE_URI, "version": 2 },
                "contentChanges": [{ "text": ".card { color: blue; }" }],
            },
        }),
    );

    let response = request(
        &mut state,
        2,
        "query",
        json!({
            "snapshotId": snapshot_id,
            "queryKind": "styleSummary",
            "input": { "stylePath": STYLE_URI },
        }),
    );
    assert_eq!(
        response.pointer("/error/data/error/class"),
        Some(&json!("workspace")),
    );
    assert_eq!(
        response.pointer("/error/data/error/context/code"),
        Some(&json!("workspace.snapshot-mismatch")),
    );
}

#[test]
fn sdk_workflow_keeps_workspace_roots_isolated() {
    let mut state = LspShellState::default();
    open_style(&mut state, 1, ".card { color: red; }");
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-b/src/Other.module.scss",
                    "languageId": "scss",
                    "version": 1,
                    "text": ".other { color: blue; }",
                },
            },
        }),
    );
    let snapshot = request(
        &mut state,
        1,
        "snapshot",
        json!({
            "workspaceRoot": WORKSPACE_ROOT,
        }),
    );
    let response = request(
        &mut state,
        2,
        "query",
        json!({
            "snapshotId": snapshot["result"]["snapshotId"],
            "queryKind": "styleSummary",
            "input": { "stylePath": "file:///workspace-b/src/Other.module.scss" },
        }),
    );
    assert_eq!(
        response.pointer("/error/data/error/context/code"),
        Some(&json!("workspace.style-path-not-found")),
    );
}

fn open_style(state: &mut LspShellState, version: i64, text: &str) {
    handle_lsp_message(
        state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": STYLE_URI,
                    "languageId": "scss",
                    "version": version,
                    "text": text,
                },
            },
        }),
    );
}

fn request(state: &mut LspShellState, id: u64, operation: &str, request: Value) -> Value {
    handle_lsp_message(
        state,
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": SDK_WORKFLOW_REQUEST,
            "params": {
                "workspaceRoot": WORKSPACE_ROOT,
                "operation": operation,
                "request": request,
            },
        }),
    )
    .unwrap()
}
