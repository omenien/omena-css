use super::*;

fn open_document(state: &mut LspShellState, uri: &str, language_id: &str, text: &str) {
    handle_lsp_message(
        state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "languageId": language_id,
                    "version": 1,
                    "text": text,
                },
            },
        }),
    );
}

#[test]
fn escaped_css_module_member_supports_navigation_diagnostics_and_completion() -> TestResult {
    let style_uri = "file:///workspace-a/src/App.module.css";
    let source_uri = "file:///workspace-a/src/App.tsx";
    let style_text = r#".\6d d\:flex { display: flex; }
.plain { display: block; }
"#;
    let source_text = r#"import styles from "./App.module.css";
export const escapedClass = styles["md\\:flex"];
export const plainClass = styles.plain;
"#;
    let escaped_access_offset = fixture_find(
        source_text,
        r#"md\\:flex"#,
        "source contains escaped computed member",
    )? + 2;
    let plain_access_offset =
        fixture_find(source_text, "styles.plain", "source contains plain member")?
            + "styles.".len()
            + 2;
    let escaped_style_offset =
        fixture_find(style_text, r#"\6d d\:flex"#, "style contains escaped class")? + 2;
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 0,
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
    open_document(&mut state, source_uri, "typescriptreact", source_text);
    open_document(&mut state, style_uri, "css", style_text);
    let source_document = state
        .document(source_uri)
        .ok_or_else(|| std::io::Error::other("source document should be open"))?;
    assert!(
        source_document
            .source_syntax_index
            .selector_references
            .iter()
            .any(|reference| reference.selector_name.as_deref() == Some(r#"md\:flex"#)),
        "source syntax index should carry the JS-decoded CSS spelling: {:?}",
        source_document.source_syntax_index.selector_references
    );
    assert!(
        source_document
            .source_selector_candidates
            .iter()
            .any(|candidate| candidate.name.to_string() == r#"md\:flex"#),
        "LSP candidates should preserve the semantic selector name: {:?}",
        source_document.source_selector_candidates
    );

    for (id, offset) in [(1, escaped_access_offset), (2, plain_access_offset)] {
        let definition = handle_lsp_message(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": "textDocument/definition",
                "params": {
                    "textDocument": { "uri": source_uri },
                    "position": parser_position_for_byte_offset(source_text, offset),
                },
            }),
        );
        assert!(
            definition
                .as_ref()
                .and_then(|response| response.pointer("/result"))
                .and_then(Value::as_array)
                .is_some_and(|locations| locations.iter().any(|location| {
                    location.get("uri").and_then(Value::as_str) == Some(style_uri)
                })),
            "source definition should resolve to the style class: {definition:?}"
        );
    }

    let references = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/references",
            "params": {
                "textDocument": { "uri": style_uri },
                "position": parser_position_for_byte_offset(style_text, escaped_style_offset),
                "context": { "includeDeclaration": true },
            },
        }),
    );
    assert!(
        references
            .as_ref()
            .and_then(|response| response.pointer("/result"))
            .and_then(Value::as_array)
            .is_some_and(|locations| locations.iter().any(|location| {
                location.get("uri").and_then(Value::as_str) == Some(source_uri)
            })),
        "escaped class references should include the TSX member access: {references:?}"
    );

    let diagnostics = resolve_style_diagnostics_for_uri(&state, style_uri);
    let unused_messages = diagnostics
        .as_array()
        .into_iter()
        .flatten()
        .filter(|diagnostic| diagnostic.get("code") == Some(&json!("unusedSelector")))
        .filter_map(|diagnostic| diagnostic.get("message").and_then(Value::as_str))
        .collect::<Vec<_>>();
    assert!(
        !unused_messages
            .iter()
            .any(|message| message.contains("md") || message.contains("plain")),
        "both escaped and plain references should suppress unusedSelector: {unused_messages:?}"
    );

    let completion = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "textDocument/completion",
            "params": {
                "textDocument": { "uri": source_uri },
                "position": parser_position_for_byte_offset(
                    source_text,
                    escaped_access_offset,
                ),
            },
        }),
    );
    let escaped_item = completion
        .as_ref()
        .and_then(|response| response.pointer("/result/items"))
        .and_then(Value::as_array)
        .and_then(|items| {
            items
                .iter()
                .find(|item| item.get("label") == Some(&json!(r#"\6d d\:flex"#)))
        })
        .ok_or_else(|| std::io::Error::other("escaped class completion should be present"))?;
    assert_eq!(
        escaped_item.get("insertText"),
        Some(&json!(r#"\\6d d\\:flex"#)),
        "computed-member completion must escape the CSS spelling for a JS string"
    );
    Ok(())
}
