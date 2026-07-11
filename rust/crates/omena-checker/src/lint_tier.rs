use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::{
    OmenaCheckerRuleCodeV0, OmenaCheckerRuleTierV0, list_omena_checker_rule_codes,
    summarize_omena_checker_rule_enforcement_coverage_v0,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaCheckerLintTierV0 {
    Syntax,
    Semantic,
    SourceAware,
}

impl OmenaCheckerLintTierV0 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Syntax => "syntax",
            Self::Semantic => "semantic",
            Self::SourceAware => "sourceAware",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerLintTierMappingV0 {
    pub rule_code: OmenaCheckerRuleCodeV0,
    pub rule_code_name: &'static str,
    pub checker_tier: OmenaCheckerRuleTierV0,
    pub checker_tier_name: &'static str,
    pub lint_tier: OmenaCheckerLintTierV0,
    pub lint_tier_name: &'static str,
    pub rationale: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerLintTierCoverageV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub registered_rule_count: usize,
    pub mapped_rule_count: usize,
    pub syntax_rule_count: usize,
    pub semantic_rule_count: usize,
    pub source_aware_rule_count: usize,
    pub missing_rule_names: Vec<&'static str>,
    pub extra_rule_names: Vec<&'static str>,
    pub duplicate_rule_names: Vec<&'static str>,
    pub enforcement_coverage_passed: bool,
    pub coverage_passed: bool,
    pub mappings: Vec<OmenaCheckerLintTierMappingV0>,
}

use OmenaCheckerLintTierV0::{Semantic, SourceAware, Syntax};
use OmenaCheckerRuleCodeV0::{
    CascadeDeepConflict, CascadeSMTViolation, CascadeUnreachableRule,
    CategoricalCascadeEvidenceInconsistency, CircularVar, DeadCascadeLayer, DesignSystemMdlBudget,
    DesignerIntentInconsistency, IacvtProne, InvalidPropertyValue, MissingComposedModule,
    MissingComposedSelector, MissingCustomProperty, MissingImportedValue, MissingKeyframes,
    MissingModule, MissingResolvedClassDomain, MissingResolvedClassValues, MissingSassSymbol,
    MissingStaticClass, MissingTemplatePrefix, MissingValueModule, NoImpossibleSelector,
    NoImpreciseValue, NoUnknownDynamicClass, RegisteredPropertyTypeMismatch,
    ReplicaEnsembleInconsistency, RgFlowRelevantOperator, StreamingIfdsPrecisionParity,
    UnreachableDeclaration, UnspecifiedCascadeTie, UnusedSelector,
};

const SOURCE_ABSTRACT_VALUE: &str = "requires source expression or source/style binding facts";
const STYLE_SEMANTICS: &str =
    "requires style resolution, cascade, semantic graph, or mechanism evidence";
const VALUE_SYNTAX: &str = "validates a declaration value against parser-owned property syntax";

const LINT_TIER_MAPPING: &[(OmenaCheckerRuleCodeV0, OmenaCheckerLintTierV0, &str)] = &[
    (NoUnknownDynamicClass, SourceAware, SOURCE_ABSTRACT_VALUE),
    (NoImpreciseValue, SourceAware, SOURCE_ABSTRACT_VALUE),
    (NoImpossibleSelector, SourceAware, SOURCE_ABSTRACT_VALUE),
    (MissingModule, SourceAware, SOURCE_ABSTRACT_VALUE),
    (MissingStaticClass, SourceAware, SOURCE_ABSTRACT_VALUE),
    (MissingTemplatePrefix, SourceAware, SOURCE_ABSTRACT_VALUE),
    (
        MissingResolvedClassValues,
        SourceAware,
        SOURCE_ABSTRACT_VALUE,
    ),
    (
        MissingResolvedClassDomain,
        SourceAware,
        SOURCE_ABSTRACT_VALUE,
    ),
    (UnusedSelector, SourceAware, SOURCE_ABSTRACT_VALUE),
    (MissingComposedModule, Semantic, STYLE_SEMANTICS),
    (MissingComposedSelector, Semantic, STYLE_SEMANTICS),
    (MissingValueModule, Semantic, STYLE_SEMANTICS),
    (MissingImportedValue, Semantic, STYLE_SEMANTICS),
    (MissingKeyframes, Semantic, STYLE_SEMANTICS),
    (MissingCustomProperty, Semantic, STYLE_SEMANTICS),
    (MissingSassSymbol, Semantic, STYLE_SEMANTICS),
    (UnreachableDeclaration, Semantic, STYLE_SEMANTICS),
    (DeadCascadeLayer, Semantic, STYLE_SEMANTICS),
    (IacvtProne, Semantic, STYLE_SEMANTICS),
    (CircularVar, Semantic, STYLE_SEMANTICS),
    (UnspecifiedCascadeTie, Semantic, STYLE_SEMANTICS),
    (DesignerIntentInconsistency, Semantic, STYLE_SEMANTICS),
    (CascadeSMTViolation, Semantic, STYLE_SEMANTICS),
    (DesignSystemMdlBudget, Semantic, STYLE_SEMANTICS),
    (StreamingIfdsPrecisionParity, Semantic, STYLE_SEMANTICS),
    (RgFlowRelevantOperator, Semantic, STYLE_SEMANTICS),
    (ReplicaEnsembleInconsistency, Semantic, STYLE_SEMANTICS),
    (CascadeDeepConflict, Semantic, STYLE_SEMANTICS),
    (CascadeUnreachableRule, Semantic, STYLE_SEMANTICS),
    (
        CategoricalCascadeEvidenceInconsistency,
        Semantic,
        STYLE_SEMANTICS,
    ),
    (RegisteredPropertyTypeMismatch, Semantic, STYLE_SEMANTICS),
    (InvalidPropertyValue, Syntax, VALUE_SYNTAX),
];

pub fn list_omena_checker_lint_tier_mappings_v0() -> Vec<OmenaCheckerLintTierMappingV0> {
    let checker_tiers = super::list_omena_checker_rule_descriptors()
        .into_iter()
        .map(|descriptor| (descriptor.code, descriptor.tier))
        .collect::<BTreeMap<_, _>>();
    LINT_TIER_MAPPING
        .iter()
        .filter_map(|(rule_code, lint_tier, rationale)| {
            checker_tiers.get(rule_code).copied().map(|checker_tier| {
                OmenaCheckerLintTierMappingV0 {
                    rule_code: *rule_code,
                    rule_code_name: rule_code.as_str(),
                    checker_tier,
                    checker_tier_name: checker_tier.as_str(),
                    lint_tier: *lint_tier,
                    lint_tier_name: lint_tier.as_str(),
                    rationale,
                }
            })
        })
        .collect()
}

pub fn lint_tier_for_omena_checker_rule_v0(
    rule_code: OmenaCheckerRuleCodeV0,
) -> Option<OmenaCheckerLintTierV0> {
    LINT_TIER_MAPPING
        .iter()
        .find_map(|(candidate, tier, _)| (*candidate == rule_code).then_some(*tier))
}

pub fn summarize_omena_checker_lint_tier_coverage_v0() -> OmenaCheckerLintTierCoverageV0 {
    summarize_mapping_coverage(list_omena_checker_lint_tier_mappings_v0())
}

fn summarize_mapping_coverage(
    mappings: Vec<OmenaCheckerLintTierMappingV0>,
) -> OmenaCheckerLintTierCoverageV0 {
    let registered = list_omena_checker_rule_codes()
        .into_iter()
        .map(OmenaCheckerRuleCodeV0::as_str)
        .collect::<BTreeSet<_>>();
    let mut occurrences = BTreeMap::<&str, usize>::new();
    for mapping in &mappings {
        *occurrences.entry(mapping.rule_code_name).or_default() += 1;
    }
    let mapped = occurrences.keys().copied().collect::<BTreeSet<_>>();
    let missing_rule_names = registered.difference(&mapped).copied().collect::<Vec<_>>();
    let extra_rule_names = mapped.difference(&registered).copied().collect::<Vec<_>>();
    let duplicate_rule_names = occurrences
        .iter()
        .filter_map(|(rule_name, count)| (*count > 1).then_some(*rule_name))
        .collect::<Vec<_>>();
    let enforcement_coverage_passed =
        summarize_omena_checker_rule_enforcement_coverage_v0().coverage_passed;
    let coverage_passed = enforcement_coverage_passed
        && missing_rule_names.is_empty()
        && extra_rule_names.is_empty()
        && duplicate_rule_names.is_empty()
        && mappings.len() == registered.len();

    OmenaCheckerLintTierCoverageV0 {
        schema_version: "0",
        product: "omena-checker.lint-tier-coverage",
        registered_rule_count: registered.len(),
        mapped_rule_count: mappings.len(),
        syntax_rule_count: count_tier(&mappings, Syntax),
        semantic_rule_count: count_tier(&mappings, Semantic),
        source_aware_rule_count: count_tier(&mappings, SourceAware),
        missing_rule_names,
        extra_rule_names,
        duplicate_rule_names,
        enforcement_coverage_passed,
        coverage_passed,
        mappings,
    }
}

fn count_tier(mappings: &[OmenaCheckerLintTierMappingV0], tier: OmenaCheckerLintTierV0) -> usize {
    mappings
        .iter()
        .filter(|mapping| mapping.lint_tier == tier)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lint_tier_mapping_is_total_unique_and_enforced() {
        let coverage = summarize_omena_checker_lint_tier_coverage_v0();
        assert!(coverage.coverage_passed);
        assert!(coverage.enforcement_coverage_passed);
        assert_eq!(coverage.registered_rule_count, 32);
        assert_eq!(coverage.mapped_rule_count, 32);
        assert_eq!(coverage.syntax_rule_count, 1);
        assert_eq!(coverage.semantic_rule_count, 22);
        assert_eq!(coverage.source_aware_rule_count, 9);
    }

    #[test]
    fn missing_and_duplicate_mappings_fail_coverage() {
        let mut mappings = list_omena_checker_lint_tier_mappings_v0();
        mappings.pop();
        let missing = summarize_mapping_coverage(mappings);
        assert!(!missing.coverage_passed);
        assert_eq!(missing.missing_rule_names, ["invalid-property-value"]);

        let mut mappings = list_omena_checker_lint_tier_mappings_v0();
        if let Some(first) = mappings.first().copied() {
            mappings.push(first);
        }
        let duplicate = summarize_mapping_coverage(mappings);
        assert!(!duplicate.coverage_passed);
        assert_eq!(duplicate.duplicate_rule_names, ["no-unknown-dynamic-class"]);
    }
}
