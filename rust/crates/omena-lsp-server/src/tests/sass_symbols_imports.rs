use super::*;

#[test]
fn resolves_sass_internal_symbols_through_wildcard_import_targets() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-sass-symbols-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let source_style_path = workspace_path.join("src/Card.module.scss");
    let target_style_path = workspace_path.join("src/shared/_utils.scss");
    fs::create_dir_all(fixture_parent(
        target_style_path.as_path(),
        "target style fixture path has parent directory",
    )?)?;
    fs::create_dir_all(fixture_parent(
        source_style_path.as_path(),
        "source style fixture path has parent directory",
    )?)?;
    fs::write(
        workspace_path.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "$shared/*": ["src/shared/*"]
    }
  }
}"#,
    )?;
    let source_text = "@import \"$shared/utils\";\n.title {\n  @include defign_typography20;\n  border-top: 1px solid $defign_gray200;\n}\n";
    let target_text = "$defign_gray200: #eee;\n@mixin defign_typography20 { font-size: 20px; }\n";
    fs::write(source_style_path.as_path(), source_text)?;
    fs::write(target_style_path.as_path(), target_text)?;

    let workspace_uri = path_to_file_uri(workspace_path.as_path());
    let source_uri = path_to_file_uri(source_style_path.as_path());
    let target_uri = path_to_file_uri(target_style_path.as_path());
    let mixin_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "defign_typography20",
            "source fixture contains mixin include",
        )?,
    );
    let variable_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "$defign_gray200",
            "source fixture contains variable reference",
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
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": source_text,
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
                    "uri": target_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": target_text,
                },
            },
        }),
    );

    let mixin_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": mixin_position,
            },
        }),
    );
    assert_eq!(
        mixin_definition
            .as_ref()
            .and_then(|value| value.pointer("/result/0/uri")),
        Some(&json!(target_uri)),
    );
    assert_eq!(
        mixin_definition
            .as_ref()
            .and_then(|value| value.pointer("/result/0/range")),
        Some(&json!({
            "start": {
                "line": 1,
                "character": 7,
            },
            "end": {
                "line": 1,
                "character": 26,
            },
        })),
    );

    let variable_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": variable_position,
            },
        }),
    );
    assert_eq!(
        variable_definition
            .as_ref()
            .and_then(|value| value.pointer("/result/0/uri")),
        Some(&json!(target_uri)),
    );
    assert_eq!(
        variable_definition
            .as_ref()
            .and_then(|value| value.pointer("/result/0/range")),
        Some(&json!({
            "start": {
                "line": 0,
                "character": 1,
            },
            "end": {
                "line": 0,
                "character": 15,
            },
        })),
    );

    let variable_hover = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "textDocument/hover",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": variable_position,
            },
        }),
    );
    assert_eq!(
        variable_hover
            .as_ref()
            .and_then(|value| value.pointer("/result/contents/kind")),
        Some(&json!("markdown")),
    );
    assert_eq!(
        variable_hover
            .as_ref()
            .and_then(|value| value.pointer("/result/range")),
        Some(&json!({
            "start": {
                "line": 3,
                "character": 24,
            },
            "end": {
                "line": 3,
                "character": 39,
            },
        })),
    );
    let variable_hover_text = variable_hover
        .as_ref()
        .and_then(|value| value.pointer("/result/contents/value"))
        .and_then(Value::as_str)
        .ok_or_else(|| std::io::Error::other("variable hover should render markdown"))?;
    assert!(variable_hover_text.contains("Value: `#eee`"));
    assert!(variable_hover_text.contains("$defign_gray200: #eee;"));

    let mixin_hover = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "textDocument/hover",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": mixin_position,
            },
        }),
    );
    let mixin_hover_text = mixin_hover
        .as_ref()
        .and_then(|value| value.pointer("/result/contents/value"))
        .and_then(Value::as_str)
        .ok_or_else(|| std::io::Error::other("mixin hover should render markdown"))?;
    assert!(mixin_hover_text.contains("@mixin defign_typography20"));
    assert!(mixin_hover_text.contains("font-size: 20px;"));

    let _ = fs::remove_dir_all(workspace_path.as_path());
    Ok(())
}
