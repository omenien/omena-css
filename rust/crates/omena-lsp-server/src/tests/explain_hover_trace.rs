use super::*;

#[test]
fn explain_hover_trace_reports_style_selector_definition() {
    let style_uri = "file:///workspace-a/src/App.module.scss";
    let style_text = ".root { color: red; }\n.theme { color: blue; }";
    let mut state = LspShellState::default();
    open_style_document(&mut state, style_uri, style_text);

    let response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": EXPLAIN_HOVER_TRACE_REQUEST,
            "params": {
                "textDocument": {
                    "uri": style_uri,
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
        Some(&json!("omena-lsp-server.explain-hover-trace")),
    );
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/fileKind")),
        Some(&json!("style")),
    );
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/matched")),
        Some(&json!(true)),
    );
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/definitionCount")),
        Some(&json!(1)),
    );
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/definitions/0/name")),
        Some(&json!("root")),
    );
    let rendered_markdown = response
        .as_ref()
        .and_then(|value| value.pointer("/result/renderedMarkdown"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    assert!(
        rendered_markdown.contains("color: red"),
        "rendered markdown: {rendered_markdown}"
    );
}

#[test]
fn explain_hover_trace_reports_source_selector_resolution() -> TestResult {
    let style_uri = "file:///workspace-a/src/App.module.scss";
    let source_uri = "file:///workspace-a/src/App.tsx";
    let style_text = ".foo { color: red; }\n";
    let source_text = "import bind from \"classnames/bind\";\nimport styles from \"./App.module.scss\";\nconst cx = bind.bind(styles);\nexport const view = <div className={cx(\"foo\")} />;\n";
    let mut state = LspShellState::default();
    initialize_workspace(&mut state);
    open_style_document(&mut state, style_uri, style_text);
    open_source_document(&mut state, source_uri, source_text);

    let response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": EXPLAIN_HOVER_TRACE_REQUEST,
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": parser_position_for_byte_offset(
                    source_text,
                    fixture_find(source_text, "\"foo\"", "source fixture contains selector")? + 2,
                ),
            },
        }),
    );

    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/product")),
        Some(&json!("omena-lsp-server.explain-hover-trace")),
    );
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/fileKind")),
        Some(&json!("source")),
    );
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/matched")),
        Some(&json!(true)),
    );
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/matchedCandidateCount")),
        Some(&json!(1)),
    );
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/definitionCount")),
        Some(&json!(1)),
    );
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/definitions/0/uri")),
        Some(&json!(style_uri)),
    );
    let rendered_markdown = response
        .as_ref()
        .and_then(|value| value.pointer("/result/renderedMarkdown"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    assert!(
        rendered_markdown.contains("color: red"),
        "rendered markdown: {rendered_markdown}"
    );
    Ok(())
}

#[test]
fn explain_hover_trace_reports_type_fact_targets_that_were_never_attempted() -> TestResult {
    let source_uri = "file:///workspace-a/src/App.tsx";
    let source_text = r#"import bind from "classnames/bind";
import styles from "./App.module.scss";
const cx = bind.bind(styles);
export const view = (active: boolean) =>
  <div className={cx(`theme-${active ? "a" : "legacy"}`)} />;
"#;
    let mut state = LspShellState::default();
    initialize_workspace(&mut state);
    open_source_document(&mut state, source_uri, source_text);

    let response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": EXPLAIN_HOVER_TRACE_REQUEST,
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": parser_position_for_byte_offset(
                    source_text,
                    fixture_find(source_text, "active ? \"a\"", "source fixture contains ternary")?,
                ),
            },
        }),
    );

    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/reason")),
        Some(&json!("noSourceCandidateAtPosition")),
    );
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/typeFactTier")),
        Some(&json!({
            "attempted": false,
            "outcome": "neverAttempted",
            "reason": "unsupportedExpressionShape",
            "skippedTargetCount": 1,
        })),
    );
    Ok(())
}

#[test]
fn explain_hover_trace_reports_unavailable_type_fact_provider_attempts() -> TestResult {
    let source_uri = "file:///workspace-a/src/App.tsx";
    let source_text = r#"import bind from "classnames/bind";
import styles from "./App.module.scss";
const cx = bind.bind(styles);
export const view = (variant: string) =>
  <div className={cx(`theme-${variant}`)} />;
"#;
    let mut state = LspShellState::default();
    initialize_workspace(&mut state);
    open_source_document(&mut state, source_uri, source_text);

    let response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": EXPLAIN_HOVER_TRACE_REQUEST,
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": parser_position_for_byte_offset(
                    source_text,
                    fixture_find(source_text, "variant}`", "source fixture contains identifier")?,
                ),
            },
        }),
    );

    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/typeFactTier")),
        Some(&json!({
            "attempted": true,
            "outcome": "unavailable",
            "reason": "projectMiss",
            "skippedTargetCount": 0,
        })),
    );
    Ok(())
}

#[test]
fn explain_hover_trace_reports_domain_class_value_universe() -> TestResult {
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
    initialize_workspace(&mut state);
    open_source_document(&mut state, source_uri, source_text);

    let response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": EXPLAIN_HOVER_TRACE_REQUEST,
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

    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/hoverKind")),
        Some(&json!("domainClassReference")),
    );
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/sourceOwner")),
        Some(&json!("button")),
    );
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/axisName")),
        Some(&json!("intent")),
    );
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/knownOptions")),
        Some(&json!(["primary", "secondary"])),
    );
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/optionName")),
        Some(&json!("primary")),
    );
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/prefix")),
        Some(&Value::Null),
    );
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/definitionCount")),
        Some(&json!(0)),
    );
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/resolutionPath")),
        Some(&json!([
            "sourceSyntaxIndex",
            "classValueUniverseProvider",
            "sourceDomainReferenceHover",
        ])),
    );
    let rendered_markdown = response
        .as_ref()
        .and_then(|value| value.pointer("/result/renderedMarkdown"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    assert!(
        rendered_markdown.contains("button.intent.primary"),
        "rendered markdown: {rendered_markdown}"
    );
    Ok(())
}

fn initialize_workspace(state: &mut LspShellState) {
    handle_lsp_message(
        state,
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
}

fn open_style_document(state: &mut LspShellState, uri: &str, text: &str) {
    handle_lsp_message(
        state,
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

fn open_source_document(state: &mut LspShellState, uri: &str, text: &str) {
    handle_lsp_message(
        state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "languageId": "typescriptreact",
                    "version": 1,
                    "text": text,
                },
            },
        }),
    );
}
