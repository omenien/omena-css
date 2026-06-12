use super::*;

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
fn projects_late_opened_style_type_facts_across_canonical_uri_mismatch() -> TestResult {
    let source_uri = "file:///workspace-late/src/Late.tsx";
    let style_uri = "file:///workspace-late/src/Late.module.scss";
    let raw_style_uri = "file:///workspace-late/src/./Late.module.scss";
    let source_text = r#"import bind from "classnames/bind";
import styles from "./Late.module.scss";
const cx = bind.bind(styles);
interface Props { tone: "info" | "warn"; }
export function LateBadge({ tone }: Props) {
  return <span className={cx(tone)} />;
}"#;
    let style_text = ".info { color: blue; }\n.warn { color: orange; }";

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
                        "uri": "file:///workspace-late",
                        "name": "workspace-late",
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
                    "uri": raw_style_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": style_text,
                },
            },
        }),
    );

    let tone_target = state
        .document(source_uri)
        .ok_or_else(|| std::io::Error::other("source document should be indexed"))?
        .source_syntax_index
        .type_fact_targets
        .iter()
        .find(|target| &source_text[target.byte_span.start..target.byte_span.end] == "tone")
        .cloned()
        .ok_or_else(|| std::io::Error::other("tone type fact target should exist"))?;
    assert_eq!(tone_target.target_style_uri.as_deref(), Some(style_uri));
    apply_source_type_fact_results_to_document(
        &mut state,
        source_uri,
        &[TsgoTypeFactResultEntryV0 {
            file_path: "/workspace-late/src/Late.tsx".to_string(),
            expression_id: tone_target.expression_id.clone(),
            resolved_type: TsgoResolvedTypeV0 {
                kind: "union",
                values: vec!["info".to_string(), "warn".to_string()],
            },
        }],
    );

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
                "position": parser_position_for_byte_offset(source_text, tone_target.byte_span.start),
            },
        }),
    );
    let hover_text = hover
        .as_ref()
        .and_then(|value| value.pointer("/result/contents/value"))
        .and_then(Value::as_str)
        .ok_or_else(|| std::io::Error::other("tone hover should render markdown"))?;
    assert!(hover_text.contains("`.info`"));
    assert!(hover_text.contains("`.warn`"));
    Ok(())
}
