use std::collections::BTreeMap;

use crate::{
    ConstraintDetailCounts, ConstraintDetailInput, EngineInputV2, ExpressionDomainCandidateV0,
    ExpressionDomainCandidatesV0, ExpressionDomainCanonicalCandidateBundleV0,
    ExpressionDomainCanonicalProducerSignalV0, ExpressionDomainControlFlowAnalysisEntryV0,
    ExpressionDomainControlFlowAnalysisV0, ExpressionDomainEvaluatorCandidatePayloadV0,
    ExpressionDomainEvaluatorCandidateV0, ExpressionDomainEvaluatorCandidatesV0,
    ExpressionDomainFlowAnalysisEntryV0, ExpressionDomainFlowAnalysisV0,
    ExpressionDomainFlowGraphEntryV0, ExpressionDomainFragmentV0, ExpressionDomainFragmentsV0,
    ExpressionDomainPlanSummaryV0, TypeFactEntryV2, abstract_value_facts,
    collect_constraint_detail_counts, map_reduced_expression_value_domain_derivation,
    map_reduced_expression_value_domain_kind,
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
    let analyses = collect_expression_domain_flow_graphs(input)
        .into_iter()
        .map(|entry| {
            let cfg = expression_domain_control_flow_graph(&entry.graph);
            ExpressionDomainControlFlowAnalysisEntryV0 {
                graph_id: entry.graph_id,
                file_path: entry.file_path,
                analysis: omena_abstract_value::analyze_class_value_control_flow_graph(&cfg),
            }
        })
        .collect();

    ExpressionDomainControlFlowAnalysisV0 {
        schema_version: "0",
        product: "engine-input-producers.expression-domain-control-flow-analysis",
        input_version: input.version.clone(),
        analyses,
    }
}

pub fn collect_expression_domain_flow_graphs(
    input: &EngineInputV2,
) -> Vec<ExpressionDomainFlowGraphEntryV0> {
    let mut by_file = BTreeMap::<String, Vec<&TypeFactEntryV2>>::new();
    for entry in &input.type_facts {
        by_file
            .entry(entry.file_path.clone())
            .or_default()
            .push(entry);
    }

    by_file
        .into_iter()
        .map(|(file_path, mut entries)| {
            entries.sort_by(|a, b| a.expression_id.cmp(&b.expression_id));
            let graph_id = format!("{file_path}:expression-domain-flow");
            let mut nodes = entries
                .iter()
                .map(|entry| omena_abstract_value::ClassValueFlowNodeV0 {
                    id: entry.expression_id.clone(),
                    predecessors: Vec::new(),
                    transfer: omena_abstract_value::ClassValueFlowTransferV0::AssignFacts(
                        abstract_value_facts(&entry.facts),
                    ),
                })
                .collect::<Vec<_>>();

            if entries.len() > 1 {
                nodes.push(omena_abstract_value::ClassValueFlowNodeV0 {
                    id: "file-merge".to_string(),
                    predecessors: entries
                        .iter()
                        .map(|entry| entry.expression_id.clone())
                        .collect(),
                    transfer: omena_abstract_value::ClassValueFlowTransferV0::Join,
                });
            }

            let graph = omena_abstract_value::ClassValueFlowGraphV0 {
                context_key: Some(graph_id.clone()),
                nodes,
            };

            ExpressionDomainFlowGraphEntryV0 {
                graph_id,
                file_path,
                graph,
            }
        })
        .collect()
}

fn expression_domain_control_flow_graph(
    graph: &omena_abstract_value::ClassValueFlowGraphV0,
) -> omena_abstract_value::ClassValueControlFlowGraphV0 {
    let merge_node_id = "file-merge";
    let has_merge = graph.nodes.iter().any(|node| node.id == merge_node_id);
    let mut blocks = Vec::new();

    if has_merge {
        blocks.push(omena_abstract_value::ClassValueControlFlowBlockV0 {
            id: "entry".to_string(),
            nodes: Vec::new(),
            successor_block_ids: graph
                .nodes
                .iter()
                .filter(|node| node.id != merge_node_id)
                .map(|node| format!("expr:{}", node.id))
                .collect(),
        });

        for node in graph.nodes.iter().filter(|node| node.id != merge_node_id) {
            blocks.push(omena_abstract_value::ClassValueControlFlowBlockV0 {
                id: format!("expr:{}", node.id),
                nodes: vec![node.clone()],
                successor_block_ids: vec!["merge".to_string()],
            });
        }

        if let Some(merge) = graph.nodes.iter().find(|node| node.id == merge_node_id) {
            blocks.push(omena_abstract_value::ClassValueControlFlowBlockV0 {
                id: "merge".to_string(),
                nodes: vec![merge.clone()],
                successor_block_ids: Vec::new(),
            });
        }
    } else {
        blocks.extend(graph.nodes.iter().map(|node| {
            omena_abstract_value::ClassValueControlFlowBlockV0 {
                id: format!("expr:{}", node.id),
                nodes: vec![node.clone()],
                successor_block_ids: Vec::new(),
            }
        }));
    }

    let entry_block_id = blocks
        .first()
        .map(|block| block.id.clone())
        .unwrap_or_else(|| "entry".to_string());

    omena_abstract_value::ClassValueControlFlowGraphV0 {
        context_key: graph.context_key.clone(),
        entry_block_id,
        blocks,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        collect_expression_domain_flow_graphs, summarize_expression_domain_candidates_input,
        summarize_expression_domain_canonical_candidate_bundle_input,
        summarize_expression_domain_canonical_producer_signal_input,
        summarize_expression_domain_control_flow_analysis_input,
        summarize_expression_domain_evaluator_candidates_input,
        summarize_expression_domain_flow_analysis_input,
        summarize_expression_domain_fragments_input, summarize_expression_domain_plan_input,
    };
    use crate::{StringTypeFactsV2, TypeFactEntryV2, test_support::sample_input};
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
            },
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
        assert_eq!(summary.analyses.len(), 1);
        assert_eq!(summary.analyses[0].file_path, "/tmp/App.tsx");
        assert_eq!(summary.analyses[0].analysis.context_sensitivity, "1-cfa");
        assert!(summary.analyses[0].analysis.converged);
        assert_eq!(
            summary.analyses[0]
                .analysis
                .nodes
                .iter()
                .find(|node| node.id == "file-merge")
                .map(|node| (node.value_kind, &node.value)),
            Some((
                "finiteSet",
                &AbstractClassValueV0::FiniteSet {
                    values: vec![
                        "btn-primary".to_string(),
                        "btn-secondary".to_string(),
                        "card".to_string(),
                    ]
                }
            ))
        );
    }

    #[test]
    fn exposes_expression_domain_flow_graphs_for_query_runtime_reuse() {
        let mut input = sample_input();
        input.type_facts = vec![
            exact_type_fact("expr-branch-a", "btn-primary"),
            exact_type_fact("expr-branch-b", "btn-secondary"),
        ];

        let graphs = collect_expression_domain_flow_graphs(&input);

        assert_eq!(graphs.len(), 1);
        assert_eq!(graphs[0].graph_id, "/tmp/App.tsx:expression-domain-flow");
        assert_eq!(
            graphs[0].graph.context_key.as_deref(),
            Some(graphs[0].graph_id.as_str())
        );
        assert!(
            graphs[0]
                .graph
                .nodes
                .iter()
                .any(|node| node.id == "file-merge")
        );
    }

    #[test]
    fn summarizes_expression_domain_control_flow_analysis() {
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
        assert_eq!(summary.analyses.len(), 1);
        assert_eq!(summary.analyses[0].analysis.block_count, 4);
        assert_eq!(summary.analyses[0].analysis.edge_count, 4);
        assert_eq!(
            summary.analyses[0].analysis.flow_analysis.product,
            "omena-abstract-value.flow-analysis"
        );
        assert!(
            summary.analyses[0]
                .analysis
                .unreachable_block_ids
                .is_empty()
        );
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
        TypeFactEntryV2 {
            file_path: "/tmp/App.tsx".to_string(),
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
            },
        }
    }
}
