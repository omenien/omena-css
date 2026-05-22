use super::*;

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
