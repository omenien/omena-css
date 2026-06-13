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

#[test]
fn completes_sass_symbols_through_package_alias_forward_chain() -> TestResult {
    let root = std::env::temp_dir().join(format!(
        "omena-lsp-completion-package-forward-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let source = root.join("src/App.module.scss");
    let package_root = root.join("node_modules/@design/tokens");
    let index_style = package_root.join("dist/index.scss");
    let tokens_style = package_root.join("dist/_tokens.scss");
    fs::create_dir_all(fixture_parent(source.as_path(), "source parent")?)?;
    fs::create_dir_all(fixture_parent(
        index_style.as_path(),
        "package style parent",
    )?)?;
    fs::write(
        root.join("package.json"),
        r##"{"imports":{"#theme":"@design/tokens/theme"}}"##,
    )?;
    fs::write(
        package_root.join("package.json"),
        r#"{"exports":{"./theme":{"style":"./dist/index.scss"}}}"#,
    )?;
    let source_text = r##"@use "#theme" as tokens;
.button { color: tokens.$ }
"##;
    fs::write(source.as_path(), source_text)?;
    fs::write(index_style.as_path(), r#"@forward "./tokens";"#)?;
    fs::write(tokens_style.as_path(), "$brand: green;\n")?;

    let workspace_uri = path_to_file_uri(root.as_path());
    let source_uri = path_to_file_uri(source.as_path());
    let index_uri = path_to_file_uri(index_style.as_path());
    let tokens_uri = path_to_file_uri(tokens_style.as_path());
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "workspace",
                    },
                ],
            },
        }),
    );
    for (uri, text) in [
        (source_uri.as_str(), source_text),
        (index_uri.as_str(), r#"@forward "./tokens";"#),
        (tokens_uri.as_str(), "$brand: green;\n"),
    ] {
        open_style_document(&mut state, uri, text, 1);
    }
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(&state, Some(workspace_uri.as_str()));
    assert!(
        !resolution_inputs.package_manifests.is_empty(),
        "workspace package manifests should be available to completion resolution"
    );

    let labels = completion_labels(
        &mut state,
        source_uri.as_str(),
        parser_position_for_byte_offset(
            source_text,
            fixture_find(
                source_text,
                "tokens.$",
                "source fixture contains namespace prefix",
            )? + 8,
        ),
    )?;
    assert!(
        labels.contains(&"tokens.$brand".to_string()),
        "package alias + forward-chain completion should include forwarded symbol: {labels:?}"
    );

    let _ = fs::remove_dir_all(root.as_path());
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
