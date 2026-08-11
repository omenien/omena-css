use std::collections::{BTreeMap, BTreeSet, VecDeque};

use omena_incremental::{
    IncrementalGraphInputV0, IncrementalNodeInputV0, IncrementalRevisionV0, IncrementalSnapshotV0,
    OmenaIncrementalDatabaseV0,
};
use serde::Serialize;

use crate::automaton::finite_language_values;
use crate::*;

const CLASS_VALUE_FLOW_SOLVER_REVISION_V0: &str = "iterativeRpoLoopHeaderPrefixWideningNarrowingV1";
const MAX_FLOW_NARROWING_ROUNDS: usize = MAX_FINITE_CLASS_VALUES;

/// Summarizes analyses over caller-supplied graphs.
///
/// Call-site-aware entry points are listed as separate scopes and retain their
/// own context labels.
pub fn summarize_omena_abstract_value_flow_analysis() -> AbstractValueFlowAnalysisSummaryV0 {
    AbstractValueFlowAnalysisSummaryV0 {
        schema_version: "0",
        product: "omena-abstract-value.flow-analysis",
        context_sensitivity: "perSuppliedGraph",
        incremental_engine: "omena-incremental",
        analysis_scopes: vec![
            "singleContext",
            "multiContextBatch",
            "callSiteBatch",
            "zeroCfaCallSiteBatch",
            "kLimitedCallSiteBatch",
            "controlFlowGraph",
        ],
        reuse_policy: "reuse previous context analysis only when its omena-incremental plan is clean and deterministic fresh-equivalence verification matches",
        transfer_kinds: vec!["assignFacts", "refineFacts", "concatFacts", "join"],
        max_iterations: MAX_FLOW_ANALYSIS_ITERATIONS,
    }
}

/// Minimal explicit-edge graph shape used by reachability helpers.
pub trait ControlFlowEdgeGraphV0 {
    type BlockId: Clone + Ord;

    fn entry_block_id(&self) -> Option<Self::BlockId>;

    fn successor_block_ids_by_source(&self) -> Vec<(Self::BlockId, Vec<Self::BlockId>)>;
}

impl ControlFlowEdgeGraphV0 for ClassValueControlFlowGraphV0 {
    type BlockId = String;

    fn entry_block_id(&self) -> Option<Self::BlockId> {
        Some(self.entry_block_id.clone())
    }

    fn successor_block_ids_by_source(&self) -> Vec<(Self::BlockId, Vec<Self::BlockId>)> {
        self.blocks
            .iter()
            .map(|block| (block.id.clone(), block.successor_block_ids.clone()))
            .collect()
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct FlowNodeTopology {
    node_ids: Vec<String>,
    has_unresolved_predecessor: Vec<bool>,
    predecessors: Vec<Vec<usize>>,
    successors: Vec<Vec<usize>>,
}

fn assert_unique_flow_node_ids(node_ids: &[String]) {
    let mut seen = BTreeSet::new();
    for id in node_ids {
        assert!(
            seen.insert(id.as_str()),
            "flow graph node ids must be unique; duplicate id {id:?}"
        );
    }
}

fn flow_node_topology(node_ids: &[String], predecessor_ids: &[Vec<String>]) -> FlowNodeTopology {
    assert_unique_flow_node_ids(node_ids);
    let mut index_by_id = BTreeMap::new();
    for (index, id) in node_ids.iter().enumerate() {
        index_by_id.entry(id.as_str()).or_insert(index);
    }

    let mut predecessors = vec![Vec::new(); node_ids.len()];
    let mut successors = vec![Vec::new(); node_ids.len()];
    let mut has_unresolved_predecessor = vec![false; node_ids.len()];
    for (target_index, ids) in predecessor_ids.iter().enumerate() {
        for id in ids {
            let Some(&source_index) = index_by_id.get(id.as_str()) else {
                has_unresolved_predecessor[target_index] = true;
                continue;
            };
            predecessors[target_index].push(source_index);
            successors[source_index].push(target_index);
        }
    }
    for indices in predecessors.iter_mut().chain(successors.iter_mut()) {
        indices.sort_by(|left, right| node_ids[*left].cmp(&node_ids[*right]).then(left.cmp(right)));
        indices.dedup();
    }

    FlowNodeTopology {
        node_ids: node_ids.to_vec(),
        has_unresolved_predecessor,
        predecessors,
        successors,
    }
}

fn sorted_node_indices(topology: &FlowNodeTopology) -> Vec<usize> {
    let mut indices = (0..topology.node_ids.len()).collect::<Vec<_>>();
    indices.sort_by(|left, right| {
        topology.node_ids[*left]
            .cmp(&topology.node_ids[*right])
            .then(left.cmp(right))
    });
    indices
}

fn append_depth_first_postorder(
    start: usize,
    edges: &[Vec<usize>],
    visited: &mut [bool],
    postorder: &mut Vec<usize>,
) {
    if visited[start] {
        return;
    }
    visited[start] = true;
    let mut stack = vec![(start, 0usize)];

    while let Some((node_index, next_edge_index)) = stack.last_mut() {
        if let Some(&successor_index) = edges[*node_index].get(*next_edge_index) {
            *next_edge_index += 1;
            if !visited[successor_index] {
                visited[successor_index] = true;
                stack.push((successor_index, 0));
            }
            continue;
        }

        let Some((finished_index, _)) = stack.pop() else {
            break;
        };
        postorder.push(finished_index);
    }
}

fn reverse_postorder_indices(topology: &FlowNodeTopology) -> Vec<usize> {
    let mut visited = vec![false; topology.successors.len()];
    let mut postorder = Vec::with_capacity(topology.successors.len());
    let node_indices = sorted_node_indices(topology);

    for &root_index in node_indices
        .iter()
        .filter(|index| topology.predecessors[**index].is_empty())
    {
        append_depth_first_postorder(
            root_index,
            &topology.successors,
            &mut visited,
            &mut postorder,
        );
    }
    for node_index in node_indices {
        append_depth_first_postorder(
            node_index,
            &topology.successors,
            &mut visited,
            &mut postorder,
        );
    }

    postorder.reverse();
    postorder
}

fn strongly_connected_components(topology: &FlowNodeTopology) -> Vec<Vec<usize>> {
    let mut visited = vec![false; topology.successors.len()];
    let mut postorder = Vec::with_capacity(topology.successors.len());
    for node_index in sorted_node_indices(topology) {
        append_depth_first_postorder(
            node_index,
            &topology.successors,
            &mut visited,
            &mut postorder,
        );
    }

    visited.fill(false);
    let mut components = Vec::new();
    for &start in postorder.iter().rev() {
        if visited[start] {
            continue;
        }
        visited[start] = true;
        let mut component = Vec::new();
        let mut stack = vec![start];
        while let Some(node_index) = stack.pop() {
            component.push(node_index);
            for &predecessor_index in topology.predecessors[node_index].iter().rev() {
                if !visited[predecessor_index] {
                    visited[predecessor_index] = true;
                    stack.push(predecessor_index);
                }
            }
        }
        component.sort_by(|left, right| {
            topology.node_ids[*left]
                .cmp(&topology.node_ids[*right])
                .then(left.cmp(right))
        });
        components.push(component);
    }
    components
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
    let node_ids = nodes.iter().map(|node| node.id.clone()).collect::<Vec<_>>();
    let predecessor_ids = nodes
        .iter()
        .map(|node| node.predecessor_ids.clone())
        .collect::<Vec<_>>();
    let topology = flow_node_topology(&node_ids, &predecessor_ids);
    let schedule = reverse_postorder_indices(&topology);
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

        let mut round_worklist = VecDeque::from(schedule.clone());
        while let Some(node_index) = round_worklist.pop_front() {
            let node = &nodes[node_index];
            let input_value = canonical_string_ids(&node.predecessor_ids)
                .into_iter()
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ClassValueFlowSolverPolicy {
    reverse_postorder: bool,
    loop_header_widening: bool,
    narrowing: bool,
}

const PRODUCTION_CLASS_VALUE_FLOW_SOLVER_POLICY: ClassValueFlowSolverPolicy =
    ClassValueFlowSolverPolicy {
        reverse_postorder: true,
        loop_header_widening: true,
        narrowing: true,
    };

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClassValueFlowTestPolicyV0 {
    Full,
    CauseAOnly,
    CauseBOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ClassValueFlowSolverDiagnostics {
    loop_header_count: usize,
    old_seeded_header_count: usize,
    widened_header_count: usize,
    narrowing_attempted: bool,
    narrowing_round_count: usize,
    narrowing_stable: bool,
    narrowing_reverted: bool,
    post_fixpoint: bool,
}

struct ClassValueFlowSolverOutcome {
    analysis: ClassValueFlowAnalysisV0,
    #[cfg_attr(not(test), allow(dead_code))]
    diagnostics: ClassValueFlowSolverDiagnostics,
}

#[cfg(test)]
pub(crate) struct ClassValueFlowTestOutcomeV0 {
    pub analysis: ClassValueFlowAnalysisV0,
    pub loop_header_count: usize,
    pub old_seeded_header_count: usize,
    pub widened_header_count: usize,
    pub narrowing_attempted: bool,
    pub narrowing_round_count: usize,
    pub narrowing_stable: bool,
    pub narrowing_reverted: bool,
    pub post_fixpoint: bool,
}

#[cfg(test)]
pub(crate) fn analyze_class_value_flow_with_test_policy(
    graph: &ClassValueFlowGraphV0,
    policy: ClassValueFlowTestPolicyV0,
) -> ClassValueFlowTestOutcomeV0 {
    let policy = match policy {
        ClassValueFlowTestPolicyV0::Full => PRODUCTION_CLASS_VALUE_FLOW_SOLVER_POLICY,
        ClassValueFlowTestPolicyV0::CauseAOnly => ClassValueFlowSolverPolicy {
            reverse_postorder: true,
            loop_header_widening: false,
            narrowing: false,
        },
        ClassValueFlowTestPolicyV0::CauseBOnly => ClassValueFlowSolverPolicy {
            reverse_postorder: false,
            loop_header_widening: true,
            narrowing: true,
        },
    };
    let outcome = analyze_class_value_flow_with_policy(graph, policy);
    ClassValueFlowTestOutcomeV0 {
        analysis: outcome.analysis,
        loop_header_count: outcome.diagnostics.loop_header_count,
        old_seeded_header_count: outcome.diagnostics.old_seeded_header_count,
        widened_header_count: outcome.diagnostics.widened_header_count,
        narrowing_attempted: outcome.diagnostics.narrowing_attempted,
        narrowing_round_count: outcome.diagnostics.narrowing_round_count,
        narrowing_stable: outcome.diagnostics.narrowing_stable,
        narrowing_reverted: outcome.diagnostics.narrowing_reverted,
        post_fixpoint: outcome.diagnostics.post_fixpoint,
    }
}

fn cyclic_scc_header_indices(topology: &FlowNodeTopology) -> BTreeSet<usize> {
    let mut headers = BTreeSet::new();
    for component in strongly_connected_components(topology) {
        let cyclic = component.len() > 1
            || component
                .first()
                .is_some_and(|node_index| topology.successors[*node_index].contains(node_index));
        if !cyclic {
            continue;
        }
        let component_indices = component.iter().copied().collect::<BTreeSet<_>>();
        // Every open-SCC entry is a header, regardless of its transfer. A
        // closed SCC has no natural entry, so its first node in stable
        // (id, index) order is the sole fallback header.
        let mut component_headers = component
            .iter()
            .copied()
            .filter(|node_index| {
                topology.has_unresolved_predecessor[*node_index]
                    || topology.predecessors[*node_index]
                        .iter()
                        .any(|predecessor_index| !component_indices.contains(predecessor_index))
            })
            .collect::<Vec<_>>();
        if component_headers.is_empty() {
            component_headers.extend(component.first().copied());
        }
        headers.extend(component_headers);
    }
    headers
}

#[cfg(test)]
pub(crate) fn class_value_flow_loop_header_ids_for_test(
    graph: &ClassValueFlowGraphV0,
) -> Vec<String> {
    let node_ids = graph
        .nodes
        .iter()
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    let predecessor_ids = graph
        .nodes
        .iter()
        .map(|node| node.predecessors.clone())
        .collect::<Vec<_>>();
    let topology = flow_node_topology(&node_ids, &predecessor_ids);
    cyclic_scc_header_indices(&topology)
        .into_iter()
        .map(|node_index| graph.nodes[node_index].id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn is_strict_abstract_class_value_subset(
    left: &AbstractClassValueV0,
    right: &AbstractClassValueV0,
) -> bool {
    abstract_class_value_is_subset(left, right) && !abstract_class_value_is_subset(right, left)
}

fn char_safe_longest_common_prefix(values: &[String]) -> String {
    let Some(first) = values.first() else {
        return String::new();
    };
    let mut prefix = first.clone();
    for value in values.iter().skip(1) {
        let match_length = prefix
            .chars()
            .zip(value.chars())
            .take_while(|(left, right)| left == right)
            .map(|(char, _)| char.len_utf8())
            .sum();
        prefix.truncate(match_length);
        if prefix.is_empty() {
            break;
        }
    }
    prefix
}

fn widen_ascending_loop_header_value(
    previous: &AbstractClassValueV0,
    candidate: &AbstractClassValueV0,
) -> Option<AbstractClassValueV0> {
    if matches!(previous, AbstractClassValueV0::Bottom)
        || !is_strict_abstract_class_value_subset(previous, candidate)
    {
        return None;
    }

    let values = finite_language_values(candidate)?;
    let minimum_length = values.iter().map(|value| value.chars().count()).min()?;
    let maximum_length = values.iter().map(|value| value.chars().count()).max()?;
    if maximum_length.saturating_sub(minimum_length) <= MAX_FINITE_CLASS_VALUES {
        return None;
    }

    let prefix = char_safe_longest_common_prefix(&values);
    if prefix.is_empty() {
        return None;
    }
    let widened = prefix_class_value(prefix, None);
    (abstract_class_value_is_subset(previous, &widened)
        && abstract_class_value_is_subset(candidate, &widened))
    .then_some(widened)
}

#[cfg(test)]
pub(crate) fn widen_ascending_loop_header_value_for_test(
    previous: &AbstractClassValueV0,
    candidate: &AbstractClassValueV0,
) -> Option<AbstractClassValueV0> {
    widen_ascending_loop_header_value(previous, candidate)
}

fn descending_narrow_class_value(
    previous: &AbstractClassValueV0,
    candidate: &AbstractClassValueV0,
) -> AbstractClassValueV0 {
    if is_strict_abstract_class_value_subset(candidate, previous) {
        candidate.clone()
    } else {
        previous.clone()
    }
}

#[cfg(test)]
pub(crate) fn descending_narrow_class_value_for_test(
    previous: &AbstractClassValueV0,
    candidate: &AbstractClassValueV0,
) -> AbstractClassValueV0 {
    descending_narrow_class_value(previous, candidate)
}

struct ClassValueFlowNarrowingResult {
    values: BTreeMap<String, AbstractClassValueV0>,
    stable: bool,
    round_count: usize,
}

fn narrow_class_value_flow(
    graph: &ClassValueFlowGraphV0,
    schedule: &[usize],
    initial_values: &BTreeMap<String, AbstractClassValueV0>,
    max_rounds: usize,
) -> ClassValueFlowNarrowingResult {
    let mut values = initial_values.clone();
    let mut round_count = 0;

    for round in 1..=max_rounds {
        round_count = round;
        let mut changed = false;
        let mut round_worklist = VecDeque::from(schedule.to_vec());
        while let Some(node_index) = round_worklist.pop_front() {
            let node = &graph.nodes[node_index];
            let incoming = join_predecessor_flow_values(node, &values, None);
            let candidate = apply_flow_transfer(&incoming, &node.transfer);
            let previous = values
                .get(&node.id)
                .cloned()
                .unwrap_or_else(bottom_class_value);
            let narrowed = descending_narrow_class_value(&previous, &candidate);
            if narrowed != previous {
                values.insert(node.id.clone(), narrowed);
                changed = true;
            }
        }
        if !changed {
            return ClassValueFlowNarrowingResult {
                values,
                stable: true,
                round_count,
            };
        }
    }

    ClassValueFlowNarrowingResult {
        values,
        stable: false,
        round_count,
    }
}

fn inflationary_flow_candidate(
    node: &ClassValueFlowNodeV0,
    values: &BTreeMap<String, AbstractClassValueV0>,
    previous: &AbstractClassValueV0,
    accelerate_loop_header: bool,
) -> AbstractClassValueV0 {
    if accelerate_loop_header && !matches!(node.transfer, ClassValueFlowTransferV0::Join) {
        // Concat and refinement distribute over predecessor-language union.
        // Applying them per predecessor avoids losing a common prefix in an
        // intermediate, representation-limited join. Seed the join of those
        // output values with the previous output; seeding the transfer input
        // would reapply a non-identity transfer to an old result. Assign
        // ignores input and therefore contributes one output value.
        if matches!(node.transfer, ClassValueFlowTransferV0::AssignFacts(_)) {
            let transferred = apply_flow_transfer(&bottom_class_value(), &node.transfer);
            join_abstract_class_values(previous, &transferred)
        } else {
            canonical_string_ids(&node.predecessors).into_iter().fold(
                previous.clone(),
                |joined, id| {
                    let predecessor = values.get(id).cloned().unwrap_or_else(|| {
                        top_class_value_with_provenance(
                            AbstractClassValueProvenanceV0::MissingFlowPredecessor,
                        )
                    });
                    let transferred = apply_flow_transfer(&predecessor, &node.transfer);
                    join_abstract_class_values(&joined, &transferred)
                },
            )
        }
    } else {
        let join_header_seed = accelerate_loop_header.then_some(previous);
        let incoming = join_predecessor_flow_values(node, values, join_header_seed);
        apply_flow_transfer(&incoming, &node.transfer)
    }
}

fn class_value_flow_values_are_post_fixpoint(
    graph: &ClassValueFlowGraphV0,
    values: &BTreeMap<String, AbstractClassValueV0>,
    loop_header_indices: &BTreeSet<usize>,
    accelerate_loop_headers: bool,
) -> bool {
    graph.nodes.iter().enumerate().all(|(node_index, node)| {
        let stored = values
            .get(&node.id)
            .cloned()
            .unwrap_or_else(bottom_class_value);
        let candidate = inflationary_flow_candidate(
            node,
            values,
            &stored,
            accelerate_loop_headers && loop_header_indices.contains(&node_index),
        );
        abstract_class_value_is_subset(&candidate, &stored)
    })
}

/// Runs bounded deterministic iteration over one supplied graph without
/// deriving a call graph or call-site context.
pub fn analyze_class_value_flow(graph: &ClassValueFlowGraphV0) -> ClassValueFlowAnalysisV0 {
    analyze_class_value_flow_with_policy(graph, PRODUCTION_CLASS_VALUE_FLOW_SOLVER_POLICY).analysis
}

fn analyze_class_value_flow_with_policy(
    graph: &ClassValueFlowGraphV0,
    policy: ClassValueFlowSolverPolicy,
) -> ClassValueFlowSolverOutcome {
    let node_ids = graph
        .nodes
        .iter()
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    let predecessor_ids = graph
        .nodes
        .iter()
        .map(|node| node.predecessors.clone())
        .collect::<Vec<_>>();
    let topology = flow_node_topology(&node_ids, &predecessor_ids);
    let reverse_postorder = reverse_postorder_indices(&topology);
    let schedule = if policy.reverse_postorder {
        reverse_postorder.clone()
    } else {
        (0..graph.nodes.len()).collect()
    };
    let loop_header_indices = cyclic_scc_header_indices(&topology);
    let mut values = graph
        .nodes
        .iter()
        .map(|node| (node.id.clone(), bottom_class_value()))
        .collect::<BTreeMap<_, _>>();
    let mut converged = false;
    let mut iteration_count = 0;
    let mut old_seeded_header_indices = BTreeSet::new();
    let mut widened_header_indices = BTreeSet::new();

    for iteration in 1..=MAX_FLOW_ANALYSIS_ITERATIONS {
        iteration_count = iteration;
        let mut changed = false;

        let mut round_worklist = VecDeque::from(schedule.clone());
        while let Some(node_index) = round_worklist.pop_front() {
            let node = &graph.nodes[node_index];
            let previous = values
                .get(&node.id)
                .cloned()
                .unwrap_or_else(bottom_class_value);
            // Join headers are old-seeded before their identity transfer.
            // Other headers join the old value after their transfer so the
            // accelerated equation remains inflationary without reapplying a
            // non-identity transfer to the old value.
            let accelerates_loop_header =
                policy.loop_header_widening && loop_header_indices.contains(&node_index);
            let uses_old_seed =
                accelerates_loop_header && matches!(node.transfer, ClassValueFlowTransferV0::Join);
            if uses_old_seed {
                old_seeded_header_indices.insert(node_index);
            }
            let candidate =
                inflationary_flow_candidate(node, &values, &previous, accelerates_loop_header);
            let next = if accelerates_loop_header {
                if let Some(widened) = widen_ascending_loop_header_value(&previous, &candidate) {
                    widened_header_indices.insert(node_index);
                    widened
                } else {
                    candidate
                }
            } else {
                candidate
            };

            if next != previous {
                values.insert(node.id.clone(), next);
                changed = true;
            }
        }

        if !changed {
            converged = true;
            break;
        }
    }

    let mut narrowing_attempted = false;
    let mut narrowing_round_count = 0;
    let mut narrowing_stable = true;
    let mut narrowing_reverted = false;
    if converged && policy.narrowing && !widened_header_indices.is_empty() {
        // Narrow against the unseeded transfer functional, accept only strict
        // descents, and keep the widening post-fixpoint unless a bounded phase
        // reaches a stable post-fixpoint of the accelerated header equations.
        let widening_post_fixpoint = values.clone();
        let remaining_rounds = MAX_FLOW_ANALYSIS_ITERATIONS.saturating_sub(iteration_count);
        let narrowing_round_budget = remaining_rounds.min(MAX_FLOW_NARROWING_ROUNDS);
        if narrowing_round_budget > 0 {
            narrowing_attempted = true;
            let narrowing =
                narrow_class_value_flow(graph, &reverse_postorder, &values, narrowing_round_budget);
            iteration_count += narrowing.round_count;
            narrowing_round_count = narrowing.round_count;
            let narrowing_is_sound_post_fixpoint = narrowing.stable
                && class_value_flow_values_are_post_fixpoint(
                    graph,
                    &narrowing.values,
                    &loop_header_indices,
                    policy.loop_header_widening,
                );
            narrowing_stable = narrowing_is_sound_post_fixpoint;
            if narrowing_is_sound_post_fixpoint {
                values = narrowing.values;
            } else {
                values = widening_post_fixpoint;
                narrowing_reverted = true;
            }
        }
    }

    let post_fixpoint = converged
        && class_value_flow_values_are_post_fixpoint(
            graph,
            &values,
            &loop_header_indices,
            policy.loop_header_widening,
        );
    if !post_fixpoint {
        converged = false;
        for value in values.values_mut() {
            *value =
                top_class_value_with_provenance(AbstractClassValueProvenanceV0::FlowIterationLimit);
        }
    }
    let analysis = ClassValueFlowAnalysisV0 {
        schema_version: "0",
        product: "omena-abstract-value.flow-analysis",
        context_sensitivity: "perSuppliedGraph",
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
    };
    ClassValueFlowSolverOutcome {
        analysis,
        diagnostics: ClassValueFlowSolverDiagnostics {
            loop_header_count: loop_header_indices.len(),
            old_seeded_header_count: old_seeded_header_indices.len(),
            widened_header_count: widened_header_indices.len(),
            narrowing_attempted,
            narrowing_round_count,
            narrowing_stable,
            narrowing_reverted,
            post_fixpoint,
        },
    }
}

/// Prunes and evaluates one supplied control-flow graph without deriving a
/// call graph or call-site context.
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
        context_sensitivity: "perSuppliedGraph",
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
    let mut incremental_database = OmenaIncrementalDatabaseV0::default();
    if let Some(previous_snapshot) = previous_snapshot {
        incremental_database.restore_snapshot(previous_snapshot);
    }
    analyze_class_value_flow_incremental_with_database(
        graph,
        &mut incremental_database,
        previous_analysis,
        revision,
    )
}

pub fn analyze_class_value_flow_incremental_with_database(
    graph: &ClassValueFlowGraphV0,
    incremental_database: &mut OmenaIncrementalDatabaseV0,
    previous_analysis: Option<&ClassValueFlowAnalysisV0>,
    revision: u64,
) -> ClassValueFlowIncrementalAnalysisV0 {
    let incremental_input = class_value_flow_incremental_input(graph, revision);
    let update = incremental_database.plan_and_upsert_graph_input(&incremental_input);
    let structurally_reusable_previous_analysis = previous_analysis.filter(|analysis| {
        update.incremental_plan.dirty_node_count == 0
            && previous_analysis_matches_flow_graph_identity(analysis, graph)
    });
    // The public API receives the snapshot and analysis as separate values, so
    // a clean snapshot cannot by itself prove that the supplied analysis came
    // from that snapshot. Verify deterministic fresh equivalence before
    // admitting reuse; otherwise a mixed snapshot/analysis pair can return a
    // stale value while claiming a clean incremental plan.
    let verified_fresh_analysis = structurally_reusable_previous_analysis
        .is_some()
        .then(|| analyze_class_value_flow(graph));
    let reusable_previous_analysis = structurally_reusable_previous_analysis
        .filter(|analysis| verified_fresh_analysis.as_ref() == Some(*analysis));
    let reused_previous_analysis = reusable_previous_analysis.is_some();
    let analysis = reusable_previous_analysis
        .cloned()
        .or(verified_fresh_analysis)
        .unwrap_or_else(|| analyze_class_value_flow(graph));

    ClassValueFlowIncrementalAnalysisV0 {
        schema_version: "0",
        product: "omena-abstract-value.incremental-flow-analysis",
        reused_previous_analysis,
        incremental_plan: update.incremental_plan,
        next_snapshot: update.next_snapshot,
        analysis,
    }
}

fn previous_analysis_matches_flow_graph_identity(
    analysis: &ClassValueFlowAnalysisV0,
    graph: &ClassValueFlowGraphV0,
) -> bool {
    analysis.context_key == graph.context_key
        && analysis
            .nodes
            .iter()
            .map(|node| node.id.as_str())
            .eq(graph.nodes.iter().map(|node| node.id.as_str()))
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

/// Partitions caller-supplied graphs by caller-supplied call-site identifiers.
///
/// This does not derive a call graph or run an interprocedural fixed point.
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

/// Partitions caller-supplied graphs by bounded caller-supplied call-site
/// stacks, then performs one join pass per retained context.
///
/// This does not derive a call graph or run an interprocedural fixed point.
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
    let node_ids = graph
        .nodes
        .iter()
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    assert_unique_flow_node_ids(&node_ids);
    IncrementalGraphInputV0 {
        revision: IncrementalRevisionV0 { value: revision },
        nodes: graph
            .nodes
            .iter()
            .enumerate()
            .map(|(output_ordinal, node)| IncrementalNodeInputV0 {
                id: node.id.clone(),
                digest: flow_node_incremental_digest(
                    graph.context_key.as_deref(),
                    output_ordinal,
                    node,
                ),
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

pub fn reachable_control_flow_block_ids<TGraph>(
    graph: &TGraph,
) -> BTreeSet<<TGraph as ControlFlowEdgeGraphV0>::BlockId>
where
    TGraph: ControlFlowEdgeGraphV0,
{
    let successors_by_id = graph
        .successor_block_ids_by_source()
        .into_iter()
        .collect::<BTreeMap<_, _>>();
    let mut reachable = BTreeSet::new();
    let mut worklist = graph.entry_block_id().into_iter().collect::<Vec<_>>();

    while let Some(block_id) = worklist.pop() {
        if !reachable.insert(block_id.clone()) {
            continue;
        }
        let Some(successor_ids) = successors_by_id.get(&block_id) else {
            continue;
        };
        worklist.extend(successor_ids.iter().cloned());
    }

    reachable
}

pub fn control_flow_predecessor_counts<TGraph>(
    graph: &TGraph,
) -> BTreeMap<<TGraph as ControlFlowEdgeGraphV0>::BlockId, usize>
where
    TGraph: ControlFlowEdgeGraphV0,
{
    let successors_by_id = graph.successor_block_ids_by_source();
    let mut counts = successors_by_id
        .iter()
        .map(|(block_id, _)| (block_id.clone(), 0usize))
        .collect::<BTreeMap<_, _>>();

    for (_, successor_ids) in successors_by_id {
        for successor_id in successor_ids {
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
    seed: Option<&AbstractClassValueV0>,
) -> AbstractClassValueV0 {
    let mut joined = seed.cloned();
    for id in canonical_string_ids(&node.predecessors) {
        let predecessor = values.get(id).cloned().unwrap_or_else(|| {
            top_class_value_with_provenance(AbstractClassValueProvenanceV0::MissingFlowPredecessor)
        });
        joined = Some(match joined {
            Some(current) => join_abstract_class_values(&current, &predecessor),
            None => predecessor,
        });
    }
    joined.unwrap_or_else(bottom_class_value)
}

fn canonical_string_ids(ids: &[String]) -> Vec<&str> {
    let mut ids = ids.iter().map(String::as_str).collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();
    ids
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

fn flow_node_incremental_digest(
    graph_context_key: Option<&str>,
    output_ordinal: usize,
    node: &ClassValueFlowNodeV0,
) -> String {
    let transfer = match &node.transfer {
        ClassValueFlowTransferV0::AssignFacts(facts) => {
            ClassValueFlowTransferDigestPayload::AssignFacts(canonical_facts_digest_payload(facts))
        }
        ClassValueFlowTransferV0::RefineFacts(facts) => {
            ClassValueFlowTransferDigestPayload::RefineFacts(canonical_facts_digest_payload(facts))
        }
        ClassValueFlowTransferV0::ConcatFacts(facts) => {
            ClassValueFlowTransferDigestPayload::ConcatFacts(canonical_facts_digest_payload(facts))
        }
        ClassValueFlowTransferV0::Join => ClassValueFlowTransferDigestPayload::Join,
    };
    let payload = ClassValueFlowNodeDigestPayload {
        solver_revision: CLASS_VALUE_FLOW_SOLVER_REVISION_V0,
        context_key: graph_context_key,
        output_ordinal,
        node_id: &node.id,
        dependency_ids: &node.predecessors,
        transfer,
    };

    serde_json::to_string(&payload).unwrap_or_else(|error| error.to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClassValueFlowNodeDigestPayload<'a> {
    solver_revision: &'static str,
    context_key: Option<&'a str>,
    output_ordinal: usize,
    node_id: &'a str,
    dependency_ids: &'a [String],
    transfer: ClassValueFlowTransferDigestPayload<'a>,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "facts", rename_all = "camelCase")]
enum ClassValueFlowTransferDigestPayload<'a> {
    AssignFacts(ExternalStringTypeFactsDigestPayload<'a>),
    RefineFacts(ExternalStringTypeFactsDigestPayload<'a>),
    ConcatFacts(ExternalStringTypeFactsDigestPayload<'a>),
    Join,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExternalStringTypeFactsDigestPayload<'a> {
    kind: &'a str,
    constraint_kind: Option<&'a str>,
    values: Option<Vec<&'a str>>,
    prefix: Option<&'a str>,
    suffix: Option<&'a str>,
    min_len: Option<Utf16CodeUnitLengthV0>,
    max_len: Option<Utf16CodeUnitLengthV0>,
    char_must: Option<&'a str>,
    char_may: Option<&'a str>,
    may_include_other_chars: Option<bool>,
}

fn canonical_facts_digest_payload(
    facts: &ExternalStringTypeFactsV0,
) -> ExternalStringTypeFactsDigestPayload<'_> {
    let values = facts.values.as_ref().map(|values| {
        let mut values = values.iter().map(String::as_str).collect::<Vec<_>>();
        values.sort_unstable();
        values.dedup();
        values
    });
    ExternalStringTypeFactsDigestPayload {
        kind: &facts.kind,
        constraint_kind: facts.constraint_kind.as_deref(),
        values,
        prefix: facts.prefix.as_deref(),
        suffix: facts.suffix.as_deref(),
        min_len: facts.min_len,
        max_len: facts.max_len,
        char_must: facts.char_must.as_deref(),
        char_may: facts.char_may.as_deref(),
        may_include_other_chars: facts.may_include_other_chars,
    }
}
