use super::*;

#[test]
fn refreshes_source_bindings_after_bundler_config_watch_change() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-bundler-alias-refresh-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let old_style_path = workspace_path
        .join("src")
        .join("old")
        .join("some-component.module.scss");
    let new_style_path = workspace_path
        .join("src")
        .join("new")
        .join("some-component.module.scss");
    let config_path = workspace_path.join("vite.config.ts");
    fs::create_dir_all(fixture_parent(
        old_style_path.as_path(),
        "old style fixture path has parent directory",
    )?)?;
    fs::create_dir_all(fixture_parent(
        new_style_path.as_path(),
        "new style fixture path has parent directory",
    )?)?;
    fs::write(
        config_path.as_path(),
        r#"export default { resolve: { alias: { "@styles": "./src/old" } } };"#,
    )?;
    fs::write(old_style_path.as_path(), ".article { display: block; }")?;
    fs::write(new_style_path.as_path(), ".article { color: green; }")?;

    let workspace_uri = path_to_file_uri(workspace_path.as_path());
    let source_uri = path_to_file_uri(workspace_path.join("src/App.tsx").as_path());
    let old_style_uri = path_to_file_uri(old_style_path.as_path());
    let new_style_uri = path_to_file_uri(new_style_path.as_path());
    let config_uri = path_to_file_uri(config_path.as_path());

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
                        "name": "workspace-a",
                    },
                ],
            },
        }),
    );
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                    "languageId": "typescriptreact",
                    "version": 1,
                    "text": "import bind from \"classnames/bind\";\nimport styles from \"@styles/some-component.module.scss\";\nconst cx = bind.bind(styles);\nexport const className = cx(\"article\");",
                },
            },
        }),
    );
    for (uri, text) in [
        (old_style_uri.as_str(), ".article { display: block; }"),
        (new_style_uri.as_str(), ".article { color: green; }"),
    ] {
        handle_lsp_message(
            &mut state,
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

    assert_source_binding_target(&state, source_uri.as_str(), old_style_uri.as_str());

    fs::write(
        config_path.as_path(),
        r#"export default { resolve: { alias: { "@styles": "./src/new" } } };"#,
    )?;
    let outputs = handle_lsp_message_outputs(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeWatchedFiles",
            "params": {
                "changes": [
                    {
                        "uri": config_uri,
                        "type": 2,
                    },
                ],
            },
        }),
    );

    assert_source_binding_target(&state, source_uri.as_str(), new_style_uri.as_str());
    assert!(
        outputs
            .iter()
            .any(|output| output.pointer("/params/uri") == Some(&json!(source_uri)))
    );

    let definition_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": {
                    "line": 3,
                    "character": 31,
                },
            },
        }),
    );

    assert_eq!(
        definition_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/uri")),
        Some(&json!(new_style_uri)),
    );
    assert_eq!(
        definition_response
            .as_ref()
            .and_then(|value| value.pointer("/result/1/uri")),
        None,
    );

    let _ = fs::remove_dir_all(workspace_path.as_path());
    Ok(())
}

#[test]
fn refreshes_source_bindings_after_tsconfig_watch_change() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-tsconfig-alias-refresh-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let old_style_path = workspace_path
        .join("src")
        .join("old")
        .join("some-component.module.scss");
    let new_style_path = workspace_path
        .join("src")
        .join("new")
        .join("some-component.module.scss");
    let config_path = workspace_path.join("tsconfig.json");
    fs::create_dir_all(fixture_parent(
        old_style_path.as_path(),
        "old style fixture path has parent directory",
    )?)?;
    fs::create_dir_all(fixture_parent(
        new_style_path.as_path(),
        "new style fixture path has parent directory",
    )?)?;
    fs::write(
        config_path.as_path(),
        r#"{"compilerOptions":{"baseUrl":".","paths":{"$styles/*":["src/old/*"]}}}"#,
    )?;
    fs::write(old_style_path.as_path(), ".article { display: block; }")?;
    fs::write(new_style_path.as_path(), ".article { color: green; }")?;

    let workspace_uri = path_to_file_uri(workspace_path.as_path());
    let source_uri = path_to_file_uri(workspace_path.join("src/App.tsx").as_path());
    let old_style_uri = path_to_file_uri(old_style_path.as_path());
    let new_style_uri = path_to_file_uri(new_style_path.as_path());
    let config_uri = path_to_file_uri(config_path.as_path());

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
                        "name": "workspace-a",
                    },
                ],
            },
        }),
    );
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                    "languageId": "typescriptreact",
                    "version": 1,
                    "text": "import bind from \"classnames/bind\";\nimport styles from \"$styles/some-component.module.scss\";\nconst cx = bind.bind(styles);\nexport const className = cx(\"article\");",
                },
            },
        }),
    );
    for (uri, text) in [
        (old_style_uri.as_str(), ".article { display: block; }"),
        (new_style_uri.as_str(), ".article { color: green; }"),
    ] {
        handle_lsp_message(
            &mut state,
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

    assert_source_binding_target(&state, source_uri.as_str(), old_style_uri.as_str());

    fs::write(
        config_path.as_path(),
        r#"{"compilerOptions":{"baseUrl":".","paths":{"$styles/*":["src/new/*"]}}}"#,
    )?;
    let outputs = handle_lsp_message_outputs(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeWatchedFiles",
            "params": {
                "changes": [
                    {
                        "uri": config_uri,
                        "type": 2,
                    },
                ],
            },
        }),
    );

    assert_source_binding_target(&state, source_uri.as_str(), new_style_uri.as_str());
    assert!(
        outputs
            .iter()
            .any(|output| output.pointer("/params/uri") == Some(&json!(source_uri)))
    );

    let definition_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": {
                    "line": 3,
                    "character": 31,
                },
            },
        }),
    );
    assert_definition_response_single_target(&definition_response, new_style_uri.as_str());

    let _ = fs::remove_dir_all(workspace_path.as_path());
    Ok(())
}
