use std::{
    collections::BTreeSet,
    error::Error,
    fmt,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{ChangePolicyV0, ReactiveEngineV0, ReactiveStateV0};

static NEXT_REACTIVE_GRAPH_ID_V0: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ReactiveGraphIdV0(u64);

impl ReactiveGraphIdV0 {
    fn fresh() -> Self {
        let id = match NEXT_REACTIVE_GRAPH_ID_V0.fetch_update(
            Ordering::Relaxed,
            Ordering::Relaxed,
            |next| next.checked_add(1),
        ) {
            Ok(id) => id,
            Err(_) => std::process::abort(),
        };
        Self(id)
    }

    fn value(self) -> u64 {
        self.0
    }
}

/// An append-only node index branded by the builder that minted it.
///
/// `Ord` is intentionally derived with `index` declared first, so iteration
/// order within one graph remains insertion order. Builders expose no removal
/// or recycling operation and `build(self)` consumes the builder, so node
/// generations are deliberately not represented.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReactiveNodeIdV0 {
    pub(crate) index: usize,
    pub(crate) graph: ReactiveGraphIdV0,
}

impl ReactiveNodeIdV0 {
    pub fn index(self) -> usize {
        self.index
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ReactiveNodeKindV0 {
    Input,
    Map,
    Zip,
    Switch,
    DeltaFold,
    AsyncResult,
    EffectBoundary,
}

pub type MapOperationV0 = fn(&ReactiveStateV0) -> ReactiveStateV0;
pub type ZipOperationV0 = fn(&ReactiveStateV0, &ReactiveStateV0) -> ReactiveStateV0;

#[derive(Clone)]
pub(crate) enum NodeOperationV0 {
    Input,
    Map { operation: MapOperationV0 },
    Zip { operation: ZipOperationV0 },
    Switch,
    DeltaFold { keys: Vec<String> },
    AsyncResult,
    EffectBoundary { channel: String },
}

impl NodeOperationV0 {
    pub(crate) fn kind(&self) -> ReactiveNodeKindV0 {
        match self {
            Self::Input => ReactiveNodeKindV0::Input,
            Self::Map { .. } => ReactiveNodeKindV0::Map,
            Self::Zip { .. } => ReactiveNodeKindV0::Zip,
            Self::Switch => ReactiveNodeKindV0::Switch,
            Self::DeltaFold { .. } => ReactiveNodeKindV0::DeltaFold,
            Self::AsyncResult => ReactiveNodeKindV0::AsyncResult,
            Self::EffectBoundary { .. } => ReactiveNodeKindV0::EffectBoundary,
        }
    }
}

#[derive(Clone)]
pub(crate) struct NodeBlueprintV0 {
    pub(crate) operation: NodeOperationV0,
    pub(crate) dependencies: Vec<ReactiveNodeIdV0>,
    pub(crate) height: u32,
    pub(crate) initial_state: ReactiveStateV0,
    pub(crate) change_policy: ChangePolicyV0,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ReactiveGraphBuildErrorV0 {
    #[non_exhaustive]
    EmptyChangePolicyName { node_index: usize },
    #[non_exhaustive]
    DuplicateDeltaKey { key: String },
    #[non_exhaustive]
    ForeignNodeId {
        node_index: usize,
        expected_graph: u64,
        actual_graph: u64,
    },
}

impl fmt::Display for ReactiveGraphBuildErrorV0 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyChangePolicyName { node_index } => {
                write!(
                    formatter,
                    "node {node_index} has an empty change-policy name"
                )
            }
            Self::DuplicateDeltaKey { key } => {
                write!(formatter, "delta-fold key `{key}` is duplicated")
            }
            Self::ForeignNodeId {
                node_index,
                expected_graph,
                actual_graph,
            } => write!(
                formatter,
                "node {node_index} belongs to reactive graph {actual_graph}, not {expected_graph}"
            ),
        }
    }
}

impl Error for ReactiveGraphBuildErrorV0 {}

pub struct ReactiveGraphBuilderV0 {
    graph_id: ReactiveGraphIdV0,
    nodes: Vec<NodeBlueprintV0>,
    foreign_dependency: Option<(usize, ReactiveGraphIdV0)>,
}

impl Default for ReactiveGraphBuilderV0 {
    fn default() -> Self {
        Self::new()
    }
}

impl ReactiveGraphBuilderV0 {
    pub fn new() -> Self {
        Self {
            graph_id: ReactiveGraphIdV0::fresh(),
            nodes: Vec::new(),
            foreign_dependency: None,
        }
    }

    pub fn add_input(
        &mut self,
        initial_state: ReactiveStateV0,
        change_policy: ChangePolicyV0,
    ) -> ReactiveNodeIdV0 {
        self.push(
            NodeOperationV0::Input,
            Vec::new(),
            initial_state,
            change_policy,
        )
    }

    pub fn add_async_result(
        &mut self,
        initial_state: ReactiveStateV0,
        change_policy: ChangePolicyV0,
    ) -> ReactiveNodeIdV0 {
        self.push(
            NodeOperationV0::AsyncResult,
            Vec::new(),
            initial_state,
            change_policy,
        )
    }

    pub fn add_map(
        &mut self,
        dependency: ReactiveNodeIdV0,
        operation: MapOperationV0,
        change_policy: ChangePolicyV0,
    ) -> ReactiveNodeIdV0 {
        self.push(
            NodeOperationV0::Map { operation },
            vec![dependency],
            ReactiveStateV0::pending(),
            change_policy,
        )
    }

    pub fn add_zip(
        &mut self,
        left: ReactiveNodeIdV0,
        right: ReactiveNodeIdV0,
        operation: ZipOperationV0,
        change_policy: ChangePolicyV0,
    ) -> ReactiveNodeIdV0 {
        self.push(
            NodeOperationV0::Zip { operation },
            vec![left, right],
            ReactiveStateV0::pending(),
            change_policy,
        )
    }

    /// Adds a static two-branch switch. Branch identities cannot be changed
    /// after the graph is built.
    pub fn add_switch(
        &mut self,
        selector: ReactiveNodeIdV0,
        when_false: ReactiveNodeIdV0,
        when_true: ReactiveNodeIdV0,
        change_policy: ChangePolicyV0,
    ) -> ReactiveNodeIdV0 {
        self.push(
            NodeOperationV0::Switch,
            vec![selector, when_false, when_true],
            ReactiveStateV0::pending(),
            change_policy,
        )
    }

    pub fn add_delta_fold(
        &mut self,
        entries: Vec<(String, ReactiveNodeIdV0)>,
        change_policy: ChangePolicyV0,
    ) -> Result<ReactiveNodeIdV0, ReactiveGraphBuildErrorV0> {
        if let Some((_, dependency)) = entries
            .iter()
            .find(|(_, dependency)| dependency.graph != self.graph_id)
        {
            return Err(ReactiveGraphBuildErrorV0::ForeignNodeId {
                node_index: self.nodes.len(),
                expected_graph: self.graph_id.value(),
                actual_graph: dependency.graph.value(),
            });
        }
        let mut seen = BTreeSet::new();
        for (key, _) in &entries {
            if !seen.insert(key.clone()) {
                return Err(ReactiveGraphBuildErrorV0::DuplicateDeltaKey { key: key.clone() });
            }
        }
        let (keys, dependencies): (Vec<_>, Vec<_>) = entries.into_iter().unzip();
        Ok(self.push(
            NodeOperationV0::DeltaFold { keys },
            dependencies,
            ReactiveStateV0::pending(),
            change_policy,
        ))
    }

    pub fn add_effect_boundary(
        &mut self,
        dependency: ReactiveNodeIdV0,
        channel: impl Into<String>,
        change_policy: ChangePolicyV0,
    ) -> ReactiveNodeIdV0 {
        self.push(
            NodeOperationV0::EffectBoundary {
                channel: channel.into(),
            },
            vec![dependency],
            ReactiveStateV0::pending(),
            change_policy,
        )
    }

    pub fn build(self) -> Result<ReactiveEngineV0, ReactiveGraphBuildErrorV0> {
        let Self {
            graph_id,
            nodes,
            foreign_dependency,
        } = self;
        if let Some((node_index, actual_graph)) = foreign_dependency {
            return Err(ReactiveGraphBuildErrorV0::ForeignNodeId {
                node_index,
                expected_graph: graph_id.value(),
                actual_graph: actual_graph.value(),
            });
        }
        for (node_index, node) in nodes.iter().enumerate() {
            if node.change_policy.name().is_empty() {
                return Err(ReactiveGraphBuildErrorV0::EmptyChangePolicyName { node_index });
            }
        }
        Ok(ReactiveEngineV0::from_blueprints(graph_id, nodes))
    }

    fn push(
        &mut self,
        operation: NodeOperationV0,
        dependencies: Vec<ReactiveNodeIdV0>,
        initial_state: ReactiveStateV0,
        change_policy: ChangePolicyV0,
    ) -> ReactiveNodeIdV0 {
        let node_index = self.nodes.len();
        if self.foreign_dependency.is_none() {
            self.foreign_dependency = dependencies
                .iter()
                .find(|dependency| dependency.graph != self.graph_id)
                .map(|dependency| (node_index, dependency.graph));
        }
        debug_assert!(
            dependencies.iter().all(|dependency| {
                dependency.graph != self.graph_id || dependency.index() < self.nodes.len()
            }),
            "same-graph dependencies must refer to earlier nodes; graph ownership is checked separately"
        );
        let height = dependencies
            .iter()
            .filter(|dependency| dependency.graph == self.graph_id)
            .filter_map(|dependency| self.nodes.get(dependency.index()))
            .map(|dependency| dependency.height)
            .max()
            .map_or(0, |height| height.saturating_add(1));
        let id = ReactiveNodeIdV0 {
            index: node_index,
            graph: self.graph_id,
        };
        self.nodes.push(NodeBlueprintV0 {
            operation,
            dependencies,
            height,
            initial_state,
            change_policy,
        });
        id
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::ReactiveValueV0;

    fn exact_policy() -> ChangePolicyV0 {
        ChangePolicyV0::exact("graphOwnershipTestValue")
    }

    fn counter(value: u64) -> ReactiveStateV0 {
        ReactiveStateV0::available(ReactiveValueV0::Counter(value))
    }

    fn identity(state: &ReactiveStateV0) -> ReactiveStateV0 {
        state.clone()
    }

    #[test]
    fn graph_build_rejects_a_dependency_from_another_builder() {
        let mut first = ReactiveGraphBuilderV0::new();
        first.add_input(counter(1), exact_policy());

        let mut second = ReactiveGraphBuilderV0::new();
        let foreign = second.add_input(counter(2), exact_policy());

        first.add_map(foreign, identity, exact_policy());
        assert!(matches!(
            first.build(),
            Err(ReactiveGraphBuildErrorV0::ForeignNodeId {
                node_index: 1,
                expected_graph,
                actual_graph,
            }) if expected_graph != actual_graph
        ));
    }

    #[test]
    fn structurally_identical_graphs_still_have_distinct_identities() {
        let mut first = ReactiveGraphBuilderV0::new();
        first.add_input(counter(1), exact_policy());

        let mut second = ReactiveGraphBuilderV0::new();
        let foreign = second.add_input(counter(1), exact_policy());

        first.add_map(foreign, identity, exact_policy());
        assert!(matches!(
            first.build(),
            Err(ReactiveGraphBuildErrorV0::ForeignNodeId {
                node_index: 1,
                expected_graph,
                actual_graph,
            }) if expected_graph != actual_graph
        ));
    }

    #[test]
    fn delta_fold_rejects_a_foreign_dependency_immediately() {
        let mut first = ReactiveGraphBuilderV0::new();
        first.add_input(counter(1), exact_policy());

        let mut second = ReactiveGraphBuilderV0::new();
        let foreign = second.add_input(counter(2), exact_policy());

        assert!(matches!(
            first.add_delta_fold(vec![("foreign".to_string(), foreign)], exact_policy()),
            Err(ReactiveGraphBuildErrorV0::ForeignNodeId {
                node_index: 1,
                expected_graph,
                actual_graph,
            }) if expected_graph != actual_graph
        ));
    }

    #[test]
    fn derived_node_order_preserves_insertion_order_within_one_graph() {
        let mut graph = ReactiveGraphBuilderV0::new();
        let ids = (0..4)
            .map(|value| graph.add_input(counter(value), exact_policy()))
            .collect::<BTreeSet<_>>();

        assert_eq!(
            ids.into_iter()
                .map(ReactiveNodeIdV0::index)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3]
        );
    }
}
