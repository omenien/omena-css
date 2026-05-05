use std::collections::BTreeSet;

use omena_abstract_value::{
    AbstractClassValueV0, SelectorProjectionCertaintyV0, enumerate_finite_class_values,
    project_abstract_value_selectors,
};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OmenaCheckerRuleCodeV0 {
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
}

impl OmenaCheckerRuleCodeV0 {
    pub const fn as_str(self) -> &'static str {
        match self {
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
}

impl OmenaCheckerCodeBundleNameV0 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CiDefault => "ci-default",
            Self::SourceMissing => "source-missing",
            Self::StyleRecovery => "style-recovery",
            Self::StyleUnused => "style-unused",
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

pub fn list_omena_checker_rule_descriptors() -> Vec<OmenaCheckerRuleDescriptorV0> {
    use OmenaCheckerFindingCategoryV0::{Source, Style};
    use OmenaCheckerRuleCodeV0::{
        MissingComposedModule, MissingComposedSelector, MissingCustomProperty,
        MissingImportedValue, MissingKeyframes, MissingModule, MissingResolvedClassDomain,
        MissingResolvedClassValues, MissingSassSymbol, MissingStaticClass, MissingTemplatePrefix,
        MissingValueModule, UnusedSelector,
    };
    use OmenaCheckerRuleFixabilityV0::{CodeAction, None};
    use OmenaCheckerRulePresetV0::{Recommended, Strict};
    use OmenaCheckerSeverityV0::{Hint, Warning};

    vec![
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
    use OmenaCheckerCodeBundleNameV0::{CiDefault, SourceMissing, StyleRecovery, StyleUnused};
    use OmenaCheckerRuleCodeV0::{
        MissingComposedModule, MissingComposedSelector, MissingImportedValue, MissingKeyframes,
        MissingModule, MissingResolvedClassDomain, MissingResolvedClassValues, MissingSassSymbol,
        MissingStaticClass, MissingTemplatePrefix, MissingValueModule, UnusedSelector,
    };

    vec![
        bundle(
            CiDefault,
            &[
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

fn rule(
    code: OmenaCheckerRuleCodeV0,
    category: OmenaCheckerFindingCategoryV0,
    default_severity: OmenaCheckerSeverityV0,
    fixability: OmenaCheckerRuleFixabilityV0,
    presets: &[OmenaCheckerRulePresetV0],
    description: &'static str,
) -> OmenaCheckerRuleDescriptorV0 {
    OmenaCheckerRuleDescriptorV0 {
        code,
        code_name: code.as_str(),
        category,
        category_name: category.as_str(),
        default_severity,
        default_severity_name: default_severity.as_str(),
        fixability,
        fixability_name: fixability.as_str(),
        presets: presets.to_vec(),
        preset_names: presets.iter().map(|preset| preset.as_str()).collect(),
        description,
    }
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
    };

    use super::*;

    #[test]
    fn lists_current_checker_registry_in_stable_ts_order() {
        assert_eq!(
            list_omena_checker_rule_code_names(),
            vec![
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
        assert_eq!(summary.rule_count, 13);
        assert_eq!(summary.source_rule_count, 5);
        assert_eq!(summary.style_rule_count, 8);
        assert_eq!(summary.bundle_count, 4);
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
}
