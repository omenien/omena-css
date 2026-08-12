//! Core query runtime primitives below the public `omena-query` facade.
//!
//! This crate owns producer-fragment summaries and expression-domain runtime
//! state. `omena-query` re-exports these surfaces, but no longer needs to depend
//! directly on each lower-level producer crate for this part of the dataflow.

#[cfg(feature = "test-support")]
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};

pub use engine_input_producers::{
    ClassExpressionInputV2, EngineInputV2, ExpressionDomainCallSiteFlowAnalysisV0,
    ExpressionDomainControlFlowAnalysisV0, ExpressionDomainFlowAnalysisV0,
    ExpressionDomainProvenanceExplanationsV0, ExpressionDomainReducedProductIterationV0,
    ExpressionSemanticsCanonicalProducerSignalV0, ExpressionSemanticsQueryFragmentsV0, PositionV2,
    RangeV2, SelectorUsageCanonicalProducerSignalV0, SelectorUsageQueryFragmentsV0,
    SourceAnalysisInputV2, SourceDocumentV2, SourceResolutionCanonicalProducerSignalV0,
    SourceResolutionQueryFragmentsV0, StringTypeFactsV2, StyleAnalysisInputV2, StyleDocumentV2,
    StyleSelectorV2, TypeFactEntryV2,
};
use engine_input_producers::{
    collect_expression_domain_flow_graphs,
    summarize_expression_domain_call_site_flow_analysis_input,
    summarize_expression_domain_control_flow_analysis_input,
    summarize_expression_domain_flow_analysis_input,
    summarize_expression_domain_provenance_explanations_input,
    summarize_expression_domain_reduced_product_iteration_input,
    summarize_expression_semantics_canonical_producer_signal_input,
    summarize_expression_semantics_query_fragments_input,
    summarize_selector_usage_canonical_producer_signal_input,
    summarize_selector_usage_query_fragments_input,
};
use omena_abstract_value::{
    AbstractClassValueProvenanceV0, analyze_class_value_flow_incremental_with_database,
    project_abstract_value_selectors, summarize_omena_abstract_value_domain,
    summarize_reduced_class_value_product,
};
pub use omena_abstract_value::{
    AbstractClassValueV0, AbstractPropertyValueCandidateV0, AbstractPropertyValueNarrowingV0,
    AbstractPropertyValueV0, AbstractValueDomainSummaryV0, CascadeContextV0,
    CascadeValueFamilyMemberV0, ClassBoundaryEffectV0, ClassValueFlowAnalysisV0,
    ClassValueFlowIncrementalAnalysisV0, CssValueValidationClassV0, ExternalStringTypeFactsV0,
    FactPrecision, FirstWitnessErrorV0, GuardAtomV0, GuardedTokenInputV0, GuardedTokenLanguageV0,
    GuardedTokenMapInputV0, GuardedTokenMapV0, GuardedTokenObserverV0, Lin01ProvenanceSemiringV0,
    LinearProvenancePathV0, LinearProvenanceV0, NaturalCountProvenanceSemiringV0,
    OmenaAbstractValueCoverageDirectionV0, OmenaAbstractValuePrecisionBasisV0,
    OmenaAbstractValuePrecisionWitnessV0, PolynomialProvenanceProjectionV0,
    PolynomialProvenanceTermV0, PolynomialProvenanceV0, PolynomialProvenanceVariableV0,
    ProvenanceSemiringLawReportV0, ReducedClassValueProductIterationV0, ReducedClassValueProductV0,
    SelectorProjectionCertaintyV0, TokenObserverProjectionV0, abstract_class_value_from_facts,
    abstract_class_value_kind, derive_context_indexed_cascade_restriction_maps_v0,
    fact_precision_from_class_value, fact_precision_from_class_value_with_witness,
    iterate_reduced_class_value_product_constraints, join_abstract_class_values,
    narrow_abstract_property_value_for_cascade_branch,
    narrow_abstract_property_value_for_pseudo_state, prefix_suffix_class_value,
    summarize_context_indexed_cascade_value_family_v0,
    summarize_polynomial_provenance_from_linear_v0, top_class_value,
    validate_registered_property_value_v0, verify_provenance_semiring_laws_on_fixtures,
};
#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "use the context-indexed cascade value family adapters; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub use omena_abstract_value::{
    derive_cascade_restriction_maps_v0, summarize_cascade_value_family_v0,
};
pub use omena_incremental::{
    IncrementalEditDistancePriorityInputV0, IncrementalGraphInputV0,
    IncrementalInvalidationPriorityPlanV0, IncrementalNodeInputV0, IncrementalRevisionV0,
    OmenaIncrementalDatabaseV0, OmenaSalsaDatabaseV0, OmenaWorkspaceSnapshotIdV0,
    snapshot_from_graph_input,
};
pub use omena_refinement::{
    CascadeDimensionalRefinementBridgeV0, RefinementPropertyPredicateV0,
    summarize_cascade_dimensional_refinement_bridge_v0,
};
pub use omena_resolver::OmenaResolverSourceResolutionRuntimeIndexV0;
use omena_resolver::{
    summarize_omena_resolver_canonical_producer_signal, summarize_omena_resolver_query_fragments,
    summarize_omena_resolver_source_resolution_runtime,
};
pub use omena_value_lattice::{
    canonicalize_css_value, split_top_level_value_arguments,
    split_top_level_whitespace_value_components,
};
use serde::{Deserialize, Serialize};

pub const OMENA_QUERY_CURRENT_SCHEMA_VERSION: &str = "0";
pub const OMENA_QUERY_CURRENT_SCHEMA_VERSION_LABEL: &str = "V0";

#[cfg(feature = "test-support")]
thread_local! {
    static SELECTOR_PROJECTION_EVALUATION_COUNT: Cell<usize> = const { Cell::new(0) };
}

#[cfg(feature = "test-support")]
pub fn reset_selector_projection_evaluation_count_for_test() {
    SELECTOR_PROJECTION_EVALUATION_COUNT.with(|counter| counter.set(0));
}

#[cfg(feature = "test-support")]
pub fn selector_projection_evaluation_count_for_test() -> usize {
    SELECTOR_PROJECTION_EVALUATION_COUNT.with(Cell::get)
}

#[cfg(feature = "test-support")]
fn record_selector_projection_evaluation_for_test() {
    SELECTOR_PROJECTION_EVALUATION_COUNT.with(|counter| counter.set(counter.get() + 1));
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryAnalysisPrecisionV0 {
    pub product: String,
    pub value_domain: String,
    pub flow_sensitivity: String,
    pub context_sensitivity: String,
    pub revision_axis: String,
}

const OMENA_QUERY_ANALYSIS_FACT_PRECISION_BY_VALUE_DOMAIN: &[(&str, FactPrecision)] = &[
    ("cascadeAtPosition", FactPrecision::Exact),
    ("styleModuleResolution", FactPrecision::Exact),
    ("classValueResolution", FactPrecision::Conservative),
    ("classValueUniverse", FactPrecision::Conservative),
    ("classValueFlow", FactPrecision::Heuristic),
    ("unknown", FactPrecision::Unknown),
];

pub fn fact_precision_from_analysis_precision(
    precision: &OmenaQueryAnalysisPrecisionV0,
) -> FactPrecision {
    OMENA_QUERY_ANALYSIS_FACT_PRECISION_BY_VALUE_DOMAIN
        .iter()
        .find_map(|(value_domain, mapped)| {
            (*value_domain == precision.value_domain).then_some(*mapped)
        })
        .unwrap_or(FactPrecision::Unknown)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryAnalysisResultV0<TValue> {
    pub schema_version: String,
    pub product: String,
    pub value: TValue,
    pub precision: OmenaQueryAnalysisPrecisionV0,
    pub provenance: Vec<String>,
    pub revision: u64,
}

impl<TValue> OmenaQueryAnalysisResultV0<TValue> {
    pub fn new(
        value: TValue,
        precision: OmenaQueryAnalysisPrecisionV0,
        provenance: Vec<String>,
        revision: u64,
    ) -> Self {
        Self {
            schema_version: OMENA_QUERY_CURRENT_SCHEMA_VERSION.to_string(),
            product: "omena-query.analysis-result".to_string(),
            value,
            precision,
            provenance,
            revision,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryFragmentBundleV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub input_version: String,
    pub expression_semantics: ExpressionSemanticsQueryFragmentsV0,
    pub source_resolution: SourceResolutionQueryFragmentsV0,
    pub selector_usage: SelectorUsageQueryFragmentsV0,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryExpressionDomainIncrementalFlowAnalysisV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub input_version: String,
    pub revision: u64,
    pub graph_count: usize,
    pub dirty_graph_count: usize,
    pub reused_graph_count: usize,
    pub analyses: Vec<OmenaQueryExpressionDomainIncrementalFlowAnalysisEntryV0>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryExpressionDomainIncrementalFlowAnalysisEntryV0 {
    pub graph_id: String,
    pub file_path: String,
    pub analysis: ClassValueFlowIncrementalAnalysisV0,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryExpressionDomainSelectorProjectionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub input_version: String,
    pub projection_count: usize,
    pub projections: Vec<OmenaQueryExpressionDomainSelectorProjectionEntryV0>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryExpressionDomainSelectorProjectionEntryV0 {
    pub graph_id: String,
    pub file_path: String,
    pub node_id: String,
    pub target_style_paths: Vec<String>,
    pub value_kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduced_product: Option<ReducedClassValueProductV0>,
    pub selector_names: Vec<String>,
    pub certainty: SelectorProjectionCertaintyV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryExpressionDomainSelectorPrecisionV0 {
    pub graph_id: String,
    pub node_id: String,
    pub precision: FactPrecision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExpressionDomainSelectorCertaintyFlowHedgeV0 {
    graph_converged: bool,
    contains_flow_iteration_limit: bool,
}

#[derive(Default)]
pub struct OmenaQueryExpressionDomainFlowRuntimeV0 {
    revision: u64,
    databases_by_graph_id: BTreeMap<String, OmenaIncrementalDatabaseV0>,
    previous_analyses_by_graph_id: BTreeMap<String, ClassValueFlowAnalysisV0>,
}

impl OmenaQueryExpressionDomainFlowRuntimeV0 {
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn graph_count(&self) -> usize {
        self.databases_by_graph_id.len()
    }

    pub fn analyze_input(
        &mut self,
        input: &EngineInputV2,
    ) -> OmenaQueryExpressionDomainIncrementalFlowAnalysisV0 {
        self.revision += 1;
        let revision = self.revision;
        let flow_graphs = collect_expression_domain_flow_graphs(input);
        let live_graph_ids = flow_graphs
            .iter()
            .map(|entry| entry.graph_id.clone())
            .collect::<BTreeSet<_>>();

        self.databases_by_graph_id
            .retain(|graph_id, _| live_graph_ids.contains(graph_id));
        self.previous_analyses_by_graph_id
            .retain(|graph_id, _| live_graph_ids.contains(graph_id));

        let analyses = flow_graphs
            .into_iter()
            .map(|entry| {
                let database = self
                    .databases_by_graph_id
                    .entry(entry.graph_id.clone())
                    .or_default();
                let previous_analysis = self.previous_analyses_by_graph_id.get(&entry.graph_id);
                let analysis = analyze_class_value_flow_incremental_with_database(
                    &entry.graph,
                    database,
                    previous_analysis,
                    revision,
                );
                self.previous_analyses_by_graph_id
                    .insert(entry.graph_id.clone(), analysis.analysis.clone());

                OmenaQueryExpressionDomainIncrementalFlowAnalysisEntryV0 {
                    graph_id: entry.graph_id,
                    file_path: entry.file_path,
                    analysis,
                }
            })
            .collect::<Vec<_>>();

        let dirty_graph_count = analyses
            .iter()
            .filter(|entry| entry.analysis.incremental_plan.dirty_node_count > 0)
            .count();
        let reused_graph_count = analyses
            .iter()
            .filter(|entry| entry.analysis.reused_previous_analysis)
            .count();

        OmenaQueryExpressionDomainIncrementalFlowAnalysisV0 {
            schema_version: OMENA_QUERY_CURRENT_SCHEMA_VERSION,
            product: "omena-query.expression-domain-incremental-flow-analysis",
            input_version: input.version.clone(),
            revision,
            graph_count: analyses.len(),
            dirty_graph_count,
            reused_graph_count,
            analyses,
        }
    }
}

pub fn summarize_omena_query_core_abstract_value_domain() -> AbstractValueDomainSummaryV0 {
    summarize_omena_abstract_value_domain()
}

pub fn summarize_omena_query_fragment_bundle(input: &EngineInputV2) -> OmenaQueryFragmentBundleV0 {
    OmenaQueryFragmentBundleV0 {
        schema_version: OMENA_QUERY_CURRENT_SCHEMA_VERSION,
        product: "omena-query.fragment-bundle",
        input_version: input.version.clone(),
        expression_semantics: summarize_omena_query_expression_semantics_query_fragments(input),
        source_resolution: summarize_omena_query_source_resolution_query_fragments(input),
        selector_usage: summarize_omena_query_selector_usage_query_fragments(input),
    }
}

pub fn summarize_omena_query_expression_semantics_query_fragments(
    input: &EngineInputV2,
) -> ExpressionSemanticsQueryFragmentsV0 {
    summarize_expression_semantics_query_fragments_input(input)
}

pub fn summarize_omena_query_expression_domain_flow_analysis(
    input: &EngineInputV2,
) -> ExpressionDomainFlowAnalysisV0 {
    summarize_expression_domain_flow_analysis_input(input)
}

pub fn summarize_omena_query_expression_domain_control_flow_analysis(
    input: &EngineInputV2,
) -> ExpressionDomainControlFlowAnalysisV0 {
    summarize_expression_domain_control_flow_analysis_input(input)
}

pub fn summarize_omena_query_expression_domain_call_site_flow_analysis(
    input: &EngineInputV2,
) -> ExpressionDomainCallSiteFlowAnalysisV0 {
    summarize_expression_domain_call_site_flow_analysis_input(input)
}

pub fn summarize_omena_query_expression_domain_provenance_explanations(
    input: &EngineInputV2,
) -> ExpressionDomainProvenanceExplanationsV0 {
    summarize_expression_domain_provenance_explanations_input(input)
}

pub fn summarize_omena_query_expression_domain_reduced_product_iteration(
    input: &EngineInputV2,
) -> ExpressionDomainReducedProductIterationV0 {
    summarize_expression_domain_reduced_product_iteration_input(input)
}

pub fn summarize_omena_query_expression_domain_incremental_flow_analysis(
    input: &EngineInputV2,
    runtime: &mut OmenaQueryExpressionDomainFlowRuntimeV0,
) -> OmenaQueryExpressionDomainIncrementalFlowAnalysisV0 {
    runtime.analyze_input(input)
}

pub fn summarize_omena_query_expression_domain_incremental_flow_analysis_result(
    input: &EngineInputV2,
    runtime: &mut OmenaQueryExpressionDomainFlowRuntimeV0,
) -> OmenaQueryAnalysisResultV0<OmenaQueryExpressionDomainIncrementalFlowAnalysisV0> {
    let value = runtime.analyze_input(input);
    let revision = value.revision;
    OmenaQueryAnalysisResultV0::new(
        value,
        OmenaQueryAnalysisPrecisionV0 {
            product: "omena-query.analysis-precision".to_string(),
            value_domain: "classValueFlow".to_string(),
            flow_sensitivity: "incrementalDataflow".to_string(),
            context_sensitivity: "perExpressionGraph".to_string(),
            revision_axis: "OmenaQueryExpressionDomainFlowRuntimeV0.revision".to_string(),
        },
        vec![
            "omena-query-core.expression-domain-runtime".to_string(),
            "omena-abstract-value.incremental-class-value-flow".to_string(),
        ],
        revision,
    )
}

pub fn summarize_omena_query_expression_domain_selector_projection(
    input: &EngineInputV2,
) -> OmenaQueryExpressionDomainSelectorProjectionV0 {
    summarize_omena_query_expression_domain_selector_projection_with_precision(input).0
}

pub fn summarize_omena_query_expression_domain_selector_projection_with_precision(
    input: &EngineInputV2,
) -> (
    OmenaQueryExpressionDomainSelectorProjectionV0,
    Vec<OmenaQueryExpressionDomainSelectorPrecisionV0>,
) {
    summarize_omena_query_expression_domain_selector_projection_with_precision_and_style_path_resolver(
        input,
        |target, _known| Some(target.to_string()),
    )
}

pub fn summarize_omena_query_expression_domain_selector_projection_with_precision_and_style_path_resolver<
    F,
>(
    input: &EngineInputV2,
    resolve_style_path: F,
) -> (
    OmenaQueryExpressionDomainSelectorProjectionV0,
    Vec<OmenaQueryExpressionDomainSelectorPrecisionV0>,
)
where
    F: Fn(&str, &[String]) -> Option<String>,
{
    #[cfg(feature = "test-support")]
    record_selector_projection_evaluation_for_test();
    let selector_certainty_flow_hedges = expression_domain_selector_certainty_flow_hedges(input);
    summarize_omena_query_expression_domain_selector_projection_with_flow_hedges(
        input,
        resolve_style_path,
        &selector_certainty_flow_hedges,
    )
}

fn summarize_omena_query_expression_domain_selector_projection_with_flow_hedges<F>(
    input: &EngineInputV2,
    resolve_style_path: F,
    selector_certainty_flow_hedges: &BTreeMap<
        (String, String),
        ExpressionDomainSelectorCertaintyFlowHedgeV0,
    >,
) -> (
    OmenaQueryExpressionDomainSelectorProjectionV0,
    Vec<OmenaQueryExpressionDomainSelectorPrecisionV0>,
)
where
    F: Fn(&str, &[String]) -> Option<String>,
{
    let style_selectors_by_path = style_selector_universe_by_path(input);
    let known_style_paths = style_selectors_by_path.keys().cloned().collect::<Vec<_>>();
    let expression_targets = expression_target_style_paths(input);
    let flow_analysis = summarize_omena_query_expression_domain_flow_analysis(input);
    let mut projections = Vec::new();
    let mut precisions = Vec::new();

    for graph in flow_analysis.analyses {
        for node in graph.analysis.nodes {
            let target_style_paths = target_style_paths_for_flow_node(
                node.id.as_str(),
                node.predecessor_ids.as_slice(),
                &expression_targets,
            );
            let resolved_target_style_paths = target_style_paths
                .iter()
                .map(|path| resolve_style_path(path, known_style_paths.as_slice()))
                .collect::<Option<Vec<_>>>();
            let selector_universe = selector_universe_for_targets(
                resolved_target_style_paths.as_deref(),
                &style_selectors_by_path,
            );
            let projection = project_abstract_value_selectors(&node.value, &selector_universe);
            let certainty = hedge_selector_projection_certainty(
                projection.certainty,
                selector_certainty_flow_hedges.get(&(graph.file_path.clone(), node.id.clone())),
            );
            precisions.push(OmenaQueryExpressionDomainSelectorPrecisionV0 {
                graph_id: graph.graph_id.clone(),
                node_id: node.id.clone(),
                precision: fact_precision_from_class_value(&node.value),
            });
            projections.push(OmenaQueryExpressionDomainSelectorProjectionEntryV0 {
                graph_id: graph.graph_id.clone(),
                file_path: graph.file_path.clone(),
                node_id: node.id,
                target_style_paths,
                value_kind: node.value_kind,
                reduced_product: summarize_reduced_class_value_product(&node.value),
                selector_names: projection.selector_names,
                certainty,
            });
        }
    }

    (
        OmenaQueryExpressionDomainSelectorProjectionV0 {
            schema_version: OMENA_QUERY_CURRENT_SCHEMA_VERSION,
            product: "omena-query.expression-domain-selector-projection",
            input_version: input.version.clone(),
            projection_count: projections.len(),
            projections,
        },
        precisions,
    )
}

fn expression_domain_selector_certainty_flow_hedges(
    input: &EngineInputV2,
) -> BTreeMap<(String, String), ExpressionDomainSelectorCertaintyFlowHedgeV0> {
    bind_expression_domain_selector_certainty_flow_hedges(
        input,
        summarize_omena_query_expression_domain_control_flow_analysis(input),
    )
}

fn bind_expression_domain_selector_certainty_flow_hedges(
    input: &EngineInputV2,
    control_flow: ExpressionDomainControlFlowAnalysisV0,
) -> BTreeMap<(String, String), ExpressionDomainSelectorCertaintyFlowHedgeV0> {
    let control_flow_facts = input
        .type_facts
        .iter()
        .filter(|entry| entry.control_flow_graph.is_some())
        .collect::<Vec<_>>();
    assert_eq!(
        control_flow_facts.len(),
        control_flow.analyses.len(),
        "selector-certainty flow hedge requires one control analysis per control-flow type fact"
    );

    let mut hedges = BTreeMap::new();
    for (entry, analyzed) in control_flow_facts.into_iter().zip(control_flow.analyses) {
        let diagnostic_graph_id = format!(
            "{}:{}:expression-domain-control-flow",
            entry.file_path, entry.expression_id
        );
        assert_eq!(
            analyzed.file_path, entry.file_path,
            "selector-certainty flow hedge control analysis order/file mismatch"
        );
        assert_eq!(
            analyzed.graph_id, diagnostic_graph_id,
            "selector-certainty flow hedge control analysis order/graph mismatch"
        );

        let contains_flow_iteration_limit = analyzed
            .analysis
            .flow_analysis
            .nodes
            .iter()
            .any(|node| abstract_value_contains_flow_iteration_limit(&node.value));
        let key = (entry.file_path.clone(), entry.expression_id.clone());
        let previous = hedges.insert(
            key.clone(),
            ExpressionDomainSelectorCertaintyFlowHedgeV0 {
                graph_converged: analyzed.analysis.flow_analysis.converged,
                contains_flow_iteration_limit,
            },
        );
        assert!(
            previous.is_none(),
            "selector-certainty flow hedge duplicate type-fact key: file_path={:?} expression_id={:?}",
            key.0,
            key.1
        );
    }

    hedges
}

fn hedge_selector_projection_certainty(
    base: SelectorProjectionCertaintyV0,
    flow_hedge: Option<&ExpressionDomainSelectorCertaintyFlowHedgeV0>,
) -> SelectorProjectionCertaintyV0 {
    flow_hedge.map_or(base, |hedge| {
        hedge_selector_certainty_for_flow(
            base,
            hedge.graph_converged,
            hedge.contains_flow_iteration_limit,
        )
    })
}

fn hedge_selector_certainty_for_flow(
    base: SelectorProjectionCertaintyV0,
    graph_converged: bool,
    contains_flow_iteration_limit: bool,
) -> SelectorProjectionCertaintyV0 {
    if graph_converged && !contains_flow_iteration_limit {
        base
    } else {
        SelectorProjectionCertaintyV0::Possible
    }
}

fn abstract_value_contains_flow_iteration_limit(value: &AbstractClassValueV0) -> bool {
    let provenance = match value {
        AbstractClassValueV0::Automaton { provenance, .. }
        | AbstractClassValueV0::Prefix { provenance, .. }
        | AbstractClassValueV0::Suffix { provenance, .. }
        | AbstractClassValueV0::PrefixSuffix { provenance, .. }
        | AbstractClassValueV0::CharInclusion { provenance, .. }
        | AbstractClassValueV0::Composite { provenance, .. }
        | AbstractClassValueV0::Top { provenance } => *provenance,
        AbstractClassValueV0::Bottom
        | AbstractClassValueV0::Exact { .. }
        | AbstractClassValueV0::FiniteSet { .. } => None,
    };

    provenance == Some(AbstractClassValueProvenanceV0::FlowIterationLimit)
}

fn expression_target_style_paths(input: &EngineInputV2) -> BTreeMap<String, String> {
    input
        .sources
        .iter()
        .flat_map(|source| source.document.class_expressions.iter())
        .map(|expression| (expression.id.clone(), expression.scss_module_path.clone()))
        .collect()
}

fn style_selector_universe_by_path(input: &EngineInputV2) -> BTreeMap<String, Vec<String>> {
    input
        .styles
        .iter()
        .map(|style| {
            let selector_names = style
                .document
                .selectors
                .iter()
                .map(|selector| {
                    selector
                        .canonical_name
                        .clone()
                        .unwrap_or_else(|| selector.name.clone())
                })
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            (style.file_path.clone(), selector_names)
        })
        .collect()
}

fn target_style_paths_for_flow_node(
    node_id: &str,
    predecessor_ids: &[String],
    expression_targets: &BTreeMap<String, String>,
) -> Vec<String> {
    let mut targets = BTreeSet::new();
    if let Some(target) = expression_targets.get(node_id) {
        targets.insert(target.clone());
    }
    for predecessor_id in predecessor_ids {
        if let Some(target) = expression_targets.get(predecessor_id) {
            targets.insert(target.clone());
        }
    }
    targets.into_iter().collect()
}

fn selector_universe_for_targets(
    target_style_paths: Option<&[String]>,
    style_selectors_by_path: &BTreeMap<String, Vec<String>>,
) -> Vec<String> {
    let mut selectors = BTreeSet::new();
    if target_style_paths.is_none_or(<[String]>::is_empty) {
        for selector_names in style_selectors_by_path.values() {
            selectors.extend(selector_names.iter().cloned());
        }
    } else if let Some(target_style_paths) = target_style_paths {
        for target_style_path in target_style_paths {
            if let Some(selector_names) = style_selectors_by_path.get(target_style_path) {
                selectors.extend(selector_names.iter().cloned());
            }
        }
    }
    selectors.into_iter().collect()
}

pub fn summarize_omena_query_source_resolution_query_fragments(
    input: &EngineInputV2,
) -> SourceResolutionQueryFragmentsV0 {
    summarize_omena_resolver_query_fragments(input)
}

pub fn summarize_omena_query_selector_usage_query_fragments(
    input: &EngineInputV2,
) -> SelectorUsageQueryFragmentsV0 {
    summarize_selector_usage_query_fragments_input(input)
}

pub fn summarize_omena_query_source_resolution_canonical_producer_signal(
    input: &EngineInputV2,
) -> SourceResolutionCanonicalProducerSignalV0 {
    summarize_omena_resolver_canonical_producer_signal(input)
}

pub fn summarize_omena_query_source_resolution_runtime(
    input: &EngineInputV2,
) -> OmenaResolverSourceResolutionRuntimeIndexV0 {
    summarize_omena_resolver_source_resolution_runtime(input)
}

pub fn summarize_omena_query_expression_semantics_canonical_producer_signal(
    input: &EngineInputV2,
) -> ExpressionSemanticsCanonicalProducerSignalV0 {
    summarize_expression_semantics_canonical_producer_signal_input(input)
}

pub fn summarize_omena_query_selector_usage_canonical_producer_signal(
    input: &EngineInputV2,
) -> SelectorUsageCanonicalProducerSignalV0 {
    summarize_selector_usage_canonical_producer_signal_input(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn selector_certainty_product_input() -> EngineInputV2 {
        let range = RangeV2 {
            start: PositionV2 {
                line: 0,
                character: 0,
            },
            end: PositionV2 {
                line: 0,
                character: 11,
            },
        };
        EngineInputV2 {
            version: "selector-certainty-product".to_string(),
            sources: vec![SourceAnalysisInputV2 {
                document: SourceDocumentV2 {
                    class_expressions: vec![ClassExpressionInputV2 {
                        id: "expr-certainty".to_string(),
                        kind: "styleAccess".to_string(),
                        scss_module_path: "/tmp/App.module.scss".to_string(),
                        range: range.clone(),
                        class_name: Some("x".to_string()),
                        root_binding_decl_id: None,
                        access_path: Some(vec!["styles".to_string(), "x".to_string()]),
                    }],
                },
            }],
            styles: vec![StyleAnalysisInputV2 {
                file_path: "/tmp/App.module.scss".to_string(),
                source: None,
                document: StyleDocumentV2 {
                    selectors: vec![StyleSelectorV2 {
                        name: "x".to_string(),
                        view_kind: "canonical".to_string(),
                        canonical_name: Some("x".to_string()),
                        range,
                        nested_safety: Some("safe".to_string()),
                        composes: None,
                        bem_suffix: None,
                    }],
                },
            }],
            type_facts: vec![TypeFactEntryV2 {
                file_path: "/tmp/App.tsx".to_string(),
                expression_id: "expr-certainty".to_string(),
                facts: StringTypeFactsV2 {
                    kind: "exact".to_string(),
                    constraint_kind: None,
                    values: Some(vec!["x".to_string()]),
                    prefix: None,
                    suffix: None,
                    min_len: None,
                    max_len: None,
                    char_must: None,
                    char_may: None,
                    may_include_other_chars: None,
                    provenance: None,
                },
                control_flow_graph: Some(engine_input_producers::TypeFactControlFlowGraphV2 {
                    entry_block_id: "seed".to_string(),
                    blocks: vec![
                        engine_input_producers::TypeFactControlFlowBlockV2 {
                            id: "seed".to_string(),
                            kind: "assignment".to_string(),
                            transfer_kind: "assignFacts".to_string(),
                            successor_block_ids: vec!["loop".to_string()],
                            symbol_ordinal: None,
                            variable_name: None,
                            expression_kind: None,
                            boundary_effect: "unknownBoundary".to_string(),
                            facts: Some(StringTypeFactsV2 {
                                kind: "finiteSet".to_string(),
                                constraint_kind: None,
                                values: Some(vec!["a".to_string(), "b".to_string()]),
                                prefix: None,
                                suffix: None,
                                min_len: None,
                                max_len: None,
                                char_must: None,
                                char_may: None,
                                may_include_other_chars: None,
                                provenance: None,
                            }),
                        },
                        engine_input_producers::TypeFactControlFlowBlockV2 {
                            id: "loop".to_string(),
                            kind: "loop".to_string(),
                            transfer_kind: "concatFacts".to_string(),
                            successor_block_ids: vec!["loop".to_string()],
                            symbol_ordinal: None,
                            variable_name: None,
                            expression_kind: None,
                            boundary_effect: "unknownBoundary".to_string(),
                            facts: None,
                        },
                    ],
                }),
            }],
        }
    }

    fn selector_certainty_colon_collision_input() -> EngineInputV2 {
        let mut input = selector_certainty_product_input();
        input.sources[0].document.class_expressions[0].id = "b:c".to_string();
        input.sources[0].document.class_expressions[0].class_name = Some("x".to_string());
        input.sources[0].document.class_expressions[0].access_path =
            Some(vec!["styles".to_string(), "x".to_string()]);
        let second_expression = ClassExpressionInputV2 {
            id: "c".to_string(),
            kind: "styleAccess".to_string(),
            scss_module_path: "/tmp/App.module.scss".to_string(),
            range: input.sources[0].document.class_expressions[0].range.clone(),
            class_name: Some("y".to_string()),
            root_binding_decl_id: None,
            access_path: Some(vec!["styles".to_string(), "y".to_string()]),
        };
        input.sources[0]
            .document
            .class_expressions
            .push(second_expression);

        let second_selector = StyleSelectorV2 {
            name: "y".to_string(),
            view_kind: "canonical".to_string(),
            canonical_name: Some("y".to_string()),
            range: input.styles[0].document.selectors[0].range.clone(),
            nested_safety: Some("safe".to_string()),
            composes: None,
            bem_suffix: None,
        };
        input.styles[0].document.selectors.push(second_selector);

        let mut first_fact = input.type_facts[0].clone();
        first_fact.file_path = "/tmp/A".to_string();
        first_fact.expression_id = "b:c".to_string();
        let mut second_fact = first_fact.clone();
        second_fact.file_path = "/tmp/A:b".to_string();
        second_fact.expression_id = "c".to_string();
        second_fact.facts.values = Some(vec!["y".to_string()]);
        if let Some(second_graph) = second_fact.control_flow_graph.as_mut() {
            second_graph.blocks.truncate(1);
            second_graph.blocks[0].successor_block_ids.clear();
        }
        input.type_facts = vec![first_fact, second_fact];
        input
    }

    #[test]
    fn expression_domain_runtime_reuses_graph_databases_across_revisions() {
        let input = EngineInputV2 {
            version: "core-runtime".to_string(),
            sources: Vec::new(),
            styles: Vec::new(),
            type_facts: Vec::new(),
        };
        let mut runtime = OmenaQueryExpressionDomainFlowRuntimeV0::default();

        let first =
            summarize_omena_query_expression_domain_incremental_flow_analysis(&input, &mut runtime);
        let second =
            summarize_omena_query_expression_domain_incremental_flow_analysis(&input, &mut runtime);

        assert_eq!(first.revision, 1);
        assert_eq!(second.revision, 2);
        assert_eq!(runtime.revision(), 2);
    }

    #[test]
    fn nonconverged_flow_hedge_demotes_typed_query_projection() {
        let input = selector_certainty_product_input();
        let graph_id = "/tmp/App.tsx:expr-certainty:expression-domain-control-flow";
        let control_flow = summarize_omena_query_expression_domain_control_flow_analysis(&input);
        let projection = summarize_omena_query_expression_domain_selector_projection(&input);
        let control_entry = &control_flow.analyses[0];
        let entry = &projection.projections[0];

        println!(
            "certainty-hedge-census graphId={graph_id} hedged=1 base=exact certainty=possible"
        );
        assert_eq!(control_entry.graph_id, graph_id);
        assert!(!control_entry.analysis.flow_analysis.converged);
        assert!(
            control_entry
                .analysis
                .flow_analysis
                .nodes
                .iter()
                .all(|node| {
                    matches!(
                        &node.value,
                        AbstractClassValueV0::Top {
                            provenance: Some(AbstractClassValueProvenanceV0::FlowIterationLimit)
                        }
                    )
                })
        );
        assert_eq!(entry.value_kind, "exact");
        assert_eq!(entry.selector_names, vec!["x".to_string()]);
        assert_eq!(entry.certainty, SelectorProjectionCertaintyV0::Possible);
    }

    #[test]
    fn colon_colliding_graph_ids_bind_selector_certainty_by_type_fact_tuple() {
        let input = selector_certainty_colon_collision_input();
        let control_flow = summarize_omena_query_expression_domain_control_flow_analysis(&input);
        let projection = summarize_omena_query_expression_domain_selector_projection(&input);

        assert_eq!(control_flow.analyses.len(), 2);
        assert_eq!(
            control_flow.analyses[0].graph_id, control_flow.analyses[1].graph_id,
            "fixture must retain the diagnostic graph-id collision"
        );
        let nonconverged = projection
            .projections
            .iter()
            .filter(|entry| entry.file_path == "/tmp/A" && entry.node_id == "b:c")
            .collect::<Vec<_>>();
        let converged = projection
            .projections
            .iter()
            .filter(|entry| entry.file_path == "/tmp/A:b" && entry.node_id == "c")
            .collect::<Vec<_>>();

        assert_eq!(nonconverged.len(), 1);
        assert_eq!(converged.len(), 1);
        assert_eq!(
            nonconverged[0].certainty,
            SelectorProjectionCertaintyV0::Possible
        );
        assert_eq!(converged[0].certainty, SelectorProjectionCertaintyV0::Exact);
    }

    #[test]
    #[should_panic(expected = "selector-certainty flow hedge duplicate type-fact key")]
    fn duplicate_selector_certainty_type_fact_key_fails_closed() {
        let mut input = selector_certainty_product_input();
        input.type_facts.push(input.type_facts[0].clone());

        let _ = summarize_omena_query_expression_domain_selector_projection(&input);
    }

    #[test]
    #[should_panic(
        expected = "selector-certainty flow hedge requires one control analysis per control-flow type fact"
    )]
    fn missing_selector_certainty_control_analysis_fails_closed() {
        let input = selector_certainty_product_input();
        let mut control_flow =
            summarize_omena_query_expression_domain_control_flow_analysis(&input);
        control_flow.analyses.clear();

        let _ = bind_expression_domain_selector_certainty_flow_hedges(&input, control_flow);
    }

    #[test]
    fn analysis_precision_view_maps_known_producers_and_fails_closed() {
        let precision = |value_domain: &str| OmenaQueryAnalysisPrecisionV0 {
            product: "omena-query.analysis-precision".to_string(),
            value_domain: value_domain.to_string(),
            flow_sensitivity: "fixture".to_string(),
            context_sensitivity: "fixture".to_string(),
            revision_axis: "fixture".to_string(),
        };

        assert_eq!(
            fact_precision_from_analysis_precision(&precision("cascadeAtPosition")),
            FactPrecision::Exact
        );
        assert_eq!(
            fact_precision_from_analysis_precision(&precision("styleModuleResolution")),
            FactPrecision::Exact
        );
        assert_eq!(
            fact_precision_from_analysis_precision(&precision("classValueResolution")),
            FactPrecision::Conservative
        );
        assert_eq!(
            fact_precision_from_analysis_precision(&precision("classValueUniverse")),
            FactPrecision::Conservative
        );
        assert_eq!(
            fact_precision_from_analysis_precision(&precision("classValueFlow")),
            FactPrecision::Heuristic
        );
        assert_eq!(
            fact_precision_from_analysis_precision(&precision("unknown")),
            FactPrecision::Unknown
        );
        assert_eq!(
            fact_precision_from_analysis_precision(&precision("unregistered")),
            FactPrecision::Unknown
        );
    }
}
