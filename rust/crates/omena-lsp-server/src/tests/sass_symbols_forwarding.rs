use super::*;

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
