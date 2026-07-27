use std::{collections::BTreeSet, error::Error, fmt};

use crate::{ChangePolicyV0, ReactiveEngineV0, ReactiveStateV0};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReactiveNodeIdV0(pub(crate) usize);

impl ReactiveNodeIdV0 {
    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
pub enum ReactiveGraphBuildErrorV0 {
    EmptyChangePolicyName { node_index: usize },
    DuplicateDeltaKey { key: String },
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
        }
    }
}

impl Error for ReactiveGraphBuildErrorV0 {}

#[derive(Default)]
pub struct ReactiveGraphBuilderV0 {
    nodes: Vec<NodeBlueprintV0>,
}

impl ReactiveGraphBuilderV0 {
    pub fn new() -> Self {
        Self::default()
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
        for (node_index, node) in self.nodes.iter().enumerate() {
            if node.change_policy.name().is_empty() {
                return Err(ReactiveGraphBuildErrorV0::EmptyChangePolicyName { node_index });
            }
        }
        Ok(ReactiveEngineV0::from_blueprints(self.nodes))
    }

    fn push(
        &mut self,
        operation: NodeOperationV0,
        dependencies: Vec<ReactiveNodeIdV0>,
        initial_state: ReactiveStateV0,
        change_policy: ChangePolicyV0,
    ) -> ReactiveNodeIdV0 {
        debug_assert!(
            dependencies
                .iter()
                .all(|dependency| dependency.index() < self.nodes.len()),
            "static graph dependencies must refer to earlier nodes"
        );
        let height = dependencies
            .iter()
            .map(|dependency| self.nodes[dependency.index()].height)
            .max()
            .map_or(0, |height| height.saturating_add(1));
        let id = ReactiveNodeIdV0(self.nodes.len());
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
