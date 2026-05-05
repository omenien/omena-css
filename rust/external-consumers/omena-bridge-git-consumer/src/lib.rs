use engine_input_producers::{
    ClassExpressionInputV2, EngineInputV2, PositionV2, RangeV2, SourceAnalysisInputV2,
    SourceDocumentV2, StringTypeFactsV2, StyleAnalysisInputV2, StyleDocumentV2, StyleSelectorV2,
    TypeFactEntryV2,
};
use omena_bridge::{
    StyleSemanticGraphSummaryV0, summarize_omena_bridge_boundary,
    summarize_omena_bridge_style_semantic_graph_from_source,
};

pub fn consume_bridge_boundary_product() -> &'static str {
    summarize_omena_bridge_boundary().product
}

pub fn consume_style_semantic_graph() -> Option<StyleSemanticGraphSummaryV0> {
    summarize_omena_bridge_style_semantic_graph_from_source(
        "/tmp/Component.module.scss",
        ".button { color: red; }",
        &sample_input(),
    )
}

fn sample_input() -> EngineInputV2 {
    EngineInputV2 {
        version: "2".to_string(),
        sources: vec![SourceAnalysisInputV2 {
            document: SourceDocumentV2 {
                class_expressions: vec![ClassExpressionInputV2 {
                    id: "expr-button".to_string(),
                    kind: "literal".to_string(),
                    scss_module_path: "/tmp/Component.module.scss".to_string(),
                    range: range(4, 12, 4, 18),
                    class_name: Some("button".to_string()),
                    root_binding_decl_id: None,
                    access_path: None,
                }],
            },
        }],
        styles: vec![StyleAnalysisInputV2 {
            file_path: "/tmp/Component.module.scss".to_string(),
            document: StyleDocumentV2 {
                selectors: vec![StyleSelectorV2 {
                    name: "button".to_string(),
                    view_kind: "canonical".to_string(),
                    canonical_name: Some("button".to_string()),
                    range: range(0, 1, 0, 7),
                    nested_safety: Some("flat".to_string()),
                    composes: None,
                    bem_suffix: None,
                }],
            },
        }],
        type_facts: vec![TypeFactEntryV2 {
            file_path: "/tmp/Component.tsx".to_string(),
            expression_id: "expr-button".to_string(),
            facts: StringTypeFactsV2 {
                kind: "exact".to_string(),
                constraint_kind: None,
                values: Some(vec!["button".to_string()]),
                prefix: None,
                suffix: None,
                min_len: None,
                max_len: None,
                char_must: None,
                char_may: None,
                may_include_other_chars: None,
            },
        }],
    }
}

fn range(
    start_line: usize,
    start_character: usize,
    end_line: usize,
    end_character: usize,
) -> RangeV2 {
    RangeV2 {
        start: PositionV2 {
            line: start_line,
            character: start_character,
        },
        end: PositionV2 {
            line: end_line,
            character: end_character,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{consume_bridge_boundary_product, consume_style_semantic_graph, sample_input};
    use omena_bridge::{
        summarize_omena_bridge_binder_plugin_boundary, summarize_omena_bridge_boundary,
        summarize_omena_bridge_selector_reference_engine,
        summarize_omena_bridge_source_input_evidence,
    };
    use serde_json::json;

    #[test]
    fn consumes_remote_bridge_boundary_via_git_dependency() {
        let boundary = summarize_omena_bridge_boundary();
        let binder_boundary = summarize_omena_bridge_binder_plugin_boundary();

        assert_eq!(
            consume_bridge_boundary_product(),
            "omena-bridge.cme-semantic-bridge"
        );
        assert_eq!(
            boundary.graph_product,
            "omena-semantic.style-semantic-graph"
        );
        assert!(
            boundary
                .bridge_owned_surfaces
                .contains(&"styleSemanticGraphFromSource")
        );
        assert_eq!(
            boundary.binder_plugin_product,
            "omena-bridge.binder-plugin-boundary"
        );
        assert_eq!(binder_boundary.contract_name, "BinderPluginV0");
        assert_eq!(
            binder_boundary.default_plugin.id,
            "css-modules-classnames-bind"
        );
        assert!(!binder_boundary.external_plugin_abi_stable);
        assert!(boundary.cme_coupled_surfaces.contains(&"EngineInputV2"));
    }

    #[test]
    fn consumes_remote_bridge_graph_from_source() -> Result<(), String> {
        let graph = consume_style_semantic_graph()
            .ok_or_else(|| "expected bridge graph from source".to_string())?;

        assert_eq!(graph.product, "omena-semantic.style-semantic-graph");
        assert_eq!(graph.selector_reference_engine.selector_count, 1);
        assert_eq!(graph.selector_reference_engine.referenced_selector_count, 1);
        assert_eq!(
            graph.source_input_evidence.reference_site_identity.status,
            "ready"
        );
        assert_eq!(
            graph
                .source_input_evidence
                .value_domain_explanation
                .derivation_count,
            1
        );
        assert!(graph.promotion_evidence.blocking_gaps.is_empty());
        Ok(())
    }

    #[test]
    fn consumes_remote_selector_reference_engine_contract() {
        let references = summarize_omena_bridge_selector_reference_engine(
            &sample_input(),
            Some("/tmp/Component.module.scss"),
        );

        assert_eq!(references.product, "omena-semantic.selector-references");
        assert_eq!(
            references.style_path,
            Some("/tmp/Component.module.scss".to_string())
        );
        assert_eq!(references.total_reference_sites, 1);
        assert_eq!(references.selectors[0].canonical_id, "selector:button");
    }

    #[test]
    fn serializes_remote_source_evidence_for_downstream_consumers() -> Result<(), String> {
        let evidence = summarize_omena_bridge_source_input_evidence(&sample_input());
        let value = serde_json::to_value(&evidence).map_err(|error| error.to_string())?;

        assert_eq!(value["schemaVersion"], json!("0"));
        assert_eq!(
            value["product"],
            json!("omena-semantic.source-input-evidence")
        );
        assert_eq!(value["referenceSiteIdentity"]["status"], json!("ready"));
        assert_eq!(
            value["valueDomainExplanation"]["derivationOperationCounts"]["baseFromFacts"],
            json!(1)
        );
        Ok(())
    }
}
