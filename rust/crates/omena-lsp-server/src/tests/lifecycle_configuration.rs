use super::*;

#[test]
fn honors_feature_configuration_toggles() {
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
                    "text": ".root { color: red; }",
                },
            },
        }),
    );

    let enabled_hover = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
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
        enabled_hover
            .as_ref()
            .and_then(|value| value.pointer("/result/contents/kind")),
        Some(&json!("markdown")),
    );

    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeConfiguration",
            "params": {
                "settings": {
                    "omena": {
                        "features": {
                            "hover": false,
                        },
                        "diagnostics": {
                            "severity": "error",
                            "deepAnalysis": true,
                        },
                    },
                },
            },
        }),
    );

    let disabled_hover = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
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
        disabled_hover
            .as_ref()
            .and_then(|value| value.get("result")),
        Some(&Value::Null),
    );
    assert!(!state.snapshot().features.hover);
    assert_eq!(state.snapshot().diagnostics.severity, 1);
    assert!(state.snapshot().diagnostics.deep_analysis);
}
