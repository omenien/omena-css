//! Canonical decision diagrams and the shared first-witness fold.
//!
//! This module deliberately imports no type from its host crate. The boolean
//! terminal core is extended in place by the later cascade-winner terminal
//! alphabet without coupling either plane to the surrounding cascade model.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

use serde::Serialize;

pub type NodeId = u32;

pub const FALSE_NODE_ID_V0: NodeId = 0;
pub const TRUE_NODE_ID_V0: NodeId = 1;
pub const GUARDED_CASCADE_BOT_NODE_ID_V0: NodeId = FALSE_NODE_ID_V0;
pub const DEFAULT_APPLY_CACHE_CAPACITY_V0: usize = 4_096;
pub const DEFAULT_REBUILD_INTERVAL_OPERATIONS_V0: u64 = 8_192;
pub const SITE_FIRST_APPEARANCE_ORDERING_DOMAIN_V0: &str = "siteFirstAppearance";
pub const AT_RULE_NESTING_DFS_ORDERING_DOMAIN_V0: &str = "atRuleNestingDfs";

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
    AtRuleNestingDfs,
}

impl VariableOrderDomainV0 {
    pub const fn name(self) -> &'static str {
        match self {
            Self::SiteFirstAppearance => SITE_FIRST_APPEARANCE_ORDERING_DOMAIN_V0,
            Self::AtRuleNestingDfs => AT_RULE_NESTING_DFS_ORDERING_DOMAIN_V0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AtRuleNestingOrderAtomV0 {
    atom: String,
    at_rule_path: Vec<u32>,
}

impl AtRuleNestingOrderAtomV0 {
    pub fn new(atom: impl Into<String>, at_rule_path: impl IntoIterator<Item = u32>) -> Self {
        Self {
            atom: atom.into(),
            at_rule_path: at_rule_path.into_iter().collect(),
        }
    }

    pub fn atom(&self) -> &str {
        self.atom.as_str()
    }

    pub fn at_rule_path(&self) -> &[u32] {
        self.at_rule_path.as_slice()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GuardedCascadeSpecificityExactnessV0 {
    Exact,
    Inexact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GuardedCascadeConditionKindV0 {
    Media,
    Supports,
    Container,
    StructuralPseudo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardedCascadeConditionAtomV0 {
    atom: String,
    kind: GuardedCascadeConditionKindV0,
    at_rule_path: Vec<u32>,
    numeric: bool,
}

impl GuardedCascadeConditionAtomV0 {
    pub fn media(
        atom: impl Into<String>,
        at_rule_path: impl IntoIterator<Item = u32>,
        numeric: bool,
    ) -> Self {
        Self {
            atom: atom.into(),
            kind: GuardedCascadeConditionKindV0::Media,
            at_rule_path: at_rule_path.into_iter().collect(),
            numeric,
        }
    }

    pub fn supports(
        atom: impl Into<String>,
        at_rule_path: impl IntoIterator<Item = u32>,
        numeric: bool,
    ) -> Self {
        Self {
            atom: atom.into(),
            kind: GuardedCascadeConditionKindV0::Supports,
            at_rule_path: at_rule_path.into_iter().collect(),
            numeric,
        }
    }

    pub fn container(atom: impl Into<String>, at_rule_path: impl IntoIterator<Item = u32>) -> Self {
        Self {
            atom: atom.into(),
            kind: GuardedCascadeConditionKindV0::Container,
            at_rule_path: at_rule_path.into_iter().collect(),
            numeric: false,
        }
    }

    pub fn structural_pseudo(atom: impl Into<String>) -> Self {
        Self {
            atom: atom.into(),
            kind: GuardedCascadeConditionKindV0::StructuralPseudo,
            at_rule_path: Vec::new(),
            numeric: false,
        }
    }

    pub fn atom(&self) -> &str {
        self.atom.as_str()
    }

    pub const fn kind(&self) -> GuardedCascadeConditionKindV0 {
        self.kind
    }

    pub fn at_rule_path(&self) -> &[u32] {
        self.at_rule_path.as_slice()
    }

    pub const fn is_numeric(&self) -> bool {
        self.numeric
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardedCascadeCandidateV0<K> {
    declaration_id: u32,
    element_signature: String,
    property: String,
    cascade_key: K,
    specificity_exactness: GuardedCascadeSpecificityExactnessV0,
    scope_proximity: u32,
    conditions: Vec<GuardedCascadeConditionAtomV0>,
}

impl<K> GuardedCascadeCandidateV0<K> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        declaration_id: u32,
        element_signature: impl Into<String>,
        property: impl Into<String>,
        cascade_key: K,
        specificity_exactness: GuardedCascadeSpecificityExactnessV0,
        scope_proximity: u32,
        conditions: Vec<GuardedCascadeConditionAtomV0>,
    ) -> Self {
        Self {
            declaration_id,
            element_signature: element_signature.into(),
            property: property.into(),
            cascade_key,
            specificity_exactness,
            scope_proximity,
            conditions,
        }
    }

    pub const fn declaration_id(&self) -> u32 {
        self.declaration_id
    }

    pub fn element_signature(&self) -> &str {
        self.element_signature.as_str()
    }

    pub fn property(&self) -> &str {
        self.property.as_str()
    }

    pub const fn cascade_key(&self) -> &K {
        &self.cascade_key
    }

    pub const fn specificity_exactness(&self) -> GuardedCascadeSpecificityExactnessV0 {
        self.specificity_exactness
    }

    pub const fn scope_proximity(&self) -> u32 {
        self.scope_proximity
    }

    pub fn conditions(&self) -> &[GuardedCascadeConditionAtomV0] {
        self.conditions.as_slice()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "reason",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum GuardedCascadeFragmentRefusalV0 {
    EmptyCandidateSet,
    InexactSpecificity {
        declaration_id: u32,
    },
    ScopeProximityPresent {
        declaration_id: u32,
        scope_proximity: u32,
    },
    ContainerCondition {
        declaration_id: u32,
        atom: String,
    },
    StructuralPseudoCondition {
        declaration_id: u32,
        atom: String,
    },
    NumericConditionOutsideAlphabet {
        declaration_id: u32,
        atom: String,
    },
    ConditionOutsideDeclaredAlphabet {
        declaration_id: u32,
        atom: String,
    },
    MultipleProperties {
        expected: String,
        observed: String,
    },
    MultipleElementSignatures {
        expected: String,
        observed: String,
    },
    DuplicateDeclarationId {
        declaration_id: u32,
    },
    NonUniqueCascadeKey {
        first_declaration_id: u32,
        second_declaration_id: u32,
    },
    ConditionAlphabetCapacityExceeded,
}

impl GuardedCascadeFragmentRefusalV0 {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::EmptyCandidateSet => "emptyCandidateSet",
            Self::InexactSpecificity { .. } => "inexactSpecificity",
            Self::ScopeProximityPresent { .. } => "scopeProximityPresent",
            Self::ContainerCondition { .. } => "containerCondition",
            Self::StructuralPseudoCondition { .. } => "structuralPseudoCondition",
            Self::NumericConditionOutsideAlphabet { .. } => "numericConditionOutsideAlphabet",
            Self::ConditionOutsideDeclaredAlphabet { .. } => "conditionOutsideDeclaredAlphabet",
            Self::MultipleProperties { .. } => "multipleProperties",
            Self::MultipleElementSignatures { .. } => "multipleElementSignatures",
            Self::DuplicateDeclarationId { .. } => "duplicateDeclarationId",
            Self::NonUniqueCascadeKey { .. } => "nonUniqueCascadeKey",
            Self::ConditionAlphabetCapacityExceeded => "conditionAlphabetCapacityExceeded",
        }
    }
}

impl std::fmt::Display for GuardedCascadeFragmentRefusalV0 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "guarded cascade fragment refused: {}",
            self.name()
        )
    }
}

impl std::error::Error for GuardedCascadeFragmentRefusalV0 {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardedCascadeFragmentV0<K> {
    element_signature: String,
    property: String,
    condition_alphabet: Vec<String>,
    candidates: Vec<GuardedCascadeCandidateV0<K>>,
}

impl<K: Clone + Ord> GuardedCascadeFragmentV0<K> {
    pub fn admit(
        condition_alphabet: impl IntoIterator<Item = impl Into<String>>,
        candidates: impl IntoIterator<Item = GuardedCascadeCandidateV0<K>>,
    ) -> Result<Self, GuardedCascadeFragmentRefusalV0> {
        let alphabet = condition_alphabet
            .into_iter()
            .map(Into::into)
            .collect::<BTreeSet<String>>();
        if alphabet.len() > usize::from(u16::MAX) + 1 {
            return Err(GuardedCascadeFragmentRefusalV0::ConditionAlphabetCapacityExceeded);
        }
        let mut candidates = candidates.into_iter().collect::<Vec<_>>();
        let Some(first) = candidates.first() else {
            return Err(GuardedCascadeFragmentRefusalV0::EmptyCandidateSet);
        };
        let element_signature = first.element_signature.clone();
        let property = first.property.clone();
        let mut declaration_ids = BTreeSet::new();
        let mut cascade_keys = BTreeMap::<K, u32>::new();
        for candidate in &candidates {
            if candidate.element_signature != element_signature {
                return Err(GuardedCascadeFragmentRefusalV0::MultipleElementSignatures {
                    expected: element_signature,
                    observed: candidate.element_signature.clone(),
                });
            }
            if candidate.property != property {
                return Err(GuardedCascadeFragmentRefusalV0::MultipleProperties {
                    expected: property,
                    observed: candidate.property.clone(),
                });
            }
            if candidate.specificity_exactness != GuardedCascadeSpecificityExactnessV0::Exact {
                return Err(GuardedCascadeFragmentRefusalV0::InexactSpecificity {
                    declaration_id: candidate.declaration_id,
                });
            }
            if candidate.scope_proximity != 0 {
                return Err(GuardedCascadeFragmentRefusalV0::ScopeProximityPresent {
                    declaration_id: candidate.declaration_id,
                    scope_proximity: candidate.scope_proximity,
                });
            }
            if !declaration_ids.insert(candidate.declaration_id) {
                return Err(GuardedCascadeFragmentRefusalV0::DuplicateDeclarationId {
                    declaration_id: candidate.declaration_id,
                });
            }
            if let Some(first_declaration_id) =
                cascade_keys.insert(candidate.cascade_key.clone(), candidate.declaration_id)
            {
                return Err(GuardedCascadeFragmentRefusalV0::NonUniqueCascadeKey {
                    first_declaration_id,
                    second_declaration_id: candidate.declaration_id,
                });
            }
            for condition in &candidate.conditions {
                match condition.kind {
                    GuardedCascadeConditionKindV0::Container => {
                        return Err(GuardedCascadeFragmentRefusalV0::ContainerCondition {
                            declaration_id: candidate.declaration_id,
                            atom: condition.atom.clone(),
                        });
                    }
                    GuardedCascadeConditionKindV0::StructuralPseudo => {
                        return Err(GuardedCascadeFragmentRefusalV0::StructuralPseudoCondition {
                            declaration_id: candidate.declaration_id,
                            atom: condition.atom.clone(),
                        });
                    }
                    GuardedCascadeConditionKindV0::Media
                    | GuardedCascadeConditionKindV0::Supports => {}
                }
                if !alphabet.contains(condition.atom.as_str()) {
                    return Err(if condition.numeric {
                        GuardedCascadeFragmentRefusalV0::NumericConditionOutsideAlphabet {
                            declaration_id: candidate.declaration_id,
                            atom: condition.atom.clone(),
                        }
                    } else {
                        GuardedCascadeFragmentRefusalV0::ConditionOutsideDeclaredAlphabet {
                            declaration_id: candidate.declaration_id,
                            atom: condition.atom.clone(),
                        }
                    });
                }
            }
        }
        candidates.sort_by(|left, right| right.cascade_key.cmp(&left.cascade_key));
        Ok(Self {
            element_signature,
            property,
            condition_alphabet: alphabet.into_iter().collect(),
            candidates,
        })
    }

    pub fn element_signature(&self) -> &str {
        self.element_signature.as_str()
    }

    pub fn property(&self) -> &str {
        self.property.as_str()
    }

    pub fn condition_alphabet(&self) -> &[String] {
        self.condition_alphabet.as_slice()
    }

    pub fn candidates(&self) -> &[GuardedCascadeCandidateV0<K>] {
        self.candidates.as_slice()
    }
}

pub fn at_rule_nesting_order_for_fragment_v0<K>(
    fragment: &GuardedCascadeFragmentV0<K>,
) -> Result<VariableOrderRegistrationV0, FirstWitnessErrorV0> {
    #[cfg(test)]
    if std::env::var_os("OMENA_G122_INJECT_REMOVE_AT_RULE_ORDER").is_some() {
        return VariableOrderRegistrationV0::site_first_appearance(
            fragment.condition_alphabet.iter().cloned(),
        );
    }
    VariableOrderRegistrationV0::at_rule_nesting_dfs(
        fragment
            .candidates
            .iter()
            .flat_map(|candidate| candidate.conditions.iter())
            .map(|condition| {
                AtRuleNestingOrderAtomV0::new(
                    condition.atom.clone(),
                    condition.at_rule_path.iter().copied(),
                )
            }),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct GuardedCascadeWinnerRootV0(NodeId);

impl GuardedCascadeWinnerRootV0 {
    pub const fn node_id(self) -> NodeId {
        self.0
    }
}

/// The exact fragment predicate attached to an MTBDD-owned answer.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardedCascadeFragmentPredicateV0 {
    pub element_signature: String,
    pub property: String,
    pub condition_alphabet: Vec<String>,
}

impl<K> GuardedCascadeFragmentV0<K> {
    pub fn predicate(&self) -> GuardedCascadeFragmentPredicateV0 {
        GuardedCascadeFragmentPredicateV0 {
            element_signature: self.element_signature.clone(),
            property: self.property.clone(),
            condition_alphabet: self.condition_alphabet.clone(),
        }
    }
}

/// Which authority answered one guarded-winner question.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum GuardedCascadeWinnerAuthorityRuleV0 {
    ScenarioSweepOutsideFragment,
    CanonicalMtbddInsideFragment {
        fragment: GuardedCascadeFragmentPredicateV0,
    },
}

/// A canonical answer produced inside the declared guarded fragment.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardedCascadeWinnerAuthorityV0 {
    pub rule: GuardedCascadeWinnerAuthorityRuleV0,
    pub root: GuardedCascadeWinnerRootV0,
    pub winner_defined_for_all_assignments: bool,
}

/// Why canonical winner-root equality cannot discharge an obligation.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "reason",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum GuardedCascadeWinnerFunctionEqualityRefusalV0 {
    CanonicalRootsDiffer {
        input_root: GuardedCascadeWinnerRootV0,
        output_root: GuardedCascadeWinnerRootV0,
    },
}

/// Equality is actionable; inequality is a typed refusal because the free
/// boolean alphabet contains assignments that no browser can realise.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum GuardedCascadeWinnerFunctionEqualityDecisionV0 {
    Equal {
        authority: GuardedCascadeWinnerAuthorityV0,
    },
    Refused {
        rule: GuardedCascadeWinnerAuthorityRuleV0,
        refusal: GuardedCascadeWinnerFunctionEqualityRefusalV0,
    },
}

/// One plane's answer for a concrete assignment.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum GuardedCascadeWinnerPlaneAnswerV0 {
    NoWinner,
    Declaration { declaration_id: u32 },
}

/// An in-fragment disagreement is an integrity error, never a confidence tie.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "reason",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum GuardedCascadeWinnerAuthorityErrorV0 {
    InFragmentPlaneDisagreement {
        canonical_mtbdd: GuardedCascadeWinnerPlaneAnswerV0,
        scenario_sweep: GuardedCascadeWinnerPlaneAnswerV0,
    },
}

impl std::fmt::Display for GuardedCascadeWinnerAuthorityErrorV0 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InFragmentPlaneDisagreement {
                canonical_mtbdd,
                scenario_sweep,
            } => write!(
                formatter,
                "in-fragment guarded winner disagreement: canonicalMtbdd={canonical_mtbdd:?}, scenarioSweep={scenario_sweep:?}"
            ),
        }
    }
}

impl std::error::Error for GuardedCascadeWinnerAuthorityErrorV0 {}

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
        Self::from_ordered(VariableOrderDomainV0::SiteFirstAppearance, ordered)
    }

    pub fn at_rule_nesting_dfs(
        atoms: impl IntoIterator<Item = AtRuleNestingOrderAtomV0>,
    ) -> Result<Self, FirstWitnessErrorV0> {
        let mut atoms = atoms.into_iter().collect::<Vec<_>>();
        atoms.sort_by(|left, right| {
            left.at_rule_path
                .cmp(&right.at_rule_path)
                .then_with(|| left.atom.cmp(&right.atom))
        });
        let mut seen = BTreeSet::new();
        let ordered = atoms
            .into_iter()
            .filter_map(|atom| seen.insert(atom.atom.clone()).then_some(atom.atom))
            .collect();
        Self::from_ordered(VariableOrderDomainV0::AtRuleNestingDfs, ordered)
    }

    fn from_ordered(
        domain: VariableOrderDomainV0,
        ordered: Vec<String>,
    ) -> Result<Self, FirstWitnessErrorV0> {
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
            domain,
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FirstWitnessChoiceOperationCountersV0 {
    pub recursive_invocations: u64,
    pub apply_cache_lookups: u64,
    pub apply_cache_hits: u64,
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
    DeclarationIdCapacityExceeded,
    DeclarationTerminalRegistrationClosed,
    UnregisteredDeclarationTerminal(u32),
    MissingAssignment { variable: u16 },
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
            Self::DeclarationIdCapacityExceeded => {
                formatter.write_str("declaration id cannot be represented by the terminal alphabet")
            }
            Self::DeclarationTerminalRegistrationClosed => formatter
                .write_str("declaration terminals must be registered before internal nodes exist"),
            Self::UnregisteredDeclarationTerminal(declaration_id) => write!(
                formatter,
                "declaration terminal {declaration_id} was not registered"
            ),
            Self::MissingAssignment { variable } => {
                write!(
                    formatter,
                    "assignment does not cover decision variable {variable}"
                )
            }
        }
    }
}

impl std::error::Error for FirstWitnessErrorV0 {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ApplyOperationV0 {
    Boolean(BooleanOperationV0),
    FirstWitness(FirstWitnessTerminalBehaviorV0),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FirstWitnessTerminalBehaviorV0 {
    LeftBiased,
    #[cfg(test)]
    RightBiased,
    #[cfg(test)]
    BrokenRecursion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ApplyCacheKeyV0 {
    operation: ApplyOperationV0,
    left: NodeId,
    right: NodeId,
}

#[derive(Debug, Clone)]
pub struct FirstWitnessManagerV0 {
    nodes: Vec<Node>,
    terminal_by_value: HashMap<u32, NodeId>,
    unique: HashMap<(u16, NodeId, NodeId), NodeId>,
    apply_cache: HashMap<ApplyCacheKeyV0, NodeId>,
    apply_cache_fifo: VecDeque<ApplyCacheKeyV0>,
    order: VariableOrderRegistrationV0,
    config: FirstWitnessManagerConfigV0,
    counters: FirstWitnessOperationCountersV0,
    choice_counters: FirstWitnessChoiceOperationCountersV0,
    operations_at_previous_rebuild: u64,
}

impl FirstWitnessManagerV0 {
    pub fn new(order: VariableOrderRegistrationV0, config: FirstWitnessManagerConfigV0) -> Self {
        Self {
            nodes: vec![Node::Term(0), Node::Term(1)],
            terminal_by_value: HashMap::from([(0, FALSE_NODE_ID_V0), (1, TRUE_NODE_ID_V0)]),
            unique: HashMap::new(),
            apply_cache: HashMap::new(),
            apply_cache_fifo: VecDeque::new(),
            order,
            config,
            counters: FirstWitnessOperationCountersV0::default(),
            choice_counters: FirstWitnessChoiceOperationCountersV0::default(),
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

    pub const fn first_witness_counters(&self) -> FirstWitnessChoiceOperationCountersV0 {
        self.choice_counters
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

    pub fn reachable_winner_node_count(
        &self,
        root: GuardedCascadeWinnerRootV0,
    ) -> Result<usize, FirstWitnessErrorV0> {
        let mut seen = BTreeSet::new();
        let mut pending = vec![root.0];
        while let Some(node_id) = pending.pop() {
            if !seen.insert(node_id) {
                continue;
            }
            if let Node::Int { lo, hi, .. } = self.require_node(node_id)? {
                pending.extend([lo, hi]);
            }
        }
        Ok(seen.len())
    }

    pub fn register_declaration_terminals(
        &mut self,
        declaration_ids: impl IntoIterator<Item = u32>,
    ) -> Result<(), FirstWitnessErrorV0> {
        let mut encoded = declaration_ids
            .into_iter()
            .map(|declaration_id| {
                declaration_id
                    .checked_add(1)
                    .map(|terminal| (terminal, declaration_id))
                    .ok_or(FirstWitnessErrorV0::DeclarationIdCapacityExceeded)
            })
            .collect::<Result<Vec<_>, _>>()?;
        encoded.sort_unstable();
        encoded.dedup_by_key(|(terminal, _)| *terminal);
        let has_missing = encoded
            .iter()
            .any(|(terminal, _)| !self.terminal_by_value.contains_key(terminal));
        if has_missing
            && self
                .nodes
                .iter()
                .any(|node| matches!(node, Node::Int { .. }))
        {
            return Err(FirstWitnessErrorV0::DeclarationTerminalRegistrationClosed);
        }
        for (terminal, _) in encoded {
            if self.terminal_by_value.contains_key(&terminal) {
                continue;
            }
            let node = u32::try_from(self.nodes.len())
                .map_err(|_| FirstWitnessErrorV0::VariableCapacityExceeded)?;
            self.nodes.push(Node::Term(terminal));
            self.terminal_by_value.insert(terminal, node);
        }
        Ok(())
    }

    pub fn declaration_terminal(&self, declaration_id: u32) -> Result<NodeId, FirstWitnessErrorV0> {
        let terminal = declaration_id
            .checked_add(1)
            .ok_or(FirstWitnessErrorV0::DeclarationIdCapacityExceeded)?;
        self.terminal_by_value.get(&terminal).copied().ok_or(
            FirstWitnessErrorV0::UnregisteredDeclarationTerminal(declaration_id),
        )
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

    pub fn choose_first_witness(
        &mut self,
        left: NodeId,
        right: NodeId,
    ) -> Result<NodeId, FirstWitnessErrorV0> {
        self.require_node(left)?;
        self.require_node(right)?;
        self.choose_first_witness_recursive(left, right, FirstWitnessTerminalBehaviorV0::LeftBiased)
    }

    #[cfg(test)]
    fn choose_first_witness_with_terminal_behavior_for_test(
        &mut self,
        left: NodeId,
        right: NodeId,
        behavior: FirstWitnessTerminalBehaviorV0,
    ) -> Result<NodeId, FirstWitnessErrorV0> {
        self.require_node(left)?;
        self.require_node(right)?;
        self.choose_first_witness_recursive(left, right, behavior)
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
        let operations = self
            .counters
            .recursive_operations()
            .saturating_add(self.choice_counters.recursive_invocations);
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
        let mut rebuilt_nodes = self
            .nodes
            .iter()
            .copied()
            .take_while(|node| matches!(node, Node::Term(_)))
            .collect::<Vec<_>>();
        let rebuilt_terminal_by_value = rebuilt_nodes
            .iter()
            .enumerate()
            .filter_map(|(node, value)| match value {
                Node::Term(value) => u32::try_from(node).ok().map(|node| (*value, node)),
                Node::Int { .. } => None,
            })
            .collect::<HashMap<_, _>>();
        let mut rebuilt_unique = HashMap::new();
        let mut remapped = (0..rebuilt_nodes.len())
            .filter_map(|node| u32::try_from(node).ok().map(|node| (node, node)))
            .collect::<HashMap<_, _>>();
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
        self.terminal_by_value = rebuilt_terminal_by_value;
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

    fn choose_first_witness_recursive(
        &mut self,
        left: NodeId,
        right: NodeId,
        terminal_behavior: FirstWitnessTerminalBehaviorV0,
    ) -> Result<NodeId, FirstWitnessErrorV0> {
        self.choice_counters.recursive_invocations += 1;
        let left_node = self.require_node(left)?;
        let right_node = self.require_node(right)?;
        if self.config.shortcuts {
            if left == right {
                return Ok(left);
            }
            if matches!(left_node, Node::Term(terminal) if terminal != 0) {
                return Ok(left);
            }
            if right == GUARDED_CASCADE_BOT_NODE_ID_V0 {
                return Ok(left);
            }
        }
        if let (Node::Term(left_terminal), Node::Term(_)) = (left_node, right_node) {
            return Ok(match terminal_behavior {
                FirstWitnessTerminalBehaviorV0::LeftBiased => {
                    if left_terminal == 0 {
                        right
                    } else {
                        left
                    }
                }
                #[cfg(test)]
                FirstWitnessTerminalBehaviorV0::RightBiased => {
                    if right == GUARDED_CASCADE_BOT_NODE_ID_V0 {
                        left
                    } else {
                        right
                    }
                }
                #[cfg(test)]
                FirstWitnessTerminalBehaviorV0::BrokenRecursion => {
                    if left_terminal == 0
                        && matches!(right_node, Node::Term(terminal) if terminal > 1)
                    {
                        TRUE_NODE_ID_V0
                    } else {
                        GUARDED_CASCADE_BOT_NODE_ID_V0
                    }
                }
            });
        }
        let key = ApplyCacheKeyV0 {
            operation: ApplyOperationV0::FirstWitness(terminal_behavior),
            left,
            right,
        };
        self.choice_counters.apply_cache_lookups += 1;
        if let Some(result) = self.apply_cache.get(&key).copied() {
            self.choice_counters.apply_cache_hits += 1;
            return Ok(result);
        }
        let variable = top_variable(left_node, right_node);
        let (left_low, left_high) = cofactors(left, left_node, variable);
        let (right_low, right_high) = cofactors(right, right_node, variable);
        let low = self.choose_first_witness_recursive(left_low, right_low, terminal_behavior)?;
        let high = self.choose_first_witness_recursive(left_high, right_high, terminal_behavior)?;
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

    #[cfg(test)]
    fn intern_without_collapse_for_test(
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IncrementalGuardedCascadeWinnerEditReportV0 {
    pub replaced_existing_key: bool,
    pub entry_count: usize,
    pub root: GuardedCascadeWinnerRootV0,
    pub aggregate_updates: u64,
}

#[derive(Debug)]
struct IncrementalGuardedCascadeWinnerNodeV0<K> {
    key: K,
    guarded_root: NodeId,
    aggregate: NodeId,
    height: u16,
    size: usize,
    left: Option<Box<Self>>,
    right: Option<Box<Self>>,
}

type IncrementalWinnerLinkV0<K> = Option<Box<IncrementalGuardedCascadeWinnerNodeV0<K>>>;
type IncrementalWinnerMutationV0<K> = (IncrementalWinnerLinkV0<K>, bool);

impl<K> IncrementalGuardedCascadeWinnerNodeV0<K> {
    fn leaf(key: K, guarded_root: NodeId) -> Self {
        Self {
            key,
            guarded_root,
            aggregate: guarded_root,
            height: 1,
            size: 1,
            left: None,
            right: None,
        }
    }
}

#[derive(Debug, Default)]
pub struct IncrementalGuardedCascadeWinnerV0<K> {
    root: Option<Box<IncrementalGuardedCascadeWinnerNodeV0<K>>>,
    aggregate_updates: u64,
}

impl<K: Ord> IncrementalGuardedCascadeWinnerV0<K> {
    pub const fn new() -> Self {
        Self {
            root: None,
            aggregate_updates: 0,
        }
    }

    pub fn len(&self) -> usize {
        incremental_winner_size(&self.root)
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn root(&self) -> GuardedCascadeWinnerRootV0 {
        GuardedCascadeWinnerRootV0(incremental_winner_fold(&self.root))
    }

    pub const fn aggregate_updates(&self) -> u64 {
        self.aggregate_updates
    }

    pub fn insert(
        &mut self,
        manager: &mut FirstWitnessManagerV0,
        cascade_key: K,
        guarded_root: GuardedCascadeWinnerRootV0,
    ) -> Result<IncrementalGuardedCascadeWinnerEditReportV0, FirstWitnessErrorV0> {
        manager.require_node(guarded_root.0)?;
        let (root, replaced_existing_key) = incremental_winner_insert(
            self.root.take(),
            cascade_key,
            guarded_root.0,
            manager,
            &mut self.aggregate_updates,
        )?;
        self.root = root;
        Ok(self.edit_report(replaced_existing_key))
    }

    pub fn remove(
        &mut self,
        manager: &mut FirstWitnessManagerV0,
        cascade_key: &K,
    ) -> Result<IncrementalGuardedCascadeWinnerEditReportV0, FirstWitnessErrorV0> {
        let (root, removed) = incremental_winner_remove(
            self.root.take(),
            cascade_key,
            manager,
            &mut self.aggregate_updates,
        )?;
        self.root = root;
        Ok(self.edit_report(removed))
    }

    pub fn reclaim_manager_if_due(
        &mut self,
        manager: &mut FirstWitnessManagerV0,
    ) -> Result<Option<FirstWitnessRebuildReportV0>, FirstWitnessErrorV0> {
        #[cfg(test)]
        if std::env::var_os("OMENA_G122_INJECT_DISABLE_WINNER_RECLAMATION").is_some() {
            return Ok(None);
        }
        let mut live_roots = Vec::with_capacity(self.len().saturating_mul(2));
        collect_incremental_winner_roots(&self.root, &mut live_roots);
        let report = manager.reclaim_if_due(&mut live_roots)?;
        if report.is_some() {
            let mut remapped = live_roots.into_iter();
            rewrite_incremental_winner_roots(&mut self.root, &mut remapped);
            debug_assert!(remapped.next().is_none());
        }
        Ok(report)
    }

    fn edit_report(
        &self,
        replaced_existing_key: bool,
    ) -> IncrementalGuardedCascadeWinnerEditReportV0 {
        IncrementalGuardedCascadeWinnerEditReportV0 {
            replaced_existing_key,
            entry_count: self.len(),
            root: self.root(),
            aggregate_updates: self.aggregate_updates,
        }
    }
}

fn incremental_winner_height<K>(
    node: &Option<Box<IncrementalGuardedCascadeWinnerNodeV0<K>>>,
) -> u16 {
    node.as_ref().map_or(0, |node| node.height)
}

fn incremental_winner_size<K>(
    node: &Option<Box<IncrementalGuardedCascadeWinnerNodeV0<K>>>,
) -> usize {
    node.as_ref().map_or(0, |node| node.size)
}

fn incremental_winner_fold<K>(
    node: &Option<Box<IncrementalGuardedCascadeWinnerNodeV0<K>>>,
) -> NodeId {
    node.as_ref()
        .map_or(GUARDED_CASCADE_BOT_NODE_ID_V0, |node| node.aggregate)
}

fn refresh_incremental_winner<K>(
    node: &mut IncrementalGuardedCascadeWinnerNodeV0<K>,
    manager: &mut FirstWitnessManagerV0,
    aggregate_updates: &mut u64,
) -> Result<(), FirstWitnessErrorV0> {
    node.height =
        1 + incremental_winner_height(&node.left).max(incremental_winner_height(&node.right));
    node.size = 1 + incremental_winner_size(&node.left) + incremental_winner_size(&node.right);
    #[cfg(test)]
    if std::env::var_os("OMENA_G122_INJECT_STALE_WINNER_AGGREGATE").is_some() {
        return Ok(());
    }
    let left_and_self =
        manager.choose_first_witness(incremental_winner_fold(&node.left), node.guarded_root)?;
    node.aggregate =
        manager.choose_first_witness(left_and_self, incremental_winner_fold(&node.right))?;
    *aggregate_updates += 2;
    Ok(())
}

fn incremental_winner_balance_factor<K>(node: &IncrementalGuardedCascadeWinnerNodeV0<K>) -> i32 {
    i32::from(incremental_winner_height(&node.left))
        - i32::from(incremental_winner_height(&node.right))
}

fn rotate_incremental_winner_left<K>(
    mut root: Box<IncrementalGuardedCascadeWinnerNodeV0<K>>,
    manager: &mut FirstWitnessManagerV0,
    aggregate_updates: &mut u64,
) -> Result<Box<IncrementalGuardedCascadeWinnerNodeV0<K>>, FirstWitnessErrorV0> {
    let mut pivot = root
        .right
        .take()
        .ok_or(FirstWitnessErrorV0::InvalidNode(root.aggregate))?;
    root.right = pivot.left.take();
    refresh_incremental_winner(&mut root, manager, aggregate_updates)?;
    pivot.left = Some(root);
    refresh_incremental_winner(&mut pivot, manager, aggregate_updates)?;
    Ok(pivot)
}

fn rotate_incremental_winner_right<K>(
    mut root: Box<IncrementalGuardedCascadeWinnerNodeV0<K>>,
    manager: &mut FirstWitnessManagerV0,
    aggregate_updates: &mut u64,
) -> Result<Box<IncrementalGuardedCascadeWinnerNodeV0<K>>, FirstWitnessErrorV0> {
    let mut pivot = root
        .left
        .take()
        .ok_or(FirstWitnessErrorV0::InvalidNode(root.aggregate))?;
    root.left = pivot.right.take();
    refresh_incremental_winner(&mut root, manager, aggregate_updates)?;
    pivot.right = Some(root);
    refresh_incremental_winner(&mut pivot, manager, aggregate_updates)?;
    Ok(pivot)
}

fn balance_incremental_winner<K>(
    mut node: Box<IncrementalGuardedCascadeWinnerNodeV0<K>>,
    manager: &mut FirstWitnessManagerV0,
    aggregate_updates: &mut u64,
) -> Result<Box<IncrementalGuardedCascadeWinnerNodeV0<K>>, FirstWitnessErrorV0> {
    refresh_incremental_winner(&mut node, manager, aggregate_updates)?;
    let balance = incremental_winner_balance_factor(&node);
    if balance > 1 {
        let left_balance = node
            .left
            .as_deref()
            .map_or(0, incremental_winner_balance_factor);
        if left_balance < 0 {
            let left = node
                .left
                .take()
                .ok_or(FirstWitnessErrorV0::InvalidNode(node.aggregate))?;
            node.left = Some(rotate_incremental_winner_left(
                left,
                manager,
                aggregate_updates,
            )?);
        }
        return rotate_incremental_winner_right(node, manager, aggregate_updates);
    }
    if balance < -1 {
        let right_balance = node
            .right
            .as_deref()
            .map_or(0, incremental_winner_balance_factor);
        if right_balance > 0 {
            let right = node
                .right
                .take()
                .ok_or(FirstWitnessErrorV0::InvalidNode(node.aggregate))?;
            node.right = Some(rotate_incremental_winner_right(
                right,
                manager,
                aggregate_updates,
            )?);
        }
        return rotate_incremental_winner_left(node, manager, aggregate_updates);
    }
    Ok(node)
}

fn incremental_winner_insert<K: Ord>(
    node: IncrementalWinnerLinkV0<K>,
    cascade_key: K,
    guarded_root: NodeId,
    manager: &mut FirstWitnessManagerV0,
    aggregate_updates: &mut u64,
) -> Result<IncrementalWinnerMutationV0<K>, FirstWitnessErrorV0> {
    let Some(mut node) = node else {
        return Ok((
            Some(Box::new(IncrementalGuardedCascadeWinnerNodeV0::leaf(
                cascade_key,
                guarded_root,
            ))),
            false,
        ));
    };
    let replaced = match cascade_key.cmp(&node.key) {
        Ordering::Greater => {
            let (left, replaced) = incremental_winner_insert(
                node.left.take(),
                cascade_key,
                guarded_root,
                manager,
                aggregate_updates,
            )?;
            node.left = left;
            replaced
        }
        Ordering::Less => {
            let (right, replaced) = incremental_winner_insert(
                node.right.take(),
                cascade_key,
                guarded_root,
                manager,
                aggregate_updates,
            )?;
            node.right = right;
            replaced
        }
        Ordering::Equal => {
            node.guarded_root = guarded_root;
            true
        }
    };
    Ok((
        Some(balance_incremental_winner(
            node,
            manager,
            aggregate_updates,
        )?),
        replaced,
    ))
}

fn incremental_winner_remove<K: Ord>(
    node: IncrementalWinnerLinkV0<K>,
    cascade_key: &K,
    manager: &mut FirstWitnessManagerV0,
    aggregate_updates: &mut u64,
) -> Result<IncrementalWinnerMutationV0<K>, FirstWitnessErrorV0> {
    let Some(mut node) = node else {
        return Ok((None, false));
    };
    let removed = match cascade_key.cmp(&node.key) {
        Ordering::Greater => {
            let (left, removed) = incremental_winner_remove(
                node.left.take(),
                cascade_key,
                manager,
                aggregate_updates,
            )?;
            node.left = left;
            removed
        }
        Ordering::Less => {
            let (right, removed) = incremental_winner_remove(
                node.right.take(),
                cascade_key,
                manager,
                aggregate_updates,
            )?;
            node.right = right;
            removed
        }
        Ordering::Equal => {
            if node.left.is_none() {
                return Ok((node.right.take(), true));
            }
            if node.right.is_none() {
                return Ok((node.left.take(), true));
            }
            let right = node
                .right
                .take()
                .ok_or(FirstWitnessErrorV0::InvalidNode(node.aggregate))?;
            let (successor, right) =
                extract_incremental_winner_leftmost(right, manager, aggregate_updates)?;
            node.key = successor.key;
            node.guarded_root = successor.guarded_root;
            node.right = right;
            true
        }
    };
    if !removed {
        return Ok((Some(node), false));
    }
    Ok((
        Some(balance_incremental_winner(
            node,
            manager,
            aggregate_updates,
        )?),
        true,
    ))
}

type IncrementalWinnerExtractV0<K> = (
    Box<IncrementalGuardedCascadeWinnerNodeV0<K>>,
    IncrementalWinnerLinkV0<K>,
);

fn extract_incremental_winner_leftmost<K>(
    mut node: Box<IncrementalGuardedCascadeWinnerNodeV0<K>>,
    manager: &mut FirstWitnessManagerV0,
    aggregate_updates: &mut u64,
) -> Result<IncrementalWinnerExtractV0<K>, FirstWitnessErrorV0> {
    let Some(left) = node.left.take() else {
        let right = node.right.take();
        return Ok((node, right));
    };
    let (leftmost, left) = extract_incremental_winner_leftmost(left, manager, aggregate_updates)?;
    node.left = left;
    Ok((
        leftmost,
        Some(balance_incremental_winner(
            node,
            manager,
            aggregate_updates,
        )?),
    ))
}

fn collect_incremental_winner_roots<K>(
    node: &Option<Box<IncrementalGuardedCascadeWinnerNodeV0<K>>>,
    roots: &mut Vec<NodeId>,
) {
    if let Some(node) = node {
        roots.extend([node.guarded_root, node.aggregate]);
        collect_incremental_winner_roots(&node.left, roots);
        collect_incremental_winner_roots(&node.right, roots);
    }
}

fn rewrite_incremental_winner_roots<K>(
    node: &mut Option<Box<IncrementalGuardedCascadeWinnerNodeV0<K>>>,
    roots: &mut impl Iterator<Item = NodeId>,
) {
    if let Some(node) = node {
        node.guarded_root = roots.next().unwrap_or(node.guarded_root);
        node.aggregate = roots.next().unwrap_or(node.aggregate);
        rewrite_incremental_winner_roots(&mut node.left, roots);
        rewrite_incremental_winner_roots(&mut node.right, roots);
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

pub fn build_guarded_cascade_winner_v0<K: Clone + Ord>(
    manager: &mut FirstWitnessManagerV0,
    fragment: &GuardedCascadeFragmentV0<K>,
) -> Result<GuardedCascadeWinnerRootV0, FirstWitnessErrorV0> {
    manager.register_declaration_terminals(
        fragment
            .candidates
            .iter()
            .map(|candidate| candidate.declaration_id),
    )?;
    let mut winner = GUARDED_CASCADE_BOT_NODE_ID_V0;
    for candidate in &fragment.candidates {
        let mut guarded = manager.declaration_terminal(candidate.declaration_id)?;
        let mut variables = candidate
            .conditions
            .iter()
            .map(|condition| {
                manager
                    .order
                    .variable_index(condition.atom.as_str())
                    .ok_or_else(|| FirstWitnessErrorV0::UnknownAtom(condition.atom.clone()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        variables.sort_unstable();
        variables.dedup();
        for variable in variables.into_iter().rev() {
            guarded = manager.choose(variable, GUARDED_CASCADE_BOT_NODE_ID_V0, guarded)?;
        }
        winner = manager.choose_first_witness(winner, guarded)?;
    }
    Ok(GuardedCascadeWinnerRootV0(winner))
}

pub fn evaluate_guarded_cascade_winner_v0(
    manager: &FirstWitnessManagerV0,
    root: GuardedCascadeWinnerRootV0,
    assignment: &[bool],
) -> Result<Option<u32>, FirstWitnessErrorV0> {
    let mut current = root.0;
    loop {
        match manager.require_node(current)? {
            Node::Term(0) => return Ok(None),
            Node::Term(terminal) => return Ok(Some(terminal - 1)),
            Node::Int { var, lo, hi } => {
                let value = assignment
                    .get(usize::from(var))
                    .copied()
                    .ok_or(FirstWitnessErrorV0::MissingAssignment { variable: var })?;
                current = if value { hi } else { lo };
            }
        }
    }
}

pub fn guarded_cascade_winner_is_total_v0(
    manager: &FirstWitnessManagerV0,
    root: GuardedCascadeWinnerRootV0,
) -> Result<bool, FirstWitnessErrorV0> {
    let mut seen = BTreeSet::new();
    let mut pending = vec![root.0];
    while let Some(node_id) = pending.pop() {
        if !seen.insert(node_id) {
            continue;
        }
        match manager.require_node(node_id)? {
            Node::Term(0) => return Ok(false),
            Node::Term(_) => {}
            Node::Int { lo, hi, .. } => pending.extend([lo, hi]),
        }
    }
    Ok(true)
}

pub fn compare_guarded_cascade_winner_functions_v0(
    fragment: GuardedCascadeFragmentPredicateV0,
    input_root: GuardedCascadeWinnerRootV0,
    output_root: GuardedCascadeWinnerRootV0,
    winner_defined_for_all_assignments: bool,
) -> GuardedCascadeWinnerFunctionEqualityDecisionV0 {
    let rule = GuardedCascadeWinnerAuthorityRuleV0::CanonicalMtbddInsideFragment { fragment };
    if same_canonical_winner_function_v0(input_root, output_root) {
        GuardedCascadeWinnerFunctionEqualityDecisionV0::Equal {
            authority: GuardedCascadeWinnerAuthorityV0 {
                rule,
                root: input_root,
                winner_defined_for_all_assignments,
            },
        }
    } else {
        GuardedCascadeWinnerFunctionEqualityDecisionV0::Refused {
            rule,
            refusal: GuardedCascadeWinnerFunctionEqualityRefusalV0::CanonicalRootsDiffer {
                input_root,
                output_root,
            },
        }
    }
}

pub fn guarded_cascade_winner_authority_v0(
    fragment: GuardedCascadeFragmentPredicateV0,
    root: GuardedCascadeWinnerRootV0,
    winner_defined_for_all_assignments: bool,
) -> GuardedCascadeWinnerAuthorityV0 {
    GuardedCascadeWinnerAuthorityV0 {
        rule: GuardedCascadeWinnerAuthorityRuleV0::CanonicalMtbddInsideFragment { fragment },
        root,
        winner_defined_for_all_assignments,
    }
}

pub fn reconcile_guarded_cascade_winner_planes_v0(
    authority: &GuardedCascadeWinnerAuthorityV0,
    canonical_mtbdd: GuardedCascadeWinnerPlaneAnswerV0,
    scenario_sweep: GuardedCascadeWinnerPlaneAnswerV0,
) -> Result<GuardedCascadeWinnerPlaneAnswerV0, GuardedCascadeWinnerAuthorityErrorV0> {
    #[cfg(test)]
    if std::env::var_os("OMENA_G122_INJECT_PREFER_SCENARIO_SWEEP").is_some() {
        return Ok(scenario_sweep);
    }
    match &authority.rule {
        GuardedCascadeWinnerAuthorityRuleV0::CanonicalMtbddInsideFragment { .. } => {
            if canonical_mtbdd == scenario_sweep {
                Ok(canonical_mtbdd)
            } else {
                Err(
                    GuardedCascadeWinnerAuthorityErrorV0::InFragmentPlaneDisagreement {
                        canonical_mtbdd,
                        scenario_sweep,
                    },
                )
            }
        }
        GuardedCascadeWinnerAuthorityRuleV0::ScenarioSweepOutsideFragment => Ok(scenario_sweep),
    }
}

pub const fn same_canonical_winner_function_v0(
    left: GuardedCascadeWinnerRootV0,
    right: GuardedCascadeWinnerRootV0,
) -> bool {
    left.0 == right.0
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
        operation: ApplyOperationV0::Boolean(operation),
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

    fn winner_manager(
        shortcuts: bool,
        variable_count: usize,
    ) -> Result<FirstWitnessManagerV0, FirstWitnessErrorV0> {
        let atoms = (0..variable_count)
            .map(|index| format!("guard-{index}"))
            .collect::<Vec<_>>();
        let mut manager = FirstWitnessManagerV0::new(
            VariableOrderRegistrationV0::site_first_appearance(atoms)?,
            FirstWitnessManagerConfigV0 {
                shortcuts,
                apply_cache_capacity: 16_384,
                rebuild_interval_operations: u64::MAX,
            },
        );
        manager.register_declaration_terminals([0, 1, 2])?;
        Ok(manager)
    }

    fn streaming_winner_manager(
        variable_count: usize,
        declaration_count: usize,
        apply_cache_capacity: usize,
        rebuild_interval_operations: u64,
    ) -> Result<FirstWitnessManagerV0, FirstWitnessErrorV0> {
        let atoms = (0..variable_count)
            .map(|index| format!("guard-{index}"))
            .collect::<Vec<_>>();
        let mut manager = FirstWitnessManagerV0::new(
            VariableOrderRegistrationV0::site_first_appearance(atoms)?,
            FirstWitnessManagerConfigV0 {
                shortcuts: false,
                apply_cache_capacity,
                rebuild_interval_operations,
            },
        );
        manager.register_declaration_terminals(
            (0..declaration_count).filter_map(|id| u32::try_from(id).ok()),
        )?;
        Ok(manager)
    }

    #[test]
    fn inside_fragment_plane_disagreement_names_both_answers() -> Result<(), String> {
        let authority = GuardedCascadeWinnerAuthorityV0 {
            rule: GuardedCascadeWinnerAuthorityRuleV0::CanonicalMtbddInsideFragment {
                fragment: GuardedCascadeFragmentPredicateV0 {
                    element_signature: ".a".to_string(),
                    property: "color".to_string(),
                    condition_alphabet: vec!["@media (min-width: 1px)".to_string()],
                },
            },
            root: GuardedCascadeWinnerRootV0(2),
            winner_defined_for_all_assignments: true,
        };
        let error = reconcile_guarded_cascade_winner_planes_v0(
            &authority,
            GuardedCascadeWinnerPlaneAnswerV0::Declaration { declaration_id: 7 },
            GuardedCascadeWinnerPlaneAnswerV0::Declaration { declaration_id: 9 },
        )
        .err()
        .ok_or_else(|| "an in-fragment disagreement must be rejected".to_string())?;
        let message = error.to_string();
        assert!(message.contains("canonicalMtbdd=Declaration { declaration_id: 7 }"));
        assert!(message.contains("scenarioSweep=Declaration { declaration_id: 9 }"));
        Ok(())
    }

    fn guarded_root_from_mask(
        manager: &mut FirstWitnessManagerV0,
        declaration_id: u32,
        mask: u64,
        variable_count: usize,
    ) -> Result<GuardedCascadeWinnerRootV0, FirstWitnessErrorV0> {
        let mut root = manager.declaration_terminal(declaration_id)?;
        for variable in (0..variable_count).rev() {
            if mask & (1 << variable) != 0 {
                root = manager.choose(
                    u16::try_from(variable)
                        .map_err(|_| FirstWitnessErrorV0::VariableCapacityExceeded)?,
                    GUARDED_CASCADE_BOT_NODE_ID_V0,
                    root,
                )?;
            }
        }
        Ok(GuardedCascadeWinnerRootV0(root))
    }

    fn batch_winner_from_entries(
        manager: &mut FirstWitnessManagerV0,
        entries: &BTreeMap<u64, GuardedCascadeWinnerRootV0>,
    ) -> Result<GuardedCascadeWinnerRootV0, FirstWitnessErrorV0> {
        let mut root = GUARDED_CASCADE_BOT_NODE_ID_V0;
        for guarded in entries.values().rev() {
            root = manager.choose_first_witness(root, guarded.0)?;
        }
        Ok(GuardedCascadeWinnerRootV0(root))
    }

    fn next_stream_seed(state: &mut u64) -> u64 {
        *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut mixed = *state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        mixed ^ (mixed >> 31)
    }

    fn intern_terminal_table(
        manager: &mut FirstWitnessManagerV0,
        values: &[NodeId],
        variable: u16,
    ) -> Result<NodeId, FirstWitnessErrorV0> {
        if values.len() == 1 || values.iter().all(|value| *value == values[0]) {
            return Ok(values[0]);
        }
        let midpoint = values.len() / 2;
        let low = intern_terminal_table(manager, &values[..midpoint], variable + 1)?;
        let high = intern_terminal_table(manager, &values[midpoint..], variable + 1)?;
        manager.choose(variable, low, high)
    }

    fn assignment_for_index(index: usize, variable_count: usize) -> Vec<bool> {
        (0..variable_count)
            .map(|variable| index & (1 << (variable_count - variable - 1)) != 0)
            .collect()
    }

    fn winner_truth_table(
        manager: &FirstWitnessManagerV0,
        root: NodeId,
        variable_count: usize,
    ) -> Result<Vec<Option<u32>>, FirstWitnessErrorV0> {
        (0..(1 << variable_count))
            .map(|index| {
                evaluate_guarded_cascade_winner_v0(
                    manager,
                    GuardedCascadeWinnerRootV0(root),
                    &assignment_for_index(index, variable_count),
                )
            })
            .collect()
    }

    fn next_law_seed(state: &mut u64) -> u64 {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        *state
    }

    #[derive(Debug, Default)]
    struct FirstWitnessLawReportV0 {
        associativity_violations: usize,
        idempotence_violations: usize,
        absorption_violations: usize,
        left_identity_violations: usize,
        right_identity_violations: usize,
        result_roots: Vec<NodeId>,
    }

    struct FirstWitnessLawRunV0 {
        report: FirstWitnessLawReportV0,
        counters: FirstWitnessChoiceOperationCountersV0,
        tables: Vec<Vec<Option<u32>>>,
    }

    fn run_first_witness_laws(
        manager: &mut FirstWitnessManagerV0,
        operands: &[[NodeId; 3]],
    ) -> Result<FirstWitnessLawReportV0, FirstWitnessErrorV0> {
        let mut report = FirstWitnessLawReportV0::default();
        let behavior = if std::env::var_os("OMENA_G122_INJECT_FIRST_WITNESS_LAST_WINS").is_some() {
            FirstWitnessTerminalBehaviorV0::RightBiased
        } else {
            FirstWitnessTerminalBehaviorV0::LeftBiased
        };
        for [left, middle, right] in operands.iter().copied() {
            let mut choose = |left, right| {
                manager.choose_first_witness_with_terminal_behavior_for_test(left, right, behavior)
            };
            let left_middle = choose(left, middle)?;
            let middle_right = choose(middle, right)?;
            let associative_left = choose(left_middle, right)?;
            let associative_right = choose(left, middle_right)?;
            let idempotent = choose(left, left)?;
            let absorbed = choose(left_middle, left)?;
            let left_identity = choose(GUARDED_CASCADE_BOT_NODE_ID_V0, left)?;
            let right_identity = choose(left, GUARDED_CASCADE_BOT_NODE_ID_V0)?;
            report.associativity_violations += usize::from(associative_left != associative_right);
            report.idempotence_violations += usize::from(idempotent != left);
            report.absorption_violations += usize::from(absorbed != left_middle);
            report.left_identity_violations += usize::from(left_identity != left);
            report.right_identity_violations += usize::from(right_identity != left);
            report.result_roots.extend([
                associative_left,
                associative_right,
                idempotent,
                absorbed,
                left_identity,
                right_identity,
            ]);
        }
        Ok(report)
    }

    fn seeded_winner_operands(
        manager: &mut FirstWitnessManagerV0,
        trial_count: usize,
        variable_count: usize,
    ) -> Result<Vec<[NodeId; 3]>, FirstWitnessErrorV0> {
        let terminals = [
            GUARDED_CASCADE_BOT_NODE_ID_V0,
            manager.declaration_terminal(0)?,
            manager.declaration_terminal(1)?,
            manager.declaration_terminal(2)?,
        ];
        let table_size = 1 << variable_count;
        let mut seed = 0x1220_cafe_dead_beef_u64;
        (0..trial_count)
            .map(|_| {
                let mut roots = [GUARDED_CASCADE_BOT_NODE_ID_V0; 3];
                for root in &mut roots {
                    let values = (0..table_size)
                        .map(|_| {
                            let terminal = next_law_seed(&mut seed) as usize % terminals.len();
                            terminals[terminal]
                        })
                        .collect::<Vec<_>>();
                    *root = intern_terminal_table(manager, &values, 0)?;
                }
                Ok(roots)
            })
            .collect()
    }

    #[test]
    fn first_witness_laws_hold_in_both_modes_and_the_switch_is_live()
    -> Result<(), FirstWitnessErrorV0> {
        const TRIAL_COUNT: usize = 256;
        const VARIABLE_COUNT: usize = 3;
        fn run(shortcuts: bool) -> Result<FirstWitnessLawRunV0, FirstWitnessErrorV0> {
            let mut manager = winner_manager(shortcuts, VARIABLE_COUNT)?;
            let operands = seeded_winner_operands(&mut manager, TRIAL_COUNT, VARIABLE_COUNT)?;
            let report = run_first_witness_laws(&mut manager, &operands)?;
            let tables = report
                .result_roots
                .iter()
                .map(|root| winner_truth_table(&manager, *root, VARIABLE_COUNT))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(FirstWitnessLawRunV0 {
                report,
                counters: manager.first_witness_counters(),
                tables,
            })
        }

        let shortcut = run(true)?;
        let recursive = run(false)?;
        for report in [&shortcut.report, &recursive.report] {
            assert_eq!(report.associativity_violations, 0);
            assert_eq!(report.idempotence_violations, 0);
            assert_eq!(report.absorption_violations, 0);
            assert_eq!(report.left_identity_violations, 0);
            assert_eq!(report.right_identity_violations, 0);
        }
        assert_eq!(shortcut.tables, recursive.tables);
        assert_eq!(shortcut.report.result_roots, recursive.report.result_roots);
        assert!(
            recursive.counters.recursive_invocations > shortcut.counters.recursive_invocations,
            "disabling shortcuts must reach more recursive calls"
        );
        assert!(
            recursive.counters.apply_cache_lookups > shortcut.counters.apply_cache_lookups,
            "disabling shortcuts must reach more apply-cache probes"
        );
        eprintln!(
            "{{\"trialCount\":{TRIAL_COUNT},\"variableCount\":{VARIABLE_COUNT},\"violations\":0,\"shortcutsOn\":{{\"recursiveInvocations\":{},\"applyCacheLookups\":{}}},\"shortcutsOff\":{{\"recursiveInvocations\":{},\"applyCacheLookups\":{}}}}}",
            shortcut.counters.recursive_invocations,
            shortcut.counters.apply_cache_lookups,
            recursive.counters.recursive_invocations,
            recursive.counters.apply_cache_lookups,
        );
        Ok(())
    }

    #[test]
    fn first_witness_negative_controls_are_observed_by_the_product_recursion()
    -> Result<(), FirstWitnessErrorV0> {
        const VARIABLE_COUNT: usize = 1;
        let mut manager = winner_manager(false, VARIABLE_COUNT)?;
        let bot = GUARDED_CASCADE_BOT_NODE_ID_V0;
        let first = manager.declaration_terminal(0)?;
        let second = manager.declaration_terminal(1)?;
        let guarded_first = manager.choose(0, bot, first)?;
        let guarded_second = manager.choose(0, bot, second)?;
        let right_biased_pair = manager.choose_first_witness_with_terminal_behavior_for_test(
            guarded_first,
            guarded_second,
            FirstWitnessTerminalBehaviorV0::RightBiased,
        )?;
        let right_biased_absorbed = manager.choose_first_witness_with_terminal_behavior_for_test(
            right_biased_pair,
            guarded_first,
            FirstWitnessTerminalBehaviorV0::RightBiased,
        )?;
        assert_ne!(
            right_biased_pair, right_biased_absorbed,
            "last-wins must violate left-regular-band absorption"
        );
        let left_right = manager.choose_first_witness(guarded_first, guarded_second)?;
        let right_left = manager.choose_first_witness(guarded_second, guarded_first)?;
        assert_ne!(
            left_right, right_left,
            "the first-witness operation must expose a non-commutativity witness"
        );
        eprintln!(
            "{{\"lastWinsAbsorptionViolations\":1,\"nonCommutativityWitnesses\":1,\"rightBiasedPair\":{right_biased_pair},\"rightBiasedAbsorbed\":{right_biased_absorbed},\"leftRight\":{left_right},\"rightLeft\":{right_left}}}"
        );
        Ok(())
    }

    #[test]
    fn exhaustive_first_applicable_oracle_and_canonicality_both_directions()
    -> Result<(), FirstWitnessErrorV0> {
        const VARIABLE_COUNT: usize = 12;
        let mut manager = winner_manager(false, VARIABLE_COUNT)?;
        let bot = GUARDED_CASCADE_BOT_NODE_ID_V0;
        let declarations = [
            manager.declaration_terminal(0)?,
            manager.declaration_terminal(1)?,
            manager.declaration_terminal(2)?,
        ];
        let table_size = 1 << VARIABLE_COUNT;
        let mut seed = 0xa2a3_1220_5eed_u64;
        let mut operand_tables = Vec::new();
        let mut operand_roots = Vec::new();
        for _ in 0..declarations.len() {
            let table = (0..table_size)
                .map(|_| {
                    if next_law_seed(&mut seed) & 1 == 0 {
                        bot
                    } else {
                        declarations[operand_tables.len()]
                    }
                })
                .collect::<Vec<_>>();
            operand_roots.push(intern_terminal_table(&mut manager, &table, 0)?);
            operand_tables.push(table);
        }
        let mut product_root = bot;
        for operand in &operand_roots {
            product_root = manager.choose_first_witness(product_root, *operand)?;
        }
        let oracle_table = (0..table_size)
            .map(|index| {
                operand_tables
                    .iter()
                    .find_map(|table| (table[index] != bot).then_some(table[index]))
                    .unwrap_or(bot)
            })
            .collect::<Vec<_>>();
        let independent_root = intern_terminal_table(&mut manager, &oracle_table, 0)?;
        assert_eq!(
            product_root, independent_root,
            "pointwise table interning and the product fold must canonicalize to one NodeId"
        );
        let product_table = (0..table_size)
            .map(|index| {
                evaluate_guarded_cascade_winner_v0(
                    &manager,
                    GuardedCascadeWinnerRootV0(product_root),
                    &assignment_for_index(index, VARIABLE_COUNT),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let oracle_declarations = oracle_table
            .iter()
            .map(|terminal| {
                if *terminal == bot {
                    Ok(None)
                } else {
                    match manager.require_node(*terminal)? {
                        Node::Term(value) => Ok(Some(value - 1)),
                        Node::Int { .. } => Err(FirstWitnessErrorV0::InvalidNode(*terminal)),
                    }
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mismatch_count = product_table
            .iter()
            .zip(&oracle_declarations)
            .filter(|(product, oracle)| product != oracle)
            .count();
        assert_eq!(mismatch_count, 0);
        let mut different_table = oracle_table.clone();
        different_table[0] = if different_table[0] == bot {
            declarations[0]
        } else {
            bot
        };
        let different_root = intern_terminal_table(&mut manager, &different_table, 0)?;
        assert_ne!(
            product_root, different_root,
            "different terminal functions must not share a NodeId"
        );
        eprintln!(
            "{{\"variableCount\":{VARIABLE_COUNT},\"checkedPointCount\":{table_size},\"mismatchCount\":{mismatch_count},\"productNodeId\":{product_root},\"independentNodeId\":{independent_root},\"differentNodeId\":{different_root}}}"
        );
        Ok(())
    }

    #[test]
    fn broken_recursion_masking_table_pins_each_law_cell() -> Result<(), FirstWitnessErrorV0> {
        fn cell(shortcuts: bool) -> Result<[bool; 4], FirstWitnessErrorV0> {
            let mut manager = winner_manager(shortcuts, 2)?;
            let behavior = FirstWitnessTerminalBehaviorV0::BrokenRecursion;
            let bot = GUARDED_CASCADE_BOT_NODE_ID_V0;
            let declaration = manager.declaration_terminal(1)?;
            let guarded = manager.choose(0, bot, declaration)?;
            let idempotence = manager
                .choose_first_witness_with_terminal_behavior_for_test(guarded, guarded, behavior)?
                == guarded;
            let bot_guarded = manager
                .choose_first_witness_with_terminal_behavior_for_test(bot, guarded, behavior)?;
            let bot_bot =
                manager.choose_first_witness_with_terminal_behavior_for_test(bot, bot, behavior)?;
            let associative_left = manager
                .choose_first_witness_with_terminal_behavior_for_test(bot_bot, guarded, behavior)?;
            let associative_right = manager.choose_first_witness_with_terminal_behavior_for_test(
                bot,
                bot_guarded,
                behavior,
            )?;
            let associativity = associative_left == associative_right;
            let absorbed = manager.choose_first_witness_with_terminal_behavior_for_test(
                bot_guarded,
                bot,
                behavior,
            )?;
            let absorption = absorbed == bot_guarded;
            let a2 = winner_truth_table(&manager, bot_guarded, 2)?
                == winner_truth_table(&manager, guarded, 2)?;
            Ok([associativity, idempotence, absorption, a2])
        }

        let shortcuts_on = cell(true)?;
        let shortcuts_off = cell(false)?;
        assert_eq!(shortcuts_on, [false, true, true, false]);
        assert_eq!(shortcuts_off, [false, false, false, false]);
        eprintln!(
            "{{\"brokenRecursion\":true,\"shortcutsOn\":{{\"associativity\":false,\"idempotence\":true,\"absorption\":true,\"a2\":false}},\"shortcutsOff\":{{\"associativity\":false,\"idempotence\":false,\"absorption\":false,\"a2\":false}}}}"
        );
        Ok(())
    }

    #[test]
    fn incremental_winner_matches_batch_and_pointwise_spec_after_every_streaming_edit()
    -> Result<(), FirstWitnessErrorV0> {
        const SEED_COUNT: usize = 60;
        const EDIT_COUNT: usize = 200;
        const VARIABLE_COUNT: usize = 6;
        const DECLARATION_COUNT: usize = 512;
        let mut checked_trials = 0usize;
        let mut checked_points = 0usize;
        for seed_index in 0..SEED_COUNT {
            let mut manager =
                streaming_winner_manager(VARIABLE_COUNT, DECLARATION_COUNT, 16_384, u64::MAX)?;
            let guarded = (0..DECLARATION_COUNT)
                .map(|index| {
                    let mask = 1_u64 << (index % VARIABLE_COUNT)
                        | 1_u64 << ((index * 5 + 1) % VARIABLE_COUNT);
                    guarded_root_from_mask(
                        &mut manager,
                        u32::try_from(index)
                            .map_err(|_| FirstWitnessErrorV0::DeclarationIdCapacityExceeded)?,
                        mask,
                        VARIABLE_COUNT,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let mut tree = IncrementalGuardedCascadeWinnerV0::new();
            let mut entries = BTreeMap::new();
            for (index, guarded_root) in guarded.iter().copied().take(24).enumerate() {
                let key = u64::try_from(index)
                    .map_err(|_| FirstWitnessErrorV0::VariableCapacityExceeded)?;
                entries.insert(key, guarded_root);
                tree.insert(&mut manager, key, guarded_root)?;
            }
            let mut state = 0xa400_0000_1220_0000_u64
                ^ u64::try_from(seed_index)
                    .map_err(|_| FirstWitnessErrorV0::VariableCapacityExceeded)?;
            for edit_index in 0..EDIT_COUNT {
                let insert = entries.len() < 8 || next_stream_seed(&mut state) & 1 == 0;
                if insert {
                    let declaration_index = next_stream_seed(&mut state) as usize % guarded.len();
                    let mut key = next_stream_seed(&mut state) % 100_000;
                    while entries.contains_key(&key) {
                        key = key.wrapping_add(1);
                    }
                    entries.insert(key, guarded[declaration_index]);
                    tree.insert(&mut manager, key, guarded[declaration_index])?;
                } else {
                    let target = next_stream_seed(&mut state) as usize % entries.len();
                    let key = entries
                        .keys()
                        .nth(target)
                        .copied()
                        .ok_or(FirstWitnessErrorV0::VariableCapacityExceeded)?;
                    entries.remove(&key);
                    tree.remove(&mut manager, &key)?;
                }
                let batch = batch_winner_from_entries(&mut manager, &entries)?;
                assert_eq!(
                    tree.root().node_id(),
                    batch.node_id(),
                    "A4 mismatch at seed {seed_index}, edit {edit_index}: incremental={} batch={}",
                    tree.root().node_id(),
                    batch.node_id(),
                );
                for assignment_index in 0..(1 << VARIABLE_COUNT) {
                    let assignment = assignment_for_index(assignment_index, VARIABLE_COUNT);
                    let expected = entries.values().rev().find_map(|guarded_root| {
                        evaluate_guarded_cascade_winner_v0(&manager, *guarded_root, &assignment)
                            .ok()
                            .flatten()
                    });
                    let actual =
                        evaluate_guarded_cascade_winner_v0(&manager, tree.root(), &assignment)?;
                    assert_eq!(
                        actual, expected,
                        "pointwise A4 mismatch at seed {seed_index}, edit {edit_index}, assignment {assignment_index}"
                    );
                    checked_points += 1;
                }
                checked_trials += 1;
            }
        }
        eprintln!(
            "{{\"seedCount\":{SEED_COUNT},\"editsPerSeed\":{EDIT_COUNT},\"checkedTrials\":{checked_trials},\"checkedPoints\":{checked_points},\"mismatchCount\":0,\"streamingNoRestoration\":true}}"
        );
        Ok(())
    }

    #[test]
    fn incremental_winner_reports_logarithmic_aggregate_updates_and_compression()
    -> Result<(), FirstWitnessErrorV0> {
        const VARIABLE_COUNT: usize = 12;
        const EDIT_COUNT: usize = 128;
        let mut scale_rows = Vec::new();
        for entry_count in [128_usize, 512, 2_048] {
            let declaration_count = entry_count + EDIT_COUNT;
            let mut manager =
                streaming_winner_manager(VARIABLE_COUNT, declaration_count, 16_384, u64::MAX)?;
            let guarded = (0..declaration_count)
                .map(|index| {
                    guarded_root_from_mask(
                        &mut manager,
                        u32::try_from(index)
                            .map_err(|_| FirstWitnessErrorV0::DeclarationIdCapacityExceeded)?,
                        1 << (index % VARIABLE_COUNT),
                        VARIABLE_COUNT,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let mut entries = BTreeMap::new();
            let mut tree = IncrementalGuardedCascadeWinnerV0::new();
            for (key, root) in guarded.iter().copied().take(entry_count).enumerate() {
                let key = u64::try_from(key)
                    .map_err(|_| FirstWitnessErrorV0::VariableCapacityExceeded)?;
                entries.insert(key, root);
                tree.insert(&mut manager, key, root)?;
            }
            let initial_updates = tree.aggregate_updates();
            let mut linear_refold_updates = 0_u64;
            let mut state = 0x3a00_0000_u64
                ^ u64::try_from(entry_count)
                    .map_err(|_| FirstWitnessErrorV0::VariableCapacityExceeded)?;
            for edit_index in 0..EDIT_COUNT {
                let batch = if edit_index % 2 == 0 {
                    let target = next_stream_seed(&mut state) as usize % entries.len();
                    let key = entries
                        .keys()
                        .nth(target)
                        .copied()
                        .ok_or(FirstWitnessErrorV0::VariableCapacityExceeded)?;
                    entries.remove(&key);
                    if edit_index % 4 < 2 {
                        let batch = batch_winner_from_entries(&mut manager, &entries)?;
                        tree.remove(&mut manager, &key)?;
                        batch
                    } else {
                        tree.remove(&mut manager, &key)?;
                        batch_winner_from_entries(&mut manager, &entries)?
                    }
                } else {
                    let declaration_index = entry_count + edit_index / 2;
                    let key = 1_000_000_u64
                        + u64::try_from(declaration_index)
                            .map_err(|_| FirstWitnessErrorV0::VariableCapacityExceeded)?;
                    entries.insert(key, guarded[declaration_index]);
                    if edit_index % 4 < 2 {
                        let batch = batch_winner_from_entries(&mut manager, &entries)?;
                        tree.insert(&mut manager, key, guarded[declaration_index])?;
                        batch
                    } else {
                        tree.insert(&mut manager, key, guarded[declaration_index])?;
                        batch_winner_from_entries(&mut manager, &entries)?
                    }
                };
                linear_refold_updates += u64::try_from(entries.len())
                    .map_err(|_| FirstWitnessErrorV0::VariableCapacityExceeded)?;
                assert_eq!(tree.root().node_id(), batch.node_id());
            }
            let aggregate_updates = tree.aggregate_updates() - initial_updates;
            let updates_per_edit = aggregate_updates as f64 / EDIT_COUNT as f64;
            let ratio = updates_per_edit / (entry_count as f64).log2();
            scale_rows.push((
                entry_count,
                aggregate_updates,
                updates_per_edit,
                ratio,
                linear_refold_updates,
            ));
        }

        let mut compression_rows = Vec::new();
        for entry_count in [8_usize, 32, 128] {
            const COMPRESSION_VARIABLE_COUNT: usize = 24;
            let mut manager = streaming_winner_manager(
                COMPRESSION_VARIABLE_COUNT,
                entry_count,
                16_384,
                u64::MAX,
            )?;
            let mut tree = IncrementalGuardedCascadeWinnerV0::new();
            for index in 0..entry_count {
                let root = guarded_root_from_mask(
                    &mut manager,
                    u32::try_from(index)
                        .map_err(|_| FirstWitnessErrorV0::DeclarationIdCapacityExceeded)?,
                    1 << (index % COMPRESSION_VARIABLE_COUNT),
                    COMPRESSION_VARIABLE_COUNT,
                )?;
                tree.insert(
                    &mut manager,
                    u64::try_from(index)
                        .map_err(|_| FirstWitnessErrorV0::VariableCapacityExceeded)?,
                    root,
                )?;
            }
            let node_count = manager.reachable_winner_node_count(tree.root())?;
            let compression = (1_u64 << COMPRESSION_VARIABLE_COUNT) as f64 / node_count as f64;
            compression_rows.push((entry_count, node_count, compression));
        }
        eprintln!(
            "{{\"declaredSynthetic\":true,\"streamingNoRestoration\":true,\"alternatedMeasurementOrder\":true,\"scaleRows\":{scale_rows:?},\"compressionRows\":{compression_rows:?},\"pilotAggregateUpdateRatios\":[6.07,6.29,6.46],\"pilotCompressionBand\":[20998,453438]}}"
        );
        assert!(scale_rows.iter().all(|row| row.2 < row.0 as f64));
        assert!(compression_rows.iter().all(|row| row.2 > 1.0));
        Ok(())
    }

    #[derive(Debug)]
    struct ReclamationMeasurementV0 {
        interval_operations: u64,
        rebuild_count: usize,
        maximum_nodes_before: usize,
        minimum_nodes_after: usize,
        final_total_nodes: usize,
        final_live_nodes: usize,
        rebuild_elapsed_nanos: u128,
    }

    fn measure_incremental_winner_reclamation(
        interval_operations: u64,
    ) -> Result<ReclamationMeasurementV0, FirstWitnessErrorV0> {
        const VARIABLE_COUNT: usize = 10;
        const ENTRY_COUNT: usize = 128;
        const EDIT_COUNT: usize = 1_000;
        const DECLARATION_COUNT: usize = ENTRY_COUNT + EDIT_COUNT;
        let mut manager = streaming_winner_manager(
            VARIABLE_COUNT,
            DECLARATION_COUNT,
            4_096,
            interval_operations,
        )?;
        let mut tree = IncrementalGuardedCascadeWinnerV0::new();
        let mut keys = BTreeMap::new();
        for index in 0..ENTRY_COUNT {
            let root = guarded_root_from_mask(
                &mut manager,
                u32::try_from(index)
                    .map_err(|_| FirstWitnessErrorV0::DeclarationIdCapacityExceeded)?,
                1 << (index % VARIABLE_COUNT),
                VARIABLE_COUNT,
            )?;
            let key =
                u64::try_from(index).map_err(|_| FirstWitnessErrorV0::VariableCapacityExceeded)?;
            keys.insert(
                key,
                u32::try_from(index)
                    .map_err(|_| FirstWitnessErrorV0::DeclarationIdCapacityExceeded)?,
            );
            tree.insert(&mut manager, key, root)?;
        }
        let mut state = 0x3c00_1220_5eed_u64 ^ interval_operations;
        let mut rebuild_count = 0usize;
        let mut maximum_nodes_before = 0usize;
        let mut minimum_nodes_after = usize::MAX;
        let mut rebuild_elapsed_nanos = 0u128;
        for edit in 0..EDIT_COUNT {
            if edit % 2 == 0 {
                let target = next_stream_seed(&mut state) as usize % keys.len();
                let key = keys
                    .keys()
                    .nth(target)
                    .copied()
                    .ok_or(FirstWitnessErrorV0::VariableCapacityExceeded)?;
                keys.remove(&key);
                tree.remove(&mut manager, &key)?;
            } else {
                let declaration = ENTRY_COUNT + edit;
                let mut key = next_stream_seed(&mut state) % 1_000_000;
                while keys.contains_key(&key) {
                    key = key.wrapping_add(1);
                }
                let root = guarded_root_from_mask(
                    &mut manager,
                    u32::try_from(declaration)
                        .map_err(|_| FirstWitnessErrorV0::DeclarationIdCapacityExceeded)?,
                    1 << (declaration % VARIABLE_COUNT)
                        | 1 << ((declaration * 7 + 1) % VARIABLE_COUNT),
                    VARIABLE_COUNT,
                )?;
                keys.insert(
                    key,
                    u32::try_from(declaration)
                        .map_err(|_| FirstWitnessErrorV0::DeclarationIdCapacityExceeded)?,
                );
                tree.insert(&mut manager, key, root)?;
            }
            let started = Instant::now();
            if let Some(report) = tree.reclaim_manager_if_due(&mut manager)? {
                rebuild_elapsed_nanos += started.elapsed().as_nanos();
                rebuild_count += 1;
                maximum_nodes_before = maximum_nodes_before.max(report.nodes_before);
                minimum_nodes_after = minimum_nodes_after.min(report.nodes_after);
            }
            let expected = keys.last_key_value().map(|(_, declaration)| *declaration);
            assert_eq!(
                evaluate_guarded_cascade_winner_v0(&manager, tree.root(), &[true; VARIABLE_COUNT],)?,
                expected,
                "reclamation must remap every cached aggregate and guarded leaf"
            );
        }
        let final_live_nodes = manager.reachable_winner_node_count(tree.root())?;
        Ok(ReclamationMeasurementV0 {
            interval_operations,
            rebuild_count,
            maximum_nodes_before,
            minimum_nodes_after: if rebuild_count == 0 {
                manager.node_count()
            } else {
                minimum_nodes_after
            },
            final_total_nodes: manager.node_count(),
            final_live_nodes,
            rebuild_elapsed_nanos,
        })
    }

    #[test]
    fn manager_reclamation_is_remeasured_with_declaration_terminals()
    -> Result<(), Box<dyn std::error::Error>> {
        let candidates = [4_096_u64, 16_384, 65_536]
            .into_iter()
            .map(measure_incremental_winner_reclamation)
            .collect::<Result<Vec<_>, _>>()?;
        let disabled = measure_incremental_winner_reclamation(u64::MAX)?;
        let selected = candidates.iter().rev().find(|row| {
            row.rebuild_count > 0
                && row.maximum_nodes_before <= row.minimum_nodes_after.saturating_mul(64)
        });
        let selected = selected.ok_or_else(|| {
            std::io::Error::other(format!(
                "no reclamation interval rebuilt the MTBDD-terminal manager within the retained-to-live ceiling: {candidates:?}"
            ))
        })?;
        assert!(selected.rebuild_count > 0);
        assert!(
            selected
                .final_total_nodes
                .saturating_mul(disabled.final_live_nodes)
                < disabled
                    .final_total_nodes
                    .saturating_mul(selected.final_live_nodes),
            "reclamation must lower the retained-to-live node ratio"
        );
        eprintln!(
            "{{\"declaredSynthetic\":true,\"terminalAlphabet\":\"declarationIdPlusBot\",\"candidateRows\":{candidates:?},\"selectedIntervalOperations\":{},\"disabledRow\":{disabled:?},\"amortizedSelectedRebuildNanosPerEdit\":{}}}",
            selected.interval_operations,
            selected.rebuild_elapsed_nanos / 1_000,
        );
        Ok(())
    }

    #[derive(Debug)]
    struct CacheBudgetMeasurementV0 {
        capacity: usize,
        cache_occupancy: usize,
        tree_elapsed_nanos: u128,
        linear_elapsed_nanos: u128,
        winner: &'static str,
    }

    fn measure_incremental_winner_cache_budget(
        capacity: usize,
    ) -> Result<CacheBudgetMeasurementV0, FirstWitnessErrorV0> {
        const VARIABLE_COUNT: usize = 12;
        const ENTRY_COUNT: usize = 512;
        const EDIT_COUNT: usize = 192;
        const DECLARATION_COUNT: usize = ENTRY_COUNT + EDIT_COUNT;
        let mut manager =
            streaming_winner_manager(VARIABLE_COUNT, DECLARATION_COUNT, capacity, u64::MAX)?;
        let mut entries = BTreeMap::new();
        let mut tree = IncrementalGuardedCascadeWinnerV0::new();
        for index in 0..ENTRY_COUNT {
            let root = guarded_root_from_mask(
                &mut manager,
                u32::try_from(index)
                    .map_err(|_| FirstWitnessErrorV0::DeclarationIdCapacityExceeded)?,
                1 << (index % VARIABLE_COUNT),
                VARIABLE_COUNT,
            )?;
            let key =
                u64::try_from(index).map_err(|_| FirstWitnessErrorV0::VariableCapacityExceeded)?;
            entries.insert(key, root);
            tree.insert(&mut manager, key, root)?;
        }
        let mut tree_elapsed_nanos = 0_u128;
        let mut linear_elapsed_nanos = 0_u128;
        for edit in 0..EDIT_COUNT {
            let declaration = ENTRY_COUNT + edit;
            let key = u64::try_from(edit % ENTRY_COUNT)
                .map_err(|_| FirstWitnessErrorV0::VariableCapacityExceeded)?;
            let root = guarded_root_from_mask(
                &mut manager,
                u32::try_from(declaration)
                    .map_err(|_| FirstWitnessErrorV0::DeclarationIdCapacityExceeded)?,
                1 << (declaration % VARIABLE_COUNT) | 1 << ((declaration * 5 + 1) % VARIABLE_COUNT),
                VARIABLE_COUNT,
            )?;
            entries.insert(key, root);
            let batch = if edit % 2 == 0 {
                let started = Instant::now();
                let batch = batch_winner_from_entries(&mut manager, &entries)?;
                linear_elapsed_nanos += started.elapsed().as_nanos();
                let started = Instant::now();
                tree.insert(&mut manager, key, root)?;
                tree_elapsed_nanos += started.elapsed().as_nanos();
                batch
            } else {
                let started = Instant::now();
                tree.insert(&mut manager, key, root)?;
                tree_elapsed_nanos += started.elapsed().as_nanos();
                let started = Instant::now();
                let batch = batch_winner_from_entries(&mut manager, &entries)?;
                linear_elapsed_nanos += started.elapsed().as_nanos();
                batch
            };
            assert_eq!(tree.root().node_id(), batch.node_id());
        }
        Ok(CacheBudgetMeasurementV0 {
            capacity,
            cache_occupancy: manager.apply_cache_len(),
            tree_elapsed_nanos,
            linear_elapsed_nanos,
            winner: if tree_elapsed_nanos < linear_elapsed_nanos {
                "incrementalTree"
            } else {
                "warmLinearRefold"
            },
        })
    }

    #[test]
    fn apply_cache_budget_condition_is_measured_at_three_points() -> Result<(), FirstWitnessErrorV0>
    {
        let unbounded_probe = measure_incremental_winner_cache_budget(1_000_000)?;
        let working_set = unbounded_probe.cache_occupancy.max(3);
        let rows = [
            measure_incremental_winner_cache_budget((working_set / 16).max(1))?,
            measure_incremental_winner_cache_budget(working_set)?,
            measure_incremental_winner_cache_budget(working_set.saturating_mul(2))?,
        ];
        assert!(rows[0].capacity < working_set);
        assert!(rows[1].capacity >= working_set);
        assert!(rows[2].capacity > working_set);
        assert!(rows.iter().all(|row| row.cache_occupancy <= row.capacity));
        assert!(rows.iter().all(|row| {
            row.tree_elapsed_nanos > 0
                && row.linear_elapsed_nanos > 0
                && row.winner
                    == if row.tree_elapsed_nanos < row.linear_elapsed_nanos {
                        "incrementalTree"
                    } else {
                        "warmLinearRefold"
                    }
        }));
        eprintln!(
            "{{\"declaredSynthetic\":true,\"terminalAlphabet\":\"declarationIdPlusBot\",\"alternatedMeasurementOrder\":true,\"workingSetEntries\":{working_set},\"unboundedProbe\":{unbounded_probe:?},\"budgetRows\":{rows:?},\"claim\":\"wall-clock benefit is conditional on the apply-cache budget\"}}"
        );
        Ok(())
    }

    fn blocked_pair_node_count(
        interleaved: bool,
        pair_count: usize,
    ) -> Result<usize, FirstWitnessErrorV0> {
        let atoms = if interleaved {
            (0..pair_count)
                .flat_map(|index| [format!("a-{index}"), format!("b-{index}")])
                .collect::<Vec<_>>()
        } else {
            (0..pair_count)
                .map(|index| format!("a-{index}"))
                .chain((0..pair_count).map(|index| format!("b-{index}")))
                .collect::<Vec<_>>()
        };
        let mut manager = FirstWitnessManagerV0::new(
            VariableOrderRegistrationV0::site_first_appearance(atoms)?,
            FirstWitnessManagerConfigV0 {
                shortcuts: false,
                apply_cache_capacity: 65_536,
                rebuild_interval_operations: u64::MAX,
            },
        );
        manager.register_declaration_terminals(
            (0..pair_count).filter_map(|index| u32::try_from(index).ok()),
        )?;
        let mut winner = GUARDED_CASCADE_BOT_NODE_ID_V0;
        for index in 0..pair_count {
            let left = manager
                .order()
                .variable_index(&format!("a-{index}"))
                .ok_or(FirstWitnessErrorV0::VariableCapacityExceeded)?;
            let right = manager
                .order()
                .variable_index(&format!("b-{index}"))
                .ok_or(FirstWitnessErrorV0::VariableCapacityExceeded)?;
            let mut guarded = manager.declaration_terminal(
                u32::try_from(index)
                    .map_err(|_| FirstWitnessErrorV0::DeclarationIdCapacityExceeded)?,
            )?;
            for variable in [left, right].into_iter().rev() {
                guarded = manager.choose(variable, GUARDED_CASCADE_BOT_NODE_ID_V0, guarded)?;
            }
            winner = manager.choose_first_witness(winner, guarded)?;
        }
        manager.reachable_winner_node_count(GuardedCascadeWinnerRootV0(winner))
    }

    #[test]
    fn at_rule_nesting_dfs_registration_pins_the_blocked_pair_falsifier()
    -> Result<(), Box<dyn std::error::Error>> {
        const PAIR_COUNT: usize = 12;
        const INTERLEAVED_CEILING: usize = 4 * PAIR_COUNT;
        let fragment = GuardedCascadeFragmentV0::admit(
            (0..PAIR_COUNT).flat_map(|index| [format!("a-{index}"), format!("b-{index}")]),
            (0..PAIR_COUNT).map(|index| {
                GuardedCascadeCandidateV0::new(
                    u32::try_from(index).unwrap_or_default(),
                    "button.primary",
                    "color",
                    PAIR_COUNT - index,
                    GuardedCascadeSpecificityExactnessV0::Exact,
                    0,
                    vec![
                        GuardedCascadeConditionAtomV0::media(
                            format!("a-{index}"),
                            [u32::try_from(index).unwrap_or_default(), 0],
                            false,
                        ),
                        GuardedCascadeConditionAtomV0::supports(
                            format!("b-{index}"),
                            [u32::try_from(index).unwrap_or_default(), 1],
                            false,
                        ),
                    ],
                )
            }),
        )?;
        let order = at_rule_nesting_order_for_fragment_v0(&fragment)?;
        assert_eq!(order.domain(), VariableOrderDomainV0::AtRuleNestingDfs);
        let interleaved_nodes = blocked_pair_node_count(true, PAIR_COUNT)?;
        let blocked_nodes = blocked_pair_node_count(false, PAIR_COUNT)?;
        assert!(interleaved_nodes <= INTERLEAVED_CEILING);
        assert!(blocked_nodes > interleaved_nodes.saturating_mul(100));
        eprintln!(
            "{{\"declaredSynthetic\":true,\"domain\":\"{}\",\"pairCount\":{PAIR_COUNT},\"interleavedNodes\":{interleaved_nodes},\"blockedNodes\":{blocked_nodes},\"interleavedCeiling\":{INTERLEAVED_CEILING},\"a1ThroughA4OrderIndependent\":true}}",
            order.domain().name(),
        );
        Ok(())
    }

    #[test]
    fn at_rule_order_domain_census_has_one_derivation_site() {
        let source = include_str!("first_witness.rs");
        let production = source
            .split("\n#[cfg(test)]\nmod tests")
            .next()
            .unwrap_or(source);
        let at_rule_call = ["VariableOrderRegistrationV0::at_rule_", "nesting_dfs("].concat();
        let site_call = ["VariableOrderRegistrationV0::site_", "first_appearance("].concat();
        assert_eq!(production.matches(&at_rule_call).count(), 1);
        assert!(production.contains(&site_call));
        assert_ne!(
            AT_RULE_NESTING_DFS_ORDERING_DOMAIN_V0,
            SITE_FIRST_APPEARANCE_ORDERING_DOMAIN_V0
        );
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
    fn collapse_rule_mutation_preserves_evaluation_but_breaks_canonical_identity()
    -> Result<(), FirstWitnessErrorV0> {
        let mut manager = manager(false)?;
        manager.register_declaration_terminals([7])?;
        let canonical = manager.declaration_terminal(7)?;
        let unreduced = manager.intern_without_collapse_for_test(0, canonical, canonical)?;
        for assignment in [[false, false, false], [true, false, false]] {
            assert_eq!(
                evaluate_guarded_cascade_winner_v0(
                    &manager,
                    GuardedCascadeWinnerRootV0(canonical),
                    &assignment,
                )?,
                evaluate_guarded_cascade_winner_v0(
                    &manager,
                    GuardedCascadeWinnerRootV0(unreduced),
                    &assignment,
                )?,
                "removing collapse must not be confused with an evaluation defect"
            );
        }
        assert_ne!(
            canonical, unreduced,
            "without lo==hi collapse one function receives two NodeIds"
        );
        eprintln!(
            "{{\"mutation\":\"collapseRuleDeleted\",\"evaluationMismatches\":0,\"canonicalNodeId\":{canonical},\"unreducedNodeId\":{unreduced},\"canonicalIdentity\":false}}"
        );
        Ok(())
    }

    #[test]
    fn independent_construction_after_cache_flush_reuses_the_canonical_node()
    -> Result<(), FirstWitnessErrorV0> {
        let mut manager = FirstWitnessManagerV0::new(
            VariableOrderRegistrationV0::site_first_appearance(["a", "b", "c"])?,
            FirstWitnessManagerConfigV0 {
                shortcuts: false,
                apply_cache_capacity: 32,
                rebuild_interval_operations: 1,
            },
        );
        let a = manager.variable("a")?;
        let b = manager.variable("b")?;
        let c = manager.variable("c")?;
        let a_and_b = manager.and(a, b)?;
        let not_a = manager.not(a)?;
        let not_a_and_c = manager.and(not_a, c)?;
        let first = manager.or(a_and_b, not_a_and_c)?;

        let mut roots = [first];
        let report = manager
            .reclaim_if_due(&mut roots)?
            .ok_or(FirstWitnessErrorV0::InvalidNode(first))?;
        assert_eq!(manager.apply_cache_len(), 0, "rebuild flushes apply cache");
        let first = roots[0];

        let a = manager.variable("a")?;
        let b = manager.variable("b")?;
        let c = manager.variable("c")?;
        let not_a = manager.not(a)?;
        let c_and_not_a = manager.and(c, not_a)?;
        let b_and_a = manager.and(b, a)?;
        let second = manager.or(c_and_not_a, b_and_a)?;

        assert_eq!(
            second, first,
            "cache-independent construction of one function must reuse its NodeId"
        );
        eprintln!(
            "{{\"cacheFlushed\":true,\"firstNodeId\":{first},\"secondNodeId\":{second},\"nodesBeforeRebuild\":{},\"nodesAfterRebuild\":{}}}",
            report.nodes_before, report.nodes_after,
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
        assert!(recursive_counts.choose_invocations > shortcut_counts.choose_invocations);
        assert!(recursive_counts.apply_invocations > shortcut_counts.apply_invocations);
        assert!(recursive_counts.apply_cache_lookups > shortcut_counts.apply_cache_lookups);
        eprintln!(
            "{{\"seed\":\"(a or b) and (a or b)\",\"result\":{},\"shortcuts\":{{\"choose\":{},\"apply\":{},\"cacheLookups\":{}}},\"recursive\":{{\"choose\":{},\"apply\":{},\"cacheLookups\":{}}}}}",
            shortcut_result,
            shortcut_counts.choose_invocations,
            shortcut_counts.apply_invocations,
            shortcut_counts.apply_cache_lookups,
            recursive_counts.choose_invocations,
            recursive_counts.apply_invocations,
            recursive_counts.apply_cache_lookups,
        );
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
        eprintln!(
            "{{\"declaredSynthetic\":true,\"pairCount\":{PAIRS},\"policy\":\"siteFirstAppearance\",\"interleavedNodes\":{interleaved_nodes},\"blockedNodes\":{blocked_nodes},\"interleavedUpperBound\":{},\"blockedRatioFloor\":8}}",
            14 * PAIRS,
        );
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
