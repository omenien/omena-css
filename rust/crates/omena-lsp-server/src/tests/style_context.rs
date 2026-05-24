use super::*;

#[test]
fn resolves_query_owned_cascade_and_context_requests_from_opened_style_documents() {
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                    "languageId": "scss",
                    "version": 1,
                    "text": ":root { --surface: white; }\n:root { --surface: black; }\n@layer components {\n  @container card (min-width: 20rem) {\n    @scope (.button) {\n      .button { color: var(--surface); }\n    }\n  }\n}\n",
                },
            },
        }),
    );

    let cascade_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": CASCADE_AT_POSITION_REQUEST,
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "position": {
                    "line": 5,
                    "character": 28,
                },
            },
        }),
    );
    assert_eq!(
        cascade_response
            .as_ref()
            .and_then(|value| value.pointer("/result/product")),
        Some(&json!("omena-query.read-cascade-at-position")),
    );
    assert_eq!(
        cascade_response
            .as_ref()
            .and_then(|value| value.pointer("/result/status")),
        Some(&json!("resolved")),
    );
    assert_eq!(
        cascade_response
            .as_ref()
            .and_then(|value| value.pointer("/result/referenceName")),
        Some(&json!("--surface")),
    );
    assert_eq!(
        cascade_response
            .as_ref()
            .and_then(|value| value.pointer("/result/cascadeEngine")),
        Some(&json!("omena-cascade")),
    );
    assert_eq!(
        cascade_response
            .as_ref()
            .and_then(|value| value.pointer("/result/categoricalEvidence")),
        None,
    );

    let categorical_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": CASCADE_AT_POSITION_REQUEST,
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "position": {
                    "line": 5,
                    "character": 28,
                },
                "context": {
                    "includeCategoricalEvidence": true,
                },
            },
        }),
    );
    assert_eq!(
        categorical_response
            .as_ref()
            .and_then(|value| value.pointer("/result/categoricalEvidence/product")),
        Some(&json!("omena-categorical.cascade-evidence")),
    );
    assert_eq!(
        categorical_response
            .as_ref()
            .and_then(|value| value.pointer("/result/categoricalEvidence/endpointCount")),
        Some(&json!(10)),
    );

    let context_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": STYLE_CONTEXT_INDEX_REQUEST,
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
            },
        }),
    );
    assert_eq!(
        context_response
            .as_ref()
            .and_then(|value| value.pointer("/result/product")),
        Some(&json!("omena-query.style-context-index")),
    );
    assert_eq!(
        context_response
            .as_ref()
            .and_then(|value| value.pointer("/result/contextIndexSource")),
        Some(&json!("omena-semantic.style-context-index")),
    );
    assert_eq!(
        context_response
            .as_ref()
            .and_then(|value| value.pointer("/result/contextIndex/layerIndex/namedLayerCount")),
        Some(&json!(1)),
    );
    assert_eq!(
        context_response.as_ref().and_then(|value| {
            value.pointer("/result/contextIndex/containerIndex/namedContainerCount")
        }),
        Some(&json!(1)),
    );
    assert_eq!(
        context_response
            .as_ref()
            .and_then(|value| value.pointer("/result/contextIndex/scopeIndex/scopes"))
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(1),
    );
    assert_eq!(
        context_response
            .as_ref()
            .and_then(|value| value.pointer("/result/contextIndex/scopeIndex/scopedSelectorCount")),
        Some(&json!(1)),
    );
}
