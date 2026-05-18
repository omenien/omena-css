//! Cascade-formal substrate for the Omena CSS track.
//!
//! The crate starts with the load-bearing algebra from the research plan:
//! lexicographic cascade keys, specificity, provenance proofs, and a finite
//! custom-property substitution function with explicit cycle handling.

use serde::Serialize;
use std::{
    cmp::{Ordering, Reverse},
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
    fn unsupported(selector: &str) -> Self {
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

pub type CustomPropertyEnv = BTreeMap<String, CascadeValue>;

pub fn summarize_cascade_boundary() -> CascadeBoundarySummary {
    CascadeBoundarySummary {
        product: "omena-cascade.boundary",
        ordering_model: "lexicographicCascadeKey",
        substitution_model: "finiteCustomPropertyLeastFixedPoint",
        least_fixed_point_proof_model: "finite-env monotone custom-property substitution with cycle-to-guaranteed-invalid bottoming and env-size iteration bound",
        ready_surfaces: vec![
            "cascadeKeyOrdering",
            "specificityOrdering",
            "cascadeOutcomeProof",
            "genericCascadeWinner",
            "semanticDesignTokenRanking",
            "queryReadCascadeAtPosition",
            "selectorContextWitness",
            "selectorMatchWitness",
            "cascadeConformanceSeedCorpus",
            "customPropertySubstitution",
            "customPropertyLeastFixedPoint",
            "customPropertyLeastFixedPointProof",
            "customPropertyLeastFixedPointTrace",
            "cycleToGuaranteedInvalid",
            "computedValueResolutionSeed",
            "inheritanceInitialValueSeed",
            "shorthandCombinationProof",
            "supportsStaticEvalWitness",
            "scopeFlattenProof",
            "layerFlattenProof",
            "wptCascadeSeedCorpus",
        ],
        not_ready_surfaces: vec!["fullInitialValueTable", "fullWptCascadeCorpus"],
    }
}

pub fn run_cascade_conformance_seed_corpus() -> CascadeConformanceSeedReport {
    let results = cascade_conformance_seed_cases()
        .into_iter()
        .map(run_cascade_conformance_seed_case)
        .collect::<Vec<_>>();
    let passed_count = results.iter().filter(|result| result.passed).count();
    let case_count = results.len();

    CascadeConformanceSeedReport {
        schema_version: "0",
        product: "omena-cascade.conformance-seed-corpus",
        case_count,
        passed_count,
        failed_count: case_count.saturating_sub(passed_count),
        results,
    }
}

pub fn run_wpt_cascade_seed_corpus() -> CascadeConformanceSeedReport {
    let results = wpt_cascade_seed_cases()
        .into_iter()
        .map(run_cascade_conformance_seed_case)
        .collect::<Vec<_>>();
    let passed_count = results.iter().filter(|result| result.passed).count();
    let case_count = results.len();

    CascadeConformanceSeedReport {
        schema_version: "0",
        product: "omena-cascade.wpt-cascade-seed-corpus",
        case_count,
        passed_count,
        failed_count: case_count.saturating_sub(passed_count),
        results,
    }
}

fn run_cascade_conformance_seed_case(
    case: CascadeConformanceSeedCase,
) -> CascadeConformanceSeedResult {
    let outcome = cascade_property(case.declarations, case.property);
    let (actual_outcome, actual_winner_id) = match outcome {
        CascadeOutcome::Definite { winner, .. } => ("definite", Some(winner.id)),
        CascadeOutcome::RankedSet(_) => ("rankedSet", None),
        CascadeOutcome::Inherit => ("inherit", None),
        CascadeOutcome::Top => ("top", None),
    };
    let passed =
        actual_outcome == case.expected_outcome && actual_winner_id == case.expected_winner_id;

    CascadeConformanceSeedResult {
        name: case.name,
        passed,
        expected_outcome: case.expected_outcome,
        actual_outcome,
        expected_winner_id: case.expected_winner_id,
        actual_winner_id,
    }
}

fn cascade_conformance_seed_cases() -> Vec<CascadeConformanceSeedCase> {
    vec![
        CascadeConformanceSeedCase {
            name: "source-order-breaks-identical-key".to_string(),
            property: "color",
            declarations: vec![
                conformance_decl(
                    "source-earlier",
                    "color",
                    "red",
                    conformance_key(
                        CascadeLevel::AuthorNormal,
                        0,
                        0,
                        Specificity::new(0, 1, 0),
                        1,
                    ),
                ),
                conformance_decl(
                    "source-later",
                    "color",
                    "blue",
                    conformance_key(
                        CascadeLevel::AuthorNormal,
                        0,
                        0,
                        Specificity::new(0, 1, 0),
                        2,
                    ),
                ),
            ],
            expected_outcome: "definite",
            expected_winner_id: Some("source-later".to_string()),
        },
        CascadeConformanceSeedCase {
            name: "specificity-beats-source-order".to_string(),
            property: "color",
            declarations: vec![
                conformance_decl(
                    "specificity-low-later",
                    "color",
                    "red",
                    conformance_key(
                        CascadeLevel::AuthorNormal,
                        0,
                        0,
                        Specificity::new(0, 1, 0),
                        2,
                    ),
                ),
                conformance_decl(
                    "specificity-high-earlier",
                    "color",
                    "blue",
                    conformance_key(
                        CascadeLevel::AuthorNormal,
                        0,
                        0,
                        Specificity::new(1, 0, 0),
                        1,
                    ),
                ),
            ],
            expected_outcome: "definite",
            expected_winner_id: Some("specificity-high-earlier".to_string()),
        },
        CascadeConformanceSeedCase {
            name: "important-origin-beats-inline-normal".to_string(),
            property: "color",
            declarations: vec![
                conformance_decl(
                    "inline-normal",
                    "color",
                    "red",
                    conformance_key(
                        CascadeLevel::InlineNormal,
                        0,
                        0,
                        Specificity::new(1, 0, 0),
                        2,
                    ),
                ),
                conformance_decl(
                    "author-important",
                    "color",
                    "blue",
                    conformance_key(
                        CascadeLevel::AuthorImportant,
                        0,
                        0,
                        Specificity::new(0, 1, 0),
                        1,
                    ),
                ),
            ],
            expected_outcome: "definite",
            expected_winner_id: Some("author-important".to_string()),
        },
        CascadeConformanceSeedCase {
            name: "layer-rank-beats-specificity-within-level".to_string(),
            property: "color",
            declarations: vec![
                conformance_decl(
                    "lower-layer-specific",
                    "color",
                    "red",
                    conformance_key(
                        CascadeLevel::AuthorNormal,
                        1,
                        0,
                        Specificity::new(1, 0, 0),
                        2,
                    ),
                ),
                conformance_decl(
                    "higher-layer",
                    "color",
                    "blue",
                    conformance_key(
                        CascadeLevel::AuthorNormal,
                        2,
                        0,
                        Specificity::new(0, 1, 0),
                        1,
                    ),
                ),
            ],
            expected_outcome: "definite",
            expected_winner_id: Some("higher-layer".to_string()),
        },
        CascadeConformanceSeedCase {
            name: "scope-proximity-beats-specificity-tie".to_string(),
            property: "color",
            declarations: vec![
                conformance_decl(
                    "far-scope",
                    "color",
                    "red",
                    conformance_key(
                        CascadeLevel::AuthorNormal,
                        0,
                        5,
                        Specificity::new(0, 1, 0),
                        2,
                    ),
                ),
                conformance_decl(
                    "near-scope",
                    "color",
                    "blue",
                    conformance_key(
                        CascadeLevel::AuthorNormal,
                        0,
                        1,
                        Specificity::new(0, 1, 0),
                        1,
                    ),
                ),
            ],
            expected_outcome: "definite",
            expected_winner_id: Some("near-scope".to_string()),
        },
        CascadeConformanceSeedCase {
            name: "missing-property-inherits".to_string(),
            property: "background",
            declarations: vec![conformance_decl(
                "color-only",
                "color",
                "red",
                conformance_key(
                    CascadeLevel::AuthorNormal,
                    0,
                    0,
                    Specificity::new(0, 1, 0),
                    1,
                ),
            )],
            expected_outcome: "inherit",
            expected_winner_id: None,
        },
    ]
}

fn wpt_cascade_seed_cases() -> Vec<CascadeConformanceSeedCase> {
    let levels = [
        CascadeLevel::UserAgentNormal,
        CascadeLevel::UserNormal,
        CascadeLevel::AuthorNormal,
        CascadeLevel::InlineNormal,
        CascadeLevel::Animation,
        CascadeLevel::AuthorImportant,
        CascadeLevel::UserImportant,
        CascadeLevel::UserAgentImportant,
        CascadeLevel::Transition,
    ];
    let specificities = [
        Specificity::new(0, 0, 1),
        Specificity::new(0, 1, 0),
        Specificity::new(1, 0, 0),
    ];

    let mut cases = Vec::new();

    for left in levels {
        for right in levels {
            if left == right {
                continue;
            }

            let winner = if left > right { "left" } else { "right" };
            cases.push(CascadeConformanceSeedCase {
                name: format!("wpt-origin-importance-order-{left:?}-vs-{right:?}"),
                property: "color",
                declarations: vec![
                    conformance_decl(
                        "left",
                        "color",
                        "red",
                        conformance_key(left, 0, 0, Specificity::new(0, 1, 0), 1),
                    ),
                    conformance_decl(
                        "right",
                        "color",
                        "blue",
                        conformance_key(right, 0, 0, Specificity::new(0, 1, 0), 2),
                    ),
                ],
                expected_outcome: "definite",
                expected_winner_id: Some(winner.to_string()),
            });
        }
    }

    for layer_left in -3..=3 {
        for layer_right in -3..=3 {
            if layer_left == layer_right {
                continue;
            }

            let winner = if layer_left > layer_right {
                "left"
            } else {
                "right"
            };
            cases.push(CascadeConformanceSeedCase {
                name: format!("wpt-layer-order-{layer_left}-vs-{layer_right}"),
                property: "color",
                declarations: vec![
                    conformance_decl(
                        "left",
                        "color",
                        "red",
                        conformance_key(
                            CascadeLevel::AuthorNormal,
                            layer_left,
                            0,
                            Specificity::new(0, 1, 0),
                            2,
                        ),
                    ),
                    conformance_decl(
                        "right",
                        "color",
                        "blue",
                        conformance_key(
                            CascadeLevel::AuthorNormal,
                            layer_right,
                            0,
                            Specificity::new(1, 0, 0),
                            1,
                        ),
                    ),
                ],
                expected_outcome: "definite",
                expected_winner_id: Some(winner.to_string()),
            });
        }
    }

    for scope_left in 0..=7 {
        for scope_right in 0..=7 {
            if scope_left == scope_right {
                continue;
            }

            let winner = if scope_left < scope_right {
                "left"
            } else {
                "right"
            };
            cases.push(CascadeConformanceSeedCase {
                name: format!("wpt-scope-proximity-{scope_left}-vs-{scope_right}"),
                property: "color",
                declarations: vec![
                    conformance_decl(
                        "left",
                        "color",
                        "red",
                        conformance_key(
                            CascadeLevel::AuthorNormal,
                            0,
                            scope_left,
                            Specificity::new(0, 1, 0),
                            2,
                        ),
                    ),
                    conformance_decl(
                        "right",
                        "color",
                        "blue",
                        conformance_key(
                            CascadeLevel::AuthorNormal,
                            0,
                            scope_right,
                            Specificity::new(0, 1, 0),
                            1,
                        ),
                    ),
                ],
                expected_outcome: "definite",
                expected_winner_id: Some(winner.to_string()),
            });
        }
    }

    for left in specificities {
        for right in specificities {
            if left == right {
                continue;
            }

            let winner = if left > right { "left" } else { "right" };
            cases.push(CascadeConformanceSeedCase {
                name: format!("wpt-specificity-order-{left:?}-vs-{right:?}"),
                property: "color",
                declarations: vec![
                    conformance_decl(
                        "left",
                        "color",
                        "red",
                        conformance_key(CascadeLevel::AuthorNormal, 0, 0, left, 1),
                    ),
                    conformance_decl(
                        "right",
                        "color",
                        "blue",
                        conformance_key(CascadeLevel::AuthorNormal, 0, 0, right, 2),
                    ),
                ],
                expected_outcome: "definite",
                expected_winner_id: Some(winner.to_string()),
            });
        }
    }

    for source_left in 0..=15 {
        for source_right in 0..=15 {
            if source_left == source_right {
                continue;
            }

            let winner = if source_left > source_right {
                "left"
            } else {
                "right"
            };
            cases.push(CascadeConformanceSeedCase {
                name: format!("wpt-source-order-{source_left}-vs-{source_right}"),
                property: "color",
                declarations: vec![
                    conformance_decl(
                        "left",
                        "color",
                        "red",
                        conformance_key(
                            CascadeLevel::AuthorNormal,
                            0,
                            0,
                            Specificity::new(0, 1, 0),
                            source_left,
                        ),
                    ),
                    conformance_decl(
                        "right",
                        "color",
                        "blue",
                        conformance_key(
                            CascadeLevel::AuthorNormal,
                            0,
                            0,
                            Specificity::new(0, 1, 0),
                            source_right,
                        ),
                    ),
                ],
                expected_outcome: "definite",
                expected_winner_id: Some(winner.to_string()),
            });
        }
    }

    cases
}

fn conformance_key(
    level: CascadeLevel,
    layer_rank: i32,
    scope_proximity: u32,
    specificity: Specificity,
    source_order: u32,
) -> CascadeKey {
    CascadeKey::new(
        level,
        LayerRank(layer_rank),
        scope_proximity,
        specificity,
        source_order,
    )
}

fn conformance_decl(id: &str, property: &str, value: &str, key: CascadeKey) -> CascadeDeclaration {
    CascadeDeclaration {
        id: id.to_string(),
        property: property.to_string(),
        value: CascadeValue::Literal(value.to_string()),
        key,
    }
}

pub fn cascade_property(
    declarations: impl IntoIterator<Item = CascadeDeclaration>,
    property: &str,
) -> CascadeOutcome {
    let mut matching: Vec<CascadeDeclaration> = declarations
        .into_iter()
        .filter(|declaration| declaration.property == property)
        .collect();

    if matching.is_empty() {
        return CascadeOutcome::Inherit;
    }

    matching.sort_by_key(|declaration| std::cmp::Reverse(declaration.key));
    let winner = matching.remove(0);
    let proof = CascadeProof::from_declaration(&winner);
    CascadeOutcome::Definite {
        winner,
        proof,
        also_considered: matching,
    }
}

pub fn compute_cascade_computed_value(
    input: CascadeComputedValueInputV0,
) -> CascadeComputedValueResultV0 {
    let property = input.property.clone();
    let outcome = cascade_property(input.declarations, &property);
    let (winner_declaration_id, cascaded_value, mut derivation_steps) = match outcome {
        CascadeOutcome::Definite { winner, .. } => (
            Some(winner.id),
            winner.value,
            vec!["cascadeWinnerSelected", "computedValueResolutionStarted"],
        ),
        CascadeOutcome::Inherit => (
            None,
            if property_is_inherited(&property) {
                CascadeValue::Inherit
            } else {
                CascadeValue::Initial
            },
            vec!["noCascadeWinner", "inheritanceOrInitialSelected"],
        ),
        CascadeOutcome::RankedSet(_) | CascadeOutcome::Top => {
            return CascadeComputedValueResultV0 {
                schema_version: "0",
                product: "omena-cascade.computed-value",
                property,
                status: ComputedCascadeValueStatusV0::InvalidAtComputedValueTime,
                value: CascadeValue::GuaranteedInvalid,
                winner_declaration_id: None,
                inherited: false,
                used_initial_value: false,
                invalid_at_computed_value_time: true,
                derivation_steps: vec!["cascadeOutcomeIndeterminate"],
            };
        }
    };

    let substituted_value =
        substitute_custom_properties(&cascaded_value, &input.custom_property_env);
    if substituted_value == CascadeValue::GuaranteedInvalid {
        derivation_steps.push("substitutionProducedGuaranteedInvalid");
        derivation_steps.push("invalidAtComputedValueTimeFallsBackAsUnset");
        return computed_value_from_unset(
            property,
            winner_declaration_id,
            input.parent_computed_value,
            true,
            derivation_steps,
        );
    }

    match substituted_value {
        CascadeValue::Unset => computed_value_from_unset(
            property,
            winner_declaration_id,
            input.parent_computed_value,
            false,
            {
                derivation_steps.push("unsetKeywordResolved");
                derivation_steps
            },
        ),
        CascadeValue::Inherit => computed_value_from_inherit(
            property,
            winner_declaration_id,
            input.parent_computed_value,
            {
                derivation_steps.push("inheritKeywordResolved");
                derivation_steps
            },
        ),
        CascadeValue::Initial => computed_value_from_initial(property, winner_declaration_id, {
            derivation_steps.push("initialKeywordResolved");
            derivation_steps
        }),
        value => {
            derivation_steps.push("computedValueResolved");
            CascadeComputedValueResultV0 {
                schema_version: "0",
                product: "omena-cascade.computed-value",
                property,
                status: ComputedCascadeValueStatusV0::Resolved,
                value,
                winner_declaration_id,
                inherited: false,
                used_initial_value: false,
                invalid_at_computed_value_time: false,
                derivation_steps,
            }
        }
    }
}

fn computed_value_from_unset(
    property: String,
    winner_declaration_id: Option<String>,
    parent_computed_value: Option<CascadeValue>,
    invalid_at_computed_value_time: bool,
    mut derivation_steps: Vec<&'static str>,
) -> CascadeComputedValueResultV0 {
    if property_is_inherited(&property) {
        derivation_steps.push("unsetForInheritedPropertyUsesInheritance");
        return computed_value_from_inherit(
            property,
            winner_declaration_id,
            parent_computed_value,
            derivation_steps,
        )
        .with_invalid_at_computed_value_time(invalid_at_computed_value_time);
    }

    derivation_steps.push("unsetForNonInheritedPropertyUsesInitial");
    computed_value_from_initial(property, winner_declaration_id, derivation_steps)
        .with_invalid_at_computed_value_time(invalid_at_computed_value_time)
}

fn computed_value_from_inherit(
    property: String,
    winner_declaration_id: Option<String>,
    parent_computed_value: Option<CascadeValue>,
    mut derivation_steps: Vec<&'static str>,
) -> CascadeComputedValueResultV0 {
    match parent_computed_value {
        Some(value) => {
            derivation_steps.push("parentComputedValueUsed");
            CascadeComputedValueResultV0 {
                schema_version: "0",
                product: "omena-cascade.computed-value",
                property,
                status: ComputedCascadeValueStatusV0::Inherited,
                value,
                winner_declaration_id,
                inherited: true,
                used_initial_value: false,
                invalid_at_computed_value_time: false,
                derivation_steps,
            }
        }
        None => {
            derivation_steps.push("missingParentFallsBackToInitial");
            computed_value_from_initial(property, winner_declaration_id, derivation_steps)
        }
    }
}

fn computed_value_from_initial(
    property: String,
    winner_declaration_id: Option<String>,
    mut derivation_steps: Vec<&'static str>,
) -> CascadeComputedValueResultV0 {
    derivation_steps.push("initialValueTableConsulted");
    CascadeComputedValueResultV0 {
        schema_version: "0",
        product: "omena-cascade.computed-value",
        value: initial_cascade_value_for_property(&property),
        property,
        status: ComputedCascadeValueStatusV0::Initial,
        winner_declaration_id,
        inherited: false,
        used_initial_value: true,
        invalid_at_computed_value_time: false,
        derivation_steps,
    }
}

impl CascadeComputedValueResultV0 {
    fn with_invalid_at_computed_value_time(mut self, invalid_at_computed_value_time: bool) -> Self {
        if invalid_at_computed_value_time {
            self.status = ComputedCascadeValueStatusV0::InvalidAtComputedValueTime;
            self.invalid_at_computed_value_time = true;
        }
        self
    }
}

fn property_is_inherited(property: &str) -> bool {
    property.starts_with("--")
        || matches!(
            property,
            "color"
                | "cursor"
                | "direction"
                | "font"
                | "font-family"
                | "font-size"
                | "font-style"
                | "font-variant"
                | "font-weight"
                | "letter-spacing"
                | "line-height"
                | "text-align"
                | "text-indent"
                | "text-transform"
                | "visibility"
                | "white-space"
                | "word-spacing"
        )
}

fn initial_cascade_value_for_property(property: &str) -> CascadeValue {
    if property.starts_with("--") {
        return CascadeValue::GuaranteedInvalid;
    }

    let value = match property {
        "background-color" | "border-color" | "caret-color" | "outline-color" => "transparent",
        "border-style" | "display" => "none",
        "border-width" | "margin" | "padding" => "0",
        "box-shadow" | "text-shadow" => "none",
        "color" => "canvastext",
        "cursor" => "auto",
        "font-family" => "serif",
        "font-size" => "medium",
        "font-style" | "font-variant" | "font-weight" => "normal",
        "letter-spacing" | "line-height" | "word-spacing" => "normal",
        "opacity" => "1",
        "text-align" => "start",
        "text-indent" => "0",
        "text-transform" => "none",
        "visibility" => "visible",
        "white-space" => "normal",
        _ => "initial",
    };
    CascadeValue::Literal(value.to_string())
}

pub fn run_cascade_evaluation_fuzz_case(
    case: CascadeEvaluationFuzzCaseV0,
) -> CascadeEvaluationFuzzResultV0 {
    let declaration_count = case.declaration_count.clamp(1, 64);
    let declarations = generated_cascade_fuzz_declarations(case.seed, declaration_count);
    let matching = declarations
        .iter()
        .filter(|declaration| declaration.property == "color")
        .cloned()
        .collect::<Vec<_>>();
    let expected_winner_id = rank_cascade_items(matching.clone(), |declaration| declaration.key)
        .first()
        .map(|declaration| declaration.id.clone());
    let actual = cascade_property(declarations, "color");
    let actual_winner_id = match actual {
        CascadeOutcome::Definite { winner, .. } => Some(winner.id),
        _ => None,
    };
    let ranked_count = matching.len();
    let passed = actual_winner_id == expected_winner_id && ranked_count > 0;

    CascadeEvaluationFuzzResultV0 {
        seed: case.seed,
        declaration_count,
        actual_winner_id,
        expected_winner_id,
        ranked_count,
        passed,
    }
}

pub fn run_var_substitution_fuzz_case(
    case: VarSubstitutionFuzzCaseV0,
) -> VarSubstitutionFuzzResultV0 {
    let chain_len = case.chain_len.clamp(1, 32);
    let mut env = CustomPropertyEnv::new();
    let terminal = CascadeValue::Literal(format!("seed-{}", case.seed));

    for index in 0..chain_len {
        let name = fuzz_var_name(index);
        let next_value = if index == 0 && !case.cycle {
            terminal.clone()
        } else if index == 0 {
            CascadeValue::Var {
                name: fuzz_var_name(chain_len - 1),
                fallback: Some(Box::new(CascadeValue::Literal(
                    "cycle-fallback".to_string(),
                ))),
            }
        } else {
            CascadeValue::Var {
                name: fuzz_var_name(index - 1),
                fallback: None,
            }
        };
        env.insert(name, next_value);
    }

    let input = CascadeValue::Var {
        name: fuzz_var_name(chain_len - 1),
        fallback: Some(Box::new(CascadeValue::Literal(
            "outer-fallback".to_string(),
        ))),
    };
    let result = substitute_custom_properties(&input, &env);
    let expected = if case.cycle {
        CascadeValue::Literal("outer-fallback".to_string())
    } else {
        terminal
    };
    let passed = result == expected;

    VarSubstitutionFuzzResultV0 {
        seed: case.seed,
        chain_len,
        cycle: case.cycle,
        result,
        expected,
        passed,
    }
}

pub fn run_cascade_fuzz_seed_corpus() -> CascadeFuzzSeedReportV0 {
    let seeds = [0, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144];
    let cascade_results = seeds
        .into_iter()
        .enumerate()
        .map(|(index, seed)| {
            run_cascade_evaluation_fuzz_case(CascadeEvaluationFuzzCaseV0 {
                seed,
                declaration_count: index + 1,
            })
        })
        .collect::<Vec<_>>();
    let var_results = seeds
        .into_iter()
        .enumerate()
        .map(|(index, seed)| {
            run_var_substitution_fuzz_case(VarSubstitutionFuzzCaseV0 {
                seed,
                chain_len: index + 1,
                cycle: index % 3 == 0,
            })
        })
        .collect::<Vec<_>>();
    let passed_count = cascade_results
        .iter()
        .filter(|result| result.passed)
        .count()
        + var_results.iter().filter(|result| result.passed).count();
    let case_count = cascade_results.len() + var_results.len();

    CascadeFuzzSeedReportV0 {
        schema_version: "0",
        product: "omena-cascade.fuzz-seed-corpus",
        case_count,
        passed_count,
        failed_count: case_count - passed_count,
        cascade_results,
        var_results,
    }
}

pub fn rank_cascade_items<T>(
    items: impl IntoIterator<Item = T>,
    key_for: impl Fn(&T) -> CascadeKey,
) -> Vec<T> {
    let mut ranked = items.into_iter().collect::<Vec<_>>();
    ranked.sort_by_key(|item| Reverse(key_for(item)));
    ranked
}

pub fn select_cascade_winner<T>(
    items: impl IntoIterator<Item = T>,
    key_for: impl Fn(&T) -> CascadeKey,
) -> Option<(T, Vec<T>)> {
    let mut ranked = rank_cascade_items(items, key_for);
    if ranked.is_empty() {
        return None;
    }

    let winner = ranked.remove(0);
    Some((winner, ranked))
}

pub fn prove_box_shorthand_combination(
    shorthand_property: &str,
    longhands: &[BoxLonghandInputV0],
) -> ShorthandCombinationProofV0 {
    let expected = match box_shorthand_longhands(shorthand_property) {
        Some(expected) => expected,
        None => {
            return shorthand_combination_proof(
                shorthand_property,
                false,
                Some("unsupported shorthand property"),
                longhands,
                "",
            );
        }
    };

    if longhands.len() != expected.len() {
        return shorthand_combination_proof(
            shorthand_property,
            false,
            Some("incomplete longhand quartet"),
            longhands,
            "",
        );
    }

    if longhands
        .iter()
        .zip(expected.iter())
        .any(|(actual, expected)| actual.property != *expected)
    {
        return shorthand_combination_proof(
            shorthand_property,
            false,
            Some("longhands are not in canonical top/right/bottom/left order"),
            longhands,
            "",
        );
    }

    if longhands.iter().any(|longhand| longhand.important) {
        return shorthand_combination_proof(
            shorthand_property,
            false,
            Some("important longhands require explicit cascade equivalence proof"),
            longhands,
            "",
        );
    }

    if longhands.iter().any(|longhand| longhand.value.is_empty()) {
        return shorthand_combination_proof(
            shorthand_property,
            false,
            Some("empty longhand value"),
            longhands,
            "",
        );
    }

    if longhands
        .windows(2)
        .any(|pair| pair[1].source_order != pair[0].source_order + 1)
    {
        return shorthand_combination_proof(
            shorthand_property,
            false,
            Some("intervening declaration may change cascade outcome"),
            longhands,
            "",
        );
    }

    shorthand_combination_proof(
        shorthand_property,
        true,
        None,
        longhands,
        "all four longhands are adjacent, non-important, and in canonical order",
    )
}

pub fn evaluate_static_supports_condition(
    condition: &str,
    assumption: StaticSupportsAssumptionV0,
) -> StaticSupportsEvalWitnessV0 {
    let normalized_condition = normalize_ascii_whitespace(condition);
    let (verdict, reason) = match assumption {
        StaticSupportsAssumptionV0::ModernBrowser => {
            evaluate_modern_static_supports_condition(&normalized_condition)
        }
    };

    StaticSupportsEvalWitnessV0 {
        schema_version: "0",
        product: "omena-cascade.supports-static-eval",
        condition: normalized_condition,
        assumption,
        verdict,
        reason,
        provenance_preserved: verdict != StaticSupportsEvalVerdictV0::Unknown,
    }
}

fn evaluate_modern_static_supports_condition(
    condition: &str,
) -> (StaticSupportsEvalVerdictV0, &'static str) {
    if let Some(inner) = strip_supports_grouping_parens(condition) {
        return evaluate_modern_static_supports_condition(inner);
    }

    if let Some(parts) = parse_static_supports_logical_parts(condition, "or") {
        let verdicts = parts
            .iter()
            .map(|part| evaluate_modern_static_supports_condition(part).0)
            .collect::<Vec<_>>();
        if verdicts.contains(&StaticSupportsEvalVerdictV0::AlwaysTrue) {
            return (
                StaticSupportsEvalVerdictV0::AlwaysTrue,
                "modern-browser assumption accepts a true simple declaration inside disjunction",
            );
        }
        if verdicts
            .iter()
            .all(|verdict| *verdict == StaticSupportsEvalVerdictV0::AlwaysFalse)
        {
            return (
                StaticSupportsEvalVerdictV0::AlwaysFalse,
                "modern-browser assumption rejects all simple declarations inside disjunction",
            );
        }
        return (
            StaticSupportsEvalVerdictV0::Unknown,
            "unsupported supports disjunction member",
        );
    }

    if let Some(parts) = parse_static_supports_logical_parts(condition, "and") {
        let verdicts = parts
            .iter()
            .map(|part| evaluate_modern_static_supports_condition(part).0)
            .collect::<Vec<_>>();
        if verdicts.contains(&StaticSupportsEvalVerdictV0::AlwaysFalse) {
            return (
                StaticSupportsEvalVerdictV0::AlwaysFalse,
                "modern-browser assumption rejects a false simple declaration inside conjunction",
            );
        }
        if verdicts
            .iter()
            .all(|verdict| *verdict == StaticSupportsEvalVerdictV0::AlwaysTrue)
        {
            return (
                StaticSupportsEvalVerdictV0::AlwaysTrue,
                "modern-browser assumption accepts all simple declarations inside conjunction",
            );
        }
        return (
            StaticSupportsEvalVerdictV0::Unknown,
            "unsupported supports conjunction member",
        );
    }

    if let Some(inner) = parse_static_supports_not_condition(condition) {
        return match evaluate_modern_static_supports_condition(inner).0 {
            StaticSupportsEvalVerdictV0::AlwaysTrue => (
                StaticSupportsEvalVerdictV0::AlwaysFalse,
                "modern-browser assumption rejects negated supported condition queries",
            ),
            StaticSupportsEvalVerdictV0::AlwaysFalse => (
                StaticSupportsEvalVerdictV0::AlwaysTrue,
                "modern-browser assumption accepts negated unsupported condition queries",
            ),
            StaticSupportsEvalVerdictV0::Unknown => (
                StaticSupportsEvalVerdictV0::Unknown,
                "unsupported negated supports condition shape",
            ),
        };
    }

    if let Some(selector) = parse_supports_selector_condition(condition) {
        return match evaluate_modern_supports_selector_condition(selector) {
            StaticSupportsEvalVerdictV0::AlwaysTrue => (
                StaticSupportsEvalVerdictV0::AlwaysTrue,
                "modern-browser assumption accepts selector() feature queries",
            ),
            StaticSupportsEvalVerdictV0::AlwaysFalse => (
                StaticSupportsEvalVerdictV0::AlwaysFalse,
                "modern-browser assumption rejects known obsolete selector() feature queries",
            ),
            StaticSupportsEvalVerdictV0::Unknown => (
                StaticSupportsEvalVerdictV0::Unknown,
                "unsupported selector() feature query",
            ),
        };
    }

    if let Some((property, value)) = parse_simple_supports_declaration(condition) {
        return match evaluate_modern_simple_supports_declaration(property, value) {
            StaticSupportsEvalVerdictV0::AlwaysTrue => (
                StaticSupportsEvalVerdictV0::AlwaysTrue,
                "modern-browser assumption accepts simple declaration feature queries",
            ),
            StaticSupportsEvalVerdictV0::AlwaysFalse => (
                StaticSupportsEvalVerdictV0::AlwaysFalse,
                "modern-browser assumption rejects known obsolete declaration feature queries",
            ),
            StaticSupportsEvalVerdictV0::Unknown => (
                StaticSupportsEvalVerdictV0::Unknown,
                "unsupported simple declaration feature query",
            ),
        };
    }

    (
        StaticSupportsEvalVerdictV0::Unknown,
        "unsupported supports condition shape",
    )
}

pub fn prove_scope_flatten_candidate(input: ScopeFlattenInputV0) -> ScopeFlattenProofV0 {
    let blocked_reason = if input.limit_selector.is_some() {
        Some("scope limit selector cannot be encoded by the conservative flatten predicate")
    } else if input.root_selector.trim() != ":root" {
        Some("non-root scope flattening requires selector/proximity equivalence proof")
    } else if input.peer_scope_count > 0 {
        Some("peer scopes may change scope-proximity cascade ordering")
    } else if input.competing_unscoped_rule_count > 0 {
        Some("unscoped competitors may observe changed scope-proximity ordering")
    } else if input.inside_layer {
        Some("layer plus scope composition requires product cascade proof")
    } else {
        None
    };
    let accepted = blocked_reason.is_none();
    ScopeFlattenProofV0 {
        schema_version: "0",
        product: "omena-cascade.scope-flatten-proof",
        accepted,
        blocked_reason,
        root_selector: input.root_selector,
        provenance_preserved: accepted,
        cascade_safe_witness: if accepted {
            "root scope without limit, peer scopes, unscoped competition, or layer context"
        } else {
            "scope proximity cannot be erased by local syntax alone"
        }
        .to_string(),
    }
}

pub fn prove_layer_flatten_candidate(input: LayerFlattenInputV0) -> LayerFlattenProofV0 {
    let blocked_reason = if !input.closed_bundle {
        Some("layer flattening requires a closed bundle witness")
    } else if input.peer_layer_count > 0 {
        Some("peer layers may change layer-rank cascade ordering")
    } else if input.unlayered_rule_count > 0 {
        Some("unlayered rules compete differently from layered normal rules")
    } else if input.important_declaration_count > 0 {
        Some("important declarations invert layer ordering")
    } else {
        None
    };
    let accepted = blocked_reason.is_none();
    LayerFlattenProofV0 {
        schema_version: "0",
        product: "omena-cascade.layer-flatten-proof",
        accepted,
        blocked_reason,
        layer_name: input.layer_name,
        provenance_preserved: accepted,
        cascade_safe_witness: if accepted {
            "closed bundle with a single non-important layer and no unlayered competitors"
        } else {
            "layer rank cannot be erased by local syntax alone"
        }
        .to_string(),
    }
}

fn parse_static_supports_not_condition(condition: &str) -> Option<&str> {
    let inner = condition.strip_prefix("not ")?.trim();
    (!inner.is_empty()).then_some(inner)
}

fn parse_simple_supports_declaration(condition: &str) -> Option<(&str, &str)> {
    let inner = condition.strip_prefix('(')?.strip_suffix(')')?.trim();
    let (property, value) = inner.split_once(':')?;
    let property = property.trim();
    let value = value.trim();
    if property.is_empty()
        || value.is_empty()
        || property.contains(|ch: char| !is_supports_declaration_token_char(ch))
        || value.contains(['{', '}', ';'])
        || !supports_declaration_value_has_balanced_parentheses(value)
    {
        return None;
    }
    Some((property, value))
}

fn supports_declaration_value_has_balanced_parentheses(value: &str) -> bool {
    let mut depth = 0usize;
    for ch in value.chars() {
        match ch {
            '(' => depth += 1,
            ')' => {
                let Some(next_depth) = depth.checked_sub(1) else {
                    return false;
                };
                depth = next_depth;
            }
            _ => {}
        }
    }
    depth == 0
}

fn parse_supports_selector_condition(condition: &str) -> Option<&str> {
    let arguments = condition.strip_prefix("selector")?.trim_start();
    let inner = arguments.strip_prefix('(')?.strip_suffix(')')?.trim();
    (!inner.is_empty()
        && supports_outer_parens_wrap_entire_condition(arguments)
        && !inner.contains(['{', '}', ';']))
    .then_some(inner)
}

fn strip_supports_grouping_parens(condition: &str) -> Option<&str> {
    let inner = condition.strip_prefix('(')?.strip_suffix(')')?.trim();
    if parse_simple_supports_declaration(condition).is_some()
        || !supports_outer_parens_wrap_entire_condition(condition)
        || inner.is_empty()
    {
        return None;
    }
    Some(inner)
}

fn supports_outer_parens_wrap_entire_condition(condition: &str) -> bool {
    let mut depth = 0usize;
    for (index, ch) in condition.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 && index + ch.len_utf8() < condition.len() {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0
}

fn parse_static_supports_logical_parts<'a>(
    condition: &'a str,
    operator: &str,
) -> Option<Vec<&'a str>> {
    let delimiter = match operator {
        "and" => " and ",
        "or" => " or ",
        _ => return None,
    };
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut part_start = 0usize;
    let mut index = 0usize;

    while index < condition.len() {
        let ch = condition[index..].chars().next()?;
        match ch {
            '(' => {
                depth += 1;
                index += ch.len_utf8();
            }
            ')' => {
                depth = depth.saturating_sub(1);
                index += ch.len_utf8();
            }
            _ if depth == 0 && condition[index..].starts_with(delimiter) => {
                parts.push(condition[part_start..index].trim());
                index += delimiter.len();
                part_start = index;
            }
            _ => index += ch.len_utf8(),
        }
    }

    if parts.is_empty() {
        return None;
    }
    parts.push(condition[part_start..].trim());
    parts.iter().all(|part| !part.is_empty()).then_some(parts)
}

fn evaluate_modern_simple_supports_declaration(
    property: &str,
    value: &str,
) -> StaticSupportsEvalVerdictV0 {
    if property.starts_with("-ms-") || value.starts_with("-ms-") {
        StaticSupportsEvalVerdictV0::AlwaysFalse
    } else {
        StaticSupportsEvalVerdictV0::AlwaysTrue
    }
}

fn evaluate_modern_supports_selector_condition(selector: &str) -> StaticSupportsEvalVerdictV0 {
    if selector.contains("-ms-") {
        StaticSupportsEvalVerdictV0::AlwaysFalse
    } else {
        StaticSupportsEvalVerdictV0::AlwaysTrue
    }
}

fn is_supports_declaration_token_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_')
}

fn normalize_ascii_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn box_shorthand_longhands(shorthand_property: &str) -> Option<[&'static str; 4]> {
    match shorthand_property {
        "margin" => Some(["margin-top", "margin-right", "margin-bottom", "margin-left"]),
        "padding" => Some([
            "padding-top",
            "padding-right",
            "padding-bottom",
            "padding-left",
        ]),
        "border-color" => Some([
            "border-top-color",
            "border-right-color",
            "border-bottom-color",
            "border-left-color",
        ]),
        "border-style" => Some([
            "border-top-style",
            "border-right-style",
            "border-bottom-style",
            "border-left-style",
        ]),
        "border-width" => Some([
            "border-top-width",
            "border-right-width",
            "border-bottom-width",
            "border-left-width",
        ]),
        _ => None,
    }
}

fn shorthand_combination_proof(
    shorthand_property: &str,
    accepted: bool,
    blocked_reason: Option<&'static str>,
    longhands: &[BoxLonghandInputV0],
    witness: &str,
) -> ShorthandCombinationProofV0 {
    ShorthandCombinationProofV0 {
        schema_version: "0",
        product: "omena-cascade.shorthand-combination-proof",
        shorthand_property: shorthand_property.to_string(),
        accepted,
        blocked_reason,
        ordered_longhand_properties: longhands
            .iter()
            .map(|longhand| longhand.property.clone())
            .collect(),
        provenance_preserved: accepted,
        cascade_safe_witness: witness.to_string(),
    }
}

pub fn selector_context_witness(
    declaration_selectors: &[String],
    reference_selectors: &[String],
) -> SelectorContextWitness {
    if declaration_selectors.is_empty() {
        return SelectorContextWitness {
            kind: SelectorContextMatchKind::Global,
            matched: true,
            rank: 1,
            declaration_selector: None,
            reference_selector: None,
        };
    }

    let mut best = SelectorContextWitness::no_match();
    for declaration_selector in declaration_selectors {
        let candidate = selector_context_witness_for_declaration(
            declaration_selector.as_str(),
            reference_selectors,
        );
        if candidate.rank > best.rank {
            best = candidate;
        }
    }
    best
}

pub fn selector_context_witness_for_declaration(
    declaration_selector: &str,
    reference_selectors: &[String],
) -> SelectorContextWitness {
    if declaration_selector == ":root" {
        return SelectorContextWitness {
            kind: SelectorContextMatchKind::Root,
            matched: true,
            rank: 1,
            declaration_selector: Some(declaration_selector.to_string()),
            reference_selector: None,
        };
    }

    for reference_selector in reference_selectors {
        if reference_selector == declaration_selector {
            return SelectorContextWitness {
                kind: SelectorContextMatchKind::Exact,
                matched: true,
                rank: 2,
                declaration_selector: Some(declaration_selector.to_string()),
                reference_selector: Some(reference_selector.clone()),
            };
        }
    }

    for reference_selector in reference_selectors {
        if reference_selector.contains(declaration_selector) {
            return SelectorContextWitness {
                kind: SelectorContextMatchKind::ContainsSelector,
                matched: true,
                rank: 2,
                declaration_selector: Some(declaration_selector.to_string()),
                reference_selector: Some(reference_selector.clone()),
            };
        }
    }

    SelectorContextWitness {
        kind: SelectorContextMatchKind::NoMatch,
        matched: false,
        rank: 0,
        declaration_selector: Some(declaration_selector.to_string()),
        reference_selector: None,
    }
}

pub fn selector_match_witness(selector: &str, element: &ElementSignature) -> SelectorMatchWitness {
    let branches = split_selector_list(selector);
    if branches.is_empty() {
        return SelectorMatchWitness::unsupported(selector);
    }

    let mut witnesses = branches
        .iter()
        .map(|branch| selector_match_branch_witness(branch, element))
        .collect::<Vec<_>>();

    let yes = strongest_by_verdict(&witnesses, SelectorMatchVerdict::Yes);
    if let Some(index) = yes {
        let mut witness = witnesses.remove(index);
        witness.selector = selector.to_string();
        if branches.len() > 1 {
            witness.reason = SelectorMatchReason::SelectorList;
            witness.unsupported_branches = witnesses
                .into_iter()
                .flat_map(|witness| witness.unsupported_branches)
                .collect();
        }
        return witness;
    }

    let maybe = strongest_by_verdict(&witnesses, SelectorMatchVerdict::Maybe);
    if let Some(index) = maybe {
        let mut witness = witnesses.remove(index);
        witness.selector = selector.to_string();
        if branches.len() > 1 {
            witness.reason = SelectorMatchReason::SelectorList;
            witness.unsupported_branches = witnesses
                .into_iter()
                .flat_map(|witness| witness.unsupported_branches)
                .collect();
        }
        return witness;
    }

    let mut witness = witnesses
        .into_iter()
        .max_by(|left, right| left.specificity.cmp(&right.specificity))
        .unwrap_or_else(|| SelectorMatchWitness::unsupported(selector));
    witness.selector = selector.to_string();
    if branches.len() > 1 {
        witness.reason = SelectorMatchReason::SelectorList;
    }
    witness
}

pub fn parse_simple_selector_signature(selector: &str) -> Option<SelectorSignature> {
    parse_simple_selector_signature_inner(selector.trim())
}

fn selector_match_branch_witness(
    selector: &str,
    element: &ElementSignature,
) -> SelectorMatchWitness {
    let Some(signature) = parse_simple_selector_signature(selector) else {
        return SelectorMatchWitness::unsupported(selector);
    };

    let mut witness = SelectorMatchWitness {
        selector: selector.to_string(),
        matched_branch: Some(selector.to_string()),
        verdict: SelectorMatchVerdict::Yes,
        reason: if signature.required_tag.is_none()
            && signature.required_id.is_none()
            && signature.required_classes.is_empty()
            && signature.required_attributes.is_empty()
            && signature.required_pseudo_states.is_empty()
        {
            SelectorMatchReason::Universal
        } else {
            SelectorMatchReason::SimpleCompound
        },
        specificity: signature.specificity,
        missing_tag: None,
        missing_id: None,
        missing_classes: BTreeSet::new(),
        missing_attributes: BTreeSet::new(),
        missing_pseudo_states: BTreeSet::new(),
        unsupported_branches: Vec::new(),
    };

    if let Some(required_tag) = &signature.required_tag {
        match element.tag.as_deref() {
            Some(tag) if tag == required_tag => {}
            _ if !element.tag_is_exact => {
                witness.verdict = SelectorMatchVerdict::Maybe;
                witness.reason = SelectorMatchReason::MissingTag;
                witness.missing_tag = Some(required_tag.clone());
            }
            _ => {
                witness.verdict = SelectorMatchVerdict::No;
                witness.reason = SelectorMatchReason::MissingTag;
                witness.missing_tag = Some(required_tag.clone());
            }
        }
    }

    if let Some(required_id) = &signature.required_id {
        match element.id.as_deref() {
            Some(id) if id == required_id => {}
            _ if !element.id_is_exact && witness.verdict != SelectorMatchVerdict::No => {
                witness.verdict = SelectorMatchVerdict::Maybe;
                witness.reason = SelectorMatchReason::MissingId;
                witness.missing_id = Some(required_id.clone());
            }
            _ => {
                witness.verdict = SelectorMatchVerdict::No;
                witness.reason = SelectorMatchReason::MissingId;
                witness.missing_id = Some(required_id.clone());
            }
        }
    }

    for required_class in &signature.required_classes {
        if element.classes.contains(required_class) {
            continue;
        }
        if !element.classes_are_exact && witness.verdict != SelectorMatchVerdict::No {
            witness.verdict = SelectorMatchVerdict::Maybe;
        } else {
            witness.verdict = SelectorMatchVerdict::No;
        }
        witness.reason = SelectorMatchReason::MissingClass;
        witness.missing_classes.insert(required_class.clone());
    }

    for required_attribute in &signature.required_attributes {
        if element.attributes.contains(required_attribute) {
            continue;
        }
        if !element.attributes_are_exact && witness.verdict != SelectorMatchVerdict::No {
            witness.verdict = SelectorMatchVerdict::Maybe;
        } else {
            witness.verdict = SelectorMatchVerdict::No;
        }
        witness.reason = SelectorMatchReason::MissingAttribute;
        witness
            .missing_attributes
            .insert(required_attribute.clone());
    }

    for required_pseudo_state in &signature.required_pseudo_states {
        if element.pseudo_states.contains(required_pseudo_state) {
            continue;
        }
        if !element.pseudo_states_are_exact && witness.verdict != SelectorMatchVerdict::No {
            witness.verdict = SelectorMatchVerdict::Maybe;
        } else {
            witness.verdict = SelectorMatchVerdict::No;
        }
        witness.reason = SelectorMatchReason::MissingPseudoState;
        witness
            .missing_pseudo_states
            .insert(required_pseudo_state.clone());
    }

    witness
}

fn strongest_by_verdict(
    witnesses: &[SelectorMatchWitness],
    verdict: SelectorMatchVerdict,
) -> Option<usize> {
    witnesses
        .iter()
        .enumerate()
        .filter(|(_, witness)| witness.verdict == verdict)
        .max_by(|(_, left), (_, right)| left.specificity.cmp(&right.specificity))
        .map(|(index, _)| index)
}

fn parse_simple_selector_signature_inner(selector: &str) -> Option<SelectorSignature> {
    if selector.is_empty() || selector_has_unsupported_top_level_syntax(selector) {
        return None;
    }

    let mut required_tag = None;
    let mut required_id = None;
    let mut required_classes = BTreeSet::new();
    let mut required_attributes = BTreeSet::new();
    let mut required_pseudo_states = BTreeSet::new();
    let mut specificity = Specificity::ZERO;
    let chars = selector.chars().collect::<Vec<_>>();
    let mut index = 0;

    while index < chars.len() {
        match chars[index] {
            '*' => index += 1,
            '.' => {
                index += 1;
                let (name, next) = read_identifier(&chars, index)?;
                specificity.classes += 1;
                required_classes.insert(name);
                index = next;
            }
            '#' => {
                index += 1;
                let (name, next) = read_identifier(&chars, index)?;
                specificity.ids += 1;
                required_id = Some(name);
                index = next;
            }
            '[' => {
                let close = find_closing_bracket(&chars, index)?;
                let attribute = chars[index + 1..close].iter().collect::<String>();
                let attribute_name = read_attribute_name(attribute.trim())?;
                specificity.classes += 1;
                required_attributes.insert(attribute_name);
                index = close + 1;
            }
            ':' => {
                if matches!(chars.get(index + 1), Some(':')) {
                    index += 2;
                    let (_, next) = read_identifier(&chars, index)?;
                    specificity.elements += 1;
                    index = next;
                } else {
                    index += 1;
                    let (name, next) = read_identifier(&chars, index)?;
                    if matches!(chars.get(next), Some('(')) {
                        return None;
                    }
                    specificity.classes += 1;
                    required_pseudo_states.insert(name);
                    index = next;
                }
            }
            ch if is_identifier_start(ch) => {
                let (name, next) = read_identifier(&chars, index)?;
                if required_tag.is_some() {
                    return None;
                }
                specificity.elements += 1;
                required_tag = Some(name);
                index = next;
            }
            _ => return None,
        }
    }

    Some(SelectorSignature {
        selector: selector.to_string(),
        required_tag,
        required_id,
        required_classes,
        required_attributes,
        required_pseudo_states,
        specificity,
    })
}

fn split_selector_list(selector: &str) -> Vec<String> {
    let mut branches = Vec::new();
    let mut start = 0;
    let mut paren_depth: usize = 0;
    let mut bracket_depth: usize = 0;
    let chars = selector.char_indices().collect::<Vec<_>>();

    for (index, ch) in &chars {
        match *ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            ',' if paren_depth == 0 && bracket_depth == 0 => {
                let branch = selector[start..*index].trim();
                if !branch.is_empty() {
                    branches.push(branch.to_string());
                }
                start = *index + 1;
            }
            _ => {}
        }
    }

    let tail = selector[start..].trim();
    if !tail.is_empty() {
        branches.push(tail.to_string());
    }
    branches
}

fn selector_has_unsupported_top_level_syntax(selector: &str) -> bool {
    let mut paren_depth: usize = 0;
    let mut bracket_depth: usize = 0;
    for ch in selector.chars() {
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '>' | '+' | '~' if paren_depth == 0 && bracket_depth == 0 => return true,
            ch if ch.is_whitespace() && paren_depth == 0 && bracket_depth == 0 => return true,
            _ => {}
        }
    }
    false
}

fn find_closing_bracket(chars: &[char], open_index: usize) -> Option<usize> {
    chars
        .iter()
        .enumerate()
        .skip(open_index + 1)
        .find_map(|(index, ch)| if *ch == ']' { Some(index) } else { None })
}

fn read_attribute_name(attribute: &str) -> Option<String> {
    let name = attribute
        .split(|ch: char| ch.is_whitespace() || matches!(ch, '=' | '~' | '|' | '^' | '$' | '*'))
        .find(|part| !part.is_empty())?;
    Some(name.to_string())
}

fn read_identifier(chars: &[char], start: usize) -> Option<(String, usize)> {
    if start >= chars.len() || !is_identifier_start(chars[start]) {
        return None;
    }
    let mut end = start + 1;
    while end < chars.len() && is_identifier_continue(chars[end]) {
        end += 1;
    }
    Some((chars[start..end].iter().collect(), end))
}

fn is_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || matches!(ch, '_' | '-')
}

fn is_identifier_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')
}

pub fn substitute_custom_properties(value: &CascadeValue, env: &CustomPropertyEnv) -> CascadeValue {
    let mut visiting = BTreeSet::new();
    substitute_custom_properties_inner(value, env, &mut visiting)
}

pub fn resolve_custom_property_env_least_fixed_point(env: &CustomPropertyEnv) -> CustomPropertyEnv {
    compute_custom_property_env_least_fixed_point(env).resolved_env
}

pub fn summarize_custom_property_least_fixed_point(
    env: &CustomPropertyEnv,
) -> CustomPropertyLeastFixedPointSummaryV0 {
    let computation = compute_custom_property_env_least_fixed_point(env);
    let entries = env
        .iter()
        .map(|(name, input)| {
            let resolved = computation
                .resolved_env
                .get(name)
                .cloned()
                .unwrap_or(CascadeValue::GuaranteedInvalid);
            CustomPropertyLeastFixedPointEntryV0 {
                name: name.clone(),
                input: input.clone(),
                changed: &resolved != input,
                guaranteed_invalid: resolved == CascadeValue::GuaranteedInvalid,
                resolved,
            }
        })
        .collect::<Vec<_>>();
    let resolved_count = entries
        .iter()
        .filter(|entry| cascade_value_is_resolved(&entry.resolved))
        .count();
    let guaranteed_invalid_count = entries
        .iter()
        .filter(|entry| entry.guaranteed_invalid)
        .count();

    CustomPropertyLeastFixedPointSummaryV0 {
        schema_version: "0",
        product: "omena-cascade.custom-property-least-fixed-point",
        input_count: env.len(),
        resolved_count,
        guaranteed_invalid_count,
        iteration_count: computation.iteration_count,
        iteration_bound: computation.iteration_bound,
        reached_fixed_point: computation.reached_fixed_point,
        monotone_witness_valid: custom_property_iteration_trace_is_monotone(
            &computation.iteration_trace,
        ),
        proof: custom_property_least_fixed_point_proof(),
        iteration_trace: computation.iteration_trace,
        entries,
        ready_surfaces: vec![
            "customPropertySubstitution",
            "customPropertyLeastFixedPoint",
            "customPropertyLeastFixedPointProof",
            "customPropertyLeastFixedPointTrace",
            "cycleToGuaranteedInvalid",
        ],
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CustomPropertyLeastFixedPointComputation {
    resolved_env: CustomPropertyEnv,
    iteration_count: usize,
    iteration_bound: usize,
    reached_fixed_point: bool,
    iteration_trace: Vec<CustomPropertyLeastFixedPointIterationV0>,
}

fn compute_custom_property_env_least_fixed_point(
    env: &CustomPropertyEnv,
) -> CustomPropertyLeastFixedPointComputation {
    let mut current = env.clone();
    let max_iterations = env.len().saturating_add(1).max(1);
    let mut iteration_trace = Vec::new();

    for iteration in 1..=max_iterations {
        let next = env
            .iter()
            .map(|(name, value)| (name.clone(), substitute_custom_properties(value, &current)))
            .collect::<CustomPropertyEnv>();
        iteration_trace.push(custom_property_least_fixed_point_iteration_witness(
            iteration, env, &next,
        ));
        if next == current {
            return CustomPropertyLeastFixedPointComputation {
                resolved_env: next,
                iteration_count: iteration,
                iteration_bound: max_iterations,
                reached_fixed_point: true,
                iteration_trace,
            };
        }
        current = next;
    }

    CustomPropertyLeastFixedPointComputation {
        resolved_env: current,
        iteration_count: max_iterations,
        iteration_bound: max_iterations,
        reached_fixed_point: false,
        iteration_trace,
    }
}

fn custom_property_least_fixed_point_iteration_witness(
    iteration: usize,
    input_env: &CustomPropertyEnv,
    resolved_env: &CustomPropertyEnv,
) -> CustomPropertyLeastFixedPointIterationV0 {
    let changed_count = input_env
        .iter()
        .filter(|(name, input)| {
            resolved_env
                .get(*name)
                .is_some_and(|resolved| resolved != *input)
        })
        .count();
    let settled_count = resolved_env
        .values()
        .filter(|value| !cascade_value_contains_var_reference(value))
        .count();
    let guaranteed_invalid_count = resolved_env
        .values()
        .filter(|value| **value == CascadeValue::GuaranteedInvalid)
        .count();

    CustomPropertyLeastFixedPointIterationV0 {
        iteration,
        changed_count,
        settled_count,
        guaranteed_invalid_count,
    }
}

fn custom_property_iteration_trace_is_monotone(
    trace: &[CustomPropertyLeastFixedPointIterationV0],
) -> bool {
    trace
        .windows(2)
        .all(|pair| pair[0].settled_count <= pair[1].settled_count)
}

fn cascade_value_contains_var_reference(value: &CascadeValue) -> bool {
    match value {
        CascadeValue::Var { .. } => true,
        CascadeValue::Composite(values) => values.iter().any(cascade_value_contains_var_reference),
        CascadeValue::Literal(_)
        | CascadeValue::Initial
        | CascadeValue::Inherit
        | CascadeValue::GuaranteedInvalid
        | CascadeValue::Unset => false,
    }
}

fn custom_property_least_fixed_point_proof() -> CustomPropertyLeastFixedPointProofV0 {
    CustomPropertyLeastFixedPointProofV0 {
        finite_domain: "custom-property environment keys are fixed during iteration",
        transfer_function: "each step substitutes every original binding against the previous environment approximation",
        monotone_witness: "iteration trace records a nondecreasing settled-value count across the fixed-key environment",
        iteration_bound_formula: "max(1, env.len() + 1)",
        cycle_policy: "recursive var() cycles are detected by the visiting set and collapsed to guaranteed-invalid or fallback",
        proof_obligations: vec![
            "fixed-key environment",
            "deterministic simultaneous transfer",
            "nondecreasing settled-value trace",
            "cycle-to-guaranteed-invalid bottoming",
            "finite iteration bound",
            "explicit fixed-point equality check",
        ],
    }
}

fn substitute_custom_properties_inner(
    value: &CascadeValue,
    env: &CustomPropertyEnv,
    visiting: &mut BTreeSet<String>,
) -> CascadeValue {
    match value {
        CascadeValue::Literal(_)
        | CascadeValue::Initial
        | CascadeValue::Inherit
        | CascadeValue::GuaranteedInvalid
        | CascadeValue::Unset => value.clone(),
        CascadeValue::Composite(parts) => {
            let resolved_parts = parts
                .iter()
                .map(|part| substitute_custom_properties_inner(part, env, visiting))
                .collect::<Vec<_>>();
            if resolved_parts.contains(&CascadeValue::GuaranteedInvalid) {
                return CascadeValue::GuaranteedInvalid;
            }
            CascadeValue::Composite(resolved_parts)
        }
        CascadeValue::Var { name, fallback } => {
            if !visiting.insert(name.clone()) {
                return CascadeValue::GuaranteedInvalid;
            }
            let resolved = match env.get(name) {
                Some(CascadeValue::Unset) | None => fallback
                    .as_deref()
                    .map(|fallback| substitute_custom_properties_inner(fallback, env, visiting))
                    .unwrap_or(CascadeValue::GuaranteedInvalid),
                Some(value) => {
                    let resolved = substitute_custom_properties_inner(value, env, visiting);
                    if resolved == CascadeValue::GuaranteedInvalid {
                        fallback
                            .as_deref()
                            .map(|fallback| {
                                substitute_custom_properties_inner(fallback, env, visiting)
                            })
                            .unwrap_or(CascadeValue::GuaranteedInvalid)
                    } else {
                        resolved
                    }
                }
            };
            visiting.remove(name);
            resolved
        }
    }
}

fn cascade_value_is_resolved(value: &CascadeValue) -> bool {
    match value {
        CascadeValue::Literal(_) => true,
        CascadeValue::Composite(parts) => parts.iter().all(cascade_value_is_resolved),
        CascadeValue::Var { .. }
        | CascadeValue::Initial
        | CascadeValue::Inherit
        | CascadeValue::GuaranteedInvalid
        | CascadeValue::Unset => false,
    }
}

fn generated_cascade_fuzz_declarations(
    seed: u64,
    declaration_count: usize,
) -> Vec<CascadeDeclaration> {
    let mut state = seed ^ 0x9e37_79b9_7f4a_7c15;
    (0..declaration_count)
        .map(|index| {
            let property = if index == 0 || fuzz_next(&mut state).is_multiple_of(3) {
                "color"
            } else {
                "margin"
            };
            CascadeDeclaration {
                id: format!("decl-{seed}-{index}"),
                property: property.to_string(),
                value: CascadeValue::Literal(format!("v{}", fuzz_next(&mut state) % 17)),
                key: CascadeKey::new(
                    fuzz_cascade_level(fuzz_next(&mut state)),
                    LayerRank((fuzz_next(&mut state) % 9) as i32 - 4),
                    (fuzz_next(&mut state) % 12) as u32,
                    Specificity::new(
                        (fuzz_next(&mut state) % 4) as u32,
                        (fuzz_next(&mut state) % 8) as u32,
                        (fuzz_next(&mut state) % 12) as u32,
                    ),
                    index as u32,
                ),
            }
        })
        .collect()
}

fn fuzz_cascade_level(value: u64) -> CascadeLevel {
    match value % 9 {
        0 => CascadeLevel::UserAgentNormal,
        1 => CascadeLevel::UserNormal,
        2 => CascadeLevel::AuthorNormal,
        3 => CascadeLevel::InlineNormal,
        4 => CascadeLevel::Animation,
        5 => CascadeLevel::AuthorImportant,
        6 => CascadeLevel::UserImportant,
        7 => CascadeLevel::UserAgentImportant,
        _ => CascadeLevel::Transition,
    }
}

fn fuzz_var_name(index: usize) -> String {
    format!("--fuzz-{index}")
}

fn fuzz_next(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *state
}

#[cfg(test)]
mod tests {
    use super::*;

    fn declaration(id: &str, value: &str, key: CascadeKey) -> CascadeDeclaration {
        CascadeDeclaration {
            id: id.to_string(),
            property: "color".to_string(),
            value: CascadeValue::Literal(value.to_string()),
            key,
        }
    }

    fn property_declaration(
        id: &str,
        property: &str,
        value: CascadeValue,
        source_order: u32,
    ) -> CascadeDeclaration {
        CascadeDeclaration {
            id: id.to_string(),
            property: property.to_string(),
            value,
            key: key(
                CascadeLevel::AuthorNormal,
                0,
                1,
                Specificity::new(0, 1, 0),
                source_order,
            ),
        }
    }

    fn key(
        level: CascadeLevel,
        layer_rank: i32,
        scope_proximity: u32,
        specificity: Specificity,
        source_order: u32,
    ) -> CascadeKey {
        CascadeKey::new(
            level,
            LayerRank(layer_rank),
            scope_proximity,
            specificity,
            source_order,
        )
    }

    #[test]
    fn orders_specificity_lexicographically() {
        assert!(Specificity::new(1, 0, 0) > Specificity::new(0, 99, 99));
        assert!(Specificity::new(0, 2, 0) > Specificity::new(0, 1, 99));
        assert!(Specificity::new(0, 0, 2) > Specificity::new(0, 0, 1));
    }

    #[test]
    fn orders_cascade_keys_by_level_layer_scope_specificity_and_source() {
        let base = key(
            CascadeLevel::AuthorNormal,
            0,
            3,
            Specificity::new(0, 1, 0),
            1,
        );
        assert!(
            key(
                CascadeLevel::AuthorImportant,
                0,
                3,
                Specificity::new(0, 1, 0),
                1,
            ) > base
        );
        assert!(
            key(
                CascadeLevel::AuthorNormal,
                1,
                3,
                Specificity::new(0, 1, 0),
                1,
            ) > base
        );
        assert!(
            key(
                CascadeLevel::AuthorNormal,
                0,
                1,
                Specificity::new(0, 1, 0),
                1,
            ) > base
        );
        assert!(
            key(
                CascadeLevel::AuthorNormal,
                0,
                3,
                Specificity::new(0, 2, 0),
                1,
            ) > base
        );
        assert!(
            key(
                CascadeLevel::AuthorNormal,
                0,
                3,
                Specificity::new(0, 1, 0),
                2,
            ) > base
        );
    }

    #[test]
    fn selects_definite_winner_with_proof() {
        let earlier = declaration(
            "earlier",
            "red",
            key(
                CascadeLevel::AuthorNormal,
                0,
                1,
                Specificity::new(0, 1, 0),
                1,
            ),
        );
        let later = declaration(
            "later",
            "blue",
            key(
                CascadeLevel::AuthorNormal,
                0,
                1,
                Specificity::new(0, 1, 0),
                2,
            ),
        );

        let outcome = cascade_property([earlier, later], "color");

        assert!(matches!(outcome, CascadeOutcome::Definite { .. }));
        if let CascadeOutcome::Definite {
            winner,
            proof,
            also_considered,
        } = outcome
        {
            assert_eq!(winner.id, "later");
            assert_eq!(proof.declaration_id, "later");
            assert_eq!(also_considered.len(), 1);
        }
    }

    #[test]
    fn selects_generic_winner_with_same_cascade_ordering() {
        let ranked = select_cascade_winner(["earlier", "later"], |item| match *item {
            "earlier" => key(
                CascadeLevel::AuthorNormal,
                0,
                1,
                Specificity::new(0, 1, 0),
                1,
            ),
            _ => key(
                CascadeLevel::AuthorNormal,
                0,
                1,
                Specificity::new(0, 1, 0),
                2,
            ),
        });

        let Some((winner, also_considered)) = ranked else {
            unreachable!("test input contains candidates")
        };
        assert_eq!(winner, "later");
        assert_eq!(also_considered, vec!["earlier"]);
    }

    #[test]
    fn computes_values_through_var_substitution() {
        let mut env = CustomPropertyEnv::new();
        env.insert(
            "--brand".to_string(),
            CascadeValue::Literal("red".to_string()),
        );

        let result = compute_cascade_computed_value(CascadeComputedValueInputV0 {
            property: "color".to_string(),
            declarations: vec![property_declaration(
                "color-decl",
                "color",
                CascadeValue::Var {
                    name: "--brand".to_string(),
                    fallback: None,
                },
                1,
            )],
            custom_property_env: env,
            parent_computed_value: Some(CascadeValue::Literal("blue".to_string())),
        });

        assert_eq!(result.product, "omena-cascade.computed-value");
        assert_eq!(result.status, ComputedCascadeValueStatusV0::Resolved);
        assert_eq!(result.value, CascadeValue::Literal("red".to_string()));
        assert_eq!(result.winner_declaration_id.as_deref(), Some("color-decl"));
        assert!(!result.inherited);
        assert!(!result.used_initial_value);
        assert!(!result.invalid_at_computed_value_time);
        assert!(result.derivation_steps.contains(&"computedValueResolved"));
    }

    #[test]
    fn resolves_inheritance_initial_and_unset_keywords() {
        let inherited = compute_cascade_computed_value(CascadeComputedValueInputV0 {
            property: "color".to_string(),
            declarations: Vec::new(),
            custom_property_env: CustomPropertyEnv::new(),
            parent_computed_value: Some(CascadeValue::Literal("purple".to_string())),
        });
        assert_eq!(inherited.status, ComputedCascadeValueStatusV0::Inherited);
        assert_eq!(inherited.value, CascadeValue::Literal("purple".to_string()));
        assert!(inherited.inherited);

        let initial = compute_cascade_computed_value(CascadeComputedValueInputV0 {
            property: "opacity".to_string(),
            declarations: Vec::new(),
            custom_property_env: CustomPropertyEnv::new(),
            parent_computed_value: Some(CascadeValue::Literal("0.5".to_string())),
        });
        assert_eq!(initial.status, ComputedCascadeValueStatusV0::Initial);
        assert_eq!(initial.value, CascadeValue::Literal("1".to_string()));
        assert!(initial.used_initial_value);

        let unset_inherited = compute_cascade_computed_value(CascadeComputedValueInputV0 {
            property: "color".to_string(),
            declarations: vec![property_declaration(
                "unset-color",
                "color",
                CascadeValue::Unset,
                1,
            )],
            custom_property_env: CustomPropertyEnv::new(),
            parent_computed_value: Some(CascadeValue::Literal("green".to_string())),
        });
        assert_eq!(
            unset_inherited.status,
            ComputedCascadeValueStatusV0::Inherited
        );
        assert_eq!(
            unset_inherited.value,
            CascadeValue::Literal("green".to_string())
        );

        let unset_initial = compute_cascade_computed_value(CascadeComputedValueInputV0 {
            property: "opacity".to_string(),
            declarations: vec![property_declaration(
                "unset-opacity",
                "opacity",
                CascadeValue::Unset,
                1,
            )],
            custom_property_env: CustomPropertyEnv::new(),
            parent_computed_value: Some(CascadeValue::Literal("0.5".to_string())),
        });
        assert_eq!(unset_initial.status, ComputedCascadeValueStatusV0::Initial);
        assert_eq!(unset_initial.value, CascadeValue::Literal("1".to_string()));
    }

    #[test]
    fn treats_guaranteed_invalid_var_substitution_as_iacvt_unset() {
        let mut env = CustomPropertyEnv::new();
        env.insert(
            "--a".to_string(),
            CascadeValue::Var {
                name: "--b".to_string(),
                fallback: None,
            },
        );
        env.insert(
            "--b".to_string(),
            CascadeValue::Var {
                name: "--a".to_string(),
                fallback: None,
            },
        );

        let result = compute_cascade_computed_value(CascadeComputedValueInputV0 {
            property: "color".to_string(),
            declarations: vec![property_declaration(
                "cycle-color",
                "color",
                CascadeValue::Var {
                    name: "--a".to_string(),
                    fallback: None,
                },
                1,
            )],
            custom_property_env: env,
            parent_computed_value: Some(CascadeValue::Literal("canvas".to_string())),
        });

        assert_eq!(
            result.status,
            ComputedCascadeValueStatusV0::InvalidAtComputedValueTime
        );
        assert_eq!(result.value, CascadeValue::Literal("canvas".to_string()));
        assert!(result.inherited);
        assert!(result.invalid_at_computed_value_time);
        assert!(
            result
                .derivation_steps
                .contains(&"invalidAtComputedValueTimeFallsBackAsUnset")
        );
    }

    #[test]
    fn proves_adjacent_box_longhands_can_combine_to_shorthand() {
        let proof = prove_box_shorthand_combination(
            "margin",
            &[
                BoxLonghandInputV0 {
                    property: "margin-top".to_string(),
                    value: "1px".to_string(),
                    important: false,
                    source_order: 1,
                },
                BoxLonghandInputV0 {
                    property: "margin-right".to_string(),
                    value: "2px".to_string(),
                    important: false,
                    source_order: 2,
                },
                BoxLonghandInputV0 {
                    property: "margin-bottom".to_string(),
                    value: "3px".to_string(),
                    important: false,
                    source_order: 3,
                },
                BoxLonghandInputV0 {
                    property: "margin-left".to_string(),
                    value: "4px".to_string(),
                    important: false,
                    source_order: 4,
                },
            ],
        );

        assert_eq!(proof.product, "omena-cascade.shorthand-combination-proof");
        assert!(proof.accepted);
        assert_eq!(proof.blocked_reason, None);
        assert!(proof.provenance_preserved);
        assert!(proof.cascade_safe_witness.contains("canonical order"));

        let border_proof = prove_box_shorthand_combination(
            "border-color",
            &[
                BoxLonghandInputV0 {
                    property: "border-top-color".to_string(),
                    value: "red".to_string(),
                    important: false,
                    source_order: 1,
                },
                BoxLonghandInputV0 {
                    property: "border-right-color".to_string(),
                    value: "blue".to_string(),
                    important: false,
                    source_order: 2,
                },
                BoxLonghandInputV0 {
                    property: "border-bottom-color".to_string(),
                    value: "red".to_string(),
                    important: false,
                    source_order: 3,
                },
                BoxLonghandInputV0 {
                    property: "border-left-color".to_string(),
                    value: "blue".to_string(),
                    important: false,
                    source_order: 4,
                },
            ],
        );
        assert!(border_proof.accepted);
        assert!(border_proof.provenance_preserved);
    }

    #[test]
    fn blocks_box_shorthand_combination_when_intervening_order_is_possible() {
        let proof = prove_box_shorthand_combination(
            "padding",
            &[
                BoxLonghandInputV0 {
                    property: "padding-top".to_string(),
                    value: "1px".to_string(),
                    important: false,
                    source_order: 1,
                },
                BoxLonghandInputV0 {
                    property: "padding-right".to_string(),
                    value: "2px".to_string(),
                    important: false,
                    source_order: 3,
                },
                BoxLonghandInputV0 {
                    property: "padding-bottom".to_string(),
                    value: "3px".to_string(),
                    important: false,
                    source_order: 4,
                },
                BoxLonghandInputV0 {
                    property: "padding-left".to_string(),
                    value: "4px".to_string(),
                    important: false,
                    source_order: 5,
                },
            ],
        );

        assert!(!proof.accepted);
        assert_eq!(
            proof.blocked_reason,
            Some("intervening declaration may change cascade outcome")
        );
        assert!(!proof.provenance_preserved);
    }

    #[test]
    fn evaluates_simple_supports_conditions_under_modern_browser_assumption() {
        let positive = evaluate_static_supports_condition(
            "(display: grid)",
            StaticSupportsAssumptionV0::ModernBrowser,
        );
        assert_eq!(positive.product, "omena-cascade.supports-static-eval");
        assert_eq!(positive.verdict, StaticSupportsEvalVerdictV0::AlwaysTrue);
        assert!(positive.provenance_preserved);

        let negative = evaluate_static_supports_condition(
            "not (display: grid)",
            StaticSupportsAssumptionV0::ModernBrowser,
        );
        assert_eq!(negative.verdict, StaticSupportsEvalVerdictV0::AlwaysFalse);
        assert!(negative.provenance_preserved);

        let conjunction = evaluate_static_supports_condition(
            "(display: grid) and (color: red)",
            StaticSupportsAssumptionV0::ModernBrowser,
        );
        assert_eq!(conjunction.verdict, StaticSupportsEvalVerdictV0::AlwaysTrue);
        assert!(conjunction.provenance_preserved);

        let disjunction = evaluate_static_supports_condition(
            "(display: grid) or (selector(:has(*)))",
            StaticSupportsAssumptionV0::ModernBrowser,
        );
        assert_eq!(disjunction.verdict, StaticSupportsEvalVerdictV0::AlwaysTrue);
        assert!(disjunction.provenance_preserved);

        let selector = evaluate_static_supports_condition(
            "selector(:has(*))",
            StaticSupportsAssumptionV0::ModernBrowser,
        );
        assert_eq!(selector.verdict, StaticSupportsEvalVerdictV0::AlwaysTrue);
        assert!(selector.provenance_preserved);

        let obsolete_selector = evaluate_static_supports_condition(
            "selector(:-ms-input-placeholder)",
            StaticSupportsAssumptionV0::ModernBrowser,
        );
        assert_eq!(
            obsolete_selector.verdict,
            StaticSupportsEvalVerdictV0::AlwaysFalse
        );
        assert!(obsolete_selector.provenance_preserved);

        let negated_selector = evaluate_static_supports_condition(
            "not selector(:has(*))",
            StaticSupportsAssumptionV0::ModernBrowser,
        );
        assert_eq!(
            negated_selector.verdict,
            StaticSupportsEvalVerdictV0::AlwaysFalse
        );
        assert!(negated_selector.provenance_preserved);

        let color_function = evaluate_static_supports_condition(
            "(color: color(display-p3 1 0 0))",
            StaticSupportsAssumptionV0::ModernBrowser,
        );
        assert_eq!(
            color_function.verdict,
            StaticSupportsEvalVerdictV0::AlwaysTrue
        );
        assert!(color_function.provenance_preserved);

        let gradient_function = evaluate_static_supports_condition(
            "(background-image: linear-gradient(red, blue))",
            StaticSupportsAssumptionV0::ModernBrowser,
        );
        assert_eq!(
            gradient_function.verdict,
            StaticSupportsEvalVerdictV0::AlwaysTrue
        );
        assert!(gradient_function.provenance_preserved);

        let malformed_function = evaluate_static_supports_condition(
            "(color: color(display-p3 1 0 0)",
            StaticSupportsAssumptionV0::ModernBrowser,
        );
        assert_eq!(
            malformed_function.verdict,
            StaticSupportsEvalVerdictV0::Unknown
        );
        assert!(!malformed_function.provenance_preserved);

        let grouped_disjunction = evaluate_static_supports_condition(
            "((display: grid) or (display: -ms-grid))",
            StaticSupportsAssumptionV0::ModernBrowser,
        );
        assert_eq!(
            grouped_disjunction.verdict,
            StaticSupportsEvalVerdictV0::AlwaysTrue
        );
        assert!(grouped_disjunction.provenance_preserved);

        let grouped_conjunction = evaluate_static_supports_condition(
            "((display: grid) or (display: -ms-grid)) and (color: red)",
            StaticSupportsAssumptionV0::ModernBrowser,
        );
        assert_eq!(
            grouped_conjunction.verdict,
            StaticSupportsEvalVerdictV0::AlwaysTrue
        );
        assert!(grouped_conjunction.provenance_preserved);

        let obsolete_disjunction = evaluate_static_supports_condition(
            "(display: -ms-grid) or (-ms-ime-align: auto)",
            StaticSupportsAssumptionV0::ModernBrowser,
        );
        assert_eq!(
            obsolete_disjunction.verdict,
            StaticSupportsEvalVerdictV0::AlwaysFalse
        );
        assert!(obsolete_disjunction.provenance_preserved);

        let obsolete = evaluate_static_supports_condition(
            "(display: -ms-grid)",
            StaticSupportsAssumptionV0::ModernBrowser,
        );
        assert_eq!(obsolete.verdict, StaticSupportsEvalVerdictV0::AlwaysFalse);
        assert!(obsolete.provenance_preserved);

        let negated_obsolete = evaluate_static_supports_condition(
            "not (display: -ms-grid)",
            StaticSupportsAssumptionV0::ModernBrowser,
        );
        assert_eq!(
            negated_obsolete.verdict,
            StaticSupportsEvalVerdictV0::AlwaysTrue
        );
        assert!(negated_obsolete.provenance_preserved);

        let negated_grouped_obsolete = evaluate_static_supports_condition(
            "not ((display: -ms-grid) or (-ms-ime-align: auto))",
            StaticSupportsAssumptionV0::ModernBrowser,
        );
        assert_eq!(
            negated_grouped_obsolete.verdict,
            StaticSupportsEvalVerdictV0::AlwaysTrue
        );
        assert!(negated_grouped_obsolete.provenance_preserved);

        let negated_grouped_supported = evaluate_static_supports_condition(
            "not ((display: grid) or (display: -ms-grid))",
            StaticSupportsAssumptionV0::ModernBrowser,
        );
        assert_eq!(
            negated_grouped_supported.verdict,
            StaticSupportsEvalVerdictV0::AlwaysFalse
        );
        assert!(negated_grouped_supported.provenance_preserved);
    }

    #[test]
    fn proves_only_root_scope_flatten_candidates_without_competition() {
        let accepted = prove_scope_flatten_candidate(ScopeFlattenInputV0 {
            root_selector: ":root".to_string(),
            limit_selector: None,
            scoped_rule_count: 1,
            peer_scope_count: 0,
            competing_unscoped_rule_count: 0,
            inside_layer: false,
        });
        assert_eq!(accepted.product, "omena-cascade.scope-flatten-proof");
        assert!(accepted.accepted);
        assert!(accepted.provenance_preserved);

        let blocked = prove_scope_flatten_candidate(ScopeFlattenInputV0 {
            root_selector: ".card".to_string(),
            limit_selector: None,
            scoped_rule_count: 1,
            peer_scope_count: 0,
            competing_unscoped_rule_count: 0,
            inside_layer: false,
        });
        assert!(!blocked.accepted);
        assert_eq!(
            blocked.blocked_reason,
            Some("non-root scope flattening requires selector/proximity equivalence proof")
        );
    }

    #[test]
    fn proves_layer_flatten_only_for_closed_single_layer_candidates() {
        let accepted = prove_layer_flatten_candidate(LayerFlattenInputV0 {
            layer_name: Some("theme".to_string()),
            layer_rule_count: 1,
            peer_layer_count: 0,
            unlayered_rule_count: 0,
            important_declaration_count: 0,
            closed_bundle: true,
        });
        assert_eq!(accepted.product, "omena-cascade.layer-flatten-proof");
        assert!(accepted.accepted);
        assert!(accepted.provenance_preserved);

        let blocked = prove_layer_flatten_candidate(LayerFlattenInputV0 {
            layer_name: Some("theme".to_string()),
            layer_rule_count: 1,
            peer_layer_count: 0,
            unlayered_rule_count: 1,
            important_declaration_count: 0,
            closed_bundle: true,
        });
        assert!(!blocked.accepted);
        assert_eq!(
            blocked.blocked_reason,
            Some("unlayered rules compete differently from layered normal rules")
        );
    }

    #[test]
    fn reports_selector_context_witness_rank() {
        let root = selector_context_witness(&[":root".to_string()], &[".button".to_string()]);
        assert_eq!(root.kind, SelectorContextMatchKind::Root);
        assert!(root.matched);
        assert_eq!(root.rank, 1);

        let exact = selector_context_witness(&[".button".to_string()], &[".button".to_string()]);
        assert_eq!(exact.kind, SelectorContextMatchKind::Exact);
        assert_eq!(exact.rank, 2);

        let descendant =
            selector_context_witness(&[".theme".to_string()], &[".theme .button".to_string()]);
        assert_eq!(descendant.kind, SelectorContextMatchKind::ContainsSelector);
        assert_eq!(
            descendant.reference_selector.as_deref(),
            Some(".theme .button")
        );

        let miss = selector_context_witness(&[".card".to_string()], &[".button".to_string()]);
        assert_eq!(miss.kind, SelectorContextMatchKind::NoMatch);
        assert!(!miss.matched);
    }

    #[test]
    fn parses_simple_selector_specificity() {
        let signature = parse_simple_selector_signature("button#save.primary[data-state]:hover");
        assert!(signature.is_some());
        if let Some(signature) = signature {
            assert_eq!(signature.required_tag.as_deref(), Some("button"));
            assert_eq!(signature.required_id.as_deref(), Some("save"));
            assert!(signature.required_classes.contains("primary"));
            assert!(signature.required_attributes.contains("data-state"));
            assert!(signature.required_pseudo_states.contains("hover"));
            assert_eq!(signature.specificity, Specificity::new(1, 3, 1));
        }
    }

    #[test]
    fn matches_simple_compound_selectors_against_concrete_signature() {
        let mut element =
            ElementSignature::concrete(Some("button"), Some("save"), ["primary", "active"]);
        element.attributes.insert("data-state".to_string());
        element.pseudo_states.insert("hover".to_string());

        let witness = selector_match_witness("button#save.primary[data-state]:hover", &element);

        assert_eq!(witness.verdict, SelectorMatchVerdict::Yes);
        assert_eq!(witness.reason, SelectorMatchReason::SimpleCompound);
        assert_eq!(witness.specificity, Specificity::new(1, 3, 1));
    }

    #[test]
    fn reports_missing_class_and_id_as_no_for_exact_signature() {
        let element = ElementSignature::concrete(Some("button"), Some("save"), ["primary"]);

        let class_miss = selector_match_witness(".missing", &element);
        assert_eq!(class_miss.verdict, SelectorMatchVerdict::No);
        assert_eq!(class_miss.reason, SelectorMatchReason::MissingClass);
        assert!(class_miss.missing_classes.contains("missing"));

        let id_miss = selector_match_witness("#cancel", &element);
        assert_eq!(id_miss.verdict, SelectorMatchVerdict::No);
        assert_eq!(id_miss.reason, SelectorMatchReason::MissingId);
        assert_eq!(id_miss.missing_id.as_deref(), Some("cancel"));
    }

    #[test]
    fn returns_maybe_for_inexact_abstract_class_sets() {
        let element = ElementSignature::at_least_classes(["button"]);

        let witness = selector_match_witness(".button.primary", &element);

        assert_eq!(witness.verdict, SelectorMatchVerdict::Maybe);
        assert_eq!(witness.reason, SelectorMatchReason::MissingClass);
        assert!(witness.missing_classes.contains("primary"));
    }

    #[test]
    fn selector_lists_choose_strongest_matching_branch() {
        let element = ElementSignature::concrete(Some("button"), Some("save"), ["primary"]);

        let witness = selector_match_witness(".missing, button#save.primary", &element);

        assert_eq!(witness.verdict, SelectorMatchVerdict::Yes);
        assert_eq!(witness.reason, SelectorMatchReason::SelectorList);
        assert_eq!(
            witness.matched_branch.as_deref(),
            Some("button#save.primary")
        );
        assert_eq!(witness.specificity, Specificity::new(1, 1, 1));
    }

    #[test]
    fn unsupported_combinators_are_reported_as_maybe() {
        let element = ElementSignature::concrete(Some("span"), None::<String>, ["icon"]);

        let witness = selector_match_witness(".button > .icon", &element);

        assert_eq!(witness.verdict, SelectorMatchVerdict::Maybe);
        assert_eq!(witness.reason, SelectorMatchReason::UnsupportedSelector);
        assert_eq!(witness.unsupported_branches, vec![".button > .icon"]);
    }

    #[test]
    fn substitutes_custom_property_fallbacks_and_references() {
        let mut env = CustomPropertyEnv::new();
        env.insert(
            "--brand".to_string(),
            CascadeValue::Literal("red".to_string()),
        );

        let resolved = substitute_custom_properties(
            &CascadeValue::Var {
                name: "--brand".to_string(),
                fallback: Some(Box::new(CascadeValue::Literal("blue".to_string()))),
            },
            &env,
        );
        assert_eq!(resolved, CascadeValue::Literal("red".to_string()));

        let fallback = substitute_custom_properties(
            &CascadeValue::Var {
                name: "--missing".to_string(),
                fallback: Some(Box::new(CascadeValue::Literal("blue".to_string()))),
            },
            &env,
        );
        assert_eq!(fallback, CascadeValue::Literal("blue".to_string()));
    }

    #[test]
    fn substitutes_custom_properties_inside_composite_values() {
        let mut env = CustomPropertyEnv::new();
        env.insert(
            "--gap".to_string(),
            CascadeValue::Literal("2px".to_string()),
        );
        env.insert(
            "--shadow".to_string(),
            CascadeValue::Composite(vec![
                CascadeValue::Literal("0 0 ".to_string()),
                CascadeValue::Var {
                    name: "--gap".to_string(),
                    fallback: None,
                },
            ]),
        );
        env.insert(
            "--invalid-shadow".to_string(),
            CascadeValue::Composite(vec![
                CascadeValue::Literal("0 0 ".to_string()),
                CascadeValue::Var {
                    name: "--missing".to_string(),
                    fallback: None,
                },
            ]),
        );

        let resolved = substitute_custom_properties(
            &CascadeValue::Var {
                name: "--shadow".to_string(),
                fallback: None,
            },
            &env,
        );
        assert_eq!(
            resolved,
            CascadeValue::Composite(vec![
                CascadeValue::Literal("0 0 ".to_string()),
                CascadeValue::Literal("2px".to_string()),
            ])
        );

        let fallback = substitute_custom_properties(
            &CascadeValue::Var {
                name: "--invalid-shadow".to_string(),
                fallback: Some(Box::new(CascadeValue::Literal("none".to_string()))),
            },
            &env,
        );
        assert_eq!(fallback, CascadeValue::Literal("none".to_string()));
    }

    #[test]
    fn substitutes_cycles_to_guaranteed_invalid() {
        let mut env = CustomPropertyEnv::new();
        env.insert(
            "--a".to_string(),
            CascadeValue::Var {
                name: "--b".to_string(),
                fallback: None,
            },
        );
        env.insert(
            "--b".to_string(),
            CascadeValue::Var {
                name: "--a".to_string(),
                fallback: None,
            },
        );

        let resolved = substitute_custom_properties(
            &CascadeValue::Var {
                name: "--a".to_string(),
                fallback: None,
            },
            &env,
        );

        assert_eq!(resolved, CascadeValue::GuaranteedInvalid);

        let fallback = substitute_custom_properties(
            &CascadeValue::Var {
                name: "--a".to_string(),
                fallback: Some(Box::new(CascadeValue::Literal("blue".to_string()))),
            },
            &env,
        );

        assert_eq!(fallback, CascadeValue::Literal("blue".to_string()));
    }

    #[test]
    fn summarizes_custom_property_least_fixed_point() {
        let mut env = CustomPropertyEnv::new();
        env.insert(
            "--brand".to_string(),
            CascadeValue::Literal("red".to_string()),
        );
        env.insert(
            "--alias".to_string(),
            CascadeValue::Var {
                name: "--brand".to_string(),
                fallback: None,
            },
        );
        env.insert(
            "--shadow".to_string(),
            CascadeValue::Composite(vec![
                CascadeValue::Literal("0 0 ".to_string()),
                CascadeValue::Var {
                    name: "--alias".to_string(),
                    fallback: None,
                },
            ]),
        );
        env.insert(
            "--cycle-a".to_string(),
            CascadeValue::Var {
                name: "--cycle-b".to_string(),
                fallback: None,
            },
        );
        env.insert(
            "--cycle-b".to_string(),
            CascadeValue::Var {
                name: "--cycle-a".to_string(),
                fallback: None,
            },
        );

        let summary = summarize_custom_property_least_fixed_point(&env);

        assert_eq!(
            summary.product,
            "omena-cascade.custom-property-least-fixed-point"
        );
        assert_eq!(summary.input_count, 5);
        assert_eq!(summary.resolved_count, 3);
        assert_eq!(summary.guaranteed_invalid_count, 2);
        assert!(summary.iteration_count >= 2);
        assert_eq!(summary.iteration_bound, 6);
        assert!(summary.reached_fixed_point);
        assert!(summary.monotone_witness_valid);
        assert_eq!(summary.iteration_trace.len(), summary.iteration_count);
        assert!(
            summary
                .iteration_trace
                .windows(2)
                .all(|pair| pair[0].settled_count <= pair[1].settled_count)
        );
        assert_eq!(
            summary.proof.iteration_bound_formula,
            "max(1, env.len() + 1)"
        );
        assert!(
            summary
                .proof
                .proof_obligations
                .contains(&"explicit fixed-point equality check")
        );
        assert!(
            summary
                .proof
                .proof_obligations
                .contains(&"nondecreasing settled-value trace")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"customPropertyLeastFixedPoint")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"customPropertyLeastFixedPointProof")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"customPropertyLeastFixedPointTrace")
        );
        assert!(summary.entries.iter().any(|entry| {
            entry.name == "--alias" && entry.resolved == CascadeValue::Literal("red".to_string())
        }));
        assert!(summary.entries.iter().any(|entry| {
            entry.name == "--shadow"
                && entry.resolved
                    == CascadeValue::Composite(vec![
                        CascadeValue::Literal("0 0 ".to_string()),
                        CascadeValue::Literal("red".to_string()),
                    ])
        }));
        assert!(summary.entries.iter().any(|entry| {
            entry.name == "--cycle-a" && entry.resolved == CascadeValue::GuaranteedInvalid
        }));
    }

    #[test]
    fn fuzz_seed_corpus_preserves_cascade_and_var_invariants() {
        let report = run_cascade_fuzz_seed_corpus();

        assert_eq!(report.product, "omena-cascade.fuzz-seed-corpus");
        assert_eq!(report.failed_count, 0);
        assert_eq!(report.passed_count, report.case_count);
        assert!(
            report
                .var_results
                .iter()
                .any(|result| result.cycle && matches!(result.result, CascadeValue::Literal(_)))
        );
    }

    #[test]
    fn summarizes_current_boundary_status() {
        let summary = summarize_cascade_boundary();

        assert_eq!(summary.product, "omena-cascade.boundary");
        assert_eq!(summary.ordering_model, "lexicographicCascadeKey");
        assert_eq!(
            summary.least_fixed_point_proof_model,
            "finite-env monotone custom-property substitution with cycle-to-guaranteed-invalid bottoming and env-size iteration bound"
        );
        assert!(summary.ready_surfaces.contains(&"cascadeKeyOrdering"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"customPropertyLeastFixedPoint")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"customPropertyLeastFixedPointProof")
        );
        assert!(summary.ready_surfaces.contains(&"genericCascadeWinner"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"semanticDesignTokenRanking")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"queryReadCascadeAtPosition")
        );
        assert!(summary.ready_surfaces.contains(&"selectorContextWitness"));
        assert!(summary.ready_surfaces.contains(&"selectorMatchWitness"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"supportsStaticEvalWitness")
        );
        assert!(summary.ready_surfaces.contains(&"scopeFlattenProof"));
        assert!(summary.ready_surfaces.contains(&"layerFlattenProof"));
        assert!(summary.ready_surfaces.contains(&"wptCascadeSeedCorpus"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"cascadeConformanceSeedCorpus")
        );
        assert!(!summary.not_ready_surfaces.contains(&"selectorMatchWitness"));
        assert!(!summary.not_ready_surfaces.contains(&"wptCascadeCorpus"));
        assert!(summary.not_ready_surfaces.contains(&"fullWptCascadeCorpus"));
    }

    #[test]
    fn seed_conformance_corpus_passes_current_cascade_model() {
        let report = run_cascade_conformance_seed_corpus();

        assert_eq!(report.product, "omena-cascade.conformance-seed-corpus");
        assert_eq!(report.case_count, 6);
        assert_eq!(report.passed_count, report.case_count);
        assert_eq!(report.failed_count, 0);
        assert!(report.results.iter().all(|result| result.passed));
    }

    #[test]
    fn wpt_cascade_seed_corpus_passes_current_cascade_model() {
        let report = run_wpt_cascade_seed_corpus();

        assert_eq!(report.product, "omena-cascade.wpt-cascade-seed-corpus");
        assert!(report.case_count >= 200);
        assert_eq!(report.passed_count, report.case_count);
        assert_eq!(report.failed_count, 0);
        assert!(report.results.iter().all(|result| result.passed));
    }
}
