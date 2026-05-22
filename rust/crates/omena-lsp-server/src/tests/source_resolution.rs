use super::*;

#[test]
fn resolves_classnames_bind_source_definition_from_opened_documents() {
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
                        "uri": "file:///workspace-a",
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
                    "uri": "file:///workspace-a/src/App.tsx",
                    "languageId": "typescriptreact",
                    "version": 1,
                    "text": "import bind from \"classnames/bind\";\nimport styles from \"./styles.module.scss\";\nconst cx = bind.bind(styles);\nexport const className = cx(\"root\");",
                },
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
                    "uri": "file:///workspace-a/src/styles.module.scss",
                    "languageId": "scss",
                    "version": 1,
                    "text": ".root { display: block; }",
                },
            },
        }),
    );

    let definition_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.tsx",
                },
                "position": {
                    "line": 3,
                    "character": 30,
                },
            },
        }),
    );

    assert_eq!(
        definition_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/uri")),
        Some(&json!("file:///workspace-a/src/styles.module.scss")),
    );
    assert_eq!(
        definition_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/range")),
        Some(&json!({
            "start": {
                "line": 0,
                "character": 1,
            },
            "end": {
                "line": 0,
                "character": 5,
            },
        })),
    );
}

#[test]
fn resolves_classnames_bind_source_definition_through_tsconfig_path_alias() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-path-alias-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let target_style_path = workspace_path
        .join("src")
        .join("domain")
        .join("components")
        .join("some-component.module.scss");
    fs::create_dir_all(fixture_parent(
        target_style_path.as_path(),
        "target style fixture path has parent directory",
    )?)?;
    fs::write(
        workspace_path.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "$domain/*": ["src/domain/*"]
    }
  }
}"#,
    )?;
    fs::write(target_style_path.as_path(), ".article { display: block; }")?;

    let workspace_uri = path_to_file_uri(workspace_path.as_path());
    let source_uri = path_to_file_uri(workspace_path.join("src/App.tsx").as_path());
    let target_style_uri = path_to_file_uri(target_style_path.as_path());
    let unrelated_style_uri =
        path_to_file_uri(workspace_path.join("src/other.module.scss").as_path());

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
                    "text": "import bind from \"classnames/bind\";\nimport styles from \"$domain/components/some-component.module.scss\";\nconst cx = bind.bind(styles);\nexport const className = cx(\"article\");",
                },
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
                    "uri": target_style_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": ".article { display: block; }",
                },
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
                    "uri": unrelated_style_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": ".article { color: red; }",
                },
            },
        }),
    );

    let source_index = state
        .document(source_uri.as_str())
        .map(|document| document.source_syntax_index.clone());
    assert_eq!(
        source_index
            .as_ref()
            .map(|index| index.imported_style_bindings.as_slice()),
        Some(
            [ImportedStyleBinding {
                binding: "styles".to_string(),
                style_uri: target_style_uri.clone(),
            }]
            .as_slice()
        ),
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
        Some(&json!(target_style_uri)),
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
fn resolves_classnames_bind_source_definition_through_tsconfig_extends_alias() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-path-alias-extends-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let target_style_path = workspace_path
        .join("src")
        .join("shared")
        .join("some-component.module.scss");
    let config_dir = workspace_path.join("config");
    fs::create_dir_all(fixture_parent(
        target_style_path.as_path(),
        "target style fixture path has parent directory",
    )?)?;
    fs::create_dir_all(config_dir.as_path())?;
    fs::write(
        config_dir.join("base.json"),
        r#"{"compilerOptions":{"baseUrl":"..","paths":{"$shared/*":["src/shared/*"]}}}"#,
    )?;
    fs::write(
        workspace_path.join("tsconfig.json"),
        r#"{"extends":"./config/base"}"#,
    )?;
    fs::write(target_style_path.as_path(), ".article { display: block; }")?;

    let workspace_uri = path_to_file_uri(workspace_path.as_path());
    let source_uri = path_to_file_uri(workspace_path.join("src/App.tsx").as_path());
    let target_style_uri = path_to_file_uri(target_style_path.as_path());

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
                    "text": "import bind from \"classnames/bind\";\nimport styles from \"$shared/some-component.module.scss\";\nconst cx = bind.bind(styles);\nexport const className = cx(\"article\");",
                },
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
                    "uri": target_style_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": ".article { display: block; }",
                },
            },
        }),
    );

    assert_source_binding_target(&state, source_uri.as_str(), target_style_uri.as_str());
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
    assert_definition_response_single_target(&definition_response, target_style_uri.as_str());

    let _ = fs::remove_dir_all(workspace_path.as_path());
    Ok(())
}

#[test]
fn resolves_classnames_bind_source_definition_through_vite_bundler_alias() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-bundler-alias-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let target_style_path = workspace_path
        .join("src")
        .join("styles")
        .join("some-component.module.scss");
    fs::create_dir_all(fixture_parent(
        target_style_path.as_path(),
        "target style fixture path has parent directory",
    )?)?;
    fs::write(
        workspace_path.join("vite.config.ts"),
        r#"export default { resolve: { alias: { "@styles": "./src/styles" } } };"#,
    )?;
    fs::write(target_style_path.as_path(), ".article { display: block; }")?;

    let workspace_uri = path_to_file_uri(workspace_path.as_path());
    let source_uri = path_to_file_uri(workspace_path.join("src/App.tsx").as_path());
    let target_style_uri = path_to_file_uri(target_style_path.as_path());
    let unrelated_style_uri =
        path_to_file_uri(workspace_path.join("src/other.module.scss").as_path());

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
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": target_style_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": ".article { display: block; }",
                },
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
                    "uri": unrelated_style_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": ".article { color: red; }",
                },
            },
        }),
    );

    let source_index = state
        .document(source_uri.as_str())
        .map(|document| document.source_syntax_index.clone());
    assert_eq!(
        source_index
            .as_ref()
            .map(|index| index.imported_style_bindings.as_slice()),
        Some(
            [ImportedStyleBinding {
                binding: "styles".to_string(),
                style_uri: target_style_uri.clone(),
            }]
            .as_slice()
        ),
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
        Some(&json!(target_style_uri)),
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
fn resolves_classnames_bind_source_definition_through_webpack_bundler_alias() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-webpack-alias-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let target_style_path = workspace_path
        .join("src")
        .join("theme")
        .join("deep")
        .join("some-component.module.scss");
    let specific_style_path = workspace_path
        .join("src")
        .join("specific")
        .join("some-component.module.scss");
    fs::create_dir_all(fixture_parent(
        target_style_path.as_path(),
        "target style fixture path has parent directory",
    )?)?;
    fs::create_dir_all(fixture_parent(
        specific_style_path.as_path(),
        "specific style fixture path has parent directory",
    )?)?;
    fs::write(
        workspace_path.join("webpack.config.js"),
        r#"module.exports = { resolve: { alias: [{ find: "@theme", replacement: "./src/theme" }, { find: "@theme/deep", replacement: "./src/specific" }] } };"#,
    )?;
    fs::write(target_style_path.as_path(), ".article { display: block; }")?;
    fs::write(
        specific_style_path.as_path(),
        ".article { color: hotpink; }",
    )?;

    let workspace_uri = path_to_file_uri(workspace_path.as_path());
    let source_uri = path_to_file_uri(workspace_path.join("src/App.tsx").as_path());
    let target_style_uri = path_to_file_uri(target_style_path.as_path());
    let specific_style_uri = path_to_file_uri(specific_style_path.as_path());
    let unrelated_style_uri =
        path_to_file_uri(workspace_path.join("src/other.module.scss").as_path());

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
                    "text": "import bind from \"classnames/bind\";\nimport styles from \"@theme/deep/some-component.module.scss\";\nconst cx = bind.bind(styles);\nexport const className = cx(\"article\");",
                },
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
                    "uri": target_style_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": ".article { display: block; }",
                },
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
                    "uri": specific_style_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": ".article { color: hotpink; }",
                },
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
                    "uri": unrelated_style_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": ".article { color: red; }",
                },
            },
        }),
    );

    assert_source_binding_target(&state, source_uri.as_str(), target_style_uri.as_str());
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

    assert_definition_response_single_target(&definition_response, target_style_uri.as_str());
    let _ = fs::remove_dir_all(workspace_path.as_path());
    Ok(())
}
