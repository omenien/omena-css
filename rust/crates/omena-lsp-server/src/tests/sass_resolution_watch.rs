use super::*;

#[test]
fn keeps_sass_resolution_on_cached_bundler_config_until_watch_change() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-bundler-alias-request-cache-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let source_style_path = workspace_path.join("src/App.module.scss");
    let old_style_path = workspace_path.join("src/old/_tokens.scss");
    let new_style_path = workspace_path.join("src/new/_tokens.scss");
    let config_path = workspace_path.join("vite.config.ts");
    fs::create_dir_all(fixture_parent(
        source_style_path.as_path(),
        "source style fixture path has parent directory",
    )?)?;
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
    let source_text = r#"@use "@styles/tokens" as tokens;
.button { color: tokens.$brand; }
"#;
    fs::write(source_style_path.as_path(), source_text)?;
    fs::write(old_style_path.as_path(), "$brand: red;\n")?;
    fs::write(new_style_path.as_path(), "$brand: green;\n")?;

    let workspace_uri = path_to_file_uri(workspace_path.as_path());
    let source_uri = path_to_file_uri(source_style_path.as_path());
    let old_style_uri = path_to_file_uri(old_style_path.as_path());
    let new_style_uri = path_to_file_uri(new_style_path.as_path());
    let config_uri = path_to_file_uri(config_path.as_path());
    let brand_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "$brand",
            "source fixture contains Sass variable reference",
        )? + 1,
    );

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
    for (uri, text) in [
        (source_uri.as_str(), source_text),
        (old_style_uri.as_str(), "$brand: red;\n"),
        (new_style_uri.as_str(), "$brand: green;\n"),
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

    fs::write(
        config_path.as_path(),
        r#"export default { resolve: { alias: { "@styles": "./src/new" } } };"#,
    )?;
    let cached_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": brand_position,
            },
        }),
    );
    assert_definition_response_single_target(&cached_definition, old_style_uri.as_str());

    handle_lsp_message(
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
    let refreshed_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": brand_position,
            },
        }),
    );
    assert_definition_response_single_target(&refreshed_definition, new_style_uri.as_str());

    let _ = fs::remove_dir_all(workspace_path.as_path());
    Ok(())
}

#[test]
fn resolves_sass_definition_after_package_manifest_watch_change() -> TestResult {
    let root = std::env::temp_dir().join(format!(
        "omena_lsp_package_manifest_refresh_{}_{}",
        std::process::id(),
        current_time_millis()
    ));
    let source = root.join("src/App.module.scss");
    let package_root = root.join("node_modules/@design/tokens");
    let old_style = package_root.join("old.scss");
    let new_style = package_root.join("new.scss");
    let package_json = package_root.join("package.json");
    fs::create_dir_all(fixture_parent(source.as_path(), "source parent")?)?;
    fs::create_dir_all(fixture_parent(old_style.as_path(), "style parent")?)?;
    fs::write(package_json.as_path(), r#"{"sass":"old.scss"}"#)?;
    let source_text = r#"@use "@design/tokens" as tokens;
.button { color: tokens.$brand; }
"#;
    fs::write(source.as_path(), source_text)?;
    fs::write(old_style.as_path(), "$brand: red;\n")?;
    fs::write(new_style.as_path(), "$brand: green;\n")?;

    let workspace_uri = path_to_file_uri(root.as_path());
    let source_uri = path_to_file_uri(source.as_path());
    let old_style_uri = path_to_file_uri(old_style.as_path());
    let new_style_uri = path_to_file_uri(new_style.as_path());
    let package_json_uri = path_to_file_uri(package_json.as_path());
    let brand_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "$brand",
            "source fixture contains Sass variable reference",
        )? + 1,
    );
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
        (old_style_uri.as_str(), "$brand: red;\n"),
        (new_style_uri.as_str(), "$brand: green;\n"),
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

    let initial_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": brand_position,
            },
        }),
    );
    assert_definition_response_single_target(&initial_definition, old_style_uri.as_str());

    fs::write(package_json.as_path(), r#"{"sass":"new.scss"}"#)?;
    let cached_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": brand_position,
            },
        }),
    );
    assert_definition_response_single_target(&cached_definition, old_style_uri.as_str());

    let outputs = handle_lsp_message_outputs(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeWatchedFiles",
            "params": {
                "changes": [
                    {
                        "uri": package_json_uri,
                        "type": 2,
                    },
                ],
            },
        }),
    );
    assert!(
        outputs
            .iter()
            .any(|output| output.pointer("/params/uri") == Some(&json!(source_uri)))
    );

    let updated_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": brand_position,
            },
        }),
    );
    assert_definition_response_single_target(&updated_definition, new_style_uri.as_str());

    let _ = fs::remove_dir_all(root.as_path());
    Ok(())
}
