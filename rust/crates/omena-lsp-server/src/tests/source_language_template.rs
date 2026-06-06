use super::*;

#[test]
fn resolves_server_template_class_definition_to_opened_style_selector() -> TestResult {
    let source = r#"{% if enabled %}
<main class="root">content</main>
{% endif %}
"#;
    let style = ".root { color: red; }\n";
    let source_uri = "file:///workspace-a/src/page.liquid";
    let style_uri = "file:///workspace-a/src/Page.module.scss";
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
                    "languageId": "liquid",
                    "version": 1,
                    "text": source,
                },
            },
        }),
    );

    let document = state
        .document(source_uri)
        .ok_or_else(|| std::io::Error::other("Liquid document is open"))?;
    assert!(
        document
            .source_syntax_index
            .selector_references
            .iter()
            .any(|reference| {
                &source[reference.byte_span.start..reference.byte_span.end] == "root"
                    && reference.target_style_uri.is_none()
            })
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
                    fixture_find(source, "root", "Liquid template contains root class")? + 1,
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
