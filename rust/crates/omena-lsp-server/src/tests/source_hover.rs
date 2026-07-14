use super::*;

#[test]
fn utility_hover_cites_matching_selector_from_existing_style_graph() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-utility-hover-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let src_dir = workspace_path.join("src");
    fs::create_dir_all(src_dir.as_path())?;
    fs::write(
        workspace_path.join("tailwind.config.ts"),
        r##"export default { theme: { extend: { colors: { brand: "#123" } } } }"##,
    )?;
    let source_path = src_dir.join("App.tsx");
    let style_path = src_dir.join("utilities.css");
    let source_text = r#"export const App = () => <div className="bg-brand" />;"#;
    let source_uri = path_to_file_uri(source_path.as_path());
    let style_uri = path_to_file_uri(style_path.as_path());
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": { "workspaceFolders": [{
                "uri": path_to_file_uri(workspace_path.as_path()),
                "name": "workspace",
            }] },
        }),
    );
    for (uri, language_id, text) in [
        (style_uri.as_str(), "css", ".bg-brand { color: #123; }"),
        (source_uri.as_str(), "typescriptreact", source_text),
    ] {
        handle_lsp_message(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": { "textDocument": {
                    "uri": uri,
                    "languageId": language_id,
                    "version": 1,
                    "text": text,
                } },
            }),
        );
    }

    let response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/hover",
            "params": {
                "textDocument": { "uri": source_uri },
                "position": parser_position_for_byte_offset(
                    source_text,
                    fixture_find(source_text, "bg-brand", "source fixture contains class")? + 2,
                ),
            },
        }),
    );
    let hover_text = response
        .as_ref()
        .and_then(|value| value.pointer("/result/contents/value"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    assert!(
        hover_text.contains("workspace.class.bg-brand"),
        "{hover_text}"
    );
    assert!(hover_text.contains("CSS graph bindings:"), "{hover_text}");
    assert!(hover_text.contains(style_uri.as_str()), "{hover_text}");

    let _ = fs::remove_dir_all(workspace_path);
    Ok(())
}

#[test]
fn source_hover_renders_variant_recipe_option_universe() -> TestResult {
    let source_uri = "file:///workspace-a/src/App.tsx";
    let source_text = r#"import { cva } from "class-variance-authority";
const button = cva("btn", {
  variants: {
    intent: {
      primary: "btn-primary",
      secondary: "btn-secondary",
    },
  },
});
const view = button({ intent: "primary" });
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

    let hover_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/hover",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": parser_position_for_byte_offset(
                    source_text,
                    fixture_find(source_text, "primary\" });", "source fixture contains option")? + 2,
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
        hover_text.contains("button.intent.primary"),
        "hover text: {hover_text}"
    );
    assert!(
        hover_text.contains("known option"),
        "hover text: {hover_text}"
    );
    assert!(
        hover_text.contains("Known options: `primary`, `secondary`."),
        "hover text: {hover_text}"
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
fn source_hover_renders_type_fact_projected_selector_definitions() -> TestResult {
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
                    "text": ".item--primary { color: red; }\n.item--secondary { color: blue; }\n.item--muted { color: gray; }\n",
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

    let hover_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/hover",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": parser_position_for_byte_offset(
                    source_text,
                    fixture_find(source_text, "variant}`", "source fixture contains dynamic variant")?,
                ),
            },
        }),
    );
    let hover_text = hover_response
        .as_ref()
        .and_then(|value| value.pointer("/result/contents/value"))
        .and_then(Value::as_str)
        .unwrap_or_default();

    assert!(hover_text.contains(".item--primary"), "{hover_text}");
    assert!(hover_text.contains("color: red"), "{hover_text}");
    assert!(hover_text.contains(".item--secondary"), "{hover_text}");
    assert!(hover_text.contains("color: blue"), "{hover_text}");
    assert!(!hover_text.contains(".item--muted"), "{hover_text}");
    Ok(())
}
