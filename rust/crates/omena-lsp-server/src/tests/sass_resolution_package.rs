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
                    "omena": {
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

#[test]
fn indexes_foreign_package_forward_chain_for_references_without_opening_dependency() -> TestResult {
    let root = std::env::temp_dir().join(format!(
        "omena_lsp_foreign_package_reference_index_{}_{}",
        std::process::id(),
        current_time_millis()
    ));
    let source = root.join("src/App.module.scss");
    let package_root = root.join("node_modules/@acme/tokens");
    let index_style = package_root.join("_index.scss");
    let primitives_style = package_root.join("_primitives.scss");
    fs::create_dir_all(fixture_parent(source.as_path(), "source parent")?)?;
    fs::create_dir_all(package_root.as_path())?;
    fs::write(
        package_root.join("package.json"),
        r#"{"name":"@acme/tokens","version":"1.2.3","sass":"./_index.scss"}"#,
    )?;
    let source_text = r#"@use "@acme/tokens" as t;
.button { border-radius: t.$token-radius-small; }
"#;
    let index_text = r#"@forward "primitives" as token-*;"#;
    let primitives_text = "$radius-small: 4px;\n";
    fs::write(source.as_path(), source_text)?;
    fs::write(index_style.as_path(), index_text)?;
    fs::write(primitives_style.as_path(), primitives_text)?;

    let workspace_uri = path_to_file_uri(root.as_path());
    let source_uri = path_to_file_uri(source.as_path());
    let index_uri = path_to_file_uri(index_style.as_path());
    let primitives_uri = path_to_file_uri(primitives_style.as_path());
    let reference_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "$token-radius-small",
            "source fixture contains forwarded Sass variable reference",
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

    assert_eq!(
        state
            .document(index_uri.as_str())
            .map(|document| &document.origin),
        Some(&LspDocumentOrigin::Foreign),
        "package entrypoint should be admitted as a read-only foreign document"
    );
    assert_eq!(
        state
            .document(primitives_uri.as_str())
            .map(|document| &document.origin),
        Some(&LspDocumentOrigin::Foreign),
        "forwarded package primitive should be admitted as a read-only foreign document"
    );
    assert!(
        state.document_mut(primitives_uri.as_str()).is_none(),
        "foreign package documents must not expose mutable edit handles"
    );
    let diagnostics = lsp_style_diagnostics(&mut state, source_uri.as_str())?;
    assert!(
        diagnostics.iter().all(|diagnostic| {
            diagnostic.pointer("/code") != Some(&json!("missingSassSymbol"))
                && diagnostic.pointer("/code") != Some(&json!("missingExternalSif"))
        }),
        "forwarded foreign Sass variables should not surface missing-symbol diagnostics: {diagnostics:?}"
    );
    assert_no_foreign_path_leak("foreign Sass diagnostics", &json!(diagnostics))?;

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
                "position": reference_position,
            },
        }),
    );
    assert_definition_response_single_target(&definition, primitives_uri.as_str());

    let references = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/references",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": reference_position,
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
        .ok_or_else(|| std::io::Error::other("foreign references should return locations"))?;
    assert!(
        locations.iter().any(|location| location
            .get("uri")
            .and_then(Value::as_str)
            .is_some_and(|uri| file_uri_equivalent(uri, source_uri.as_str()))),
        "foreign declaration references should include the local consumer: {references:?}"
    );
    assert!(
        locations.iter().any(|location| location
            .get("uri")
            .and_then(Value::as_str)
            .is_some_and(|uri| file_uri_equivalent(uri, primitives_uri.as_str()))),
        "foreign declaration references should include the read-only declaration: {references:?}"
    );
    let warm_definition_json = serde_json::to_string(
        definition
            .as_ref()
            .and_then(|response| response.pointer("/result"))
            .ok_or_else(|| std::io::Error::other("warm definition should include a result"))?,
    )?;
    let warm_references_json = serde_json::to_string(
        references
            .as_ref()
            .and_then(|response| response.pointer("/result"))
            .ok_or_else(|| std::io::Error::other("warm references should include a result"))?,
    )?;
    assert_foreign_occurrence_artifacts_are_workspace_cache_confined(
        &state,
        workspace_uri.as_str(),
        package_root.as_path(),
    )?;

    state.remove_document_uri(index_uri.as_str());
    state.remove_document_uri(primitives_uri.as_str());
    *state.workspace_occurrence_index_memo_lock() = None;
    assert!(
        state.document(index_uri.as_str()).is_none()
            && state.document(primitives_uri.as_str()).is_none(),
        "evicted foreign package documents should force the cold disk-read path"
    );

    let cold_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 30,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": reference_position,
            },
        }),
    );
    assert_eq!(
        warm_definition_json,
        serde_json::to_string(
            cold_definition
                .as_ref()
                .and_then(|response| response.pointer("/result"))
                .ok_or_else(|| std::io::Error::other("cold definition should include a result"))?,
        )?,
        "foreign Sass definition should be byte-identical between warm indexed and cold disk-read paths"
    );

    let cold_references = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 31,
            "method": "textDocument/references",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": reference_position,
                "context": {
                    "includeDeclaration": true,
                },
            },
        }),
    );
    assert_eq!(
        warm_references_json,
        serde_json::to_string(
            cold_references
                .as_ref()
                .and_then(|response| response.pointer("/result"))
                .ok_or_else(|| std::io::Error::other("cold references should include a result"))?,
        )?,
        "foreign Sass references should be byte-identical between warm indexed and cold disk-read paths"
    );

    fs::write(primitives_style.as_path(), "\n$radius-small: 4px;\n")?;
    state.remove_document_uri(index_uri.as_str());
    state.remove_document_uri(primitives_uri.as_str());
    *state.workspace_occurrence_index_memo_lock() = None;
    let shifted_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 32,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": reference_position,
            },
        }),
    );
    assert_definition_response_single_target(&shifted_definition, primitives_uri.as_str());
    assert_eq!(
        shifted_definition
            .as_ref()
            .and_then(|response| response.pointer("/result/0/range/start/line"))
            .and_then(Value::as_u64),
        Some(1),
        "evicted foreign definitions should re-read changed disk content: {shifted_definition:?}"
    );

    let shifted_references = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 33,
            "method": "textDocument/references",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": reference_position,
                "context": {
                    "includeDeclaration": true,
                },
            },
        }),
    );
    let shifted_reference_locations = shifted_references
        .as_ref()
        .and_then(|response| response.pointer("/result"))
        .and_then(Value::as_array)
        .ok_or_else(|| {
            std::io::Error::other("shifted foreign references should return locations")
        })?;
    assert!(
        shifted_reference_locations.iter().any(|location| {
            location
                .get("uri")
                .and_then(Value::as_str)
                .is_some_and(|uri| file_uri_equivalent(uri, primitives_uri.as_str()))
                && location
                    .pointer("/range/start/line")
                    .and_then(Value::as_u64)
                    == Some(1)
        }),
        "foreign references should re-derive declaration locations from changed disk content: {shifted_references:?}"
    );

    let rename = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "textDocument/rename",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": reference_position,
                "newName": "$token-radius-large",
            },
        }),
    );
    let changes = rename
        .as_ref()
        .and_then(|response| response.pointer("/result/changes"))
        .and_then(Value::as_object)
        .ok_or_else(|| std::io::Error::other("rename should return local edits"))?;
    assert!(
        changes
            .keys()
            .any(|uri| file_uri_equivalent(uri.as_str(), source_uri.as_str())),
        "rename should edit the local reference: {rename:?}"
    );
    assert!(
        !changes
            .keys()
            .any(|uri| file_uri_equivalent(uri.as_str(), primitives_uri.as_str())),
        "rename must not edit foreign package declarations: {rename:?}"
    );
    assert_no_foreign_path_leak(
        "foreign Sass rename response",
        rename
            .as_ref()
            .ok_or_else(|| std::io::Error::other("rename should return a response"))?,
    )?;

    let _ = fs::remove_dir_all(root.as_path());
    Ok(())
}

#[test]
fn resolves_sass_definition_through_package_import_and_export_array_fallbacks() -> TestResult {
    let root = std::env::temp_dir().join(format!(
        "omena_lsp_package_array_fallbacks_{}_{}",
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
        r##"{"imports":{"#theme":[{"node":"./src/theme.js"},{"style":"@design/tokens/theme"}]}}"##,
    )?;
    fs::write(
        package_root.join("package.json"),
        r#"{"exports":{"./theme":[{"import":"./dist/theme.js"},{"sass":"./dist/theme.scss"}]}}"#,
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

#[test]
fn loads_workspace_lock_external_sifs_for_lsp_style_diagnostics() -> TestResult {
    let root = std::env::temp_dir().join(format!(
        "omena_lsp_lock_external_sifs_{}_{}",
        std::process::id(),
        current_time_millis()
    ));
    let source = root.join("src/App.module.scss");
    let sif_path = root.join("sif/tokens.sif.json");
    fs::create_dir_all(fixture_parent(source.as_path(), "source parent")?)?;
    fs::create_dir_all(fixture_parent(sif_path.as_path(), "sif parent")?)?;

    let source_text = r#"@use "https://cdn.example/tokens.scss" as tokens;
.button { color: tokens.$brand; }"#;
    fs::write(source.as_path(), source_text)?;

    let sif = fixture_external_sif("https://cdn.example/tokens.scss")?;
    fs::write(
        sif_path.as_path(),
        omena_sif::write_omena_sif_json_v1(&sif)?,
    )?;
    let lock = omena_sif::OmenaLockV1::new(vec![omena_sif::build_omena_lock_sif_entry_v1(
        "sif/tokens.sif.json",
        &sif,
    )?]);
    fs::write(
        root.join("omena.lock"),
        omena_sif::write_omena_lock_json_v1(&lock)?,
    )?;

    let workspace_uri = path_to_file_uri(root.as_path());
    let source_uri = path_to_file_uri(source.as_path());
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
    assert!(
        state.resolution.external_sifs.iter().any(|sif| {
            sif.canonical_url == "https://cdn.example/tokens.scss"
                && sif.sif.canonical_url == "https://cdn.example/tokens.scss"
        }),
        "initialize should load workspace omena.lock SIFs into the LSP diagnostics state"
    );

    open_style_document(&mut state, source_uri.as_str(), source_text);
    let diagnostics = lsp_style_diagnostics(&mut state, source_uri.as_str())?;
    assert!(
        diagnostics.iter().all(|diagnostic| {
            diagnostic.pointer("/code") != Some(&json!("missingSassSymbol"))
                && diagnostic.pointer("/code") != Some(&json!("missingExternalSif"))
        }),
        "lock-backed external SIF should satisfy the foreign Sass reference: {diagnostics:?}"
    );

    let _ = fs::remove_dir_all(root.as_path());
    Ok(())
}

#[test]
fn source_absent_sif_exports_feed_hover_refs_and_completion_without_locations() -> TestResult {
    let root = std::env::temp_dir().join(format!(
        "omena_lsp_source_absent_sif_exports_{}_{}",
        std::process::id(),
        current_time_millis()
    ));
    let source = root.join("src/App.module.scss");
    let sif_path = root.join("sif/tokens.sif.json");
    fs::create_dir_all(fixture_parent(source.as_path(), "source parent")?)?;
    fs::create_dir_all(fixture_parent(sif_path.as_path(), "sif parent")?)?;

    let source_text = r#"@use "https://cdn.example/tokens.scss" as tokens;
.button {
  color: tokens.$brand;
  border-color: tokens.$brand;
  outline-color: tokens.;
}"#;
    fs::write(source.as_path(), source_text)?;
    let sif = fixture_external_sif("https://cdn.example/tokens.scss")?;
    fs::write(
        sif_path.as_path(),
        omena_sif::write_omena_sif_json_v1(&sif)?,
    )?;
    let lock = omena_sif::OmenaLockV1::new(vec![omena_sif::build_omena_lock_sif_entry_v1(
        "sif/tokens.sif.json",
        &sif,
    )?]);
    fs::write(
        root.join("omena.lock"),
        omena_sif::write_omena_lock_json_v1(&lock)?,
    )?;

    let workspace_uri = path_to_file_uri(root.as_path());
    let source_uri = path_to_file_uri(source.as_path());
    let reference_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "$brand",
            "source fixture contains SIF-backed Sass variable reference",
        )? + 1,
    );
    let completion_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "tokens.;",
            "source fixture contains SIF-backed member completion point",
        )? + "tokens.".len(),
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
    open_style_document(&mut state, source_uri.as_str(), source_text);

    let hover = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/hover",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": reference_position,
            },
        }),
    );
    let hover_text = hover
        .as_ref()
        .and_then(|response| response.pointer("/result/contents/value"))
        .and_then(Value::as_str)
        .ok_or_else(|| std::io::Error::other("SIF-backed hover should render markdown"))?;
    assert!(
        hover_text.contains("External Sass interface"),
        "{hover_text}"
    );
    assert!(
        hover_text.contains("https://cdn.example/tokens.scss"),
        "{hover_text}"
    );
    assert!(
        hover_text.contains("Source location is unavailable"),
        "{hover_text}"
    );
    assert!(hover_text.contains("Value: `red`"), "{hover_text}");

    let definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": reference_position,
            },
        }),
    );
    assert!(
        definition
            .as_ref()
            .and_then(|response| response.pointer("/result"))
            .is_some_and(Value::is_null),
        "SIF-only symbols must not fabricate definition locations: {definition:?}"
    );

    let references = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "textDocument/references",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": reference_position,
                "context": {
                    "includeDeclaration": true,
                },
            },
        }),
    );
    let reference_locations = references
        .as_ref()
        .and_then(|response| response.pointer("/result"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("SIF-backed references should return locations"))?;
    assert_eq!(
        reference_locations.len(),
        2,
        "references should join local uses through the SIF external moniker without adding a fake declaration: {references:?}"
    );
    assert!(
        reference_locations.iter().all(|location| location
            .get("uri")
            .and_then(Value::as_str)
            .is_some_and(|uri| file_uri_equivalent(uri, source_uri.as_str()))),
        "SIF-backed references should stay on source locations only: {references:?}"
    );
    let sif_definition_json = serde_json::to_string(
        definition
            .as_ref()
            .and_then(|response| response.pointer("/result"))
            .ok_or_else(|| {
                std::io::Error::other("SIF-backed definition should include a result")
            })?,
    )?;
    let sif_references_json = serde_json::to_string(
        references
            .as_ref()
            .and_then(|response| response.pointer("/result"))
            .ok_or_else(|| {
                std::io::Error::other("SIF-backed references should include a result")
            })?,
    )?;
    *state.workspace_occurrence_index_memo_lock() = None;

    let rebuilt_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 40,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": reference_position,
            },
        }),
    );
    assert_eq!(
        sif_definition_json,
        serde_json::to_string(
            rebuilt_definition
                .as_ref()
                .and_then(|response| response.pointer("/result"))
                .ok_or_else(|| std::io::Error::other(
                    "rebuilt SIF definition should include a result"
                ))?,
        )?,
        "SIF-only definitions should remain byte-identical without fabricating source locations"
    );

    let rebuilt_references = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 41,
            "method": "textDocument/references",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": reference_position,
                "context": {
                    "includeDeclaration": true,
                },
            },
        }),
    );
    assert_eq!(
        sif_references_json,
        serde_json::to_string(
            rebuilt_references
                .as_ref()
                .and_then(|response| response.pointer("/result"))
                .ok_or_else(|| std::io::Error::other(
                    "rebuilt SIF references should include a result"
                ))?,
        )?,
        "SIF-only references should remain byte-identical without adding a fake declaration"
    );

    let completion = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "textDocument/completion",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": completion_position,
            },
        }),
    );
    let completion_items = completion
        .as_ref()
        .and_then(|response| response.pointer("/result/items"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("SIF-backed completion should return items"))?;
    assert!(
        completion_items.iter().any(|item| item
            .get("label")
            .and_then(Value::as_str)
            .is_some_and(|label| label == "tokens.$brand")),
        "completion should expose SIF-backed visible Sass symbols: {completion:?}"
    );

    let _ = fs::remove_dir_all(root.as_path());
    Ok(())
}

#[test]
fn refreshes_workspace_lock_external_sifs_from_watched_files() -> TestResult {
    let root = std::env::temp_dir().join(format!(
        "omena_lsp_lock_watch_external_sifs_{}_{}",
        std::process::id(),
        current_time_millis()
    ));
    let source = root.join("src/App.module.scss");
    let sif_path = root.join("sif/tokens.sif.json");
    let lock_path = root.join("omena.lock");
    fs::create_dir_all(fixture_parent(source.as_path(), "source parent")?)?;
    fs::create_dir_all(fixture_parent(sif_path.as_path(), "sif parent")?)?;

    let source_text = r#"@use "https://cdn.example/tokens.scss" as tokens;
.button { color: tokens.$brand; }"#;
    fs::write(source.as_path(), source_text)?;

    let workspace_uri = path_to_file_uri(root.as_path());
    let source_uri = path_to_file_uri(source.as_path());
    let lock_uri = path_to_file_uri(lock_path.as_path());
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
    open_style_document(&mut state, source_uri.as_str(), source_text);
    assert!(
        state.resolution.external_sifs.is_empty(),
        "workspace starts without lock-backed external SIFs"
    );

    let sif = fixture_external_sif("https://cdn.example/tokens.scss")?;
    fs::write(
        sif_path.as_path(),
        omena_sif::write_omena_sif_json_v1(&sif)?,
    )?;
    let lock = omena_sif::OmenaLockV1::new(vec![omena_sif::build_omena_lock_sif_entry_v1(
        "sif/tokens.sif.json",
        &sif,
    )?]);
    fs::write(
        lock_path.as_path(),
        omena_sif::write_omena_lock_json_v1(&lock)?,
    )?;
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeWatchedFiles",
            "params": {
                "changes": [
                    {
                        "uri": lock_uri,
                        "type": 2,
                    },
                ],
            },
        }),
    );

    assert!(
        state
            .resolution
            .external_sifs
            .iter()
            .any(|sif| sif.canonical_url == "https://cdn.example/tokens.scss"),
        "watched omena.lock changes should refresh external SIF state"
    );
    let diagnostics = lsp_style_diagnostics(&mut state, source_uri.as_str())?;
    assert!(
        diagnostics.iter().all(|diagnostic| {
            diagnostic.pointer("/code") != Some(&json!("missingSassSymbol"))
                && diagnostic.pointer("/code") != Some(&json!("missingExternalSif"))
        }),
        "watch-refreshed external SIF should satisfy the Sass reference: {diagnostics:?}"
    );

    let _ = fs::remove_dir_all(root.as_path());
    Ok(())
}

#[test]
fn bridges_file_external_sass_edges_for_lsp_style_diagnostics() -> TestResult {
    let root = std::env::temp_dir().join(format!(
        "omena_lsp_bridge_external_sifs_{}_{}",
        std::process::id(),
        current_time_millis()
    ));
    let source = root.join("src/App.module.scss");
    let external = root.join("vendor/_tokens.scss");
    fs::create_dir_all(fixture_parent(source.as_path(), "source parent")?)?;
    fs::create_dir_all(fixture_parent(external.as_path(), "external parent")?)?;
    fs::write(external.as_path(), "$brand: red !default;\n")?;

    let external_uri = path_to_file_uri(external.as_path());
    let source_text = format!(
        "@use \"{}\" as tokens;\n.button {{ color: tokens.$brand; }}",
        external_uri
    );
    fs::write(source.as_path(), source_text.as_str())?;

    let workspace_uri = path_to_file_uri(root.as_path());
    let source_uri = path_to_file_uri(source.as_path());
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
    open_style_document(&mut state, source_uri.as_str(), source_text.as_str());

    assert!(
        state
            .resolution
            .external_sifs
            .iter()
            .any(|sif| sif.canonical_url == external_uri),
        "opening a style document should bridge readable file:// external Sass edges"
    );
    let diagnostics = lsp_style_diagnostics(&mut state, source_uri.as_str())?;
    assert!(
        diagnostics.iter().all(|diagnostic| {
            diagnostic.pointer("/code") != Some(&json!("missingSassSymbol"))
                && diagnostic.pointer("/code") != Some(&json!("missingExternalSif"))
        }),
        "bridge-generated external SIF should satisfy the file:// Sass reference: {diagnostics:?}"
    );

    let _ = fs::remove_dir_all(root.as_path());
    Ok(())
}

#[test]
fn bridges_bare_package_forward_chain_for_lsp_style_diagnostics() -> TestResult {
    let root = std::env::temp_dir().join(format!(
        "omena_lsp_bare_package_forward_sifs_{}_{}",
        std::process::id(),
        current_time_millis()
    ));
    let source = root.join("src/App.module.scss");
    let app_package = root.join("node_modules/@app/theme");
    let design_package = root.join("node_modules/@design/tokens");
    fs::create_dir_all(fixture_parent(source.as_path(), "source parent")?)?;
    fs::create_dir_all(app_package.as_path())?;
    fs::create_dir_all(design_package.as_path())?;
    fs::write(
        app_package.join("package.json"),
        r#"{"exports":{"./index":{"sass":"./index.scss"}}}"#,
    )?;
    fs::write(
        design_package.join("package.json"),
        r#"{"exports":{"./colors":{"sass":"./colors.scss"}}}"#,
    )?;
    fs::write(
        app_package.join("index.scss"),
        "@forward \"@design/tokens/colors\";\n@forward \"./radius\";\n",
    )?;
    fs::write(app_package.join("_radius.scss"), "$ds_radius-card: 12px;\n")?;
    fs::write(design_package.join("colors.scss"), "$ds_gray-700: #333;\n")?;

    let source_text = "@use \"@app/theme/index\" as ds;\n.button { color: ds.$ds_gray-700; border-radius: ds.$ds_radius-card; }\n";
    fs::write(source.as_path(), source_text)?;

    let workspace_uri = path_to_file_uri(root.as_path());
    let source_uri = path_to_file_uri(source.as_path());
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
    open_style_document(&mut state, source_uri.as_str(), source_text);

    assert!(
        state
            .resolution
            .external_sifs
            .iter()
            .any(|sif| sif.canonical_url == "@design/tokens/colors"),
        "bare transitive forward should be represented as a verbatim external SIF alias: {:?}",
        state.resolution.external_sifs
    );
    let diagnostics = lsp_style_diagnostics(&mut state, source_uri.as_str())?;
    assert!(
        diagnostics.iter().all(|diagnostic| {
            diagnostic.pointer("/code") != Some(&json!("missingSassSymbol"))
                && diagnostic.pointer("/code") != Some(&json!("missingExternalSif"))
        }),
        "bare package forward chain should satisfy Sass references: {diagnostics:?}"
    );

    let _ = fs::remove_dir_all(root.as_path());
    Ok(())
}

#[test]
fn style_document_bridge_changes_refresh_external_sifs_without_corpus_rebuild() -> TestResult {
    let root = std::env::temp_dir().join(format!(
        "omena_lsp_bridge_external_sifs_delta_{}_{}",
        std::process::id(),
        current_time_millis()
    ));
    let source = root.join("src/App.module.scss");
    let peer = root.join("src/Peer.module.scss");
    let external_a = root.join("vendor/_a.scss");
    let external_b = root.join("vendor/_b.scss");
    fs::create_dir_all(fixture_parent(source.as_path(), "source parent")?)?;
    fs::create_dir_all(fixture_parent(peer.as_path(), "peer parent")?)?;
    fs::create_dir_all(fixture_parent(external_a.as_path(), "external parent")?)?;
    fs::write(
        root.join("omena.lock"),
        r#"{"lockfileVersion":"1","entries":[]}"#,
    )?;
    fs::write(source.as_path(), ".button { color: red; }\n")?;
    fs::write(external_a.as_path(), "$brand-a: red !default;\n")?;
    fs::write(external_b.as_path(), "$brand-b: blue !default;\n")?;

    let workspace_uri = path_to_file_uri(root.as_path());
    let source_uri = path_to_file_uri(source.as_path());
    let peer_uri = path_to_file_uri(peer.as_path());
    let external_a_uri = path_to_file_uri(external_a.as_path());
    let external_b_uri = path_to_file_uri(external_b.as_path());
    let peer_text = format!(
        "@use \"{}\" as tokens;\n.peer {{ color: tokens.$brand-a; }}\n",
        external_a_uri
    );
    let source_initial_text = ".button { color: red; }\n";
    let source_changed_text = format!(
        "@use \"{}\" as tokens;\n.button {{ color: tokens.$brand-b; }}\n",
        external_b_uri
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
    open_style_document(&mut state, peer_uri.as_str(), peer_text.as_str());
    open_style_document(&mut state, source_uri.as_str(), source_initial_text);

    let lock_reads_before_change = state.external_sif_lock_read_count;
    let bridge_generations_before_change = state.external_sif_bridge_generation_count;
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                    "version": 2,
                },
                "contentChanges": [
                    {
                        "text": source_changed_text,
                    },
                ],
            },
        }),
    );

    assert_eq!(
        state.external_sif_lock_read_count - lock_reads_before_change,
        0,
        "bridge source didChange must not reread workspace lockfiles"
    );
    assert_eq!(
        state.external_sif_bridge_generation_count - bridge_generations_before_change,
        1,
        "bridge source didChange should generate only the newly-added bridge SIF"
    );
    assert!(
        state
            .resolution
            .external_sifs
            .iter()
            .any(|sif| sif.canonical_url == external_a_uri),
        "existing bridge SIF from another document must remain active"
    );
    assert!(
        state
            .resolution
            .external_sifs
            .iter()
            .any(|sif| sif.canonical_url == external_b_uri),
        "new bridge SIF from the changed document must be added"
    );

    let _ = fs::remove_dir_all(root.as_path());
    Ok(())
}

fn fixture_external_sif(canonical_url: &str) -> Result<omena_sif::OmenaSifV1, serde_json::Error> {
    omena_sif::OmenaSifV1::from_static_exports(
        canonical_url,
        omena_sif::OmenaSifGeneratorV1 {
            name: "fixture-sifgen".to_string(),
            version: "0.1.0".to_string(),
            toolchain_id: "fixture-sifgen@0.1.0".to_string(),
        },
        omena_sif::OmenaSifSourceV1 {
            syntax: omena_sif::OmenaSifSourceSyntaxV1::Scss,
        },
        omena_sif::OmenaSifExportsV1 {
            variables: vec![omena_sif::OmenaSifVariableExportV1 {
                name: "$brand".to_string(),
                defaulted: true,
                value_repr: Some("red".to_string()),
            }],
            mixins: Vec::new(),
            functions: Vec::new(),
            placeholders: Vec::new(),
            forwards: Vec::new(),
        },
        Vec::new(),
        b"$brand: red !default;",
    )
}

fn open_style_document(state: &mut LspShellState, uri: &str, text: &str) {
    handle_lsp_message(
        state,
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

fn lsp_style_diagnostics(
    state: &mut LspShellState,
    uri: &str,
) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let response = handle_lsp_message(
        state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": STYLE_DIAGNOSTICS_REQUEST,
            "params": {
                "textDocument": {
                    "uri": uri,
                },
            },
        }),
    );
    response
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .cloned()
        .ok_or_else(|| std::io::Error::other("style diagnostics response contains an array").into())
}

fn assert_no_foreign_path_leak(label: &str, value: &Value) -> TestResult {
    let serialized = serde_json::to_string(value)?;
    assert!(
        !serialized.contains("node_modules"),
        "{label} must not expose node_modules-origin paths: {serialized}"
    );
    Ok(())
}

fn assert_foreign_occurrence_artifacts_are_workspace_cache_confined(
    state: &LspShellState,
    workspace_uri: &str,
    package_root: &Path,
) -> TestResult {
    let workspace_root = file_uri_to_path(workspace_uri)
        .ok_or_else(|| std::io::Error::other("workspace URI should map to a file path"))?;
    let workspace_cache_root = workspace_root.join(".cache").join("omena");
    let sidecar_path =
        crate::style_symbol_occurrence_cache::style_symbol_occurrence_sidecar_file_path_for_test(
            state,
            Some(workspace_uri),
        )
        .ok_or_else(|| {
            std::io::Error::other("style symbol occurrence sidecar path should resolve")
        })?;
    assert!(
        sidecar_path.starts_with(workspace_cache_root.as_path()),
        "foreign occurrence sidecar must stay under the workspace cache root: {sidecar_path:?}"
    );
    assert!(
        sidecar_path.exists(),
        "foreign occurrence sidecar should be persisted: {sidecar_path:?}"
    );
    assert!(
        workspace_cache_root
            .join("workspace-occurrence-shards-v1")
            .exists(),
        "foreign occurrence shards should be persisted below the workspace cache root"
    );
    assert!(
        !package_root.join(".cache").join("omena").exists(),
        "foreign package directories must not receive omena cache artifacts"
    );
    Ok(())
}
