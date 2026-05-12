use super::*;

pub fn summarize_omena_query_boundary(input: &EngineInputV2) -> OmenaQueryBoundarySummaryV0 {
    let fragment_bundle = summarize_omena_query_fragment_bundle(input);
    let expression_semantics_query_count = fragment_bundle.expression_semantics.fragments.len();
    let source_resolution_query_count = fragment_bundle.source_resolution.fragments.len();
    let selector_usage_query_count = fragment_bundle.selector_usage.fragments.len();

    OmenaQueryBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-query.boundary",
        query_engine_name: "omena-query",
        input_version: input.version.clone(),
        abstract_value_domain: summarize_omena_abstract_value_domain(),
        selected_query_adapter_capabilities:
            summarize_omena_query_selected_query_adapter_capabilities(),
        delegated_fragment_products: vec![
            "engine-input-producers.expression-semantics-query-fragments",
            "engine-input-producers.source-resolution-query-fragments",
            "omena-resolver.boundary",
            "omena-resolver.source-resolution-runtime-index",
            "engine-input-producers.selector-usage-query-fragments",
            "engine-input-producers.expression-domain-flow-analysis",
            "engine-input-producers.expression-domain-control-flow-analysis",
            "omena-query.expression-domain-incremental-flow-analysis",
            "omena-query.expression-domain-selector-projection",
            "omena-parser.style-facts",
            "omena-transform-bundle.source",
            "omena-transform-target.plan",
            "omena-transform-egg.plan",
            "omena-transform-print.artifact",
            "omena-transform-passes.plan",
            "omena-transform-passes.execution",
            "omena-query.transform-execute",
            "omena-query.transform-context-from-engine-input",
            "omena-query.consumer-check-style-source",
            "omena-query.consumer-build-style-source",
            "omena-query.evaluation-runtime",
        ],
        expression_semantics_query_count,
        source_resolution_query_count,
        selector_usage_query_count,
        total_query_count: expression_semantics_query_count
            + source_resolution_query_count
            + selector_usage_query_count,
        ready_surfaces: vec![
            "queryFragmentBundle",
            "abstractValueProjectionContract",
            "sourceResolutionResolverBoundary",
            "sourceResolutionRuntimeIndex",
            "expressionDomainFlowAnalysisBoundary",
            "expressionDomainControlFlowAnalysisBoundary",
            "expressionDomainSalsaRuntime",
            "expressionDomainSelectorProjection",
            "styleHoverRenderParts",
            "styleMissingCustomPropertyDiagnostics",
            "readCascadeAtPosition",
            "sourceMissingSelectorDiagnostics",
            "sourceProviderCandidateResolution",
            "selectorRenameEditPlanning",
            "sassSymbolResolutionPrimitives",
            "sassModuleSourceSelection",
            "omenaParserStyleFactExtraction",
            "transformPlanFacade",
            "transformExecutionRuntime",
            "transformExecutionRunner",
            "semanticReachabilityTransformContext",
            "consumerCheckFacade",
            "consumerBuildFacade",
            "consumerTransformPassListFacade",
            "queryBoundarySummary",
            "selectedQueryBackendAdapter",
            "queryEvaluationRuntime",
            "omenaParserStyleDocumentSummary",
            "omenaParserPublicContractTypes",
        ],
        cme_coupled_surfaces: vec!["EngineInputV2", "producerQueryFragments"],
        next_decoupling_targets: Vec::new(),
    }
}

pub fn summarize_omena_query_evaluation_runtime(
    input: &EngineInputV2,
    runtime: &mut OmenaQueryExpressionDomainFlowRuntimeV0,
) -> OmenaQueryEvaluationRuntimeSummaryV0 {
    let selected_query_adapter_capabilities =
        summarize_omena_query_selected_query_adapter_capabilities();
    let source_resolution_runtime = summarize_omena_query_source_resolution_runtime(input);
    let expression_domain_runtime =
        summarize_omena_query_expression_domain_incremental_flow_analysis(input, runtime);

    OmenaQueryEvaluationRuntimeSummaryV0 {
        schema_version: "0",
        product: "omena-query.evaluation-runtime",
        input_version: input.version.clone(),
        selected_query_adapter_capabilities,
        runtime_products: vec![
            source_resolution_runtime.product,
            expression_domain_runtime.product,
            "omena-query.style-document-summary",
        ],
        source_resolution_expression_count: source_resolution_runtime.expression_count,
        source_resolution_unresolved_expression_count: source_resolution_runtime
            .unresolved_expression_count,
        expression_domain_revision: expression_domain_runtime.revision,
        expression_domain_graph_count: expression_domain_runtime.graph_count,
        expression_domain_dirty_graph_count: expression_domain_runtime.dirty_graph_count,
        expression_domain_reused_graph_count: expression_domain_runtime.reused_graph_count,
        style_document_summary_source: "omena-parser.style-facts",
        ready_surfaces: vec![
            "selectedQueryBackendAdapter",
            "sourceResolutionRuntimeIndex",
            "expressionDomainSalsaRuntime",
            "expressionDomainSelectorProjection",
            "omenaParserStyleDocumentSummary",
            "omenaParserPublicContractTypes",
        ],
        retired_couplings: vec![
            "engineStyleParserStyleDocumentSummary",
            "engineStyleParserQueryPublicTypes",
        ],
    }
}

pub fn summarize_omena_query_selected_query_adapter_capabilities()
-> SelectedQueryAdapterCapabilitiesV0 {
    SelectedQueryAdapterCapabilitiesV0 {
        schema_version: "0",
        product: "omena-query.selected-query-adapter-capabilities",
        default_candidate_backend: "rust-selected-query",
        backend_kinds: vec![
            SelectedQueryBackendCapabilityV0 {
                backend_kind: "typescript-current",
                source_resolution: false,
                expression_semantics: false,
                selector_usage: false,
                style_semantic_graph: false,
            },
            SelectedQueryBackendCapabilityV0 {
                backend_kind: "rust-source-resolution",
                source_resolution: true,
                expression_semantics: false,
                selector_usage: false,
                style_semantic_graph: false,
            },
            SelectedQueryBackendCapabilityV0 {
                backend_kind: "rust-expression-semantics",
                source_resolution: false,
                expression_semantics: true,
                selector_usage: false,
                style_semantic_graph: false,
            },
            SelectedQueryBackendCapabilityV0 {
                backend_kind: "rust-selector-usage",
                source_resolution: false,
                expression_semantics: false,
                selector_usage: true,
                style_semantic_graph: false,
            },
            SelectedQueryBackendCapabilityV0 {
                backend_kind: "rust-selected-query",
                source_resolution: true,
                expression_semantics: true,
                selector_usage: true,
                style_semantic_graph: true,
            },
        ],
        runner_commands: vec![
            SelectedQueryRunnerCommandV0 {
                surface: "queryEvaluationRuntime",
                command: "input-omena-query-evaluation-runtime",
                input_contract: "EngineInputV2 + OmenaQueryExpressionDomainFlowRuntimeV0",
                output_product: "omena-query.evaluation-runtime",
            },
            SelectedQueryRunnerCommandV0 {
                surface: "sourceResolution",
                command: "input-source-resolution-canonical-producer",
                input_contract: "EngineInputV2",
                output_product: "engine-input-producers.source-resolution-canonical-producer",
            },
            SelectedQueryRunnerCommandV0 {
                surface: "sourceResolutionRuntime",
                command: "input-omena-resolver-source-resolution-runtime",
                input_contract: "EngineInputV2",
                output_product: "omena-resolver.source-resolution-runtime-index",
            },
            SelectedQueryRunnerCommandV0 {
                surface: "expressionSemantics",
                command: "input-expression-semantics-canonical-producer",
                input_contract: "EngineInputV2",
                output_product: "engine-input-producers.expression-semantics-canonical-producer",
            },
            SelectedQueryRunnerCommandV0 {
                surface: "expressionDomainFlowAnalysis",
                command: "input-expression-domain-flow-analysis",
                input_contract: "EngineInputV2",
                output_product: "engine-input-producers.expression-domain-flow-analysis",
            },
            SelectedQueryRunnerCommandV0 {
                surface: "expressionDomainControlFlowAnalysis",
                command: "input-expression-domain-control-flow-analysis",
                input_contract: "EngineInputV2",
                output_product: "engine-input-producers.expression-domain-control-flow-analysis",
            },
            SelectedQueryRunnerCommandV0 {
                surface: "expressionDomainIncrementalFlowAnalysis",
                command: "input-expression-domain-incremental-flow-analysis",
                input_contract: "EngineInputV2 + OmenaQueryExpressionDomainFlowRuntimeV0",
                output_product: "omena-query.expression-domain-incremental-flow-analysis",
            },
            SelectedQueryRunnerCommandV0 {
                surface: "expressionDomainSelectorProjection",
                command: "input-expression-domain-selector-projection",
                input_contract: "EngineInputV2",
                output_product: "omena-query.expression-domain-selector-projection",
            },
            SelectedQueryRunnerCommandV0 {
                surface: "selectorUsage",
                command: "input-selector-usage-canonical-producer",
                input_contract: "EngineInputV2",
                output_product: "engine-input-producers.selector-usage-canonical-producer",
            },
            SelectedQueryRunnerCommandV0 {
                surface: "omenaParserStyleFacts",
                command: "omena-parser-style-facts",
                input_contract: "OmenaParserStyleFactsInputV0",
                output_product: "omena-parser.style-facts",
            },
            SelectedQueryRunnerCommandV0 {
                surface: "styleSemanticGraph",
                command: "style-semantic-graph",
                input_contract: "StyleSemanticGraphInputV0",
                output_product: "omena-semantic.style-semantic-graph",
            },
            SelectedQueryRunnerCommandV0 {
                surface: "readCascadeAtPosition",
                command: "read-cascade-at-position",
                input_contract: "ReadCascadeAtPositionInputV0",
                output_product: "omena-query.read-cascade-at-position",
            },
            SelectedQueryRunnerCommandV0 {
                surface: "styleSemanticGraphBatch",
                command: "style-semantic-graph-batch",
                input_contract: "StyleSemanticGraphBatchInputV0",
                output_product: "omena-semantic.style-semantic-graph-batch",
            },
            SelectedQueryRunnerCommandV0 {
                surface: "transformPlan",
                command: "transform-plan",
                input_contract: "TransformPlanInputV0",
                output_product: "omena-query.transform-plan",
            },
            SelectedQueryRunnerCommandV0 {
                surface: "transformContext",
                command: "transform-context",
                input_contract: "TransformContextInputV0",
                output_product: "omena-query.transform-context",
            },
            SelectedQueryRunnerCommandV0 {
                surface: "semanticReachabilityTransformContext",
                command: "transform-context-from-engine-input",
                input_contract: "EngineInputV2 + targetStylePath + closedStyleWorld",
                output_product: "omena-query.transform-context-from-engine-input",
            },
            SelectedQueryRunnerCommandV0 {
                surface: "transformExecute",
                command: "transform-execute",
                input_contract: "TransformExecuteInputV0",
                output_product: "omena-query.transform-execute",
            },
            SelectedQueryRunnerCommandV0 {
                surface: "consumerCheckStyleSource",
                command: "consumer-check-style-source",
                input_contract: "ConsumerStyleSourceInputV0",
                output_product: "omena-query.consumer-check-style-source",
            },
            SelectedQueryRunnerCommandV0 {
                surface: "consumerBuildStyleSource",
                command: "consumer-build-style-source",
                input_contract: "ConsumerStyleSourceBuildInputV0",
                output_product: "omena-query.consumer-build-style-source",
            },
            SelectedQueryRunnerCommandV0 {
                surface: "consumerBuildStyleSources",
                command: "consumer-build-style-sources",
                input_contract: "ConsumerStyleSourcesBuildInputV0",
                output_product: "omena-query.consumer-build-style-source",
            },
            SelectedQueryRunnerCommandV0 {
                surface: "consumerTransformPassList",
                command: "consumer-transform-pass-list",
                input_contract: "None",
                output_product: "omena-query.transform-pass-list",
            },
        ],
        expression_semantics_payload_contracts: vec![
            "valueDomainKind",
            "valueDomainDerivation",
            "valueDomainProvenanceTree",
        ],
        required_input_contracts: vec![
            "EngineInputV2",
            "StyleSemanticGraphInputV0",
            "ReadCascadeAtPositionInputV0",
            "StyleSemanticGraphBatchInputV0",
            "OmenaParserStyleFactsInputV0",
            "TransformPlanInputV0",
            "TransformContextInputV0",
            "TransformContextFromEngineInputV0",
            "TransformExecuteInputV0",
            "ConsumerStyleSourceInputV0",
            "ConsumerStyleSourceBuildInputV0",
            "ConsumerStyleSourcesBuildInputV0",
        ],
        adapter_readiness: vec![
            "backendCapabilityMatrix",
            "canonicalProducerWrapperBoundary",
            "styleSemanticGraphBridgeBoundary",
            "runnerCommandContract",
            "fragmentBundleBoundary",
            "sourceResolutionRuntimeIndex",
            "expressionSemanticsDerivationPayload",
            "expressionDomainFlowAnalysisRunner",
            "expressionDomainControlFlowAnalysisRunner",
            "expressionDomainSalsaRuntime",
            "expressionDomainSelectorProjection",
            "omenaParserStyleFactExtraction",
            "readCascadeAtPosition",
            "transformPlanRunner",
            "transformContextProducer",
            "semanticReachabilityTransformContext",
            "transformExecutionRunner",
            "consumerCheckFacade",
            "consumerBuildFacade",
            "consumerTransformPassListFacade",
            "queryEvaluationRuntime",
        ],
        routing_status: "runtimeBacked",
    }
}
