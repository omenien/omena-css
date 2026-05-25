//! Public data model for cascade ordering, selector witnesses, and proof reports.
//!
//! These serializable types are the stable boundary consumed by query,
//! transform, conformance, fuzz, and LSP surfaces. They intentionally expose
//! evidence fields instead of opaque booleans so later passes can explain why a
//! cascade-sensitive rewrite was accepted or blocked.

use serde::Serialize;
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CascadeLevel {
    UserAgentNormal,
    UserNormal,
    AuthorNormal,
    InlineNormal,
    Animation,
    AuthorImportant,
    UserImportant,
    UserAgentImportant,
    Transition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LayerRank(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Specificity {
    pub ids: u32,
    pub classes: u32,
    pub elements: u32,
}

impl Specificity {
    pub const ZERO: Self = Self {
        ids: 0,
        classes: 0,
        elements: 0,
    };

    pub const fn new(ids: u32, classes: u32, elements: u32) -> Self {
        Self {
            ids,
            classes,
            elements,
        }
    }
}

impl Ord for Specificity {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.ids, self.classes, self.elements).cmp(&(other.ids, other.classes, other.elements))
    }
}

impl PartialOrd for Specificity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeKey {
    pub level: CascadeLevel,
    pub layer_rank: LayerRank,
    pub scope_proximity: u32,
    pub specificity: Specificity,
    pub source_order: u32,
}

impl CascadeKey {
    pub const fn new(
        level: CascadeLevel,
        layer_rank: LayerRank,
        scope_proximity: u32,
        specificity: Specificity,
        source_order: u32,
    ) -> Self {
        Self {
            level,
            layer_rank,
            scope_proximity,
            specificity,
            source_order,
        }
    }
}

impl Ord for CascadeKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.level
            .cmp(&other.level)
            .then_with(|| self.layer_rank.cmp(&other.layer_rank))
            .then_with(|| other.scope_proximity.cmp(&self.scope_proximity))
            .then_with(|| self.specificity.cmp(&other.specificity))
            .then_with(|| self.source_order.cmp(&other.source_order))
    }
}

impl PartialOrd for CascadeKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeDeclaration {
    pub id: String,
    pub property: String,
    pub value: CascadeValue,
    pub key: CascadeKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeProof {
    pub declaration_id: String,
    pub property: String,
    pub level: CascadeLevel,
    pub layer_rank: LayerRank,
    pub scope_proximity: u32,
    pub specificity: Specificity,
    pub source_order: u32,
}

impl CascadeProof {
    pub fn from_declaration(declaration: &CascadeDeclaration) -> Self {
        Self {
            declaration_id: declaration.id.clone(),
            property: declaration.property.clone(),
            level: declaration.key.level,
            layer_rank: declaration.key.layer_rank,
            scope_proximity: declaration.key.scope_proximity,
            specificity: declaration.key.specificity,
            source_order: declaration.key.source_order,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CascadeOutcome {
    Definite {
        winner: CascadeDeclaration,
        proof: CascadeProof,
        also_considered: Vec<CascadeDeclaration>,
    },
    RankedSet(Vec<CascadeDeclaration>),
    Inherit,
    Top,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CascadeValue {
    Literal(String),
    Composite(Vec<CascadeValue>),
    Var {
        name: String,
        fallback: Option<Box<CascadeValue>>,
    },
    Initial,
    Inherit,
    GuaranteedInvalid,
    Unset,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ComputedCascadeValueStatusV0 {
    Resolved,
    Inherited,
    Initial,
    InvalidAtComputedValueTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeComputedValueInputV0 {
    pub property: String,
    pub declarations: Vec<CascadeDeclaration>,
    pub custom_property_env: CustomPropertyEnv,
    pub parent_computed_value: Option<CascadeValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeComputedValueResultV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub property: String,
    pub status: ComputedCascadeValueStatusV0,
    pub value: CascadeValue,
    pub winner_declaration_id: Option<String>,
    pub inherited: bool,
    pub used_initial_value: bool,
    pub invalid_at_computed_value_time: bool,
    pub derivation_steps: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SelectorContextMatchKind {
    NoMatch,
    Global,
    Root,
    Exact,
    ContainsSelector,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorContextWitness {
    pub kind: SelectorContextMatchKind,
    pub matched: bool,
    pub rank: usize,
    pub declaration_selector: Option<String>,
    pub reference_selector: Option<String>,
}

impl SelectorContextWitness {
    pub fn no_match() -> Self {
        Self {
            kind: SelectorContextMatchKind::NoMatch,
            matched: false,
            rank: 0,
            declaration_selector: None,
            reference_selector: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementSignature {
    pub tag: Option<String>,
    pub id: Option<String>,
    pub classes: BTreeSet<String>,
    pub attributes: BTreeSet<String>,
    pub pseudo_states: BTreeSet<String>,
    pub classes_are_exact: bool,
    pub attributes_are_exact: bool,
    pub pseudo_states_are_exact: bool,
    pub tag_is_exact: bool,
    pub id_is_exact: bool,
}

impl ElementSignature {
    pub fn concrete(
        tag: Option<impl Into<String>>,
        id: Option<impl Into<String>>,
        classes: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            tag: tag.map(Into::into),
            id: id.map(Into::into),
            classes: classes.into_iter().map(Into::into).collect(),
            attributes: BTreeSet::new(),
            pseudo_states: BTreeSet::new(),
            classes_are_exact: true,
            attributes_are_exact: true,
            pseudo_states_are_exact: true,
            tag_is_exact: true,
            id_is_exact: true,
        }
    }

    pub fn at_least_classes(classes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            classes_are_exact: false,
            ..Self::concrete(None::<String>, None::<String>, classes)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorSignature {
    pub selector: String,
    pub required_tag: Option<String>,
    pub required_id: Option<String>,
    pub required_classes: BTreeSet<String>,
    pub required_attributes: BTreeSet<String>,
    pub required_pseudo_states: BTreeSet<String>,
    pub specificity: Specificity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SelectorMatchVerdict {
    No,
    Maybe,
    Yes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SelectorMatchReason {
    Universal,
    SimpleCompound,
    SelectorList,
    MissingTag,
    MissingId,
    MissingClass,
    MissingAttribute,
    MissingPseudoState,
    UnsupportedSelector,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorMatchWitness {
    pub selector: String,
    pub matched_branch: Option<String>,
    pub verdict: SelectorMatchVerdict,
    pub reason: SelectorMatchReason,
    pub specificity: Specificity,
    pub missing_tag: Option<String>,
    pub missing_id: Option<String>,
    pub missing_classes: BTreeSet<String>,
    pub missing_attributes: BTreeSet<String>,
    pub missing_pseudo_states: BTreeSet<String>,
    pub unsupported_branches: Vec<String>,
}

impl SelectorMatchWitness {
    pub(crate) fn unsupported(selector: &str) -> Self {
        Self {
            selector: selector.to_string(),
            matched_branch: Some(selector.to_string()),
            verdict: SelectorMatchVerdict::Maybe,
            reason: SelectorMatchReason::UnsupportedSelector,
            specificity: Specificity::ZERO,
            missing_tag: None,
            missing_id: None,
            missing_classes: BTreeSet::new(),
            missing_attributes: BTreeSet::new(),
            missing_pseudo_states: BTreeSet::new(),
            unsupported_branches: vec![selector.to_string()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeBoundarySummary {
    pub product: &'static str,
    pub ordering_model: &'static str,
    pub substitution_model: &'static str,
    pub least_fixed_point_proof_model: &'static str,
    pub ready_surfaces: Vec<&'static str>,
    pub not_ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeConformanceSeedCase {
    pub name: String,
    pub property: &'static str,
    pub declarations: Vec<CascadeDeclaration>,
    pub expected_outcome: &'static str,
    pub expected_winner_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeConformanceSeedResult {
    pub name: String,
    pub passed: bool,
    pub expected_outcome: &'static str,
    pub actual_outcome: &'static str,
    pub expected_winner_id: Option<String>,
    pub actual_winner_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeConformanceSeedReport {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub case_count: usize,
    pub passed_count: usize,
    pub failed_count: usize,
    pub results: Vec<CascadeConformanceSeedResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeEvaluationFuzzCaseV0 {
    pub seed: u64,
    pub declaration_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeEvaluationFuzzResultV0 {
    pub seed: u64,
    pub declaration_count: usize,
    pub actual_winner_id: Option<String>,
    pub expected_winner_id: Option<String>,
    pub ranked_count: usize,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VarSubstitutionFuzzCaseV0 {
    pub seed: u64,
    pub chain_len: usize,
    pub cycle: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VarSubstitutionFuzzResultV0 {
    pub seed: u64,
    pub chain_len: usize,
    pub cycle: bool,
    pub result: CascadeValue,
    pub expected: CascadeValue,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomPropertyLeastFixedPointSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub input_count: usize,
    pub resolved_count: usize,
    pub guaranteed_invalid_count: usize,
    pub iteration_count: usize,
    pub iteration_bound: usize,
    pub reached_fixed_point: bool,
    pub monotone_witness_valid: bool,
    pub proof: CustomPropertyLeastFixedPointProofV0,
    pub iteration_trace: Vec<CustomPropertyLeastFixedPointIterationV0>,
    pub entries: Vec<CustomPropertyLeastFixedPointEntryV0>,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomPropertyLeastFixedPointProofV0 {
    pub finite_domain: &'static str,
    pub transfer_function: &'static str,
    pub monotone_witness: &'static str,
    pub iteration_bound_formula: &'static str,
    pub cycle_policy: &'static str,
    pub proof_obligations: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomPropertyLeastFixedPointIterationV0 {
    pub iteration: usize,
    pub changed_count: usize,
    pub settled_count: usize,
    pub guaranteed_invalid_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomPropertyLeastFixedPointEntryV0 {
    pub name: String,
    pub input: CascadeValue,
    pub resolved: CascadeValue,
    pub changed: bool,
    pub guaranteed_invalid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeFuzzSeedReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub case_count: usize,
    pub passed_count: usize,
    pub failed_count: usize,
    pub cascade_results: Vec<CascadeEvaluationFuzzResultV0>,
    pub var_results: Vec<VarSubstitutionFuzzResultV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoxLonghandInputV0 {
    pub property: String,
    pub value: String,
    pub important: bool,
    pub source_order: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShorthandCombinationProofV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub shorthand_property: String,
    pub accepted: bool,
    pub blocked_reason: Option<&'static str>,
    pub ordered_longhand_properties: Vec<String>,
    pub provenance_preserved: bool,
    pub cascade_safe_witness: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StaticSupportsAssumptionV0 {
    ModernBrowser,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StaticSupportsEvalVerdictV0 {
    AlwaysTrue,
    AlwaysFalse,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticSupportsEvalWitnessV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub condition: String,
    pub assumption: StaticSupportsAssumptionV0,
    pub verdict: StaticSupportsEvalVerdictV0,
    pub reason: &'static str,
    pub provenance_preserved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeFlattenInputV0 {
    pub root_selector: String,
    pub limit_selector: Option<String>,
    pub scoped_rule_count: usize,
    pub peer_scope_count: usize,
    pub competing_unscoped_rule_count: usize,
    pub inside_layer: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeFlattenProofV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub accepted: bool,
    pub blocked_reason: Option<&'static str>,
    pub root_selector: String,
    pub provenance_preserved: bool,
    pub cascade_safe_witness: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LayerFlattenInputV0 {
    pub layer_name: Option<String>,
    pub layer_rule_count: usize,
    pub peer_layer_count: usize,
    pub unlayered_rule_count: usize,
    pub important_declaration_count: usize,
    pub closed_bundle: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LayerFlattenProofV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub accepted: bool,
    pub blocked_reason: Option<&'static str>,
    pub layer_name: Option<String>,
    pub provenance_preserved: bool,
    pub cascade_safe_witness: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "witnessKind", content = "witness", rename_all = "camelCase")]
pub enum ModalCheckWitnessSourceV0 {
    ShorthandCombination(ShorthandCombinationProofV0),
    StaticSupportsEval(StaticSupportsEvalWitnessV0),
    ScopeFlatten(ScopeFlattenProofV0),
    LayerFlatten(LayerFlattenProofV0),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
/// V0 freeze-candidate witness aggregation over existing cascade proof outputs.
///
/// This is a staged strict-superset surface for release evidence. It does not
/// claim a completed modal theorem, paper-grade proof system, or Cargo 1.0 API.
pub struct ModalCheckWitnessV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub modal_family: &'static str,
    pub substrate: &'static str,
    pub obligation_count: usize,
    pub accepted_count: usize,
    pub blocked_count: usize,
    pub all_provenance_preserved: bool,
    pub source_products: Vec<&'static str>,
    pub witnesses: Vec<ModalCheckWitnessSourceV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeMarginSchemaV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub margin_kind: &'static str,
    pub axis_order: Vec<&'static str>,
    pub calibration_stage: &'static str,
    pub public_safety_claim_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeMarginV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub margin_kind: &'static str,
    pub winner_declaration_id: String,
    pub challenger_declaration_id: Option<String>,
    pub dominant_axis: &'static str,
    pub signed_distance: i64,
    pub winner_key: CascadeKey,
    pub challenger_key: Option<CascadeKey>,
    pub calibration_stage: &'static str,
    pub public_safety_claim_ready: bool,
}

pub type CustomPropertyEnv = BTreeMap<String, CascadeValue>;
