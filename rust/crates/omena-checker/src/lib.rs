use std::collections::{BTreeMap, BTreeSet};

use omena_abstract_value::{
    AbstractClassValueV0, SelectorProjectionCertaintyV0, enumerate_finite_class_values,
    project_abstract_value_selectors,
};
use serde::{Deserialize, Serialize};

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
}

impl OmenaCheckerRuleTierV0 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::M => "m-tier",
            Self::S => "s-tier",
            Self::T => "t-tier",
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
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerCascadeDeclarationInputV0 {
    pub declaration_id: String,
    pub selector: String,
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
}

pub fn list_omena_checker_rule_descriptors() -> Vec<OmenaCheckerRuleDescriptorV0> {
    use OmenaCheckerFindingCategoryV0::{Source, Style};
    use OmenaCheckerRuleCodeV0::{
        CircularVar, DeadCascadeLayer, IacvtProne, MissingComposedModule, MissingComposedSelector,
        MissingCustomProperty, MissingImportedValue, MissingKeyframes, MissingModule,
        MissingResolvedClassDomain, MissingResolvedClassValues, MissingSassSymbol,
        MissingStaticClass, MissingTemplatePrefix, MissingValueModule, NoImpossibleSelector,
        NoImpreciseValue, NoUnknownDynamicClass, UnreachableDeclaration, UnspecifiedCascadeTie,
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
        MissingModule, MissingResolvedClassDomain, MissingResolvedClassValues, MissingStaticClass,
        MissingTemplatePrefix,
    };

    vec![
        MissingModule,
        MissingStaticClass,
        MissingTemplatePrefix,
        MissingResolvedClassValues,
        MissingResolvedClassDomain,
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
        MissingValueModule, UnreachableDeclaration, UnspecifiedCascadeTie, UnusedSelector,
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
        UnspecifiedCascadeTie,
    ]
}

pub fn list_omena_checker_t_tier_rule_code_names() -> Vec<&'static str> {
    list_omena_checker_t_tier_rule_codes()
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

pub fn list_omena_checker_code_bundles() -> Vec<OmenaCheckerCodeBundleV0> {
    use OmenaCheckerCodeBundleNameV0::{
        CascadeAware, CiDefault, SourceMissing, StyleRecovery, StyleUnused,
    };
    use OmenaCheckerRuleCodeV0::{
        CircularVar, DeadCascadeLayer, IacvtProne, MissingComposedModule, MissingComposedSelector,
        MissingImportedValue, MissingKeyframes, MissingModule, MissingResolvedClassDomain,
        MissingResolvedClassValues, MissingSassSymbol, MissingStaticClass, MissingTemplatePrefix,
        MissingValueModule, NoImpossibleSelector, NoImpreciseValue, NoUnknownDynamicClass,
        UnreachableDeclaration, UnspecifiedCascadeTie, UnusedSelector,
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
                UnspecifiedCascadeTie,
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
            }
        }
    }

    evaluations
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
    OmenaCheckerCascadeEvaluationV0 {
        rule_code,
        rule_code_name: rule_code.as_str(),
        severity,
        severity_name: severity.as_str(),
        declaration_ids,
        layer_name,
        custom_property_names,
        message: message.to_string(),
    }
}

fn declaration_outranks(
    candidate: &OmenaCheckerCascadeDeclarationInputV0,
    declaration: &OmenaCheckerCascadeDeclarationInputV0,
) -> bool {
    if !declarations_share_cascade_context(candidate, declaration) {
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
    {
        return false;
    }
    match (
        candidate.layer_order,
        declaration.layer_order,
        declaration.important,
    ) {
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
        && left.important == right.important
        && left.layer_order == right.layer_order
        && left.layer_name == right.layer_name
        && left.source_order != right.source_order
}

fn declarations_share_cascade_context(
    left: &OmenaCheckerCascadeDeclarationInputV0,
    right: &OmenaCheckerCascadeDeclarationInputV0,
) -> bool {
    left.selector == right.selector
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

fn rule(
    code: OmenaCheckerRuleCodeV0,
    category: OmenaCheckerFindingCategoryV0,
    default_severity: OmenaCheckerSeverityV0,
    fixability: OmenaCheckerRuleFixabilityV0,
    presets: &[OmenaCheckerRulePresetV0],
    description: &'static str,
) -> OmenaCheckerRuleDescriptorV0 {
    let tier = rule_tier_for_code(code);
    OmenaCheckerRuleDescriptorV0 {
        code,
        code_name: code.as_str(),
        category,
        category_name: category.as_str(),
        tier,
        tier_name: tier.as_str(),
        default_severity,
        default_severity_name: default_severity.as_str(),
        fixability,
        fixability_name: fixability.as_str(),
        presets: presets.to_vec(),
        preset_names: presets.iter().map(|preset| preset.as_str()).collect(),
        description,
    }
}

fn rule_tier_for_code(code: OmenaCheckerRuleCodeV0) -> OmenaCheckerRuleTierV0 {
    use OmenaCheckerRuleCodeV0::{
        CircularVar, DeadCascadeLayer, IacvtProne, MissingComposedModule, MissingComposedSelector,
        MissingCustomProperty, MissingImportedValue, MissingKeyframes, MissingModule,
        MissingResolvedClassDomain, MissingResolvedClassValues, MissingSassSymbol,
        MissingStaticClass, MissingTemplatePrefix, MissingValueModule, NoImpossibleSelector,
        NoImpreciseValue, NoUnknownDynamicClass, UnreachableDeclaration, UnspecifiedCascadeTie,
        UnusedSelector,
    };

    match code {
        NoUnknownDynamicClass | NoImpreciseValue | NoImpossibleSelector => {
            OmenaCheckerRuleTierV0::M
        }
        MissingModule
        | MissingStaticClass
        | MissingTemplatePrefix
        | MissingResolvedClassValues
        | MissingResolvedClassDomain => OmenaCheckerRuleTierV0::S,
        UnusedSelector
        | MissingComposedModule
        | MissingComposedSelector
        | MissingValueModule
        | MissingImportedValue
        | MissingKeyframes
        | MissingCustomProperty
        | MissingSassSymbol
        | UnreachableDeclaration
        | DeadCascadeLayer
        | IacvtProne
        | CircularVar
        | UnspecifiedCascadeTie => OmenaCheckerRuleTierV0::T,
    }
}

fn count_rules_in_tier(
    descriptors: &[OmenaCheckerRuleDescriptorV0],
    tier: OmenaCheckerRuleTierV0,
) -> usize {
    descriptors
        .iter()
        .filter(|descriptor| descriptor.tier == tier)
        .count()
}

fn bundle(
    bundle: OmenaCheckerCodeBundleNameV0,
    codes: &[OmenaCheckerRuleCodeV0],
) -> OmenaCheckerCodeBundleV0 {
    OmenaCheckerCodeBundleV0 {
        bundle,
        bundle_name: bundle.as_str(),
        codes: codes.to_vec(),
        code_names: codes.iter().map(|code| code.as_str()).collect(),
    }
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
            ],
        );
    }

    #[test]
    fn descriptors_have_required_metadata_without_duplicate_codes() {
        let descriptors = list_omena_checker_rule_descriptors();
        let mut codes = BTreeSet::new();

        for descriptor in &descriptors {
            assert!(codes.insert(descriptor.code_name));
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
        assert_eq!(summary.rule_count, 21);
        assert_eq!(summary.source_rule_count, 8);
        assert_eq!(summary.style_rule_count, 13);
        assert_eq!(summary.m_tier_rule_count, 3);
        assert_eq!(summary.s_tier_rule_count, 5);
        assert_eq!(summary.t_tier_rule_count, 13);
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
                "unspecified-cascade-tie",
            ]
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
        });

        let rule_names = evaluations
            .iter()
            .map(|evaluation| evaluation.rule_code_name)
            .collect::<BTreeSet<_>>();
        assert!(!rule_names.contains("unreachable-declaration"));
        assert!(!rule_names.contains("unspecified-cascade-tie"));
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
            selector: fixture.selector.to_string(),
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
}
