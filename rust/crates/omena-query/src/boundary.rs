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
            "queryBoundarySummary",
        ],
        cme_coupled_surfaces: vec!["EngineInputV2", "producerQueryFragments"],
        next_decoupling_targets: vec![
            "queryEvaluationRuntime",
            "selectedQueryBackendAdapter",
            "engineStyleParserStyleDocumentSummary",
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
                output_product: "omena-query.omena-parser-style-facts",
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
        ],
        routing_status: "declaredOnly",
    }
}
