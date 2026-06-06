use super::*;

#[test]
fn resolves_svelte_script_css_module_definition_to_opened_style_selector() -> TestResult {
    let source = r#"<script lang="ts">
import styles from "./Card.module.scss";
export const root = styles.root;
</script>

<section class="ignored">{root}</section>
"#;
    let style = ".root { color: red; }\n";
    let source_uri = "file:///workspace-a/src/Card.svelte";
    let style_uri = "file:///workspace-a/src/Card.module.scss";
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
                    "uri": style_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": style,
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
                    "uri": source_uri,
                    "languageId": "svelte",
                    "version": 1,
                    "text": source,
                },
            },
        }),
    );

    let document = state
        .document(source_uri)
        .ok_or_else(|| std::io::Error::other("Svelte document is open"))?;
    assert_eq!(
        document
            .source_syntax_index
            .imported_style_bindings
            .as_slice(),
        [ImportedStyleBinding {
            binding: "styles".to_string(),
            style_uri: style_uri.to_string(),
        }]
        .as_slice(),
    );

    let definition_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": parser_position_for_byte_offset(
                    source,
                    fixture_find(source, "styles.root", "Svelte source contains styles.root")?
                        + "styles.".len()
                        + 1,
                ),
            },
        }),
    );
    let expected_selector_start =
        fixture_find(style, ".root { color", "style contains root selector")? + 1;

    assert_eq!(
        definition_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/uri")),
        Some(&json!(style_uri)),
    );
    assert_eq!(
        definition_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/range")),
        Some(&json!(parser_range_for_byte_span(
            style,
            ParserByteSpanV0 {
                start: expected_selector_start,
                end: expected_selector_start + "root".len(),
            },
        ))),
    );
    Ok(())
}
