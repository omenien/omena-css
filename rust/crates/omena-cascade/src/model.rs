//! Public data model for cascade ordering, selector witnesses, and proof reports.
//!
//! These serializable types are the stable boundary consumed by query,
//! transform, conformance, fuzz, and LSP surfaces. They intentionally expose
//! evidence fields instead of opaque booleans so later passes can explain why a
//! cascade-sensitive rewrite was accepted or blocked.

use omena_syntax::ident::{
    AuthoredPropertyTextV0, CanonicalCustomPropertyNameV0, CanonicalPropertyKeyV0,
};
use serde::{Deserialize, Serialize};
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
    InlineImportant,
    UserImportant,
    UserAgentImportant,
    Transition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LayerRank(i32);

impl LayerRank {
    /// Returns the opaque scalar used by the cascade key ordering.
    pub const fn get(self) -> i32 {
        self.0
    }
}

/// Position in a flattened cascade-layer order before importance normalization.
///
/// The sentinel-safe domain is `0 <= ordinal < i32::MAX`; `None` represents an
/// unlayered declaration at the normalization boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct LayerOrdinal(i32);

impl LayerOrdinal {
    /// Rejects ordinals that would collide with the unlayered sentinels.
    pub const fn new(ordinal: i32) -> Option<Self> {
        if 0 <= ordinal && ordinal < i32::MAX {
            Some(Self(ordinal))
        } else {
            None
        }
    }

    pub const fn get(self) -> i32 {
        self.0
    }
}

/// Maps a layer ordinal into the comparison domain used by `CascadeKey`.
///
/// Unlayered declarations form an implicit final layer, and the whole layer
/// order is reversed for important declarations. This scalar encoding is sound
/// because `CascadeKey` compares `level` before `layer_rank`, so normal and
/// important declarations never rely on their shared zero value.
pub const fn normalized_layer_rank(important: bool, ordinal: Option<LayerOrdinal>) -> LayerRank {
    match (important, ordinal) {
        (false, Some(ordinal)) => LayerRank(ordinal.get()),
        (false, None) => LayerRank(i32::MAX),
        (true, Some(ordinal)) => LayerRank(-ordinal.get()),
        (true, None) => LayerRank(i32::MIN),
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
/// Whether a specificity estimate is complete enough for exact cascade ordering.
pub enum SpecificityExactnessV0 {
    /// Every selector component that contributes specificity was modeled.
    Exact,
    /// The numeric specificity is only a lower bound because some syntax was unmodeled.
    Inexact,
}

impl Ord for Specificity {
    fn cmp(&self, other: &Self) -> Ordering {
        crate::axis_order::compare_specificity_axes_v0(self, other)
    }
}

impl PartialOrd for Specificity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleRank {
    pub distance_priority: u32,
    pub import_order_priority: u32,
    pub file_order_priority: u32,
}

impl ModuleRank {
    pub const ZERO: Self = Self {
        distance_priority: 0,
        import_order_priority: 0,
        file_order_priority: 0,
    };

    pub const fn new(
        distance_priority: u32,
        import_order_priority: u32,
        file_order_priority: u32,
    ) -> Self {
        Self {
            distance_priority,
            import_order_priority,
            file_order_priority,
        }
    }
}

impl Ord for ModuleRank {
    fn cmp(&self, other: &Self) -> Ordering {
        (
            self.distance_priority,
            self.import_order_priority,
            self.file_order_priority,
        )
            .cmp(&(
                other.distance_priority,
                other.import_order_priority,
                other.file_order_priority,
            ))
    }
}

impl PartialOrd for ModuleRank {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Provenance evidence used only to make open-world ties deterministic.
///
/// This evidence is deliberately separate from [`CascadeKey`]: it is not a
/// spec-defined cascade axis and cannot make an otherwise ambiguous cascade
/// outcome definite.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenWorldTieEvidence {
    pub module_rank: ModuleRank,
}

impl OpenWorldTieEvidence {
    /// No provenance preference is available.
    pub const NONE: Self = Self {
        module_rank: ModuleRank::ZERO,
    };

    /// Numeric zero form retained for callers that model evidence as a rank.
    pub const ZERO: Self = Self::NONE;

    pub const fn new(module_rank: ModuleRank) -> Self {
        Self { module_rank }
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

pub(crate) fn compare_cascade_axis_prefix(left: &CascadeKey, right: &CascadeKey) -> Ordering {
    crate::axis_order::compare_cascade_axis_prefix_v0(left, right)
}

impl Ord for CascadeKey {
    fn cmp(&self, other: &Self) -> Ordering {
        crate::axis_order::compare_cascade_key_axes_v0(self, other)
    }
}

impl PartialOrd for CascadeKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeDeclaration {
    pub id: String,
    pub property: AuthoredPropertyTextV0,
    pub property_key: CanonicalPropertyKeyV0,
    pub value: CascadeValue,
    pub key: CascadeKey,
    /// Non-spec evidence for deterministic ordering of open-world ties.
    pub open_world_tie_evidence: OpenWorldTieEvidence,
    /// Trust boundary for using `key.specificity` to mint an exact winner.
    pub specificity_exactness: SpecificityExactnessV0,
}

impl PartialEq for CascadeDeclaration {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.property_key == other.property_key
            && self.value == other.value
            && self.key == other.key
            && self.open_world_tie_evidence == other.open_world_tie_evidence
            && self.specificity_exactness == other.specificity_exactness
    }
}

impl Eq for CascadeDeclaration {}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeProof {
    pub declaration_id: String,
    pub property: AuthoredPropertyTextV0,
    pub property_key: CanonicalPropertyKeyV0,
    pub level: CascadeLevel,
    pub layer_rank: LayerRank,
    pub scope_proximity: u32,
    pub specificity: Specificity,
    pub module_rank: ModuleRank,
    pub source_order: u32,
}

impl PartialEq for CascadeProof {
    fn eq(&self, other: &Self) -> bool {
        self.declaration_id == other.declaration_id
            && self.property_key == other.property_key
            && self.level == other.level
            && self.layer_rank == other.layer_rank
            && self.scope_proximity == other.scope_proximity
            && self.specificity == other.specificity
            && self.module_rank == other.module_rank
            && self.source_order == other.source_order
    }
}

impl Eq for CascadeProof {}

impl CascadeProof {
    pub fn from_declaration(declaration: &CascadeDeclaration) -> Self {
        assert_eq!(
            declaration.specificity_exactness,
            SpecificityExactnessV0::Exact,
            "cascade proofs require exact specificity"
        );
        Self {
            declaration_id: declaration.id.clone(),
            property: declaration.property.clone(),
            property_key: declaration.property_key.clone(),
            level: declaration.key.level,
            layer_rank: declaration.key.layer_rank,
            scope_proximity: declaration.key.scope_proximity,
            specificity: declaration.key.specificity,
            module_rank: declaration.open_world_tie_evidence.module_rank,
            source_order: declaration.key.source_order,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CascadeOutcome {
    Definite {
        winner: CascadeDeclaration,
        proof: Box<CascadeProof>,
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
        name: CanonicalCustomPropertyNameV0,
        fallback: Option<Box<CascadeValue>>,
    },
    Initial,
    Inherit,
    Indeterminate,
    GuaranteedInvalid,
    Unset,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ComputedCascadeValueStatusV0 {
    Resolved,
    Inherited,
    Initial,
    Indeterminate,
    InvalidAtComputedValueTime,
}

macro_rules! define_computed_cascade_indeterminate_reasons {
    ($($variant:ident => $wire_name:literal),+ $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(rename_all = "camelCase")]
        pub enum ComputedCascadeIndeterminateReasonV0 {
            $($variant),+
        }

        impl ComputedCascadeIndeterminateReasonV0 {
            #[cfg(test)]
            pub(crate) const ALL: &'static [Self] = &[$(Self::$variant),+];

            pub const fn wire_name(self) -> &'static str {
                match self {
                    $(Self::$variant => $wire_name),+
                }
            }
        }
    };
}

define_computed_cascade_indeterminate_reasons! {
    CascadeOutcomeIndeterminate => "cascadeOutcomeIndeterminate",
    PropertyInheritanceMetadataUnavailable => "propertyInheritanceMetadataUnavailable",
    PropertyInitialValueMetadataUnavailable => "propertyInitialValueMetadataUnavailable",
    RegisteredPropertySyntaxIndeterminate => "registeredPropertySyntaxIndeterminate",
    StandardPropertySyntaxIndeterminate => "standardPropertySyntaxIndeterminate",
    InheritedFromIndeterminateParent => "inheritedFromIndeterminateParent",
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CascadeRegisteredValueVerdictV0 {
    Matched,
    Unmatched,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CascadeStandardValueVerdictV0 {
    Matched,
    Unmatched,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeRegisteredCustomPropertyV0 {
    pub name: String,
    pub inherits: bool,
    pub initial_value: CascadeValue,
    pub declaration_value_verdicts: BTreeMap<String, CascadeRegisteredValueVerdictV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeComputedValueInputV0 {
    pub property: String,
    pub declarations: Vec<CascadeDeclaration>,
    pub custom_property_env: CustomPropertyEnv,
    pub parent_computed_value: Option<CascadeValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registered_custom_property: Option<CascadeRegisteredCustomPropertyV0>,
    /// Caller-supplied grammar verdicts for standard (non-custom) properties,
    /// keyed by declaration id. `omena-cascade` does not own a property grammar;
    /// the authority is `omena-abstract-value::validate_standard_property_value_v0`,
    /// consulted by the caller. Absence means no verdict is available, not that
    /// the value is valid.
    pub standard_property_value_verdicts: BTreeMap<String, CascadeStandardValueVerdictV0>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indeterminate_reason: Option<ComputedCascadeIndeterminateReasonV0>,
    /// Why the value the declaration falls back to could not be determined when
    /// the declaration itself became invalid at computed-value time. This is
    /// orthogonal to `indeterminate_reason`, which remains absent unless the
    /// result status is `Indeterminate`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_indeterminate_reason: Option<ComputedCascadeIndeterminateReasonV0>,
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
    ApproximateSelector,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorContextWitness {
    pub kind: SelectorContextMatchKind,
    pub verdict: SelectorMatchVerdict,
    pub matched: bool,
    pub rank: usize,
    pub declaration_selector: Option<String>,
    pub reference_selector: Option<String>,
}

impl SelectorContextWitness {
    pub fn no_match() -> Self {
        Self {
            kind: SelectorContextMatchKind::NoMatch,
            verdict: SelectorMatchVerdict::No,
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementIdentityV0 {
    pub source_path: String,
    pub byte_start: usize,
    pub byte_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementSignatureWithParentsV0 {
    pub identity: ElementIdentityV0,
    pub signature: ElementSignature,
    pub parent_chain: Vec<ElementIdentityV0>,
    pub parent_chain_complete: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ElementParentChainStatusV0 {
    Complete,
    MissingSource,
    MissingElement,
    AmbiguousParent,
    Cycle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementParentChainV0 {
    pub target: ElementIdentityV0,
    pub ancestors: Vec<ElementIdentityV0>,
    pub status: ElementParentChainStatusV0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ScopeProximityStatusV0 {
    Known,
    IncompleteParentChain,
    MissingElementSignature,
    UnsupportedRootSelector,
    NoMatchingRoot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeProximityV0 {
    pub status: ScopeProximityStatusV0,
    pub distance: Option<u32>,
    pub matched_root: Option<ElementIdentityV0>,
    pub examined_element_count: usize,
}

impl ScopeProximityV0 {
    pub const fn unknown(status: ScopeProximityStatusV0) -> Self {
        Self {
            status,
            distance: None,
            matched_root: None,
            examined_element_count: 0,
        }
    }
}

impl ElementParentChainV0 {
    pub fn is_complete(&self) -> bool {
        self.status == ElementParentChainStatusV0::Complete
    }
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
pub struct SelectorFunctionalPseudoConstraintV0 {
    pub name: String,
    pub arguments: String,
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
    pub functional_pseudo_constraints: Vec<SelectorFunctionalPseudoConstraintV0>,
    pub specificity: Specificity,
    pub specificity_exactness: SpecificityExactnessV0,
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
    pub specificity_exactness: SpecificityExactnessV0,
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
            specificity_exactness: SpecificityExactnessV0::Inexact,
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

/// Historical compatibility shape for the custom-property structural computation witness.
///
/// New code should prefer [`CustomPropertyBoundedFixedPointComputationWitnessV0`].
/// The proof-oriented name and fields remain available for 0.x consumers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomPropertyLeastFixedPointProofV0 {
    pub finite_domain: &'static str,
    pub transfer_function: &'static str,
    #[serde(skip_serializing)]
    pub bounded_fixed_point_computation_witness: &'static str,
    /// Compatibility wording; prefer [`Self::monotonic_progress_witness`].
    pub monotone_witness: &'static str,
    #[serde(skip_serializing)]
    pub monotonic_progress_witness: &'static str,
    pub iteration_bound_formula: &'static str,
    pub cycle_policy: &'static str,
    /// Compatibility wording retained alongside the computation-witness fields.
    pub proof_obligations: Vec<&'static str>,
}

/// Compatibility alias retained for the structural custom-property computation witness.
pub type CustomPropertyBoundedFixedPointComputationWitnessV0 = CustomPropertyLeastFixedPointProofV0;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomPropertyLeastFixedPointIterationV0 {
    pub iteration: usize,
    pub changed_count: usize,
    pub settled_count: usize,
    pub guaranteed_invalid_count: usize,
}

/// The structural reason a custom-property binding became guaranteed-invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CustomPropertyGuaranteedInvalidReasonV0 {
    CycleMember,
    MissingReference,
    InvalidDependencyWithoutFallback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomPropertyLeastFixedPointEntryV0 {
    pub name: String,
    pub input: CascadeValue,
    pub resolved: CascadeValue,
    pub changed: bool,
    pub guaranteed_invalid: bool,
    pub guaranteed_invalid_reason: Option<CustomPropertyGuaranteedInvalidReasonV0>,
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

pub type LonghandMergeInputV0 = BoxLonghandInputV0;

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

pub type LonghandMergeProofV0 = ShorthandCombinationProofV0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SupportsTargetCapabilityV0 {
    pub supports_light_dark: bool,
    pub supports_color_mix: bool,
    pub supports_oklch_oklab: bool,
    pub supports_color_function: bool,
    pub supports_relative_color: bool,
    pub supports_logical_properties: bool,
    pub supports_css_nesting: bool,
    pub supports_css_scope: bool,
    pub supports_cascade_layers: bool,
}

impl SupportsTargetCapabilityV0 {
    pub const fn all_supported() -> Self {
        Self {
            supports_light_dark: true,
            supports_color_mix: true,
            supports_oklch_oklab: true,
            supports_color_function: true,
            supports_relative_color: true,
            supports_logical_properties: true,
            supports_css_nesting: true,
            supports_css_scope: true,
            supports_cascade_layers: true,
        }
    }

    pub const fn none_supported() -> Self {
        Self {
            supports_light_dark: false,
            supports_color_mix: false,
            supports_oklch_oklab: false,
            supports_color_function: false,
            supports_relative_color: false,
            supports_logical_properties: false,
            supports_css_nesting: false,
            supports_css_scope: false,
            supports_cascade_layers: false,
        }
    }
}

impl Default for SupportsTargetCapabilityV0 {
    fn default() -> Self {
        Self::none_supported()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StaticSupportsAssumptionV0 {
    ModernBrowser,
    TargetCapability(SupportsTargetCapabilityV0),
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

pub type CustomPropertyEnv = BTreeMap<CanonicalCustomPropertyNameV0, CascadeValue>;
