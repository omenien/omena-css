//! Canonical decision diagrams and the shared first-witness fold.
//!
//! This module deliberately imports no type from its host crate. The boolean
//! terminal core is extended in place by the later cascade-winner terminal
//! alphabet without coupling either plane to the surrounding cascade model.

use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

pub type NodeId = u32;

pub const FALSE_NODE_ID_V0: NodeId = 0;
pub const TRUE_NODE_ID_V0: NodeId = 1;
pub const DEFAULT_APPLY_CACHE_CAPACITY_V0: usize = 4_096;
pub const DEFAULT_REBUILD_INTERVAL_OPERATIONS_V0: u64 = 8_192;
pub const SITE_FIRST_APPEARANCE_ORDERING_DOMAIN_V0: &str = "siteFirstAppearance";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Node {
    Term(u32),
    Int { var: u16, lo: NodeId, hi: NodeId },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BooleanOperationV0 {
    And,
    Or,
    Xor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableOrderDomainV0 {
    SiteFirstAppearance,
}

impl VariableOrderDomainV0 {
    pub const fn name(self) -> &'static str {
        match self {
            Self::SiteFirstAppearance => SITE_FIRST_APPEARANCE_ORDERING_DOMAIN_V0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableOrderRegistrationV0 {
    domain: VariableOrderDomainV0,
    atoms: Vec<String>,
    indices: BTreeMap<String, u16>,
}

impl VariableOrderRegistrationV0 {
    pub fn site_first_appearance(
        atoms: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, FirstWitnessErrorV0> {
        let mut ordered = Vec::new();
        let mut seen = BTreeSet::new();
        for atom in atoms {
            let atom = atom.into();
            if seen.insert(atom.clone()) {
                ordered.push(atom);
            }
        }
        let indices = ordered
            .iter()
            .enumerate()
            .map(|(index, atom)| {
                u16::try_from(index)
                    .map(|index| (atom.clone(), index))
                    .map_err(|_| FirstWitnessErrorV0::VariableCapacityExceeded)
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        Ok(Self {
            domain: VariableOrderDomainV0::SiteFirstAppearance,
            atoms: ordered,
            indices,
        })
    }

    pub const fn domain(&self) -> VariableOrderDomainV0 {
        self.domain
    }

    pub fn atoms(&self) -> &[String] {
        &self.atoms
    }

    pub fn variable_index(&self, atom: &str) -> Option<u16> {
        self.indices.get(atom).copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FirstWitnessManagerConfigV0 {
    pub apply_cache_capacity: usize,
    pub rebuild_interval_operations: u64,
    pub shortcuts: bool,
}

impl Default for FirstWitnessManagerConfigV0 {
    fn default() -> Self {
        Self {
            apply_cache_capacity: DEFAULT_APPLY_CACHE_CAPACITY_V0,
            rebuild_interval_operations: DEFAULT_REBUILD_INTERVAL_OPERATIONS_V0,
            shortcuts: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FirstWitnessOperationCountersV0 {
    pub choose_invocations: u64,
    pub apply_invocations: u64,
    pub apply_cache_lookups: u64,
    pub apply_cache_hits: u64,
    pub rebuilds: u64,
    pub rebuild_node_visits: u64,
}

impl FirstWitnessOperationCountersV0 {
    pub const fn recursive_operations(self) -> u64 {
        self.choose_invocations + self.apply_invocations
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FirstWitnessRebuildReportV0 {
    pub operations_since_previous_rebuild: u64,
    pub nodes_before: usize,
    pub nodes_after: usize,
    pub live_root_count: usize,
    pub visited_node_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FirstWitnessErrorV0 {
    UnknownAtom(String),
    InvalidNode(NodeId),
    InvalidTerminal(u32),
    VariableOrderViolation { parent: u16, child: u16 },
    VariableCapacityExceeded,
}

impl std::fmt::Display for FirstWitnessErrorV0 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownAtom(atom) => write!(formatter, "unregistered decision atom {atom}"),
            Self::InvalidNode(node) => write!(formatter, "invalid decision node {node}"),
            Self::InvalidTerminal(terminal) => {
                write!(formatter, "invalid boolean terminal {terminal}")
            }
            Self::VariableOrderViolation { parent, child } => write!(
                formatter,
                "decision variable order violation: parent {parent}, child {child}"
            ),
            Self::VariableCapacityExceeded => {
                formatter.write_str("decision variable or node capacity exceeded")
            }
        }
    }
}

impl std::error::Error for FirstWitnessErrorV0 {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ApplyCacheKeyV0 {
    operation: BooleanOperationV0,
    left: NodeId,
    right: NodeId,
}

#[derive(Debug, Clone)]
pub struct FirstWitnessManagerV0 {
    nodes: Vec<Node>,
    unique: HashMap<(u16, NodeId, NodeId), NodeId>,
    apply_cache: HashMap<ApplyCacheKeyV0, NodeId>,
    apply_cache_fifo: VecDeque<ApplyCacheKeyV0>,
    order: VariableOrderRegistrationV0,
    config: FirstWitnessManagerConfigV0,
    counters: FirstWitnessOperationCountersV0,
    operations_at_previous_rebuild: u64,
}

impl FirstWitnessManagerV0 {
    pub fn new(order: VariableOrderRegistrationV0, config: FirstWitnessManagerConfigV0) -> Self {
        Self {
            nodes: vec![Node::Term(0), Node::Term(1)],
            unique: HashMap::new(),
            apply_cache: HashMap::new(),
            apply_cache_fifo: VecDeque::new(),
            order,
            config,
            counters: FirstWitnessOperationCountersV0::default(),
            operations_at_previous_rebuild: 0,
        }
    }

    pub fn order(&self) -> &VariableOrderRegistrationV0 {
        &self.order
    }

    pub const fn config(&self) -> FirstWitnessManagerConfigV0 {
        self.config
    }

    pub const fn counters(&self) -> FirstWitnessOperationCountersV0 {
        self.counters
    }

    pub fn node(&self, node: NodeId) -> Option<Node> {
        self.nodes.get(node as usize).copied()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn unique_table_len(&self) -> usize {
        self.unique.len()
    }

    pub fn apply_cache_len(&self) -> usize {
        self.apply_cache.len()
    }

    pub fn variable(&mut self, atom: &str) -> Result<NodeId, FirstWitnessErrorV0> {
        let variable = self
            .order
            .variable_index(atom)
            .ok_or_else(|| FirstWitnessErrorV0::UnknownAtom(atom.to_string()))?;
        self.choose(variable, FALSE_NODE_ID_V0, TRUE_NODE_ID_V0)
    }

    pub fn choose(
        &mut self,
        variable: u16,
        low: NodeId,
        high: NodeId,
    ) -> Result<NodeId, FirstWitnessErrorV0> {
        self.counters.choose_invocations += 1;
        self.intern(variable, low, high)
    }

    pub fn not(&mut self, value: NodeId) -> Result<NodeId, FirstWitnessErrorV0> {
        self.apply(BooleanOperationV0::Xor, value, TRUE_NODE_ID_V0)
    }

    pub fn and(&mut self, left: NodeId, right: NodeId) -> Result<NodeId, FirstWitnessErrorV0> {
        self.apply(BooleanOperationV0::And, left, right)
    }

    pub fn or(&mut self, left: NodeId, right: NodeId) -> Result<NodeId, FirstWitnessErrorV0> {
        self.apply(BooleanOperationV0::Or, left, right)
    }

    pub fn xor(&mut self, left: NodeId, right: NodeId) -> Result<NodeId, FirstWitnessErrorV0> {
        self.apply(BooleanOperationV0::Xor, left, right)
    }

    pub fn apply(
        &mut self,
        operation: BooleanOperationV0,
        left: NodeId,
        right: NodeId,
    ) -> Result<NodeId, FirstWitnessErrorV0> {
        self.require_node(left)?;
        self.require_node(right)?;
        self.apply_recursive(operation, left, right)
    }

    pub fn is_tautology(&self, root: NodeId) -> bool {
        root == TRUE_NODE_ID_V0
    }

    pub fn is_satisfiable(&self, root: NodeId) -> bool {
        root != FALSE_NODE_ID_V0
    }

    pub fn reclaim_if_due(
        &mut self,
        live_roots: &mut [NodeId],
    ) -> Result<Option<FirstWitnessRebuildReportV0>, FirstWitnessErrorV0> {
        let operations = self.counters.recursive_operations();
        let operations_since_previous_rebuild =
            operations.saturating_sub(self.operations_at_previous_rebuild);
        if self.config.rebuild_interval_operations == 0
            || operations_since_previous_rebuild < self.config.rebuild_interval_operations
        {
            return Ok(None);
        }
        for root in live_roots.iter().copied() {
            self.require_node(root)?;
        }
        let nodes_before = self.nodes.len();
        let mut rebuilt_nodes = vec![Node::Term(0), Node::Term(1)];
        let mut rebuilt_unique = HashMap::new();
        let mut remapped = HashMap::from([
            (FALSE_NODE_ID_V0, FALSE_NODE_ID_V0),
            (TRUE_NODE_ID_V0, TRUE_NODE_ID_V0),
        ]);
        let mut visited_node_count = 0usize;
        for root in live_roots.iter_mut() {
            *root = clone_live_node(
                *root,
                &self.nodes,
                &mut rebuilt_nodes,
                &mut rebuilt_unique,
                &mut remapped,
                &mut visited_node_count,
            )?;
        }
        self.nodes = rebuilt_nodes;
        self.unique = rebuilt_unique;
        self.apply_cache.clear();
        self.apply_cache_fifo.clear();
        self.counters.rebuilds += 1;
        self.counters.rebuild_node_visits += visited_node_count as u64;
        self.operations_at_previous_rebuild = operations;
        Ok(Some(FirstWitnessRebuildReportV0 {
            operations_since_previous_rebuild,
            nodes_before,
            nodes_after: self.nodes.len(),
            live_root_count: live_roots.len(),
            visited_node_count,
        }))
    }

    fn apply_recursive(
        &mut self,
        operation: BooleanOperationV0,
        left: NodeId,
        right: NodeId,
    ) -> Result<NodeId, FirstWitnessErrorV0> {
        self.counters.apply_invocations += 1;
        if self.config.shortcuts
            && let Some(result) = boolean_shortcut(operation, left, right)
        {
            return Ok(result);
        }
        let left_node = self.require_node(left)?;
        let right_node = self.require_node(right)?;
        if let (Node::Term(left), Node::Term(right)) = (left_node, right_node) {
            return terminal_boolean_result(operation, left, right);
        }
        let key = canonical_apply_key(operation, left, right);
        self.counters.apply_cache_lookups += 1;
        if let Some(result) = self.apply_cache.get(&key).copied() {
            self.counters.apply_cache_hits += 1;
            return Ok(result);
        }
        let variable = top_variable(left_node, right_node);
        let (left_low, left_high) = cofactors(left, left_node, variable);
        let (right_low, right_high) = cofactors(right, right_node, variable);
        let low = self.apply_recursive(operation, left_low, right_low)?;
        let high = self.apply_recursive(operation, left_high, right_high)?;
        let result = self.choose(variable, low, high)?;
        self.cache_insert(key, result);
        Ok(result)
    }

    fn intern(
        &mut self,
        variable: u16,
        low: NodeId,
        high: NodeId,
    ) -> Result<NodeId, FirstWitnessErrorV0> {
        let low_node = self.require_node(low)?;
        let high_node = self.require_node(high)?;
        for child in [low_node, high_node] {
            if let Node::Int { var: child, .. } = child
                && child <= variable
            {
                return Err(FirstWitnessErrorV0::VariableOrderViolation {
                    parent: variable,
                    child,
                });
            }
        }
        if low == high {
            return Ok(low);
        }
        if let Some(node) = self.unique.get(&(variable, low, high)).copied() {
            return Ok(node);
        }
        let node = u32::try_from(self.nodes.len())
            .map_err(|_| FirstWitnessErrorV0::VariableCapacityExceeded)?;
        self.nodes.push(Node::Int {
            var: variable,
            lo: low,
            hi: high,
        });
        self.unique.insert((variable, low, high), node);
        Ok(node)
    }

    fn require_node(&self, node: NodeId) -> Result<Node, FirstWitnessErrorV0> {
        self.node(node)
            .ok_or(FirstWitnessErrorV0::InvalidNode(node))
    }

    fn cache_insert(&mut self, key: ApplyCacheKeyV0, value: NodeId) {
        if self.config.apply_cache_capacity == 0 || self.apply_cache.contains_key(&key) {
            return;
        }
        while self.apply_cache.len() >= self.config.apply_cache_capacity {
            let Some(evicted) = self.apply_cache_fifo.pop_front() else {
                break;
            };
            self.apply_cache.remove(&evicted);
        }
        self.apply_cache.insert(key, value);
        self.apply_cache_fifo.push_back(key);
    }
}

pub fn merge_first_witness_by_key_v0<T: Clone, K: Ord>(
    left: &[T],
    right: &[T],
    key: impl Fn(&T) -> K,
) -> Vec<T> {
    let mut seen = BTreeSet::new();
    left.iter()
        .chain(right)
        .filter(|value| seen.insert(key(value)))
        .cloned()
        .collect()
}

pub fn first_witness_fold_v0<T: Clone + Ord>(left: &[T], right: &[T]) -> Vec<T> {
    left.iter()
        .chain(right)
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn boolean_shortcut(operation: BooleanOperationV0, left: NodeId, right: NodeId) -> Option<NodeId> {
    match operation {
        BooleanOperationV0::And => {
            if left == FALSE_NODE_ID_V0 || right == FALSE_NODE_ID_V0 {
                Some(FALSE_NODE_ID_V0)
            } else if left == TRUE_NODE_ID_V0 {
                Some(right)
            } else if right == TRUE_NODE_ID_V0 || left == right {
                Some(left)
            } else {
                None
            }
        }
        BooleanOperationV0::Or => {
            if left == TRUE_NODE_ID_V0 || right == TRUE_NODE_ID_V0 {
                Some(TRUE_NODE_ID_V0)
            } else if left == FALSE_NODE_ID_V0 {
                Some(right)
            } else if right == FALSE_NODE_ID_V0 || left == right {
                Some(left)
            } else {
                None
            }
        }
        BooleanOperationV0::Xor => {
            if left == right {
                Some(FALSE_NODE_ID_V0)
            } else if left == FALSE_NODE_ID_V0 {
                Some(right)
            } else if right == FALSE_NODE_ID_V0 {
                Some(left)
            } else {
                None
            }
        }
    }
}

fn terminal_boolean_result(
    operation: BooleanOperationV0,
    left: u32,
    right: u32,
) -> Result<NodeId, FirstWitnessErrorV0> {
    if left > 1 {
        return Err(FirstWitnessErrorV0::InvalidTerminal(left));
    }
    if right > 1 {
        return Err(FirstWitnessErrorV0::InvalidTerminal(right));
    }
    let left = left == 1;
    let right = right == 1;
    Ok(match operation {
        BooleanOperationV0::And => left && right,
        BooleanOperationV0::Or => left || right,
        BooleanOperationV0::Xor => left ^ right,
    } as NodeId)
}

fn canonical_apply_key(
    operation: BooleanOperationV0,
    left: NodeId,
    right: NodeId,
) -> ApplyCacheKeyV0 {
    let (left, right) = if left <= right {
        (left, right)
    } else {
        (right, left)
    };
    ApplyCacheKeyV0 {
        operation,
        left,
        right,
    }
}

fn top_variable(left: Node, right: Node) -> u16 {
    match (left, right) {
        (Node::Int { var: left, .. }, Node::Int { var: right, .. }) => left.min(right),
        (Node::Int { var, .. }, Node::Term(_)) | (Node::Term(_), Node::Int { var, .. }) => var,
        (Node::Term(_), Node::Term(_)) => {
            unreachable!("terminal pairs are handled before recursion")
        }
    }
}

fn cofactors(node_id: NodeId, node: Node, variable: u16) -> (NodeId, NodeId) {
    match node {
        Node::Int { var, lo, hi } if var == variable => (lo, hi),
        _ => (node_id, node_id),
    }
}

fn clone_live_node(
    old: NodeId,
    old_nodes: &[Node],
    new_nodes: &mut Vec<Node>,
    new_unique: &mut HashMap<(u16, NodeId, NodeId), NodeId>,
    remapped: &mut HashMap<NodeId, NodeId>,
    visited: &mut usize,
) -> Result<NodeId, FirstWitnessErrorV0> {
    if let Some(mapped) = remapped.get(&old).copied() {
        return Ok(mapped);
    }
    *visited += 1;
    let node = old_nodes
        .get(old as usize)
        .copied()
        .ok_or(FirstWitnessErrorV0::InvalidNode(old))?;
    let (var, lo, hi) = match node {
        Node::Int { var, lo, hi } => (var, lo, hi),
        Node::Term(terminal) => return Err(FirstWitnessErrorV0::InvalidTerminal(terminal)),
    };
    let low = clone_live_node(lo, old_nodes, new_nodes, new_unique, remapped, visited)?;
    let high = clone_live_node(hi, old_nodes, new_nodes, new_unique, remapped, visited)?;
    let mapped = if low == high {
        low
    } else if let Some(node) = new_unique.get(&(var, low, high)).copied() {
        node
    } else {
        let node = u32::try_from(new_nodes.len())
            .map_err(|_| FirstWitnessErrorV0::VariableCapacityExceeded)?;
        new_nodes.push(Node::Int {
            var,
            lo: low,
            hi: high,
        });
        new_unique.insert((var, low, high), node);
        node
    };
    remapped.insert(old, mapped);
    Ok(mapped)
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    fn manager(shortcuts: bool) -> Result<FirstWitnessManagerV0, FirstWitnessErrorV0> {
        Ok(FirstWitnessManagerV0::new(
            VariableOrderRegistrationV0::site_first_appearance(["a", "b", "c"])?,
            FirstWitnessManagerConfigV0 {
                shortcuts,
                apply_cache_capacity: 32,
                rebuild_interval_operations: 64,
            },
        ))
    }

    #[test]
    fn canonical_nodes_identify_functions_both_ways() -> Result<(), FirstWitnessErrorV0> {
        let mut manager = manager(true)?;
        let a = manager.variable("a")?;
        let b = manager.variable("b")?;
        let a_and_b = manager.and(a, b)?;
        let b_and_a = manager.and(b, a)?;
        let a_or_b = manager.or(a, b)?;
        assert_eq!(a_and_b, b_and_a, "same function must share one node");
        assert_ne!(
            a_and_b, a_or_b,
            "distinct functions must not share one node"
        );
        Ok(())
    }

    #[test]
    fn contradiction_and_excluded_middle_reduce_to_terminals() -> Result<(), FirstWitnessErrorV0> {
        let mut manager = manager(true)?;
        let condition = manager.variable("c")?;
        let negated = manager.not(condition)?;
        let contradiction = manager.and(condition, negated)?;
        let excluded_middle = manager.or(condition, negated)?;
        assert_eq!(contradiction, FALSE_NODE_ID_V0);
        assert_eq!(excluded_middle, TRUE_NODE_ID_V0);
        Ok(())
    }

    #[test]
    fn shortcut_switch_changes_work_not_results() -> Result<(), FirstWitnessErrorV0> {
        fn fixed_seed(
            shortcuts: bool,
        ) -> Result<(NodeId, FirstWitnessOperationCountersV0), FirstWitnessErrorV0> {
            let mut manager = manager(shortcuts)?;
            let a = manager.variable("a")?;
            let b = manager.variable("b")?;
            let shared = manager.or(a, b)?;
            let result = manager.and(shared, shared)?;
            Ok((result, manager.counters()))
        }
        let (shortcut_result, shortcut_counts) = fixed_seed(true)?;
        let (recursive_result, recursive_counts) = fixed_seed(false)?;
        assert_eq!(shortcut_result, recursive_result);
        assert!(recursive_counts.apply_invocations > shortcut_counts.apply_invocations);
        assert!(recursive_counts.apply_cache_lookups > shortcut_counts.apply_cache_lookups);
        Ok(())
    }

    #[test]
    fn boolean_laws_recompute_with_shortcuts_disabled() -> Result<(), FirstWitnessErrorV0> {
        let mut manager = manager(false)?;
        let a = manager.variable("a")?;
        let b = manager.variable("b")?;
        let c = manager.variable("c")?;
        let a_and_b = manager.and(a, b)?;
        let b_and_c = manager.and(b, c)?;
        let left_associative = manager.and(a_and_b, c)?;
        let right_associative = manager.and(a, b_and_c)?;
        assert_eq!(left_associative, right_associative);
        assert_eq!(manager.and(a, a)?, a);
        let a_or_b = manager.or(a, b)?;
        assert_eq!(manager.and(a, a_or_b)?, a);
        let not_a = manager.not(a)?;
        assert_eq!(manager.and(a, not_a)?, FALSE_NODE_ID_V0);
        assert_eq!(manager.or(a, not_a)?, TRUE_NODE_ID_V0);
        Ok(())
    }

    #[test]
    fn apply_cache_capacity_is_a_live_bound() -> Result<(), FirstWitnessErrorV0> {
        let mut manager = FirstWitnessManagerV0::new(
            VariableOrderRegistrationV0::site_first_appearance(["a", "b", "c"])?,
            FirstWitnessManagerConfigV0 {
                shortcuts: false,
                apply_cache_capacity: 2,
                rebuild_interval_operations: u64::MAX,
            },
        );
        let a = manager.variable("a")?;
        let b = manager.variable("b")?;
        let c = manager.variable("c")?;
        let _ = manager.and(a, b)?;
        let _ = manager.or(a, c)?;
        let _ = manager.xor(b, c)?;
        assert!(manager.apply_cache_len() <= 2);
        assert_eq!(manager.config().apply_cache_capacity, 2);
        Ok(())
    }

    #[test]
    fn rebuild_reclaims_unreachable_nodes_and_remaps_live_roots() -> Result<(), FirstWitnessErrorV0>
    {
        let mut manager = FirstWitnessManagerV0::new(
            VariableOrderRegistrationV0::site_first_appearance(["a", "b", "c"])?,
            FirstWitnessManagerConfigV0 {
                shortcuts: false,
                apply_cache_capacity: 16,
                rebuild_interval_operations: 1,
            },
        );
        let a = manager.variable("a")?;
        let b = manager.variable("b")?;
        let c = manager.variable("c")?;
        let live = manager.and(a, b)?;
        let _dead = manager.or(a, c)?;
        let before = manager.node_count();
        let mut roots = [live];
        let report = manager.reclaim_if_due(&mut roots)?;
        assert!(report.is_some(), "rebuild interval reached");
        let Some(report) = report else {
            return Ok(());
        };
        assert!(manager.node_count() < before);
        assert_eq!(
            manager.node(roots[0]),
            Some(Node::Int {
                var: 0,
                lo: 0,
                hi: 2
            })
        );
        assert_eq!(report.nodes_after, manager.node_count());
        assert_eq!(manager.counters().rebuilds, 1);
        Ok(())
    }

    #[test]
    fn site_first_appearance_policy_pins_synthetic_blocked_pair_bound()
    -> Result<(), FirstWitnessErrorV0> {
        const PAIRS: usize = 7;
        fn build(order: Vec<String>) -> Result<usize, FirstWitnessErrorV0> {
            let mut manager = FirstWitnessManagerV0::new(
                VariableOrderRegistrationV0::site_first_appearance(order)?,
                FirstWitnessManagerConfigV0 {
                    shortcuts: false,
                    apply_cache_capacity: 4_096,
                    rebuild_interval_operations: u64::MAX,
                },
            );
            let mut root = TRUE_NODE_ID_V0;
            for index in 0..PAIRS {
                let left = manager.variable(&format!("x{index}"))?;
                let right = manager.variable(&format!("y{index}"))?;
                let pair = manager.xor(left, right)?;
                root = manager.and(root, pair)?;
            }
            assert!(manager.is_satisfiable(root));
            Ok(manager.node_count())
        }
        let interleaved = (0..PAIRS)
            .flat_map(|index| [format!("x{index}"), format!("y{index}")])
            .collect();
        let blocked = (0..PAIRS)
            .map(|index| format!("x{index}"))
            .chain((0..PAIRS).map(|index| format!("y{index}")))
            .collect();
        let interleaved_nodes = build(interleaved)?;
        let blocked_nodes = build(blocked)?;
        assert!(
            interleaved_nodes <= 14 * PAIRS,
            "interleaved={interleaved_nodes}, blocked={blocked_nodes}"
        );
        assert!(
            blocked_nodes >= interleaved_nodes * 8,
            "interleaved={interleaved_nodes}, blocked={blocked_nodes}"
        );
        Ok(())
    }

    #[test]
    fn first_witness_fold_is_commutative_and_idempotent() {
        let left = vec!["alpha", "shared"];
        let right = vec!["beta", "shared"];
        assert_eq!(
            first_witness_fold_v0(&left, &right),
            first_witness_fold_v0(&right, &left)
        );
        assert_eq!(first_witness_fold_v0(&left, &left), left);
    }

    #[test]
    fn core_is_disjoint_from_the_attractor_strategy_slot_and_host_model() {
        let core = include_str!("first_witness.rs");
        let production = core.split("#[cfg(test)]").next().unwrap_or(core);
        let attractor_strategy = ["Attractor", "EnumerationStrategyV0"].concat();
        assert!(!production.contains(&attractor_strategy));
        assert!(!production.contains("use crate::"));
        assert!(!production.contains("use super::"));
        let grn = include_str!("grn.rs");
        let module_name = ["first_", "witness"].concat();
        assert!(!grn.contains(&module_name));
    }

    #[test]
    fn first_witness_declared_synthetic_measurement_report() -> Result<(), FirstWitnessErrorV0> {
        const VARIABLE_COUNT: usize = 12;
        const EDIT_COUNT: usize = 2_000;
        let atoms = (0..VARIABLE_COUNT)
            .map(|index| format!("g{index}"))
            .collect::<Vec<_>>();
        let mut manager = FirstWitnessManagerV0::new(
            VariableOrderRegistrationV0::site_first_appearance(atoms.clone())?,
            FirstWitnessManagerConfigV0::default(),
        );
        let mut root = TRUE_NODE_ID_V0;
        let mut rebuild_count = 0usize;
        let mut rebuilt_nodes_before = 0usize;
        let mut rebuilt_nodes_after = 0usize;
        let mut rebuild_elapsed_nanos = 0u128;
        for edit in 0..EDIT_COUNT {
            let left = manager.variable(&atoms[edit % VARIABLE_COUNT])?;
            let right = manager.variable(&atoms[(edit * 5 + 1) % VARIABLE_COUNT])?;
            let not_right = manager.not(right)?;
            let candidate = manager.and(left, not_right)?;
            root = if edit % 2 == 0 {
                manager.or(root, candidate)?
            } else {
                manager.xor(root, candidate)?
            };
            let started = Instant::now();
            let mut roots = [root];
            if let Some(report) = manager.reclaim_if_due(&mut roots)? {
                rebuild_elapsed_nanos += started.elapsed().as_nanos();
                root = roots[0];
                rebuild_count += 1;
                rebuilt_nodes_before += report.nodes_before;
                rebuilt_nodes_after += report.nodes_after;
            }
        }
        assert!(rebuild_count > 0);
        assert!(manager.apply_cache_len() <= DEFAULT_APPLY_CACHE_CAPACITY_V0);
        assert!(manager.is_satisfiable(root));
        eprintln!(
            "{{\"declaredSynthetic\":true,\"variableCount\":{VARIABLE_COUNT},\"editCount\":{EDIT_COUNT},\"cacheCapacity\":{},\"cacheOccupancy\":{},\"rebuildIntervalOperations\":{},\"rebuildCount\":{rebuild_count},\"nodesBeforeRebuildTotal\":{rebuilt_nodes_before},\"nodesAfterRebuildTotal\":{rebuilt_nodes_after},\"rebuildElapsedNanos\":{rebuild_elapsed_nanos},\"finalNodeCount\":{},\"operationCounters\":{{\"choose\":{},\"apply\":{},\"cacheLookups\":{},\"cacheHits\":{}}}}}",
            DEFAULT_APPLY_CACHE_CAPACITY_V0,
            manager.apply_cache_len(),
            DEFAULT_REBUILD_INTERVAL_OPERATIONS_V0,
            manager.node_count(),
            manager.counters().choose_invocations,
            manager.counters().apply_invocations,
            manager.counters().apply_cache_lookups,
            manager.counters().apply_cache_hits,
        );
        Ok(())
    }
}
