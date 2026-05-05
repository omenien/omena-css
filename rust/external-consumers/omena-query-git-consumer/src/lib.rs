use engine_input_producers::{
    ClassExpressionInputV2, EngineInputV2, PositionV2, RangeV2, SourceAnalysisInputV2,
    SourceDocumentV2, StringTypeFactsV2, StyleAnalysisInputV2, StyleDocumentV2, StyleSelectorV2,
    TypeFactEntryV2,
};
use omena_query::{
    OmenaQueryBoundarySummaryV0, OmenaQueryStyleSemanticGraphBatchOutputV0,
    summarize_omena_query_boundary, summarize_omena_query_style_semantic_graph_batch_from_sources,
};

pub fn consume_query_boundary() -> OmenaQueryBoundarySummaryV0 {
    summarize_omena_query_boundary(&sample_input())
}

pub fn consume_style_graph_batch() -> OmenaQueryStyleSemanticGraphBatchOutputV0 {
    summarize_omena_query_style_semantic_graph_batch_from_sources(
        [
            ("/tmp/App.module.scss", ".btn-active { color: red; }"),
            ("/tmp/Card.module.scss", ".card-header { color: blue; }"),
        ],
        &sample_input(),
    )
}

fn sample_input() -> EngineInputV2 {
    EngineInputV2 {
        version: "2".to_string(),
        sources: vec![SourceAnalysisInputV2 {
            document: SourceDocumentV2 {
                class_expressions: vec![
                    ClassExpressionInputV2 {
                        id: "expr-1".to_string(),
                        kind: "symbolRef".to_string(),
                        scss_module_path: "/tmp/App.module.scss".to_string(),
                        range: range(4, 12, 4, 16),
                        class_name: None,
                        root_binding_decl_id: Some("decl-1".to_string()),
                        access_path: None,
                    },
                    ClassExpressionInputV2 {
                        id: "expr-2".to_string(),
                        kind: "styleAccess".to_string(),
                        scss_module_path: "/tmp/Card.module.scss".to_string(),
                        range: range(6, 9, 6, 20),
                        class_name: Some("card-header".to_string()),
                        root_binding_decl_id: None,
                        access_path: Some(vec!["card".to_string(), "header".to_string()]),
                    },
                ],
            },
        }],
        styles: vec![
            StyleAnalysisInputV2 {
                file_path: "/tmp/App.module.scss".to_string(),
                document: StyleDocumentV2 {
                    selectors: vec![StyleSelectorV2 {
                        name: "btn-active".to_string(),
                        view_kind: "canonical".to_string(),
                        canonical_name: Some("btn-active".to_string()),
                        range: range(1, 1, 1, 12),
                        nested_safety: Some("safe".to_string()),
                        composes: None,
                        bem_suffix: None,
                    }],
                },
            },
            StyleAnalysisInputV2 {
                file_path: "/tmp/Card.module.scss".to_string(),
                document: StyleDocumentV2 {
                    selectors: vec![StyleSelectorV2 {
                        name: "card-header".to_string(),
                        view_kind: "canonical".to_string(),
                        canonical_name: Some("card-header".to_string()),
                        range: range(3, 1, 3, 13),
                        nested_safety: Some("unsafe".to_string()),
                        composes: None,
                        bem_suffix: None,
                    }],
                },
            },
        ],
        type_facts: vec![
            TypeFactEntryV2 {
                file_path: "/tmp/App.tsx".to_string(),
                expression_id: "expr-1".to_string(),
                facts: StringTypeFactsV2 {
                    kind: "constrained".to_string(),
                    constraint_kind: Some("prefixSuffix".to_string()),
                    values: None,
                    prefix: Some("btn-".to_string()),
                    suffix: Some("-active".to_string()),
                    min_len: Some(10),
                    max_len: None,
                    char_must: None,
                    char_may: None,
                    may_include_other_chars: None,
                },
            },
            TypeFactEntryV2 {
                file_path: "/tmp/Card.tsx".to_string(),
                expression_id: "expr-2".to_string(),
                facts: StringTypeFactsV2 {
                    kind: "finiteSet".to_string(),
                    constraint_kind: None,
                    values: Some(vec!["card-header".to_string(), "card-body".to_string()]),
                    prefix: None,
                    suffix: None,
                    min_len: None,
                    max_len: None,
                    char_must: None,
                    char_may: None,
                    may_include_other_chars: None,
                },
            },
        ],
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
    use super::{consume_query_boundary, consume_style_graph_batch, sample_input};
    use omena_query::{
        summarize_omena_query_expression_domain_flow_analysis,
        summarize_omena_query_expression_domain_selector_projection,
        summarize_omena_query_expression_semantics_canonical_producer_signal,
        summarize_omena_query_fragment_bundle,
        summarize_omena_query_selected_query_adapter_capabilities,
        summarize_omena_query_source_resolution_runtime,
    };
    use serde_json::json;

    #[test]
    fn consumes_remote_query_boundary_via_git_dependency() {
        let boundary = consume_query_boundary();

        assert_eq!(boundary.product, "omena-query.boundary");
        assert_eq!(boundary.query_engine_name, "omena-query");
        assert_eq!(
            boundary.abstract_value_domain.product,
            "omena-abstract-value.domain"
        );
        assert_eq!(boundary.total_query_count, 6);
        assert!(boundary.ready_surfaces.contains(&"queryBoundarySummary"));
        assert!(
            boundary
                .ready_surfaces
                .contains(&"expressionDomainFlowAnalysisBoundary")
        );
        assert!(
            boundary
                .ready_surfaces
                .contains(&"sourceResolutionRuntimeIndex")
        );
        assert!(
            boundary
                .delegated_fragment_products
                .contains(&"omena-resolver.source-resolution-runtime-index")
        );
    }

    #[test]
    fn consumes_remote_fragment_bundle_contract() {
        let bundle = summarize_omena_query_fragment_bundle(&sample_input());

        assert_eq!(bundle.product, "omena-query.fragment-bundle");
        assert_eq!(bundle.expression_semantics.fragments.len(), 2);
        assert_eq!(bundle.source_resolution.fragments.len(), 2);
        assert_eq!(bundle.selector_usage.fragments.len(), 2);
    }

    #[test]
    fn consumes_remote_selected_query_capability_contract() {
        let capabilities = summarize_omena_query_selected_query_adapter_capabilities();

        assert_eq!(
            capabilities.product,
            "omena-query.selected-query-adapter-capabilities"
        );
        assert_eq!(
            capabilities.default_candidate_backend,
            "rust-selected-query"
        );
        assert_eq!(capabilities.routing_status, "declaredOnly");
        assert!(
            capabilities
                .runner_commands
                .iter()
                .any(|command| command.command == "input-omena-resolver-source-resolution-runtime")
        );
        assert!(
            capabilities
                .runner_commands
                .iter()
                .any(|command| command.command == "input-expression-domain-flow-analysis")
        );
        assert!(
            capabilities.runner_commands.iter().any(|command| {
                command.command == "input-expression-domain-selector-projection"
            })
        );
        assert!(
            capabilities
                .runner_commands
                .iter()
                .any(|command| command.command == "style-semantic-graph-batch")
        );
        assert!(
            capabilities
                .expression_semantics_payload_contracts
                .contains(&"valueDomainDerivation")
        );
        assert!(
            capabilities
                .adapter_readiness
                .contains(&"sourceResolutionRuntimeIndex")
        );
        assert!(
            capabilities
                .adapter_readiness
                .contains(&"expressionDomainSelectorProjection")
        );
    }

    #[test]
    fn consumes_remote_source_resolution_runtime_contract() {
        let runtime_index = summarize_omena_query_source_resolution_runtime(&sample_input());

        assert_eq!(
            runtime_index.product,
            "omena-resolver.source-resolution-runtime-index"
        );
        assert_eq!(runtime_index.expression_count, 2);
        assert_eq!(runtime_index.resolved_expression_count, 2);
        assert_eq!(runtime_index.unresolved_expression_count, 0);
        assert!(
            runtime_index
                .entries
                .iter()
                .any(|entry| entry.expression_id == "expr-1"
                    && entry.selector_names == ["btn-active"])
        );
    }

    #[test]
    fn consumes_remote_expression_domain_flow_analysis_contract() {
        let summary = summarize_omena_query_expression_domain_flow_analysis(&sample_input());

        assert_eq!(
            summary.product,
            "engine-input-producers.expression-domain-flow-analysis"
        );
        assert_eq!(summary.analyses.len(), 2);
        assert!(
            summary
                .analyses
                .iter()
                .all(|entry| entry.analysis.converged)
        );
    }

    #[test]
    fn consumes_remote_expression_domain_selector_projection_contract() {
        let summary = summarize_omena_query_expression_domain_selector_projection(&sample_input());

        assert_eq!(
            summary.product,
            "omena-query.expression-domain-selector-projection"
        );
        assert_eq!(summary.projection_count, 2);
        assert!(summary.projections.iter().any(|projection| {
            projection.node_id == "expr-1"
                && projection.target_style_paths == ["/tmp/App.module.scss"]
                && projection.selector_names == ["btn-active"]
        }));
    }

    #[test]
    fn consumes_remote_expression_semantics_derivation_contract() {
        let signal =
            summarize_omena_query_expression_semantics_canonical_producer_signal(&sample_input());
        let payload = &signal.evaluator_candidates.results[0].payload;

        assert_eq!(
            payload.value_domain_derivation.product,
            "omena-abstract-value.reduced-class-value-derivation"
        );
        assert_eq!(payload.value_domain_derivation.reduced_kind, "prefixSuffix");
    }

    #[test]
    fn serializes_remote_style_graph_batch_for_downstream_consumers() -> Result<(), String> {
        let batch = consume_style_graph_batch();
        let value = serde_json::to_value(&batch).map_err(|error| error.to_string())?;

        assert_eq!(
            value["product"],
            json!("omena-semantic.style-semantic-graph-batch")
        );
        assert_eq!(
            value["graphs"][0]["stylePath"],
            json!("/tmp/App.module.scss")
        );
        assert_eq!(
            value["graphs"][0]["graph"]["product"],
            json!("omena-semantic.style-semantic-graph")
        );
        assert_eq!(
            value["graphs"][1]["graph"]["selectorReferenceEngine"]["stylePath"],
            json!("/tmp/Card.module.scss")
        );
        Ok(())
    }
}
