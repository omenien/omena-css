use super::*;

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
