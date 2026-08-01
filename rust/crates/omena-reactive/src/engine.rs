use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    error::Error,
    fmt,
};

use crate::{
    ChangePolicyV0, ReactiveNodeIdV0, ReactiveNodeKindV0, ReactiveStateV0, ReactiveUnavailableV0,
    ReactiveValueV0,
    graph::{NodeBlueprintV0, NodeOperationV0, ReactiveGraphIdV0},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectReceiptV0 {
    pub channel: String,
    pub wave: u64,
    pub state: ReactiveStateV0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StabilizeStatusV0 {
    Settled {
        wave: u64,
        recomputed_node_count: usize,
    },
    Pending {
        wave: u64,
        recomputed_node_count: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReactiveEngineErrorV0 {
    InvalidNode { node_index: usize },
    NodeDoesNotAcceptDeposits { node_index: usize },
    ObserverMutationDuringWave,
    ZeroStepBudget,
}

impl fmt::Display for ReactiveEngineErrorV0 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNode { node_index } => {
                write!(formatter, "reactive node {node_index} does not exist")
            }
            Self::NodeDoesNotAcceptDeposits { node_index } => {
                write!(
                    formatter,
                    "reactive node {node_index} does not accept deposits"
                )
            }
            Self::ObserverMutationDuringWave => {
                write!(
                    formatter,
                    "observers cannot be changed during stabilization"
                )
            }
            Self::ZeroStepBudget => write!(formatter, "stabilization step budget must be non-zero"),
        }
    }
}

impl Error for ReactiveEngineErrorV0 {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeltaFoldParityErrorV0 {
    InvalidNode {
        node_index: usize,
    },
    NotDeltaFold {
        node: ReactiveNodeIdV0,
    },
    Diverged {
        node: ReactiveNodeIdV0,
        incremental_digest: [u8; 32],
        rebuilt_digest: [u8; 32],
    },
}

impl fmt::Display for DeltaFoldParityErrorV0 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNode { node_index } => {
                write!(formatter, "reactive node {node_index} does not exist")
            }
            Self::NotDeltaFold { node } => {
                write!(
                    formatter,
                    "reactive node {} is not a delta fold",
                    node.index()
                )
            }
            Self::Diverged { node, .. } => write!(
                formatter,
                "delta-fold node {} diverged from a full rebuild",
                node.index()
            ),
        }
    }
}

impl Error for DeltaFoldParityErrorV0 {}

struct RuntimeNodeV0 {
    operation: NodeOperationV0,
    dependencies: Vec<ReactiveNodeIdV0>,
    parents: Vec<ReactiveNodeIdV0>,
    height: u32,
    state: ReactiveStateV0,
    change_policy: ChangePolicyV0,
    observer_count: usize,
    needed_by: usize,
    stale: bool,
    scheduled_wave: Option<u64>,
    last_recomputed_wave: Option<u64>,
    recompute_count: usize,
    delta_entries: BTreeMap<String, ReactiveStateV0>,
    delta_dirty_dependencies: BTreeSet<ReactiveNodeIdV0>,
    delta_update_count: usize,
}

pub struct ReactiveEngineV0 {
    nodes: Vec<RuntimeNodeV0>,
    deferred_deposits: BTreeMap<ReactiveNodeIdV0, ReactiveStateV0>,
    activation_nodes: BTreeSet<ReactiveNodeIdV0>,
    queues: BTreeMap<u32, VecDeque<ReactiveNodeIdV0>>,
    effect_receipts: Vec<EffectReceiptV0>,
    current_wave: u64,
    current_wave_recomputes: usize,
    stabilizing: bool,
}

impl ReactiveEngineV0 {
    pub(crate) fn from_blueprints(
        graph_id: ReactiveGraphIdV0,
        blueprints: Vec<NodeBlueprintV0>,
    ) -> Self {
        let mut nodes: Vec<_> = blueprints
            .into_iter()
            .map(|blueprint| {
                let stale = !matches!(
                    blueprint.operation,
                    NodeOperationV0::Input | NodeOperationV0::AsyncResult
                );
                RuntimeNodeV0 {
                    operation: blueprint.operation,
                    dependencies: blueprint.dependencies,
                    parents: Vec::new(),
                    height: blueprint.height,
                    state: blueprint.initial_state,
                    change_policy: blueprint.change_policy,
                    observer_count: 0,
                    needed_by: 0,
                    stale,
                    scheduled_wave: None,
                    last_recomputed_wave: None,
                    recompute_count: 0,
                    delta_entries: BTreeMap::new(),
                    delta_dirty_dependencies: BTreeSet::new(),
                    delta_update_count: 0,
                }
            })
            .collect();

        for node_index in 0..nodes.len() {
            let parent = ReactiveNodeIdV0 {
                index: node_index,
                graph: graph_id,
            };
            for dependency in nodes[node_index].dependencies.clone() {
                nodes[dependency.index()].parents.push(parent);
            }
        }

        Self {
            nodes,
            deferred_deposits: BTreeMap::new(),
            activation_nodes: BTreeSet::new(),
            queues: BTreeMap::new(),
            effect_receipts: Vec::new(),
            current_wave: 0,
            current_wave_recomputes: 0,
            stabilizing: false,
        }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn node_kind(
        &self,
        node: ReactiveNodeIdV0,
    ) -> Result<ReactiveNodeKindV0, ReactiveEngineErrorV0> {
        Ok(self.node(node)?.operation.kind())
    }

    pub fn state(&self, node: ReactiveNodeIdV0) -> Result<&ReactiveStateV0, ReactiveEngineErrorV0> {
        Ok(&self.node(node)?.state)
    }

    pub fn is_necessary(&self, node: ReactiveNodeIdV0) -> Result<bool, ReactiveEngineErrorV0> {
        Ok(self.node(node)?.needed_by > 0)
    }

    pub fn is_stale(&self, node: ReactiveNodeIdV0) -> Result<bool, ReactiveEngineErrorV0> {
        Ok(self.node(node)?.stale)
    }

    pub fn node_recompute_count(
        &self,
        node: ReactiveNodeIdV0,
    ) -> Result<usize, ReactiveEngineErrorV0> {
        Ok(self.node(node)?.recompute_count)
    }

    pub fn delta_update_count(
        &self,
        node: ReactiveNodeIdV0,
    ) -> Result<usize, ReactiveEngineErrorV0> {
        Ok(self.node(node)?.delta_update_count)
    }

    pub fn current_wave(&self) -> u64 {
        self.current_wave
    }

    pub fn is_stabilizing(&self) -> bool {
        self.stabilizing
    }

    pub fn has_pending_work(&self) -> bool {
        self.stabilizing || !self.deferred_deposits.is_empty() || !self.activation_nodes.is_empty()
    }

    /// Queues the latest value for an input-like node. The deposit is applied
    /// only when the next wave begins, including when it arrives between
    /// bounded steps of the current wave.
    pub fn deposit(
        &mut self,
        node: ReactiveNodeIdV0,
        state: ReactiveStateV0,
    ) -> Result<(), ReactiveEngineErrorV0> {
        match self.node(node)?.operation {
            NodeOperationV0::Input | NodeOperationV0::AsyncResult => {
                self.deferred_deposits.insert(node, state);
                Ok(())
            }
            _ => Err(ReactiveEngineErrorV0::NodeDoesNotAcceptDeposits {
                node_index: node.index(),
            }),
        }
    }

    pub fn observe(&mut self, node: ReactiveNodeIdV0) -> Result<(), ReactiveEngineErrorV0> {
        if self.stabilizing {
            return Err(ReactiveEngineErrorV0::ObserverMutationDuringWave);
        }
        self.node(node)?;
        self.nodes[node.index()].observer_count += 1;
        if self.nodes[node.index()].observer_count == 1 {
            self.increase_necessity(node);
        }
        Ok(())
    }

    pub fn unobserve(&mut self, node: ReactiveNodeIdV0) -> Result<(), ReactiveEngineErrorV0> {
        if self.stabilizing {
            return Err(ReactiveEngineErrorV0::ObserverMutationDuringWave);
        }
        self.node(node)?;
        if self.nodes[node.index()].observer_count == 0 {
            return Ok(());
        }
        self.nodes[node.index()].observer_count -= 1;
        if self.nodes[node.index()].observer_count == 0 {
            self.decrease_necessity(node);
        }
        Ok(())
    }

    pub fn stabilize_step(
        &mut self,
        maximum_recomputes: usize,
    ) -> Result<StabilizeStatusV0, ReactiveEngineErrorV0> {
        if maximum_recomputes == 0 {
            return Err(ReactiveEngineErrorV0::ZeroStepBudget);
        }
        if !self.stabilizing {
            self.begin_wave();
        }
        if !self.stabilizing {
            return Ok(StabilizeStatusV0::Settled {
                wave: self.current_wave,
                recomputed_node_count: 0,
            });
        }

        let mut recomputed = 0;
        while recomputed < maximum_recomputes {
            let Some(node) = self.pop_next_scheduled() else {
                self.stabilizing = false;
                return Ok(StabilizeStatusV0::Settled {
                    wave: self.current_wave,
                    recomputed_node_count: recomputed,
                });
            };
            if self.nodes[node.index()].needed_by == 0 {
                self.nodes[node.index()].stale = true;
                continue;
            }
            self.recompute(node);
            recomputed += 1;
            self.current_wave_recomputes += 1;
        }

        if self.queues.values().all(VecDeque::is_empty) {
            self.stabilizing = false;
            Ok(StabilizeStatusV0::Settled {
                wave: self.current_wave,
                recomputed_node_count: recomputed,
            })
        } else {
            Ok(StabilizeStatusV0::Pending {
                wave: self.current_wave,
                recomputed_node_count: recomputed,
            })
        }
    }

    pub fn stabilize_until_settled(
        &mut self,
        maximum_recomputes: usize,
    ) -> Result<StabilizeStatusV0, ReactiveEngineErrorV0> {
        if maximum_recomputes == 0 {
            return Err(ReactiveEngineErrorV0::ZeroStepBudget);
        }
        let mut remaining = maximum_recomputes;
        loop {
            let status = self.stabilize_step(remaining)?;
            match status {
                StabilizeStatusV0::Settled { .. } => return Ok(status),
                StabilizeStatusV0::Pending {
                    recomputed_node_count,
                    ..
                } => {
                    remaining = remaining.saturating_sub(recomputed_node_count);
                    if remaining == 0 {
                        return Ok(status);
                    }
                }
            }
        }
    }

    pub fn drain_effect_receipts(&mut self) -> Vec<EffectReceiptV0> {
        std::mem::take(&mut self.effect_receipts)
    }

    pub fn verify_delta_fold(&self, node: ReactiveNodeIdV0) -> Result<(), DeltaFoldParityErrorV0> {
        let Some(runtime) = self.nodes.get(node.index()) else {
            return Err(DeltaFoldParityErrorV0::InvalidNode {
                node_index: node.index(),
            });
        };
        if !matches!(runtime.operation, NodeOperationV0::DeltaFold { .. }) {
            return Err(DeltaFoldParityErrorV0::NotDeltaFold { node });
        }
        let ReactiveStateV0::Available(ReactiveValueV0::Digest(incremental_digest)) =
            &runtime.state
        else {
            return Ok(());
        };
        let rebuilt_digest = full_delta_digest(&runtime.delta_entries);
        if *incremental_digest == rebuilt_digest {
            Ok(())
        } else {
            Err(DeltaFoldParityErrorV0::Diverged {
                node,
                incremental_digest: *incremental_digest,
                rebuilt_digest,
            })
        }
    }

    fn begin_wave(&mut self) {
        if self.deferred_deposits.is_empty() && self.activation_nodes.is_empty() {
            return;
        }
        self.current_wave = self.current_wave.saturating_add(1);
        self.current_wave_recomputes = 0;
        self.stabilizing = true;

        let deposits = std::mem::take(&mut self.deferred_deposits);
        for (node, next) in deposits {
            let previous = self.nodes[node.index()].state.clone();
            let policy = self.nodes[node.index()].change_policy;
            self.nodes[node.index()].state = next.clone();
            self.nodes[node.index()].stale = false;
            if !policy.equivalent(&previous, &next) {
                self.schedule_parents(node);
            }
        }

        let activation_nodes = std::mem::take(&mut self.activation_nodes);
        for node in activation_nodes {
            if self.nodes[node.index()].needed_by > 0 && self.nodes[node.index()].stale {
                self.schedule(node);
            }
        }

        if self.queues.values().all(VecDeque::is_empty) {
            self.stabilizing = false;
        }
    }

    fn recompute(&mut self, node: ReactiveNodeIdV0) {
        debug_assert_ne!(
            self.nodes[node.index()].last_recomputed_wave,
            Some(self.current_wave),
            "a node must not recompute more than once in one wave"
        );
        let next = self.evaluate(node);
        let previous = self.nodes[node.index()].state.clone();
        let policy = self.nodes[node.index()].change_policy;
        let changed = !policy.equivalent(&previous, &next);

        self.nodes[node.index()].state = next.clone();
        self.nodes[node.index()].stale = false;
        self.nodes[node.index()].last_recomputed_wave = Some(self.current_wave);
        self.nodes[node.index()].recompute_count += 1;

        if !changed {
            return;
        }
        if let NodeOperationV0::EffectBoundary { channel } = &self.nodes[node.index()].operation {
            self.effect_receipts.push(EffectReceiptV0 {
                channel: channel.clone(),
                wave: self.current_wave,
                state: next,
            });
        }
        self.schedule_parents(node);
    }

    fn evaluate(&mut self, node: ReactiveNodeIdV0) -> ReactiveStateV0 {
        let operation = self.nodes[node.index()].operation.clone();
        let dependencies = self.nodes[node.index()].dependencies.clone();
        match operation {
            NodeOperationV0::Input | NodeOperationV0::AsyncResult => {
                self.nodes[node.index()].state.clone()
            }
            NodeOperationV0::Map { operation } => {
                operation(&self.nodes[dependencies[0].index()].state)
            }
            NodeOperationV0::Zip { operation } => operation(
                &self.nodes[dependencies[0].index()].state,
                &self.nodes[dependencies[1].index()].state,
            ),
            NodeOperationV0::Switch => match &self.nodes[dependencies[0].index()].state {
                ReactiveStateV0::Available(ReactiveValueV0::Bool(false)) => {
                    self.nodes[dependencies[1].index()].state.clone()
                }
                ReactiveStateV0::Available(ReactiveValueV0::Bool(true)) => {
                    self.nodes[dependencies[2].index()].state.clone()
                }
                ReactiveStateV0::Available(_) => {
                    ReactiveStateV0::Unavailable(ReactiveUnavailableV0::new(
                        "switchSelectorTypeMismatch",
                        "a switch selector must carry a boolean value",
                    ))
                }
                ReactiveStateV0::Unavailable(unavailable) => {
                    ReactiveStateV0::Unavailable(unavailable.clone())
                }
            },
            NodeOperationV0::DeltaFold { keys } => {
                let mut entries = std::mem::take(&mut self.nodes[node.index()].delta_entries);
                let dirty_dependencies =
                    std::mem::take(&mut self.nodes[node.index()].delta_dirty_dependencies);
                let mut digest = match &self.nodes[node.index()].state {
                    ReactiveStateV0::Available(ReactiveValueV0::Digest(digest)) => *digest,
                    _ => [0; 32],
                };
                let refresh_all = entries.is_empty() || dirty_dependencies.is_empty();
                for (key, dependency) in keys.iter().zip(dependencies) {
                    if !refresh_all && !dirty_dependencies.contains(&dependency) {
                        continue;
                    }
                    let next = self.nodes[dependency.index()].state.clone();
                    if let Some(previous) = entries.insert(key.clone(), next.clone()) {
                        xor_digest(&mut digest, &digest_entry(key, &previous));
                    }
                    xor_digest(&mut digest, &digest_entry(key, &next));
                    self.nodes[node.index()].delta_update_count += 1;
                }
                self.nodes[node.index()].delta_entries = entries;
                ReactiveStateV0::Available(ReactiveValueV0::Digest(digest))
            }
            NodeOperationV0::EffectBoundary { .. } => {
                self.nodes[dependencies[0].index()].state.clone()
            }
        }
    }

    fn increase_necessity(&mut self, node: ReactiveNodeIdV0) {
        self.nodes[node.index()].needed_by += 1;
        if self.nodes[node.index()].needed_by != 1 {
            return;
        }
        for dependency in self.nodes[node.index()].dependencies.clone() {
            self.increase_necessity(dependency);
        }
        if !matches!(
            self.nodes[node.index()].operation,
            NodeOperationV0::Input | NodeOperationV0::AsyncResult
        ) && self.nodes[node.index()].stale
        {
            self.activation_nodes.insert(node);
        }
    }

    fn decrease_necessity(&mut self, node: ReactiveNodeIdV0) {
        debug_assert!(self.nodes[node.index()].needed_by > 0);
        self.nodes[node.index()].needed_by -= 1;
        if self.nodes[node.index()].needed_by != 0 {
            return;
        }
        for dependency in self.nodes[node.index()].dependencies.clone() {
            self.decrease_necessity(dependency);
        }
    }

    fn schedule_parents(&mut self, node: ReactiveNodeIdV0) {
        for parent in self.nodes[node.index()].parents.clone() {
            if matches!(
                self.nodes[parent.index()].operation,
                NodeOperationV0::DeltaFold { .. }
            ) {
                self.nodes[parent.index()]
                    .delta_dirty_dependencies
                    .insert(node);
            }
            if self.nodes[parent.index()].needed_by > 0 {
                self.schedule(parent);
            } else {
                self.nodes[parent.index()].stale = true;
            }
        }
    }

    fn schedule(&mut self, node: ReactiveNodeIdV0) {
        if self.nodes[node.index()].scheduled_wave == Some(self.current_wave) {
            return;
        }
        self.nodes[node.index()].scheduled_wave = Some(self.current_wave);
        self.queues
            .entry(self.nodes[node.index()].height)
            .or_default()
            .push_back(node);
    }

    fn pop_next_scheduled(&mut self) -> Option<ReactiveNodeIdV0> {
        loop {
            let mut entry = self.queues.first_entry()?;
            let queue = entry.get_mut();
            if let Some(node) = queue.pop_front() {
                if queue.is_empty() {
                    entry.remove();
                }
                return Some(node);
            }
            entry.remove();
        }
    }

    fn node(&self, node: ReactiveNodeIdV0) -> Result<&RuntimeNodeV0, ReactiveEngineErrorV0> {
        self.nodes
            .get(node.index())
            .ok_or(ReactiveEngineErrorV0::InvalidNode {
                node_index: node.index(),
            })
    }

    #[cfg(test)]
    pub(crate) fn corrupt_delta_fold_entry_for_test(
        &mut self,
        node: ReactiveNodeIdV0,
        key: &str,
        state: ReactiveStateV0,
    ) {
        self.nodes[node.index()]
            .delta_entries
            .insert(key.to_string(), state);
    }
}

fn full_delta_digest(entries: &BTreeMap<String, ReactiveStateV0>) -> [u8; 32] {
    let mut digest = [0; 32];
    for (key, state) in entries {
        xor_digest(&mut digest, &digest_entry(key, state));
    }
    digest
}

fn digest_entry(key: &str, state: &ReactiveStateV0) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"omena-reactive-delta-entry-v0");
    hash_text(&mut hasher, key);
    hash_state(&mut hasher, state);
    *hasher.finalize().as_bytes()
}

fn hash_state(hasher: &mut blake3::Hasher, state: &ReactiveStateV0) {
    match state {
        ReactiveStateV0::Available(value) => {
            hasher.update(&[0]);
            hash_value(hasher, value);
        }
        ReactiveStateV0::Unavailable(unavailable) => {
            hasher.update(&[1]);
            hash_text(hasher, &unavailable.code);
            hash_text(hasher, &unavailable.detail);
        }
    }
}

fn hash_value(hasher: &mut blake3::Hasher, value: &ReactiveValueV0) {
    match value {
        ReactiveValueV0::Unit => {
            hasher.update(&[0]);
        }
        ReactiveValueV0::Bool(value) => {
            hasher.update(&[1, u8::from(*value)]);
        }
        ReactiveValueV0::Counter(value) => {
            hasher.update(&[2]);
            hasher.update(&value.to_le_bytes());
        }
        ReactiveValueV0::Text(value) => {
            hasher.update(&[3]);
            hash_text(hasher, value);
        }
        ReactiveValueV0::StringSet(values) => {
            hasher.update(&[4]);
            hasher.update(&(values.len() as u64).to_le_bytes());
            for value in values {
                hash_text(hasher, value);
            }
        }
        ReactiveValueV0::TextMap(values) => {
            hasher.update(&[5]);
            hasher.update(&(values.len() as u64).to_le_bytes());
            for (key, value) in values {
                hash_text(hasher, key);
                hash_text(hasher, value);
            }
        }
        ReactiveValueV0::Tuple(values) => {
            hasher.update(&[6]);
            hasher.update(&(values.len() as u64).to_le_bytes());
            for value in values {
                hash_value(hasher, value);
            }
        }
        ReactiveValueV0::Digest(value) => {
            hasher.update(&[7]);
            hasher.update(value);
        }
    }
}

fn hash_text(hasher: &mut blake3::Hasher, value: &str) {
    hasher.update(&(value.len() as u64).to_le_bytes());
    hasher.update(value.as_bytes());
}

fn xor_digest(left: &mut [u8; 32], right: &[u8; 32]) {
    for (left_byte, right_byte) in left.iter_mut().zip(right) {
        *left_byte ^= right_byte;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::{ReactiveGraphBuilderV0, ReactiveValueV0};

    fn exact_policy() -> ChangePolicyV0 {
        ChangePolicyV0::exact("testSemanticValue")
    }

    fn counter_map(state: &ReactiveStateV0) -> ReactiveStateV0 {
        match state {
            ReactiveStateV0::Available(ReactiveValueV0::Counter(value)) => {
                ReactiveStateV0::available(ReactiveValueV0::Counter(value.saturating_add(1)))
            }
            _ => ReactiveStateV0::unavailable("counterInputRequired", "expected a counter"),
        }
    }

    fn counter_zip(left: &ReactiveStateV0, right: &ReactiveStateV0) -> ReactiveStateV0 {
        match (left, right) {
            (
                ReactiveStateV0::Available(ReactiveValueV0::Counter(left)),
                ReactiveStateV0::Available(ReactiveValueV0::Counter(right)),
            ) => ReactiveStateV0::available(ReactiveValueV0::Counter(left.saturating_add(*right))),
            _ => ReactiveStateV0::unavailable("counterInputsRequired", "expected two counters"),
        }
    }

    fn tuple_revision(state: &ReactiveStateV0) -> ReactiveStateV0 {
        match state {
            ReactiveStateV0::Available(ReactiveValueV0::Tuple(values)) => values
                .get(1)
                .cloned()
                .map(ReactiveStateV0::available)
                .unwrap_or_else(|| {
                    ReactiveStateV0::unavailable(
                        "revisionMissing",
                        "the tuple must carry a revision",
                    )
                }),
            _ => ReactiveStateV0::unavailable("tupleRequired", "expected a tuple"),
        }
    }

    fn always_different(_: &ReactiveStateV0, _: &ReactiveStateV0) -> bool {
        false
    }

    fn text_field_only(previous: &ReactiveStateV0, next: &ReactiveStateV0) -> bool {
        fn text(state: &ReactiveStateV0) -> Option<&str> {
            let ReactiveStateV0::Available(ReactiveValueV0::Tuple(values)) = state else {
                return None;
            };
            let Some(ReactiveValueV0::Text(text)) = values.first() else {
                return None;
            };
            Some(text)
        }
        text(previous) == text(next)
    }

    fn revisioned_text(text: &str, revision: u64) -> ReactiveStateV0 {
        ReactiveStateV0::available(ReactiveValueV0::Tuple(vec![
            ReactiveValueV0::Text(text.to_string()),
            ReactiveValueV0::Counter(revision),
        ]))
    }

    fn settled(engine: &mut ReactiveEngineV0) -> Result<(), ReactiveEngineErrorV0> {
        loop {
            if matches!(
                engine.stabilize_step(64)?,
                StabilizeStatusV0::Settled { .. }
            ) {
                return Ok(());
            }
        }
    }

    #[test]
    fn graph_exposes_all_static_node_kinds() -> Result<(), Box<dyn Error>> {
        let mut graph = ReactiveGraphBuilderV0::new();
        let input = graph.add_input(
            ReactiveStateV0::available(ReactiveValueV0::Counter(1)),
            exact_policy(),
        );
        let asynchronous = graph.add_async_result(
            ReactiveStateV0::available(ReactiveValueV0::Counter(2)),
            exact_policy(),
        );
        let mapped = graph.add_map(input, counter_map, exact_policy());
        let zipped = graph.add_zip(mapped, asynchronous, counter_zip, exact_policy());
        let selector = graph.add_input(
            ReactiveStateV0::available(ReactiveValueV0::Bool(true)),
            exact_policy(),
        );
        let switched = graph.add_switch(selector, mapped, zipped, exact_policy());
        let folded = graph.add_delta_fold(
            vec![
                ("mapped".to_string(), mapped),
                ("switched".to_string(), switched),
            ],
            exact_policy(),
        )?;
        let boundary = graph.add_effect_boundary(folded, "diagnostics", exact_policy());
        let engine = graph.build()?;

        let kinds = [
            input,
            mapped,
            zipped,
            switched,
            folded,
            asynchronous,
            boundary,
        ]
        .into_iter()
        .map(|node| engine.node_kind(node))
        .collect::<Result<BTreeSet<_>, _>>()?;
        assert_eq!(
            kinds,
            BTreeSet::from([
                ReactiveNodeKindV0::Input,
                ReactiveNodeKindV0::Map,
                ReactiveNodeKindV0::Zip,
                ReactiveNodeKindV0::Switch,
                ReactiveNodeKindV0::DeltaFold,
                ReactiveNodeKindV0::AsyncResult,
                ReactiveNodeKindV0::EffectBoundary,
            ])
        );
        Ok(())
    }

    #[test]
    fn deposits_arriving_mid_wave_wait_for_the_next_wave() -> Result<(), Box<dyn Error>> {
        let mut graph = ReactiveGraphBuilderV0::new();
        let input = graph.add_input(
            ReactiveStateV0::available(ReactiveValueV0::Counter(0)),
            exact_policy(),
        );
        let mapped = graph.add_map(input, counter_map, exact_policy());
        let boundary = graph.add_effect_boundary(mapped, "result", exact_policy());
        let mut engine = graph.build()?;
        engine.observe(boundary)?;
        engine.deposit(
            input,
            ReactiveStateV0::available(ReactiveValueV0::Counter(1)),
        )?;

        assert!(matches!(
            engine.stabilize_step(1)?,
            StabilizeStatusV0::Pending { .. }
        ));
        engine.deposit(
            input,
            ReactiveStateV0::available(ReactiveValueV0::Counter(2)),
        )?;
        settled(&mut engine)?;
        assert_eq!(
            engine.drain_effect_receipts(),
            vec![EffectReceiptV0 {
                channel: "result".to_string(),
                wave: 1,
                state: ReactiveStateV0::available(ReactiveValueV0::Counter(2)),
            }]
        );

        settled(&mut engine)?;
        assert_eq!(
            engine.drain_effect_receipts(),
            vec![EffectReceiptV0 {
                channel: "result".to_string(),
                wave: 2,
                state: ReactiveStateV0::available(ReactiveValueV0::Counter(3)),
            }]
        );
        Ok(())
    }

    #[test]
    fn delta_fold_is_checked_against_a_full_rebuild() -> Result<(), Box<dyn Error>> {
        let mut graph = ReactiveGraphBuilderV0::new();
        let first = graph.add_input(
            ReactiveStateV0::available(ReactiveValueV0::Counter(1)),
            exact_policy(),
        );
        let second = graph.add_input(
            ReactiveStateV0::available(ReactiveValueV0::Counter(2)),
            exact_policy(),
        );
        let fold = graph.add_delta_fold(
            vec![("first".to_string(), first), ("second".to_string(), second)],
            exact_policy(),
        )?;
        let mut engine = graph.build()?;
        engine.observe(fold)?;
        settled(&mut engine)?;
        engine.verify_delta_fold(fold)?;
        assert_eq!(engine.delta_update_count(fold)?, 2);

        engine.deposit(
            first,
            ReactiveStateV0::available(ReactiveValueV0::Counter(3)),
        )?;
        settled(&mut engine)?;
        engine.verify_delta_fold(fold)?;
        assert_eq!(
            engine.delta_update_count(fold)?,
            3,
            "only the changed key should be revisited after the initial rebuild"
        );

        engine.corrupt_delta_fold_entry_for_test(
            fold,
            "first",
            ReactiveStateV0::available(ReactiveValueV0::Counter(99)),
        );
        assert!(engine.verify_delta_fold(fold).is_err());
        Ok(())
    }

    #[test]
    fn unavailable_nodes_do_not_poison_independent_branches() -> Result<(), Box<dyn Error>> {
        let mut graph = ReactiveGraphBuilderV0::new();
        let unavailable = graph.add_async_result(
            ReactiveStateV0::unavailable("queryCancelled", "the query was cancelled"),
            exact_policy(),
        );
        let healthy = graph.add_input(
            ReactiveStateV0::available(ReactiveValueV0::Counter(7)),
            exact_policy(),
        );
        let failed_map = graph.add_map(unavailable, counter_map, exact_policy());
        let healthy_map = graph.add_map(healthy, counter_map, exact_policy());
        let mut engine = graph.build()?;
        engine.observe(failed_map)?;
        engine.observe(healthy_map)?;
        settled(&mut engine)?;

        assert!(matches!(
            engine.state(failed_map)?,
            ReactiveStateV0::Unavailable(_)
        ));
        assert_eq!(
            engine.state(healthy_map)?,
            &ReactiveStateV0::available(ReactiveValueV0::Counter(8))
        );
        Ok(())
    }

    #[test]
    fn unobserved_nodes_stay_stale_until_needed() -> Result<(), Box<dyn Error>> {
        let mut graph = ReactiveGraphBuilderV0::new();
        let input = graph.add_input(
            ReactiveStateV0::available(ReactiveValueV0::Counter(1)),
            exact_policy(),
        );
        let mapped = graph.add_map(input, counter_map, exact_policy());
        let mut engine = graph.build()?;
        engine.deposit(
            input,
            ReactiveStateV0::available(ReactiveValueV0::Counter(2)),
        )?;
        settled(&mut engine)?;
        assert!(engine.is_stale(mapped)?);
        assert_eq!(engine.node_recompute_count(mapped)?, 0);

        engine.observe(mapped)?;
        settled(&mut engine)?;
        assert!(!engine.is_stale(mapped)?);
        assert_eq!(engine.node_recompute_count(mapped)?, 1);
        Ok(())
    }

    #[test]
    fn bounded_steps_recompute_each_node_at_most_once_per_wave() -> Result<(), Box<dyn Error>> {
        let mut graph = ReactiveGraphBuilderV0::new();
        let input = graph.add_input(
            ReactiveStateV0::available(ReactiveValueV0::Counter(0)),
            exact_policy(),
        );
        let first = graph.add_map(input, counter_map, exact_policy());
        let second = graph.add_map(first, counter_map, exact_policy());
        let boundary = graph.add_effect_boundary(second, "result", exact_policy());
        let mut engine = graph.build()?;
        engine.observe(boundary)?;
        engine.deposit(
            input,
            ReactiveStateV0::available(ReactiveValueV0::Counter(1)),
        )?;

        let mut steps = 0;
        loop {
            steps += 1;
            if matches!(engine.stabilize_step(1)?, StabilizeStatusV0::Settled { .. }) {
                break;
            }
        }
        assert_eq!(steps, 3);
        assert_eq!(engine.node_recompute_count(first)?, 1);
        assert_eq!(engine.node_recompute_count(second)?, 1);
        assert_eq!(engine.node_recompute_count(boundary)?, 1);
        Ok(())
    }

    #[test]
    fn semantic_policy_avoids_spurious_refires_for_equal_allocations() -> Result<(), Box<dyn Error>>
    {
        let mut exact_graph = ReactiveGraphBuilderV0::new();
        let exact_input = exact_graph.add_input(revisioned_text("same", 1), exact_policy());
        let exact_map = exact_graph.add_map(exact_input, tuple_revision, exact_policy());
        let mut exact_engine = exact_graph.build()?;
        exact_engine.observe(exact_map)?;
        settled(&mut exact_engine)?;
        exact_engine.deposit(exact_input, revisioned_text("same", 1))?;
        settled(&mut exact_engine)?;
        assert_eq!(exact_engine.node_recompute_count(exact_map)?, 1);

        let mut identity_graph = ReactiveGraphBuilderV0::new();
        let identity_input = identity_graph.add_input(
            revisioned_text("same", 1),
            ChangePolicyV0::custom("allocationIdentity", always_different),
        );
        let identity_map = identity_graph.add_map(identity_input, tuple_revision, exact_policy());
        let mut identity_engine = identity_graph.build()?;
        identity_engine.observe(identity_map)?;
        settled(&mut identity_engine)?;
        identity_engine.deposit(identity_input, revisioned_text("same", 1))?;
        settled(&mut identity_engine)?;
        assert_eq!(identity_engine.node_recompute_count(identity_map)?, 2);
        Ok(())
    }

    #[test]
    fn semantic_policy_preserves_updates_hidden_by_partial_equality() -> Result<(), Box<dyn Error>>
    {
        let mut exact_graph = ReactiveGraphBuilderV0::new();
        let exact_input = exact_graph.add_input(revisioned_text("stable", 1), exact_policy());
        let exact_map = exact_graph.add_map(exact_input, tuple_revision, exact_policy());
        let mut exact_engine = exact_graph.build()?;
        exact_engine.observe(exact_map)?;
        settled(&mut exact_engine)?;
        exact_engine.deposit(exact_input, revisioned_text("stable", 2))?;
        settled(&mut exact_engine)?;
        assert_eq!(
            exact_engine.state(exact_map)?,
            &ReactiveStateV0::available(ReactiveValueV0::Counter(2))
        );

        let mut partial_graph = ReactiveGraphBuilderV0::new();
        let partial_input = partial_graph.add_input(
            revisioned_text("stable", 1),
            ChangePolicyV0::custom("textFieldOnly", text_field_only),
        );
        let partial_map = partial_graph.add_map(partial_input, tuple_revision, exact_policy());
        let mut partial_engine = partial_graph.build()?;
        partial_engine.observe(partial_map)?;
        settled(&mut partial_engine)?;
        partial_engine.deposit(partial_input, revisioned_text("stable", 2))?;
        settled(&mut partial_engine)?;
        assert_eq!(
            partial_engine.state(partial_map)?,
            &ReactiveStateV0::available(ReactiveValueV0::Counter(1))
        );
        Ok(())
    }
}
