use std::collections::{BTreeMap, BTreeSet};

use omena_abstract_value::{
    AbstractClassValueV0, CssValueValidationClassV0, SelectorProjectionCertaintyV0,
    enumerate_finite_class_values, project_abstract_value_selectors,
    registered_property_syntax_requires_initial_value_v0, validate_registered_property_value_v0,
    validate_standard_property_value_v0,
};
use omena_cascade::{GrnBooleanState, GrnVertexStateV0, GrnVertexV0, project_grn_outcome};
#[cfg(not(feature = "smt-z3"))]
use omena_cascade_proof::{
    SmtBackendKindV0, SmtBackendSatResultV0, SmtBackendV0, StubSmtBackendV0, canonical_smt_input_v0,
};
pub use omena_product_hints::CategoricalCascadeEvidenceV0;
use omena_product_hints::{
    CascadeFunctorApplicationV0, PatternIntentV0, apply_cascade_role_mapping_functor_v0,
    coupling_space, designer_intent_posterior_input_v0, dominant_designer_intent_v0,
    estimate_coupling_jacobian_spectrum_v0, infer_designer_intent_posterior_v0,
};
pub use omena_product_hints::{
    RG_FLOW_DEFAULT_PRODUCT_DECISION_MECHANISM_V0, RG_FLOW_MECHANISM_SCOPE_V0,
    RG_FLOW_PRODUCT_SURFACE_V0,
};
#[cfg(feature = "smt-z3")]
use omena_smt::Z3SmtBackendV0;
#[cfg(feature = "smt-z3")]
use omena_smt::{
    SmtBackendKindV0, SmtBackendSatResultV0, SmtBackendV0, SmtVerdictV0, canonical_smt_input_v0,
    layer_inversion_declaration_v0, smt_check_layer_flatten_inversion_v0,
};
use serde::{Deserialize, Serialize};

mod enforcement_coverage;
mod fix_safety;
mod frame_emission;
mod lint_tier;
mod rule_metadata;
mod selectors;
pub use enforcement_coverage::*;
pub use fix_safety::*;
pub use frame_emission::*;
pub use lint_tier::*;
use rule_metadata::{bundle, count_rules_in_tier, rule, rule_tier_for_code};
pub use selectors::{CanonicalSelector, RawSelector};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OmenaCheckerRuleCodeV0 {
    NoUnknownDynamicClass,
    NoImpreciseValue,
    NoImpossibleSelector,
    MissingModule,
    MissingStaticClass,
    MissingTemplatePrefix,
    MissingResolvedClassValues,
    MissingResolvedClassDomain,
    UnusedSelector,
    MissingComposedModule,
    MissingComposedSelector,
    MissingValueModule,
    MissingImportedValue,
    MissingKeyframes,
    MissingCustomProperty,
    MissingSassSymbol,
    UnreachableDeclaration,
    DeadCascadeLayer,
    IacvtProne,
    CircularVar,
    UnspecifiedCascadeTie,
    DesignSystemMdlBudget,
    CascadeDeepConflict,
    CascadeUnreachableRule,
    CascadeSMTViolation,
    DesignerIntentInconsistency,
    StreamingIfdsPrecisionParity,
    RgFlowRelevantOperator,
    ReplicaEnsembleInconsistency,
    CategoricalCascadeEvidenceInconsistency,
    RegisteredPropertyTypeMismatch,
    InvalidPropertyValue,
}

impl OmenaCheckerRuleCodeV0 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoUnknownDynamicClass => "no-unknown-dynamic-class",
            Self::NoImpreciseValue => "no-imprecise-value",
            Self::NoImpossibleSelector => "no-impossible-selector",
            Self::MissingModule => "missing-module",
            Self::MissingStaticClass => "missing-static-class",
            Self::MissingTemplatePrefix => "missing-template-prefix",
            Self::MissingResolvedClassValues => "missing-resolved-class-values",
            Self::MissingResolvedClassDomain => "missing-resolved-class-domain",
            Self::UnusedSelector => "unused-selector",
            Self::MissingComposedModule => "missing-composed-module",
            Self::MissingComposedSelector => "missing-composed-selector",
            Self::MissingValueModule => "missing-value-module",
            Self::MissingImportedValue => "missing-imported-value",
            Self::MissingKeyframes => "missing-keyframes",
            Self::MissingCustomProperty => "missing-custom-property",
            Self::MissingSassSymbol => "missing-sass-symbol",
            Self::UnreachableDeclaration => "unreachable-declaration",
            Self::DeadCascadeLayer => "dead-cascade-layer",
            Self::IacvtProne => "iacvt-prone",
            Self::CircularVar => "circular-var",
            Self::UnspecifiedCascadeTie => "unspecified-cascade-tie",
            Self::DesignSystemMdlBudget => "design-system-mdl-budget",
            Self::CascadeDeepConflict => "cascade.deep-conflict",
            Self::CascadeUnreachableRule => "cascade.unreachable-rule",
            Self::CascadeSMTViolation => "cascade.smt-violation",
            Self::DesignerIntentInconsistency => "designer-intent-inconsistency",
            Self::StreamingIfdsPrecisionParity => "streaming-ifds-precision-parity",
            Self::RgFlowRelevantOperator => "rg-flow-relevant-operator",
            Self::ReplicaEnsembleInconsistency => "replica-ensemble-inconsistency",
            Self::CategoricalCascadeEvidenceInconsistency => {
                "categorical-cascade-evidence-inconsistency"
            }
            Self::RegisteredPropertyTypeMismatch => "registered-property-type-mismatch",
            Self::InvalidPropertyValue => "invalid-property-value",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaCheckerFindingCategoryV0 {
    Source,
    Style,
}

impl OmenaCheckerFindingCategoryV0 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Style => "style",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OmenaCheckerRuleTierV0 {
    M,
    S,
    T,
    I,
}

impl OmenaCheckerRuleTierV0 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::M => "m-tier",
            Self::S => "s-tier",
            Self::T => "t-tier",
            Self::I => "i-tier",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaCheckerSeverityV0 {
    Warning,
    Hint,
}

impl OmenaCheckerSeverityV0 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Warning => "warning",
            Self::Hint => "hint",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaCheckerRuleFixabilityV0 {
    None,
    CodeAction,
    Autofix,
}

impl OmenaCheckerRuleFixabilityV0 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::CodeAction => "codeAction",
            Self::Autofix => "autofix",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OmenaCheckerRulePresetV0 {
    Recommended,
    Strict,
}

impl OmenaCheckerRulePresetV0 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Recommended => "recommended",
            Self::Strict => "strict",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OmenaCheckerCodeBundleNameV0 {
    CiDefault,
    SourceMissing,
    StyleRecovery,
    StyleUnused,
    CascadeAware,
}

impl OmenaCheckerCodeBundleNameV0 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CiDefault => "ci-default",
            Self::SourceMissing => "source-missing",
            Self::StyleRecovery => "style-recovery",
            Self::StyleUnused => "style-unused",
            Self::CascadeAware => "cascade-aware",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerRuleDescriptorV0 {
    pub code: OmenaCheckerRuleCodeV0,
    pub code_name: &'static str,
    pub ordinal: u16,
    pub category: OmenaCheckerFindingCategoryV0,
    pub category_name: &'static str,
    pub tier: OmenaCheckerRuleTierV0,
    pub tier_name: &'static str,
    pub default_severity: OmenaCheckerSeverityV0,
    pub default_severity_name: &'static str,
    pub fixability: OmenaCheckerRuleFixabilityV0,
    pub fixability_name: &'static str,
    pub presets: Vec<OmenaCheckerRulePresetV0>,
    pub preset_names: Vec<&'static str>,
    pub description: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaCheckerSmtBackendKindV0 {
    Stub,
    Z3,
}

impl OmenaCheckerSmtBackendKindV0 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stub => "stub",
            Self::Z3 => "z3",
        }
    }

    pub const fn product_name(self) -> &'static str {
        match self {
            Self::Stub => "omena-smt.backend.stub",
            Self::Z3 => "omena-smt.backend.z3",
        }
    }

    pub const fn product_scope(self) -> &'static str {
        match self {
            Self::Stub => "defaultSolverFreeStubProductGate",
            Self::Z3 => "explicitOptInZ3SolverBackedProductGate",
        }
    }

    pub const fn solver_backed(self) -> bool {
        matches!(self, Self::Z3)
    }
}

#[cfg(feature = "smt-z3")]
pub const fn active_omena_checker_smt_backend_kind_v0() -> OmenaCheckerSmtBackendKindV0 {
    OmenaCheckerSmtBackendKindV0::Z3
}

#[cfg(not(feature = "smt-z3"))]
pub const fn active_omena_checker_smt_backend_kind_v0() -> OmenaCheckerSmtBackendKindV0 {
    OmenaCheckerSmtBackendKindV0::Stub
}

pub const fn active_omena_checker_smt_backend_kind_name_v0() -> &'static str {
    active_omena_checker_smt_backend_kind_v0().as_str()
}

pub const fn active_omena_checker_smt_product_scope_v0() -> &'static str {
    active_omena_checker_smt_backend_kind_v0().product_scope()
}

pub const fn active_omena_checker_smt_solver_backed_v0() -> bool {
    active_omena_checker_smt_backend_kind_v0().solver_backed()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerCodeBundleV0 {
    pub bundle: OmenaCheckerCodeBundleNameV0,
    pub bundle_name: &'static str,
    pub codes: Vec<OmenaCheckerRuleCodeV0>,
    pub code_names: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerBoundarySummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub owner_crate: &'static str,
    pub rule_registry_product: &'static str,
    pub bundle_registry_product: &'static str,
    pub rule_count: usize,
    pub bundle_count: usize,
    pub source_rule_count: usize,
    pub style_rule_count: usize,
    pub m_tier_rule_count: usize,
    pub s_tier_rule_count: usize,
    pub t_tier_rule_count: usize,
    pub i_tier_rule_count: usize,
    pub bridge_policy: Vec<&'static str>,
    pub next_migration_targets: Vec<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaCheckerDynamicClassDomainOutcomeV0 {
    Known,
    MissingResolvedClassValues,
    MissingResolvedClassDomain,
}

impl OmenaCheckerDynamicClassDomainOutcomeV0 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Known => "known",
            Self::MissingResolvedClassValues => "missingResolvedClassValues",
            Self::MissingResolvedClassDomain => "missingResolvedClassDomain",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerDynamicClassDomainInputV0 {
    pub abstract_value: AbstractClassValueV0,
    pub selector_universe: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerDynamicClassDomainEvaluationV0 {
    pub outcome: OmenaCheckerDynamicClassDomainOutcomeV0,
    pub outcome_name: &'static str,
    pub rule_code: Option<OmenaCheckerRuleCodeV0>,
    pub rule_code_name: Option<&'static str>,
    pub selector_names: Vec<String>,
    pub selector_certainty: SelectorProjectionCertaintyV0,
    pub finite_values: Option<Vec<String>>,
    pub missing_values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerMTierEvaluationV0 {
    pub rule_code: OmenaCheckerRuleCodeV0,
    pub rule_code_name: &'static str,
    pub severity: OmenaCheckerSeverityV0,
    pub severity_name: &'static str,
    pub selector_names: Vec<String>,
    pub selector_certainty: SelectorProjectionCertaintyV0,
    pub finite_values: Option<Vec<String>>,
    pub missing_values: Vec<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerCascadeInputV0 {
    pub declarations: Vec<OmenaCheckerCascadeDeclarationInputV0>,
    pub custom_properties: Vec<OmenaCheckerCustomPropertyInputV0>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_property_registrations: Vec<OmenaCheckerCustomPropertyRegistrationInputV0>,
}

/// Cascade declaration consumed by the checker.
///
/// The selector field intentionally requires [`CanonicalSelector`]. Raw parser
/// selectors must be expanded by the query/parser boundary first.
///
/// ```compile_fail
/// use omena_checker::OmenaCheckerCascadeDeclarationInputV0;
///
/// let _declaration = OmenaCheckerCascadeDeclarationInputV0 {
///     declaration_id: "decl-0".to_string(),
///     selector: ".button".to_string(),
///     property: "color".to_string(),
///     value: "red".to_string(),
///     source_order: 0,
///     condition_context: Vec::new(),
///     layer_name: None,
///     layer_order: None,
///     important: false,
///     var_references: Vec::new(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerCascadeDeclarationInputV0 {
    pub declaration_id: String,
    pub selector: CanonicalSelector,
    pub property: String,
    pub value: String,
    pub source_order: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub condition_context: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layer_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layer_order: Option<i32>,
    pub important: bool,
    pub var_references: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerCustomPropertyInputV0 {
    pub name: String,
    pub dependencies: Vec<String>,
    pub guaranteed_invalid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerCustomPropertyRegistrationInputV0 {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syntax: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherits: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerCascadeEvaluationV0 {
    pub rule_code: OmenaCheckerRuleCodeV0,
    pub rule_code_name: &'static str,
    pub severity: OmenaCheckerSeverityV0,
    pub severity_name: &'static str,
    pub declaration_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layer_name: Option<String>,
    pub custom_property_names: Vec<String>,
    pub message: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mechanism_products: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerGrnInputV0 {
    pub vertices: Vec<OmenaCheckerGrnVertexStateInputV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerGrnVertexStateInputV0 {
    pub vertex_id: String,
    pub selector: String,
    pub property: String,
    pub state: OmenaCheckerGrnVertexStateKindV0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaCheckerGrnVertexStateKindV0 {
    Applied,
    LosingButEligible,
    Inactive,
    Top,
}

impl OmenaCheckerGrnVertexStateKindV0 {
    fn into_cascade_state(self) -> GrnBooleanState {
        match self {
            Self::Applied => GrnBooleanState::Applied,
            Self::LosingButEligible => GrnBooleanState::LosingButEligible,
            Self::Inactive => GrnBooleanState::Inactive,
            Self::Top => GrnBooleanState::Top,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerGrnEvaluationV0 {
    pub rule_code: OmenaCheckerRuleCodeV0,
    pub rule_code_name: &'static str,
    pub severity: OmenaCheckerSeverityV0,
    pub severity_name: &'static str,
    pub vertex_ids: Vec<String>,
    pub message: String,
    pub mechanism_products: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerSmtInputV0 {
    pub obligations: Vec<OmenaCheckerSmtObligationInputV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerSmtObligationInputV0 {
    pub obligation_id: String,
    pub l1_primitive: String,
    pub canonical_terms: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerSmtLayerInversionInputV0 {
    pub obligations: Vec<OmenaCheckerSmtLayerInversionObligationInputV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerSmtLayerInversionObligationInputV0 {
    pub obligation_id: String,
    pub declarations: Vec<OmenaCheckerSmtLayerInversionDeclarationInputV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerSmtLayerInversionDeclarationInputV0 {
    pub declaration_id: String,
    pub layer_rank: i64,
    pub source_order: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerSmtEvaluationV0 {
    pub rule_code: OmenaCheckerRuleCodeV0,
    pub rule_code_name: &'static str,
    pub severity: OmenaCheckerSeverityV0,
    pub severity_name: &'static str,
    pub obligation_id: String,
    pub backend_kind_name: &'static str,
    pub sat_result_name: &'static str,
    pub message: String,
    pub mechanism_products: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerMdlInputV0 {
    pub summaries: Vec<OmenaCheckerMdlSummaryInputV0>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerMdlSummaryInputV0 {
    pub source_uri: String,
    pub total_bits: f64,
    pub budget_bits: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerMdlEvaluationV0 {
    pub rule_code: OmenaCheckerRuleCodeV0,
    pub rule_code_name: &'static str,
    pub severity: OmenaCheckerSeverityV0,
    pub severity_name: &'static str,
    pub source_uri: String,
    pub total_bits: f64,
    pub budget_bits: f64,
    pub message: String,
    pub mechanism_products: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerStreamingIfdsInputV0 {
    pub reports: Vec<OmenaCheckerStreamingIfdsReportInputV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerStreamingIfdsReportInputV0 {
    pub report_id: String,
    pub incremental_precision_parity_with_batch: bool,
    pub reachability_fallback_applied: bool,
    pub fact_fallback_applied: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerStreamingIfdsEvaluationV0 {
    pub rule_code: OmenaCheckerRuleCodeV0,
    pub rule_code_name: &'static str,
    pub severity: OmenaCheckerSeverityV0,
    pub severity_name: &'static str,
    pub report_id: String,
    pub incremental_precision_parity_with_batch: bool,
    pub reachability_fallback_applied: bool,
    pub fact_fallback_applied: bool,
    pub message: String,
    pub mechanism_products: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerRgFlowInputV0 {
    pub flows: Vec<OmenaCheckerRgFlowCouplingInputV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerRgFlowCouplingInputV0 {
    pub workspace_path: String,
    pub before: OmenaCheckerRgFlowCouplingSpaceInputV0,
    pub after: OmenaCheckerRgFlowCouplingSpaceInputV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerRgFlowCouplingSpaceInputV0 {
    pub k_env: usize,
    pub k_decl: usize,
    pub k_cycle: usize,
    pub k_dirty: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerRgFlowEvaluationV0 {
    pub rule_code: OmenaCheckerRuleCodeV0,
    pub rule_code_name: &'static str,
    pub severity: OmenaCheckerSeverityV0,
    pub severity_name: &'static str,
    pub workspace_path: String,
    pub spectral_radius: f64,
    pub eigenvalues: Vec<f64>,
    pub mechanism_scope: &'static str,
    pub product_surface: &'static str,
    pub default_product_decision_mechanism: bool,
    pub message: String,
    pub mechanism_products: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerReplicaEnsembleInputV0 {
    pub reports: Vec<OmenaCheckerReplicaEnsembleReportInputV0>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerReplicaEnsembleReportInputV0 {
    pub workspace_root: String,
    pub recommendation: String,
    pub mean_q: f64,
    pub variance_q: f64,
    pub top_disagreement_pair_count: usize,
    #[serde(default = "default_replica_ensemble_mechanism_scope_v0")]
    pub mechanism_scope: String,
    #[serde(default = "default_replica_ensemble_product_surface_v0")]
    pub product_surface: String,
    #[serde(default)]
    pub default_product_decision_mechanism: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerReplicaEnsembleEvaluationV0 {
    pub rule_code: OmenaCheckerRuleCodeV0,
    pub rule_code_name: &'static str,
    pub severity: OmenaCheckerSeverityV0,
    pub severity_name: &'static str,
    pub workspace_root: String,
    pub recommendation: String,
    pub mean_q: f64,
    pub variance_q: f64,
    pub top_disagreement_pair_count: usize,
    pub mechanism_scope: String,
    pub product_surface: String,
    pub default_product_decision_mechanism: bool,
    pub message: String,
    pub mechanism_products: Vec<&'static str>,
}

fn default_replica_ensemble_mechanism_scope_v0() -> String {
    "unspecifiedReplicaEnsembleScope".to_string()
}

fn default_replica_ensemble_product_surface_v0() -> String {
    "unspecifiedReplicaEnsembleProductSurface".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerCategoricalInputV0 {
    pub mappings: Vec<OmenaCheckerCategoricalRoleMappingInputV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerCategoricalRoleMappingInputV0 {
    pub mapping_id: String,
    pub primitive_role_pairs: Vec<OmenaCheckerCategoricalPrimitiveRolePairInputV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerCategoricalPrimitiveRolePairInputV0 {
    pub primitive_name: String,
    pub categorical_role: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerCategoricalEvaluationV0 {
    pub rule_code: OmenaCheckerRuleCodeV0,
    pub rule_code_name: &'static str,
    pub severity: OmenaCheckerSeverityV0,
    pub severity_name: &'static str,
    pub mapping_id: String,
    pub object_mapping_count: usize,
    pub morphism_mapping_count: usize,
    pub identity_preserved: bool,
    pub composition_preserved: bool,
    pub functor_accepted: bool,
    pub message: String,
    pub mechanism_products: Vec<&'static str>,
}

pub fn list_omena_checker_rule_descriptors() -> Vec<OmenaCheckerRuleDescriptorV0> {
    use OmenaCheckerFindingCategoryV0::{Source, Style};
    use OmenaCheckerRuleCodeV0::{
        CascadeDeepConflict, CascadeSMTViolation, CascadeUnreachableRule,
        CategoricalCascadeEvidenceInconsistency, CircularVar, DeadCascadeLayer,
        DesignSystemMdlBudget, DesignerIntentInconsistency, IacvtProne, InvalidPropertyValue,
        MissingComposedModule, MissingComposedSelector, MissingCustomProperty,
        MissingImportedValue, MissingKeyframes, MissingModule, MissingResolvedClassDomain,
        MissingResolvedClassValues, MissingSassSymbol, MissingStaticClass, MissingTemplatePrefix,
        MissingValueModule, NoImpossibleSelector, NoImpreciseValue, NoUnknownDynamicClass,
        RegisteredPropertyTypeMismatch, ReplicaEnsembleInconsistency, RgFlowRelevantOperator,
        StreamingIfdsPrecisionParity, UnreachableDeclaration, UnspecifiedCascadeTie,
        UnusedSelector,
    };
    use OmenaCheckerRuleFixabilityV0::{CodeAction, None};
    use OmenaCheckerRulePresetV0::{Recommended, Strict};
    use OmenaCheckerSeverityV0::{Hint, Warning};

    vec![
        rule(
            NoUnknownDynamicClass,
            Source,
            Warning,
            None,
            &[Recommended],
            "Report dynamic class expressions whose abstract value cannot be proven against the selector universe.",
        ),
        rule(
            NoImpreciseValue,
            Source,
            Hint,
            None,
            &[Strict],
            "Report class value domains whose selector projection remains inferred or possible instead of exact.",
        ),
        rule(
            NoImpossibleSelector,
            Source,
            Warning,
            None,
            &[Strict],
            "Report finite dynamic class values that project to no selector and therefore cannot match.",
        ),
        rule(
            MissingModule,
            Source,
            Warning,
            CodeAction,
            &[Recommended],
            "Report unresolved CSS Module imports from source files.",
        ),
        rule(
            MissingStaticClass,
            Source,
            Warning,
            CodeAction,
            &[Recommended],
            "Report static class names that do not exist in the target CSS Module.",
        ),
        rule(
            MissingTemplatePrefix,
            Source,
            Warning,
            None,
            &[Recommended],
            "Report template-literal class prefixes that match no target selector.",
        ),
        rule(
            MissingResolvedClassValues,
            Source,
            Warning,
            None,
            &[Recommended],
            "Report finite dynamic class values that resolve outside the selector set.",
        ),
        rule(
            MissingResolvedClassDomain,
            Source,
            Warning,
            None,
            &[Recommended],
            "Report dynamic class domains that cannot be proven against known selectors.",
        ),
        rule(
            UnusedSelector,
            Style,
            Hint,
            None,
            &[Strict],
            "Report CSS Module selectors with no indexed source references.",
        ),
        rule(
            MissingComposedModule,
            Style,
            Warning,
            CodeAction,
            &[Recommended],
            "Report unresolved composes-from module specifiers.",
        ),
        rule(
            MissingComposedSelector,
            Style,
            Warning,
            CodeAction,
            &[Recommended],
            "Report composed class names missing from the resolved target module.",
        ),
        rule(
            MissingValueModule,
            Style,
            Warning,
            CodeAction,
            &[Recommended],
            "Report unresolved Sass/CSS @value module specifiers.",
        ),
        rule(
            MissingImportedValue,
            Style,
            Warning,
            CodeAction,
            &[Recommended],
            "Report @value names missing from the resolved target module.",
        ),
        rule(
            MissingKeyframes,
            Style,
            Warning,
            CodeAction,
            &[Recommended],
            "Report animation names that do not resolve to local @keyframes.",
        ),
        rule(
            MissingCustomProperty,
            Style,
            Warning,
            None,
            &[Strict],
            "Report CSS custom property references with no indexed declaration.",
        ),
        rule(
            MissingSassSymbol,
            Style,
            Warning,
            None,
            &[Recommended],
            "Report unresolved Sass/Less variable, mixin, and function references.",
        ),
        rule(
            UnreachableDeclaration,
            Style,
            Hint,
            None,
            &[Strict],
            "Report declarations that are always outranked by another declaration with the same selector and property.",
        ),
        rule(
            DeadCascadeLayer,
            Style,
            Hint,
            None,
            &[Strict],
            "Report cascade layers whose declarations are all outranked by declarations from other layers.",
        ),
        rule(
            IacvtProne,
            Style,
            Warning,
            None,
            &[Recommended],
            "Report declarations whose var() references may produce an invalid-at-computed-value-time result.",
        ),
        rule(
            CircularVar,
            Style,
            Warning,
            None,
            &[Recommended],
            "Report custom property dependency cycles that make participating variables guaranteed-invalid.",
        ),
        rule(
            UnspecifiedCascadeTie,
            Style,
            Hint,
            None,
            &[Strict],
            "Report same-selector same-property declaration pairs that rely on source order as the final cascade tie-breaker.",
        ),
        rule(
            DesignerIntentInconsistency,
            Style,
            Hint,
            None,
            &[Strict],
            "Report variational designer-intent posterior evidence that disagrees with deterministic cascade facts.",
        ),
        rule(
            CascadeSMTViolation,
            Style,
            Warning,
            None,
            &[Strict],
            "Report cascade proof obligations whose opt-in SMT backend rejects the L1 cascade proof primitive.",
        ),
        rule(
            DesignSystemMdlBudget,
            Style,
            Hint,
            None,
            &[Strict],
            "Report design-system compression candidates whose MDL budget is worse than the configured canonical fallback.",
        ),
        rule(
            StreamingIfdsPrecisionParity,
            Style,
            Hint,
            None,
            &[Strict],
            "Report streaming IFDS results that fail exact parity with the batch hypergraph oracle.",
        ),
        rule(
            RgFlowRelevantOperator,
            Style,
            Hint,
            None,
            &[Strict],
            "Report RG-flow coupling spectra whose relevant operator indicates unstable cascade sensitivity.",
        ),
        rule(
            ReplicaEnsembleInconsistency,
            Style,
            Hint,
            None,
            &[Strict],
            "Report replica-ensemble cross-file inconsistency reports that recommend investigation.",
        ),
        rule(
            CascadeDeepConflict,
            Style,
            Warning,
            None,
            &[Recommended],
            "Report cascade conflict clusters whose GRN attractor basin indicates a stable deep conflict.",
        ),
        rule(
            CascadeUnreachableRule,
            Style,
            Hint,
            None,
            &[Strict],
            "Report cascade rules that the GRN state model proves unreachable under the current fixture slice.",
        ),
        rule(
            CategoricalCascadeEvidenceInconsistency,
            Style,
            Hint,
            None,
            &[Strict],
            "Report cascade primitive-to-role mappings whose categorical functor fails identity or composition preservation.",
        ),
        rule(
            RegisteredPropertyTypeMismatch,
            Style,
            Warning,
            None,
            &[Recommended],
            "Report custom property declarations whose values definitely do not match their same-file @property syntax registration.",
        ),
        rule(
            InvalidPropertyValue,
            Style,
            Warning,
            None,
            &[Recommended],
            "Report standard property declarations whose value is provably outside the pinned property grammar.",
        ),
    ]
}

pub fn list_omena_checker_rule_codes() -> Vec<OmenaCheckerRuleCodeV0> {
    list_omena_checker_rule_descriptors()
        .into_iter()
        .map(|descriptor| descriptor.code)
        .collect()
}

pub fn list_omena_checker_rule_code_names() -> Vec<&'static str> {
    list_omena_checker_rule_codes()
        .into_iter()
        .map(OmenaCheckerRuleCodeV0::as_str)
        .collect()
}

pub fn list_omena_checker_m_tier_rule_codes() -> Vec<OmenaCheckerRuleCodeV0> {
    vec![
        OmenaCheckerRuleCodeV0::NoUnknownDynamicClass,
        OmenaCheckerRuleCodeV0::NoImpreciseValue,
        OmenaCheckerRuleCodeV0::NoImpossibleSelector,
    ]
}

pub fn list_omena_checker_m_tier_rule_code_names() -> Vec<&'static str> {
    list_omena_checker_m_tier_rule_codes()
        .into_iter()
        .map(OmenaCheckerRuleCodeV0::as_str)
        .collect()
}

pub fn list_omena_checker_s_tier_rule_codes() -> Vec<OmenaCheckerRuleCodeV0> {
    use OmenaCheckerRuleCodeV0::{
        CascadeDeepConflict, CascadeSMTViolation, MissingModule, MissingResolvedClassDomain,
        MissingResolvedClassValues, MissingStaticClass, MissingTemplatePrefix,
    };

    vec![
        MissingModule,
        MissingStaticClass,
        MissingTemplatePrefix,
        MissingResolvedClassValues,
        MissingResolvedClassDomain,
        CascadeSMTViolation,
        CascadeDeepConflict,
    ]
}

pub fn list_omena_checker_s_tier_rule_code_names() -> Vec<&'static str> {
    list_omena_checker_s_tier_rule_codes()
        .into_iter()
        .map(OmenaCheckerRuleCodeV0::as_str)
        .collect()
}

pub fn list_omena_checker_t_tier_rule_codes() -> Vec<OmenaCheckerRuleCodeV0> {
    use OmenaCheckerRuleCodeV0::{
        CircularVar, DeadCascadeLayer, IacvtProne, MissingComposedModule, MissingComposedSelector,
        MissingCustomProperty, MissingImportedValue, MissingKeyframes, MissingSassSymbol,
        MissingValueModule, RegisteredPropertyTypeMismatch, UnreachableDeclaration,
        UnspecifiedCascadeTie, UnusedSelector,
    };

    vec![
        UnusedSelector,
        MissingComposedModule,
        MissingComposedSelector,
        MissingValueModule,
        MissingImportedValue,
        MissingKeyframes,
        MissingCustomProperty,
        MissingSassSymbol,
        UnreachableDeclaration,
        DeadCascadeLayer,
        IacvtProne,
        CircularVar,
        RegisteredPropertyTypeMismatch,
        UnspecifiedCascadeTie,
    ]
}

pub fn list_omena_checker_t_tier_rule_code_names() -> Vec<&'static str> {
    list_omena_checker_t_tier_rule_codes()
        .into_iter()
        .map(OmenaCheckerRuleCodeV0::as_str)
        .collect()
}

pub fn list_omena_checker_i_tier_rule_codes() -> Vec<OmenaCheckerRuleCodeV0> {
    use OmenaCheckerRuleCodeV0::{
        CascadeUnreachableRule, CategoricalCascadeEvidenceInconsistency, DesignSystemMdlBudget,
        DesignerIntentInconsistency, ReplicaEnsembleInconsistency, RgFlowRelevantOperator,
        StreamingIfdsPrecisionParity,
    };

    vec![
        DesignerIntentInconsistency,
        DesignSystemMdlBudget,
        StreamingIfdsPrecisionParity,
        RgFlowRelevantOperator,
        ReplicaEnsembleInconsistency,
        CascadeUnreachableRule,
        CategoricalCascadeEvidenceInconsistency,
    ]
}

pub fn list_omena_checker_i_tier_rule_code_names() -> Vec<&'static str> {
    list_omena_checker_i_tier_rule_codes()
        .into_iter()
        .map(OmenaCheckerRuleCodeV0::as_str)
        .collect()
}

pub fn is_omena_checker_rule_code(value: &str) -> bool {
    list_omena_checker_rule_codes()
        .into_iter()
        .any(|code| code.as_str() == value)
}

pub fn get_omena_checker_rule_descriptor(
    code: OmenaCheckerRuleCodeV0,
) -> Option<OmenaCheckerRuleDescriptorV0> {
    list_omena_checker_rule_descriptors()
        .into_iter()
        .find(|descriptor| descriptor.code == code)
}

pub fn resolve_omena_checker_rule_tier_for_smt_backend(
    code: OmenaCheckerRuleCodeV0,
    backend_kind: OmenaCheckerSmtBackendKindV0,
) -> OmenaCheckerRuleTierV0 {
    if code != OmenaCheckerRuleCodeV0::CascadeSMTViolation {
        return rule_tier_for_code(code);
    }

    match backend_kind {
        OmenaCheckerSmtBackendKindV0::Stub => OmenaCheckerRuleTierV0::I,
        OmenaCheckerSmtBackendKindV0::Z3 => OmenaCheckerRuleTierV0::S,
    }
}

pub fn checker_categorical_cascade_evidence_v0(
    source_product: &'static str,
) -> CategoricalCascadeEvidenceV0 {
    omena_product_hints::categorical_cascade_evidence_v0(source_product)
}

/// Build cascade categorical evidence whose functor application is the real
/// verdict over the cascade primitives a concrete stylesheet exercises.
///
/// Thin wrapper over the product-owned categorical hint contract so the query
/// style path can attach a verdict that depends on the parsed cascade without a
/// Query -> Theory layer skip.
pub fn checker_categorical_cascade_evidence_for_exercised_primitives_v0(
    source_product: &'static str,
    exercised_primitive_role_pairs: &[(String, String)],
) -> CategoricalCascadeEvidenceV0 {
    omena_product_hints::categorical_cascade_evidence_for_exercised_primitives_v0(
        source_product,
        exercised_primitive_role_pairs,
    )
}

/// The canonical cascade primitive-to-categorical-role catalog for default
/// product diagnostics.
///
/// Each tuple is `(primitive_name, categorical_role)`. Product callers (the
/// query style path) project this catalog down to the cascade primitives a real
/// stylesheet exercises and feed the projection into the categorical functor
/// gate; keeping the catalog here avoids a Query -> Theory layer skip and keeps
/// the role names in sync with the categorical source of truth.
pub fn checker_cascade_primitive_role_catalog_v0() -> Vec<(&'static str, &'static str)> {
    omena_product_hints::cascade_primitive_roles_v0()
        .into_iter()
        .map(|role| (role.primitive_name, role.categorical_role))
        .collect()
}

pub fn list_omena_checker_code_bundles() -> Vec<OmenaCheckerCodeBundleV0> {
    use OmenaCheckerCodeBundleNameV0::{
        CascadeAware, CiDefault, SourceMissing, StyleRecovery, StyleUnused,
    };
    use OmenaCheckerRuleCodeV0::{
        CascadeDeepConflict, CascadeSMTViolation, CascadeUnreachableRule,
        CategoricalCascadeEvidenceInconsistency, CircularVar, DeadCascadeLayer,
        DesignSystemMdlBudget, DesignerIntentInconsistency, IacvtProne, MissingComposedModule,
        MissingComposedSelector, MissingImportedValue, MissingKeyframes, MissingModule,
        MissingResolvedClassDomain, MissingResolvedClassValues, MissingSassSymbol,
        MissingStaticClass, MissingTemplatePrefix, MissingValueModule, NoImpossibleSelector,
        NoImpreciseValue, NoUnknownDynamicClass, RegisteredPropertyTypeMismatch,
        StreamingIfdsPrecisionParity, UnreachableDeclaration, UnspecifiedCascadeTie,
        UnusedSelector,
    };

    vec![
        bundle(
            CiDefault,
            &[
                NoUnknownDynamicClass,
                MissingModule,
                MissingStaticClass,
                MissingTemplatePrefix,
                MissingResolvedClassValues,
                MissingResolvedClassDomain,
                MissingComposedModule,
                MissingComposedSelector,
                MissingValueModule,
                MissingImportedValue,
                MissingKeyframes,
                MissingSassSymbol,
            ],
        ),
        bundle(
            SourceMissing,
            &[
                NoUnknownDynamicClass,
                NoImpreciseValue,
                NoImpossibleSelector,
                MissingModule,
                MissingStaticClass,
                MissingTemplatePrefix,
                MissingResolvedClassValues,
                MissingResolvedClassDomain,
            ],
        ),
        bundle(
            StyleRecovery,
            &[
                MissingComposedModule,
                MissingComposedSelector,
                MissingValueModule,
                MissingImportedValue,
                MissingKeyframes,
                MissingSassSymbol,
            ],
        ),
        bundle(StyleUnused, &[UnusedSelector]),
        bundle(
            CascadeAware,
            &[
                UnreachableDeclaration,
                DeadCascadeLayer,
                IacvtProne,
                CircularVar,
                RegisteredPropertyTypeMismatch,
                UnspecifiedCascadeTie,
                CascadeDeepConflict,
                CascadeUnreachableRule,
                DesignSystemMdlBudget,
                CascadeSMTViolation,
                DesignerIntentInconsistency,
                StreamingIfdsPrecisionParity,
                CategoricalCascadeEvidenceInconsistency,
            ],
        ),
    ]
}

pub fn summarize_omena_checker_boundary() -> OmenaCheckerBoundarySummaryV0 {
    let descriptors = list_omena_checker_rule_descriptors();
    let source_rule_count = descriptors
        .iter()
        .filter(|descriptor| descriptor.category == OmenaCheckerFindingCategoryV0::Source)
        .count();
    let style_rule_count = descriptors
        .iter()
        .filter(|descriptor| descriptor.category == OmenaCheckerFindingCategoryV0::Style)
        .count();
    let m_tier_rule_count = count_rules_in_tier(&descriptors, OmenaCheckerRuleTierV0::M);
    let s_tier_rule_count = count_rules_in_tier(&descriptors, OmenaCheckerRuleTierV0::S);
    let t_tier_rule_count = count_rules_in_tier(&descriptors, OmenaCheckerRuleTierV0::T);
    let i_tier_rule_count = count_rules_in_tier(&descriptors, OmenaCheckerRuleTierV0::I);

    OmenaCheckerBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-checker.boundary",
        owner_crate: "omena-checker",
        rule_registry_product: "omena-checker.rule-registry",
        bundle_registry_product: "omena-checker.code-bundles",
        rule_count: descriptors.len(),
        bundle_count: list_omena_checker_code_bundles().len(),
        source_rule_count,
        style_rule_count,
        m_tier_rule_count,
        s_tier_rule_count,
        t_tier_rule_count,
        i_tier_rule_count,
        bridge_policy: vec![
            "rustOwnsRuleAndBundleMetadataBeforeRuntimeMigration",
            "typescriptRuntimeMayConsumeTheSameCatalogDuringTransition",
            "diagnosticExecutionMigratesByRuleFamilyAfterRegistryParity",
        ],
        next_migration_targets: vec![
            "dynamicClassDomainRuntime",
            "missingModuleRuntime",
            "styleRecoveryRuntime",
            "unusedSelectorRuntime",
            "configSeverityOverrides",
        ],
    }
}

pub fn evaluate_omena_checker_dynamic_class_domain(
    input: OmenaCheckerDynamicClassDomainInputV0,
) -> OmenaCheckerDynamicClassDomainEvaluationV0 {
    let selector_universe = input
        .selector_universe
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let projection = project_abstract_value_selectors(&input.abstract_value, &selector_universe);
    let finite_values = enumerate_finite_class_values(&input.abstract_value);

    if let Some(finite_values) = finite_values {
        let selector_set = selector_universe.iter().collect::<BTreeSet<_>>();
        let missing_values = finite_values
            .iter()
            .filter(|value| !selector_set.contains(value))
            .cloned()
            .collect::<Vec<_>>();

        if missing_values.is_empty() {
            return dynamic_class_domain_evaluation(
                OmenaCheckerDynamicClassDomainOutcomeV0::Known,
                None,
                projection.selector_names,
                projection.certainty,
                Some(finite_values),
                missing_values,
            );
        }

        return dynamic_class_domain_evaluation(
            OmenaCheckerDynamicClassDomainOutcomeV0::MissingResolvedClassValues,
            Some(OmenaCheckerRuleCodeV0::MissingResolvedClassValues),
            projection.selector_names,
            projection.certainty,
            Some(finite_values),
            missing_values,
        );
    }

    if projection.selector_names.is_empty() {
        return dynamic_class_domain_evaluation(
            OmenaCheckerDynamicClassDomainOutcomeV0::MissingResolvedClassDomain,
            Some(OmenaCheckerRuleCodeV0::MissingResolvedClassDomain),
            projection.selector_names,
            projection.certainty,
            None,
            Vec::new(),
        );
    }

    dynamic_class_domain_evaluation(
        OmenaCheckerDynamicClassDomainOutcomeV0::Known,
        None,
        projection.selector_names,
        projection.certainty,
        None,
        Vec::new(),
    )
}

pub fn evaluate_omena_checker_m_tier_rules(
    input: OmenaCheckerDynamicClassDomainInputV0,
) -> Vec<OmenaCheckerMTierEvaluationV0> {
    let selector_universe = input
        .selector_universe
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let projection = project_abstract_value_selectors(&input.abstract_value, &selector_universe);
    let finite_values = enumerate_finite_class_values(&input.abstract_value);
    let missing_values = finite_values
        .as_ref()
        .map(|finite_values| {
            let selector_set = selector_universe.iter().collect::<BTreeSet<_>>();
            finite_values
                .iter()
                .filter(|value| !selector_set.contains(value))
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut evaluations = Vec::new();

    if projection.selector_names.is_empty() || !missing_values.is_empty() {
        evaluations.push(m_tier_evaluation(
            OmenaCheckerRuleCodeV0::NoUnknownDynamicClass,
            OmenaCheckerSeverityV0::Warning,
            projection.selector_names.clone(),
            projection.certainty,
            finite_values.clone(),
            missing_values.clone(),
            "Dynamic class expression cannot be proven against known CSS Module selectors.",
        ));
    }

    if projection.certainty != SelectorProjectionCertaintyV0::Exact {
        evaluations.push(m_tier_evaluation(
            OmenaCheckerRuleCodeV0::NoImpreciseValue,
            OmenaCheckerSeverityV0::Hint,
            projection.selector_names.clone(),
            projection.certainty,
            finite_values.clone(),
            missing_values.clone(),
            "Class value domain is not exact; downstream rename/refactor should treat it as imprecise.",
        ));
    }

    if !missing_values.is_empty() {
        evaluations.push(m_tier_evaluation(
            OmenaCheckerRuleCodeV0::NoImpossibleSelector,
            OmenaCheckerSeverityV0::Warning,
            projection.selector_names,
            projection.certainty,
            finite_values,
            missing_values,
            "Finite dynamic class values include selectors that cannot match the target CSS Module.",
        ));
    }

    evaluations
}

pub fn evaluate_omena_checker_cascade_rules(
    input: OmenaCheckerCascadeInputV0,
) -> Vec<OmenaCheckerCascadeEvaluationV0> {
    let declarations = input.declarations;
    let custom_properties = input.custom_properties;
    let active_registrations =
        active_custom_property_registrations(input.custom_property_registrations);
    let invalid_custom_properties = custom_properties
        .iter()
        .filter(|property| property.guaranteed_invalid)
        .map(|property| property.name.clone())
        .collect::<BTreeSet<_>>();
    let cyclic_custom_properties = cyclic_custom_property_names(&custom_properties);
    let known_custom_properties = custom_properties
        .iter()
        .map(|property| property.name.clone())
        .collect::<BTreeSet<_>>();
    let mut evaluations = Vec::new();

    for declaration in &declarations {
        if let Some(outranking) = declarations.iter().find(|candidate| {
            candidate.declaration_id != declaration.declaration_id
                && declaration_outranks(candidate, declaration)
        }) {
            evaluations.push(cascade_evaluation(
                OmenaCheckerRuleCodeV0::UnreachableDeclaration,
                OmenaCheckerSeverityV0::Hint,
                vec![
                    declaration.declaration_id.clone(),
                    outranking.declaration_id.clone(),
                ],
                declaration.layer_name.clone(),
                Vec::new(),
                "Declaration is always outranked by another declaration with the same selector and property.",
            ));
        }
    }

    for layer_name in declarations
        .iter()
        .filter_map(|declaration| declaration.layer_name.clone())
        .collect::<BTreeSet<_>>()
    {
        let layer_declarations = declarations
            .iter()
            .filter(|declaration| declaration.layer_name.as_deref() == Some(layer_name.as_str()))
            .collect::<Vec<_>>();
        if !layer_declarations.is_empty()
            && layer_declarations.iter().all(|declaration| {
                declarations.iter().any(|candidate| {
                    candidate.layer_name.as_deref() != Some(layer_name.as_str())
                        && declaration_outranks_by_layer(candidate, declaration)
                })
            })
        {
            evaluations.push(cascade_evaluation(
                OmenaCheckerRuleCodeV0::DeadCascadeLayer,
                OmenaCheckerSeverityV0::Hint,
                layer_declarations
                    .iter()
                    .map(|declaration| declaration.declaration_id.clone())
                    .collect(),
                Some(layer_name),
                Vec::new(),
                "Every declaration in this cascade layer is outranked by another layer.",
            ));
        }
    }

    for declaration in &declarations {
        let risky_refs = declaration
            .var_references
            .iter()
            .filter(|name| {
                !known_custom_properties.contains(*name)
                    || invalid_custom_properties.contains(*name)
                    || cyclic_custom_properties.contains(*name)
            })
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        if !risky_refs.is_empty() {
            evaluations.push(cascade_evaluation(
                OmenaCheckerRuleCodeV0::IacvtProne,
                OmenaCheckerSeverityV0::Warning,
                vec![declaration.declaration_id.clone()],
                declaration.layer_name.clone(),
                risky_refs,
                "Declaration references custom properties that may become invalid at computed-value time.",
            ));
        }
    }

    for declaration in &declarations {
        if !declaration.property.starts_with("--") {
            continue;
        }
        let Some(registration) = active_registrations.get(declaration.property.as_str()) else {
            continue;
        };
        if validate_registered_property_value_v0(
            registration.syntax.as_str(),
            declaration.value.as_str(),
        )
        .class
            == CssValueValidationClassV0::Invalid
        {
            evaluations.push(cascade_evaluation(
                OmenaCheckerRuleCodeV0::RegisteredPropertyTypeMismatch,
                OmenaCheckerSeverityV0::Warning,
                vec![declaration.declaration_id.clone()],
                declaration.layer_name.clone(),
                vec![declaration.property.clone()],
                "Custom property value does not match its same-file @property syntax registration and may be discarded at computed-value time.",
            ));
        }
    }

    for declaration in &declarations {
        if declaration.property.starts_with("--") {
            continue;
        }
        if validate_standard_property_value_v0(
            declaration.property.as_str(),
            declaration.value.as_str(),
        )
        .class
            == CssValueValidationClassV0::Invalid
        {
            evaluations.push(cascade_evaluation(
                OmenaCheckerRuleCodeV0::InvalidPropertyValue,
                OmenaCheckerSeverityV0::Warning,
                vec![declaration.declaration_id.clone()],
                declaration.layer_name.clone(),
                vec![declaration.property.clone()],
                "Property value does not match the pinned property grammar.",
            ));
        }
    }

    if !cyclic_custom_properties.is_empty() {
        evaluations.push(cascade_evaluation(
            OmenaCheckerRuleCodeV0::CircularVar,
            OmenaCheckerSeverityV0::Warning,
            Vec::new(),
            None,
            cyclic_custom_properties.into_iter().collect(),
            "Custom property dependency graph contains a cycle.",
        ));
    }

    for (left_index, left) in declarations.iter().enumerate() {
        for right in declarations.iter().skip(left_index + 1) {
            if declarations_rely_on_source_order_tie(left, right) {
                evaluations.push(cascade_evaluation(
                    OmenaCheckerRuleCodeV0::UnspecifiedCascadeTie,
                    OmenaCheckerSeverityV0::Hint,
                    vec![left.declaration_id.clone(), right.declaration_id.clone()],
                    left.layer_name.clone(),
                    Vec::new(),
                    "Declarations have equal cascade priority except source order; make the intended override explicit.",
                ));
                if designer_intent_source_order_tie_is_inconsistent(left, right) {
                    evaluations.push(cascade_evaluation_with_mechanism_products(
                        OmenaCheckerRuleCodeV0::DesignerIntentInconsistency,
                        OmenaCheckerSeverityV0::Hint,
                        vec![left.declaration_id.clone(), right.declaration_id.clone()],
                        left.layer_name.clone(),
                        Vec::new(),
                        "Variational designer-intent posterior classifies this selector as BEM, but the declarations rely on source order as the final tie-breaker.",
                        vec!["omena-variational.designer-intent-posterior"],
                    ));
                }
            }
        }
    }

    evaluations
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ActiveCustomPropertyRegistrationV0 {
    syntax: String,
}

fn active_custom_property_registrations(
    registrations: Vec<OmenaCheckerCustomPropertyRegistrationInputV0>,
) -> BTreeMap<String, ActiveCustomPropertyRegistrationV0> {
    let mut active_registrations = BTreeMap::new();
    for registration in registrations {
        let Some(syntax) = registration.syntax else {
            continue;
        };
        if registration.inherits.is_none() {
            continue;
        }
        if registered_property_syntax_requires_initial_value_v0(syntax.as_str())
            && registration.initial_value.is_none()
        {
            continue;
        }
        active_registrations.insert(
            registration.name,
            ActiveCustomPropertyRegistrationV0 { syntax },
        );
    }
    active_registrations
}

pub fn evaluate_omena_checker_grn_rules(
    input: OmenaCheckerGrnInputV0,
) -> Vec<OmenaCheckerGrnEvaluationV0> {
    let vertices = input
        .vertices
        .into_iter()
        .map(|vertex| GrnVertexStateV0 {
            vertex: GrnVertexV0 {
                vertex_id: vertex.vertex_id,
                selector: vertex.selector,
                property: vertex.property,
            },
            state: vertex.state.into_cascade_state(),
        })
        .collect::<Vec<_>>();
    let projection = project_grn_outcome(vertices.as_slice());
    let losing_but_eligible_vertex_ids =
        grn_vertex_ids_for_state(vertices.as_slice(), GrnBooleanState::LosingButEligible);
    let inactive_vertex_ids =
        grn_vertex_ids_for_state(vertices.as_slice(), GrnBooleanState::Inactive);
    let mut evaluations = Vec::new();

    if projection.deep_conflict_report.conflicting_vertex_count > 0 {
        evaluations.push(grn_evaluation(
            OmenaCheckerRuleCodeV0::CascadeDeepConflict,
            OmenaCheckerSeverityV0::Warning,
            losing_but_eligible_vertex_ids,
            "GRN cascade projection found losing-but-eligible vertices in a stable conflict basin.",
        ));
    }

    if projection.mode_distribution.inactive_count > 0 {
        evaluations.push(grn_evaluation(
            OmenaCheckerRuleCodeV0::CascadeUnreachableRule,
            OmenaCheckerSeverityV0::Hint,
            inactive_vertex_ids,
            "GRN cascade projection found inactive vertices that remain unreachable in the current basin.",
        ));
    }

    evaluations
}

pub fn evaluate_omena_checker_smt_rules(
    input: OmenaCheckerSmtInputV0,
) -> Vec<OmenaCheckerSmtEvaluationV0> {
    #[cfg(feature = "smt-z3")]
    let backend = Z3SmtBackendV0::default();
    #[cfg(not(feature = "smt-z3"))]
    let backend = StubSmtBackendV0::default();
    let mut evaluations = Vec::new();

    for obligation in input.obligations {
        let canonical_input = canonical_smt_input_v0(
            obligation.obligation_id,
            checker_smt_l1_primitive_name(obligation.l1_primitive.as_str()),
            obligation.canonical_terms,
        );
        let check = backend.check_canonical_input_v0(&canonical_input);
        if check.sat_result == SmtBackendSatResultV0::Unsat {
            evaluations.push(smt_evaluation(
                canonical_input.obligation_id,
                smt_backend_kind_name(backend.backend_kind()),
                smt_sat_result_name(check.sat_result),
                "SMT backend rejected the cascade proof obligation.",
                smt_backend_product_name(backend.backend_kind()),
            ));
        }
    }

    evaluations
}

pub fn evaluate_omena_checker_smt_layer_inversion_rules(
    input: OmenaCheckerSmtLayerInversionInputV0,
) -> Vec<OmenaCheckerSmtEvaluationV0> {
    #[cfg(not(feature = "smt-z3"))]
    {
        let _ = input;
        Vec::new()
    }

    #[cfg(feature = "smt-z3")]
    {
        let backend = Z3SmtBackendV0::default();
        let mut evaluations = Vec::new();

        for obligation in input.obligations {
            let declarations = obligation
                .declarations
                .into_iter()
                .map(|declaration| {
                    layer_inversion_declaration_v0(
                        declaration.declaration_id,
                        declaration.layer_rank,
                        declaration.source_order,
                    )
                })
                .collect::<Vec<_>>();
            let verdict = smt_check_layer_flatten_inversion_v0(&declarations, &backend);
            if verdict.verdict == SmtVerdictV0::Rejected {
                let mut evaluation = smt_evaluation(
                    obligation.obligation_id,
                    smt_backend_kind_name(verdict.backend),
                    smt_sat_result_name(verdict.sat_result),
                    "Opt-in z3 SMT backend found a layer-flatten inversion.",
                    smt_backend_product_name(verdict.backend),
                );
                evaluation
                    .mechanism_products
                    .push("omena-smt.layer-flatten-inversion");
                evaluations.push(evaluation);
            }
        }

        evaluations
    }
}

pub fn evaluate_omena_checker_mdl_rules(
    input: OmenaCheckerMdlInputV0,
) -> Vec<OmenaCheckerMdlEvaluationV0> {
    input
        .summaries
        .into_iter()
        .filter(|summary| summary.total_bits > summary.budget_bits)
        .map(|summary| {
            mdl_evaluation(
                summary.source_uri,
                summary.total_bits,
                summary.budget_bits,
                "Design-system minimum-description length exceeds the configured budget.",
            )
        })
        .collect()
}

pub fn evaluate_omena_checker_streaming_ifds_rules(
    input: OmenaCheckerStreamingIfdsInputV0,
) -> Vec<OmenaCheckerStreamingIfdsEvaluationV0> {
    input
        .reports
        .into_iter()
        .filter(|report| !report.incremental_precision_parity_with_batch)
        .map(|report| {
            streaming_ifds_evaluation(
                report.report_id,
                report.incremental_precision_parity_with_batch,
                report.reachability_fallback_applied,
                report.fact_fallback_applied,
                "Streaming IFDS analysis failed exact batch precision parity.",
            )
        })
        .collect()
}

pub fn evaluate_omena_checker_rg_flow_rules(
    input: OmenaCheckerRgFlowInputV0,
) -> Vec<OmenaCheckerRgFlowEvaluationV0> {
    input
        .flows
        .into_iter()
        .filter_map(|flow| {
            let before = checker_rg_flow_coupling_space(flow.before);
            let after = checker_rg_flow_coupling_space(flow.after);
            let spectrum = estimate_coupling_jacobian_spectrum_v0(&before, &after);
            (spectrum.spectral_radius > 1.0).then(|| {
                rg_flow_evaluation(
                    flow.workspace_path,
                    spectrum.spectral_radius,
                    spectrum.eigenvalues,
                    "RG-flow opt-in deep-analysis hint found a relevant coupling operator; review custom-property fixed-point sensitivity. This is not a default product decision mechanism.",
                )
            })
        })
        .collect()
}

pub fn evaluate_omena_checker_replica_ensemble_rules(
    input: OmenaCheckerReplicaEnsembleInputV0,
) -> Vec<OmenaCheckerReplicaEnsembleEvaluationV0> {
    input
        .reports
        .into_iter()
        .filter(|report| {
            report.recommendation != "noActionNeeded"
                || report.top_disagreement_pair_count > 0
                || report.mean_q < 1.0
        })
        .map(replica_ensemble_evaluation)
        .collect()
}

pub fn evaluate_omena_checker_categorical_rules(
    input: OmenaCheckerCategoricalInputV0,
) -> Vec<OmenaCheckerCategoricalEvaluationV0> {
    input
        .mappings
        .into_iter()
        .filter_map(|mapping| {
            let object_role_pairs = mapping
                .primitive_role_pairs
                .iter()
                .map(|pair| (pair.primitive_name.clone(), pair.categorical_role.clone()))
                .collect::<Vec<_>>();
            let functor = apply_cascade_role_mapping_functor_v0(
                &format!("checker-cascade-role-functor:{}", mapping.mapping_id),
                "omena-checker.categorical-cascade-role-functor",
                &object_role_pairs,
            );
            (!functor.accepted).then(|| categorical_evaluation(mapping.mapping_id, &functor))
        })
        .collect()
}

fn categorical_evaluation(
    mapping_id: String,
    functor: &CascadeFunctorApplicationV0,
) -> OmenaCheckerCategoricalEvaluationV0 {
    OmenaCheckerCategoricalEvaluationV0 {
        rule_code: OmenaCheckerRuleCodeV0::CategoricalCascadeEvidenceInconsistency,
        rule_code_name: OmenaCheckerRuleCodeV0::CategoricalCascadeEvidenceInconsistency.as_str(),
        severity: OmenaCheckerSeverityV0::Hint,
        severity_name: OmenaCheckerSeverityV0::Hint.as_str(),
        mapping_id,
        object_mapping_count: functor.object_mapping_count,
        morphism_mapping_count: functor.morphism_mapping_count,
        identity_preserved: functor.identity_preserved,
        composition_preserved: functor.composition_preserved,
        functor_accepted: functor.accepted,
        message:
            "Categorical cascade-role functor obligation failed: identity or composition is not \
             preserved, so the cascade primitive-to-role mapping is not functorial."
                .to_string(),
        mechanism_products: vec!["omena-categorical.cascade-primitive-role-functor"],
    }
}

fn dynamic_class_domain_evaluation(
    outcome: OmenaCheckerDynamicClassDomainOutcomeV0,
    rule_code: Option<OmenaCheckerRuleCodeV0>,
    selector_names: Vec<String>,
    selector_certainty: SelectorProjectionCertaintyV0,
    finite_values: Option<Vec<String>>,
    missing_values: Vec<String>,
) -> OmenaCheckerDynamicClassDomainEvaluationV0 {
    OmenaCheckerDynamicClassDomainEvaluationV0 {
        outcome,
        outcome_name: outcome.as_str(),
        rule_code,
        rule_code_name: rule_code.map(OmenaCheckerRuleCodeV0::as_str),
        selector_names,
        selector_certainty,
        finite_values,
        missing_values,
    }
}

fn m_tier_evaluation(
    rule_code: OmenaCheckerRuleCodeV0,
    severity: OmenaCheckerSeverityV0,
    selector_names: Vec<String>,
    selector_certainty: SelectorProjectionCertaintyV0,
    finite_values: Option<Vec<String>>,
    missing_values: Vec<String>,
    message: &'static str,
) -> OmenaCheckerMTierEvaluationV0 {
    OmenaCheckerMTierEvaluationV0 {
        rule_code,
        rule_code_name: rule_code.as_str(),
        severity,
        severity_name: severity.as_str(),
        selector_names,
        selector_certainty,
        finite_values,
        missing_values,
        message: message.to_string(),
    }
}

fn cascade_evaluation(
    rule_code: OmenaCheckerRuleCodeV0,
    severity: OmenaCheckerSeverityV0,
    declaration_ids: Vec<String>,
    layer_name: Option<String>,
    custom_property_names: Vec<String>,
    message: &'static str,
) -> OmenaCheckerCascadeEvaluationV0 {
    cascade_evaluation_with_mechanism_products(
        rule_code,
        severity,
        declaration_ids,
        layer_name,
        custom_property_names,
        message,
        Vec::new(),
    )
}

fn cascade_evaluation_with_mechanism_products(
    rule_code: OmenaCheckerRuleCodeV0,
    severity: OmenaCheckerSeverityV0,
    declaration_ids: Vec<String>,
    layer_name: Option<String>,
    custom_property_names: Vec<String>,
    message: &'static str,
    mechanism_products: Vec<&'static str>,
) -> OmenaCheckerCascadeEvaluationV0 {
    OmenaCheckerCascadeEvaluationV0 {
        rule_code,
        rule_code_name: rule_code.as_str(),
        severity,
        severity_name: severity.as_str(),
        declaration_ids,
        layer_name,
        custom_property_names,
        message: message.to_string(),
        mechanism_products,
    }
}

fn grn_evaluation(
    rule_code: OmenaCheckerRuleCodeV0,
    severity: OmenaCheckerSeverityV0,
    vertex_ids: Vec<String>,
    message: &'static str,
) -> OmenaCheckerGrnEvaluationV0 {
    OmenaCheckerGrnEvaluationV0 {
        rule_code,
        rule_code_name: rule_code.as_str(),
        severity,
        severity_name: severity.as_str(),
        vertex_ids,
        message: message.to_string(),
        mechanism_products: vec!["omena-cascade.grn-outcome-projection"],
    }
}

fn grn_vertex_ids_for_state(vertices: &[GrnVertexStateV0], state: GrnBooleanState) -> Vec<String> {
    vertices
        .iter()
        .filter(|vertex| vertex.state == state)
        .map(|vertex| vertex.vertex.vertex_id.clone())
        .collect()
}

fn smt_evaluation(
    obligation_id: String,
    backend_kind_name: &'static str,
    sat_result_name: &'static str,
    message: &'static str,
    backend_product_name: &'static str,
) -> OmenaCheckerSmtEvaluationV0 {
    OmenaCheckerSmtEvaluationV0 {
        rule_code: OmenaCheckerRuleCodeV0::CascadeSMTViolation,
        rule_code_name: OmenaCheckerRuleCodeV0::CascadeSMTViolation.as_str(),
        severity: OmenaCheckerSeverityV0::Warning,
        severity_name: OmenaCheckerSeverityV0::Warning.as_str(),
        obligation_id,
        backend_kind_name,
        sat_result_name,
        message: message.to_string(),
        mechanism_products: vec!["omena-smt.backend-check", backend_product_name],
    }
}

fn checker_smt_l1_primitive_name(value: &str) -> &'static str {
    match value {
        "boxShorthandCombination" => "boxShorthandCombination",
        "scopeFlattenCandidate" => "scopeFlattenCandidate",
        "layerFlattenCandidate" => "layerFlattenCandidate",
        "staticSupportsCondition" => "staticSupportsCondition",
        _ => "checkerSmtObligation",
    }
}

#[cfg(not(feature = "smt-z3"))]
fn smt_backend_kind_name(kind: SmtBackendKindV0) -> &'static str {
    match kind {
        SmtBackendKindV0::Stub => "stub",
        SmtBackendKindV0::Z3 => "z3",
    }
}

#[cfg(feature = "smt-z3")]
fn smt_backend_kind_name(_kind: SmtBackendKindV0) -> &'static str {
    "z3"
}

#[cfg(not(feature = "smt-z3"))]
fn smt_backend_product_name(kind: SmtBackendKindV0) -> &'static str {
    match kind {
        SmtBackendKindV0::Stub => "omena-smt.backend.stub",
        SmtBackendKindV0::Z3 => "omena-smt.backend.z3",
    }
}

#[cfg(feature = "smt-z3")]
fn smt_backend_product_name(_kind: SmtBackendKindV0) -> &'static str {
    "omena-smt.backend.z3"
}

fn smt_sat_result_name(result: SmtBackendSatResultV0) -> &'static str {
    match result {
        SmtBackendSatResultV0::Sat => "sat",
        SmtBackendSatResultV0::Unsat => "unsat",
        SmtBackendSatResultV0::Unknown => "unknown",
    }
}

fn mdl_evaluation(
    source_uri: String,
    total_bits: f64,
    budget_bits: f64,
    message: &'static str,
) -> OmenaCheckerMdlEvaluationV0 {
    OmenaCheckerMdlEvaluationV0 {
        rule_code: OmenaCheckerRuleCodeV0::DesignSystemMdlBudget,
        rule_code_name: OmenaCheckerRuleCodeV0::DesignSystemMdlBudget.as_str(),
        severity: OmenaCheckerSeverityV0::Hint,
        severity_name: OmenaCheckerSeverityV0::Hint.as_str(),
        source_uri,
        total_bits,
        budget_bits,
        message: message.to_string(),
        mechanism_products: vec!["omena-query.design-system-minimum-description"],
    }
}

fn streaming_ifds_evaluation(
    report_id: String,
    incremental_precision_parity_with_batch: bool,
    reachability_fallback_applied: bool,
    fact_fallback_applied: bool,
    message: &'static str,
) -> OmenaCheckerStreamingIfdsEvaluationV0 {
    OmenaCheckerStreamingIfdsEvaluationV0 {
        rule_code: OmenaCheckerRuleCodeV0::StreamingIfdsPrecisionParity,
        rule_code_name: OmenaCheckerRuleCodeV0::StreamingIfdsPrecisionParity.as_str(),
        severity: OmenaCheckerSeverityV0::Hint,
        severity_name: OmenaCheckerSeverityV0::Hint.as_str(),
        report_id,
        incremental_precision_parity_with_batch,
        reachability_fallback_applied,
        fact_fallback_applied,
        message: message.to_string(),
        mechanism_products: vec!["omena-streaming-ifds.analysis-report"],
    }
}

fn rg_flow_evaluation(
    workspace_path: String,
    spectral_radius: f64,
    eigenvalues: Vec<f64>,
    message: &'static str,
) -> OmenaCheckerRgFlowEvaluationV0 {
    OmenaCheckerRgFlowEvaluationV0 {
        rule_code: OmenaCheckerRuleCodeV0::RgFlowRelevantOperator,
        rule_code_name: OmenaCheckerRuleCodeV0::RgFlowRelevantOperator.as_str(),
        severity: OmenaCheckerSeverityV0::Hint,
        severity_name: OmenaCheckerSeverityV0::Hint.as_str(),
        workspace_path,
        spectral_radius,
        eigenvalues,
        mechanism_scope: RG_FLOW_MECHANISM_SCOPE_V0,
        product_surface: RG_FLOW_PRODUCT_SURFACE_V0,
        default_product_decision_mechanism: RG_FLOW_DEFAULT_PRODUCT_DECISION_MECHANISM_V0,
        message: message.to_string(),
        mechanism_products: vec!["omena-rg-flow.coupling-jacobian-spectrum"],
    }
}

fn replica_ensemble_evaluation(
    report: OmenaCheckerReplicaEnsembleReportInputV0,
) -> OmenaCheckerReplicaEnsembleEvaluationV0 {
    OmenaCheckerReplicaEnsembleEvaluationV0 {
        rule_code: OmenaCheckerRuleCodeV0::ReplicaEnsembleInconsistency,
        rule_code_name: OmenaCheckerRuleCodeV0::ReplicaEnsembleInconsistency.as_str(),
        severity: OmenaCheckerSeverityV0::Hint,
        severity_name: OmenaCheckerSeverityV0::Hint.as_str(),
        workspace_root: report.workspace_root,
        recommendation: report.recommendation,
        mean_q: report.mean_q,
        variance_q: report.variance_q,
        top_disagreement_pair_count: report.top_disagreement_pair_count,
        mechanism_scope: report.mechanism_scope,
        product_surface: report.product_surface,
        default_product_decision_mechanism: report.default_product_decision_mechanism,
        message: "Replica-ensemble cross-file consistency hint found inconsistent cascade outcomes; this is not a default product decision mechanism.".to_string(),
        mechanism_products: vec!["omena-ensemble.cross-file-inconsistency-report"],
    }
}

fn checker_rg_flow_coupling_space(
    input: OmenaCheckerRgFlowCouplingSpaceInputV0,
) -> omena_product_hints::CouplingSpaceV0 {
    coupling_space(input.k_env, input.k_decl, input.k_cycle, input.k_dirty)
}

fn designer_intent_source_order_tie_is_inconsistent(
    left: &OmenaCheckerCascadeDeclarationInputV0,
    right: &OmenaCheckerCascadeDeclarationInputV0,
) -> bool {
    let posterior = infer_designer_intent_posterior_v0(designer_intent_posterior_input_v0(
        left.selector.as_str().to_string(),
        2,
        1,
        left.var_references.len() + right.var_references.len(),
    ));
    matches!(
        dominant_designer_intent_v0(&posterior),
        Some(PatternIntentV0::Bem)
    )
}

fn declaration_outranks(
    candidate: &OmenaCheckerCascadeDeclarationInputV0,
    declaration: &OmenaCheckerCascadeDeclarationInputV0,
) -> bool {
    if !declarations_share_cascade_context(candidate, declaration) {
        return false;
    }
    if is_progressive_enhancement_pair(candidate, declaration) {
        return false;
    }
    if candidate.important != declaration.important {
        return candidate.important;
    }
    if declaration_outranks_by_layer(candidate, declaration) {
        return true;
    }
    candidate.layer_order == declaration.layer_order
        && candidate.source_order > declaration.source_order
}

fn declaration_outranks_by_layer(
    candidate: &OmenaCheckerCascadeDeclarationInputV0,
    declaration: &OmenaCheckerCascadeDeclarationInputV0,
) -> bool {
    if !declarations_share_cascade_context(candidate, declaration)
        || candidate.important != declaration.important
        || is_progressive_enhancement_pair(candidate, declaration)
    {
        return false;
    }
    match (
        candidate.layer_order,
        declaration.layer_order,
        declaration.important,
    ) {
        (None, Some(_), false) => true,
        (Some(_), None, false) => false,
        (Some(_), None, true) => true,
        (None, Some(_), true) => false,
        (Some(candidate_layer), Some(declaration_layer), false) => {
            candidate_layer > declaration_layer
        }
        (Some(candidate_layer), Some(declaration_layer), true) => {
            candidate_layer < declaration_layer
        }
        _ => false,
    }
}

fn declarations_rely_on_source_order_tie(
    left: &OmenaCheckerCascadeDeclarationInputV0,
    right: &OmenaCheckerCascadeDeclarationInputV0,
) -> bool {
    declarations_share_cascade_context(left, right)
        && left.value != right.value
        && !is_progressive_enhancement_pair(left, right)
        && left.important == right.important
        && left.layer_order == right.layer_order
        && left.layer_name == right.layer_name
        && left.source_order != right.source_order
}

/// Returns true when two declarations of the same property are a deliberate
/// vendor-prefix progressive-enhancement fallback rather than an accidental
/// override (e.g. `-webkit-linear-gradient(...)` then `linear-gradient(...)`,
/// or `display: flex` then `display: -ms-flexbox`).
///
/// The check is restricted to the *leading token* of each value (the
/// value-function name or keyword, i.e. the text before the first `(`,
/// whitespace, or comma). At least one side's leading token must begin with a
/// known vendor prefix and the two values must actually differ. This avoids
/// matching a `-webkit-`-containing substring buried mid-expression and keeps
/// genuine accidental duplicates (`color: red; color: blue`, neither prefixed)
/// firing as before.
fn is_progressive_enhancement_pair(
    left: &OmenaCheckerCascadeDeclarationInputV0,
    right: &OmenaCheckerCascadeDeclarationInputV0,
) -> bool {
    if left.property != right.property || left.value == right.value {
        return false;
    }
    let left_token = leading_value_token(&left.value);
    let right_token = leading_value_token(&right.value);
    value_token_has_vendor_prefix(left_token) || value_token_has_vendor_prefix(right_token)
}

/// Extracts the leading token of a CSS value: the slice before the first `(`,
/// ASCII whitespace, or `,`. For `-webkit-linear-gradient(top, #fff, #000)`
/// this is `-webkit-linear-gradient`; for `-ms-flexbox` it is the whole value.
fn leading_value_token(value: &str) -> &str {
    let trimmed = value.trim_start();
    let end = trimmed
        .find(|character: char| character == '(' || character == ',' || character.is_whitespace())
        .unwrap_or(trimmed.len());
    &trimmed[..end]
}

/// Returns true when a value's leading token begins with a known CSS vendor
/// prefix. Matching is case-insensitive on the prefix only.
fn value_token_has_vendor_prefix(token: &str) -> bool {
    const VENDOR_PREFIXES: [&str; 4] = ["-webkit-", "-moz-", "-ms-", "-o-"];
    let lowered = token.to_ascii_lowercase();
    VENDOR_PREFIXES
        .iter()
        .any(|prefix| lowered.starts_with(prefix))
}

fn declarations_share_cascade_context(
    left: &OmenaCheckerCascadeDeclarationInputV0,
    right: &OmenaCheckerCascadeDeclarationInputV0,
) -> bool {
    left.selector.as_str() == right.selector.as_str()
        && left.property == right.property
        && left.condition_context == right.condition_context
}

fn cyclic_custom_property_names(
    custom_properties: &[OmenaCheckerCustomPropertyInputV0],
) -> BTreeSet<String> {
    let graph = custom_properties
        .iter()
        .map(|property| {
            (
                property.name.clone(),
                property
                    .dependencies
                    .iter()
                    .filter(|dependency| {
                        custom_properties
                            .iter()
                            .any(|property| property.name == **dependency)
                    })
                    .cloned()
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    graph
        .keys()
        .filter(|name| custom_property_reaches_name(name, name, &graph, &mut BTreeSet::new()))
        .cloned()
        .collect()
}

fn custom_property_reaches_name(
    start: &str,
    current: &str,
    graph: &BTreeMap<String, Vec<String>>,
    visited: &mut BTreeSet<String>,
) -> bool {
    let Some(dependencies) = graph.get(current) else {
        return false;
    };
    for dependency in dependencies {
        if dependency == start {
            return true;
        }
        if visited.insert(dependency.clone())
            && custom_property_reaches_name(start, dependency, graph, visited)
        {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use omena_abstract_value::{
        CompositeClassValueInputV0, composite_class_value, finite_set_class_value,
        prefix_class_value,
    };

    use super::*;

    #[test]
    fn lists_current_checker_registry_in_stable_ts_order() {
        assert_eq!(
            list_omena_checker_rule_code_names(),
            vec![
                "no-unknown-dynamic-class",
                "no-imprecise-value",
                "no-impossible-selector",
                "missing-module",
                "missing-static-class",
                "missing-template-prefix",
                "missing-resolved-class-values",
                "missing-resolved-class-domain",
                "unused-selector",
                "missing-composed-module",
                "missing-composed-selector",
                "missing-value-module",
                "missing-imported-value",
                "missing-keyframes",
                "missing-custom-property",
                "missing-sass-symbol",
                "unreachable-declaration",
                "dead-cascade-layer",
                "iacvt-prone",
                "circular-var",
                "unspecified-cascade-tie",
                "designer-intent-inconsistency",
                "cascade.smt-violation",
                "design-system-mdl-budget",
                "streaming-ifds-precision-parity",
                "rg-flow-relevant-operator",
                "replica-ensemble-inconsistency",
                "cascade.deep-conflict",
                "cascade.unreachable-rule",
                "categorical-cascade-evidence-inconsistency",
                "registered-property-type-mismatch",
                "invalid-property-value",
            ],
        );
    }

    #[test]
    fn descriptors_have_required_metadata_without_duplicate_codes() {
        let descriptors = list_omena_checker_rule_descriptors();
        let mut codes = BTreeSet::new();
        let mut ordinals = BTreeSet::new();

        for descriptor in &descriptors {
            assert!(codes.insert(descriptor.code_name));
            assert!(ordinals.insert(descriptor.ordinal));
            assert!(descriptor.description.len() > 20);
            assert!(!descriptor.preset_names.is_empty());
            assert_eq!(descriptor.code.as_str(), descriptor.code_name);
            assert_eq!(descriptor.category.as_str(), descriptor.category_name);
            assert_eq!(descriptor.tier.as_str(), descriptor.tier_name);
            assert_eq!(
                descriptor.default_severity.as_str(),
                descriptor.default_severity_name,
            );
            assert_eq!(descriptor.fixability.as_str(), descriptor.fixability_name);
            assert_eq!(
                get_omena_checker_rule_descriptor(descriptor.code),
                Some(descriptor.clone()),
            );
            assert!(is_omena_checker_rule_code(descriptor.code_name));
        }

        assert_eq!(descriptors.len(), codes.len());
        assert_eq!(descriptors.len(), ordinals.len());
        assert_eq!(
            ordinals.into_iter().collect::<Vec<_>>(),
            (1..=32).collect::<Vec<_>>()
        );
        assert!(!is_omena_checker_rule_code("not-a-rule"));
    }

    #[test]
    fn code_bundles_reference_registered_codes() {
        let registered = list_omena_checker_rule_codes()
            .into_iter()
            .collect::<BTreeSet<_>>();

        for bundle in list_omena_checker_code_bundles() {
            assert!(!bundle.codes.is_empty());
            assert_eq!(bundle.bundle.as_str(), bundle.bundle_name);
            assert!(bundle.codes.iter().all(|code| registered.contains(code)));
            assert_eq!(bundle.codes.len(), bundle.code_names.len());
        }
    }

    #[test]
    fn boundary_declares_registry_owner_and_transition_policy() {
        let summary = summarize_omena_checker_boundary();

        assert_eq!(summary.product, "omena-checker.boundary");
        assert_eq!(summary.owner_crate, "omena-checker");
        assert_eq!(summary.rule_registry_product, "omena-checker.rule-registry");
        assert_eq!(
            summary.bundle_registry_product,
            "omena-checker.code-bundles"
        );
        assert_eq!(summary.rule_count, 32);
        assert_eq!(summary.source_rule_count, 8);
        assert_eq!(summary.style_rule_count, 24);
        assert_eq!(summary.m_tier_rule_count, 3);
        assert_eq!(summary.s_tier_rule_count, 7);
        assert_eq!(summary.t_tier_rule_count, 15);
        assert_eq!(summary.i_tier_rule_count, 7);
        assert_eq!(summary.bundle_count, 5);
        assert!(
            summary
                .bridge_policy
                .contains(&"rustOwnsRuleAndBundleMetadataBeforeRuntimeMigration"),
        );
        assert!(
            summary
                .next_migration_targets
                .contains(&"dynamicClassDomainRuntime"),
        );
    }

    #[test]
    fn evaluates_finite_dynamic_class_domains() {
        let evaluation =
            evaluate_omena_checker_dynamic_class_domain(OmenaCheckerDynamicClassDomainInputV0 {
                abstract_value: finite_set_class_value(["btn-primary", "btn-missing"]),
                selector_universe: vec!["btn-primary".to_string(), "card".to_string()],
            });

        assert_eq!(
            evaluation.outcome,
            OmenaCheckerDynamicClassDomainOutcomeV0::MissingResolvedClassValues
        );
        assert_eq!(
            evaluation.rule_code,
            Some(OmenaCheckerRuleCodeV0::MissingResolvedClassValues)
        );
        assert_eq!(evaluation.missing_values, vec!["btn-missing"]);
    }

    #[test]
    fn evaluates_constrained_dynamic_class_domains_with_abstract_value_projection() {
        let evaluation =
            evaluate_omena_checker_dynamic_class_domain(OmenaCheckerDynamicClassDomainInputV0 {
                abstract_value: composite_class_value(CompositeClassValueInputV0 {
                    prefix: Some("btn-".to_string()),
                    suffix: Some("-active".to_string()),
                    min_length: Some(16),
                    must_chars: "-abceintv".to_string(),
                    may_chars: "-abceinprtv".to_string(),
                    may_include_other_chars: false,
                    provenance: None,
                }),
                selector_universe: vec!["btn-primary".to_string(), "card".to_string()],
            });

        assert_eq!(
            evaluation.outcome,
            OmenaCheckerDynamicClassDomainOutcomeV0::MissingResolvedClassDomain
        );
        assert_eq!(
            evaluation.rule_code,
            Some(OmenaCheckerRuleCodeV0::MissingResolvedClassDomain)
        );
        assert!(evaluation.selector_names.is_empty());
    }

    #[test]
    fn lists_m_tier_rule_codes() {
        assert_eq!(
            list_omena_checker_m_tier_rule_code_names(),
            vec![
                "no-unknown-dynamic-class",
                "no-imprecise-value",
                "no-impossible-selector",
            ]
        );
    }

    #[test]
    fn lists_s_and_t_tier_rule_codes() {
        assert_eq!(
            list_omena_checker_s_tier_rule_code_names(),
            vec![
                "missing-module",
                "missing-static-class",
                "missing-template-prefix",
                "missing-resolved-class-values",
                "missing-resolved-class-domain",
                "cascade.smt-violation",
                "cascade.deep-conflict",
            ]
        );
        assert_eq!(
            list_omena_checker_t_tier_rule_code_names(),
            vec![
                "unused-selector",
                "missing-composed-module",
                "missing-composed-selector",
                "missing-value-module",
                "missing-imported-value",
                "missing-keyframes",
                "missing-custom-property",
                "missing-sass-symbol",
                "unreachable-declaration",
                "dead-cascade-layer",
                "iacvt-prone",
                "circular-var",
                "registered-property-type-mismatch",
                "unspecified-cascade-tie",
            ]
        );
    }

    #[test]
    fn lists_i_tier_rule_codes_for_m4_advisories() {
        assert_eq!(
            list_omena_checker_i_tier_rule_code_names(),
            vec![
                "designer-intent-inconsistency",
                "design-system-mdl-budget",
                "streaming-ifds-precision-parity",
                "rg-flow-relevant-operator",
                "replica-ensemble-inconsistency",
                "cascade.unreachable-rule",
                "categorical-cascade-evidence-inconsistency",
            ]
        );
    }

    #[test]
    fn m4_gamma_rule_ordinals_and_smt_backend_tiers_are_explicit() {
        assert_eq!(
            get_omena_checker_rule_descriptor(OmenaCheckerRuleCodeV0::DesignerIntentInconsistency)
                .map(|descriptor| (descriptor.ordinal, descriptor.tier)),
            Some((22, OmenaCheckerRuleTierV0::I))
        );
        assert_eq!(
            get_omena_checker_rule_descriptor(OmenaCheckerRuleCodeV0::CascadeSMTViolation)
                .map(|descriptor| (descriptor.ordinal, descriptor.tier)),
            Some((23, OmenaCheckerRuleTierV0::S))
        );
        assert_eq!(
            get_omena_checker_rule_descriptor(OmenaCheckerRuleCodeV0::DesignSystemMdlBudget)
                .map(|descriptor| descriptor.ordinal),
            Some(24)
        );
        assert_eq!(
            get_omena_checker_rule_descriptor(OmenaCheckerRuleCodeV0::StreamingIfdsPrecisionParity)
                .map(|descriptor| descriptor.ordinal),
            Some(25)
        );
        assert_eq!(
            get_omena_checker_rule_descriptor(OmenaCheckerRuleCodeV0::RgFlowRelevantOperator)
                .map(|descriptor| descriptor.ordinal),
            Some(28)
        );
        assert_eq!(
            get_omena_checker_rule_descriptor(OmenaCheckerRuleCodeV0::ReplicaEnsembleInconsistency)
                .map(|descriptor| descriptor.ordinal),
            Some(29)
        );
        assert_eq!(
            resolve_omena_checker_rule_tier_for_smt_backend(
                OmenaCheckerRuleCodeV0::CascadeSMTViolation,
                OmenaCheckerSmtBackendKindV0::Stub,
            ),
            OmenaCheckerRuleTierV0::I
        );
        assert_eq!(
            resolve_omena_checker_rule_tier_for_smt_backend(
                OmenaCheckerRuleCodeV0::CascadeSMTViolation,
                OmenaCheckerSmtBackendKindV0::Z3,
            ),
            OmenaCheckerRuleTierV0::S
        );
        assert_eq!(
            active_omena_checker_smt_backend_kind_v0(),
            if cfg!(feature = "smt-z3") {
                OmenaCheckerSmtBackendKindV0::Z3
            } else {
                OmenaCheckerSmtBackendKindV0::Stub
            }
        );
        assert_eq!(
            active_omena_checker_smt_backend_kind_name_v0(),
            if cfg!(feature = "smt-z3") {
                "z3"
            } else {
                "stub"
            }
        );
        assert_eq!(
            active_omena_checker_smt_product_scope_v0(),
            if cfg!(feature = "smt-z3") {
                "explicitOptInZ3SolverBackedProductGate"
            } else {
                "defaultSolverFreeStubProductGate"
            }
        );
        assert_eq!(
            active_omena_checker_smt_solver_backed_v0(),
            cfg!(feature = "smt-z3")
        );
    }

    #[test]
    fn evaluates_m_tier_unknown_and_impossible_dynamic_classes() {
        let evaluations =
            evaluate_omena_checker_m_tier_rules(OmenaCheckerDynamicClassDomainInputV0 {
                abstract_value: finite_set_class_value(["btn-primary", "btn-missing"]),
                selector_universe: vec!["btn-primary".to_string(), "card".to_string()],
            });
        let rule_names = evaluations
            .iter()
            .map(|evaluation| evaluation.rule_code_name)
            .collect::<Vec<_>>();

        assert_eq!(
            rule_names,
            vec![
                "no-unknown-dynamic-class",
                "no-imprecise-value",
                "no-impossible-selector",
            ]
        );
        assert_eq!(evaluations[0].missing_values, vec!["btn-missing"]);
        assert_eq!(
            evaluations[1].selector_certainty,
            SelectorProjectionCertaintyV0::Inferred
        );
    }

    #[test]
    fn evaluates_m_tier_imprecise_domains_without_unknown_values() {
        let evaluations =
            evaluate_omena_checker_m_tier_rules(OmenaCheckerDynamicClassDomainInputV0 {
                abstract_value: prefix_class_value("btn-", None),
                selector_universe: vec!["btn-primary".to_string(), "card".to_string()],
            });

        assert_eq!(evaluations.len(), 1);
        assert_eq!(
            evaluations[0].rule_code,
            OmenaCheckerRuleCodeV0::NoImpreciseValue
        );
        assert_eq!(evaluations[0].severity, OmenaCheckerSeverityV0::Hint);
    }

    #[test]
    fn evaluates_cascade_aware_rule_family() {
        let evaluations = evaluate_omena_checker_cascade_rules(OmenaCheckerCascadeInputV0 {
            declarations: vec![
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "base-color",
                    selector: ".btn",
                    property: "color",
                    value: "red",
                    source_order: 1,
                    condition_context: &[],
                    layer_name: Some("base"),
                    layer_order: Some(0),
                    important: false,
                    var_references: &[],
                }),
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "override-color",
                    selector: ".btn",
                    property: "color",
                    value: "blue",
                    source_order: 2,
                    condition_context: &[],
                    layer_name: Some("overrides"),
                    layer_order: Some(1),
                    important: false,
                    var_references: &[],
                }),
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "gap-use",
                    selector: ".card",
                    property: "margin",
                    value: "var(--gap)",
                    source_order: 3,
                    condition_context: &[],
                    layer_name: Some("components"),
                    layer_order: Some(1),
                    important: false,
                    var_references: &["--gap"],
                }),
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "tie-a",
                    selector: ".tie",
                    property: "color",
                    value: "red",
                    source_order: 4,
                    condition_context: &[],
                    layer_name: Some("utilities"),
                    layer_order: Some(2),
                    important: false,
                    var_references: &[],
                }),
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "tie-b",
                    selector: ".tie",
                    property: "color",
                    value: "green",
                    source_order: 5,
                    condition_context: &[],
                    layer_name: Some("utilities"),
                    layer_order: Some(2),
                    important: false,
                    var_references: &[],
                }),
            ],
            custom_properties: vec![
                OmenaCheckerCustomPropertyInputV0 {
                    name: "--gap".to_string(),
                    dependencies: Vec::new(),
                    guaranteed_invalid: true,
                },
                OmenaCheckerCustomPropertyInputV0 {
                    name: "--a".to_string(),
                    dependencies: vec!["--b".to_string()],
                    guaranteed_invalid: false,
                },
                OmenaCheckerCustomPropertyInputV0 {
                    name: "--b".to_string(),
                    dependencies: vec!["--a".to_string()],
                    guaranteed_invalid: false,
                },
            ],
            custom_property_registrations: Vec::new(),
        });
        let rule_names = evaluations
            .iter()
            .map(|evaluation| evaluation.rule_code_name)
            .collect::<BTreeSet<_>>();

        assert!(rule_names.contains("unreachable-declaration"));
        assert!(rule_names.contains("dead-cascade-layer"));
        assert!(rule_names.contains("iacvt-prone"));
        assert!(rule_names.contains("circular-var"));
        assert!(rule_names.contains("unspecified-cascade-tie"));
        assert!(evaluations.iter().any(|evaluation| evaluation.rule_code
            == OmenaCheckerRuleCodeV0::IacvtProne
            && evaluation.declaration_ids == vec!["gap-use"]
            && evaluation.custom_property_names == vec!["--gap"]));
        assert!(evaluations.iter().any(|evaluation| evaluation.rule_code
            == OmenaCheckerRuleCodeV0::CircularVar
            && evaluation.custom_property_names == vec!["--a", "--b"]));
    }

    #[test]
    fn registered_property_type_mismatch_only_fires_on_definite_rejects() {
        let evaluations = evaluate_omena_checker_cascade_rules(OmenaCheckerCascadeInputV0 {
            declarations: vec![
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "bad-gap",
                    selector: ".bad",
                    property: "--gap",
                    value: "red",
                    source_order: 1,
                    condition_context: &[],
                    layer_name: None,
                    layer_order: None,
                    important: false,
                    var_references: &[],
                }),
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "good-gap",
                    selector: ".good",
                    property: "--gap",
                    value: "16px",
                    source_order: 2,
                    condition_context: &[],
                    layer_name: None,
                    layer_order: None,
                    important: false,
                    var_references: &[],
                }),
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "dynamic-gap",
                    selector: ".dynamic",
                    property: "--gap",
                    value: "var(--runtime-gap)",
                    source_order: 3,
                    condition_context: &[],
                    layer_name: None,
                    layer_order: None,
                    important: false,
                    var_references: &[],
                }),
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "unregistered",
                    selector: ".plain",
                    property: "--other",
                    value: "red",
                    source_order: 4,
                    condition_context: &[],
                    layer_name: None,
                    layer_order: None,
                    important: false,
                    var_references: &[],
                }),
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "universal-mode",
                    selector: ".mode",
                    property: "--mode",
                    value: "red",
                    source_order: 5,
                    condition_context: &[],
                    layer_name: None,
                    layer_order: None,
                    important: false,
                    var_references: &[],
                }),
            ],
            custom_properties: Vec::new(),
            custom_property_registrations: vec![
                OmenaCheckerCustomPropertyRegistrationInputV0 {
                    name: "--gap".to_string(),
                    syntax: Some("'<length>'".to_string()),
                    inherits: Some("false".to_string()),
                    initial_value: Some("8px".to_string()),
                },
                OmenaCheckerCustomPropertyRegistrationInputV0 {
                    name: "--mode".to_string(),
                    syntax: Some("'*'".to_string()),
                    inherits: Some("false".to_string()),
                    initial_value: None,
                },
            ],
        });
        let mismatches = evaluations
            .iter()
            .filter(|evaluation| {
                evaluation.rule_code == OmenaCheckerRuleCodeV0::RegisteredPropertyTypeMismatch
            })
            .collect::<Vec<_>>();

        assert_eq!(mismatches.len(), 1);
        assert_eq!(mismatches[0].declaration_ids, vec!["bad-gap"]);
        assert_eq!(mismatches[0].custom_property_names, vec!["--gap"]);
    }

    #[test]
    fn registered_property_type_mismatch_keeps_underdetermined_colors_silent() {
        let evaluations = evaluate_omena_checker_cascade_rules(OmenaCheckerCascadeInputV0 {
            declarations: vec![
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "valid-named-color",
                    selector: ".valid",
                    property: "--tone",
                    value: "tomato",
                    source_order: 1,
                    condition_context: &[],
                    layer_name: None,
                    layer_order: None,
                    important: false,
                    var_references: &[],
                }),
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "bad-color",
                    selector: ".bad",
                    property: "--tone",
                    value: "8px",
                    source_order: 2,
                    condition_context: &[],
                    layer_name: None,
                    layer_order: None,
                    important: false,
                    var_references: &[],
                }),
            ],
            custom_properties: Vec::new(),
            custom_property_registrations: vec![OmenaCheckerCustomPropertyRegistrationInputV0 {
                name: "--tone".to_string(),
                syntax: Some("'<color>'".to_string()),
                inherits: Some("false".to_string()),
                initial_value: Some("red".to_string()),
            }],
        });
        let mismatches = evaluations
            .iter()
            .filter(|evaluation| {
                evaluation.rule_code == OmenaCheckerRuleCodeV0::RegisteredPropertyTypeMismatch
            })
            .collect::<Vec<_>>();

        assert_eq!(mismatches.len(), 1);
        assert_eq!(mismatches[0].declaration_ids, vec!["bad-color"]);
        assert_eq!(mismatches[0].custom_property_names, vec!["--tone"]);
    }

    #[test]
    fn registered_property_type_mismatch_ignores_inactive_registrations_and_uses_last_wins() {
        let evaluations = evaluate_omena_checker_cascade_rules(OmenaCheckerCascadeInputV0 {
            declarations: vec![
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "missing-inherits",
                    selector: ".a",
                    property: "--missing-inherits",
                    value: "red",
                    source_order: 1,
                    condition_context: &[],
                    layer_name: None,
                    layer_order: None,
                    important: false,
                    var_references: &[],
                }),
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "missing-initial",
                    selector: ".b",
                    property: "--missing-initial",
                    value: "red",
                    source_order: 2,
                    condition_context: &[],
                    layer_name: None,
                    layer_order: None,
                    important: false,
                    var_references: &[],
                }),
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "last-wins",
                    selector: ".c",
                    property: "--tone",
                    value: "red",
                    source_order: 3,
                    condition_context: &[],
                    layer_name: None,
                    layer_order: None,
                    important: false,
                    var_references: &[],
                }),
            ],
            custom_properties: Vec::new(),
            custom_property_registrations: vec![
                OmenaCheckerCustomPropertyRegistrationInputV0 {
                    name: "--missing-inherits".to_string(),
                    syntax: Some("'<length>'".to_string()),
                    inherits: None,
                    initial_value: Some("8px".to_string()),
                },
                OmenaCheckerCustomPropertyRegistrationInputV0 {
                    name: "--missing-initial".to_string(),
                    syntax: Some("'<length>'".to_string()),
                    inherits: Some("false".to_string()),
                    initial_value: None,
                },
                OmenaCheckerCustomPropertyRegistrationInputV0 {
                    name: "--tone".to_string(),
                    syntax: Some("'<length>'".to_string()),
                    inherits: Some("false".to_string()),
                    initial_value: Some("8px".to_string()),
                },
                OmenaCheckerCustomPropertyRegistrationInputV0 {
                    name: "--tone".to_string(),
                    syntax: Some("'<color>'".to_string()),
                    inherits: Some("false".to_string()),
                    initial_value: Some("red".to_string()),
                },
            ],
        });

        assert!(!evaluations.iter().any(|evaluation| {
            evaluation.rule_code == OmenaCheckerRuleCodeV0::RegisteredPropertyTypeMismatch
        }));
    }

    #[test]
    fn invalid_property_value_fires_on_definite_keyword_violation() {
        let decl = |id: &'static str, value: &'static str, order: u32| {
            cascade_declaration(CascadeDeclarationFixture {
                declaration_id: id,
                selector: ".s",
                property: "box-sizing",
                value,
                source_order: order,
                condition_context: &[],
                layer_name: None,
                layer_order: None,
                important: false,
                var_references: &[],
            })
        };
        let evaluations = evaluate_omena_checker_cascade_rules(OmenaCheckerCascadeInputV0 {
            declarations: vec![decl("bad", "inline-box", 1), decl("good", "border-box", 2)],
            custom_properties: Vec::new(),
            custom_property_registrations: Vec::new(),
        });
        let findings = evaluations
            .iter()
            .filter(|evaluation| {
                evaluation.rule_code == OmenaCheckerRuleCodeV0::InvalidPropertyValue
            })
            .collect::<Vec<_>>();

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].declaration_ids, vec!["bad"]);
        assert_eq!(findings[0].custom_property_names, vec!["box-sizing"]);
    }

    #[test]
    fn invalid_property_value_uses_complete_compound_grammar() {
        let decl = |id: &'static str, value: &'static str, order: u32| {
            cascade_declaration(CascadeDeclarationFixture {
                declaration_id: id,
                selector: ".s",
                property: "border-top",
                value,
                source_order: order,
                condition_context: &[],
                layer_name: None,
                layer_order: None,
                important: false,
                var_references: &[],
            })
        };
        let evaluations = evaluate_omena_checker_cascade_rules(OmenaCheckerCascadeInputV0 {
            declarations: vec![
                decl("invalid-compound", "1px nonsense red", 1),
                decl("valid-compound", "1px solid red", 2),
            ],
            custom_properties: Vec::new(),
            custom_property_registrations: Vec::new(),
        });
        let findings = evaluations
            .iter()
            .filter(|evaluation| {
                evaluation.rule_code == OmenaCheckerRuleCodeV0::InvalidPropertyValue
            })
            .collect::<Vec<_>>();

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].declaration_ids, vec!["invalid-compound"]);
    }

    #[test]
    fn invalid_property_value_keeps_valid_and_undecidable_values_silent() {
        let decl = |id: &'static str, property: &'static str, value: &'static str, order: u32| {
            cascade_declaration(CascadeDeclarationFixture {
                declaration_id: id,
                selector: ".s",
                property,
                value,
                source_order: order,
                condition_context: &[],
                layer_name: None,
                layer_order: None,
                important: false,
                var_references: &[],
            })
        };
        let evaluations = evaluate_omena_checker_cascade_rules(OmenaCheckerCascadeInputV0 {
            declarations: vec![
                decl("valid", "box-sizing", "border-box", 1),
                decl("css-wide", "box-sizing", "inherit", 2),
                decl("substituted", "box-sizing", "var(--x)", 3),
                decl("open-grammar", "color", "rebeccapurple", 4),
                decl("custom-prop", "--tone", "anything-goes", 5),
            ],
            custom_properties: Vec::new(),
            custom_property_registrations: Vec::new(),
        });

        assert!(!evaluations.iter().any(|evaluation| {
            evaluation.rule_code == OmenaCheckerRuleCodeV0::InvalidPropertyValue
        }));
    }

    #[test]
    fn designer_intent_inconsistency_invokes_variational_mechanism() {
        let bem_evaluations = evaluate_omena_checker_cascade_rules(OmenaCheckerCascadeInputV0 {
            declarations: vec![
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "bem-a",
                    selector: ".button--primary",
                    property: "color",
                    value: "red",
                    source_order: 1,
                    condition_context: &[],
                    layer_name: None,
                    layer_order: None,
                    important: false,
                    var_references: &[],
                }),
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "bem-b",
                    selector: ".button--primary",
                    property: "color",
                    value: "blue",
                    source_order: 2,
                    condition_context: &[],
                    layer_name: None,
                    layer_order: None,
                    important: false,
                    var_references: &[],
                }),
            ],
            custom_properties: Vec::new(),
            custom_property_registrations: Vec::new(),
        });
        let utility_evaluations =
            evaluate_omena_checker_cascade_rules(OmenaCheckerCascadeInputV0 {
                declarations: vec![
                    cascade_declaration(CascadeDeclarationFixture {
                        declaration_id: "utility-a",
                        selector: ".u-color-red",
                        property: "color",
                        value: "red",
                        source_order: 1,
                        condition_context: &[],
                        layer_name: None,
                        layer_order: None,
                        important: false,
                        var_references: &[],
                    }),
                    cascade_declaration(CascadeDeclarationFixture {
                        declaration_id: "utility-b",
                        selector: ".u-color-red",
                        property: "color",
                        value: "blue",
                        source_order: 2,
                        condition_context: &[],
                        layer_name: None,
                        layer_order: None,
                        important: false,
                        var_references: &[],
                    }),
                ],
                custom_properties: Vec::new(),
                custom_property_registrations: Vec::new(),
            });

        let designer_evaluation = bem_evaluations.iter().find(|evaluation| {
            evaluation.rule_code == OmenaCheckerRuleCodeV0::DesignerIntentInconsistency
        });
        assert!(
            designer_evaluation.is_some(),
            "BEM source-order tie should invoke variational checker"
        );
        let Some(designer_evaluation) = designer_evaluation else {
            return;
        };
        assert_eq!(
            designer_evaluation.mechanism_products,
            vec!["omena-variational.designer-intent-posterior"]
        );
        assert!(
            utility_evaluations.iter().all(|evaluation| {
                evaluation.rule_code != OmenaCheckerRuleCodeV0::DesignerIntentInconsistency
            }),
            "same cascade tie facts with utility selector evidence must not emit the BEM posterior diagnostic"
        );
    }

    #[test]
    fn categorical_evidence_inconsistency_is_gated_by_functor_verdict() {
        // Composable mapping: three distinct primitive->role pairs give two
        // non-identity morphisms that the functor can compose, so the functor
        // verdict is `accepted` and no diagnostic is emitted.
        let functorial = evaluate_omena_checker_categorical_rules(OmenaCheckerCategoricalInputV0 {
            mappings: vec![categorical_role_mapping(
                "functorial-mapping",
                &[
                    ("cascade_property", "cosheaf colimit witness"),
                    ("prove_layer_flatten_candidate", "beck-chevalley witness"),
                    (
                        "evaluate_static_supports_condition",
                        "site decidability witness",
                    ),
                ],
            )],
        });
        assert!(
            functorial.is_empty(),
            "a composable role mapping must not emit a categorical inconsistency"
        );

        // Non-composable mapping: only two pairs yield a single non-identity
        // morphism, so composition cannot be witnessed. The functor verdict is
        // rejected and the diagnostic fires from the real verdict, not a literal.
        let broken = evaluate_omena_checker_categorical_rules(OmenaCheckerCategoricalInputV0 {
            mappings: vec![categorical_role_mapping(
                "broken-mapping",
                &[
                    ("cascade_property", "cosheaf colimit witness"),
                    ("prove_layer_flatten_candidate", "beck-chevalley witness"),
                ],
            )],
        });
        assert_eq!(broken.len(), 1);
        let evaluation = &broken[0];
        assert_eq!(
            evaluation.rule_code,
            OmenaCheckerRuleCodeV0::CategoricalCascadeEvidenceInconsistency
        );
        assert_eq!(evaluation.mapping_id, "broken-mapping");
        assert!(!evaluation.functor_accepted);
        assert!(!evaluation.composition_preserved);
        assert!(evaluation.identity_preserved);
        assert_eq!(
            evaluation.mechanism_products,
            vec!["omena-categorical.cascade-primitive-role-functor"]
        );
    }

    fn categorical_role_mapping(
        mapping_id: &str,
        pairs: &[(&str, &str)],
    ) -> OmenaCheckerCategoricalRoleMappingInputV0 {
        OmenaCheckerCategoricalRoleMappingInputV0 {
            mapping_id: mapping_id.to_string(),
            primitive_role_pairs: pairs
                .iter()
                .map(|(primitive_name, categorical_role)| {
                    OmenaCheckerCategoricalPrimitiveRolePairInputV0 {
                        primitive_name: (*primitive_name).to_string(),
                        categorical_role: (*categorical_role).to_string(),
                    }
                })
                .collect(),
        }
    }

    #[test]
    fn evaluates_grn_rule_family_from_cascade_projection() {
        let evaluations = evaluate_omena_checker_grn_rules(OmenaCheckerGrnInputV0 {
            vertices: vec![
                grn_vertex(
                    "winner",
                    ".btn",
                    "color",
                    OmenaCheckerGrnVertexStateKindV0::Applied,
                ),
                grn_vertex(
                    "losing-eligible",
                    ".btn",
                    "color",
                    OmenaCheckerGrnVertexStateKindV0::LosingButEligible,
                ),
                grn_vertex(
                    "inactive-rule",
                    ".card",
                    "display",
                    OmenaCheckerGrnVertexStateKindV0::Inactive,
                ),
            ],
        });
        let rule_names = evaluations
            .iter()
            .map(|evaluation| evaluation.rule_code_name)
            .collect::<BTreeSet<_>>();

        assert!(rule_names.contains("cascade.deep-conflict"));
        assert!(rule_names.contains("cascade.unreachable-rule"));
        assert!(evaluations.iter().any(|evaluation| {
            evaluation.rule_code == OmenaCheckerRuleCodeV0::CascadeDeepConflict
                && evaluation.vertex_ids == vec!["losing-eligible"]
                && evaluation.mechanism_products == vec!["omena-cascade.grn-outcome-projection"]
        }));
        assert!(evaluations.iter().any(|evaluation| {
            evaluation.rule_code == OmenaCheckerRuleCodeV0::CascadeUnreachableRule
                && evaluation.vertex_ids == vec!["inactive-rule"]
                && evaluation.mechanism_products == vec!["omena-cascade.grn-outcome-projection"]
        }));

        let clear_evaluations = evaluate_omena_checker_grn_rules(OmenaCheckerGrnInputV0 {
            vertices: vec![grn_vertex(
                "winner",
                ".btn",
                "color",
                OmenaCheckerGrnVertexStateKindV0::Applied,
            )],
        });
        assert!(clear_evaluations.is_empty());
    }

    #[test]
    fn evaluates_smt_rule_family_from_canonical_obligations() {
        let evaluations = evaluate_omena_checker_smt_rules(OmenaCheckerSmtInputV0 {
            obligations: vec![OmenaCheckerSmtObligationInputV0 {
                obligation_id: "bad-layer-flatten".to_string(),
                l1_primitive: "layerFlattenCandidate".to_string(),
                canonical_terms: vec![
                    "require:closed-bundle=true".to_string(),
                    "require:no-unlayered-rule=false".to_string(),
                ],
            }],
        });

        assert_eq!(evaluations.len(), 1);
        assert_eq!(
            evaluations[0].rule_code,
            OmenaCheckerRuleCodeV0::CascadeSMTViolation
        );
        assert_eq!(evaluations[0].obligation_id, "bad-layer-flatten");
        assert_eq!(
            evaluations[0].backend_kind_name,
            if cfg!(feature = "smt-z3") {
                "z3"
            } else {
                "stub"
            }
        );
        assert_eq!(evaluations[0].sat_result_name, "unsat");
        assert!(
            evaluations[0]
                .mechanism_products
                .contains(&"omena-smt.backend-check")
        );
        assert!(
            evaluations[0]
                .mechanism_products
                .contains(&if cfg!(feature = "smt-z3") {
                    "omena-smt.backend.z3"
                } else {
                    "omena-smt.backend.stub"
                })
        );

        let clear_evaluations = evaluate_omena_checker_smt_rules(OmenaCheckerSmtInputV0 {
            obligations: vec![OmenaCheckerSmtObligationInputV0 {
                obligation_id: "ok-layer-flatten".to_string(),
                l1_primitive: "layerFlattenCandidate".to_string(),
                canonical_terms: vec![
                    "require:closed-bundle=true".to_string(),
                    "require:no-unlayered-rule=true".to_string(),
                ],
            }],
        });
        assert!(clear_evaluations.is_empty());
    }

    #[test]
    fn evaluates_smt_layer_inversion_only_for_solver_backed_scope() {
        let evaluations = evaluate_omena_checker_smt_layer_inversion_rules(
            OmenaCheckerSmtLayerInversionInputV0 {
                obligations: vec![OmenaCheckerSmtLayerInversionObligationInputV0 {
                    obligation_id: "layer-inversion".to_string(),
                    declarations: vec![
                        OmenaCheckerSmtLayerInversionDeclarationInputV0 {
                            declaration_id: "utilities-color".to_string(),
                            layer_rank: 1,
                            source_order: 0,
                        },
                        OmenaCheckerSmtLayerInversionDeclarationInputV0 {
                            declaration_id: "base-color".to_string(),
                            layer_rank: 0,
                            source_order: 1,
                        },
                    ],
                }],
            },
        );

        if cfg!(feature = "smt-z3") {
            assert_eq!(evaluations.len(), 1);
            assert_eq!(evaluations[0].backend_kind_name, "z3");
            assert_eq!(evaluations[0].sat_result_name, "sat");
            assert_eq!(
                evaluations[0].message,
                "Opt-in z3 SMT backend found a layer-flatten inversion."
            );
            assert!(
                evaluations[0]
                    .mechanism_products
                    .contains(&"omena-smt.layer-flatten-inversion")
            );
        } else {
            assert!(
                evaluations.is_empty(),
                "default solver-free scope must not emit z3-only layer inversion diagnostics"
            );
        }
    }

    #[test]
    fn evaluates_mdl_budget_rule_family_from_query_mdl_summaries() {
        let evaluations = evaluate_omena_checker_mdl_rules(OmenaCheckerMdlInputV0 {
            summaries: vec![OmenaCheckerMdlSummaryInputV0 {
                source_uri: "file:///workspace/Button.module.css".to_string(),
                total_bits: 14.0,
                budget_bits: 8.0,
            }],
        });

        assert_eq!(evaluations.len(), 1);
        assert_eq!(
            evaluations[0].rule_code,
            OmenaCheckerRuleCodeV0::DesignSystemMdlBudget
        );
        assert_eq!(
            evaluations[0].source_uri,
            "file:///workspace/Button.module.css"
        );
        assert_eq!(evaluations[0].total_bits, 14.0);
        assert_eq!(evaluations[0].budget_bits, 8.0);
        assert_eq!(
            evaluations[0].mechanism_products,
            vec!["omena-query.design-system-minimum-description"]
        );

        let clear_evaluations = evaluate_omena_checker_mdl_rules(OmenaCheckerMdlInputV0 {
            summaries: vec![OmenaCheckerMdlSummaryInputV0 {
                source_uri: "file:///workspace/Button.module.css".to_string(),
                total_bits: 8.0,
                budget_bits: 8.0,
            }],
        });
        assert!(clear_evaluations.is_empty());
    }

    fn streaming_ifds_report_input(
        report_id: &str,
        incremental_precision_parity_with_batch: bool,
        reachability_fallback_applied: bool,
        fact_fallback_applied: bool,
    ) -> OmenaCheckerStreamingIfdsReportInputV0 {
        OmenaCheckerStreamingIfdsReportInputV0 {
            report_id: report_id.to_string(),
            incremental_precision_parity_with_batch,
            reachability_fallback_applied,
            fact_fallback_applied,
        }
    }

    #[test]
    fn evaluates_streaming_ifds_precision_parity_rule_family() {
        let evaluations =
            evaluate_omena_checker_streaming_ifds_rules(OmenaCheckerStreamingIfdsInputV0 {
                reports: vec![streaming_ifds_report_input(
                    "streaming-report-1",
                    false,
                    false,
                    true,
                )],
            });

        assert_eq!(evaluations.len(), 1);
        assert_eq!(
            evaluations[0].rule_code,
            OmenaCheckerRuleCodeV0::StreamingIfdsPrecisionParity
        );
        assert_eq!(evaluations[0].report_id, "streaming-report-1");
        assert!(!evaluations[0].incremental_precision_parity_with_batch);
        assert!(!evaluations[0].reachability_fallback_applied);
        assert!(evaluations[0].fact_fallback_applied);
        assert_eq!(
            evaluations[0].mechanism_products,
            vec!["omena-streaming-ifds.analysis-report"]
        );

        let clear_evaluations =
            evaluate_omena_checker_streaming_ifds_rules(OmenaCheckerStreamingIfdsInputV0 {
                reports: vec![streaming_ifds_report_input(
                    "streaming-report-2",
                    true,
                    false,
                    false,
                )],
            });
        assert!(clear_evaluations.is_empty());
    }

    #[test]
    fn evaluates_rg_flow_relevant_operator_rule_family() {
        let evaluations = evaluate_omena_checker_rg_flow_rules(OmenaCheckerRgFlowInputV0 {
            flows: vec![OmenaCheckerRgFlowCouplingInputV0 {
                workspace_path: "workspace://critical-token-graph".to_string(),
                before: rg_flow_coupling(1, 1, 0, 0),
                after: rg_flow_coupling(5, 0, 0, 8),
            }],
        });

        assert_eq!(evaluations.len(), 1);
        assert_eq!(
            evaluations[0].rule_code,
            OmenaCheckerRuleCodeV0::RgFlowRelevantOperator
        );
        assert_eq!(
            evaluations[0].workspace_path,
            "workspace://critical-token-graph"
        );
        assert!(evaluations[0].spectral_radius > 1.0);
        assert_eq!(evaluations[0].mechanism_scope, RG_FLOW_MECHANISM_SCOPE_V0);
        assert_eq!(evaluations[0].product_surface, RG_FLOW_PRODUCT_SURFACE_V0);
        assert!(!evaluations[0].default_product_decision_mechanism);
        assert!(
            evaluations[0]
                .message
                .contains("not a default product decision mechanism")
        );
        assert_eq!(
            evaluations[0].mechanism_products,
            vec!["omena-rg-flow.coupling-jacobian-spectrum"]
        );

        let clear_evaluations = evaluate_omena_checker_rg_flow_rules(OmenaCheckerRgFlowInputV0 {
            flows: vec![OmenaCheckerRgFlowCouplingInputV0 {
                workspace_path: "workspace://settled-token-graph".to_string(),
                before: rg_flow_coupling(4, 2, 0, 0),
                after: rg_flow_coupling(3, 1, 0, 0),
            }],
        });
        assert!(clear_evaluations.is_empty());
    }

    #[test]
    fn evaluates_replica_ensemble_inconsistency_rule_family() {
        let evaluations =
            evaluate_omena_checker_replica_ensemble_rules(OmenaCheckerReplicaEnsembleInputV0 {
                reports: vec![OmenaCheckerReplicaEnsembleReportInputV0 {
                    workspace_root: "/workspace".to_string(),
                    recommendation: "investigateRsbBroken".to_string(),
                    mean_q: 0.5,
                    variance_q: 0.25,
                    top_disagreement_pair_count: 2,
                    mechanism_scope: "productWiredCrossFileConsistencyHintSubstrate".to_string(),
                    product_surface: "defaultCrossFileConsistencyHint".to_string(),
                    default_product_decision_mechanism: false,
                }],
            });

        assert_eq!(evaluations.len(), 1);
        assert_eq!(
            evaluations[0].rule_code,
            OmenaCheckerRuleCodeV0::ReplicaEnsembleInconsistency
        );
        assert_eq!(evaluations[0].workspace_root, "/workspace");
        assert_eq!(evaluations[0].recommendation, "investigateRsbBroken");
        assert_eq!(
            evaluations[0].mechanism_products,
            vec!["omena-ensemble.cross-file-inconsistency-report"]
        );
        assert_eq!(
            evaluations[0].mechanism_scope,
            "productWiredCrossFileConsistencyHintSubstrate"
        );
        assert_eq!(
            evaluations[0].product_surface,
            "defaultCrossFileConsistencyHint"
        );
        assert!(!evaluations[0].default_product_decision_mechanism);
        assert!(
            evaluations[0]
                .message
                .contains("not a default product decision mechanism")
        );

        let clear_evaluations =
            evaluate_omena_checker_replica_ensemble_rules(OmenaCheckerReplicaEnsembleInputV0 {
                reports: vec![OmenaCheckerReplicaEnsembleReportInputV0 {
                    workspace_root: "/workspace".to_string(),
                    recommendation: "noActionNeeded".to_string(),
                    mean_q: 1.0,
                    variance_q: 0.0,
                    top_disagreement_pair_count: 0,
                    mechanism_scope: "productWiredCrossFileConsistencyHintSubstrate".to_string(),
                    product_surface: "defaultCrossFileConsistencyHint".to_string(),
                    default_product_decision_mechanism: false,
                }],
            });
        assert!(clear_evaluations.is_empty());
    }

    #[test]
    fn cascade_rules_do_not_compare_across_conditional_contexts() {
        let evaluations = evaluate_omena_checker_cascade_rules(OmenaCheckerCascadeInputV0 {
            declarations: vec![
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "base-color",
                    selector: ".btn",
                    property: "color",
                    value: "red",
                    source_order: 1,
                    condition_context: &[],
                    layer_name: None,
                    layer_order: None,
                    important: false,
                    var_references: &[],
                }),
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "media-color",
                    selector: ".btn",
                    property: "color",
                    value: "blue",
                    source_order: 2,
                    condition_context: &["@media (min-width: 40rem)"],
                    layer_name: None,
                    layer_order: None,
                    important: false,
                    var_references: &[],
                }),
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "supports-color",
                    selector: ".btn",
                    property: "color",
                    value: "green",
                    source_order: 3,
                    condition_context: &["@supports (display: grid)"],
                    layer_name: None,
                    layer_order: None,
                    important: false,
                    var_references: &[],
                }),
            ],
            custom_properties: Vec::new(),
            custom_property_registrations: Vec::new(),
        });

        let rule_names = evaluations
            .iter()
            .map(|evaluation| evaluation.rule_code_name)
            .collect::<BTreeSet<_>>();
        assert!(!rule_names.contains("unreachable-declaration"));
        assert!(!rule_names.contains("unspecified-cascade-tie"));
    }

    #[test]
    fn cascade_rules_model_unlayered_normal_and_layered_important_precedence() {
        let evaluations = evaluate_omena_checker_cascade_rules(OmenaCheckerCascadeInputV0 {
            declarations: vec![
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "layered-normal",
                    selector: ".btn",
                    property: "color",
                    value: "red",
                    source_order: 1,
                    condition_context: &[],
                    layer_name: Some("base"),
                    layer_order: Some(0),
                    important: false,
                    var_references: &[],
                }),
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "unlayered-normal",
                    selector: ".btn",
                    property: "color",
                    value: "blue",
                    source_order: 2,
                    condition_context: &[],
                    layer_name: None,
                    layer_order: None,
                    important: false,
                    var_references: &[],
                }),
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "unlayered-important",
                    selector: ".alert",
                    property: "color",
                    value: "red",
                    source_order: 3,
                    condition_context: &[],
                    layer_name: None,
                    layer_order: None,
                    important: true,
                    var_references: &[],
                }),
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "layered-important",
                    selector: ".alert",
                    property: "color",
                    value: "blue",
                    source_order: 4,
                    condition_context: &[],
                    layer_name: Some("important-base"),
                    layer_order: Some(0),
                    important: true,
                    var_references: &[],
                }),
            ],
            custom_properties: Vec::new(),
            custom_property_registrations: Vec::new(),
        });

        assert!(evaluations.iter().any(|evaluation| evaluation.rule_code
            == OmenaCheckerRuleCodeV0::UnreachableDeclaration
            && evaluation.declaration_ids == vec!["layered-normal", "unlayered-normal"]));
        assert!(evaluations.iter().any(|evaluation| evaluation.rule_code
            == OmenaCheckerRuleCodeV0::UnreachableDeclaration
            && evaluation.declaration_ids == vec!["unlayered-important", "layered-important"]));
        assert!(evaluations.iter().any(|evaluation| evaluation.rule_code
            == OmenaCheckerRuleCodeV0::DeadCascadeLayer
            && evaluation.layer_name.as_deref() == Some("base")));
        assert!(!evaluations.iter().any(|evaluation| evaluation.rule_code
            == OmenaCheckerRuleCodeV0::DeadCascadeLayer
            && evaluation.layer_name.as_deref() == Some("important-base")));
    }

    #[test]
    fn cascade_rules_ignore_vendor_prefix_gradient_progressive_enhancement() {
        let evaluations = evaluate_omena_checker_cascade_rules(OmenaCheckerCascadeInputV0 {
            declarations: vec![
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "webkit-gradient",
                    selector: ".x",
                    property: "background-image",
                    value: "-webkit-linear-gradient(top, #fff, #000)",
                    source_order: 1,
                    condition_context: &[],
                    layer_name: None,
                    layer_order: None,
                    important: false,
                    var_references: &[],
                }),
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "moz-gradient",
                    selector: ".x",
                    property: "background-image",
                    value: "-moz-linear-gradient(top, #fff, #000)",
                    source_order: 2,
                    condition_context: &[],
                    layer_name: None,
                    layer_order: None,
                    important: false,
                    var_references: &[],
                }),
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "unprefixed-gradient",
                    selector: ".x",
                    property: "background-image",
                    value: "linear-gradient(to bottom, #fff, #000)",
                    source_order: 3,
                    condition_context: &[],
                    layer_name: None,
                    layer_order: None,
                    important: false,
                    var_references: &[],
                }),
            ],
            custom_properties: Vec::new(),
            custom_property_registrations: Vec::new(),
        });

        let rule_names = evaluations
            .iter()
            .map(|evaluation| evaluation.rule_code_name)
            .collect::<BTreeSet<_>>();
        assert!(!rule_names.contains("unreachable-declaration"));
        assert!(!rule_names.contains("unspecified-cascade-tie"));
    }

    #[test]
    fn cascade_rules_ignore_vendor_prefix_flex_progressive_enhancement() {
        let evaluations = evaluate_omena_checker_cascade_rules(OmenaCheckerCascadeInputV0 {
            declarations: vec![
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "flex",
                    selector: ".y",
                    property: "display",
                    value: "flex",
                    source_order: 1,
                    condition_context: &[],
                    layer_name: None,
                    layer_order: None,
                    important: false,
                    var_references: &[],
                }),
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "ms-flexbox",
                    selector: ".y",
                    property: "display",
                    value: "-ms-flexbox",
                    source_order: 2,
                    condition_context: &[],
                    layer_name: None,
                    layer_order: None,
                    important: false,
                    var_references: &[],
                }),
            ],
            custom_properties: Vec::new(),
            custom_property_registrations: Vec::new(),
        });

        let rule_names = evaluations
            .iter()
            .map(|evaluation| evaluation.rule_code_name)
            .collect::<BTreeSet<_>>();
        assert!(!rule_names.contains("unreachable-declaration"));
        assert!(!rule_names.contains("unspecified-cascade-tie"));
    }

    #[test]
    fn cascade_rules_still_flag_accidental_duplicate_without_vendor_prefix() {
        let evaluations = evaluate_omena_checker_cascade_rules(OmenaCheckerCascadeInputV0 {
            declarations: vec![
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "color-red",
                    selector: ".z",
                    property: "color",
                    value: "red",
                    source_order: 1,
                    condition_context: &[],
                    layer_name: None,
                    layer_order: None,
                    important: false,
                    var_references: &[],
                }),
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "color-blue",
                    selector: ".z",
                    property: "color",
                    value: "blue",
                    source_order: 2,
                    condition_context: &[],
                    layer_name: None,
                    layer_order: None,
                    important: false,
                    var_references: &[],
                }),
            ],
            custom_properties: Vec::new(),
            custom_property_registrations: Vec::new(),
        });

        assert!(evaluations.iter().any(|evaluation| evaluation.rule_code
            == OmenaCheckerRuleCodeV0::UnreachableDeclaration
            && evaluation.declaration_ids == vec!["color-red", "color-blue"]));
        assert!(evaluations.iter().any(
            |evaluation| evaluation.rule_code == OmenaCheckerRuleCodeV0::UnspecifiedCascadeTie
        ));
    }

    #[test]
    fn progressive_enhancement_pair_requires_leading_token_prefix() {
        // A `-webkit-` substring buried mid-expression must NOT be treated as a
        // progressive-enhancement fallback.
        let buried = cascade_declaration(CascadeDeclarationFixture {
            declaration_id: "buried-a",
            selector: ".w",
            property: "transition",
            value: "transform 0.2s -webkit-ease",
            source_order: 1,
            condition_context: &[],
            layer_name: None,
            layer_order: None,
            important: false,
            var_references: &[],
        });
        let plain = cascade_declaration(CascadeDeclarationFixture {
            declaration_id: "buried-b",
            selector: ".w",
            property: "transition",
            value: "transform 0.3s ease",
            source_order: 2,
            condition_context: &[],
            layer_name: None,
            layer_order: None,
            important: false,
            var_references: &[],
        });
        assert!(!is_progressive_enhancement_pair(&buried, &plain));

        // Identical values are never a progressive-enhancement pair.
        let prefixed = cascade_declaration(CascadeDeclarationFixture {
            declaration_id: "prefixed",
            selector: ".w",
            property: "display",
            value: "-ms-flexbox",
            source_order: 1,
            condition_context: &[],
            layer_name: None,
            layer_order: None,
            important: false,
            var_references: &[],
        });
        let mut prefixed_dup = prefixed.clone();
        prefixed_dup.declaration_id = "prefixed-dup".to_string();
        prefixed_dup.source_order = 2;
        assert!(!is_progressive_enhancement_pair(&prefixed, &prefixed_dup));
    }

    struct CascadeDeclarationFixture<'a> {
        declaration_id: &'a str,
        selector: &'a str,
        property: &'a str,
        value: &'a str,
        source_order: u32,
        condition_context: &'a [&'a str],
        layer_name: Option<&'a str>,
        layer_order: Option<i32>,
        important: bool,
        var_references: &'a [&'a str],
    }

    fn cascade_declaration(
        fixture: CascadeDeclarationFixture<'_>,
    ) -> OmenaCheckerCascadeDeclarationInputV0 {
        OmenaCheckerCascadeDeclarationInputV0 {
            declaration_id: fixture.declaration_id.to_string(),
            selector: CanonicalSelector::from_canonical(fixture.selector),
            property: fixture.property.to_string(),
            value: fixture.value.to_string(),
            source_order: fixture.source_order,
            condition_context: fixture
                .condition_context
                .iter()
                .map(|value| value.to_string())
                .collect(),
            layer_name: fixture.layer_name.map(str::to_string),
            layer_order: fixture.layer_order,
            important: fixture.important,
            var_references: fixture
                .var_references
                .iter()
                .map(|value| value.to_string())
                .collect(),
        }
    }

    fn rg_flow_coupling(
        k_env: usize,
        k_decl: usize,
        k_cycle: usize,
        k_dirty: usize,
    ) -> OmenaCheckerRgFlowCouplingSpaceInputV0 {
        OmenaCheckerRgFlowCouplingSpaceInputV0 {
            k_env,
            k_decl,
            k_cycle,
            k_dirty,
        }
    }

    fn grn_vertex(
        vertex_id: &str,
        selector: &str,
        property: &str,
        state: OmenaCheckerGrnVertexStateKindV0,
    ) -> OmenaCheckerGrnVertexStateInputV0 {
        OmenaCheckerGrnVertexStateInputV0 {
            vertex_id: vertex_id.to_string(),
            selector: selector.to_string(),
            property: property.to_string(),
            state,
        }
    }
}
