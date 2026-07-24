use super::*;

#[test]
fn narrows_finite_template_interpolations_without_a_type_provider() -> TestResult {
    let source_uri = "file:///workspace-a/src/App.tsx";
    let style_uri = "file:///workspace-a/src/App.module.scss";
    let source_text = r#"import bind from "classnames/bind";
import styles from "./App.module.scss";
const cx = bind.bind(styles);
export function App({ active }: { active: boolean }) {
  return <div className={cx(`theme-${active ? "a" : "legacy"}`)} />;
}"#;
    let style_text =
        ".theme-a {}\n.theme-legacy {}\n.theme-extra { color: var(--should-not-appear); }";

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

    let expression_offset = source_text
        .find("active ?")
        .ok_or_else(|| std::io::Error::other("fixture should contain a conditional"))?;
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
                "position": parser_position_for_byte_offset(source_text, expression_offset),
            },
        }),
    );
    let hover_text = hover
        .as_ref()
        .and_then(|value| value.pointer("/result/contents/value"))
        .and_then(Value::as_str)
        .ok_or_else(|| std::io::Error::other("conditional hover should render markdown"))?;

    assert!(hover_text.contains("`.theme-a`"));
    assert!(hover_text.contains("`.theme-legacy`"));
    assert!(!hover_text.contains("`.theme-extra`"));
    let explain_hover = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": EXPLAIN_HOVER_TRACE_REQUEST,
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": parser_position_for_byte_offset(source_text, expression_offset),
            },
        }),
    );
    let mut definition_names = explain_hover
        .as_ref()
        .and_then(|value| value.pointer("/result/definitions"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("explain hover should list definitions"))?
        .iter()
        .filter_map(|definition| definition.get("name").and_then(Value::as_str))
        .collect::<Vec<_>>();
    definition_names.sort();
    assert_eq!(definition_names, vec!["theme-a", "theme-legacy"]);

    let prefix_offset = source_text
        .find("theme-")
        .ok_or_else(|| std::io::Error::other("fixture should contain a template prefix"))?;
    let prefix_explain_hover = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": EXPLAIN_HOVER_TRACE_REQUEST,
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": parser_position_for_byte_offset(source_text, prefix_offset),
            },
        }),
    );
    let mut prefix_definition_names = prefix_explain_hover
        .as_ref()
        .and_then(|value| value.pointer("/result/definitions"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("prefix hover should list definitions"))?
        .iter()
        .filter_map(|definition| definition.get("name").and_then(Value::as_str))
        .collect::<Vec<_>>();
    prefix_definition_names.sort();
    assert_eq!(
        prefix_definition_names,
        vec!["theme-a", "theme-legacy"],
        "the retired prefix must not widen a fully enumerated template"
    );
    assert!(
        !state
            .document(source_uri)
            .ok_or_else(|| std::io::Error::other("source document should remain indexed"))?
            .source_syntax_index
            .selector_references
            .iter()
            .any(|reference| {
                reference.match_kind == SourceSelectorReferenceMatchKind::Prefix
                    && reference.selector_name.as_deref() == Some("theme-")
            }),
        "a fully enumerated native template must not retain its broader prefix fact"
    );
    Ok(())
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
    let style_text = ".medium { color: red; }\n.small { color: blue; }\n.font-size-10 { font-size: 10px; }\n.font-size-12 { font-size: 12px; }\n.font-size-extra { font-size: inherit; }";

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
    let entries = vec![
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
    ];
    apply_source_type_fact_results_to_document(&mut state, source_uri, entries.as_slice());

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
    let explain_hover = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 41,
            "method": EXPLAIN_HOVER_TRACE_REQUEST,
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": parser_position_for_byte_offset(
                    source_text,
                    size_target.byte_span.start,
                ),
            },
        }),
    );
    assert_eq!(
        explain_hover
            .as_ref()
            .and_then(|value| value.pointer("/result/typeFactTier")),
        Some(&json!({
            "attempted": true,
            "outcome": "resolved",
            "skippedTargetCount": 0,
        })),
    );
    let projected_references = state
        .document(source_uri)
        .ok_or_else(|| std::io::Error::other("source document should remain indexed"))?
        .source_syntax_index
        .selector_references
        .iter()
        .filter(|reference| {
            matches!(
                reference.selector_name.as_deref(),
                Some("medium" | "small" | "font-size-10" | "font-size-12")
            )
        })
        .collect::<Vec<_>>();
    assert!(!projected_references.is_empty());
    assert!(projected_references.iter().all(|reference| {
        reference.surface == SourceSelectorReferenceSurface::OmenaTsgoTypeFactProjection
    }));
    let serialized = serde_json::to_value(
        &state
            .document(source_uri)
            .ok_or_else(|| std::io::Error::other("source document should remain indexed"))?
            .source_syntax_index,
    )?;
    assert!(
        serialized
            .pointer("/selectorReferences")
            .and_then(Value::as_array)
            .is_some_and(|references| {
                references.iter().any(|reference| {
                    reference.get("surface") == Some(&json!("omenaTsgoTypeFactProjection"))
                })
            }),
        "tsgo-projected references should retain their serialized provenance"
    );
    let font_prefix_offset = source_text
        .find("font-size-")
        .ok_or_else(|| std::io::Error::other("source fixture should contain a template prefix"))?;
    let prefix_explain_hover = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 42,
            "method": EXPLAIN_HOVER_TRACE_REQUEST,
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": parser_position_for_byte_offset(source_text, font_prefix_offset),
            },
        }),
    );
    let mut prefix_definition_names = prefix_explain_hover
        .as_ref()
        .and_then(|value| value.pointer("/result/definitions"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("prefix hover should list definitions"))?
        .iter()
        .filter_map(|definition| definition.get("name").and_then(Value::as_str))
        .collect::<Vec<_>>();
    prefix_definition_names.sort();
    assert_eq!(
        prefix_definition_names,
        vec!["font-size-10", "font-size-12"],
        "a complete provider projection should supersede the template prefix"
    );

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
                    values: vec!["10".to_string(), "14".to_string()],
                },
            },
        ],
    );
    let partial_prefix_hover = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 8,
            "method": EXPLAIN_HOVER_TRACE_REQUEST,
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": parser_position_for_byte_offset(source_text, font_prefix_offset),
            },
        }),
    );
    let mut partial_definition_names = partial_prefix_hover
        .as_ref()
        .and_then(|value| value.pointer("/result/definitions"))
        .and_then(Value::as_array)
        .ok_or_else(|| {
            std::io::Error::other("partial provider projection should retain prefix definitions")
        })?
        .iter()
        .filter_map(|definition| definition.get("name").and_then(Value::as_str))
        .collect::<Vec<_>>();
    partial_definition_names.sort();
    assert_eq!(
        partial_definition_names,
        vec!["font-size-10", "font-size-12", "font-size-extra"],
        "an incomplete projection must retain the conservative prefix"
    );

    apply_source_type_fact_results_to_document(&mut state, source_uri, &[]);
    let unavailable_prefix_hover = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 9,
            "method": EXPLAIN_HOVER_TRACE_REQUEST,
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": parser_position_for_byte_offset(source_text, font_prefix_offset),
            },
        }),
    );
    let mut unavailable_definition_names = unavailable_prefix_hover
        .as_ref()
        .and_then(|value| value.pointer("/result/definitions"))
        .and_then(Value::as_array)
        .ok_or_else(|| {
            std::io::Error::other("unavailable provider prefix hover should list definitions")
        })?
        .iter()
        .filter_map(|definition| definition.get("name").and_then(Value::as_str))
        .collect::<Vec<_>>();
    unavailable_definition_names.sort();
    assert_eq!(
        unavailable_definition_names,
        vec!["font-size-10", "font-size-12", "font-size-extra"],
        "provider failure must restore the conservative prefix rather than under-approximate"
    );
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
