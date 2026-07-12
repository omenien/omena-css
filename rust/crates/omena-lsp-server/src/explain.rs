use omena_query::{
    OmenaQueryExplainFactReferenceV0, OmenaQueryExplainFactValueV0, OmenaQueryExplainInputV0,
    OmenaQueryExplainSymbolKindV0, OmenaQueryTransformExecutionContextV0,
    execute_omena_query_transform_passes_from_source, explain_omena_query,
    explain_omena_query_tree_shake_for_style_source, explain_omena_query_tree_shake_unavailable,
    resolve_omena_query_source_precision_for_source,
    summarize_omena_query_style_diagnostics_for_file,
};
use serde_json::{Value, json};

use crate::{
    LspShellState, is_style_document_uri,
    protocol::{
        byte_offset_for_parser_position, document_uri_from_params, lsp_position_from_params,
    },
    resolve_style_context_index,
};

pub(crate) const EXPLAIN_REQUEST: &str = "omena/explain";

pub(crate) fn resolve_lsp_explain(state: &LspShellState, params: Option<&Value>) -> Value {
    let Some(kind) = params
        .and_then(|value| value.get("kind"))
        .and_then(Value::as_str)
    else {
        return invalid_explain_request("missing explain kind");
    };
    if kind == "styleGraph" {
        return json!({
            "schemaVersion": "0",
            "product": "omena-lsp-server.style-graph",
            "availability": "available",
            "graph": resolve_style_context_index(state, params),
        });
    }

    let document_uri = document_uri_from_params(params);
    let Some(document) = state.document(document_uri.as_str()) else {
        return invalid_explain_request("document is not indexed");
    };
    let response = match kind {
        "diagnostic" => {
            if !is_style_document_uri(document.uri.as_str()) {
                return invalid_explain_request("diagnostic explanation requires a style document");
            }
            let summary = summarize_omena_query_style_diagnostics_for_file(
                document.uri.as_str(),
                document.text.as_str(),
                &[],
            );
            let requested_code = params
                .and_then(|value| value.get("code"))
                .and_then(Value::as_str);
            let position = lsp_position_from_params(params);
            let Some(diagnostic) = summary
                .diagnostics
                .iter()
                .find(|diagnostic| {
                    requested_code.is_none_or(|code| diagnostic.code == code)
                        && position.is_none_or(|position| {
                            diagnostic.range.start <= position && position <= diagnostic.range.end
                        })
                })
                .or_else(|| {
                    summary.diagnostics.iter().find(|diagnostic| {
                        requested_code.is_none_or(|code| diagnostic.code == code)
                    })
                })
            else {
                return invalid_explain_request("no matching diagnostic was produced");
            };
            explain_omena_query(OmenaQueryExplainInputV0::Diagnostic {
                style_path: document.uri.as_str(),
                diagnostic,
            })
        }
        "transform" => {
            if !is_style_document_uri(document.uri.as_str()) {
                return invalid_explain_request("transform explanation requires a style document");
            }
            let Some(pass_id) = params
                .and_then(|value| value.get("passId"))
                .and_then(Value::as_str)
            else {
                return invalid_explain_request("missing transform passId");
            };
            let execution = execute_omena_query_transform_passes_from_source(
                document.uri.as_str(),
                document.text.as_str(),
                &[pass_id.to_string()],
            );
            let Some((decision_ordinal, decision)) = execution
                .execution
                .decisions
                .iter()
                .enumerate()
                .find(|(_, decision)| decision.compatibility_outcome().pass_id == pass_id)
            else {
                return invalid_explain_request("transform pass produced no decision");
            };
            explain_omena_query(OmenaQueryExplainInputV0::Transform {
                decision,
                decision_ordinal,
            })
        }
        "treeShake" => {
            if !is_style_document_uri(document.uri.as_str()) {
                return invalid_explain_request("tree-shake explanation requires a style document");
            }
            let Some(symbol_name) = params
                .and_then(|value| value.get("symbolName"))
                .and_then(Value::as_str)
            else {
                return invalid_explain_request("missing symbolName");
            };
            let Some(symbol_kind) = explain_symbol_kind(params) else {
                return invalid_explain_request("invalid symbolKind");
            };
            let context = params
                .and_then(|value| value.get("context"))
                .cloned()
                .and_then(|value| serde_json::from_value(value).ok())
                .unwrap_or_else(OmenaQueryTransformExecutionContextV0::default);
            explain_omena_query_tree_shake_for_style_source(
                document.uri.as_str(),
                document.text.as_str(),
                &context,
                symbol_kind,
                symbol_name,
            )
            .unwrap_or_else(|| explain_omena_query_tree_shake_unavailable(symbol_kind, symbol_name))
        }
        "precision" => {
            let Some(position) = lsp_position_from_params(params) else {
                return invalid_explain_request("precision explanation requires a position");
            };
            let Some(byte_offset) =
                byte_offset_for_parser_position(document.text.as_str(), position)
            else {
                return invalid_explain_request("position is outside the document");
            };
            let variable_name = params
                .and_then(|value| value.get("variableName"))
                .and_then(Value::as_str)
                .map(str::to_string)
                .or_else(|| identifier_at_offset(document.text.as_str(), byte_offset))
                .unwrap_or_else(|| "<unknown>".to_string());
            let reference = resolve_omena_query_source_precision_for_source(
                document.uri.as_str(),
                document.text.as_str(),
                Some(document.language_id.as_str()),
                variable_name.as_str(),
                byte_offset,
            );
            explain_omena_query(OmenaQueryExplainInputV0::Precision {
                reference: &reference,
            })
        }
        _ => return invalid_explain_request("unknown explain kind"),
    };

    serde_json::to_value(response)
        .unwrap_or_else(|_| invalid_explain_request("serialization failed"))
}

pub(crate) fn project_hover_trace_through_explain_egress(trace: &mut Value) {
    let document_uri = trace
        .get("documentUri")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let position = trace
        .get("queryPosition")
        .cloned()
        .and_then(|value| serde_json::from_value(value).ok());
    let reason = trace
        .get("reason")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let matched = trace
        .get("matched")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let candidate_count = trace
        .get("candidateCount")
        .or_else(|| trace.get("matchedCandidateCount"))
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let definition_count = trace
        .get("definitionCount")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;

    let response = explain_omena_query(OmenaQueryExplainInputV0::HoverTrace {
        document_uri,
        position,
        reason_code: reason,
        matched,
        candidate_count,
        definition_count,
    });
    if let OmenaQueryExplainFactReferenceV0::HoverResolution { reason_code, .. } =
        response.primary_fact().reference()
    {
        trace["reason"] = json!(reason_code);
    }
    if let OmenaQueryExplainFactValueV0::HoverResolution {
        matched,
        candidate_count,
        definition_count,
    } = response.primary_fact().value()
    {
        trace["matched"] = json!(matched);
        if trace.get("matchedCandidateCount").is_some() {
            trace["matchedCandidateCount"] = json!(candidate_count);
        } else {
            trace["candidateCount"] = json!(candidate_count);
        }
        trace["definitionCount"] = json!(definition_count);
    }
    record_hover_egress_projection();
}

fn explain_symbol_kind(params: Option<&Value>) -> Option<OmenaQueryExplainSymbolKindV0> {
    match params
        .and_then(|value| value.get("symbolKind"))
        .and_then(Value::as_str)
        .unwrap_or("class")
    {
        "class" => Some(OmenaQueryExplainSymbolKindV0::Class),
        "keyframes" => Some(OmenaQueryExplainSymbolKindV0::Keyframes),
        "value" => Some(OmenaQueryExplainSymbolKindV0::Value),
        "customProperty" => Some(OmenaQueryExplainSymbolKindV0::CustomProperty),
        _ => None,
    }
}

fn identifier_at_offset(source: &str, offset: usize) -> Option<String> {
    let bytes = source.as_bytes();
    let mut start = offset.min(bytes.len());
    while start > 0 && is_identifier_byte(bytes[start - 1]) {
        start -= 1;
    }
    let mut end = offset.min(bytes.len());
    while end < bytes.len() && is_identifier_byte(bytes[end]) {
        end += 1;
    }
    (start < end).then(|| source[start..end].to_string())
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'$')
}

fn invalid_explain_request(message: &'static str) -> Value {
    json!({
        "schemaVersion": "0",
        "product": "omena-lsp-server.explain-error",
        "availability": "notFound",
        "error": message,
    })
}

#[cfg(test)]
static HOVER_EGRESS_PROJECTION_COUNT: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

#[cfg(test)]
fn record_hover_egress_projection() {
    HOVER_EGRESS_PROJECTION_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

#[cfg(not(test))]
fn record_hover_egress_projection() {}

#[cfg(test)]
pub(crate) fn hover_egress_projection_count() -> usize {
    HOVER_EGRESS_PROJECTION_COUNT.load(std::sync::atomic::Ordering::Relaxed)
}
