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
fn projects_tsgo_type_facts_for_typed_cx_identifiers_and_template_holes() -> TestResult {
    let source_uri = "file:///workspace-a/src/App.tsx";
    let style_uri = "file:///workspace-a/src/App.module.scss";
    let source_text = r#"import bind from "classnames/bind";
import styles from "./App.module.scss";
const cx = bind.bind(styles);
interface BadgeProps { size: "medium" | "small"; fontSize?: 10 | 12; }
export function Badge({ size, fontSize }: BadgeProps) {
  return <span className={cx(size, `font-size-${fontSize}`)} />;
}"#;
    let style_text = ".medium { color: red; }\n.small { color: blue; }\n.font-size-10 { font-size: 10px; }\n.font-size-12 { font-size: 12px; }";

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

    let type_fact_targets = state
        .document(source_uri)
        .ok_or_else(|| std::io::Error::other("source document should be indexed"))?
        .source_syntax_index
        .type_fact_targets
        .clone();
    let size_target = type_fact_targets
        .iter()
        .find(|target| &source_text[target.byte_span.start..target.byte_span.end] == "size")
        .ok_or_else(|| std::io::Error::other("size type fact target should exist"))?;
    let font_size_target = type_fact_targets
        .iter()
        .find(|target| &source_text[target.byte_span.start..target.byte_span.end] == "fontSize")
        .ok_or_else(|| std::io::Error::other("fontSize type fact target should exist"))?;
    apply_source_type_fact_results_to_document(
        &mut state,
        source_uri,
        &[
            TsgoTypeFactResultEntryV0 {
                file_path: "/workspace-a/src/App.tsx".to_string(),
                expression_id: size_target.expression_id.clone(),
                resolved_type: TsgoResolvedTypeV0 {
                    kind: "union",
                    values: vec!["medium".to_string(), "small".to_string()],
                },
            },
            TsgoTypeFactResultEntryV0 {
                file_path: "/workspace-a/src/App.tsx".to_string(),
                expression_id: font_size_target.expression_id.clone(),
                resolved_type: TsgoResolvedTypeV0 {
                    kind: "union",
                    values: vec!["10".to_string(), "12".to_string()],
                },
            },
        ],
    );

    let size_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": parser_position_for_byte_offset(source_text, size_target.byte_span.start),
            },
        }),
    );
    let size_results = size_definition
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("size definition should return results"))?;
    assert_eq!(size_results.len(), 2);
    assert_eq!(size_results[0].get("uri"), Some(&json!(style_uri)));
    assert_eq!(size_results[1].get("uri"), Some(&json!(style_uri)));

    let font_size_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": parser_position_for_byte_offset(source_text, font_size_target.byte_span.start),
            },
        }),
    );
    let font_size_results = font_size_definition
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("fontSize definition should return results"))?;
    assert_eq!(font_size_results.len(), 2);

    let size_hover = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "textDocument/hover",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": parser_position_for_byte_offset(source_text, size_target.byte_span.start),
            },
        }),
    );
    let hover_text = size_hover
        .as_ref()
        .and_then(|value| value.pointer("/result/contents/value"))
        .and_then(Value::as_str)
        .ok_or_else(|| std::io::Error::other("size hover should render markdown"))?;
    assert!(hover_text.contains("`.medium`"));
    assert!(hover_text.contains("`.small`"));

    let size_references = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "textDocument/references",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": parser_position_for_byte_offset(source_text, size_target.byte_span.start),
                "context": {
                    "includeDeclaration": true,
                },
            },
        }),
    );
    let reference_results = size_references
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("size references should return results"))?;
    assert!(
        reference_results
            .iter()
            .any(|location| location.get("uri") == Some(&json!(style_uri)))
    );
    assert!(
        reference_results
            .iter()
            .any(|location| location.get("uri") == Some(&json!(source_uri)))
    );

    let size_prepare_rename = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "textDocument/prepareRename",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": parser_position_for_byte_offset(source_text, size_target.byte_span.start),
            },
        }),
    );
    let placeholder = size_prepare_rename
        .as_ref()
        .and_then(|value| value.pointer("/result/placeholder"))
        .and_then(Value::as_str)
        .ok_or_else(|| std::io::Error::other("size prepareRename should use CSS selector path"))?;
    assert!(matches!(placeholder, "medium" | "small"));

    let size_rename = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "textDocument/rename",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": parser_position_for_byte_offset(source_text, size_target.byte_span.start),
                "newName": "large",
            },
        }),
    );
    let style_edits = size_rename
        .as_ref()
        .and_then(|value| value.pointer("/result/changes"))
        .and_then(|changes| changes.get(style_uri))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("size rename should produce style edits"))?;
    assert!(!style_edits.is_empty());
    Ok(())
}

#[test]
fn narrows_source_completion_candidates_by_property_access_prefix() -> TestResult {
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
                    "text": "import styles from \"./App.module.scss\";\nconst view = styles.ro;",
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
                    "text": ".root { display: block; }\n.row { display: flex; }\n.active { color: red; }",
                },
            },
        }),
    );

    let completion_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/completion",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.tsx",
                },
                "position": {
                    "line": 1,
                    "character": 22,
                },
            },
        }),
    );

    let items = completion_response
        .as_ref()
        .and_then(|value| value.pointer("/result/items"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("completion response should contain items"))?;
    let labels: Vec<String> = items
        .iter()
        .filter_map(|item| {
            item.get("label")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        })
        .collect();
    assert_eq!(labels, vec!["root".to_string(), "row".to_string()]);
    assert_eq!(
        items
            .first()
            .and_then(|item| item.pointer("/data/rankingSource")),
        Some(&json!("targetAndPrefixNarrowing")),
    );
    assert!(
        items
            .first()
            .and_then(|item| item.get("sortText"))
            .and_then(Value::as_str)
            .is_some_and(|sort_text| sort_text.starts_with("10-00-00-"))
    );
    Ok(())
}

#[test]
fn ranks_source_completion_with_value_domain_projection() -> TestResult {
    let source_uri = "file:///workspace-a/src/App.tsx";
    let style_uri = "file:///workspace-a/src/App.module.scss";
    let source_text = r#"import bind from "classnames/bind";
import styles from "./App.module.scss";
const cx = bind.bind(styles);
const view = <div className={cx(`item--${variant}`)} />;
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
                    "text": ".item--large {}\n.item--primary {}\n.item--secondary {}\n.item--muted {}\n",
                },
            },
        }),
    );

    let expression_id = state
        .document(source_uri)
        .and_then(|document| document.source_syntax_index.type_fact_targets.first())
        .map(|target| target.expression_id.clone())
        .ok_or_else(|| std::io::Error::other("expected a source type-fact target"))?;
    apply_source_type_fact_results_to_document(
        &mut state,
        source_uri,
        &[TsgoTypeFactResultEntryV0 {
            file_path: "/workspace-a/src/App.tsx".to_string(),
            expression_id,
            resolved_type: TsgoResolvedTypeV0 {
                kind: "union",
                values: vec!["primary".to_string(), "secondary".to_string()],
            },
        }],
    );

    let position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "variant}`",
            "source fixture contains dynamic selector expression",
        )?,
    );
    let completion_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/completion",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": position,
            },
        }),
    );

    let items = completion_response
        .as_ref()
        .and_then(|value| value.pointer("/result/items"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("completion response should contain items"))?;
    let labels = items
        .iter()
        .filter_map(|item| item.get("label").and_then(Value::as_str))
        .collect::<Vec<_>>();
    assert_eq!(
        labels,
        vec![
            "item--primary",
            "item--secondary",
            "item--large",
            "item--muted"
        ]
    );
    assert_eq!(
        items
            .first()
            .and_then(|item| item.pointer("/data/rankingSource")),
        Some(&json!("valueDomainSelectorProjection")),
    );
    assert!(
        items
            .first()
            .and_then(|item| item.get("sortText"))
            .and_then(Value::as_str)
            .is_some_and(|sort_text| sort_text.starts_with("00-00-"))
    );
    Ok(())
}

#[test]
fn source_hover_renders_unopened_target_style_rule_from_disk() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-disk-style-hover-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let src_dir = workspace_path.join("src");
    let source_path = src_dir.join("App.tsx");
    let style_path = src_dir.join("App.module.scss");
    fs::create_dir_all(src_dir.as_path())?;
    fs::write(style_path.as_path(), ".foo { color: red; }\n")?;

    let workspace_uri = path_to_file_uri(workspace_path.as_path());
    let source_uri = path_to_file_uri(source_path.as_path());
    let source_text = "import bind from \"classnames/bind\";\nimport styles from \"./App.module.scss\";\nconst cx = bind.bind(styles);\nexport const view = <div className={cx(\"foo\")} />;";
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
                    "languageId": "typescriptreact",
                    "version": 1,
                    "text": source_text,
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
                    "uri": path_to_file_uri(source_path.as_path()),
                },
                "position": parser_position_for_byte_offset(
                    source_text,
                    fixture_find(source_text, "\"foo\"", "source fixture contains foo")? + 1,
                ),
            },
        }),
    );
    let hover_text = hover_response
        .as_ref()
        .and_then(|value| value.pointer("/result/contents/value"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    assert!(
        hover_text.contains("color: red"),
        "hover text: {hover_text}"
    );

    let _ = fs::remove_dir_all(workspace_path.as_path());
    Ok(())
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
fn resolves_classnames_bind_dynamic_source_expressions() -> TestResult {
    let source_text = r#"import bind from "classnames/bind";
import styles from "./styles.module.scss";
const cx = bind.bind(styles);
const tone = "item--primary";
const icon = { glyph: "item__icon" };
const prefix = "item--";
export const view = <div className={cx(tone, icon.glyph, `item--${variant}`, { "item--danger": danger, item__label: true }, ok && "item--ok", active ? "item--on" : "item--off", prefix + state)} />;
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
                    "uri": "file:///workspace-a/src/App.tsx",
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
                    "uri": "file:///workspace-a/src/styles.module.scss",
                    "languageId": "scss",
                    "version": 1,
                    "text": ".item--primary {}\n.item__icon {}\n.item--large {}\n.item--danger {}\n.item__label {}\n.item--ok {}\n.item--on {}\n.item--off {}\n.item--muted {}\n",
                },
            },
        }),
    );

    let tone_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "tone,",
            "source fixture contains tone reference",
        )?,
    );
    let tone_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.tsx",
                },
                "position": tone_position,
            },
        }),
    );
    assert_eq!(
        tone_definition
            .as_ref()
            .and_then(|value| value.pointer("/result/0/range")),
        Some(&json!({
            "start": {
                "line": 0,
                "character": 1,
            },
            "end": {
                "line": 0,
                "character": 14,
            },
        })),
    );

    let icon_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "icon.glyph",
            "source fixture contains object property reference",
        )?,
    );
    let icon_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.tsx",
                },
                "position": icon_position,
            },
        }),
    );
    assert_eq!(
        icon_definition
            .as_ref()
            .and_then(|value| value.pointer("/result/0/range")),
        Some(&json!({
            "start": {
                "line": 1,
                "character": 1,
            },
            "end": {
                "line": 1,
                "character": 11,
            },
        })),
    );

    let template_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "`item--",
            "source fixture contains template prefix reference",
        )? + 1,
    );
    let template_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.tsx",
                },
                "position": template_position,
            },
        }),
    );
    let template_targets = template_definition
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(
        template_targets
            .iter()
            .any(|target| target.pointer("/range/start/line") == Some(&json!(2)))
    );
    assert!(
        !template_targets
            .iter()
            .any(|target| target.pointer("/range/start/line") == Some(&json!(1)))
    );

    let object_key_position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "item__label",
            "source fixture contains object key reference",
        )?,
    );
    let object_key_definition = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.tsx",
                },
                "position": object_key_position,
            },
        }),
    );
    assert_eq!(
        object_key_definition
            .as_ref()
            .and_then(|value| value.pointer("/result/0/range/start/line")),
        Some(&json!(4)),
    );
    Ok(())
}

#[test]
fn resolves_source_references_from_asi_imports_without_panicking() {
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
                    "text": "import {WidgetA, WidgetB} from \"@repo/widgets\"\nimport styles from \"./styles.module.scss\"\nconst view = <div className={styles.root} />",
                },
            },
        }),
    );
    let source_index = state
        .document("file:///workspace-a/src/App.tsx")
        .map(|document| document.source_syntax_index.clone());
    assert_eq!(
        source_index
            .as_ref()
            .map(|index| index.imported_style_bindings.as_slice()),
        Some(
            [ImportedStyleBinding {
                binding: "styles".to_string(),
                style_uri: "file:///workspace-a/src/styles.module.scss".to_string(),
            }]
            .as_slice()
        ),
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
                    "line": 2,
                    "character": 37,
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
fn opens_source_with_multibyte_escaped_strings_without_panicking() {
    let source_text = r#"import bind from "classnames/bind";
import styles from "./styles.module.scss";
const cx = bind.bind(styles);
const label = "\비";
export const view = <div className={cx("root", label && `상태-${tone}`)} />;
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
                    "uri": "file:///workspace-a/src/App.tsx",
                    "languageId": "typescriptreact",
                    "version": 1,
                    "text": source_text,
                },
            },
        }),
    );

    let source_index = state
        .document("file:///workspace-a/src/App.tsx")
        .map(|document| document.source_syntax_index.clone());
    assert!(
        source_index
            .as_ref()
            .is_some_and(|index| !index.selector_references.is_empty())
    );
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
