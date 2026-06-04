use super::*;

#[test]
fn resolves_style_extract_code_actions_from_omena_query() {
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
                    "text": ".button { color: #ff0000; margin: 1rem; }",
                },
            },
        }),
    );

    let code_action_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "textDocument/codeAction",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "range": {
                    "start": {
                        "line": 0,
                        "character": 17,
                    },
                    "end": {
                        "line": 0,
                        "character": 24,
                    },
                },
                "context": {
                    "diagnostics": [],
                },
            },
        }),
    );

    assert_eq!(
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/title")),
        Some(&json!("Extract CSS custom property '--extracted-color'")),
    );
    assert_eq!(
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer("/result/1/title")),
        Some(&json!("Extract @value 'extractedColor'")),
    );
    assert_eq!(
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer(
                "/result/0/edit/changes/file:~1~1~1workspace-a~1src~1App.module.scss/0/newText"
            )),
        Some(&json!(":root {\n  --extracted-color: #ff0000;\n}\n\n")),
    );
    assert_eq!(
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer(
                "/result/0/edit/changes/file:~1~1~1workspace-a~1src~1App.module.scss/1/newText"
            )),
        Some(&json!("var(--extracted-color)")),
    );
    assert_eq!(
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/data/source")),
        Some(&json!("omenaQueryStyleExtractCodeActions")),
    );
}

#[test]
fn resolves_style_inline_code_actions_from_omena_query() {
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
                    "text": ".button {\n  composes: base;\n  color: red;\n}\n.base {\n  color: blue;\n  margin: 1rem;\n}",
                },
            },
        }),
    );

    let code_action_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "textDocument/codeAction",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "range": {
                    "start": {
                        "line": 1,
                        "character": 12,
                    },
                    "end": {
                        "line": 1,
                        "character": 16,
                    },
                },
                "context": {
                    "diagnostics": [],
                },
            },
        }),
    );

    assert_eq!(
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/title")),
        Some(&json!("Inline composed class 'base'")),
    );
    assert_eq!(
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/kind")),
        Some(&json!("refactor.inline")),
    );
    assert_eq!(
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer(
                "/result/0/edit/changes/file:~1~1~1workspace-a~1src~1App.module.scss/0/newText"
            )),
        Some(&json!("color: blue;\n  margin: 1rem;")),
    );
    assert_eq!(
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/data/source")),
        Some(&json!("omenaQueryStyleInlineCodeActions")),
    );
}

#[test]
fn resolves_style_insight_code_actions_from_omena_query() {
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
                    "text": ".button {\n  margin-top: 1px;\n  margin-right: 2px;\n  margin-bottom: 3px;\n  margin-left: 4px;\n}",
                },
            },
        }),
    );

    let code_action_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "textDocument/codeAction",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "range": {
                    "start": {
                        "line": 2,
                        "character": 4,
                    },
                    "end": {
                        "line": 2,
                        "character": 4,
                    },
                },
                "context": {
                    "diagnostics": [],
                },
            },
        }),
    );

    assert_eq!(
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/title")),
        Some(&json!("Combine margin longhands into shorthand")),
    );
    assert_eq!(
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/kind")),
        Some(&json!("quickfix")),
    );
    assert_eq!(
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer(
                "/result/0/edit/changes/file:~1~1~1workspace-a~1src~1App.module.scss/0/newText"
            )),
        Some(&json!("margin: 1px 2px 3px 4px")),
    );
    assert_eq!(
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/data/source")),
        Some(&json!("omenaQueryStyleInsightCodeActions")),
    );
}
