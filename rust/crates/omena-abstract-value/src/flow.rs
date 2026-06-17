use std::collections::{BTreeMap, BTreeSet};

use omena_incremental::{
    IncrementalGraphInputV0, IncrementalNodeInputV0, IncrementalRevisionV0, IncrementalSnapshotV0,
    OmenaIncrementalDatabaseV0, plan_incremental_computation, snapshot_from_graph_input,
};

use crate::*;

pub fn summarize_omena_abstract_value_flow_analysis() -> AbstractValueFlowAnalysisSummaryV0 {
    AbstractValueFlowAnalysisSummaryV0 {
        schema_version: "0",
        product: "omena-abstract-value.flow-analysis",
        context_sensitivity: "1-cfa",
        incremental_engine: "omena-incremental",
        analysis_scopes: vec![
            "singleContext",
            "multiContextBatch",
            "callSiteBatch",
            "zeroCfaCallSiteBatch",
            "kLimitedCallSiteBatch",
            "controlFlowGraph",
        ],
        reuse_policy: "reuse previous context analysis when its omena-incremental plan is clean",
        transfer_kinds: vec!["assignFacts", "refineFacts", "concatFacts", "join"],
        max_iterations: MAX_FLOW_ANALYSIS_ITERATIONS,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedJoinFixpointNodeV0<TTransfer> {
    pub id: String,
    pub predecessor_ids: Vec<String>,
    pub transfer: TTransfer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedJoinFixpointNodeResultV0<TValue> {
    pub id: String,
    pub input_value: TValue,
    pub output_value: TValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedJoinFixpointResultV0<TValue> {
    pub converged: bool,
    pub iteration_count: usize,
    pub nodes: Vec<BoundedJoinFixpointNodeResultV0<TValue>>,
}

pub fn analyze_bounded_join_fixpoint<TValue, TTransfer>(
    nodes: &[BoundedJoinFixpointNodeV0<TTransfer>],
    max_iterations: usize,
    bottom_value: TValue,
    top_value: TValue,
    mut join_values: impl FnMut(&TValue, &TValue) -> TValue,
    mut apply_transfer: impl FnMut(&TValue, &TTransfer) -> TValue,
) -> BoundedJoinFixpointResultV0<TValue>
where
    TValue: Clone + PartialEq,
{
    let mut input_values = nodes
        .iter()
        .map(|node| (node.id.clone(), bottom_value.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut output_values = input_values.clone();
    let mut converged = nodes.is_empty();
    let mut iteration_count = 0usize;

    for iteration in 1..=max_iterations {
        iteration_count = iteration;
        let mut changed = false;

        for node in nodes {
            let input_value = node
                .predecessor_ids
                .iter()
                .map(|id| {
                    output_values
                        .get(id)
                        .cloned()
                        .unwrap_or_else(|| top_value.clone())
                })
                .reduce(|left, right| join_values(&left, &right))
                .unwrap_or_else(|| bottom_value.clone());
            let output_value = apply_transfer(&input_value, &node.transfer);

            if input_values.get(&node.id) != Some(&input_value) {
                input_values.insert(node.id.clone(), input_value);
                changed = true;
            }
            if output_values.get(&node.id) != Some(&output_value) {
                output_values.insert(node.id.clone(), output_value);
                changed = true;
            }
        }

        if !changed {
            converged = true;
            break;
        }
    }

    BoundedJoinFixpointResultV0 {
        converged,
        iteration_count,
        nodes: nodes
            .iter()
            .map(|node| BoundedJoinFixpointNodeResultV0 {
                id: node.id.clone(),
                input_value: input_values
                    .get(&node.id)
                    .cloned()
                    .unwrap_or_else(|| bottom_value.clone()),
                output_value: output_values
                    .get(&node.id)
                    .cloned()
                    .unwrap_or_else(|| bottom_value.clone()),
            })
            .collect(),
    }
}

pub fn analyze_class_value_flow(graph: &ClassValueFlowGraphV0) -> ClassValueFlowAnalysisV0 {
    let mut values = graph
        .nodes
        .iter()
        .map(|node| (node.id.clone(), bottom_class_value()))
        .collect::<BTreeMap<_, _>>();
    let mut converged = false;
    let mut iteration_count = 0;

    for iteration in 1..=MAX_FLOW_ANALYSIS_ITERATIONS {
        iteration_count = iteration;
        let mut changed = false;

        for node in &graph.nodes {
            let incoming = join_predecessor_flow_values(node, &values);
            let next = apply_flow_transfer(&incoming, &node.transfer);

            if values.get(&node.id) != Some(&next) {
                values.insert(node.id.clone(), next);
                changed = true;
            }
        }

        if !changed {
            converged = true;
            break;
        }
    }

    ClassValueFlowAnalysisV0 {
        schema_version: "0",
        product: "omena-abstract-value.flow-analysis",
        context_sensitivity: "1-cfa",
        context_key: graph.context_key.clone(),
        converged,
        iteration_count,
        nodes: graph
            .nodes
            .iter()
            .map(|node| {
                let value = values
                    .get(&node.id)
                    .cloned()
                    .unwrap_or_else(bottom_class_value);
                ClassValueFlowNodeResultV0 {
                    id: node.id.clone(),
                    predecessor_ids: node.predecessors.clone(),
                    transfer_kind: flow_transfer_kind(&node.transfer),
                    value_kind: abstract_class_value_kind(&value),
                    value,
                }
            })
            .collect(),
    }
}

pub fn analyze_class_value_control_flow_graph(
    graph: &ClassValueControlFlowGraphV0,
) -> ClassValueControlFlowAnalysisV0 {
    let reachable_block_ids = reachable_control_flow_block_ids(graph);
    let reachable_node_ids = graph
        .blocks
        .iter()
        .filter(|block| reachable_block_ids.contains(&block.id))
        .flat_map(|block| block.nodes.iter().map(|node| node.id.clone()))
        .collect::<BTreeSet<_>>();
    let flow_graph = ClassValueFlowGraphV0 {
        context_key: graph.context_key.clone(),
        nodes: graph
            .blocks
            .iter()
            .filter(|block| reachable_block_ids.contains(&block.id))
            .flat_map(|block| {
                block.nodes.iter().map(|node| ClassValueFlowNodeV0 {
                    id: node.id.clone(),
                    predecessors: node
                        .predecessors
                        .iter()
                        .filter(|id| reachable_node_ids.contains(id.as_str()))
                        .cloned()
                        .collect(),
                    transfer: node.transfer.clone(),
                })
            })
            .collect(),
    };
    let flow_analysis = analyze_class_value_flow(&flow_graph);
    let unreachable_block_ids = graph
        .blocks
        .iter()
        .filter(|block| !reachable_block_ids.contains(&block.id))
        .map(|block| block.id.clone())
        .collect::<Vec<_>>();
    let branch_block_ids = graph
        .blocks
        .iter()
        .filter(|block| block.successor_block_ids.len() > 1)
        .map(|block| block.id.clone())
        .collect::<Vec<_>>();
    let predecessor_counts = control_flow_predecessor_counts(graph);
    let join_block_ids = graph
        .blocks
        .iter()
        .filter(|block| predecessor_counts.get(&block.id).copied().unwrap_or(0) > 1)
        .map(|block| block.id.clone())
        .collect::<Vec<_>>();
    let blocks = graph
        .blocks
        .iter()
        .map(|block| {
            let reachable = reachable_block_ids.contains(&block.id);
            let exit_value = if reachable {
                block
                    .nodes
                    .iter()
                    .rev()
                    .find_map(|node| flow_analysis_node_value(&flow_analysis, &node.id))
                    .cloned()
                    .unwrap_or_else(bottom_class_value)
            } else {
                bottom_class_value()
            };

            ClassValueControlFlowBlockResultV0 {
                block_id: block.id.clone(),
                reachable,
                node_ids: block.nodes.iter().map(|node| node.id.clone()).collect(),
                successor_block_ids: block.successor_block_ids.clone(),
                exit_value_kind: abstract_class_value_kind(&exit_value),
                exit_value,
            }
        })
        .collect::<Vec<_>>();

    ClassValueControlFlowAnalysisV0 {
        schema_version: "0",
        product: "omena-abstract-value.control-flow-analysis",
        context_sensitivity: "1-cfa",
        context_key: graph.context_key.clone(),
        block_count: graph.blocks.len(),
        edge_count: graph
            .blocks
            .iter()
            .map(|block| block.successor_block_ids.len())
            .sum(),
        reachable_block_count: reachable_block_ids.len(),
        unreachable_block_ids,
        branch_block_ids,
        join_block_ids,
        flow_analysis,
        blocks,
    }
}

pub fn analyze_class_value_flow_incremental(
    graph: &ClassValueFlowGraphV0,
    previous_snapshot: Option<&IncrementalSnapshotV0>,
    revision: u64,
) -> ClassValueFlowIncrementalAnalysisV0 {
    analyze_class_value_flow_incremental_with_reuse(graph, previous_snapshot, None, revision)
}

pub fn analyze_class_value_flow_incremental_with_reuse(
    graph: &ClassValueFlowGraphV0,
    previous_snapshot: Option<&IncrementalSnapshotV0>,
    previous_analysis: Option<&ClassValueFlowAnalysisV0>,
    revision: u64,
) -> ClassValueFlowIncrementalAnalysisV0 {
    let incremental_input = class_value_flow_incremental_input(graph, revision);
    let incremental_plan = plan_incremental_computation(&incremental_input, previous_snapshot);
    let next_snapshot = snapshot_from_graph_input(&incremental_input);
    let reused_previous_analysis =
        incremental_plan.dirty_node_count == 0 && previous_analysis.is_some();
    let analysis = match (incremental_plan.dirty_node_count, previous_analysis) {
        (0, Some(previous_analysis)) => previous_analysis.clone(),
        _ => analyze_class_value_flow(graph),
    };

    ClassValueFlowIncrementalAnalysisV0 {
        schema_version: "0",
        product: "omena-abstract-value.incremental-flow-analysis",
        reused_previous_analysis,
        incremental_plan,
        next_snapshot,
        analysis,
    }
}

pub fn analyze_class_value_flow_incremental_with_database(
    graph: &ClassValueFlowGraphV0,
    incremental_database: &mut OmenaIncrementalDatabaseV0,
    previous_analysis: Option<&ClassValueFlowAnalysisV0>,
    revision: u64,
) -> ClassValueFlowIncrementalAnalysisV0 {
    let incremental_input = class_value_flow_incremental_input(graph, revision);
    let update = incremental_database.plan_and_upsert_graph_input(&incremental_input);
    let reused_previous_analysis =
        update.incremental_plan.dirty_node_count == 0 && previous_analysis.is_some();
    let analysis = match (update.incremental_plan.dirty_node_count, previous_analysis) {
        (0, Some(previous_analysis)) => previous_analysis.clone(),
        _ => analyze_class_value_flow(graph),
    };

    ClassValueFlowIncrementalAnalysisV0 {
        schema_version: "0",
        product: "omena-abstract-value.incremental-flow-analysis",
        reused_previous_analysis,
        incremental_plan: update.incremental_plan,
        next_snapshot: update.next_snapshot,
        analysis,
    }
}

pub fn analyze_class_value_flow_incremental_batch_with_reuse(
    graphs: &[ClassValueFlowGraphV0],
    previous_snapshots: &BTreeMap<String, IncrementalSnapshotV0>,
    previous_analyses: &BTreeMap<String, ClassValueFlowAnalysisV0>,
    revision: u64,
) -> ClassValueFlowIncrementalBatchAnalysisV0 {
    let entries = graphs
        .iter()
        .enumerate()
        .map(|(index, graph)| {
            let context_key = flow_graph_batch_context_key(graph, index);
            let analysis = analyze_class_value_flow_incremental_with_reuse(
                graph,
                previous_snapshots.get(context_key.as_str()),
                previous_analyses.get(context_key.as_str()),
                revision,
            );
            ClassValueFlowIncrementalBatchEntryV0 {
                context_key,
                analysis,
            }
        })
        .collect::<Vec<_>>();
    let reused_context_count = entries
        .iter()
        .filter(|entry| entry.analysis.reused_previous_analysis)
        .count();
    let dirty_context_count = entries
        .iter()
        .filter(|entry| entry.analysis.incremental_plan.dirty_node_count > 0)
        .count();

    ClassValueFlowIncrementalBatchAnalysisV0 {
        schema_version: "0",
        product: "omena-abstract-value.incremental-flow-analysis-batch",
        revision,
        context_count: entries.len(),
        dirty_context_count,
        reused_context_count,
        entries,
    }
}

pub fn analyze_one_cfa_call_site_flows(
    inputs: &[OneCfaCallSiteFlowInputV0],
) -> OneCfaCallSiteFlowAnalysisV0 {
    let entries = inputs
        .iter()
        .map(|input| {
            let context_key = one_cfa_context_key(input);
            let mut graph = input.graph.clone();
            graph.context_key = Some(context_key.clone());
            let analysis = analyze_class_value_flow(&graph);
            let exit_value = flow_analysis_node_value(&analysis, &input.exit_node_id)
                .cloned()
                .unwrap_or_else(bottom_class_value);
            let exit_value_kind = abstract_class_value_kind(&exit_value);

            OneCfaCallSiteFlowEntryV0 {
                callee_key: input.callee_key.clone(),
                call_site_id: input.call_site_id.clone(),
                context_key: context_key.clone(),
                exit_node_id: input.exit_node_id.clone(),
                exit_value_kind,
                exit_value: exit_value.clone(),
                analysis,
                derivation: one_cfa_call_site_derivation(input, &context_key, &exit_value),
            }
        })
        .collect::<Vec<_>>();
    let callee_summaries = summarize_one_cfa_callees(&entries);

    OneCfaCallSiteFlowAnalysisV0 {
        schema_version: "0",
        product: "omena-abstract-value.one-cfa-call-site-flow",
        context_sensitivity: "1-cfa",
        call_site_count: entries.len(),
        callee_count: callee_summaries.len(),
        entries,
        callee_summaries,
    }
}

pub fn analyze_k_limited_call_site_flows(
    inputs: &[KLimitedCallSiteFlowInputV0],
    max_context_depth: usize,
) -> KLimitedCallSiteFlowAnalysisV0 {
    let mut entries = inputs
        .iter()
        .map(|input| {
            let context_key = k_limited_context_key(input, max_context_depth);
            let mut graph = input.graph.clone();
            graph.context_key = Some(context_key.clone());
            let analysis = analyze_class_value_flow(&graph);
            let exit_value = flow_analysis_node_value(&analysis, &input.exit_node_id)
                .cloned()
                .unwrap_or_else(bottom_class_value);
            let exit_value_kind = abstract_class_value_kind(&exit_value);

            KLimitedCallSiteFlowEntryV0 {
                callee_key: input.callee_key.clone(),
                call_site_stack: input.call_site_stack.clone(),
                context_key,
                exit_node_id: input.exit_node_id.clone(),
                exit_value_kind,
                exit_value,
                analysis,
            }
        })
        .collect::<Vec<_>>();
    let joined_exit_values_by_context = entries.iter().fold(
        BTreeMap::<String, AbstractClassValueV0>::new(),
        |mut by_context, entry| {
            by_context
                .entry(entry.context_key.clone())
                .and_modify(|value| {
                    *value = join_abstract_class_values(value, &entry.exit_value);
                })
                .or_insert_with(|| entry.exit_value.clone());
            by_context
        },
    );
    for entry in &mut entries {
        if let Some(joined_exit_value) = joined_exit_values_by_context.get(&entry.context_key) {
            entry.exit_value = joined_exit_value.clone();
            entry.exit_value_kind = abstract_class_value_kind(&entry.exit_value);
        }
    }
    let callee_summaries = summarize_k_limited_callees(&entries);

    KLimitedCallSiteFlowAnalysisV0 {
        schema_version: "0",
        product: "omena-abstract-value.k-limited-call-site-flow",
        context_sensitivity: format!("{max_context_depth}-cfa"),
        max_context_depth,
        call_site_count: entries.len(),
        callee_count: callee_summaries.len(),
        entries,
        callee_summaries,
    }
}

pub fn class_value_flow_incremental_input(
    graph: &ClassValueFlowGraphV0,
    revision: u64,
) -> IncrementalGraphInputV0 {
    IncrementalGraphInputV0 {
        revision: IncrementalRevisionV0 { value: revision },
        nodes: graph
            .nodes
            .iter()
            .map(|node| IncrementalNodeInputV0 {
                id: node.id.clone(),
                digest: flow_node_incremental_digest(node),
                dependency_ids: node.predecessors.clone(),
            })
            .collect(),
    }
}

fn flow_graph_batch_context_key(graph: &ClassValueFlowGraphV0, index: usize) -> String {
    graph
        .context_key
        .clone()
        .unwrap_or_else(|| format!("anonymous-context-{index}"))
}

fn one_cfa_context_key(input: &OneCfaCallSiteFlowInputV0) -> String {
    format!("{}@{}", input.callee_key, input.call_site_id)
}

fn k_limited_context_key(input: &KLimitedCallSiteFlowInputV0, max_context_depth: usize) -> String {
    let retained_stack = input
        .call_site_stack
        .iter()
        .rev()
        .take(max_context_depth)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>();
    let stack = if retained_stack.is_empty() {
        "<root>".to_string()
    } else {
        retained_stack.join(" > ")
    };

    format!("{}@{}", input.callee_key, stack)
}

fn reachable_control_flow_block_ids(graph: &ClassValueControlFlowGraphV0) -> BTreeSet<String> {
    let blocks_by_id = graph
        .blocks
        .iter()
        .map(|block| (block.id.as_str(), block))
        .collect::<BTreeMap<_, _>>();
    let mut reachable = BTreeSet::new();
    let mut worklist = vec![graph.entry_block_id.clone()];

    while let Some(block_id) = worklist.pop() {
        if !reachable.insert(block_id.clone()) {
            continue;
        }
        let Some(block) = blocks_by_id.get(block_id.as_str()) else {
            continue;
        };
        worklist.extend(block.successor_block_ids.iter().cloned());
    }

    reachable
}

fn control_flow_predecessor_counts(
    graph: &ClassValueControlFlowGraphV0,
) -> BTreeMap<String, usize> {
    let mut counts = graph
        .blocks
        .iter()
        .map(|block| (block.id.clone(), 0usize))
        .collect::<BTreeMap<_, _>>();

    for block in &graph.blocks {
        for successor_id in &block.successor_block_ids {
            *counts.entry(successor_id.clone()).or_default() += 1;
        }
    }

    counts
}

fn flow_analysis_node_value<'a>(
    analysis: &'a ClassValueFlowAnalysisV0,
    node_id: &str,
) -> Option<&'a AbstractClassValueV0> {
    analysis
        .nodes
        .iter()
        .find(|node| node.id == node_id)
        .map(|node| &node.value)
}

fn one_cfa_call_site_derivation(
    input: &OneCfaCallSiteFlowInputV0,
    context_key: &str,
    exit_value: &AbstractClassValueV0,
) -> OneCfaCallSiteDerivationV0 {
    OneCfaCallSiteDerivationV0 {
        schema_version: "0",
        product: "omena-abstract-value.one-cfa-call-site-derivation",
        call_site_id: input.call_site_id.clone(),
        context_key: context_key.to_string(),
        steps: vec![
            OneCfaCallSiteDerivationStepV0 {
                operation: "contextFromCallSite",
                result_kind: "context",
                reason: "1-CFA separates flow facts by the immediate call-site identity",
            },
            OneCfaCallSiteDerivationStepV0 {
                operation: "analyzeFlowGraph",
                result_kind: "flowAnalysis",
                reason: "ran the class-value flow graph inside the call-site context",
            },
            OneCfaCallSiteDerivationStepV0 {
                operation: "projectExitNode",
                result_kind: abstract_class_value_kind(exit_value),
                reason: "projected the requested exit node as the call-site result",
            },
        ],
    }
}

fn summarize_one_cfa_callees(
    entries: &[OneCfaCallSiteFlowEntryV0],
) -> Vec<OneCfaCalleeFlowSummaryV0> {
    let mut by_callee = BTreeMap::<String, Vec<AbstractClassValueV0>>::new();
    for entry in entries {
        by_callee
            .entry(entry.callee_key.clone())
            .or_default()
            .push(entry.exit_value.clone());
    }

    by_callee
        .into_iter()
        .map(|(callee_key, values)| {
            let call_site_count = values.len();
            let joined_exit_value = values
                .into_iter()
                .reduce(|left, right| join_abstract_class_values(&left, &right))
                .unwrap_or_else(bottom_class_value);
            OneCfaCalleeFlowSummaryV0 {
                callee_key,
                call_site_count,
                joined_exit_value_kind: abstract_class_value_kind(&joined_exit_value),
                joined_exit_value,
            }
        })
        .collect()
}

fn summarize_k_limited_callees(
    entries: &[KLimitedCallSiteFlowEntryV0],
) -> Vec<OneCfaCalleeFlowSummaryV0> {
    let mut by_callee = BTreeMap::<String, Vec<AbstractClassValueV0>>::new();
    for entry in entries {
        by_callee
            .entry(entry.callee_key.clone())
            .or_default()
            .push(entry.exit_value.clone());
    }

    by_callee
        .into_iter()
        .map(|(callee_key, values)| {
            let call_site_count = values.len();
            let joined_exit_value = values
                .into_iter()
                .reduce(|left, right| join_abstract_class_values(&left, &right))
                .unwrap_or_else(bottom_class_value);
            OneCfaCalleeFlowSummaryV0 {
                callee_key,
                call_site_count,
                joined_exit_value_kind: abstract_class_value_kind(&joined_exit_value),
                joined_exit_value,
            }
        })
        .collect()
}

fn join_predecessor_flow_values(
    node: &ClassValueFlowNodeV0,
    values: &BTreeMap<String, AbstractClassValueV0>,
) -> AbstractClassValueV0 {
    node.predecessors
        .iter()
        .map(|id| values.get(id).cloned().unwrap_or_else(top_class_value))
        .reduce(|left, right| join_abstract_class_values(&left, &right))
        .unwrap_or_else(bottom_class_value)
}

fn apply_flow_transfer(
    incoming: &AbstractClassValueV0,
    transfer: &ClassValueFlowTransferV0,
) -> AbstractClassValueV0 {
    match transfer {
        ClassValueFlowTransferV0::AssignFacts(facts) => {
            reduced_abstract_class_value_from_facts(facts)
        }
        ClassValueFlowTransferV0::RefineFacts(facts) => {
            let refinement = reduced_abstract_class_value_from_facts(facts);
            intersect_abstract_class_values(incoming, &refinement)
        }
        ClassValueFlowTransferV0::ConcatFacts(facts) => {
            let right = reduced_abstract_class_value_from_facts(facts);
            concatenate_abstract_class_values(incoming, &right)
        }
        ClassValueFlowTransferV0::Join => incoming.clone(),
    }
}

fn flow_transfer_kind(transfer: &ClassValueFlowTransferV0) -> &'static str {
    match transfer {
        ClassValueFlowTransferV0::AssignFacts(_) => "assignFacts",
        ClassValueFlowTransferV0::RefineFacts(_) => "refineFacts",
        ClassValueFlowTransferV0::ConcatFacts(_) => "concatFacts",
        ClassValueFlowTransferV0::Join => "join",
    }
}

fn flow_node_incremental_digest(node: &ClassValueFlowNodeV0) -> String {
    let mut parts = vec![
        format!("id={}", node.id),
        format!("deps={}", node.predecessors.join(",")),
        format!("transfer={}", flow_transfer_kind(&node.transfer)),
    ];

    match &node.transfer {
        ClassValueFlowTransferV0::AssignFacts(facts)
        | ClassValueFlowTransferV0::RefineFacts(facts)
        | ClassValueFlowTransferV0::ConcatFacts(facts) => {
            push_external_facts_digest_parts(&mut parts, facts);
        }
        ClassValueFlowTransferV0::Join => {}
    }

    parts.join(";")
}

fn push_external_facts_digest_parts(parts: &mut Vec<String>, facts: &ExternalStringTypeFactsV0) {
    parts.push(format!("kind={}", facts.kind));
    parts.push(format!(
        "constraint={}",
        facts.constraint_kind.as_deref().unwrap_or("")
    ));
    parts.push(format!(
        "values={}",
        facts.values.as_ref().map_or_else(String::new, |values| {
            let mut values = values.clone();
            values.sort();
            values.dedup();
            values.join(",")
        })
    ));
    parts.push(format!("prefix={}", facts.prefix.as_deref().unwrap_or("")));
    parts.push(format!("suffix={}", facts.suffix.as_deref().unwrap_or("")));
    parts.push(format!(
        "minLen={}",
        facts
            .min_len
            .map_or_else(String::new, |value| value.to_string())
    ));
    parts.push(format!(
        "maxLen={}",
        facts
            .max_len
            .map_or_else(String::new, |value| value.to_string())
    ));
    parts.push(format!(
        "charMust={}",
        facts.char_must.as_deref().unwrap_or("")
    ));
    parts.push(format!(
        "charMay={}",
        facts.char_may.as_deref().unwrap_or("")
    ));
    parts.push(format!(
        "mayOther={}",
        facts
            .may_include_other_chars
            .map_or_else(String::new, |value| value.to_string())
    ));
}
