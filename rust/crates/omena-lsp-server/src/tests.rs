use super::*;

#[path = "tests/code_actions.rs"]
mod code_actions;
#[path = "tests/diagnostics.rs"]
mod diagnostics;
#[path = "tests/lifecycle.rs"]
mod lifecycle;
#[path = "tests/sass_resolution.rs"]
mod sass_resolution;
#[path = "tests/sass_symbols.rs"]
mod sass_symbols;
#[path = "tests/source_resolution.rs"]
mod source_resolution;
#[path = "tests/source_semantics.rs"]
mod source_semantics;
#[path = "tests/style_context.rs"]
mod style_context;
#[path = "tests/style_indexing.rs"]
mod style_indexing;
#[path = "tests/workspace.rs"]
mod workspace;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn fixture_parent<'a>(
    path: &'a Path,
    context: &'static str,
) -> Result<&'a Path, Box<dyn std::error::Error>> {
    path.parent()
        .ok_or_else(|| std::io::Error::other(context).into())
}

fn fixture_find(
    source: &str,
    needle: &str,
    context: &'static str,
) -> Result<usize, Box<dyn std::error::Error>> {
    source
        .find(needle)
        .ok_or_else(|| std::io::Error::other(context).into())
}

fn assert_source_binding_target(state: &LspShellState, source_uri: &str, expected_style_uri: &str) {
    let imported_style_bindings = state
        .document(source_uri)
        .map(|document| document.source_syntax_index.imported_style_bindings.clone());
    assert_eq!(
        imported_style_bindings.as_deref(),
        Some(
            [ImportedStyleBinding {
                binding: "styles".to_string(),
                style_uri: expected_style_uri.to_string(),
            }]
            .as_slice()
        ),
    );
}

fn assert_definition_response_single_target(response: &Option<Value>, expected_uri: &str) {
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/uri")),
        Some(&json!(expected_uri)),
    );
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/1/uri")),
        None,
    );
}

#[test]
fn resolves_style_hover_candidates_from_opened_style_documents() {
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
                    "text": "import styles from \"./App.module.scss\";\nconst view = <div className={styles.root} />;",
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
                    "uri": "file:///workspace-a/src/App.module.scss",
                    "languageId": "scss",
                    "version": 1,
                    "text": ".root { color: var(--brand); }\n.theme { --brand: red; }",
                },
            },
        }),
    );

    let response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": STYLE_HOVER_CANDIDATES_REQUEST,
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "position": {
                    "line": 0,
                    "character": 2,
                },
            },
        }),
    );

    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/product")),
        Some(&json!("omena-lsp-server.style-hover-candidates")),
    );
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/candidateCount")),
        Some(&json!(1)),
    );
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/candidates/0/name")),
        Some(&json!("root")),
    );
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/candidates/0/range")),
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
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/workspaceFolderUri")),
        Some(&json!("file:///workspace-a")),
    );

    let custom_property_ref_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": STYLE_HOVER_CANDIDATES_REQUEST,
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "position": {
                    "line": 0,
                    "character": 21,
                },
            },
        }),
    );
    assert_eq!(
        custom_property_ref_response
            .as_ref()
            .and_then(|value| value.pointer("/result/candidates/0/kind")),
        Some(&json!("customPropertyReference")),
    );
    assert_eq!(
        custom_property_ref_response
            .as_ref()
            .and_then(|value| value.pointer("/result/candidates/0/range")),
        Some(&json!({
            "start": {
                "line": 0,
                "character": 19,
            },
            "end": {
                "line": 0,
                "character": 26,
            },
        })),
    );

    let custom_property_decl_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": STYLE_HOVER_CANDIDATES_REQUEST,
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "position": {
                    "line": 1,
                    "character": 11,
                },
            },
        }),
    );
    assert_eq!(
        custom_property_decl_response
            .as_ref()
            .and_then(|value| value.pointer("/result/candidates/0/kind")),
        Some(&json!("customPropertyDeclaration")),
    );
    assert_eq!(
        custom_property_decl_response
            .as_ref()
            .and_then(|value| value.pointer("/result/candidates/0/range")),
        Some(&json!({
            "start": {
                "line": 1,
                "character": 9,
            },
            "end": {
                "line": 1,
                "character": 16,
            },
        })),
    );

    let hover_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "textDocument/hover",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "position": {
                    "line": 0,
                    "character": 2,
                },
            },
        }),
    );
    assert_eq!(
        hover_response
            .as_ref()
            .and_then(|value| value.pointer("/result/contents/kind")),
        Some(&json!("markdown")),
    );
    assert_eq!(
        hover_response
            .as_ref()
            .and_then(|value| value.pointer("/result/range")),
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

    let definition_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "position": {
                    "line": 0,
                    "character": 21,
                },
            },
        }),
    );
    assert_eq!(
        definition_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/uri")),
        Some(&json!("file:///workspace-a/src/App.module.scss")),
    );
    assert_eq!(
        definition_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/range")),
        Some(&json!({
            "start": {
                "line": 1,
                "character": 9,
            },
            "end": {
                "line": 1,
                "character": 16,
            },
        })),
    );

    let references_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "textDocument/references",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "position": {
                    "line": 0,
                    "character": 21,
                },
                "context": {
                    "includeDeclaration": true,
                },
            },
        }),
    );
    assert_eq!(
        references_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/range")),
        Some(&json!({
            "start": {
                "line": 0,
                "character": 19,
            },
            "end": {
                "line": 0,
                "character": 26,
            },
        })),
    );
    assert_eq!(
        references_response
            .as_ref()
            .and_then(|value| value.pointer("/result/1/range")),
        Some(&json!({
            "start": {
                "line": 1,
                "character": 9,
            },
            "end": {
                "line": 1,
                "character": 16,
            },
        })),
    );

    let completion_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 8,
            "method": "textDocument/completion",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "position": {
                    "line": 0,
                    "character": 20,
                },
            },
        }),
    );
    assert_eq!(
        completion_response
            .as_ref()
            .and_then(|value| value.pointer("/result/isIncomplete")),
        Some(&json!(false)),
    );
    assert_eq!(
        completion_response
            .as_ref()
            .and_then(|value| value.pointer("/result/items/0/label")),
        Some(&json!("--brand")),
    );
    assert_eq!(
        completion_response
            .as_ref()
            .and_then(|value| value.pointer("/result/items/0/data/rankingSource")),
        Some(&json!("sameFileSourceOrderCascade")),
    );
    assert_eq!(
        completion_response
            .as_ref()
            .and_then(|value| value.pointer("/result/items"))
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(1),
    );

    let prepare_rename_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 9,
            "method": "textDocument/prepareRename",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "position": {
                    "line": 0,
                    "character": 2,
                },
            },
        }),
    );
    assert_eq!(
        prepare_rename_response
            .as_ref()
            .and_then(|value| value.pointer("/result/placeholder")),
        Some(&json!("root")),
    );

    let rename_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 10,
            "method": "textDocument/rename",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "position": {
                    "line": 0,
                    "character": 21,
                },
                "newName": "--accent",
            },
        }),
    );
    assert_eq!(
        rename_response.as_ref().and_then(|value| value
            .pointer("/result/changes/file:~1~1~1workspace-a~1src~1App.module.scss/0/newText")),
        Some(&json!("--accent")),
    );
    assert_eq!(
        rename_response.as_ref().and_then(|value| value
            .pointer("/result/changes/file:~1~1~1workspace-a~1src~1App.module.scss/1/newText")),
        Some(&json!("--accent")),
    );

    let code_lens_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 11,
            "method": "textDocument/codeLens",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
            },
        }),
    );
    assert_eq!(
        code_lens_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/command/title")),
        Some(&json!("1 reference")),
    );
    assert_eq!(
        code_lens_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/command/command")),
        Some(&json!("editor.action.showReferences")),
    );
    assert_eq!(
        code_lens_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/command/arguments/2/0/range")),
        Some(&json!({
            "start": {
                "line": 1,
                "character": 36,
            },
            "end": {
                "line": 1,
                "character": 40,
            },
        })),
    );
}

#[test]
fn sass_symbol_hover_renders_unopened_target_definition_from_disk() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-disk-sass-hover-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let src_dir = workspace_path.join("src");
    let source_style_path = src_dir.join("App.module.scss");
    let token_style_path = src_dir.join("_tokens.scss");
    fs::create_dir_all(src_dir.as_path())?;
    fs::write(token_style_path.as_path(), "$brand: #fff;\n")?;

    let workspace_uri = path_to_file_uri(workspace_path.as_path());
    let source_style_uri = path_to_file_uri(source_style_path.as_path());
    let source_style_text = "@use \"./tokens\" as *;\n.foo { color: $brand; }\n";
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
                    "uri": source_style_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": source_style_text,
                },
            },
        }),
    );

    let hover_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/hover",
            "params": {
                "textDocument": {
                    "uri": path_to_file_uri(source_style_path.as_path()),
                },
                "position": parser_position_for_byte_offset(
                    source_style_text,
                    fixture_find(
                        source_style_text,
                        "$brand",
                        "style fixture contains brand variable",
                    )? + 1,
                ),
            },
        }),
    );
    let hover_text = hover_response
        .as_ref()
        .and_then(|value| value.pointer("/result/contents/value"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    assert!(hover_text.contains("$brand: #fff"));

    let _ = fs::remove_dir_all(workspace_path.as_path());
    Ok(())
}

#[test]
fn workspace_folder_compatibility_normalizes_percent_encoded_route_groups() {
    assert!(workspace_folder_uri_equivalent(
        "file:///workspace/app/(marketing)",
        "file:///workspace/app/%28marketing%29",
    ));
}

#[test]
fn path_to_file_uri_percent_encodes_route_group_paths() {
    let uri = path_to_file_uri(Path::new("/workspace/app/(marketing)/Card.module.scss"));

    assert_eq!(
        uri,
        "file:///workspace/app/%28marketing%29/Card.module.scss"
    );
    assert!(file_uri_equivalent(
        uri.as_str(),
        "file:///workspace/app/(marketing)/Card.module.scss",
    ));
}

#[cfg(unix)]
#[test]
fn document_map_uses_canonical_identity_for_symlinked_document_paths() -> TestResult {
    let root = std::env::temp_dir().join(format!(
        "omena_lsp_symlinked_document_identity_{}_{}",
        std::process::id(),
        current_time_millis()
    ));
    let real_src = root.join("real/src");
    let link_src = root.join("linked-src");
    let real_style = real_src.join("Button.module.scss");
    let linked_style = link_src.join("Button.module.scss");
    fs::create_dir_all(real_src.as_path())?;
    fs::write(real_style.as_path(), ".button { color: red; }")?;
    std::os::unix::fs::symlink(real_src.as_path(), link_src.as_path())?;

    let real_uri = raw_test_file_uri(real_style.as_path());
    let linked_uri = raw_test_file_uri(linked_style.as_path());
    assert!(file_uri_equivalent(real_uri.as_str(), linked_uri.as_str()));

    let mut state = LspShellState::default();
    for (uri, text) in [
        (linked_uri.as_str(), ".button { color: red; }"),
        (real_uri.as_str(), ".button { color: blue; }"),
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

    assert_eq!(state.document_count(), 1);
    assert_eq!(state.open_document_uris.len(), 1);
    assert!(state.document(real_uri.as_str()).is_some());
    assert!(state.document(linked_uri.as_str()).is_some());
    assert_eq!(state.snapshot().documents.len(), 1);
    let _ = fs::remove_dir_all(root.as_path());
    Ok(())
}

fn raw_test_file_uri(path: &Path) -> String {
    format!("file://{}", path.to_string_lossy())
}

#[test]
fn codelens_keeps_references_when_workspace_owner_uri_encoding_differs() {
    let workspace_uri = "file:///workspace/(group-a)";
    let encoded_workspace_uri = "file:///workspace/%28group-a%29";
    let source_uri = "file:///workspace/%28group-a%29/src/App.tsx";
    let style_uri = "file:///workspace/(group-a)/src/App.module.scss";
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
                        "name": "group-a",
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
                    "text": "import bind from \"classnames/bind\";\nimport styles from \"./App.module.scss\";\nconst cx = bind.bind(styles);\nexport const view = <div className={cx(\"foo\")} />;",
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
                    "text": ".foo { color: red; }",
                },
            },
        }),
    );
    if let Some(document) = state.documents.get_mut(source_uri) {
        document.workspace_folder_uri = Some(encoded_workspace_uri.to_string());
    }

    let code_lens_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/codeLens",
            "params": {
                "textDocument": {
                    "uri": style_uri,
                },
            },
        }),
    );
    assert_eq!(
        code_lens_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/command/title")),
        Some(&json!("1 reference")),
    );
}
