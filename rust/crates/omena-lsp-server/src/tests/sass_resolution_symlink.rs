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

#[cfg(unix)]
#[test]
fn refreshes_sass_resolution_after_package_symlink_retarget_watch() -> TestResult {
    let root = std::env::temp_dir().join(format!(
        "omena_lsp_symlinked_package_retarget_{}_{}",
        std::process::id(),
        current_time_millis()
    ));
    let source = root.join("src/App.module.scss");
    let real_v1 = root.join(".pnpm/@design+tokens@1.0.0/node_modules/@design/tokens");
    let real_v2 = root.join(".pnpm/@design+tokens@2.0.0/node_modules/@design/tokens");
    let linked_scope = root.join("node_modules/@design");
    let linked_package = linked_scope.join("tokens");
    let real_v1_style = real_v1.join("src/index.scss");
    let real_v2_style = real_v2.join("src/index.scss");
    fs::create_dir_all(fixture_parent(source.as_path(), "source parent")?)?;
    fs::create_dir_all(fixture_parent(real_v1_style.as_path(), "v1 style parent")?)?;
    fs::create_dir_all(fixture_parent(real_v2_style.as_path(), "v2 style parent")?)?;
    fs::create_dir_all(linked_scope.as_path())?;
    fs::write(
        real_v1.join("package.json"),
        r#"{"name":"@design/tokens","version":"1.0.0","sass":"src/index.scss"}"#,
    )?;
    fs::write(
        real_v2.join("package.json"),
        r#"{"name":"@design/tokens","version":"2.0.0","sass":"src/index.scss"}"#,
    )?;
    fs::write(real_v1_style.as_path(), "$brand: red;\n")?;
    fs::write(real_v2_style.as_path(), "$brand: green;\n")?;
    std::os::unix::fs::symlink(real_v1.as_path(), linked_package.as_path())?;
    let source_text = r#"@use "@design/tokens" as tokens;
.button { color: tokens.$brand; }
"#;
    fs::write(source.as_path(), source_text)?;

    let workspace_uri = raw_test_file_uri(root.as_path());
    let source_uri = raw_test_file_uri(source.as_path());
    let linked_package_uri = raw_test_file_uri(linked_package.as_path());
    let real_v1_style_uri = raw_test_file_uri(real_v1_style.as_path());
    let real_v2_style_uri = raw_test_file_uri(real_v2_style.as_path());
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
    assert_single_definition_equivalent(&initial_definition, real_v1_style_uri.as_str())?;

    fs::remove_file(linked_package.as_path())?;
    std::os::unix::fs::symlink(real_v2.as_path(), linked_package.as_path())?;
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeWatchedFiles",
            "params": {
                "changes": [
                    {
                        "uri": linked_package_uri,
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
    assert_single_definition_equivalent(&refreshed_definition, real_v2_style_uri.as_str())?;

    let _ = fs::remove_dir_all(root.as_path());
    Ok(())
}

fn assert_single_definition_equivalent(response: &Option<Value>, expected_uri: &str) -> TestResult {
    let targets = response
        .as_ref()
        .and_then(|value| value.get("result"))
        .and_then(Value::as_array)
        .ok_or_else(|| {
            std::io::Error::other(format!("expected definition array, got {response:?}"))
        })?;
    assert_eq!(
        targets.len(),
        1,
        "expected one definition target: {targets:?}"
    );
    assert!(
        targets[0]
            .get("uri")
            .and_then(Value::as_str)
            .is_some_and(|uri| file_uri_equivalent(uri, expected_uri)),
        "definition target should resolve to {expected_uri}: {targets:?}"
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn joins_hoisted_and_nested_package_layout_sass_references() -> TestResult {
    let root = std::env::temp_dir().join(format!(
        "omena_lsp_symlinked_package_reference_join_{}_{}",
        std::process::id(),
        current_time_millis()
    ));
    let hoisted_source = root.join("src/Hoisted.module.scss");
    let nested_source = root.join("packages/app/src/Nested.module.scss");
    let real_package = root.join(".pnpm/@design+tokens@1.0.0/node_modules/@design/tokens");
    let hoisted_scope = root.join("node_modules/@design");
    let nested_scope = root.join("packages/app/node_modules/@design");
    let real_style = real_package.join("src/index.scss");
    fs::create_dir_all(fixture_parent(
        hoisted_source.as_path(),
        "hoisted source parent",
    )?)?;
    fs::create_dir_all(fixture_parent(
        nested_source.as_path(),
        "nested source parent",
    )?)?;
    fs::create_dir_all(fixture_parent(real_style.as_path(), "style parent")?)?;
    fs::create_dir_all(hoisted_scope.as_path())?;
    fs::create_dir_all(nested_scope.as_path())?;
    fs::write(
        real_package.join("package.json"),
        r#"{"name":"@design/tokens","version":"1.0.0","sass":"src/index.scss"}"#,
    )?;
    let hoisted_text = r#"@use "@design/tokens" as tokens;
.hoisted { color: tokens.$brand; }
"#;
    let nested_text = r#"@use "@design/tokens" as tokens;
.nested { color: tokens.$brand; }
"#;
    let target_text = "$brand: #fff;\n";
    fs::write(hoisted_source.as_path(), hoisted_text)?;
    fs::write(nested_source.as_path(), nested_text)?;
    fs::write(real_style.as_path(), target_text)?;
    std::os::unix::fs::symlink(
        real_package.as_path(),
        hoisted_scope.join("tokens").as_path(),
    )?;
    std::os::unix::fs::symlink(
        real_package.as_path(),
        nested_scope.join("tokens").as_path(),
    )?;

    let workspace_uri = raw_test_file_uri(root.as_path());
    let hoisted_source_uri = raw_test_file_uri(hoisted_source.as_path());
    let nested_source_uri = raw_test_file_uri(nested_source.as_path());
    let real_style_uri = raw_test_file_uri(real_style.as_path());
    let brand_position = parser_position_for_byte_offset(
        hoisted_text,
        fixture_find(
            hoisted_text,
            "$brand",
            "hoisted source fixture contains Sass variable reference",
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
        (hoisted_source_uri.as_str(), hoisted_text),
        (nested_source_uri.as_str(), nested_text),
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

    let references = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/references",
            "params": {
                "textDocument": {
                    "uri": hoisted_source_uri,
                },
                "position": brand_position,
                "context": {
                    "includeDeclaration": true,
                },
            },
        }),
    );
    let locations = references
        .as_ref()
        .and_then(|response| response.pointer("/result"))
        .and_then(Value::as_array)
        .ok_or_else(|| {
            std::io::Error::other(format!("expected references array, got {references:?}"))
        })?;
    for expected_uri in [&hoisted_source_uri, &nested_source_uri, &real_style_uri] {
        assert!(
            locations.iter().any(|location| location
                .get("uri")
                .and_then(Value::as_str)
                .is_some_and(|uri| file_uri_equivalent(uri, expected_uri.as_str()))),
            "references should join hoisted and nested package layouts through the canonical package identity for {expected_uri}: {references:?}"
        );
    }

    let _ = fs::remove_dir_all(root.as_path());
    Ok(())
}
