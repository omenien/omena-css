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

#[test]
fn resolves_sass_namespace_symbols_through_forwarded_alias_targets() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-sass-forward-symbols-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let source_path = workspace_path.join("src").join("App.module.scss");
    let index_path = workspace_path
        .join("src")
        .join("shared")
        .join("_index.scss");
    let tokens_path = workspace_path
        .join("src")
        .join("shared")
        .join("_tokens.scss");
    fs::create_dir_all(fixture_parent(
        tokens_path.as_path(),
        "tokens fixture path has parent directory",
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
    fs::write(index_path.as_path(), r#"@forward "./tokens";"#)?;
    let target_text = r#"$gap: 1rem;
@mixin raised { box-shadow: none; }
@function tone($value) { @return $value; }
"#;
    fs::write(tokens_path.as_path(), target_text)?;
    let source_text = r#"@use "$shared/index" as tokens;
.button {
  color: tokens.$gap;
  @include tokens.raised;
  border-color: tokens.tone(tokens.$gap);
}
"#;
    fs::write(source_path.as_path(), source_text)?;

    let workspace_uri = path_to_file_uri(workspace_path.as_path());
    let source_uri = path_to_file_uri(source_path.as_path());
    let index_uri = path_to_file_uri(index_path.as_path());
    let tokens_uri = path_to_file_uri(tokens_path.as_path());
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
        (index_uri.as_str(), r#"@forward "./tokens";"#),
        (tokens_uri.as_str(), target_text),
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

    let gap_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "$gap",
            "source fixture contains namespaced variable",
        )?,
    );
    let gap_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": gap_position,
            },
        }),
    );
    assert_eq!(
        gap_definition
            .as_ref()
            .and_then(|value| value.pointer("/result/0/uri")),
        Some(&json!(tokens_uri)),
    );

    let mixin_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "raised",
            "source fixture contains namespaced mixin",
        )?,
    );
    let mixin_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
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
        Some(&json!(tokens_uri)),
    );

    let function_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "tone",
            "source fixture contains namespaced function",
        )?,
    );
    let function_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": function_position,
            },
        }),
    );
    assert_eq!(
        function_definition
            .as_ref()
            .and_then(|value| value.pointer("/result/0/uri")),
        Some(&json!(tokens_uri)),
    );

    let _ = fs::remove_dir_all(workspace_path.as_path());
    Ok(())
}

#[test]
fn indexes_sass_map_prefix_include_generated_selectors_for_source_prefixes() -> TestResult {
    let source_uri = "file:///workspace-a/src/App.tsx";
    let style_uri = "file:///workspace-a/src/App.module.scss";
    let source_text = r#"import bind from "classnames/bind";
import styles from "./App.module.scss";
const cx = bind.bind(styles);
export const view = <span className={cx(color && `color-${color}`)} />;
"#;
    let style_text = r#"@include setAllStyle(
  ("green": #0f0, "blue": #00f),
  background-color,
  ".primary.fill",
  $prefix: "color"
);
"#;
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
                    "uri": source_uri,
                    "languageId": "typescriptreact",
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
                    "uri": style_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": style_text,
                },
            },
        }),
    );

    let color_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "color-${color}",
            "source fixture contains color template prefix",
        )?,
    );
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
                "position": color_position,
            },
        }),
    );
    let results = definition
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("color prefix definition should return results"))?;
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].get("uri"), Some(&json!(style_uri)));
    assert_eq!(results[1].get("uri"), Some(&json!(style_uri)));
    Ok(())
}
