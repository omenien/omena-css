use super::*;

#[cfg(unix)]
#[test]
fn resolves_sass_definition_through_symlinked_package_canonical_identity() -> TestResult {
    let root = std::env::temp_dir().join(format!(
        "omena_lsp_symlinked_package_identity_{}_{}",
        std::process::id(),
        current_time_millis()
    ));
    let source = root.join("src/App.module.scss");
    let real_package = root.join(".pnpm/@design+tokens@1.0.0/node_modules/@design/tokens");
    let linked_scope = root.join("node_modules/@design");
    let linked_package = linked_scope.join("tokens");
    let real_style = real_package.join("src/index.scss");
    let linked_style = linked_package.join("src/index.scss");
    fs::create_dir_all(fixture_parent(source.as_path(), "source parent")?)?;
    fs::create_dir_all(fixture_parent(real_style.as_path(), "style parent")?)?;
    fs::create_dir_all(linked_scope.as_path())?;
    fs::write(
        real_package.join("package.json"),
        r#"{"sass":"src/index.scss"}"#,
    )?;
    let source_text = r#"@use "@design/tokens" as tokens;
.button { color: tokens.$brand; }
"#;
    let target_text = "$brand: #fff;\n";
    fs::write(source.as_path(), source_text)?;
    fs::write(real_style.as_path(), target_text)?;
    std::os::unix::fs::symlink(real_package.as_path(), linked_package.as_path())?;

    let workspace_uri = raw_test_file_uri(root.as_path());
    let source_uri = raw_test_file_uri(source.as_path());
    let linked_style_uri = raw_test_file_uri(linked_style.as_path());
    let real_style_uri = raw_test_file_uri(real_style.as_path());
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
        (linked_style_uri.as_str(), target_text),
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
    let targets = definition
        .as_ref()
        .and_then(|value| value.get("result"))
        .and_then(Value::as_array)
        .ok_or_else(|| {
            std::io::Error::other(format!("expected definition array, got {definition:?}"))
        })?;

    assert!(targets.iter().any(|target| {
        target
            .get("uri")
            .and_then(Value::as_str)
            .is_some_and(|uri| file_uri_equivalent(uri, real_style_uri.as_str()))
    }));
    let _ = fs::remove_dir_all(root.as_path());
    Ok(())
}
