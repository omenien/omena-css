use super::*;

#[test]
fn resolves_sass_definition_with_configured_package_manifest_path() -> TestResult {
    let root = std::env::temp_dir().join(format!(
        "omena_lsp_package_manifest_setting_{}_{}",
        std::process::id(),
        current_time_millis()
    ));
    let source = root.join("src/App.module.scss");
    let package_root = root.join("node_modules/@design/tokens");
    let override_style = package_root.join("override.scss");
    let override_manifest = package_root.join("package.lsp.json");
    fs::create_dir_all(fixture_parent(source.as_path(), "source parent")?)?;
    fs::create_dir_all(package_root.as_path())?;
    let source_text = r#"@use "pkg:@design/tokens" as tokens;
.button { color: tokens.$brand; }
"#;
    fs::write(source.as_path(), source_text)?;
    fs::write(override_style.as_path(), "$brand: green;\n")?;
    fs::write(override_manifest.as_path(), r#"{"sass":"./override.scss"}"#)?;

    let workspace_uri = path_to_file_uri(root.as_path());
    let source_uri = path_to_file_uri(source.as_path());
    let override_style_uri = path_to_file_uri(override_style.as_path());
    let override_manifest_path = override_manifest.to_string_lossy().to_string();
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
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeConfiguration",
            "params": {
                "settings": {
                    "cssModuleExplainer": {
                        "resolution": {
                            "packageManifestPaths": [override_manifest_path],
                        },
                    },
                },
            },
        }),
    );
    assert!(
        state
            .snapshot()
            .resolution
            .package_manifest_paths
            .iter()
            .any(|path| path.ends_with("node_modules/@design/tokens/package.lsp.json"))
    );

    for (uri, text) in [
        (source_uri.as_str(), source_text),
        (override_style_uri.as_str(), "$brand: green;\n"),
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

    let definition = handle_lsp_message(
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
    assert_definition_response_single_target(&definition, override_style_uri.as_str());

    let _ = fs::remove_dir_all(root.as_path());
    Ok(())
}

#[test]
fn resolves_sass_definition_through_package_imports() -> TestResult {
    let root = std::env::temp_dir().join(format!(
        "omena_lsp_package_imports_{}_{}",
        std::process::id(),
        current_time_millis()
    ));
    let source = root.join("src/App.module.scss");
    let package_root = root.join("node_modules/@design/tokens");
    let target_style = package_root.join("dist/theme.scss");
    fs::create_dir_all(fixture_parent(source.as_path(), "source parent")?)?;
    fs::create_dir_all(fixture_parent(target_style.as_path(), "target parent")?)?;
    fs::write(
        root.join("package.json"),
        r##"{"imports":{"#theme":"@design/tokens/theme"}}"##,
    )?;
    fs::write(
        package_root.join("package.json"),
        r#"{"exports":{"./theme":{"sass":"./dist/theme.scss"}}}"#,
    )?;
    let source_text = r##"@use "#theme" as tokens;
.button { color: tokens.$brand; }
"##;
    let target_text = "$brand: green;\n";
    fs::write(source.as_path(), source_text)?;
    fs::write(target_style.as_path(), target_text)?;

    let workspace_uri = path_to_file_uri(root.as_path());
    let source_uri = path_to_file_uri(source.as_path());
    let target_style_uri = path_to_file_uri(target_style.as_path());
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
        (target_style_uri.as_str(), target_text),
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

    let definition = handle_lsp_message(
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
    assert_definition_response_single_target(&definition, target_style_uri.as_str());

    let _ = fs::remove_dir_all(root.as_path());
    Ok(())
}
