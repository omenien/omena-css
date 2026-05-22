use super::{backend, sample_input};
use crate::{
    OmenaQueryExpressionDomainFlowRuntimeV0, check_omena_query_schema_version,
    summarize_omena_query_evaluation_runtime,
    summarize_omena_query_expression_semantics_canonical_producer_signal,
    summarize_omena_query_schema_version_policy,
    summarize_omena_query_selected_query_adapter_capabilities,
    summarize_omena_query_selector_usage_canonical_producer_signal,
    summarize_omena_query_source_resolution_canonical_producer_signal,
    summarize_omena_query_source_resolution_runtime,
};

#[test]
fn declares_runtime_backed_selected_query_adapter_capabilities() {
    let summary = summarize_omena_query_selected_query_adapter_capabilities();

    assert_eq!(summary.schema_version, "0");
    assert_eq!(
        summary.product,
        "omena-query.selected-query-adapter-capabilities"
    );
    assert_eq!(summary.default_candidate_backend, "rust-selected-query");
    assert_eq!(summary.routing_status, "runtimeBacked");
    assert_eq!(
        summary.schema_version_policy.product,
        "omena-query.schema-version-policy"
    );
    assert!(
        summary
            .schema_version_policy
            .migration_policy
            .contains(&"breaking payload changes require a new numeric schemaVersion and explicit migration adapter")
    );
    assert!(summary.schema_version_checks.iter().any(|check| {
        check.requested_version.as_deref() == Some("0")
            && check.status == "current"
            && check.accepted
    }));
    assert!(summary.schema_version_checks.iter().any(|check| {
        check.requested_version.as_deref() == Some("V0")
            && check.status == "labelOnlyVersionRejected"
            && !check.accepted
    }));
    assert!(summary.schema_version_checks.iter().any(|check| {
        check.requested_version.as_deref() == Some("1")
            && check.status == "unsupportedVersion"
            && !check.accepted
    }));
    assert!(summary.schema_version_checks.iter().any(|check| {
        check.requested_version.is_none() && check.status == "missingVersion" && !check.accepted
    }));

    let unified = backend(&summary, "rust-selected-query");
    assert!(unified.is_some());
    let Some(unified) = unified else {
        return;
    };
    assert!(unified.source_resolution);
    assert!(unified.expression_semantics);
    assert!(unified.selector_usage);
    assert!(unified.style_semantic_graph);

    let source_only = backend(&summary, "rust-source-resolution");
    assert!(source_only.is_some());
    let Some(source_only) = source_only else {
        return;
    };
    assert!(source_only.source_resolution);
    assert!(!source_only.expression_semantics);
    assert!(!source_only.selector_usage);
    assert!(!source_only.style_semantic_graph);

    assert!(
        summary
            .runner_commands
            .iter()
            .any(|command| command.command == "input-omena-query-evaluation-runtime")
    );
    assert!(
        summary
            .runner_commands
            .iter()
            .any(|command| command.command == "input-omena-resolver-source-resolution-runtime")
    );
    assert!(
        summary
            .runner_commands
            .iter()
            .any(|command| command.command == "input-expression-domain-flow-analysis")
    );
    assert!(
        summary
            .runner_commands
            .iter()
            .any(|command| { command.command == "input-expression-domain-control-flow-analysis" })
    );
    assert!(
        summary.runner_commands.iter().any(|command| {
            command.command == "input-expression-domain-call-site-flow-analysis"
        })
    );
    assert!(
        summary.runner_commands.iter().any(|command| {
            command.command == "input-expression-domain-provenance-explanations"
        })
    );
    assert!(
        summary.runner_commands.iter().any(|command| {
            command.command == "input-expression-domain-reduced-product-iteration"
        })
    );
    assert!(
        summary.runner_commands.iter().any(|command| {
            command.command == "input-expression-domain-incremental-flow-analysis"
        })
    );
    assert!(
        summary
            .runner_commands
            .iter()
            .any(|command| command.command == "input-expression-domain-selector-projection")
    );
    assert!(
        summary
            .runner_commands
            .iter()
            .any(|command| command.command == "style-semantic-graph-batch")
    );
    assert!(
        summary
            .runner_commands
            .iter()
            .any(|command| command.command == "read-cascade-at-position")
    );
    assert!(
        summary
            .runner_commands
            .iter()
            .any(|command| command.command == "transform-plan")
    );
    assert!(summary.runner_commands.iter().any(|command| {
        command.command == "transform-context-from-engine-input"
            && command.output_product == "omena-query.transform-context-from-engine-input"
    }));
    assert!(
        summary
            .expression_semantics_payload_contracts
            .contains(&"valueDomainDerivation")
    );
    assert!(
        summary
            .expression_semantics_payload_contracts
            .contains(&"valueDomainProvenanceTree")
    );
    assert!(summary.adapter_readiness.contains(&"runnerCommandContract"));
    assert!(
        summary
            .adapter_readiness
            .contains(&"canonicalProducerWrapperBoundary")
    );
    assert!(
        summary
            .adapter_readiness
            .contains(&"styleSemanticGraphBridgeBoundary")
    );
    assert!(
        summary
            .adapter_readiness
            .contains(&"expressionDomainFlowAnalysisRunner")
    );
    assert!(
        summary
            .adapter_readiness
            .contains(&"expressionDomainControlFlowAnalysisRunner")
    );
    assert!(
        summary
            .adapter_readiness
            .contains(&"expressionDomainCallSiteFlowAnalysisRunner")
    );
    assert!(
        summary
            .adapter_readiness
            .contains(&"expressionDomainProvenanceExplanationRunner")
    );
    assert!(
        summary
            .adapter_readiness
            .contains(&"expressionDomainSalsaRuntime")
    );
    assert!(
        summary
            .adapter_readiness
            .contains(&"expressionDomainSelectorProjection")
    );
    assert!(
        summary
            .adapter_readiness
            .contains(&"sourceResolutionRuntimeIndex")
    );
    assert!(summary.adapter_readiness.contains(&"readCascadeAtPosition"));
    assert!(summary.adapter_readiness.contains(&"transformPlanRunner"));
    assert!(
        summary
            .adapter_readiness
            .contains(&"transformEggExecutionWitnesses")
    );
    assert!(
        summary
            .adapter_readiness
            .contains(&"semanticReachabilityTransformContext")
    );
    assert!(
        summary
            .adapter_readiness
            .contains(&"queryEvaluationRuntime")
    );
}

#[test]
fn classifies_omena_query_schema_versions_before_execution() {
    let policy = summarize_omena_query_schema_version_policy();
    assert_eq!(policy.schema_version, "0");
    assert_eq!(policy.accepted_versions, vec!["0"]);
    assert!(policy.deprecated_versions.is_empty());
    assert_eq!(
        policy.rejected_version_policy,
        "rejectUnknownVersionsBeforeExecution"
    );
    assert_eq!(
        policy.compatibility_gate,
        "rust/omena-query/adapter-capabilities"
    );

    let current = check_omena_query_schema_version(Some("0"));
    assert!(current.accepted);
    assert_eq!(current.status, "current");
    assert_eq!(current.migration_action, "executeCurrentFacade");

    let label = check_omena_query_schema_version(Some("V0"));
    assert!(!label.accepted);
    assert_eq!(label.status, "labelOnlyVersionRejected");
    assert_eq!(label.migration_action, "sendNumericSchemaVersion");

    let future = check_omena_query_schema_version(Some("1"));
    assert!(!future.accepted);
    assert_eq!(future.status, "unsupportedVersion");
    assert_eq!(future.migration_action, "rejectBeforeExecution");

    let missing = check_omena_query_schema_version(None);
    assert!(!missing.accepted);
    assert_eq!(missing.status, "missingVersion");
    assert_eq!(missing.migration_action, "rejectBeforeExecution");
}

#[test]
fn summarizes_query_evaluation_runtime_without_legacy_parser_coupling() {
    let input = sample_input();
    let mut runtime = OmenaQueryExpressionDomainFlowRuntimeV0::default();

    let first = summarize_omena_query_evaluation_runtime(&input, &mut runtime);
    assert_eq!(first.schema_version, "0");
    assert_eq!(first.product, "omena-query.evaluation-runtime");
    assert_eq!(first.input_version, "2");
    assert_eq!(
        first.selected_query_adapter_capabilities.routing_status,
        "runtimeBacked"
    );
    assert!(
        first
            .runtime_products
            .contains(&"omena-resolver.source-resolution-runtime-index")
    );
    assert!(
        first
            .runtime_products
            .contains(&"omena-query.expression-domain-incremental-flow-analysis")
    );
    assert!(
        first
            .runtime_products
            .contains(&"omena-query.style-document-summary")
    );
    assert_eq!(first.source_resolution_expression_count, 2);
    assert_eq!(first.source_resolution_unresolved_expression_count, 0);
    assert_eq!(first.expression_domain_revision, 1);
    assert_eq!(first.expression_domain_graph_count, 2);
    assert_eq!(first.expression_domain_dirty_graph_count, 2);
    assert_eq!(first.expression_domain_reused_graph_count, 0);
    assert_eq!(
        first.style_document_summary_source,
        "omena-parser.style-facts"
    );
    assert!(
        first
            .ready_surfaces
            .contains(&"selectedQueryBackendAdapter")
    );
    assert!(
        first
            .ready_surfaces
            .contains(&"sourceResolutionRuntimeIndex")
    );
    assert!(
        first
            .ready_surfaces
            .contains(&"expressionDomainSalsaRuntime")
    );
    assert!(
        first
            .ready_surfaces
            .contains(&"omenaParserStyleDocumentSummary")
    );
    assert!(
        first
            .ready_surfaces
            .contains(&"omenaParserPublicContractTypes")
    );
    assert!(
        first
            .retired_couplings
            .contains(&"engineStyleParserStyleDocumentSummary")
    );
    assert!(
        first
            .retired_couplings
            .contains(&"engineStyleParserQueryPublicTypes")
    );

    let second = summarize_omena_query_evaluation_runtime(&input, &mut runtime);
    assert_eq!(second.expression_domain_revision, 2);
    assert_eq!(second.expression_domain_dirty_graph_count, 0);
    assert_eq!(second.expression_domain_reused_graph_count, 2);
}

#[test]
fn owns_selected_query_canonical_producer_wrappers_without_changing_products() {
    let input = sample_input();

    let source = summarize_omena_query_source_resolution_canonical_producer_signal(&input);
    assert_eq!(source.schema_version, "0");
    assert_eq!(source.input_version, "2");
    assert_eq!(source.canonical_bundle.query_fragments.len(), 2);
    assert_eq!(source.evaluator_candidates.results.len(), 2);

    let expression = summarize_omena_query_expression_semantics_canonical_producer_signal(&input);
    assert_eq!(expression.schema_version, "0");
    assert_eq!(expression.input_version, "2");
    assert_eq!(expression.canonical_bundle.query_fragments.len(), 2);
    assert_eq!(expression.evaluator_candidates.results.len(), 2);
    assert_eq!(
        expression.evaluator_candidates.results[0]
            .payload
            .value_domain_derivation
            .product,
        "omena-abstract-value.reduced-class-value-derivation"
    );
    assert_eq!(
        expression.evaluator_candidates.results[0]
            .payload
            .value_domain_derivation
            .reduced_kind,
        "prefixSuffix"
    );
    assert_eq!(
        expression.evaluator_candidates.results[0]
            .payload
            .value_domain_provenance_tree
            .root
            .operation,
        "constraintDomain"
    );

    let selector = summarize_omena_query_selector_usage_canonical_producer_signal(&input);
    assert_eq!(selector.schema_version, "0");
    assert_eq!(selector.input_version, "2");
    assert_eq!(selector.canonical_bundle.query_fragments.len(), 2);
    assert_eq!(selector.evaluator_candidates.results.len(), 2);
}

#[test]
fn owns_source_resolution_runtime_index_wrapper() {
    let input = sample_input();
    let runtime_index = summarize_omena_query_source_resolution_runtime(&input);

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
            .any(|entry| entry.expression_id == "expr-1" && entry.selector_names == ["btn-active"])
    );
}
