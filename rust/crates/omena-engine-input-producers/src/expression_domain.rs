use std::collections::BTreeMap;

use crate::{
    ConstraintDetailCounts, ConstraintDetailInput, EngineInputV2,
    ExpressionDomainCallSiteFlowAnalysisV0, ExpressionDomainCandidateV0,
    ExpressionDomainCandidatesV0, ExpressionDomainCanonicalCandidateBundleV0,
    ExpressionDomainCanonicalProducerSignalV0, ExpressionDomainControlFlowAnalysisEntryV0,
    ExpressionDomainControlFlowAnalysisV0, ExpressionDomainEvaluatorCandidatePayloadV0,
    ExpressionDomainEvaluatorCandidateV0, ExpressionDomainEvaluatorCandidatesV0,
    ExpressionDomainFlowAnalysisEntryV0, ExpressionDomainFlowAnalysisV0,
    ExpressionDomainFlowGraphEntryV0, ExpressionDomainFragmentV0, ExpressionDomainFragmentsV0,
    ExpressionDomainPlanSummaryV0, ExpressionDomainProvenanceExplanationV0,
    ExpressionDomainProvenanceExplanationsV0, ExpressionDomainReducedProductIterationEntryV0,
    ExpressionDomainReducedProductIterationV0, StringTypeFactsV2, TypeFactControlFlowBlockV2,
    TypeFactControlFlowGraphV2, TypeFactEntryV2, abstract_value_facts,
    collect_constraint_detail_counts, map_reduced_expression_value_domain_derivation,
    map_reduced_expression_value_domain_kind, map_reduced_expression_value_domain_provenance_tree,
};

struct ExpressionDomainInputRows {
    plan_summary: ExpressionDomainPlanSummaryV0,
    fragments: Vec<ExpressionDomainFragmentV0>,
    candidates: Vec<ExpressionDomainCandidateV0>,
    evaluator_candidates: Vec<ExpressionDomainEvaluatorCandidateV0>,
}

fn collect_expression_domain_input_rows(input: &EngineInputV2) -> ExpressionDomainInputRows {
    let mut planned_expression_ids = Vec::new();
    let mut value_domain_kinds = BTreeMap::new();
    let mut value_constraint_kinds = BTreeMap::new();
    let mut constraint_detail_counts = ConstraintDetailCounts::default();
    let mut finite_value_count = 0usize;
    let mut fragments = Vec::new();
    let mut candidates = Vec::new();
    let mut evaluator_candidates = Vec::new();

    for entry in &input.type_facts {
        planned_expression_ids.push(entry.expression_id.clone());
        *value_domain_kinds
            .entry(entry.facts.kind.clone())
            .or_insert(0) += 1;

        if let Some(values) = &entry.facts.values {
            finite_value_count += values.len();
        }

        if let Some(constraint_kind) = &entry.facts.constraint_kind {
            *value_constraint_kinds
                .entry(constraint_kind.clone())
                .or_insert(0) += 1;
        }

        collect_constraint_detail_counts(
            &mut constraint_detail_counts,
            ConstraintDetailInput {
                prefix: entry.facts.prefix.as_ref(),
                suffix: entry.facts.suffix.as_ref(),
                min_len: entry.facts.min_len,
                max_len: entry.facts.max_len,
                char_must: entry.facts.char_must.as_ref(),
                char_may: entry.facts.char_may.as_ref(),
                may_include_other_chars: entry.facts.may_include_other_chars,
            },
        );

        let fragment = ExpressionDomainFragmentV0 {
            expression_id: entry.expression_id.clone(),
            file_path: entry.file_path.clone(),
            value_domain_kind: entry.facts.kind.clone(),
            value_constraint_kind: entry.facts.constraint_kind.clone(),
            value_prefix: entry.facts.prefix.clone(),
            value_suffix: entry.facts.suffix.clone(),
            value_min_len: entry.facts.min_len,
            value_max_len: entry.facts.max_len,
            value_char_must: entry.facts.char_must.clone(),
            value_char_may: entry.facts.char_may.clone(),
            value_may_include_other_chars: entry.facts.may_include_other_chars,
            finite_value_count: entry.facts.values.as_ref().map_or(0, Vec::len),
        };
        fragments.push(fragment.clone());
        candidates.push(ExpressionDomainCandidateV0 {
            expression_id: fragment.expression_id,
            file_path: fragment.file_path,
            value_domain_kind: fragment.value_domain_kind,
            value_constraint_kind: fragment.value_constraint_kind,
            value_prefix: fragment.value_prefix,
            value_suffix: fragment.value_suffix,
            value_min_len: fragment.value_min_len,
            value_max_len: fragment.value_max_len,
            value_char_must: fragment.value_char_must,
            value_char_may: fragment.value_char_may,
            value_may_include_other_chars: fragment.value_may_include_other_chars,
            finite_value_count: fragment.finite_value_count,
        });

        evaluator_candidates.push(ExpressionDomainEvaluatorCandidateV0 {
            kind: "expression-domain",
            file_path: entry.file_path.clone(),
            query_id: entry.expression_id.clone(),
            payload: ExpressionDomainEvaluatorCandidatePayloadV0 {
                expression_id: entry.expression_id.clone(),
                value_domain_kind: map_reduced_expression_value_domain_kind(&entry.facts),
                value_constraint_kind: entry.facts.constraint_kind.clone(),
                value_prefix: entry.facts.prefix.clone(),
                value_suffix: entry.facts.suffix.clone(),
                value_min_len: entry.facts.min_len,
                value_max_len: entry.facts.max_len,
                value_char_must: entry.facts.char_must.clone(),
                value_char_may: entry.facts.char_may.clone(),
                value_may_include_other_chars: entry.facts.may_include_other_chars,
                finite_value_count: entry.facts.values.as_ref().map_or(0, Vec::len),
                value_domain_derivation: map_reduced_expression_value_domain_derivation(
                    &entry.facts,
                ),
                value_domain_provenance_tree: map_reduced_expression_value_domain_provenance_tree(
                    &entry.facts,
                ),
            },
        });
    }

    fragments.sort_by(|a, b| a.expression_id.cmp(&b.expression_id));
    candidates.sort_by(|a, b| a.expression_id.cmp(&b.expression_id));
    evaluator_candidates.sort_by(|a, b| a.query_id.cmp(&b.query_id));

    ExpressionDomainInputRows {
        plan_summary: ExpressionDomainPlanSummaryV0 {
            schema_version: "0",
            input_version: input.version.clone(),
            planned_expression_ids,
            value_domain_kinds,
            value_constraint_kinds,
            constraint_detail_counts,
            finite_value_count,
        },
        fragments,
        candidates,
        evaluator_candidates,
    }
}

pub fn summarize_expression_domain_plan_input(
    input: &EngineInputV2,
) -> ExpressionDomainPlanSummaryV0 {
    collect_expression_domain_input_rows(input).plan_summary
}

pub fn summarize_expression_domain_fragments_input(
    input: &EngineInputV2,
) -> ExpressionDomainFragmentsV0 {
    let rows = collect_expression_domain_input_rows(input);

    ExpressionDomainFragmentsV0 {
        schema_version: "0",
        input_version: input.version.clone(),
        fragments: rows.fragments,
    }
}

pub fn summarize_expression_domain_candidates_input(
    input: &EngineInputV2,
) -> ExpressionDomainCandidatesV0 {
    let rows = collect_expression_domain_input_rows(input);

    ExpressionDomainCandidatesV0 {
        schema_version: "0",
        input_version: input.version.clone(),
        candidates: rows.candidates,
    }
}

pub fn summarize_expression_domain_canonical_candidate_bundle_input(
    input: &EngineInputV2,
) -> ExpressionDomainCanonicalCandidateBundleV0 {
    let rows = collect_expression_domain_input_rows(input);

    ExpressionDomainCanonicalCandidateBundleV0 {
        schema_version: "0",
        input_version: input.version.clone(),
        plan_summary: rows.plan_summary,
        fragments: rows.fragments,
        candidates: rows.candidates,
    }
}

pub fn summarize_expression_domain_evaluator_candidates_input(
    input: &EngineInputV2,
) -> ExpressionDomainEvaluatorCandidatesV0 {
    let rows = collect_expression_domain_input_rows(input);

    ExpressionDomainEvaluatorCandidatesV0 {
        schema_version: "0",
        input_version: input.version.clone(),
        results: rows.evaluator_candidates,
    }
}

pub fn summarize_expression_domain_canonical_producer_signal_input(
    input: &EngineInputV2,
) -> ExpressionDomainCanonicalProducerSignalV0 {
    let rows = collect_expression_domain_input_rows(input);
    let input_version = input.version.clone();

    ExpressionDomainCanonicalProducerSignalV0 {
        schema_version: "0",
        input_version: input_version.clone(),
        canonical_bundle: ExpressionDomainCanonicalCandidateBundleV0 {
            schema_version: "0",
            input_version: input_version.clone(),
            plan_summary: rows.plan_summary,
            fragments: rows.fragments,
            candidates: rows.candidates,
        },
        evaluator_candidates: ExpressionDomainEvaluatorCandidatesV0 {
            schema_version: "0",
            input_version,
            results: rows.evaluator_candidates,
        },
    }
}

pub fn summarize_expression_domain_provenance_explanations_input(
    input: &EngineInputV2,
) -> ExpressionDomainProvenanceExplanationsV0 {
    let explanations = input
        .type_facts
        .iter()
        .map(|entry| {
            let derivation = map_reduced_expression_value_domain_derivation(&entry.facts);
            let provenance_tree = map_reduced_expression_value_domain_provenance_tree(&entry.facts);

            ExpressionDomainProvenanceExplanationV0 {
                expression_id: entry.expression_id.clone(),
                file_path: entry.file_path.clone(),
                input_fact_kind: derivation.input_fact_kind.clone(),
                input_constraint_kind: derivation.input_constraint_kind.clone(),
                reduced_kind: derivation.reduced_kind,
                derivation,
                provenance_tree,
            }
        })
        .collect::<Vec<_>>();

    ExpressionDomainProvenanceExplanationsV0 {
        schema_version: "0",
        product: "engine-input-producers.expression-domain-provenance-explanations",
        input_version: input.version.clone(),
        explanation_count: explanations.len(),
        explanations,
    }
}

pub fn summarize_expression_domain_flow_analysis_input(
    input: &EngineInputV2,
) -> ExpressionDomainFlowAnalysisV0 {
    let analyses = collect_expression_domain_flow_graphs(input)
        .into_iter()
        .map(|entry| ExpressionDomainFlowAnalysisEntryV0 {
            graph_id: entry.graph_id,
            file_path: entry.file_path,
            analysis: omena_abstract_value::analyze_class_value_flow(&entry.graph),
        })
        .collect();

    ExpressionDomainFlowAnalysisV0 {
        schema_version: "0",
        product: "engine-input-producers.expression-domain-flow-analysis",
        input_version: input.version.clone(),
        analyses,
    }
}

pub fn summarize_expression_domain_control_flow_analysis_input(
    input: &EngineInputV2,
) -> ExpressionDomainControlFlowAnalysisV0 {
    let analyses = collect_expression_domain_control_flow_graphs(input)
        .into_iter()
        .map(|entry| ExpressionDomainControlFlowAnalysisEntryV0 {
            graph_id: entry.graph_id,
            file_path: entry.file_path,
            analysis: omena_abstract_value::analyze_class_value_control_flow_graph(&entry.graph),
        })
        .collect();

    ExpressionDomainControlFlowAnalysisV0 {
        schema_version: "0",
        product: "engine-input-producers.expression-domain-control-flow-analysis",
        input_version: input.version.clone(),
        analyses,
    }
}

pub fn summarize_expression_domain_call_site_flow_analysis_input(
    input: &EngineInputV2,
) -> ExpressionDomainCallSiteFlowAnalysisV0 {
    let call_site_inputs = collect_expression_domain_call_site_flow_inputs(input);

    ExpressionDomainCallSiteFlowAnalysisV0 {
        schema_version: "0",
        product: "engine-input-producers.expression-domain-call-site-flow-analysis",
        input_version: input.version.clone(),
        zero_cfa: omena_abstract_value::analyze_k_limited_call_site_flows(&call_site_inputs, 0),
        one_cfa: omena_abstract_value::analyze_k_limited_call_site_flows(&call_site_inputs, 1),
    }
}

pub fn summarize_expression_domain_reduced_product_iteration_input(
    input: &EngineInputV2,
) -> ExpressionDomainReducedProductIterationV0 {
    let iterations = input
        .type_facts
        .iter()
        .filter_map(|entry| {
            let axis_constraints = reduced_product_axis_constraints_from_facts(&entry.facts);
            (!axis_constraints.is_empty()).then(|| {
                let iteration =
                    omena_abstract_value::iterate_reduced_class_value_product_constraints(
                        &axis_constraints,
                    );
                ExpressionDomainReducedProductIterationEntryV0 {
                    expression_id: entry.expression_id.clone(),
                    file_path: entry.file_path.clone(),
                    input_value_kind: map_reduced_expression_value_domain_kind(&entry.facts),
                    axis_constraint_count: axis_constraints.len(),
                    iteration,
                }
            })
        })
        .collect::<Vec<_>>();

    ExpressionDomainReducedProductIterationV0 {
        schema_version: "0",
        product: "engine-input-producers.expression-domain-reduced-product-iteration",
        input_version: input.version.clone(),
        iteration_count: iterations.len(),
        iterations,
    }
}

pub fn collect_expression_domain_flow_graphs(
    input: &EngineInputV2,
) -> Vec<ExpressionDomainFlowGraphEntryV0> {
    input
        .type_facts
        .iter()
        .map(|entry| {
            let graph_id = format!(
                "{}:{}:expression-domain-flow",
                entry.file_path, entry.expression_id
            );
            let graph = omena_abstract_value::ClassValueFlowGraphV0 {
                context_key: Some(graph_id.clone()),
                nodes: vec![omena_abstract_value::ClassValueFlowNodeV0 {
                    id: entry.expression_id.clone(),
                    predecessors: Vec::new(),
                    transfer: omena_abstract_value::ClassValueFlowTransferV0::AssignFacts(
                        abstract_value_facts(&entry.facts),
                    ),
                }],
            };

            ExpressionDomainFlowGraphEntryV0 {
                graph_id,
                file_path: entry.file_path.clone(),
                graph,
            }
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExpressionDomainControlFlowGraphEntryV0 {
    graph_id: String,
    file_path: String,
    graph: omena_abstract_value::ClassValueControlFlowGraphV0,
}

fn collect_expression_domain_control_flow_graphs(
    input: &EngineInputV2,
) -> Vec<ExpressionDomainControlFlowGraphEntryV0> {
    input
        .type_facts
        .iter()
        .filter_map(|entry| {
            entry.control_flow_graph.as_ref().map(|graph| {
                let graph_id = format!(
                    "{}:{}:expression-domain-control-flow",
                    entry.file_path, entry.expression_id
                );
                ExpressionDomainControlFlowGraphEntryV0 {
                    graph_id: graph_id.clone(),
                    file_path: entry.file_path.clone(),
                    graph: expression_domain_control_flow_graph_from_type_fact_graph(
                        &graph_id, entry, graph,
                    ),
                }
            })
        })
        .collect()
}

fn collect_expression_domain_call_site_flow_inputs(
    input: &EngineInputV2,
) -> Vec<omena_abstract_value::KLimitedCallSiteFlowInputV0> {
    collect_expression_domain_flow_graphs(input)
        .into_iter()
        .map(|entry| {
            let exit_node_id = expression_domain_flow_exit_node_id(&entry.graph);
            omena_abstract_value::KLimitedCallSiteFlowInputV0 {
                callee_key: "expression-domain-class-value".to_string(),
                call_site_stack: vec![entry.file_path, entry.graph_id],
                graph: entry.graph,
                exit_node_id,
            }
        })
        .collect()
}

fn reduced_product_axis_constraints_from_facts(
    facts: &StringTypeFactsV2,
) -> Vec<omena_abstract_value::AbstractClassValueV0> {
    let mut constraints = Vec::new();

    if let Some(prefix) = &facts.prefix {
        constraints.push(omena_abstract_value::prefix_class_value(
            prefix.clone(),
            None,
        ));
    }

    if let Some(suffix) = &facts.suffix {
        constraints.push(omena_abstract_value::suffix_class_value(
            suffix.clone(),
            None,
        ));
    }

    if facts.char_must.is_some()
        || facts.char_may.is_some()
        || facts.may_include_other_chars.is_some()
    {
        constraints.push(omena_abstract_value::char_inclusion_class_value(
            facts.char_must.clone().unwrap_or_default(),
            facts.char_may.clone().unwrap_or_default(),
            None,
            facts.may_include_other_chars.unwrap_or(false),
        ));
    }

    constraints
}

fn expression_domain_flow_exit_node_id(
    graph: &omena_abstract_value::ClassValueFlowGraphV0,
) -> String {
    graph
        .nodes
        .first()
        .map(|node| node.id.clone())
        .unwrap_or_else(|| "exit".to_string())
}

fn expression_domain_control_flow_graph_from_type_fact_graph(
    graph_id: &str,
    entry: &TypeFactEntryV2,
    graph: &TypeFactControlFlowGraphV2,
) -> omena_abstract_value::ClassValueControlFlowGraphV0 {
    let predecessor_block_ids = control_flow_predecessor_block_ids(&graph.blocks);
    let block_node_ids = graph
        .blocks
        .iter()
        .map(|block| {
            (
                block.id.clone(),
                type_fact_control_flow_node_id(entry, block),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let blocks = graph
        .blocks
        .iter()
        .map(|block| {
            let node_id = block_node_ids
                .get(&block.id)
                .cloned()
                .unwrap_or_else(|| type_fact_control_flow_node_id(entry, block));
            let predecessors = predecessor_block_ids
                .get(&block.id)
                .into_iter()
                .flat_map(|ids| ids.iter())
                .filter_map(|id| block_node_ids.get(id).cloned())
                .collect::<Vec<_>>();
            omena_abstract_value::ClassValueControlFlowBlockV0 {
                id: block.id.clone(),
                nodes: vec![omena_abstract_value::ClassValueFlowNodeV0 {
                    id: node_id,
                    predecessors,
                    transfer: type_fact_control_flow_transfer(block, &entry.facts),
                }],
                successor_block_ids: block.successor_block_ids.clone(),
            }
        })
        .collect();

    omena_abstract_value::ClassValueControlFlowGraphV0 {
        context_key: Some(graph_id.to_string()),
        entry_block_id: graph.entry_block_id.clone(),
        blocks,
    }
}

fn control_flow_predecessor_block_ids(
    blocks: &[TypeFactControlFlowBlockV2],
) -> BTreeMap<String, Vec<String>> {
    let mut predecessors = BTreeMap::<String, Vec<String>>::new();
    for block in blocks {
        for successor in &block.successor_block_ids {
            predecessors
                .entry(successor.clone())
                .or_default()
                .push(block.id.clone());
        }
    }
    predecessors
}

fn type_fact_control_flow_node_id(
    entry: &TypeFactEntryV2,
    block: &TypeFactControlFlowBlockV2,
) -> String {
    format!("{}:{}", entry.expression_id, block.id)
}

fn type_fact_control_flow_transfer(
    block: &TypeFactControlFlowBlockV2,
    facts: &StringTypeFactsV2,
) -> omena_abstract_value::ClassValueFlowTransferV0 {
    if let Some(block_facts) = &block.facts
        && matches!(block.transfer_kind.as_str(), "assignFacts" | "concatFacts")
    {
        return omena_abstract_value::ClassValueFlowTransferV0::AssignFacts(abstract_value_facts(
            block_facts,
        ));
    }

    match block.transfer_kind.as_str() {
        "assignFacts" => {
            omena_abstract_value::ClassValueFlowTransferV0::AssignFacts(abstract_value_facts(facts))
        }
        "concatFacts" => {
            omena_abstract_value::ClassValueFlowTransferV0::ConcatFacts(abstract_value_facts(facts))
        }
        _ => omena_abstract_value::ClassValueFlowTransferV0::Join,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        collect_expression_domain_flow_graphs,
        summarize_expression_domain_call_site_flow_analysis_input,
        summarize_expression_domain_candidates_input,
        summarize_expression_domain_canonical_candidate_bundle_input,
        summarize_expression_domain_canonical_producer_signal_input,
        summarize_expression_domain_control_flow_analysis_input,
        summarize_expression_domain_evaluator_candidates_input,
        summarize_expression_domain_flow_analysis_input,
        summarize_expression_domain_fragments_input, summarize_expression_domain_plan_input,
        summarize_expression_domain_provenance_explanations_input,
        summarize_expression_domain_reduced_product_iteration_input,
    };
    use crate::{
        StringTypeFactsV2, TypeFactControlFlowBlockV2, TypeFactControlFlowGraphV2, TypeFactEntryV2,
        test_support::sample_input,
    };
    use omena_abstract_value::AbstractClassValueV0;

    #[test]
    fn summarizes_expression_domain_counts() {
        let summary = summarize_expression_domain_plan_input(&sample_input());

        assert_eq!(
            summary.planned_expression_ids,
            vec!["expr-1".to_string(), "expr-2".to_string()]
        );
        assert_eq!(summary.value_domain_kinds.get("constrained"), Some(&1));
        assert_eq!(summary.value_domain_kinds.get("finiteSet"), Some(&1));
        assert_eq!(summary.value_constraint_kinds.get("prefixSuffix"), Some(&1));
        assert_eq!(summary.constraint_detail_counts.prefix_count, 1);
        assert_eq!(summary.constraint_detail_counts.suffix_count, 1);
        assert_eq!(summary.constraint_detail_counts.min_len_count, 1);
        assert_eq!(summary.finite_value_count, 2);
    }

    #[test]
    fn summarizes_expression_domain_fragments() {
        let summary = summarize_expression_domain_fragments_input(&sample_input());

        assert_eq!(summary.fragments.len(), 2);
        let first = &summary.fragments[0];
        assert_eq!(first.expression_id, "expr-1");
        assert_eq!(first.file_path, "/tmp/App.tsx");
        assert_eq!(first.value_domain_kind, "constrained");
        assert_eq!(first.value_constraint_kind.as_deref(), Some("prefixSuffix"));
        assert_eq!(first.value_prefix.as_deref(), Some("btn-"));
        assert_eq!(first.value_suffix.as_deref(), Some("-active"));
        assert_eq!(first.value_min_len, Some(10));
        assert_eq!(first.finite_value_count, 0);

        let second = &summary.fragments[1];
        assert_eq!(second.expression_id, "expr-2");
        assert_eq!(second.value_domain_kind, "finiteSet");
        assert_eq!(second.finite_value_count, 2);
    }

    #[test]
    fn summarizes_expression_domain_candidates() {
        let summary = summarize_expression_domain_candidates_input(&sample_input());

        assert_eq!(summary.candidates.len(), 2);
        assert_eq!(summary.candidates[0].expression_id, "expr-1");
        assert_eq!(summary.candidates[0].value_domain_kind, "constrained");
        assert_eq!(
            summary.candidates[0].value_constraint_kind.as_deref(),
            Some("prefixSuffix")
        );
        assert_eq!(summary.candidates[1].expression_id, "expr-2");
        assert_eq!(summary.candidates[1].finite_value_count, 2);
    }

    #[test]
    fn summarizes_expression_domain_canonical_candidate_bundle() {
        let summary = summarize_expression_domain_canonical_candidate_bundle_input(&sample_input());

        assert_eq!(summary.plan_summary.planned_expression_ids.len(), 2);
        assert_eq!(summary.fragments.len(), 2);
        assert_eq!(summary.candidates.len(), 2);
    }

    #[test]
    fn summarizes_expression_domain_evaluator_candidates() {
        let summary = summarize_expression_domain_evaluator_candidates_input(&sample_input());

        assert_eq!(summary.schema_version, "0");
        assert_eq!(summary.input_version, "2");
        assert_eq!(summary.results.len(), 2);
        assert_eq!(summary.results[0].kind, "expression-domain");
        assert_eq!(summary.results[0].query_id, "expr-1");
        assert_eq!(summary.results[0].payload.value_domain_kind, "prefixSuffix");
        assert_eq!(
            summary.results[0].payload.value_constraint_kind.as_deref(),
            Some("prefixSuffix")
        );
        assert_eq!(summary.results[1].payload.finite_value_count, 2);
    }

    #[test]
    fn summarizes_expression_domain_provenance_explanations() {
        let summary = summarize_expression_domain_provenance_explanations_input(&sample_input());

        assert_eq!(summary.schema_version, "0");
        assert_eq!(
            summary.product,
            "engine-input-producers.expression-domain-provenance-explanations"
        );
        assert_eq!(summary.input_version, "2");
        assert_eq!(summary.explanation_count, 2);
        assert_eq!(summary.explanations[0].expression_id, "expr-1");
        assert_eq!(summary.explanations[0].input_fact_kind, "constrained");
        assert_eq!(
            summary.explanations[0].input_constraint_kind.as_deref(),
            Some("prefixSuffix")
        );
        assert_eq!(summary.explanations[0].reduced_kind, "prefixSuffix");
        assert_eq!(
            summary.explanations[0].derivation.product,
            "omena-abstract-value.reduced-class-value-derivation"
        );
        assert_eq!(
            summary.explanations[0].provenance_tree.product,
            "omena-abstract-value.provenance-tree"
        );
        assert_eq!(
            summary.explanations[0].provenance_tree.root.operation,
            "constraintDomain"
        );
    }

    #[test]
    fn expression_domain_evaluator_reports_reduced_value_domain_kind() {
        let mut input = sample_input();
        input.type_facts.push(TypeFactEntryV2 {
            file_path: "/tmp/App.tsx".to_string(),
            expression_id: "expr-3".to_string(),
            facts: StringTypeFactsV2 {
                kind: "finiteSet".to_string(),
                constraint_kind: Some("prefix".to_string()),
                values: Some(vec!["btn-active".to_string(), "card".to_string()]),
                prefix: Some("btn-".to_string()),
                suffix: None,
                min_len: None,
                max_len: None,
                char_must: None,
                char_may: None,
                may_include_other_chars: None,
                provenance: None,
            },
            control_flow_graph: None,
        });

        let fragments = summarize_expression_domain_fragments_input(&input);
        let candidates = summarize_expression_domain_candidates_input(&input);
        let evaluator_candidates = summarize_expression_domain_evaluator_candidates_input(&input);

        assert_eq!(fragments.fragments[2].expression_id, "expr-3");
        assert_eq!(fragments.fragments[2].value_domain_kind, "finiteSet");
        assert_eq!(candidates.candidates[2].expression_id, "expr-3");
        assert_eq!(candidates.candidates[2].value_domain_kind, "finiteSet");
        assert_eq!(evaluator_candidates.results[2].query_id, "expr-3");
        assert_eq!(
            evaluator_candidates.results[2].payload.value_domain_kind,
            "exact"
        );
        assert_eq!(
            evaluator_candidates.results[2]
                .payload
                .value_domain_derivation
                .reduced_kind,
            "exact"
        );
        assert_eq!(
            evaluator_candidates.results[2]
                .payload
                .value_domain_derivation
                .steps[1]
                .operation,
            "intersectConstraint"
        );
        assert_eq!(
            evaluator_candidates.results[2]
                .payload
                .value_domain_provenance_tree
                .product,
            "omena-abstract-value.provenance-tree"
        );
        assert_eq!(
            evaluator_candidates.results[2]
                .payload
                .value_domain_provenance_tree
                .root
                .operation,
            "exactLiteral"
        );
    }

    #[test]
    fn summarizes_expression_domain_flow_analysis() {
        let mut input = sample_input();
        input.type_facts = vec![
            exact_type_fact("expr-branch-a", "btn-primary"),
            exact_type_fact("expr-branch-b", "btn-secondary"),
            exact_type_fact("expr-branch-c", "card"),
        ];

        let summary = summarize_expression_domain_flow_analysis_input(&input);

        assert_eq!(summary.schema_version, "0");
        assert_eq!(
            summary.product,
            "engine-input-producers.expression-domain-flow-analysis"
        );
        assert_eq!(summary.analyses.len(), 3);
        assert_eq!(summary.analyses[0].file_path, "/tmp/App.tsx");
        assert_eq!(summary.analyses[0].analysis.context_sensitivity, "1-cfa");
        assert!(
            summary
                .analyses
                .iter()
                .all(|entry| entry.analysis.converged)
        );
        assert_eq!(
            summary.analyses[0]
                .analysis
                .nodes
                .iter()
                .find(|node| node.id == "expr-branch-a")
                .map(|node| (node.value_kind, &node.value)),
            Some((
                "exact",
                &AbstractClassValueV0::Exact {
                    value: "btn-primary".to_string()
                }
            ))
        );
        assert!(summary.analyses.iter().all(|entry| {
            entry
                .analysis
                .nodes
                .iter()
                .all(|node| node.id != "file-merge")
        }));
    }

    #[test]
    fn exposes_expression_domain_flow_graphs_for_query_runtime_reuse() {
        let mut input = sample_input();
        input.type_facts = vec![
            exact_type_fact("expr-branch-a", "btn-primary"),
            exact_type_fact("expr-branch-b", "btn-secondary"),
        ];

        let graphs = collect_expression_domain_flow_graphs(&input);

        assert_eq!(graphs.len(), 2);
        assert_eq!(
            graphs
                .iter()
                .map(|entry| entry.graph_id.as_str())
                .collect::<Vec<_>>(),
            vec![
                "/tmp/App.tsx:expr-branch-a:expression-domain-flow",
                "/tmp/App.tsx:expr-branch-b:expression-domain-flow"
            ]
        );
        assert!(graphs.iter().all(|entry| {
            entry.graph.context_key.as_deref() == Some(entry.graph_id.as_str())
                && entry.graph.nodes.iter().all(|node| node.id != "file-merge")
        }));
    }

    #[test]
    fn does_not_synthesize_control_flow_analysis_without_source_cfg() {
        let mut input = sample_input();
        input.type_facts = vec![
            exact_type_fact("expr-branch-a", "btn-primary"),
            exact_type_fact("expr-branch-b", "btn-secondary"),
        ];

        let summary = summarize_expression_domain_control_flow_analysis_input(&input);

        assert_eq!(
            summary.product,
            "engine-input-producers.expression-domain-control-flow-analysis"
        );
        assert!(summary.analyses.is_empty());
    }

    #[test]
    fn consumes_type_fact_control_flow_graph_for_branchy_flow() {
        let mut input = sample_input();
        input.type_facts = vec![TypeFactEntryV2 {
            file_path: "/tmp/App.tsx".to_string(),
            expression_id: "expr-branchy".to_string(),
            facts: StringTypeFactsV2 {
                kind: "exact".to_string(),
                constraint_kind: None,
                values: Some(vec!["btn-primary".to_string()]),
                prefix: None,
                suffix: None,
                min_len: None,
                max_len: None,
                char_must: None,
                char_may: None,
                may_include_other_chars: None,
                provenance: None,
            },
            control_flow_graph: Some(branchy_type_fact_control_flow_graph()),
        }];

        let summary = summarize_expression_domain_control_flow_analysis_input(&input);

        assert_eq!(summary.analyses.len(), 1);
        assert_eq!(
            summary.analyses[0].graph_id,
            "/tmp/App.tsx:expr-branchy:expression-domain-control-flow"
        );
        let analysis = &summary.analyses[0].analysis;
        assert_eq!(analysis.block_count, 6);
        assert_eq!(analysis.edge_count, 6);
        assert_eq!(analysis.branch_block_ids, vec!["branch:0".to_string()]);
        assert_eq!(analysis.join_block_ids, vec!["join:0".to_string()]);
        assert!(
            analysis
                .blocks
                .iter()
                .all(|block| block.block_id != "file-merge")
        );
        assert!(
            analysis
                .blocks
                .iter()
                .find(|block| block.block_id == "branch:0")
                .is_some_and(|block| block.successor_block_ids.len() > 1)
        );
        assert!(analysis
            .flow_analysis
            .nodes
            .iter()
            .any(|node| node.id == "expr-branchy:then:0"
                && node.transfer_kind == "concatFacts"));
    }

    #[test]
    fn control_flow_blocks_prefer_source_frontend_facts_when_present() {
        let mut graph = branchy_type_fact_control_flow_graph();
        if let Some(block) = graph.blocks.iter_mut().find(|block| block.id == "then:0") {
            block.facts = Some(exact_type_fact("expr-block", "btn-secondary").facts);
        }
        let mut input = sample_input();
        input.type_facts = vec![TypeFactEntryV2 {
            file_path: "/tmp/App.tsx".to_string(),
            expression_id: "expr-branchy".to_string(),
            facts: StringTypeFactsV2 {
                kind: "unknown".to_string(),
                constraint_kind: None,
                values: None,
                prefix: None,
                suffix: None,
                min_len: None,
                max_len: None,
                char_must: None,
                char_may: None,
                may_include_other_chars: None,
                provenance: None,
            },
            control_flow_graph: Some(graph),
        }];

        let summary = summarize_expression_domain_control_flow_analysis_input(&input);
        let matching_nodes = summary.analyses[0]
            .analysis
            .flow_analysis
            .nodes
            .iter()
            .filter(|node| node.id == "expr-branchy:then:0")
            .collect::<Vec<_>>();
        assert_eq!(matching_nodes.len(), 1);
        let node = matching_nodes[0];

        assert_eq!(node.transfer_kind, "assignFacts");
        assert_eq!(
            node.value,
            AbstractClassValueV0::Exact {
                value: "btn-secondary".to_string()
            }
        );
    }

    #[test]
    fn summarizes_expression_domain_call_site_flow_analysis_for_zero_and_one_cfa() {
        let mut input = sample_input();
        input.type_facts = vec![
            exact_type_fact_in_file("/tmp/App.tsx", "expr-primary", "btn-primary"),
            exact_type_fact_in_file("/tmp/Card.tsx", "expr-secondary", "btn-secondary"),
        ];

        let summary = summarize_expression_domain_call_site_flow_analysis_input(&input);

        assert_eq!(summary.schema_version, "0");
        assert_eq!(
            summary.product,
            "engine-input-producers.expression-domain-call-site-flow-analysis"
        );
        assert_eq!(summary.zero_cfa.context_sensitivity, "0-cfa");
        assert_eq!(summary.one_cfa.context_sensitivity, "1-cfa");
        assert_eq!(summary.zero_cfa.call_site_count, 2);
        assert_eq!(summary.one_cfa.call_site_count, 2);
        assert_eq!(
            summary.zero_cfa.entries[0].context_key,
            "expression-domain-class-value@<root>"
        );
        assert_eq!(
            summary.zero_cfa.entries[1].context_key,
            "expression-domain-class-value@<root>"
        );
        assert_ne!(
            summary.one_cfa.entries[0].context_key,
            summary.one_cfa.entries[1].context_key
        );
        assert_eq!(
            summary.zero_cfa.entries[0].exit_value,
            AbstractClassValueV0::FiniteSet {
                values: vec!["btn-primary".to_string(), "btn-secondary".to_string()]
            }
        );
        assert_eq!(
            summary.zero_cfa.entries[1].exit_value,
            summary.zero_cfa.entries[0].exit_value
        );
        assert_eq!(
            summary.one_cfa.entries[0].exit_value,
            AbstractClassValueV0::Exact {
                value: "btn-primary".to_string()
            }
        );
        assert_eq!(
            summary.one_cfa.entries[1].exit_value,
            AbstractClassValueV0::Exact {
                value: "btn-secondary".to_string()
            }
        );
    }

    #[test]
    fn summarizes_expression_domain_reduced_product_iteration() {
        let mut input = sample_input();
        input.type_facts = vec![TypeFactEntryV2 {
            file_path: "/tmp/App.tsx".to_string(),
            expression_id: "expr-reduced".to_string(),
            facts: StringTypeFactsV2 {
                kind: "constrained".to_string(),
                constraint_kind: Some("composite".to_string()),
                values: None,
                prefix: Some("btn-".to_string()),
                suffix: Some("-active".to_string()),
                min_len: None,
                max_len: None,
                char_must: Some("a".to_string()),
                char_may: Some("-abceintv".to_string()),
                may_include_other_chars: Some(false),
                provenance: None,
            },
            control_flow_graph: None,
        }];

        let summary = summarize_expression_domain_reduced_product_iteration_input(&input);

        assert_eq!(summary.schema_version, "0");
        assert_eq!(
            summary.product,
            "engine-input-producers.expression-domain-reduced-product-iteration"
        );
        assert_eq!(summary.input_version, "2");
        assert_eq!(summary.iteration_count, 1);
        assert_eq!(summary.iterations[0].expression_id, "expr-reduced");
        assert_eq!(summary.iterations[0].axis_constraint_count, 3);
        assert_eq!(summary.iterations[0].input_value_kind, "composite");
        assert_eq!(summary.iterations[0].iteration.input_count, 3);
        assert_eq!(summary.iterations[0].iteration.applied_constraint_count, 3);
        assert!(summary.iterations[0].iteration.converged);
        assert!(summary.iterations[0].iteration.monotone_witness_valid);
        assert_eq!(summary.iterations[0].iteration.result_kind, "composite");
    }

    #[test]
    fn summarizes_expression_domain_canonical_producer_signal() {
        let summary = summarize_expression_domain_canonical_producer_signal_input(&sample_input());

        assert_eq!(summary.schema_version, "0");
        assert_eq!(summary.input_version, "2");
        assert_eq!(
            summary
                .canonical_bundle
                .plan_summary
                .planned_expression_ids
                .len(),
            2
        );
        assert_eq!(summary.canonical_bundle.fragments.len(), 2);
        assert_eq!(summary.canonical_bundle.candidates.len(), 2);
        assert_eq!(summary.evaluator_candidates.results.len(), 2);
    }

    fn exact_type_fact(expression_id: &str, value: &str) -> TypeFactEntryV2 {
        exact_type_fact_in_file("/tmp/App.tsx", expression_id, value)
    }

    fn exact_type_fact_in_file(
        file_path: &str,
        expression_id: &str,
        value: &str,
    ) -> TypeFactEntryV2 {
        TypeFactEntryV2 {
            file_path: file_path.to_string(),
            expression_id: expression_id.to_string(),
            facts: StringTypeFactsV2 {
                kind: "exact".to_string(),
                constraint_kind: None,
                values: Some(vec![value.to_string()]),
                prefix: None,
                suffix: None,
                min_len: None,
                max_len: None,
                char_must: None,
                char_may: None,
                may_include_other_chars: None,
                provenance: None,
            },
            control_flow_graph: None,
        }
    }

    fn branchy_type_fact_control_flow_graph() -> TypeFactControlFlowGraphV2 {
        TypeFactControlFlowGraphV2 {
            entry_block_id: "entry".to_string(),
            blocks: vec![
                type_fact_control_flow_block("entry", "entry", "entry", &["branch:0"]),
                type_fact_control_flow_block("branch:0", "branch", "branch", &["then:0", "else:0"]),
                type_fact_control_flow_block("then:0", "assignment", "concatFacts", &["join:0"]),
                type_fact_control_flow_block("else:0", "assignment", "assignFacts", &["join:0"]),
                type_fact_control_flow_block("join:0", "join", "join", &["exit"]),
                type_fact_control_flow_block("exit", "exit", "exit", &[]),
            ],
        }
    }

    fn type_fact_control_flow_block(
        id: &str,
        kind: &str,
        transfer_kind: &str,
        successor_block_ids: &[&str],
    ) -> TypeFactControlFlowBlockV2 {
        TypeFactControlFlowBlockV2 {
            id: id.to_string(),
            kind: kind.to_string(),
            transfer_kind: transfer_kind.to_string(),
            successor_block_ids: successor_block_ids
                .iter()
                .map(|id| (*id).to_string())
                .collect(),
            symbol_ordinal: None,
            variable_name: None,
            expression_kind: None,
            facts: None,
        }
    }
}
