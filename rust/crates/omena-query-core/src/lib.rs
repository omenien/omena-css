//! Core query runtime primitives below the public `omena-query` facade.
//!
//! This crate owns producer-fragment summaries and expression-domain runtime
//! state. `omena-query` re-exports these surfaces, but no longer needs to depend
//! directly on each lower-level producer crate for this part of the dataflow.

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
pub use omena_abstract_value::{
    AbstractClassValueV0, AbstractPropertyValueCandidateV0, AbstractPropertyValueNarrowingV0,
    AbstractPropertyValueV0, AbstractValueDomainSummaryV0, CascadeContextV0,
    CascadeValueFamilyMemberV0, ClassValueFlowAnalysisV0, ClassValueFlowIncrementalAnalysisV0,
    Lin01ProvenanceSemiringV0, LinearProvenancePathV0, LinearProvenanceV0,
    NaturalCountProvenanceSemiringV0, PolynomialProvenanceProjectionV0, PolynomialProvenanceTermV0,
    PolynomialProvenanceV0, PolynomialProvenanceVariableV0, ProvenanceSemiringLawReportV0,
    ReducedClassValueProductIterationV0, ReducedClassValueProductV0, SelectorProjectionCertaintyV0,
    derive_cascade_restriction_maps_v0, iterate_reduced_class_value_product_constraints,
    narrow_abstract_property_value_for_cascade_branch,
    narrow_abstract_property_value_for_pseudo_state, prefix_suffix_class_value,
    summarize_cascade_value_family_v0, summarize_polynomial_provenance_from_linear_v0,
    verify_provenance_semiring_laws_on_fixtures,
};
use omena_abstract_value::{
    analyze_class_value_flow_incremental_with_database, project_abstract_value_selectors,
    summarize_omena_abstract_value_domain, summarize_reduced_class_value_product,
};
use omena_incremental::OmenaIncrementalDatabaseV0;
pub use omena_incremental::{
    IncrementalEditDistancePriorityInputV0, IncrementalGraphInputV0,
    IncrementalInvalidationPriorityPlanV0, IncrementalNodeInputV0, IncrementalRevisionV0,
    plan_incremental_computation_with_priority_inputs, snapshot_from_graph_input,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryAnalysisPrecisionV0 {
    pub product: String,
    pub value_domain: String,
    pub flow_sensitivity: String,
    pub context_sensitivity: String,
    pub revision_axis: String,
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
    let style_selectors_by_path = style_selector_universe_by_path(input);
    let expression_targets = expression_target_style_paths(input);
    let flow_analysis = summarize_omena_query_expression_domain_flow_analysis(input);
    let mut projections = Vec::new();

    for graph in flow_analysis.analyses {
        for node in graph.analysis.nodes {
            let target_style_paths = target_style_paths_for_flow_node(
                node.id.as_str(),
                node.predecessor_ids.as_slice(),
                &expression_targets,
            );
            let selector_universe = selector_universe_for_targets(
                target_style_paths.as_slice(),
                &style_selectors_by_path,
            );
            let projection = project_abstract_value_selectors(&node.value, &selector_universe);
            projections.push(OmenaQueryExpressionDomainSelectorProjectionEntryV0 {
                graph_id: graph.graph_id.clone(),
                file_path: graph.file_path.clone(),
                node_id: node.id,
                target_style_paths,
                value_kind: node.value_kind,
                reduced_product: summarize_reduced_class_value_product(&node.value),
                selector_names: projection.selector_names,
                certainty: projection.certainty,
            });
        }
    }

    OmenaQueryExpressionDomainSelectorProjectionV0 {
        schema_version: OMENA_QUERY_CURRENT_SCHEMA_VERSION,
        product: "omena-query.expression-domain-selector-projection",
        input_version: input.version.clone(),
        projection_count: projections.len(),
        projections,
    }
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
    target_style_paths: &[String],
    style_selectors_by_path: &BTreeMap<String, Vec<String>>,
) -> Vec<String> {
    let mut selectors = BTreeSet::new();
    if target_style_paths.is_empty() {
        for selector_names in style_selectors_by_path.values() {
            selectors.extend(selector_names.iter().cloned());
        }
    } else {
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
}
