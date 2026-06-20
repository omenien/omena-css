use crate::{LspShellState, LspStyleHoverCandidate, LspTextDocumentState};
use serde_json::{Value, json};

pub(crate) fn current_provider_tier_feedback_data(
    document: &LspTextDocumentState,
    provider: &'static str,
) -> Option<Value> {
    let feedback = document
        .optimizing_tier_feedback
        .as_ref()
        .filter(|feedback| feedback.document_version == document.version)?;
    Some(json!({
        "product": "omena-lsp-server.provider-tier-feedback",
        "source": feedback.product,
        "provider": provider,
        "policy": feedback.policy,
        "consumer": "hoverCompletionProviderRequest",
        "tier": feedback.analyzed_graph.tier,
        "feedback": "analyzedGraphV0HotStylePrewarm",
        "documentVersion": feedback.document_version,
        "nodeCount": feedback.analyzed_graph.node_count,
        "edgeCount": feedback.analyzed_graph.edge_count,
    }))
}

pub(crate) fn provider_tier_feedback_for_hover_definitions(
    state: &LspShellState,
    definitions: &[(String, LspStyleHoverCandidate)],
) -> Option<Value> {
    definitions.iter().find_map(|(uri, _)| {
        state.document(uri.as_str()).and_then(|document| {
            current_provider_tier_feedback_data(document, "textDocument/hover")
        })
    })
}

pub(crate) fn attach_provider_tier_feedback(
    response: &mut Value,
    provider_feedback: Option<&Value>,
) {
    let Some(provider_feedback) = provider_feedback else {
        return;
    };
    let Some(response_object) = response.as_object_mut() else {
        return;
    };
    let data = response_object.entry("data").or_insert_with(|| json!({}));
    if let Some(data_object) = data.as_object_mut() {
        data_object.insert(
            "providerTierFeedback".to_string(),
            provider_feedback.clone(),
        );
    }
}
