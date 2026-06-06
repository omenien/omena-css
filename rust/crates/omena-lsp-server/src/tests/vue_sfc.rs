use super::*;

#[test]
fn resolves_vue_sfc_use_css_module_definition_to_embedded_style_module_selector() -> TestResult {
    let source = r#"<template><div :class="styles.root" /></template>
<script setup lang="ts">
import { useCssModule } from "vue";
const styles = useCssModule();
export const root = styles.root;
</script>
<style module lang="scss">
.root { color: red; }
</style>
"#;
    let vue_uri = "file:///workspace-a/src/Card.vue";
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
                    "uri": vue_uri,
                    "languageId": "vue",
                    "version": 1,
                    "text": source,
                },
            },
        }),
    );

    let document = state
        .document(vue_uri)
        .ok_or_else(|| std::io::Error::other("Vue document is open"))?;
    assert_eq!(
        document
            .style_summary
            .as_ref()
            .map(|summary| summary.selector_names.as_slice()),
        Some(["root".to_string()].as_slice()),
    );
    assert_eq!(
        document
            .source_syntax_index
            .imported_style_bindings
            .as_slice(),
        [ImportedStyleBinding {
            binding: "styles".to_string(),
            style_uri: vue_uri.to_string(),
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
                    "uri": vue_uri,
                },
                "position": parser_position_for_byte_offset(
                    source,
                    fixture_find(source, "styles.root", "Vue source contains styles.root")?
                        + "styles.".len()
                        + 1,
                ),
            },
        }),
    );
    let expected_selector_start =
        fixture_find(source, ".root { color", "Vue style contains root selector")? + 1;

    assert_eq!(
        definition_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/uri")),
        Some(&json!(vue_uri)),
    );
    assert_eq!(
        definition_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/range")),
        Some(&json!(parser_range_for_byte_span(
            source,
            ParserByteSpanV0 {
                start: expected_selector_start,
                end: expected_selector_start + "root".len(),
            },
        ))),
    );

    let template_definition_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": vue_uri,
                },
                "position": parser_position_for_byte_offset(
                    source,
                    fixture_find(source, ":class=\"styles.root", "Vue template contains styles.root")?
                        + ":class=\"styles.".len()
                        + 1,
                ),
            },
        }),
    );
    assert_eq!(
        template_definition_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/uri")),
        Some(&json!(vue_uri)),
    );
    assert_eq!(
        template_definition_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/range")),
        Some(&json!(parser_range_for_byte_span(
            source,
            ParserByteSpanV0 {
                start: expected_selector_start,
                end: expected_selector_start + "root".len(),
            },
        ))),
    );

    let diagnostics_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": SOURCE_DIAGNOSTICS_REQUEST,
            "params": {
                "textDocument": {
                    "uri": vue_uri,
                },
            },
        }),
    );
    assert_eq!(
        diagnostics_response
            .as_ref()
            .and_then(|value| value.pointer("/result")),
        Some(&json!([])),
    );
    Ok(())
}
