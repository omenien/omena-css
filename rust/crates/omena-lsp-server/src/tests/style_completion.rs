use super::*;

#[test]
fn completes_cross_file_sass_variables_from_visible_module_graph() -> TestResult {
    let app_uri = "file:///workspace-a/src/App.module.scss";
    let tokens_uri = "file:///workspace-a/src/_tokens.scss";
    let app_text = "@use \"./tokens\" as t;\n.button { border-radius: t.$ }";
    let mut state = LspShellState::default();
    open_style_document(&mut state, app_uri, app_text, 1);
    open_style_document(
        &mut state,
        tokens_uri,
        "$radius-small: 2px; $secret: 4px;",
        1,
    );

    let labels = completion_labels(
        &mut state,
        app_uri,
        parser_position_for_byte_offset(
            app_text,
            fixture_find(app_text, "t.$", "app fixture contains namespace prefix")? + 3,
        ),
    )?;
    assert!(
        labels.contains(&"t.$radius-small".to_string()),
        "{labels:?}"
    );
    assert!(!labels.contains(&"$radius-small".to_string()), "{labels:?}");

    Ok(())
}

#[test]
fn keeps_sass_symbols_out_of_custom_property_var_completion() -> TestResult {
    let app_uri = "file:///workspace-a/src/App.module.scss";
    let tokens_uri = "file:///workspace-a/src/_tokens.scss";
    let app_text = "@use \"./tokens\" as t;\n:root { --brand: red; }\n.button { color: var(--); }";
    let mut state = LspShellState::default();
    open_style_document(&mut state, app_uri, app_text, 1);
    open_style_document(&mut state, tokens_uri, "$brand: blue;", 1);

    let labels = completion_labels(
        &mut state,
        app_uri,
        parser_position_for_byte_offset(
            app_text,
            fixture_find(
                app_text,
                "var(--",
                "app fixture contains custom-property prefix",
            )? + 6,
        ),
    )?;
    assert_eq!(labels, vec!["--brand".to_string()]);

    Ok(())
}

#[test]
fn refreshes_cross_file_sass_completion_after_peer_style_edit() -> TestResult {
    let app_uri = "file:///workspace-a/src/App.module.scss";
    let tokens_uri = "file:///workspace-a/src/_tokens.scss";
    let app_text = "@use \"./tokens\" as t;\n.button { border-radius: t.$ }";
    let mut state = LspShellState::default();
    open_style_document(&mut state, app_uri, app_text, 1);
    open_style_document(&mut state, tokens_uri, "$radius-small: 2px;", 1);

    let position = parser_position_for_byte_offset(
        app_text,
        fixture_find(app_text, "t.$", "app fixture contains namespace prefix")? + 3,
    );
    let before = completion_labels(&mut state, app_uri, position)?;
    assert!(
        before.contains(&"t.$radius-small".to_string()),
        "{before:?}"
    );
    assert!(
        !before.contains(&"t.$radius-large".to_string()),
        "{before:?}"
    );

    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {
                    "uri": tokens_uri,
                    "version": 2,
                },
                "contentChanges": [
                    {
                        "text": "$radius-small: 2px; $radius-large: 8px;",
                    },
                ],
            },
        }),
    );

    let after = completion_labels(&mut state, app_uri, position)?;
    assert!(after.contains(&"t.$radius-small".to_string()), "{after:?}");
    assert!(after.contains(&"t.$radius-large".to_string()), "{after:?}");

    Ok(())
}

fn open_style_document(state: &mut LspShellState, uri: &str, text: &str, version: i64) {
    handle_lsp_message(
        state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "languageId": "scss",
                    "version": version,
                    "text": text,
                },
            },
        }),
    );
}

fn completion_labels(
    state: &mut LspShellState,
    uri: &str,
    position: ParserPositionV0,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let response = handle_lsp_message(
        state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "textDocument/completion",
            "params": {
                "textDocument": {
                    "uri": uri,
                },
                "position": position,
            },
        }),
    );
    let items = response
        .as_ref()
        .and_then(|value| value.pointer("/result/items"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("completion response should contain items"))?;
    Ok(items
        .iter()
        .filter_map(|item| item.pointer("/label").and_then(Value::as_str))
        .map(ToString::to_string)
        .collect())
}
