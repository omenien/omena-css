use omena_query::{
    OmenaQueryExplainInputV0, OmenaQueryExplainSymbolKindV0, OmenaQueryTransformExecutionContextV0,
    execute_omena_query_transform_passes_from_source, explain_omena_query,
    explain_omena_query_tree_shake_for_style_source,
    resolve_omena_query_source_precision_for_source,
    summarize_omena_query_style_diagnostics_for_file,
};

use super::*;

#[test]
fn shared_explain_request_preserves_query_response_structure() -> TestResult {
    let style_uri = "file:///workspace-a/src/App.module.scss";
    let style_text = "@import 'legacy';\n.button { color: red; }\n";
    let tree_style_uri = "file:///workspace-a/src/Tree.module.css";
    let tree_style_text = ".button { color: red; }\n";
    let source_uri = "file:///workspace-a/src/App.ts";
    let source_text = "const className = 'button';\nclassName;";
    let mut state = LspShellState::default();
    open_document(&mut state, style_uri, "scss", style_text);
    open_document(&mut state, tree_style_uri, "css", tree_style_text);
    open_document(&mut state, source_uri, "typescript", source_text);

    let diagnostic = summarize_omena_query_style_diagnostics_for_file(style_uri, style_text, &[])
        .diagnostics
        .into_iter()
        .find(|diagnostic| diagnostic.code == "deprecatedSassImport")
        .ok_or("fixture should produce the requested diagnostic")?;
    let diagnostic_direct = explain_omena_query(OmenaQueryExplainInputV0::Diagnostic {
        style_path: style_uri,
        diagnostic: &diagnostic,
    });
    assert_explain_result_equals(
        &mut state,
        json!({
            "kind": "diagnostic",
            "textDocument": { "uri": style_uri },
            "code": "deprecatedSassImport",
        }),
        &diagnostic_direct,
    );

    let execution = execute_omena_query_transform_passes_from_source(
        style_uri,
        style_text,
        &["print-css".to_string()],
    );
    let transform_direct = explain_omena_query(OmenaQueryExplainInputV0::Transform {
        decision: &execution.execution.decisions[0],
        decision_ordinal: 0,
    });
    assert_explain_result_equals(
        &mut state,
        json!({
            "kind": "transform",
            "textDocument": { "uri": style_uri },
            "passId": "print-css",
        }),
        &transform_direct,
    );

    let context: OmenaQueryTransformExecutionContextV0 = serde_json::from_value(json!({
        "reachableClassNames": ["button"],
    }))?;
    let tree_shake_direct = explain_omena_query_tree_shake_for_style_source(
        tree_style_uri,
        tree_style_text,
        &context,
        OmenaQueryExplainSymbolKindV0::Class,
        "button",
    )
    .ok_or("fixture should produce a closed-world bundle")?;
    assert_explain_result_equals(
        &mut state,
        json!({
            "kind": "treeShake",
            "textDocument": { "uri": tree_style_uri },
            "symbolKind": "class",
            "symbolName": "button",
            "context": { "reachableClassNames": ["button"] },
        }),
        &tree_shake_direct,
    );

    let reference_offset = source_text.rfind("className").ok_or("missing reference")?;
    let precision_reference = resolve_omena_query_source_precision_for_source(
        source_uri,
        source_text,
        Some("typescript"),
        "className",
        reference_offset,
    );
    let precision_direct = explain_omena_query(OmenaQueryExplainInputV0::Precision {
        reference: &precision_reference,
    });
    assert_explain_result_equals(
        &mut state,
        json!({
            "kind": "precision",
            "textDocument": { "uri": source_uri },
            "position": parser_position_for_byte_offset(source_text, reference_offset),
            "variableName": "className",
        }),
        &precision_direct,
    );

    let style_graph = explain_request(
        &mut state,
        json!({
            "kind": "styleGraph",
            "textDocument": { "uri": style_uri },
        }),
    );
    assert_eq!(
        style_graph["product"],
        json!("omena-lsp-server.style-graph")
    );
    assert_eq!(style_graph["availability"], json!("available"));
    assert_ne!(style_graph["graph"], Value::Null);
    Ok(())
}

#[test]
fn hover_trace_keeps_its_wire_shape_while_using_the_shared_egress() {
    let before = crate::explain::hover_egress_projection_count();
    let mut state = LspShellState::default();
    let response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": EXPLAIN_HOVER_TRACE_REQUEST,
            "params": {
                "textDocument": { "uri": "file:///workspace-a/src/App.module.scss" },
            },
        }),
    );
    assert_eq!(
        response.and_then(|value| value.get("result").cloned()),
        Some(json!({
            "schemaVersion": "0",
            "product": "omena-lsp-server.explain-hover-trace",
            "documentUri": "file:///workspace-a/src/App.module.scss",
            "workspaceFolderUri": null,
            "fileKind": "unknown",
            "queryPosition": null,
            "matched": false,
            "reason": "missingPosition",
            "candidateCount": 0,
            "definitionCount": 0,
            "candidates": [],
            "definitions": [],
            "resolutionPath": [],
            "readySurfaces": ["explainHoverTraceRpc"],
        })),
    );
    assert_eq!(crate::explain::hover_egress_projection_count(), before + 1);
}

fn assert_explain_result_equals<T: serde::Serialize>(
    state: &mut LspShellState,
    params: Value,
    expected: &T,
) {
    assert_eq!(explain_request(state, params), json!(expected));
}

fn explain_request(state: &mut LspShellState, params: Value) -> Value {
    handle_lsp_message(
        state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": EXPLAIN_REQUEST,
            "params": params,
        }),
    )
    .and_then(|value| value.get("result").cloned())
    .unwrap_or(Value::Null)
}

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
