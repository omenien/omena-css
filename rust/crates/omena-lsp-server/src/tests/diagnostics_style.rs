use super::*;

#[test]
fn resolves_style_diagnostics_and_code_actions_from_opened_style_documents() {
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
                    "text": ":root { --brand: red; }\n.alert { color: var(--missing); }",
                },
            },
        }),
    );

    let diagnostics_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": STYLE_DIAGNOSTICS_REQUEST,
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
            },
        }),
    );
    assert_eq!(
        diagnostics_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/message")),
        Some(&json!(
            "CSS custom property '--missing' not found in indexed style tokens."
        )),
    );
    assert_eq!(
        diagnostics_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/range")),
        Some(&json!({
            "start": {
                "line": 1,
                "character": 20,
            },
            "end": {
                "line": 1,
                "character": 29,
            },
        })),
    );
    assert_eq!(
        diagnostics_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/data/createCustomProperty/range")),
        Some(&json!({
            "start": {
                "line": 1,
                "character": 33,
            },
            "end": {
                "line": 1,
                "character": 33,
            },
        })),
    );
    assert_eq!(
        diagnostics_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/data/querySeverity")),
        Some(&json!("warning")),
    );
    assert_eq!(
        diagnostics_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/data/provenance/0")),
        Some(&json!("omena-parser.custom-property-facts")),
    );

    let diagnostic = diagnostics_response
        .as_ref()
        .and_then(|value| value.pointer("/result/0"))
        .cloned()
        .unwrap_or(Value::Null);
    let code_action_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/codeAction",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "range": {
                    "start": {
                        "line": 1,
                        "character": 20,
                    },
                    "end": {
                        "line": 1,
                        "character": 29,
                    },
                },
                "context": {
                    "diagnostics": [diagnostic],
                },
            },
        }),
    );
    assert_eq!(
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/title")),
        Some(&json!("Add '--missing' to App.module.scss")),
    );
    assert_eq!(
        code_action_response.as_ref().and_then(|value| {
            value.pointer(
                "/result/0/edit/changes/file:~1~1~1workspace-a~1src~1App.module.scss/0/newText",
            )
        }),
        Some(&json!("\n\n:root {\n  --missing: ;\n}\n")),
    );
    assert_eq!(
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/data/source")),
        Some(&json!("omenaQueryStyleDiagnosticsForFile")),
    );
    assert_eq!(
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer("/result/1/title")),
        Some(&json!("Suppress this diagnostic on the next line")),
    );
    assert_eq!(
        code_action_response.as_ref().and_then(|value| {
            value.pointer(
                "/result/1/edit/changes/file:~1~1~1workspace-a~1src~1App.module.scss/0/newText",
            )
        }),
        Some(&json!(
            "/* omena-ignore-next-line missingCustomProperty [reason: 'TODO'] */\n"
        )),
    );
    assert_eq!(
        code_action_response
            .as_ref()
            .and_then(|value| value.pointer("/result/1/data/source")),
        Some(&json!("omenaLspDiagnosticSuppressionCodeAction")),
    );
}
