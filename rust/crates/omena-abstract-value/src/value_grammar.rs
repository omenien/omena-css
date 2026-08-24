use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::{Arc, OnceLock, RwLock};

use omena_cascade::{CascadeStandardValueValidatorV0, CascadeStandardValueVerdictV0};
use omena_evidence_graph::{
    EvidenceNodeKeyV0, EvidenceNodeSeedV0, ExternalToolRunWitnessV0, FamilyStampV0, GuaranteeKindV0,
};
use omena_spec_audit::{
    SpecGrammarBoundaryClassificationV0, SpecGrammarRegistryV0, spec_grammar_registry,
};
use omena_syntax::ident::{
    CanonicalStandardPropertyNameV0, PropertyNameV0, is_custom_property_name,
};
use omena_value_lattice::{
    CssValueComponentKindV0, CssValueComponentV0, DeclarationValueLensV0, ValueNodeV0,
    css_value_component_stream, declaration_value_lens, parse_numeric_value_with_unit,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const CLOSED_WORLD_KEYWORD_CLOSURE_CERTIFICATE_SOURCE: &str =
    include_str!("../data/closed-world-keyword-closure-certificate.json");
const CLOSED_WORLD_BUILTIN_TOKEN_PROFILES_SOURCE: &str =
    include_str!("../data/closed-world-builtin-token-profiles.json");

use crate::{
    AbstractCssTypedScalarValueV0, AbstractCssTypedValueV0, AbstractCssValueV0,
    DeclaredNumericTypeV0, DeclaredValueKindV0, abstract_css_typed_scalar_from_text,
    classify_registered_property_declared_value_v0,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssValueGrammarBudgetV0 {
    pub max_match_steps: usize,
    pub max_reference_depth: usize,
    pub max_states: usize,
}

impl Default for CssValueGrammarBudgetV0 {
    fn default() -> Self {
        Self {
            max_match_steps: 50_000,
            max_reference_depth: 64,
            max_states: 4_096,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CssValueGrammarBudgetKindV0 {
    MatchSteps,
    ReferenceDepth,
    CandidateStates,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssValueGrammarLocusV0 {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum CssValueGrammarVerdictV0 {
    Matched {
        grammar: String,
        consumed_components: usize,
    },
    Unmatched {
        grammar: String,
        locus: CssValueGrammarLocusV0,
    },
    NotMatchedWithinBudget {
        grammar: String,
        locus: CssValueGrammarLocusV0,
        budget: CssValueGrammarBudgetKindV0,
        limit: usize,
        reference: Option<String>,
    },
    GrammarDefect {
        grammar: String,
        offset: usize,
        code: String,
        detail: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CssValueValidationClassV0 {
    Valid,
    Invalid,
    NotValidatable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CssValueValidationReasonV0 {
    GrammarMatched,
    GrammarUnmatched,
    GrammarDefect,
    MatchBudgetExhausted,
    DeferredSubstitution,
    VendorExtension,
    ForwardTierGrammar,
    UnvalidatedStandardFunction,
    MatcherCoverageIncomplete,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssValueValidationV0 {
    pub class: CssValueValidationClassV0,
    pub reason: CssValueValidationReasonV0,
    pub verdict: CssValueGrammarVerdictV0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssValueValidationConsumerPolicyV0 {
    pub consumer: &'static str,
    pub matched: &'static str,
    pub unmatched: &'static str,
    pub forward_tier_unmatched: &'static str,
    pub grammar_defect: &'static str,
    pub budget_exhausted: &'static str,
}

pub const CSS_VALUE_VALIDATION_CONSUMER_POLICIES_V0: [CssValueValidationConsumerPolicyV0; 5] = [
    CssValueValidationConsumerPolicyV0 {
        consumer: "checker.registeredPropertyTypeMismatch",
        matched: "accept",
        unmatched: "diagnostic",
        forward_tier_unmatched: "not-applicable",
        grammar_defect: "silent",
        budget_exhausted: "silent",
    },
    CssValueValidationConsumerPolicyV0 {
        consumer: "checker.invalidPropertyValue",
        matched: "accept",
        unmatched: "diagnostic",
        forward_tier_unmatched: "not-validatable",
        grammar_defect: "silent",
        budget_exhausted: "silent",
    },
    CssValueValidationConsumerPolicyV0 {
        consumer: "cascade.postSubstitutionStandardProperty",
        matched: "accept",
        unmatched: "reject",
        forward_tier_unmatched: "not-validatable",
        grammar_defect: "unknown",
        budget_exhausted: "unknown",
    },
    CssValueValidationConsumerPolicyV0 {
        consumer: "scss.nativeCssFunctionParameter",
        matched: "accept",
        unmatched: "reject",
        forward_tier_unmatched: "not-applicable",
        grammar_defect: "unknown",
        budget_exhausted: "unknown",
    },
    CssValueValidationConsumerPolicyV0 {
        consumer: "scss.nativeCssFunctionReturn",
        matched: "accept",
        unmatched: "reject",
        forward_tier_unmatched: "not-applicable",
        grammar_defect: "unknown",
        budget_exhausted: "unknown",
    },
];

/// Records invocation facts for a development-time value grammar oracle.
/// The external tool remains a witness; it does not determine matcher truth.
pub fn css_value_grammar_external_tool_evidence_v0(
    tool_name: &str,
    tool_version: &str,
    input_digest: &str,
    exit_status: i32,
) -> EvidenceNodeSeedV0 {
    let witness = ExternalToolRunWitnessV0 {
        tool_name: tool_name.to_string(),
        tool_version: tool_version.to_string(),
        input_digest: input_digest.to_string(),
        exit_status,
    };
    EvidenceNodeSeedV0::with_family(
        EvidenceNodeKeyV0::new(
            "omena-abstract-value.value-grammar-differential",
            input_digest,
        ),
        vec![
            format!("externalTool:{tool_name}"),
            format!("toolVersion:{tool_version}"),
            format!("exitStatus:{exit_status}"),
        ],
        GuaranteeKindV0::for_label_less_family(),
        FamilyStampV0::external_tool(&witness),
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssValueGrammarRegistryAuditV0 {
    pub total_entry_count: usize,
    pub parsed_entry_count: usize,
    pub missing_syntax_count: usize,
    pub grammar_defect_count: usize,
    pub categories: Vec<CssValueGrammarCategoryAuditV0>,
    pub defects: Vec<CssValueGrammarDefectV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssValueGrammarCategoryAuditV0 {
    pub category: String,
    pub entry_count: usize,
    pub parsed_entry_count: usize,
    pub missing_syntax_count: usize,
    pub grammar_defect_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssValueGrammarDefectV0 {
    pub category: String,
    pub name: String,
    pub offset: usize,
    pub code: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CssValueGrammarTypedMatchV0<'a> {
    pub verdict: CssValueGrammarVerdictV0,
    pub abstract_value: AbstractCssValueV0,
    pub projection: Option<CssValueTypedProjectionV0<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CssValueTypedProjectionV0<'a> {
    pub lattice: DeclarationValueLensV0<'a>,
    pub scalar_leaves: Vec<AbstractCssTypedScalarValueV0>,
}

impl CssValueGrammarVerdictV0 {
    pub const fn is_matched(&self) -> bool {
        matches!(self, Self::Matched { .. })
    }

    pub const fn is_definite_mismatch(&self) -> bool {
        matches!(self, Self::Unmatched { .. })
    }

    pub const fn is_validatable(&self) -> bool {
        matches!(self, Self::Matched { .. } | Self::Unmatched { .. })
    }
}

/// Parses every grammar supplied by the pinned registry and accounts for every
/// row. Missing source syntax and unsupported grammar shapes remain explicit
/// data instead of disappearing from a coverage percentage.
pub fn audit_css_value_grammar_registry_v0(
    registry: &SpecGrammarRegistryV0,
) -> CssValueGrammarRegistryAuditV0 {
    let mut categories = Vec::new();
    let mut defects = Vec::new();
    let mut parsed_entry_count = 0usize;
    let mut missing_syntax_count = 0usize;
    for category in ["atrules", "functions", "properties", "selectors", "types"] {
        let entries = registry.entries(category);
        let mut category_parsed = 0usize;
        let mut category_missing = 0usize;
        let defect_start = defects.len();
        for entry in entries {
            let Some(grammar) = entry.syntax.as_deref() else {
                category_missing += 1;
                missing_syntax_count += 1;
                continue;
            };
            match VdsParser::new(strip_matching_quotes(grammar.trim())).parse() {
                Ok(_) => {
                    category_parsed += 1;
                    parsed_entry_count += 1;
                }
                Err(error) => defects.push(CssValueGrammarDefectV0 {
                    category: category.to_string(),
                    name: entry.name.clone(),
                    offset: error.offset,
                    code: error.code.to_string(),
                    detail: error.detail,
                }),
            }
        }
        categories.push(CssValueGrammarCategoryAuditV0 {
            category: category.to_string(),
            entry_count: entries.len(),
            parsed_entry_count: category_parsed,
            missing_syntax_count: category_missing,
            grammar_defect_count: defects.len() - defect_start,
        });
    }
    CssValueGrammarRegistryAuditV0 {
        total_entry_count: registry.total_entry_count(),
        parsed_entry_count,
        missing_syntax_count,
        grammar_defect_count: defects.len(),
        categories,
        defects,
    }
}

/// Matches a standard property's value against the grammar supplied by the
/// pinned specification registry.
pub fn match_standard_property_value_v0(property: &str, value: &str) -> CssValueGrammarVerdictV0 {
    let property = PropertyNameV0::from_authored(property);
    match_standard_property_value_with_coverage_v0(&property, value).0
}

fn match_standard_property_value_with_coverage_v0(
    property: &PropertyNameV0,
    value: &str,
) -> (CssValueGrammarVerdictV0, bool) {
    let canonical_property = property.canonical_name();
    let registry = spec_grammar_registry();
    let Some(entry) = registry.entry("properties", canonical_property) else {
        return (
            grammar_defect(
                "",
                0,
                "unknownProperty",
                format!("property {canonical_property:?} is absent from the pinned registry"),
            ),
            false,
        );
    };
    let Some(grammar) = entry.syntax.as_deref() else {
        return (
            grammar_defect(
                "",
                0,
                "missingPropertyGrammar",
                format!("property {canonical_property:?} has no syntax in the pinned registry"),
            ),
            false,
        );
    };
    let matcher_coverage_complete = standard_property_matcher_coverage_complete(property, registry);
    if matches!(
        classify_registered_property_declared_value_v0(value),
        DeclaredValueKindV0::CssWide
    ) {
        return (
            CssValueGrammarVerdictV0::Matched {
                grammar: grammar.to_string(),
                consumed_components: 1,
            },
            matcher_coverage_complete,
        );
    }
    let components = match css_value_component_stream(value, 0) {
        Ok(components) => components,
        Err(error) => {
            return (
                grammar_defect(
                    grammar,
                    error.span.start,
                    "invalidValueTokenStream",
                    error.message,
                ),
                false,
            );
        }
    };
    let normalized = strip_matching_quotes(grammar.trim());
    let expression = match cached_pinned_vds_expression(normalized) {
        Ok(expression) => expression,
        Err(error) => {
            return (
                grammar_defect(grammar, error.offset, error.code, error.detail),
                false,
            );
        }
    };
    (
        match_css_value_grammar_components_with_expression_v0(
            grammar,
            &components,
            registry,
            CssValueGrammarBudgetV0::default(),
            expression.as_ref(),
            true,
        ),
        matcher_coverage_complete,
    )
}

/// Matches a registered custom-property or native-CSS function descriptor.
pub fn match_registered_property_value_v0(syntax: &str, value: &str) -> CssValueGrammarVerdictV0 {
    let grammar = strip_matching_quotes(syntax.trim()).trim();
    if grammar == "*" {
        return match css_value_component_stream(value, 0) {
            Ok(components) => CssValueGrammarVerdictV0::Matched {
                grammar: syntax.to_string(),
                consumed_components: components.len(),
            },
            Err(error) => grammar_defect(
                syntax,
                error.span.start,
                "invalidValueTokenStream",
                error.message,
            ),
        };
    }
    if matches!(
        classify_registered_property_declared_value_v0(value),
        DeclaredValueKindV0::CssWide
    ) {
        return CssValueGrammarVerdictV0::Matched {
            grammar: syntax.to_string(),
            consumed_components: 1,
        };
    }
    match_css_value_grammar_v0(
        grammar,
        value,
        spec_grammar_registry(),
        CssValueGrammarBudgetV0::default(),
    )
}

pub fn validate_standard_property_value_v0(property: &str, value: &str) -> CssValueValidationV0 {
    let property = PropertyNameV0::from_authored(property);
    let canonical_property = property.canonical_name();
    let registry = spec_grammar_registry();
    let classification = registry
        .entry("properties", canonical_property)
        .map(|entry| entry.boundary.classification)
        .unwrap_or(SpecGrammarBoundaryClassificationV0::InBoundary);
    let (verdict, matcher_coverage_complete) =
        match_standard_property_value_with_coverage_v0(&property, value);
    let matcher_coverage_complete = matcher_coverage_complete
        && standard_property_value_token_kinds_have_closure_authority(&property, value, registry);
    let closed_world_token_kind_mismatch =
        matches!(verdict, CssValueGrammarVerdictV0::Unmatched { .. })
            && standard_property_closed_world_token_kind_mismatch(&property, value, registry);
    adjudicate_css_value_validation_with_boundary(
        value,
        verdict,
        classification,
        matcher_coverage_complete,
        closed_world_token_kind_mismatch,
    )
}

/// Product adapter from the spec-derived grammar registry to the cascade
/// computed-value validation port.
#[derive(Debug, Clone, Copy, Default)]
pub struct SpecStandardPropertyValueValidatorV0;

impl CascadeStandardValueValidatorV0 for SpecStandardPropertyValueValidatorV0 {
    fn validate_standard_property_value(
        &self,
        property: &PropertyNameV0,
        value: &str,
    ) -> CascadeStandardValueVerdictV0 {
        match validate_standard_property_value_v0(property.canonical_name(), value).class {
            CssValueValidationClassV0::Valid => CascadeStandardValueVerdictV0::Matched,
            CssValueValidationClassV0::Invalid => CascadeStandardValueVerdictV0::Unmatched,
            CssValueValidationClassV0::NotValidatable => CascadeStandardValueVerdictV0::Unknown,
        }
    }
}

pub fn validate_registered_property_value_v0(syntax: &str, value: &str) -> CssValueValidationV0 {
    adjudicate_css_value_validation(value, match_registered_property_value_v0(syntax, value))
}

fn adjudicate_css_value_validation(
    value: &str,
    verdict: CssValueGrammarVerdictV0,
) -> CssValueValidationV0 {
    adjudicate_css_value_validation_with_boundary(
        value,
        verdict,
        SpecGrammarBoundaryClassificationV0::InBoundary,
        true,
        false,
    )
}

fn adjudicate_css_value_validation_with_boundary(
    value: &str,
    verdict: CssValueGrammarVerdictV0,
    classification: SpecGrammarBoundaryClassificationV0,
    matcher_coverage_complete: bool,
    closed_world_token_kind_mismatch: bool,
) -> CssValueValidationV0 {
    let components = css_value_component_stream(value, 0).ok();
    let has_unvalidated_standard_function = matches!(
        &verdict,
        CssValueGrammarVerdictV0::Unmatched { grammar, locus }
            if classification == SpecGrammarBoundaryClassificationV0::InBoundary
                && components.as_deref().is_some_and(|components| {
                    recognized_standard_functions_explain_unmatched_value(
                        grammar,
                        components,
                        *locus,
                    )
                })
    );
    let (class, reason) = if components
        .as_deref()
        .is_some_and(contains_deferred_css_value)
    {
        (
            CssValueValidationClassV0::NotValidatable,
            CssValueValidationReasonV0::DeferredSubstitution,
        )
    } else if components
        .as_deref()
        .is_some_and(has_leading_vendor_identifier)
    {
        (
            CssValueValidationClassV0::NotValidatable,
            CssValueValidationReasonV0::VendorExtension,
        )
    } else if has_unvalidated_standard_function {
        (
            CssValueValidationClassV0::NotValidatable,
            CssValueValidationReasonV0::UnvalidatedStandardFunction,
        )
    } else {
        match verdict {
            CssValueGrammarVerdictV0::Matched { .. } => (
                CssValueValidationClassV0::Valid,
                CssValueValidationReasonV0::GrammarMatched,
            ),
            CssValueGrammarVerdictV0::Unmatched { .. }
                if classification == SpecGrammarBoundaryClassificationV0::ForwardTier =>
            {
                (
                    CssValueValidationClassV0::NotValidatable,
                    CssValueValidationReasonV0::ForwardTierGrammar,
                )
            }
            CssValueGrammarVerdictV0::Unmatched { .. }
                if !matcher_coverage_complete && !closed_world_token_kind_mismatch =>
            {
                (
                    CssValueValidationClassV0::NotValidatable,
                    CssValueValidationReasonV0::MatcherCoverageIncomplete,
                )
            }
            CssValueGrammarVerdictV0::Unmatched { .. } => (
                CssValueValidationClassV0::Invalid,
                CssValueValidationReasonV0::GrammarUnmatched,
            ),
            CssValueGrammarVerdictV0::NotMatchedWithinBudget { .. } => (
                CssValueValidationClassV0::NotValidatable,
                CssValueValidationReasonV0::MatchBudgetExhausted,
            ),
            CssValueGrammarVerdictV0::GrammarDefect { .. } => (
                CssValueValidationClassV0::NotValidatable,
                CssValueValidationReasonV0::GrammarDefect,
            ),
        }
    };
    CssValueValidationV0 {
        class,
        reason,
        verdict,
    }
}

fn recognized_standard_functions_explain_unmatched_value(
    grammar: &str,
    components: &[CssValueComponentV0],
    locus: CssValueGrammarLocusV0,
) -> bool {
    let has_locus_function = components.iter().any(|component| {
        component.span.start < locus.end
            && locus.start < component.span.end
            && component_is_recognized_standard_function(component)
    });
    if !has_locus_function {
        return false;
    }

    let normalized = strip_matching_quotes(grammar.trim());
    let Ok(expression) = cached_pinned_vds_expression(normalized) else {
        return false;
    };
    let mut context = MatchContext {
        registry: spec_grammar_registry(),
        budget: CssValueGrammarBudgetV0::default(),
        match_steps: 0,
        first_stop: None,
        grammar_cache: HashMap::new(),
        cache_registered_grammars: true,
        allow_unvalidated_standard_function_references: true,
    };
    context
        .match_expression(expression.as_ref(), components, 0, 0)
        .contains(&components.len())
}

fn component_is_recognized_standard_function(component: &CssValueComponentV0) -> bool {
    matches!(
        &component.kind,
        CssValueComponentKindV0::Function { name, .. }
            if recognized_standard_function_names().contains(name)
    )
}

fn recognized_standard_function_names() -> &'static BTreeSet<String> {
    static NAMES: OnceLock<BTreeSet<String>> = OnceLock::new();
    NAMES.get_or_init(|| {
        spec_grammar_registry()
            .entries("functions")
            .iter()
            .filter(|entry| {
                entry.boundary.classification == SpecGrammarBoundaryClassificationV0::InBoundary
            })
            .filter_map(|entry| entry.name.strip_suffix("()"))
            .filter(|name| !name.starts_with('-') && !is_deferred_css_function_name(name))
            .map(str::to_string)
            .collect()
    })
}

fn is_deferred_css_function_name(name: &str) -> bool {
    matches!(name, "var" | "env" | "attr")
}

fn contains_deferred_css_value(components: &[CssValueComponentV0]) -> bool {
    components.iter().any(|component| match &component.kind {
        CssValueComponentKindV0::Function { name, arguments } => {
            is_deferred_css_function_name(name) || contains_deferred_css_value(arguments)
        }
        CssValueComponentKindV0::Parenthesized { values }
        | CssValueComponentKindV0::Bracketed { values }
        | CssValueComponentKindV0::Braced { values } => contains_deferred_css_value(values),
        CssValueComponentKindV0::Ident
        | CssValueComponentKindV0::Number
        | CssValueComponentKindV0::Percentage
        | CssValueComponentKindV0::Dimension
        | CssValueComponentKindV0::Hash
        | CssValueComponentKindV0::String
        | CssValueComponentKindV0::Url
        | CssValueComponentKindV0::Comma
        | CssValueComponentKindV0::Slash
        | CssValueComponentKindV0::Delimiter => false,
    })
}

fn has_leading_vendor_identifier(components: &[CssValueComponentV0]) -> bool {
    components.first().is_some_and(|component| {
        matches!(component.kind, CssValueComponentKindV0::Ident) && component.text.starts_with('-')
    })
}

/// Matches and projects a standard property value into the existing scalar
/// typed domain plus the existing value-lattice list/function topology.
pub fn match_and_type_standard_property_value_v0<'a>(
    property: &str,
    value: &'a str,
) -> CssValueGrammarTypedMatchV0<'a> {
    typed_match_result(match_standard_property_value_v0(property, value), value)
}

/// Property-independent typed projection for custom grammar consumers.
pub fn match_and_type_css_value_grammar_v0<'a>(
    grammar: &str,
    value: &'a str,
    registry: &SpecGrammarRegistryV0,
    budget: CssValueGrammarBudgetV0,
) -> CssValueGrammarTypedMatchV0<'a> {
    typed_match_result(
        match_css_value_grammar_v0(grammar, value, registry, budget),
        value,
    )
}

fn typed_match_result<'a>(
    verdict: CssValueGrammarVerdictV0,
    value: &'a str,
) -> CssValueGrammarTypedMatchV0<'a> {
    if !verdict.is_matched() {
        return CssValueGrammarTypedMatchV0 {
            verdict,
            abstract_value: AbstractCssValueV0::Raw {
                value: value.to_string(),
            },
            projection: None,
        };
    }
    let components = match css_value_component_stream(value, 0) {
        Ok(components) => components,
        Err(error) => {
            return CssValueGrammarTypedMatchV0 {
                verdict: grammar_defect(
                    verdict_grammar(&verdict),
                    error.span.start,
                    "typedProjectionTokenStreamDrift",
                    error.message,
                ),
                abstract_value: AbstractCssValueV0::Raw {
                    value: value.to_string(),
                },
                projection: None,
            };
        }
    };
    let mut scalar_leaves = Vec::new();
    collect_typed_scalar_leaves(&components, &mut scalar_leaves);
    let lattice = declaration_value_lens(value, 0);
    let typed = typed_value_from_projection(&lattice, &scalar_leaves).map(Box::new);
    CssValueGrammarTypedMatchV0 {
        verdict,
        abstract_value: AbstractCssValueV0::Exact {
            value: value.to_string(),
            typed,
        },
        projection: Some(CssValueTypedProjectionV0 {
            lattice,
            scalar_leaves,
        }),
    }
}

fn verdict_grammar(verdict: &CssValueGrammarVerdictV0) -> &str {
    match verdict {
        CssValueGrammarVerdictV0::Matched { grammar, .. }
        | CssValueGrammarVerdictV0::Unmatched { grammar, .. }
        | CssValueGrammarVerdictV0::NotMatchedWithinBudget { grammar, .. }
        | CssValueGrammarVerdictV0::GrammarDefect { grammar, .. } => grammar,
    }
}

fn collect_typed_scalar_leaves(
    components: &[CssValueComponentV0],
    leaves: &mut Vec<AbstractCssTypedScalarValueV0>,
) {
    for component in components {
        if let Some(value) = abstract_css_typed_scalar_from_text(component.text.as_str()) {
            leaves.push(value);
            continue;
        }
        match &component.kind {
            CssValueComponentKindV0::Function { arguments, .. }
            | CssValueComponentKindV0::Parenthesized { values: arguments }
            | CssValueComponentKindV0::Bracketed { values: arguments }
            | CssValueComponentKindV0::Braced { values: arguments } => {
                collect_typed_scalar_leaves(arguments, leaves);
            }
            CssValueComponentKindV0::Ident
            | CssValueComponentKindV0::Number
            | CssValueComponentKindV0::Percentage
            | CssValueComponentKindV0::Dimension
            | CssValueComponentKindV0::Hash
            | CssValueComponentKindV0::String
            | CssValueComponentKindV0::Url
            | CssValueComponentKindV0::Comma
            | CssValueComponentKindV0::Slash
            | CssValueComponentKindV0::Delimiter => {}
        }
    }
}

fn typed_value_from_projection(
    lattice: &DeclarationValueLensV0<'_>,
    scalar_leaves: &[AbstractCssTypedScalarValueV0],
) -> Option<AbstractCssTypedValueV0> {
    match (lattice.root(), scalar_leaves) {
        (ValueNodeV0::List { .. } | ValueNodeV0::Function { .. }, [_, ..]) | (_, [_, _, ..]) => {
            Some(AbstractCssTypedValueV0::Compound {
                leaves: scalar_leaves.to_vec(),
            })
        }
        (_, [value]) => Some(AbstractCssTypedValueV0::Exact {
            value: value.clone(),
        }),
        (_, []) => None,
    }
}

/// Matches a value against one CSS Value Definition Syntax expression.
pub fn match_css_value_grammar_v0(
    grammar: &str,
    value: &str,
    registry: &SpecGrammarRegistryV0,
    budget: CssValueGrammarBudgetV0,
) -> CssValueGrammarVerdictV0 {
    let components = match css_value_component_stream(value, 0) {
        Ok(components) => components,
        Err(error) => {
            return grammar_defect(
                grammar,
                error.span.start,
                "invalidValueTokenStream",
                error.message,
            );
        }
    };
    match_css_value_grammar_components_v0(grammar, &components, registry, budget)
}

/// Property-independent matcher entry point over an already tokenized value.
pub fn match_css_value_grammar_components_v0(
    grammar: &str,
    components: &[CssValueComponentV0],
    registry: &SpecGrammarRegistryV0,
    budget: CssValueGrammarBudgetV0,
) -> CssValueGrammarVerdictV0 {
    let normalized = strip_matching_quotes(grammar.trim());
    let expression = match VdsParser::new(normalized).parse() {
        Ok(expression) => expression,
        Err(error) => {
            return grammar_defect(grammar, error.offset, error.code, error.detail);
        }
    };
    match_css_value_grammar_components_with_expression_v0(
        grammar,
        components,
        registry,
        budget,
        &expression,
        false,
    )
}

fn match_css_value_grammar_components_with_expression_v0(
    grammar: &str,
    components: &[CssValueComponentV0],
    registry: &SpecGrammarRegistryV0,
    budget: CssValueGrammarBudgetV0,
    expression: &VdsExpression,
    cache_registered_grammars: bool,
) -> CssValueGrammarVerdictV0 {
    let locus = component_locus(components);
    let mut context = MatchContext {
        registry,
        budget,
        match_steps: 0,
        first_stop: None,
        grammar_cache: HashMap::new(),
        cache_registered_grammars,
        allow_unvalidated_standard_function_references: false,
    };
    let ends = context.match_expression(expression, components, 0, 0);
    if ends.contains(&components.len()) {
        return CssValueGrammarVerdictV0::Matched {
            grammar: grammar.to_string(),
            consumed_components: components.len(),
        };
    }
    if let Some(stop) = context.first_stop {
        return match stop {
            MatchStop::Budget {
                kind,
                limit,
                reference,
            } => CssValueGrammarVerdictV0::NotMatchedWithinBudget {
                grammar: grammar.to_string(),
                locus,
                budget: kind,
                limit,
                reference,
            },
            MatchStop::GrammarDefect {
                offset,
                code,
                detail,
            } => grammar_defect(grammar, offset, code, detail),
        };
    }
    CssValueGrammarVerdictV0::Unmatched {
        grammar: grammar.to_string(),
        locus,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum VdsExpression {
    Literal(String),
    Reference(VdsReference),
    Function {
        name: String,
        arguments: Box<VdsExpression>,
    },
    Sequence(Vec<VdsExpression>),
    AllInAnyOrder(Vec<VdsExpression>),
    OneOrMoreInAnyOrder(Vec<VdsExpression>),
    Choice(Vec<VdsExpression>),
    Repeat {
        expression: Box<VdsExpression>,
        min: usize,
        max: Option<usize>,
        comma_separated: bool,
    },
    Required(Box<VdsExpression>),
}

fn standard_property_matcher_coverage_complete(
    property: &PropertyNameV0,
    registry: &SpecGrammarRegistryV0,
) -> bool {
    let Some(grammar) = registry.syntax("properties", property.canonical_name()) else {
        return false;
    };
    let Ok(expression) = cached_pinned_vds_expression(strip_matching_quotes(grammar.trim())) else {
        return false;
    };
    let mut visiting = HashSet::new();
    let mut memo = HashMap::new();
    expression_matcher_coverage_complete(expression.as_ref(), registry, &mut visiting, &mut memo)
}

fn expression_matcher_coverage_complete(
    expression: &VdsExpression,
    registry: &SpecGrammarRegistryV0,
    visiting: &mut HashSet<(ReferenceCategory, String)>,
    memo: &mut HashMap<(ReferenceCategory, String), bool>,
) -> bool {
    match expression {
        VdsExpression::Literal(_) => true,
        VdsExpression::Reference(reference) => {
            if is_builtin_reference_name(reference.name.as_str()) {
                return builtin_reference_matcher_coverage_complete(reference.name.as_str());
            }
            let key = (reference.category, reference.name.clone());
            if let Some(complete) = memo.get(&key) {
                return *complete;
            }
            if !visiting.insert(key.clone()) {
                return true;
            }
            let category = match reference.category {
                ReferenceCategory::Type => "types",
                ReferenceCategory::Property => "properties",
                ReferenceCategory::Function => "functions",
            };
            let complete = registry
                .syntax(category, reference.name.as_str())
                .and_then(|source| cached_pinned_vds_expression(source).ok())
                .is_some_and(|expression| {
                    expression_matcher_coverage_complete(
                        expression.as_ref(),
                        registry,
                        visiting,
                        memo,
                    )
                });
            visiting.remove(&key);
            memo.insert(key, complete);
            complete
        }
        VdsExpression::Function { arguments, .. }
        | VdsExpression::Repeat {
            expression: arguments,
            ..
        }
        | VdsExpression::Required(arguments) => {
            expression_matcher_coverage_complete(arguments, registry, visiting, memo)
        }
        VdsExpression::Sequence(expressions)
        | VdsExpression::AllInAnyOrder(expressions)
        | VdsExpression::OneOrMoreInAnyOrder(expressions)
        | VdsExpression::Choice(expressions) => expressions.iter().all(|expression| {
            expression_matcher_coverage_complete(expression, registry, visiting, memo)
        }),
    }
}

fn builtin_reference_matcher_coverage_complete(name: &str) -> bool {
    // Only lexical token families and structurally unrestricted values are
    // certified here. Numeric dimensions admit a wider CSS math language than
    // the bounded positive matcher below, while semantic families such as
    // images, colors, custom identifiers, and transforms have extra validity
    // rules. Their positive matches are useful, but their negative matches
    // cannot certify a standards-level rejection.
    matches!(
        name,
        "declaration-value"
            | "any-value"
            | "whole-value"
            | "number-token"
            | "percentage-token"
            | "ident"
            | "ident-token"
            | "dashed-ident"
            | "custom-property-name"
            | "string"
            | "string-token"
            | "url"
            | "url-token"
            | "hex-color"
            | "zero"
            | "dimension-token"
            | "hash-token"
            | "function-token"
            | "comma-token"
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClosedWorldTokenKindV0 {
    Ident,
    Hash,
    Dimension,
    Number,
    Percentage,
    FunctionName,
    String,
    Url,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ClosedWorldTokenDomainV0 {
    open: bool,
    allowed: BTreeSet<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ClosedWorldTokenProfileV0 {
    ident: ClosedWorldTokenDomainV0,
    hash: ClosedWorldTokenDomainV0,
    dimension: ClosedWorldTokenDomainV0,
    number: ClosedWorldTokenDomainV0,
    percentage: ClosedWorldTokenDomainV0,
    function_name: ClosedWorldTokenDomainV0,
    string: ClosedWorldTokenDomainV0,
    url: ClosedWorldTokenDomainV0,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClosedWorldKeywordClosureCertificateV0 {
    schema_version: String,
    product: String,
    oracle: ClosedWorldKeywordClosureOracleV0,
    source: String,
    maximum_reference_depth: usize,
    property_count: usize,
    candidate_pair_count: usize,
    accepted_pair_count: usize,
    matched_pair_count: usize,
    matcher_gap_count: usize,
    accepted_pair_digest: String,
    certified_properties: BTreeSet<String>,
    property_tests: Vec<ClosedWorldKeywordClosurePropertyTestV0>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClosedWorldKeywordClosurePropertyTestV0 {
    property: String,
    candidate_pair_count: usize,
    tested_pair_count: usize,
    matched_pair_count: usize,
    matcher_gap_count: usize,
    accepted_keywords: Vec<String>,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct ClosedWorldKeywordAuthorityV0 {
    certified_properties: BTreeSet<CanonicalStandardPropertyNameV0>,
    accepted_keywords_by_property: BTreeMap<CanonicalStandardPropertyNameV0, BTreeSet<String>>,
}

#[derive(Debug, Deserialize)]
struct ClosedWorldKeywordClosureOracleV0 {
    name: ClosedWorldKeywordClosureOracleNameV0,
    version: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
enum ClosedWorldKeywordClosureOracleNameV0 {
    #[serde(rename = "css-tree")]
    CssTree,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClosedWorldBuiltinTokenProfilesV0 {
    schema_version: String,
    product: String,
    oracle: ClosedWorldKeywordClosureOracleV0,
    profile_count: usize,
    profiles: Vec<ClosedWorldBuiltinTokenProfileV0>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClosedWorldBuiltinTokenProfileV0 {
    name: String,
    authority: String,
    open_token_kinds: BTreeSet<String>,
    allowed_values: HashMap<String, Vec<String>>,
}

impl ClosedWorldTokenProfileV0 {
    fn domain(&self, kind: ClosedWorldTokenKindV0) -> &ClosedWorldTokenDomainV0 {
        match kind {
            ClosedWorldTokenKindV0::Ident => &self.ident,
            ClosedWorldTokenKindV0::Hash => &self.hash,
            ClosedWorldTokenKindV0::Dimension => &self.dimension,
            ClosedWorldTokenKindV0::Number => &self.number,
            ClosedWorldTokenKindV0::Percentage => &self.percentage,
            ClosedWorldTokenKindV0::FunctionName => &self.function_name,
            ClosedWorldTokenKindV0::String => &self.string,
            ClosedWorldTokenKindV0::Url => &self.url,
        }
    }

    fn domain_mut(&mut self, kind: ClosedWorldTokenKindV0) -> &mut ClosedWorldTokenDomainV0 {
        match kind {
            ClosedWorldTokenKindV0::Ident => &mut self.ident,
            ClosedWorldTokenKindV0::Hash => &mut self.hash,
            ClosedWorldTokenKindV0::Dimension => &mut self.dimension,
            ClosedWorldTokenKindV0::Number => &mut self.number,
            ClosedWorldTokenKindV0::Percentage => &mut self.percentage,
            ClosedWorldTokenKindV0::FunctionName => &mut self.function_name,
            ClosedWorldTokenKindV0::String => &mut self.string,
            ClosedWorldTokenKindV0::Url => &mut self.url,
        }
    }

    fn allow(&mut self, kind: ClosedWorldTokenKindV0, value: &str) {
        self.domain_mut(kind)
            .allowed
            .insert(value.to_ascii_lowercase());
    }

    fn mark_open(&mut self, kind: ClosedWorldTokenKindV0) {
        self.domain_mut(kind).open = true;
    }

    fn mark_all_open(&mut self) {
        for kind in CLOSED_WORLD_TOKEN_KINDS {
            self.mark_open(kind);
        }
    }

    fn merge(&mut self, other: Self) {
        for kind in CLOSED_WORLD_TOKEN_KINDS {
            let other_domain = other.domain(kind);
            let domain = self.domain_mut(kind);
            domain.open |= other_domain.open;
            domain.allowed.extend(other_domain.allowed.iter().cloned());
        }
    }
}

const CLOSED_WORLD_TOKEN_KINDS: [ClosedWorldTokenKindV0; 8] = [
    ClosedWorldTokenKindV0::Ident,
    ClosedWorldTokenKindV0::Hash,
    ClosedWorldTokenKindV0::Dimension,
    ClosedWorldTokenKindV0::Number,
    ClosedWorldTokenKindV0::Percentage,
    ClosedWorldTokenKindV0::FunctionName,
    ClosedWorldTokenKindV0::String,
    ClosedWorldTokenKindV0::Url,
];

fn certified_keyword_properties() -> &'static BTreeSet<CanonicalStandardPropertyNameV0> {
    static EMPTY: OnceLock<BTreeSet<CanonicalStandardPropertyNameV0>> = OnceLock::new();
    closed_world_keyword_authority()
        .map(|authority| &authority.certified_properties)
        .unwrap_or_else(|| EMPTY.get_or_init(BTreeSet::new))
}

fn closed_world_keyword_authority() -> Option<&'static ClosedWorldKeywordAuthorityV0> {
    static AUTHORITY: OnceLock<Option<ClosedWorldKeywordAuthorityV0>> = OnceLock::new();
    AUTHORITY
        .get_or_init(|| {
            parse_closed_world_keyword_closure_certificate(
                CLOSED_WORLD_KEYWORD_CLOSURE_CERTIFICATE_SOURCE,
            )
        })
        .as_ref()
}

fn parse_closed_world_keyword_closure_certificate(
    source: &str,
) -> Option<ClosedWorldKeywordAuthorityV0> {
    let certificate =
        serde_json::from_str::<ClosedWorldKeywordClosureCertificateV0>(source).ok()?;
    if certificate.schema_version != "0"
        || certificate.product != "omena-abstract-value.closed-world-keyword-closure-certificate"
        || certificate.oracle.name != ClosedWorldKeywordClosureOracleNameV0::CssTree
        || certificate.oracle.version != "3.2.1"
        || certificate.source != "cssTree.lexer.properties.typeAndPropertyReferenceClosure"
        || certificate.maximum_reference_depth != 12
        || certificate.property_count != 704
        || certificate.candidate_pair_count != 23_178
        || certificate.accepted_pair_count != 16_445
        || certificate.property_count != certificate.property_tests.len()
    {
        return None;
    }

    let mut previous_property: Option<&str> = None;
    let mut candidate_pair_count = 0usize;
    let mut accepted_pair_count = 0usize;
    let mut matched_pair_count = 0usize;
    let mut matcher_gap_count = 0usize;
    let mut derived_certified_properties = BTreeSet::new();
    let mut accepted_keywords_by_property = BTreeMap::new();
    let mut digest = Sha256::new();

    for property_test in &certificate.property_tests {
        if property_test.property.is_empty()
            || previous_property.is_some_and(|previous| previous >= property_test.property.as_str())
            || property_test.tested_pair_count != property_test.accepted_keywords.len()
            || property_test
                .matched_pair_count
                .checked_add(property_test.matcher_gap_count)
                != Some(property_test.tested_pair_count)
            || property_test.candidate_pair_count < property_test.tested_pair_count
            || property_test
                .accepted_keywords
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
            || property_test
                .accepted_keywords
                .iter()
                .any(|keyword| keyword.is_empty())
        {
            return None;
        }
        previous_property = Some(&property_test.property);
        candidate_pair_count =
            candidate_pair_count.checked_add(property_test.candidate_pair_count)?;
        accepted_pair_count = accepted_pair_count.checked_add(property_test.tested_pair_count)?;
        matched_pair_count = matched_pair_count.checked_add(property_test.matched_pair_count)?;
        matcher_gap_count = matcher_gap_count.checked_add(property_test.matcher_gap_count)?;

        for keyword in &property_test.accepted_keywords {
            digest.update(property_test.property.as_bytes());
            digest.update([0]);
            digest.update(keyword.as_bytes());
            digest.update([b'\n']);
        }
        let property_key = PropertyNameV0::canonical_standard_key(&property_test.property);
        if accepted_keywords_by_property
            .insert(
                property_key,
                property_test.accepted_keywords.iter().cloned().collect(),
            )
            .is_some()
        {
            return None;
        }
        if property_test.tested_pair_count > 0 && property_test.matcher_gap_count == 0 {
            derived_certified_properties.insert(property_test.property.clone());
        }
    }

    let accepted_pair_digest = digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    if certificate.candidate_pair_count != candidate_pair_count
        || certificate.accepted_pair_count != accepted_pair_count
        || certificate.matched_pair_count != matched_pair_count
        || certificate.matcher_gap_count != matcher_gap_count
        || certificate.accepted_pair_digest != accepted_pair_digest
        || certificate.certified_properties != derived_certified_properties
    {
        return None;
    }

    Some(ClosedWorldKeywordAuthorityV0 {
        certified_properties: certificate
            .certified_properties
            .into_iter()
            .map(PropertyNameV0::canonical_standard_key)
            .collect(),
        accepted_keywords_by_property,
    })
}

fn standard_property_value_token_kinds_have_closure_authority(
    property: &PropertyNameV0,
    value: &str,
    registry: &SpecGrammarRegistryV0,
) -> bool {
    let Ok(components) = css_value_component_stream(value, 0) else {
        return false;
    };
    if components
        .iter()
        .any(|component| matches!(component.kind, CssValueComponentKindV0::Ident))
    {
        let Some(property_key) = property.as_standard_key() else {
            return false;
        };
        return certified_keyword_properties().contains(property_key);
    }
    standard_property_grammar_is_machine_imported_without_overrides(property, registry)
}

fn standard_property_grammar_is_machine_imported_without_overrides(
    property: &PropertyNameV0,
    registry: &SpecGrammarRegistryV0,
) -> bool {
    let Some(entry) = registry.entry("properties", property.canonical_name()) else {
        return false;
    };
    let Some(source) = entry.syntax.as_deref() else {
        return false;
    };
    if entry.override_provenance.is_some() {
        return false;
    }
    let Ok(expression) = cached_pinned_vds_expression(strip_matching_quotes(source.trim())) else {
        return false;
    };
    let mut visiting = HashSet::new();
    expression_sources_are_machine_imported_without_overrides(
        expression.as_ref(),
        registry,
        &mut visiting,
    )
}

fn expression_sources_are_machine_imported_without_overrides(
    expression: &VdsExpression,
    registry: &SpecGrammarRegistryV0,
    visiting: &mut HashSet<(ReferenceCategory, String)>,
) -> bool {
    match expression {
        VdsExpression::Literal(_) => true,
        VdsExpression::Reference(reference) => {
            if is_builtin_reference_name(reference.name.as_str()) {
                return closed_world_builtin_token_profiles().is_some_and(|manifest| {
                    manifest
                        .profiles
                        .iter()
                        .any(|profile| profile.name == reference.name)
                });
            }
            let key = (reference.category, reference.name.clone());
            if !visiting.insert(key.clone()) {
                return true;
            }
            let category = match reference.category {
                ReferenceCategory::Type => "types",
                ReferenceCategory::Property => "properties",
                ReferenceCategory::Function => "functions",
            };
            let imported = registry
                .entry(category, reference.name.as_str())
                .filter(|entry| entry.override_provenance.is_none())
                .and_then(|entry| entry.syntax.as_deref())
                .and_then(|source| cached_pinned_vds_expression(source).ok())
                .is_some_and(|expression| {
                    expression_sources_are_machine_imported_without_overrides(
                        expression.as_ref(),
                        registry,
                        visiting,
                    )
                });
            visiting.remove(&key);
            imported
        }
        VdsExpression::Function { arguments, .. }
        | VdsExpression::Repeat {
            expression: arguments,
            ..
        }
        | VdsExpression::Required(arguments) => {
            expression_sources_are_machine_imported_without_overrides(arguments, registry, visiting)
        }
        VdsExpression::Sequence(expressions)
        | VdsExpression::AllInAnyOrder(expressions)
        | VdsExpression::OneOrMoreInAnyOrder(expressions)
        | VdsExpression::Choice(expressions) => expressions.iter().all(|expression| {
            expression_sources_are_machine_imported_without_overrides(
                expression, registry, visiting,
            )
        }),
    }
}

fn standard_property_closed_world_token_kind_mismatch(
    property: &PropertyNameV0,
    value: &str,
    registry: &SpecGrammarRegistryV0,
) -> bool {
    let Ok(components) = css_value_component_stream(value, 0) else {
        return false;
    };
    let Some(profile) = cached_standard_property_closed_world_token_profile(property, registry)
    else {
        return false;
    };
    if components.iter().any(|component| {
        closed_world_component_identity(component).is_some_and(|(kind, value)| {
            let domain = profile.domain(kind);
            kind == ClosedWorldTokenKindV0::FunctionName
                && domain.open
                && !domain.allowed.contains(value.as_str())
        })
    }) {
        return false;
    }
    components.iter().any(|component| {
        let Some((kind, value)) = closed_world_component_identity(component) else {
            return false;
        };
        if kind == ClosedWorldTokenKindV0::Ident {
            let Some(property_key) = property.as_standard_key() else {
                return false;
            };
            let Some(accepted_keywords) = closed_world_keyword_authority()
                .and_then(|authority| authority.accepted_keywords_by_property.get(property_key))
            else {
                return false;
            };
            let domain = profile.domain(kind);
            return !domain.open && !accepted_keywords.contains(value.as_str());
        }
        let domain = profile.domain(kind);
        !domain.open && !domain.allowed.contains(value.as_str())
    })
}

fn cached_standard_property_closed_world_token_profile(
    property: &PropertyNameV0,
    registry: &SpecGrammarRegistryV0,
) -> Option<ClosedWorldTokenProfileV0> {
    static CACHE: OnceLock<RwLock<HashMap<String, Option<ClosedWorldTokenProfileV0>>>> =
        OnceLock::new();
    let cache = CACHE.get_or_init(|| RwLock::new(HashMap::new()));
    let key = property.canonical_name().to_string();
    if let Some(profile) = cache
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(&key)
    {
        return profile.clone();
    }
    let profile = registry
        .entry("properties", property.canonical_name())
        .and_then(|entry| entry.syntax.as_deref().map(|grammar| (entry, grammar)))
        .and_then(|(entry, grammar)| {
            cached_pinned_vds_expression(strip_matching_quotes(grammar.trim()))
                .ok()
                .map(|expression| (entry, expression))
        })
        .map(|(entry, expression)| {
            let mut ident_visiting = HashSet::new();
            let mut ident_memo = HashMap::new();
            let ident_open = expression_has_open_ident_production(
                expression.as_ref(),
                registry,
                &mut ident_visiting,
                &mut ident_memo,
            );
            let mut visiting = HashSet::new();
            let mut memo = HashMap::new();
            let mut profile =
                closed_world_token_profile(expression.as_ref(), registry, &mut visiting, &mut memo);
            if entry.override_provenance.is_some() {
                profile.mark_all_open();
            }
            // A reviewed syntax replacement is conservative for the generic
            // token-kind profile, but it does not itself introduce an open
            // identifier production. Identifier rejection uses the expanded
            // grammar plus the independently authenticated accepted-keyword
            // table, so preserve that narrower fact after the broad fallback.
            profile.ident.open = ident_open;
            profile
        });
    cache
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .entry(key)
        .or_insert_with(|| profile.clone());
    profile
}

fn expression_has_open_ident_production(
    expression: &VdsExpression,
    registry: &SpecGrammarRegistryV0,
    visiting: &mut HashSet<(ReferenceCategory, String)>,
    memo: &mut HashMap<(ReferenceCategory, String), bool>,
) -> bool {
    match expression {
        VdsExpression::Literal(_) | VdsExpression::Function { .. } => false,
        VdsExpression::Reference(reference) => {
            reference_has_open_ident_production(reference, registry, visiting, memo)
        }
        VdsExpression::Sequence(expressions)
        | VdsExpression::AllInAnyOrder(expressions)
        | VdsExpression::OneOrMoreInAnyOrder(expressions)
        | VdsExpression::Choice(expressions) => expressions.iter().any(|expression| {
            expression_has_open_ident_production(expression, registry, visiting, memo)
        }),
        VdsExpression::Repeat { expression, .. } | VdsExpression::Required(expression) => {
            expression_has_open_ident_production(expression, registry, visiting, memo)
        }
    }
}

fn reference_has_open_ident_production(
    reference: &VdsReference,
    registry: &SpecGrammarRegistryV0,
    visiting: &mut HashSet<(ReferenceCategory, String)>,
    memo: &mut HashMap<(ReferenceCategory, String), bool>,
) -> bool {
    if reference.category == ReferenceCategory::Function {
        return false;
    }
    if reference.category == ReferenceCategory::Type
        && let Some(profile) = closed_world_builtin_profile(reference.name.as_str())
    {
        return profile.ident.open;
    }

    let key = (reference.category, reference.name.clone());
    if let Some(open) = memo.get(&key) {
        return *open;
    }
    if !visiting.insert(key.clone()) {
        return true;
    }
    let category = match reference.category {
        ReferenceCategory::Type => "types",
        ReferenceCategory::Property => "properties",
        ReferenceCategory::Function => unreachable!("function references return above"),
    };
    let open = registry
        .entry(category, reference.name.as_str())
        .and_then(|entry| entry.syntax.as_deref())
        .and_then(|source| cached_pinned_vds_expression(source).ok())
        .map(|expression| {
            expression_has_open_ident_production(expression.as_ref(), registry, visiting, memo)
        })
        .unwrap_or(true);
    visiting.remove(&key);
    memo.insert(key, open);
    open
}

fn closed_world_token_profile(
    expression: &VdsExpression,
    registry: &SpecGrammarRegistryV0,
    visiting: &mut HashSet<(ReferenceCategory, String)>,
    memo: &mut HashMap<(ReferenceCategory, String), ClosedWorldTokenProfileV0>,
) -> ClosedWorldTokenProfileV0 {
    match expression {
        VdsExpression::Literal(literal) => closed_world_literal_profile(literal),
        VdsExpression::Reference(reference) => {
            closed_world_reference_profile(reference, registry, visiting, memo)
        }
        VdsExpression::Function { name, .. } => {
            let mut profile = ClosedWorldTokenProfileV0::default();
            profile.allow(ClosedWorldTokenKindV0::FunctionName, name);
            profile
        }
        VdsExpression::Sequence(expressions)
        | VdsExpression::AllInAnyOrder(expressions)
        | VdsExpression::OneOrMoreInAnyOrder(expressions)
        | VdsExpression::Choice(expressions) => {
            let mut profile = ClosedWorldTokenProfileV0::default();
            for expression in expressions {
                profile.merge(closed_world_token_profile(
                    expression, registry, visiting, memo,
                ));
            }
            profile
        }
        VdsExpression::Repeat { expression, .. } | VdsExpression::Required(expression) => {
            closed_world_token_profile(expression, registry, visiting, memo)
        }
    }
}

fn closed_world_reference_profile(
    reference: &VdsReference,
    registry: &SpecGrammarRegistryV0,
    visiting: &mut HashSet<(ReferenceCategory, String)>,
    memo: &mut HashMap<(ReferenceCategory, String), ClosedWorldTokenProfileV0>,
) -> ClosedWorldTokenProfileV0 {
    if reference.category == ReferenceCategory::Function {
        let mut profile = ClosedWorldTokenProfileV0::default();
        profile.allow(
            ClosedWorldTokenKindV0::FunctionName,
            reference.name.trim_end_matches("()"),
        );
        return profile;
    }
    if reference.category == ReferenceCategory::Type
        && let Some(profile) = closed_world_builtin_profile(reference.name.as_str())
    {
        return profile;
    }

    let key = (reference.category, reference.name.clone());
    if let Some(profile) = memo.get(&key) {
        return profile.clone();
    }
    if !visiting.insert(key.clone()) {
        let mut profile = ClosedWorldTokenProfileV0::default();
        profile.mark_all_open();
        return profile;
    }
    let category = match reference.category {
        ReferenceCategory::Type => "types",
        ReferenceCategory::Property => "properties",
        ReferenceCategory::Function => unreachable!("function references return above"),
    };
    let mut profile = registry
        .entry(category, reference.name.as_str())
        .and_then(|entry| entry.syntax.as_deref().map(|source| (entry, source)))
        .and_then(|(entry, source)| {
            cached_pinned_vds_expression(source)
                .ok()
                .map(|expression| (entry, expression))
        })
        .map(|(entry, expression)| {
            let mut profile =
                closed_world_token_profile(expression.as_ref(), registry, visiting, memo);
            if entry.override_provenance.is_some() {
                profile.mark_all_open();
            }
            profile
        })
        .unwrap_or_else(|| {
            let mut unknown = ClosedWorldTokenProfileV0::default();
            unknown.mark_all_open();
            unknown
        });
    visiting.remove(&key);
    if reference.category == ReferenceCategory::Type
        && matches!(
            reference.name.as_str(),
            "number" | "integer" | "length" | "percentage" | "length-percentage" | "angle" | "time"
        )
    {
        profile.mark_open(ClosedWorldTokenKindV0::FunctionName);
    }
    memo.insert(key, profile.clone());
    profile
}

fn closed_world_builtin_profile(name: &str) -> Option<ClosedWorldTokenProfileV0> {
    let known_builtin = is_builtin_reference_name(name)
        || matches!(name, "declaration-value" | "any-value" | "whole-value");
    if !known_builtin {
        return None;
    }

    let Some(manifest) = closed_world_builtin_token_profiles() else {
        return Some(all_open_closed_world_token_profile());
    };
    let Some(witnessed) = manifest
        .profiles
        .iter()
        .find(|profile| profile.name == name)
    else {
        return Some(all_open_closed_world_token_profile());
    };
    if witnessed.authority == "registryDerived" {
        return None;
    }
    if !matches!(
        witnessed.authority.as_str(),
        "cssTreeWitness" | "defaultOpen"
    ) {
        return Some(all_open_closed_world_token_profile());
    }

    let mut profile = ClosedWorldTokenProfileV0::default();
    for kind in &witnessed.open_token_kinds {
        let Some(kind) = closed_world_token_kind_from_data_name(kind) else {
            return Some(all_open_closed_world_token_profile());
        };
        profile.mark_open(kind);
    }
    for (kind, values) in &witnessed.allowed_values {
        let Some(kind) = closed_world_token_kind_from_data_name(kind) else {
            return Some(all_open_closed_world_token_profile());
        };
        for value in values {
            profile.allow(kind, value);
        }
    }
    if witnessed.authority == "defaultOpen"
        && CLOSED_WORLD_TOKEN_KINDS
            .iter()
            .any(|kind| !profile.domain(*kind).open)
    {
        return Some(all_open_closed_world_token_profile());
    }
    Some(profile)
}

fn closed_world_builtin_token_profiles() -> Option<&'static ClosedWorldBuiltinTokenProfilesV0> {
    static PROFILES: OnceLock<Option<ClosedWorldBuiltinTokenProfilesV0>> = OnceLock::new();
    PROFILES
        .get_or_init(|| {
            let profiles = serde_json::from_str::<ClosedWorldBuiltinTokenProfilesV0>(
                CLOSED_WORLD_BUILTIN_TOKEN_PROFILES_SOURCE,
            )
            .ok()?;
            (profiles.schema_version == "0"
                && profiles.product == "omena-abstract-value.closed-world-builtin-token-profiles"
                && profiles.oracle.name == ClosedWorldKeywordClosureOracleNameV0::CssTree
                && profiles.oracle.version == "3.2.1"
                && profiles.profile_count == profiles.profiles.len())
            .then_some(profiles)
        })
        .as_ref()
}

fn closed_world_token_kind_from_data_name(name: &str) -> Option<ClosedWorldTokenKindV0> {
    match name {
        "ident" => Some(ClosedWorldTokenKindV0::Ident),
        "hash" => Some(ClosedWorldTokenKindV0::Hash),
        "dimension" => Some(ClosedWorldTokenKindV0::Dimension),
        "number" => Some(ClosedWorldTokenKindV0::Number),
        "percentage" => Some(ClosedWorldTokenKindV0::Percentage),
        "functionName" => Some(ClosedWorldTokenKindV0::FunctionName),
        "string" => Some(ClosedWorldTokenKindV0::String),
        "url" => Some(ClosedWorldTokenKindV0::Url),
        _ => None,
    }
}

fn all_open_closed_world_token_profile() -> ClosedWorldTokenProfileV0 {
    let mut profile = ClosedWorldTokenProfileV0::default();
    profile.mark_all_open();
    profile
}

fn closed_world_literal_profile(literal: &str) -> ClosedWorldTokenProfileV0 {
    let mut profile = ClosedWorldTokenProfileV0::default();
    let Ok(components) = css_value_component_stream(literal, 0) else {
        return profile;
    };
    if let [component] = components.as_slice()
        && let Some((kind, value)) = closed_world_component_identity(component)
    {
        profile.allow(kind, value.as_str());
    }
    profile
}

fn closed_world_component_identity(
    component: &CssValueComponentV0,
) -> Option<(ClosedWorldTokenKindV0, String)> {
    let kind = match &component.kind {
        CssValueComponentKindV0::Ident => ClosedWorldTokenKindV0::Ident,
        CssValueComponentKindV0::Hash => ClosedWorldTokenKindV0::Hash,
        CssValueComponentKindV0::Dimension => ClosedWorldTokenKindV0::Dimension,
        CssValueComponentKindV0::Number => ClosedWorldTokenKindV0::Number,
        CssValueComponentKindV0::Percentage => ClosedWorldTokenKindV0::Percentage,
        CssValueComponentKindV0::Function { name, .. } => {
            return Some((
                ClosedWorldTokenKindV0::FunctionName,
                name.to_ascii_lowercase(),
            ));
        }
        CssValueComponentKindV0::String => ClosedWorldTokenKindV0::String,
        CssValueComponentKindV0::Url => ClosedWorldTokenKindV0::Url,
        CssValueComponentKindV0::Parenthesized { .. }
        | CssValueComponentKindV0::Bracketed { .. }
        | CssValueComponentKindV0::Braced { .. }
        | CssValueComponentKindV0::Comma
        | CssValueComponentKindV0::Slash
        | CssValueComponentKindV0::Delimiter => return None,
    };
    Some((kind, component.text.to_ascii_lowercase()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VdsReference {
    category: ReferenceCategory,
    name: String,
    range: Option<NumericRange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ReferenceCategory {
    Type,
    Property,
    Function,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NumericRange {
    min: Option<String>,
    max: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VdsParseError {
    offset: usize,
    code: &'static str,
    detail: String,
}

type CachedVdsExpression = Result<Arc<VdsExpression>, VdsParseError>;

fn cached_pinned_vds_expression(source: &str) -> CachedVdsExpression {
    static CACHE: OnceLock<RwLock<HashMap<String, CachedVdsExpression>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| RwLock::new(HashMap::new()));
    if let Some(parsed) = cache
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(source)
    {
        return parsed.clone();
    }

    let parsed = VdsParser::new(source).parse().map(Arc::new);
    cache
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .entry(source.to_string())
        .or_insert(parsed)
        .clone()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VdsToken {
    kind: VdsTokenKind,
    offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum VdsTokenKind {
    Word(String),
    Reference(String),
    Literal(String),
    OpenBracket,
    CloseBracket,
    OpenParen,
    CloseParen,
    Or,
    OrOr,
    AndAnd,
    Question,
    Star,
    Plus,
    Hash,
    Range(usize, Option<usize>),
    Bang,
    End,
}

struct VdsParser<'a> {
    source: &'a str,
    tokens: Vec<VdsToken>,
    cursor: usize,
}

impl<'a> VdsParser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            cursor: 0,
        }
    }

    fn parse(mut self) -> Result<VdsExpression, VdsParseError> {
        self.tokens = lex_vds(self.source)?;
        let expression = self.parse_choice()?;
        if !matches!(self.peek(), VdsTokenKind::End) {
            return Err(self.error(
                "unexpectedGrammarToken",
                "unexpected trailing grammar token",
            ));
        }
        Ok(expression)
    }

    fn parse_choice(&mut self) -> Result<VdsExpression, VdsParseError> {
        let mut values = vec![self.parse_one_or_more_in_any_order()?];
        while matches!(self.peek(), VdsTokenKind::Or) {
            self.cursor += 1;
            values.push(self.parse_one_or_more_in_any_order()?);
        }
        Ok(flatten_expression(values, VdsExpression::Choice))
    }

    fn parse_one_or_more_in_any_order(&mut self) -> Result<VdsExpression, VdsParseError> {
        let mut values = vec![self.parse_all_in_any_order()?];
        while matches!(self.peek(), VdsTokenKind::OrOr) {
            self.cursor += 1;
            values.push(self.parse_all_in_any_order()?);
        }
        Ok(flatten_expression(
            values,
            VdsExpression::OneOrMoreInAnyOrder,
        ))
    }

    fn parse_all_in_any_order(&mut self) -> Result<VdsExpression, VdsParseError> {
        let mut values = vec![self.parse_sequence()?];
        while matches!(self.peek(), VdsTokenKind::AndAnd) {
            self.cursor += 1;
            values.push(self.parse_sequence()?);
        }
        Ok(flatten_expression(values, VdsExpression::AllInAnyOrder))
    }

    fn parse_sequence(&mut self) -> Result<VdsExpression, VdsParseError> {
        let mut values = Vec::new();
        while self.starts_primary() {
            values.push(self.parse_postfix()?);
        }
        if values.is_empty() {
            return Err(self.error("missingGrammarTerm", "expected a grammar term"));
        }
        Ok(flatten_expression(values, VdsExpression::Sequence))
    }

    fn parse_postfix(&mut self) -> Result<VdsExpression, VdsParseError> {
        let mut expression = self.parse_primary()?;
        loop {
            expression = match self.peek() {
                VdsTokenKind::Question => {
                    self.cursor += 1;
                    repeat(expression, 0, Some(1), false)
                }
                VdsTokenKind::Star => {
                    self.cursor += 1;
                    repeat(expression, 0, None, false)
                }
                VdsTokenKind::Plus => {
                    self.cursor += 1;
                    repeat(expression, 1, None, false)
                }
                VdsTokenKind::Hash => {
                    self.cursor += 1;
                    let (min, max) = match self.peek().clone() {
                        VdsTokenKind::Range(min, max) => {
                            self.cursor += 1;
                            (min, max)
                        }
                        _ => (1, None),
                    };
                    repeat(expression, min, max, true)
                }
                VdsTokenKind::Range(min, max) => {
                    let min = *min;
                    let max = *max;
                    self.cursor += 1;
                    repeat(expression, min, max, false)
                }
                VdsTokenKind::Bang => {
                    self.cursor += 1;
                    VdsExpression::Required(Box::new(expression))
                }
                _ => break,
            };
        }
        Ok(expression)
    }

    fn parse_primary(&mut self) -> Result<VdsExpression, VdsParseError> {
        let token = self.tokens[self.cursor].clone();
        self.cursor += 1;
        match token.kind {
            VdsTokenKind::Reference(source) => Ok(VdsExpression::Reference(parse_reference(
                source.as_str(),
                token.offset,
            )?)),
            VdsTokenKind::Word(word) => {
                if matches!(self.peek(), VdsTokenKind::OpenParen) {
                    self.cursor += 1;
                    if matches!(self.peek(), VdsTokenKind::CloseParen) {
                        self.cursor += 1;
                        return Ok(VdsExpression::Function {
                            name: word,
                            arguments: Box::new(VdsExpression::Sequence(Vec::new())),
                        });
                    }
                    let arguments = self.parse_choice()?;
                    self.expect_close_paren()?;
                    Ok(VdsExpression::Function {
                        name: word,
                        arguments: Box::new(arguments),
                    })
                } else {
                    Ok(VdsExpression::Literal(word))
                }
            }
            VdsTokenKind::Literal(literal) => Ok(VdsExpression::Literal(literal)),
            VdsTokenKind::OpenBracket => {
                let expression = self.parse_choice()?;
                if !matches!(self.peek(), VdsTokenKind::CloseBracket) {
                    return Err(self.error("unclosedGrammarGroup", "missing closing ]"));
                }
                self.cursor += 1;
                Ok(expression)
            }
            VdsTokenKind::OpenParen => {
                let expression = self.parse_choice()?;
                self.expect_close_paren()?;
                Ok(expression)
            }
            _ => Err(VdsParseError {
                offset: token.offset,
                code: "unexpectedGrammarPrimary",
                detail: "expected a literal, reference, function, or group".to_string(),
            }),
        }
    }

    fn expect_close_paren(&mut self) -> Result<(), VdsParseError> {
        if !matches!(self.peek(), VdsTokenKind::CloseParen) {
            return Err(self.error("unclosedGrammarFunction", "missing closing )"));
        }
        self.cursor += 1;
        Ok(())
    }

    fn starts_primary(&self) -> bool {
        matches!(
            self.peek(),
            VdsTokenKind::Reference(_)
                | VdsTokenKind::Word(_)
                | VdsTokenKind::Literal(_)
                | VdsTokenKind::OpenBracket
                | VdsTokenKind::OpenParen
        )
    }

    fn peek(&self) -> &VdsTokenKind {
        &self.tokens[self.cursor].kind
    }

    fn error(&self, code: &'static str, detail: &str) -> VdsParseError {
        VdsParseError {
            offset: self.tokens[self.cursor].offset,
            code,
            detail: detail.to_string(),
        }
    }
}

fn flatten_expression(
    mut values: Vec<VdsExpression>,
    wrap: impl FnOnce(Vec<VdsExpression>) -> VdsExpression,
) -> VdsExpression {
    if values.len() == 1 {
        values.pop().unwrap_or(VdsExpression::Sequence(Vec::new()))
    } else {
        wrap(values)
    }
}

fn repeat(
    expression: VdsExpression,
    min: usize,
    max: Option<usize>,
    comma_separated: bool,
) -> VdsExpression {
    VdsExpression::Repeat {
        expression: Box::new(expression),
        min,
        max,
        comma_separated,
    }
}

fn lex_vds(source: &str) -> Result<Vec<VdsToken>, VdsParseError> {
    let mut tokens = Vec::new();
    let mut cursor = 0usize;
    while cursor < source.len() {
        let Some(character) = source[cursor..].chars().next() else {
            break;
        };
        if character.is_whitespace() {
            cursor += character.len_utf8();
            continue;
        }
        let offset = cursor;
        let rest = &source[cursor..];
        if rest.starts_with("||") {
            tokens.push(token(VdsTokenKind::OrOr, offset));
            cursor += 2;
            continue;
        }
        if rest.starts_with("&&") {
            tokens.push(token(VdsTokenKind::AndAnd, offset));
            cursor += 2;
            continue;
        }
        if character == '<' {
            let Some(relative_end) = rest.find('>') else {
                return Err(VdsParseError {
                    offset,
                    code: "unclosedGrammarReference",
                    detail: "missing closing >".to_string(),
                });
            };
            let end = cursor + relative_end;
            tokens.push(token(
                VdsTokenKind::Reference(source[cursor + 1..end].trim().to_string()),
                offset,
            ));
            cursor = end + 1;
            continue;
        }
        if character == '{' {
            let Some(relative_end) =
                rest.char_indices()
                    .find_map(|(relative_offset, candidate)| {
                        (candidate == '}').then_some(relative_offset)
                    })
            else {
                return Err(VdsParseError {
                    offset,
                    code: "unclosedGrammarRange",
                    detail: "missing closing }".to_string(),
                });
            };
            let end = cursor + relative_end;
            let range = parse_repeat_range(&source[cursor + 1..end], offset)?;
            tokens.push(token(VdsTokenKind::Range(range.0, range.1), offset));
            cursor = end + 1;
            continue;
        }
        let simple = match character {
            '[' => Some(VdsTokenKind::OpenBracket),
            ']' => Some(VdsTokenKind::CloseBracket),
            '(' => Some(VdsTokenKind::OpenParen),
            ')' => Some(VdsTokenKind::CloseParen),
            '|' => Some(VdsTokenKind::Or),
            '?' => Some(VdsTokenKind::Question),
            '*' => Some(VdsTokenKind::Star),
            '+' => Some(VdsTokenKind::Plus),
            '#' => Some(VdsTokenKind::Hash),
            '!' => Some(VdsTokenKind::Bang),
            ',' | '/' | ':' | ';' | '=' | '@' | '~' | '^' | '$' | '&' => {
                Some(VdsTokenKind::Literal(character.to_string()))
            }
            _ => None,
        };
        if let Some(kind) = simple {
            tokens.push(token(kind, offset));
            cursor += character.len_utf8();
            continue;
        }
        if character == '\'' || character == '"' {
            let quote = character;
            cursor += character.len_utf8();
            let content_start = cursor;
            let mut escaped = false;
            let mut found_end = None;
            while cursor < source.len() {
                let Some(current) = source[cursor..].chars().next() else {
                    break;
                };
                if escaped {
                    escaped = false;
                } else if current == '\\' {
                    escaped = true;
                } else if current == quote {
                    found_end = Some(cursor);
                    break;
                }
                cursor += current.len_utf8();
            }
            let Some(end) = found_end else {
                return Err(VdsParseError {
                    offset,
                    code: "unclosedGrammarString",
                    detail: "missing closing quote".to_string(),
                });
            };
            tokens.push(token(
                VdsTokenKind::Literal(source[content_start..end].to_string()),
                offset,
            ));
            cursor = end + quote.len_utf8();
            continue;
        }
        let start = cursor;
        while cursor < source.len() {
            let Some(current) = source[cursor..].chars().next() else {
                break;
            };
            if current.is_whitespace()
                || matches!(
                    current,
                    '<' | '>'
                        | '['
                        | ']'
                        | '('
                        | ')'
                        | '{'
                        | '}'
                        | '|'
                        | '&'
                        | '?'
                        | '*'
                        | '+'
                        | '#'
                        | '!'
                        | ','
                        | '/'
                        | ':'
                        | ';'
                        | '='
                        | '@'
                        | '~'
                        | '^'
                        | '$'
                        | '\''
                        | '"'
                )
            {
                break;
            }
            cursor += current.len_utf8();
        }
        if start == cursor {
            return Err(VdsParseError {
                offset,
                code: "unsupportedGrammarCharacter",
                detail: format!("unsupported grammar character {character:?}"),
            });
        }
        tokens.push(token(
            VdsTokenKind::Word(source[start..cursor].to_string()),
            offset,
        ));
    }
    tokens.push(token(VdsTokenKind::End, source.len()));
    Ok(tokens)
}

fn token(kind: VdsTokenKind, offset: usize) -> VdsToken {
    VdsToken { kind, offset }
}

fn parse_repeat_range(
    source: &str,
    offset: usize,
) -> Result<(usize, Option<usize>), VdsParseError> {
    let mut parts = source.split(',').map(str::trim);
    let first = parts.next().unwrap_or_default();
    let second = parts.next();
    if parts.next().is_some() || first.is_empty() {
        return Err(VdsParseError {
            offset,
            code: "invalidGrammarRange",
            detail: format!("invalid repeat range {{{source}}}"),
        });
    }
    let min = first.parse::<usize>().map_err(|_| VdsParseError {
        offset,
        code: "invalidGrammarRange",
        detail: format!("invalid repeat range minimum {first:?}"),
    })?;
    let max = match second {
        None => Some(min),
        Some("") => None,
        Some(value) => Some(value.parse::<usize>().map_err(|_| VdsParseError {
            offset,
            code: "invalidGrammarRange",
            detail: format!("invalid repeat range maximum {value:?}"),
        })?),
    };
    if max.is_some_and(|max| max < min) {
        return Err(VdsParseError {
            offset,
            code: "invalidGrammarRange",
            detail: format!("repeat range maximum precedes minimum in {{{source}}}"),
        });
    }
    Ok((min, max))
}

fn parse_reference(source: &str, offset: usize) -> Result<VdsReference, VdsParseError> {
    let source = source.trim();
    if source.is_empty() {
        return Err(VdsParseError {
            offset,
            code: "emptyGrammarReference",
            detail: "empty grammar reference".to_string(),
        });
    }
    if let Some(property) = source
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
    {
        return Ok(VdsReference {
            category: ReferenceCategory::Property,
            name: PropertyNameV0::from_authored(property)
                .canonical_name()
                .to_string(),
            range: None,
        });
    }
    let (name, range) = split_reference_range(source, offset)?;
    let category = if name.ends_with("()") {
        ReferenceCategory::Function
    } else {
        ReferenceCategory::Type
    };
    Ok(VdsReference {
        category,
        name: name.to_ascii_lowercase(),
        range,
    })
}

fn split_reference_range(
    source: &str,
    offset: usize,
) -> Result<(&str, Option<NumericRange>), VdsParseError> {
    let Some(open) = source.find('[') else {
        return Ok((source.trim(), None));
    };
    let Some(close) = source.rfind(']') else {
        return Err(VdsParseError {
            offset,
            code: "unclosedReferenceRange",
            detail: format!("missing ] in reference <{source}>"),
        });
    };
    if close + 1 != source.len() {
        return Err(VdsParseError {
            offset,
            code: "trailingReferenceRangeContent",
            detail: format!("unexpected content after range in <{source}>"),
        });
    }
    let name = source[..open].trim();
    let mut bounds = source[open + 1..close].split(',').map(str::trim);
    let min = bounds.next().unwrap_or_default();
    let max = bounds.next();
    if name.is_empty() || min.is_empty() || max.is_none() || bounds.next().is_some() {
        return Err(VdsParseError {
            offset,
            code: "invalidReferenceRange",
            detail: format!("invalid numeric range in <{source}>"),
        });
    }
    let max = max.unwrap_or_default();
    Ok((
        name,
        Some(NumericRange {
            min: finite_range_bound(min),
            max: finite_range_bound(max),
        }),
    ))
}

fn finite_range_bound(source: &str) -> Option<String> {
    (!matches!(source, "∞" | "+∞" | "-∞")).then(|| source.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MatchStop {
    Budget {
        kind: CssValueGrammarBudgetKindV0,
        limit: usize,
        reference: Option<String>,
    },
    GrammarDefect {
        offset: usize,
        code: &'static str,
        detail: String,
    },
}

struct MatchContext<'a> {
    registry: &'a SpecGrammarRegistryV0,
    budget: CssValueGrammarBudgetV0,
    match_steps: usize,
    first_stop: Option<MatchStop>,
    grammar_cache: HashMap<(ReferenceCategory, String), CachedVdsExpression>,
    cache_registered_grammars: bool,
    allow_unvalidated_standard_function_references: bool,
}

#[derive(Debug, Clone, Copy)]
struct RepeatMatchPlan<'a> {
    expression: &'a VdsExpression,
    min: usize,
    max: Option<usize>,
    comma_separated: bool,
}

impl MatchContext<'_> {
    fn match_expression(
        &mut self,
        expression: &VdsExpression,
        components: &[CssValueComponentV0],
        position: usize,
        reference_depth: usize,
    ) -> BTreeSet<usize> {
        if !self.consume_step(None) {
            return BTreeSet::new();
        }
        let positions = match expression {
            VdsExpression::Literal(literal) => match_literal(literal, components, position),
            VdsExpression::Reference(reference) => {
                self.match_reference(reference, components, position, reference_depth)
            }
            VdsExpression::Function { name, arguments } => {
                self.match_function(name, arguments, components, position, reference_depth)
            }
            VdsExpression::Sequence(expressions) => {
                self.match_sequence(expressions, components, position, reference_depth)
            }
            VdsExpression::AllInAnyOrder(expressions) => {
                self.match_any_order(expressions, components, position, reference_depth, true)
            }
            VdsExpression::OneOrMoreInAnyOrder(expressions) => {
                self.match_any_order(expressions, components, position, reference_depth, false)
            }
            VdsExpression::Choice(expressions) => expressions
                .iter()
                .flat_map(|expression| {
                    self.match_expression(expression, components, position, reference_depth)
                })
                .collect(),
            VdsExpression::Repeat {
                expression,
                min,
                max,
                comma_separated,
            } => self.match_repeat(
                RepeatMatchPlan {
                    expression,
                    min: *min,
                    max: *max,
                    comma_separated: *comma_separated,
                },
                components,
                position,
                reference_depth,
            ),
            VdsExpression::Required(expression) => self
                .match_expression(expression, components, position, reference_depth)
                .into_iter()
                .filter(|end| *end > position)
                .collect(),
        };
        self.cap_states(positions, None)
    }

    fn match_sequence(
        &mut self,
        expressions: &[VdsExpression],
        components: &[CssValueComponentV0],
        position: usize,
        reference_depth: usize,
    ) -> BTreeSet<usize> {
        let mut states = BTreeSet::from([(position, false)]);
        for expression in expressions {
            let mut next = BTreeSet::new();
            for (position, previous_was_omitted) in states {
                if previous_was_omitted && is_comma_literal(expression) {
                    next.insert((position, false));
                    continue;
                }
                for end in self.match_expression(expression, components, position, reference_depth)
                {
                    next.insert((end, end == position));
                }
            }
            if next.len() > self.budget.max_states {
                self.record_stop(MatchStop::Budget {
                    kind: CssValueGrammarBudgetKindV0::CandidateStates,
                    limit: self.budget.max_states,
                    reference: None,
                });
                next.clear();
            }
            states = next;
            if states.is_empty() {
                break;
            }
        }
        states.into_iter().map(|(position, _)| position).collect()
    }

    fn match_repeat(
        &mut self,
        plan: RepeatMatchPlan<'_>,
        components: &[CssValueComponentV0],
        position: usize,
        reference_depth: usize,
    ) -> BTreeSet<usize> {
        let effective_max = plan
            .max
            .unwrap_or_else(|| components.len().saturating_add(1));
        let mut accepted = BTreeSet::new();
        let mut frontier = BTreeSet::from([position]);
        if plan.min == 0 {
            accepted.insert(position);
        }
        for count in 1..=effective_max {
            let mut next = BTreeSet::new();
            for current in &frontier {
                let item_start = if plan.comma_separated && count > 1 {
                    if components.get(*current).is_some_and(|component| {
                        matches!(component.kind, CssValueComponentKindV0::Comma)
                    }) {
                        *current + 1
                    } else {
                        continue;
                    }
                } else {
                    *current
                };
                for end in
                    self.match_expression(plan.expression, components, item_start, reference_depth)
                {
                    if end > item_start {
                        next.insert(end);
                    }
                }
            }
            frontier = self.cap_states(next, None);
            if frontier.is_empty() {
                break;
            }
            if count >= plan.min {
                accepted.extend(frontier.iter().copied());
            }
        }
        accepted
    }

    fn match_any_order(
        &mut self,
        expressions: &[VdsExpression],
        components: &[CssValueComponentV0],
        position: usize,
        reference_depth: usize,
        require_all: bool,
    ) -> BTreeSet<usize> {
        if expressions.len() > 63 {
            self.record_stop(MatchStop::Budget {
                kind: CssValueGrammarBudgetKindV0::CandidateStates,
                limit: self.budget.max_states,
                reference: None,
            });
            return BTreeSet::new();
        }
        let required_mask = (1u64 << expressions.len()) - 1;
        let mut accepted = BTreeSet::new();
        let mut stack = vec![(position, 0u64)];
        let mut visited = BTreeSet::new();
        while let Some((current, mask)) = stack.pop() {
            if !visited.insert((current, mask)) {
                continue;
            }
            if visited.len() > self.budget.max_states {
                self.record_stop(MatchStop::Budget {
                    kind: CssValueGrammarBudgetKindV0::CandidateStates,
                    limit: self.budget.max_states,
                    reference: None,
                });
                break;
            }
            if (require_all && mask == required_mask) || (!require_all && mask != 0) {
                accepted.insert(current);
            }
            for (index, expression) in expressions.iter().enumerate() {
                let bit = 1u64 << index;
                if mask & bit != 0 {
                    continue;
                }
                for end in self.match_expression(expression, components, current, reference_depth) {
                    if end > current || (require_all && end == current) {
                        stack.push((end, mask | bit));
                    }
                }
            }
        }
        accepted
    }

    fn match_function(
        &mut self,
        name: &str,
        arguments: &VdsExpression,
        components: &[CssValueComponentV0],
        position: usize,
        reference_depth: usize,
    ) -> BTreeSet<usize> {
        let Some(component) = components.get(position) else {
            return BTreeSet::new();
        };
        let CssValueComponentKindV0::Function {
            name: actual,
            arguments: actual_arguments,
        } = &component.kind
        else {
            return BTreeSet::new();
        };
        if !actual.eq_ignore_ascii_case(name) {
            return BTreeSet::new();
        }
        self.match_expression(arguments, actual_arguments, 0, reference_depth)
            .contains(&actual_arguments.len())
            .then_some(position + 1)
            .into_iter()
            .collect()
    }

    fn match_reference(
        &mut self,
        reference: &VdsReference,
        components: &[CssValueComponentV0],
        position: usize,
        reference_depth: usize,
    ) -> BTreeSet<usize> {
        if self.allow_unvalidated_standard_function_references
            && reference.category != ReferenceCategory::Function
            && components
                .get(position)
                .is_some_and(component_is_recognized_standard_function)
        {
            return BTreeSet::from([position + 1]);
        }
        if let Some(positions) =
            match_builtin_reference(reference, components, position, self.registry)
        {
            return positions;
        }
        if reference_depth >= self.budget.max_reference_depth {
            self.record_stop(MatchStop::Budget {
                kind: CssValueGrammarBudgetKindV0::ReferenceDepth,
                limit: self.budget.max_reference_depth,
                reference: Some(reference.name.clone()),
            });
            return BTreeSet::new();
        }
        let category = match reference.category {
            ReferenceCategory::Type => "types",
            ReferenceCategory::Property => "properties",
            ReferenceCategory::Function => "functions",
        };
        let Some(entry) = self.registry.entry(category, reference.name.as_str()) else {
            self.record_stop(MatchStop::GrammarDefect {
                offset: 0,
                code: "unknownGrammarReference",
                detail: format!("unknown {category} reference <{}>", reference.name),
            });
            return BTreeSet::new();
        };
        let Some(source) = entry.syntax.as_deref() else {
            self.record_stop(MatchStop::GrammarDefect {
                offset: 0,
                code: "missingReferencedGrammar",
                detail: format!("{category} reference <{}> has no syntax", reference.name),
            });
            return BTreeSet::new();
        };
        let key = (reference.category, reference.name.clone());
        let expression = match self
            .grammar_cache
            .entry(key)
            .or_insert_with(|| {
                if self.cache_registered_grammars {
                    cached_pinned_vds_expression(source)
                } else {
                    VdsParser::new(source).parse().map(Arc::new)
                }
            })
            .clone()
        {
            Ok(expression) => expression,
            Err(error) => {
                self.record_stop(MatchStop::GrammarDefect {
                    offset: error.offset,
                    code: error.code,
                    detail: format!("referenced grammar <{}>: {}", reference.name, error.detail),
                });
                return BTreeSet::new();
            }
        };
        if reference.category == ReferenceCategory::Function {
            return self.match_function_reference(
                reference,
                expression.as_ref(),
                components,
                position,
                reference_depth + 1,
            );
        }
        self.match_expression(
            expression.as_ref(),
            components,
            position,
            reference_depth + 1,
        )
    }

    fn match_function_reference(
        &mut self,
        reference: &VdsReference,
        expression: &VdsExpression,
        components: &[CssValueComponentV0],
        position: usize,
        reference_depth: usize,
    ) -> BTreeSet<usize> {
        let name = reference.name.trim_end_matches("()");
        let whole_component =
            self.match_expression(expression, components, position, reference_depth);
        if !whole_component.is_empty() {
            return whole_component;
        }
        self.match_function(name, expression, components, position, reference_depth)
    }

    fn consume_step(&mut self, reference: Option<String>) -> bool {
        self.match_steps += 1;
        if self.match_steps <= self.budget.max_match_steps {
            return true;
        }
        self.record_stop(MatchStop::Budget {
            kind: CssValueGrammarBudgetKindV0::MatchSteps,
            limit: self.budget.max_match_steps,
            reference,
        });
        false
    }

    fn cap_states(
        &mut self,
        mut states: BTreeSet<usize>,
        reference: Option<String>,
    ) -> BTreeSet<usize> {
        if states.len() <= self.budget.max_states {
            return states;
        }
        self.record_stop(MatchStop::Budget {
            kind: CssValueGrammarBudgetKindV0::CandidateStates,
            limit: self.budget.max_states,
            reference,
        });
        states.clear();
        states
    }

    fn record_stop(&mut self, stop: MatchStop) {
        if self.first_stop.is_none() {
            self.first_stop = Some(stop);
        }
    }
}

fn match_literal(
    literal: &str,
    components: &[CssValueComponentV0],
    position: usize,
) -> BTreeSet<usize> {
    components
        .get(position)
        .filter(|component| component.text.eq_ignore_ascii_case(literal))
        .map(|_| BTreeSet::from([position + 1]))
        .unwrap_or_default()
}

fn is_comma_literal(expression: &VdsExpression) -> bool {
    matches!(expression, VdsExpression::Literal(literal) if literal == ",")
}

fn is_unitless_zero(component: &CssValueComponentV0) -> bool {
    matches!(component.kind, CssValueComponentKindV0::Number)
        && parse_numeric_value_with_unit(component.text.as_str())
            .is_some_and(|numeric| numeric.value == 0.0 && numeric.unit.is_empty())
}

fn match_builtin_reference(
    reference: &VdsReference,
    components: &[CssValueComponentV0],
    position: usize,
    registry: &SpecGrammarRegistryV0,
) -> Option<BTreeSet<usize>> {
    if reference.category != ReferenceCategory::Type {
        return None;
    }
    if matches!(
        reference.name.as_str(),
        "declaration-value" | "any-value" | "whole-value"
    ) {
        return Some(((position + 1)..=components.len()).collect());
    }
    if !is_builtin_reference_name(reference.name.as_str()) {
        return None;
    }
    let Some(component) = components.get(position) else {
        return Some(BTreeSet::new());
    };
    if math_function_matches_reference(reference, component, registry) {
        return Some(BTreeSet::from([position + 1]));
    }
    let kind = classify_registered_property_declared_value_v0(component.text.as_str());
    let accepted = match reference.name.as_str() {
        "number" | "number-token" => {
            matches!(
                kind,
                DeclaredValueKindV0::Number | DeclaredValueKindV0::Integer
            )
        }
        "integer" => matches!(kind, DeclaredValueKindV0::Integer),
        "length" => {
            matches!(
                kind,
                DeclaredValueKindV0::Dimension(DeclaredNumericTypeV0::Length)
            ) || is_unitless_zero(component)
        }
        "percentage" | "percentage-token" => matches!(
            kind,
            DeclaredValueKindV0::Dimension(DeclaredNumericTypeV0::Percentage)
        ),
        "length-percentage" => {
            matches!(
                kind,
                DeclaredValueKindV0::Dimension(
                    DeclaredNumericTypeV0::Length | DeclaredNumericTypeV0::Percentage
                )
            ) || is_unitless_zero(component)
        }
        "angle" => matches!(
            kind,
            DeclaredValueKindV0::Dimension(DeclaredNumericTypeV0::Angle)
        ),
        "time" => matches!(
            kind,
            DeclaredValueKindV0::Dimension(DeclaredNumericTypeV0::Time)
        ),
        "resolution" => matches!(
            kind,
            DeclaredValueKindV0::Dimension(DeclaredNumericTypeV0::Resolution)
        ),
        "flex" => parse_numeric_value_with_unit(component.text.as_str())
            .is_some_and(|numeric| numeric.unit.eq_ignore_ascii_case("fr")),
        "hex-color" => matches!(kind, DeclaredValueKindV0::HexColor),
        "named-color" => matches!(kind, DeclaredValueKindV0::ColorKeyword(_)),
        "custom-ident" => {
            matches!(component.kind, CssValueComponentKindV0::Ident)
                && !matches!(kind, DeclaredValueKindV0::CssWide)
        }
        "ident" | "ident-token" => matches!(component.kind, CssValueComponentKindV0::Ident),
        "dashed-ident" | "custom-property-name" => {
            matches!(component.kind, CssValueComponentKindV0::Ident)
                && is_custom_property_name(&component.text)
        }
        "string" | "string-token" => matches!(kind, DeclaredValueKindV0::QuotedString),
        "url" | "url-token" => matches!(kind, DeclaredValueKindV0::Url),
        "image" => matches!(
            kind,
            DeclaredValueKindV0::ImageFunction | DeclaredValueKindV0::Url
        ),
        "transform-function" => matches!(kind, DeclaredValueKindV0::TransformFunction),
        "alpha-value" => matches!(
            kind,
            DeclaredValueKindV0::Number
                | DeclaredValueKindV0::Integer
                | DeclaredValueKindV0::Dimension(DeclaredNumericTypeV0::Percentage)
        ),
        "zero" => parse_numeric_value_with_unit(component.text.as_str())
            .is_some_and(|numeric| numeric.value == 0.0),
        "dimension-token" => matches!(component.kind, CssValueComponentKindV0::Dimension),
        "hash-token" => matches!(component.kind, CssValueComponentKindV0::Hash),
        "function-token" => matches!(component.kind, CssValueComponentKindV0::Function { .. }),
        "comma-token" => matches!(component.kind, CssValueComponentKindV0::Comma),
        _ => false,
    };
    let accepted =
        accepted && numeric_range_accepts(reference.range.as_ref(), component.text.as_str());
    Some(accepted.then_some(position + 1).into_iter().collect())
}

fn is_builtin_reference_name(name: &str) -> bool {
    matches!(
        name,
        "number"
            | "number-token"
            | "integer"
            | "length"
            | "percentage"
            | "percentage-token"
            | "length-percentage"
            | "angle"
            | "time"
            | "resolution"
            | "flex"
            | "hex-color"
            | "named-color"
            | "custom-ident"
            | "ident"
            | "ident-token"
            | "dashed-ident"
            | "custom-property-name"
            | "string"
            | "string-token"
            | "url"
            | "url-token"
            | "image"
            | "transform-function"
            | "alpha-value"
            | "zero"
            | "dimension-token"
            | "hash-token"
            | "function-token"
            | "comma-token"
    )
}

fn math_function_matches_reference(
    reference: &VdsReference,
    component: &CssValueComponentV0,
    registry: &SpecGrammarRegistryV0,
) -> bool {
    if reference.category != ReferenceCategory::Type
        || !matches!(
            reference.name.as_str(),
            "number" | "length" | "percentage" | "length-percentage" | "time" | "angle"
        )
    {
        return false;
    }
    let CssValueComponentKindV0::Function { name, arguments } = &component.kind else {
        return false;
    };
    if !matches!(name.as_str(), "calc" | "min" | "max" | "clamp") {
        return false;
    }
    let registry_name = format!("{name}()");
    if !registry
        .entry("functions", registry_name.as_str())
        .is_some_and(|entry| {
            entry.boundary.classification == SpecGrammarBoundaryClassificationV0::InBoundary
                && entry.syntax.is_some()
        })
    {
        return false;
    }
    math_function_result_kind(name, arguments, registry)
        .is_some_and(|kind| math_kind_matches_reference(kind, reference.name.as_str()))
        && math_range_is_provably_accepted(reference.range.as_ref(), arguments)
}

fn split_math_argument_groups(arguments: &[CssValueComponentV0]) -> Vec<&[CssValueComponentV0]> {
    let mut groups = Vec::new();
    let mut start = 0;
    for (index, component) in arguments.iter().enumerate() {
        if matches!(component.kind, CssValueComponentKindV0::Comma) {
            groups.push(&arguments[start..index]);
            start = index + 1;
        }
    }
    groups.push(&arguments[start..]);
    groups
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MathValueKind {
    Number,
    Length,
    Percentage,
    LengthPercentage,
    Time,
    Angle,
}

fn math_kind_matches_reference(kind: MathValueKind, reference: &str) -> bool {
    matches!(
        (kind, reference),
        (MathValueKind::Number, "number")
            | (MathValueKind::Length, "length" | "length-percentage")
            | (
                MathValueKind::Percentage,
                "percentage" | "length-percentage"
            )
            | (MathValueKind::LengthPercentage, "length-percentage")
            | (MathValueKind::Time, "time")
            | (MathValueKind::Angle, "angle")
    )
}

fn math_function_result_kind(
    name: &str,
    arguments: &[CssValueComponentV0],
    registry: &SpecGrammarRegistryV0,
) -> Option<MathValueKind> {
    if !matches!(name, "calc" | "min" | "max" | "clamp") {
        return None;
    }
    let registry_name = format!("{name}()");
    if !registry
        .entry("functions", registry_name.as_str())
        .is_some_and(|entry| {
            entry.boundary.classification == SpecGrammarBoundaryClassificationV0::InBoundary
                && entry.syntax.is_some()
        })
    {
        return None;
    }
    let groups = split_math_argument_groups(arguments);
    let arity_matches = match name {
        "calc" => groups.len() == 1,
        "min" | "max" => !groups.is_empty(),
        "clamp" => groups.len() == 3,
        _ => false,
    };
    if !arity_matches {
        return None;
    }
    let mut kinds = groups
        .iter()
        .map(|group| MathExpressionParser::new(group, registry).parse())
        .collect::<Option<Vec<_>>>()?
        .into_iter();
    let first = kinds.next()?;
    kinds.try_fold(first, unify_additive_math_kinds)
}

fn unify_additive_math_kinds(left: MathValueKind, right: MathValueKind) -> Option<MathValueKind> {
    if left == right {
        return Some(left);
    }
    if matches!(
        (left, right),
        (MathValueKind::Length, MathValueKind::Percentage)
            | (MathValueKind::Percentage, MathValueKind::Length)
            | (MathValueKind::LengthPercentage, MathValueKind::Length)
            | (MathValueKind::Length, MathValueKind::LengthPercentage)
            | (MathValueKind::LengthPercentage, MathValueKind::Percentage)
            | (MathValueKind::Percentage, MathValueKind::LengthPercentage)
    ) {
        return Some(MathValueKind::LengthPercentage);
    }
    None
}

struct MathExpressionParser<'a> {
    components: &'a [CssValueComponentV0],
    cursor: usize,
    registry: &'a SpecGrammarRegistryV0,
}

impl<'a> MathExpressionParser<'a> {
    fn new(components: &'a [CssValueComponentV0], registry: &'a SpecGrammarRegistryV0) -> Self {
        Self {
            components,
            cursor: 0,
            registry,
        }
    }

    fn parse(mut self) -> Option<MathValueKind> {
        let kind = self.parse_sum()?;
        (self.cursor == self.components.len()).then_some(kind)
    }

    fn parse_sum(&mut self) -> Option<MathValueKind> {
        let mut left = self.parse_product()?;
        loop {
            if self.peek_binary_additive_operator().is_none() {
                break;
            }
            self.cursor += 1;
            let right = self.parse_product()?;
            left = unify_additive_math_kinds(left, right)?;
        }
        Some(left)
    }

    fn parse_product(&mut self) -> Option<MathValueKind> {
        let mut left = self.parse_unary()?;
        while let Some(operator) = self.peek_operator(&["*", "/"]).map(str::to_owned) {
            self.cursor += 1;
            let right = self.parse_unary()?;
            left = match operator.as_str() {
                "*" if left == MathValueKind::Number => right,
                "*" if right == MathValueKind::Number => left,
                "/" if right == MathValueKind::Number => left,
                _ => return None,
            };
        }
        Some(left)
    }

    fn parse_unary(&mut self) -> Option<MathValueKind> {
        if self.peek_operator(&["+", "-"]).is_some() {
            self.cursor += 1;
            return self.parse_unary();
        }
        self.parse_operand()
    }

    fn parse_operand(&mut self) -> Option<MathValueKind> {
        let component = self.components.get(self.cursor)?;
        self.cursor += 1;
        match &component.kind {
            CssValueComponentKindV0::Number => Some(MathValueKind::Number),
            CssValueComponentKindV0::Percentage => Some(MathValueKind::Percentage),
            CssValueComponentKindV0::Dimension => {
                match classify_registered_property_declared_value_v0(component.text.as_str()) {
                    DeclaredValueKindV0::Dimension(DeclaredNumericTypeV0::Length) => {
                        Some(MathValueKind::Length)
                    }
                    DeclaredValueKindV0::Dimension(DeclaredNumericTypeV0::Percentage) => {
                        Some(MathValueKind::Percentage)
                    }
                    DeclaredValueKindV0::Dimension(DeclaredNumericTypeV0::Time) => {
                        Some(MathValueKind::Time)
                    }
                    DeclaredValueKindV0::Dimension(DeclaredNumericTypeV0::Angle) => {
                        Some(MathValueKind::Angle)
                    }
                    _ => None,
                }
            }
            CssValueComponentKindV0::Function { name, arguments } => {
                math_function_result_kind(name, arguments, self.registry)
            }
            CssValueComponentKindV0::Parenthesized { values } => {
                MathExpressionParser::new(values, self.registry).parse()
            }
            CssValueComponentKindV0::Ident
            | CssValueComponentKindV0::Hash
            | CssValueComponentKindV0::String
            | CssValueComponentKindV0::Url
            | CssValueComponentKindV0::Bracketed { .. }
            | CssValueComponentKindV0::Braced { .. }
            | CssValueComponentKindV0::Comma
            | CssValueComponentKindV0::Slash
            | CssValueComponentKindV0::Delimiter => None,
        }
    }

    fn peek_operator(&self, expected: &[&str]) -> Option<&str> {
        let component = self.components.get(self.cursor)?;
        (matches!(
            component.kind,
            CssValueComponentKindV0::Delimiter | CssValueComponentKindV0::Slash
        ) || (matches!(component.kind, CssValueComponentKindV0::Ident)
            && matches!(component.text.as_str(), "+" | "-")))
        .then_some(component.text.as_str())
        .filter(|operator| expected.contains(operator))
    }

    fn peek_binary_additive_operator(&self) -> Option<&str> {
        let previous = self
            .cursor
            .checked_sub(1)
            .and_then(|index| self.components.get(index))?;
        let operator_component = self.components.get(self.cursor)?;
        let next = self.components.get(self.cursor + 1)?;
        self.peek_operator(&["+", "-"]).filter(|_| {
            previous.span.end < operator_component.span.start
                && operator_component.span.end < next.span.start
        })
    }
}

fn math_range_is_provably_accepted(
    range: Option<&NumericRange>,
    components: &[CssValueComponentV0],
) -> bool {
    let Some(range) = range else {
        return true;
    };
    if !range
        .min
        .as_deref()
        .and_then(parse_numeric_value_with_unit)
        .is_some_and(|numeric| numeric.value == 0.0)
        || range.max.is_some()
    {
        return false;
    }
    components.iter().all(|component| match &component.kind {
        CssValueComponentKindV0::Number
        | CssValueComponentKindV0::Percentage
        | CssValueComponentKindV0::Dimension => parse_numeric_value_with_unit(&component.text)
            .is_some_and(|numeric| numeric.value >= 0.0),
        CssValueComponentKindV0::Function { arguments, .. }
        | CssValueComponentKindV0::Parenthesized { values: arguments } => {
            math_range_is_provably_accepted(Some(range), arguments)
        }
        CssValueComponentKindV0::Delimiter => matches!(component.text.as_str(), "+" | "-"),
        CssValueComponentKindV0::Comma | CssValueComponentKindV0::Slash => true,
        CssValueComponentKindV0::Ident => matches!(component.text.as_str(), "+" | "-"),
        CssValueComponentKindV0::Hash
        | CssValueComponentKindV0::String
        | CssValueComponentKindV0::Url
        | CssValueComponentKindV0::Bracketed { .. }
        | CssValueComponentKindV0::Braced { .. } => false,
    })
}

fn numeric_range_accepts(range: Option<&NumericRange>, source: &str) -> bool {
    let Some(range) = range else {
        return true;
    };
    let Some(numeric) = parse_numeric_value_with_unit(source) else {
        return false;
    };
    let above_min = range
        .min
        .as_deref()
        .and_then(|value| value.parse::<f64>().ok())
        .is_none_or(|minimum| numeric.value >= minimum);
    let below_max = range
        .max
        .as_deref()
        .and_then(|value| value.parse::<f64>().ok())
        .is_none_or(|maximum| numeric.value <= maximum);
    above_min && below_max
}

fn component_locus(components: &[CssValueComponentV0]) -> CssValueGrammarLocusV0 {
    match (components.first(), components.last()) {
        (Some(first), Some(last)) => CssValueGrammarLocusV0 {
            start: first.span.start,
            end: last.span.end,
        },
        _ => CssValueGrammarLocusV0 { start: 0, end: 0 },
    }
}

fn grammar_defect(
    grammar: &str,
    offset: usize,
    code: impl Into<String>,
    detail: impl Into<String>,
) -> CssValueGrammarVerdictV0 {
    CssValueGrammarVerdictV0::GrammarDefect {
        grammar: grammar.to_string(),
        offset,
        code: code.into(),
        detail: detail.into(),
    }
}

fn strip_matching_quotes(source: &str) -> &str {
    if source.len() >= 2 {
        let bytes = source.as_bytes();
        if matches!(
            (bytes[0], bytes[source.len() - 1]),
            (b'\'', b'\'') | (b'"', b'"')
        ) {
            return &source[1..source.len() - 1];
        }
    }
    source
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, sync::Arc};

    use omena_cascade::{CascadeStandardValueValidatorV0, CascadeStandardValueVerdictV0};
    use omena_spec_audit::{SpecGrammarBoundaryClassificationV0, spec_grammar_registry};
    use omena_syntax::ident::PropertyNameV0;
    use omena_value_lattice::ValueNodeV0;

    use super::{
        CLOSED_WORLD_KEYWORD_CLOSURE_CERTIFICATE_SOURCE, CLOSED_WORLD_TOKEN_KINDS,
        CSS_VALUE_VALIDATION_CONSUMER_POLICIES_V0, CssValueGrammarBudgetKindV0,
        CssValueGrammarBudgetV0, CssValueGrammarLocusV0, CssValueGrammarVerdictV0,
        CssValueValidationClassV0, CssValueValidationReasonV0,
        SpecStandardPropertyValueValidatorV0, adjudicate_css_value_validation,
        adjudicate_css_value_validation_with_boundary, audit_css_value_grammar_registry_v0,
        cached_pinned_vds_expression, closed_world_builtin_profile,
        closed_world_builtin_token_profiles, match_and_type_css_value_grammar_v0,
        match_and_type_standard_property_value_v0, match_css_value_grammar_v0,
        match_standard_property_value_v0, parse_closed_world_keyword_closure_certificate,
        standard_property_closed_world_token_kind_mismatch, validate_registered_property_value_v0,
        validate_standard_property_value_v0,
    };
    use crate::{
        AbstractCssTypedValueV0, AbstractCssValueV0, DeclaredValueKindV0,
        classify_registered_property_declared_value_v0,
    };

    fn assert_matches(grammar: &str, value: &str) {
        let verdict = match_css_value_grammar_v0(
            grammar,
            value,
            spec_grammar_registry(),
            CssValueGrammarBudgetV0::default(),
        );
        assert!(
            verdict.is_matched(),
            "{grammar:?} should match {value:?}: {verdict:?}"
        );
    }

    fn assert_unmatched(grammar: &str, value: &str) {
        let verdict = match_css_value_grammar_v0(
            grammar,
            value,
            spec_grammar_registry(),
            CssValueGrammarBudgetV0::default(),
        );
        assert!(
            verdict.is_definite_mismatch(),
            "{grammar:?} should reject {value:?}: {verdict:?}"
        );
    }

    #[test]
    fn parsed_grammar_cache_reuses_the_immutable_expression() {
        let grammar = "<length> | cache-sentinel";
        let first = cached_pinned_vds_expression(grammar);
        let second = cached_pinned_vds_expression(grammar);

        assert!(matches!(
            (&first, &second),
            (Ok(first), Ok(second)) if Arc::ptr_eq(first, second)
        ));
    }

    #[test]
    fn grammar_conformance_covers_all_combinators_and_multipliers() {
        for (grammar, value) in [
            ("<length> <color>", "1px red"),
            ("<length> && <color>", "red 1px"),
            ("<length> || <color>", "red"),
            ("auto | <length>", "auto"),
            ("[ auto | <length> ]?", ""),
            ("<length>*", "1px 2px"),
            ("<length>+", "1px 2px"),
            ("<length>#", "1px, 2px"),
            ("<length>{2,3}", "1px 2px 3px"),
            ("<length>#{2}", "1px, 2px"),
            ("[ <length>? <color>? ]!", "red"),
            ("rgb( <number>#{3} )", "rgb(1, 2, 3)"),
        ] {
            assert_matches(grammar, value);
        }
        for (grammar, value) in [
            ("<length> <color>", "red 1px"),
            ("<length> && <color>", "1px"),
            ("<length> || <color>", "auto"),
            ("<length>+", ""),
            ("<length>#", "1px 2px"),
            ("<length>{2,3}", "1px"),
            ("[ <length>? <color>? ]!", ""),
            ("rgb( <number>#{3} )", "rgb(1, 2)"),
        ] {
            assert_unmatched(grammar, value);
        }
    }

    #[test]
    fn combinator_precedence_is_juxtaposition_then_and_then_double_or_then_or() {
        let grammar = "a b && c || d | e";
        for value in ["c a b", "a b c", "d", "e"] {
            assert_matches(grammar, value);
        }
        for value in ["a c", "b c", "a b d"] {
            assert_unmatched(grammar, value);
        }
    }

    #[test]
    fn reference_depth_exhaustion_is_typed_and_provenanced() {
        let verdict = match_css_value_grammar_v0(
            "<calc-sum>",
            "calc(1px + 2px)",
            spec_grammar_registry(),
            CssValueGrammarBudgetV0 {
                max_reference_depth: 0,
                ..CssValueGrammarBudgetV0::default()
            },
        );
        assert!(matches!(
            verdict,
            CssValueGrammarVerdictV0::NotMatchedWithinBudget {
                budget: CssValueGrammarBudgetKindV0::ReferenceDepth,
                limit: 0,
                reference: Some(reference),
                ..
            } if reference == "calc-sum"
        ));
    }

    #[test]
    fn malformed_grammar_is_a_defect_not_a_mismatch() {
        let verdict = match_css_value_grammar_v0(
            "[ <length> | <color>",
            "1px",
            spec_grammar_registry(),
            CssValueGrammarBudgetV0::default(),
        );
        assert!(matches!(
            verdict,
            CssValueGrammarVerdictV0::GrammarDefect { .. }
        ));
    }

    #[test]
    fn property_and_type_references_use_the_pinned_registry() {
        assert_matches("<'box-sizing'>", "border-box");
        assert_matches("<color>", "rebeccapurple");
        assert_matches("<rgb()>", "rgb(1 2 3)");
        assert!(match_standard_property_value_v0("box-sizing", "content-box").is_matched());
        assert!(
            match_standard_property_value_v0("box-sizing", "inline-box").is_definite_mismatch()
        );
    }

    #[test]
    fn numeric_reference_ranges_are_enforced() {
        assert_matches("<number [0,1]>", "0.5");
        assert_unmatched("<number [0,1]>", "2");
        assert_matches("<length [0,∞]>", "0px");
        assert_unmatched("<length [0,∞]>", "-1px");
    }

    #[test]
    fn unitless_zero_matches_length_references_without_widening_other_dimensions() {
        for grammar in ["<length>", "<length-percentage>"] {
            assert_matches(grammar, "0");
        }
        for grammar in ["<percentage>", "<angle>", "<time>", "<resolution>"] {
            assert_unmatched(grammar, "0");
        }
        for value in ["1", "-1", "0.5"] {
            assert_unmatched("<length>", value);
            assert_unmatched("<length-percentage>", value);
        }
        let calc_verdict = match_css_value_grammar_v0(
            "<length> | <calc-sum>",
            "calc(0 + 2px)",
            spec_grammar_registry(),
            CssValueGrammarBudgetV0::default(),
        );
        assert!(
            !calc_verdict.is_matched(),
            "unitless zero cannot be added to a dimension inside calc(): {calc_verdict:?}"
        );
        assert!(
            match_css_value_grammar_v0(
                "<length> | <calc-sum>",
                "calc(0px + 2px)",
                spec_grammar_registry(),
                CssValueGrammarBudgetV0::default(),
            )
            .is_matched()
        );
    }

    #[test]
    fn standard_properties_accept_unitless_zero_without_retyping_integer_consumers() {
        for (property, value) in [
            ("padding", "0"),
            ("margin", "0 auto"),
            ("border-width", "0 4px 6px"),
            ("width", "0"),
            ("z-index", "0"),
            ("opacity", "0"),
        ] {
            let verdict = validate_standard_property_value_v0(property, value);
            assert_eq!(
                verdict.class,
                CssValueValidationClassV0::Valid,
                "{property}: {value} should be valid: {verdict:?}"
            );
        }
        assert_eq!(
            classify_registered_property_declared_value_v0("0"),
            DeclaredValueKindV0::Integer
        );
    }

    #[test]
    fn nullable_all_in_any_order_operands_can_satisfy_their_slots() {
        let grammar = "[ alpha? && [ none | beta ] && gamma? ]";
        for value in ["none", "alpha none", "none gamma", "gamma none alpha"] {
            assert_matches(grammar, value);
        }
        assert_unmatched(grammar, "none unexpected");
    }

    #[test]
    fn nested_property_references_keep_inner_comma_repetition_reachable() {
        assert_matches("<'box-shadow-color'>", "red, blue");
        assert_unmatched("<'box-shadow-color'>", "red blue");

        let grammar = "[ <'box-shadow-color'>? && [ none | <length>{2} ] [ <'box-shadow-blur'> <'box-shadow-spread'>? ]? && <'box-shadow-position'>? ]";
        assert_matches(grammar, "red none");
        assert_unmatched(grammar, "red none unexpected");
    }

    #[test]
    fn sequence_omits_a_comma_only_with_an_omitted_adjacent_component() {
        let grammar = "<length>? , <color>";
        assert_matches(grammar, "red");
        assert_matches(grammar, "1px, red");
        assert_unmatched(grammar, ", red");
        assert_unmatched(grammar, "1px red");
    }

    #[test]
    fn standard_keyword_grammars_remain_precise_through_nested_expansion() {
        for (property, value) in [("box-shadow", "none"), ("background", "transparent")] {
            let verdict = validate_standard_property_value_v0(property, value);
            assert_eq!(
                verdict.class,
                CssValueValidationClassV0::Valid,
                "{property}: {value} should be valid: {verdict:?}"
            );
        }
        for (property, value) in [
            ("box-shadow", "1px nonsense"),
            ("background", ", transparent"),
        ] {
            let verdict = validate_standard_property_value_v0(property, value);
            assert_ne!(
                verdict.class,
                CssValueValidationClassV0::Valid,
                "{property}: {value} must not be accepted: {verdict:?}"
            );
        }
    }

    #[test]
    fn reviewed_compatibility_syntax_and_boundary_policy_are_applied_independently() {
        let compatibility_value =
            validate_standard_property_value_v0("-webkit-background-clip", "text");
        assert_eq!(compatibility_value.class, CssValueValidationClassV0::Valid);
        assert_eq!(
            compatibility_value.reason,
            CssValueValidationReasonV0::GrammarMatched
        );
        let registry = spec_grammar_registry();
        let compatibility_entry = registry.entry("properties", "-webkit-background-clip");
        assert!(
            compatibility_entry.is_some(),
            "compatibility property must remain registered"
        );
        let Some(compatibility_entry) = compatibility_entry else {
            return;
        };
        assert!(compatibility_entry.override_provenance.is_some());

        let forward_tier =
            validate_standard_property_value_v0("background", "definitely-not-a-background");
        assert_eq!(
            forward_tier.class,
            CssValueValidationClassV0::NotValidatable
        );
        assert_eq!(
            forward_tier.reason,
            CssValueValidationReasonV0::ForwardTierGrammar
        );
        assert!(forward_tier.verdict.is_definite_mismatch());

        let in_boundary = validate_standard_property_value_v0("box-sizing", "inline-box");
        assert_eq!(in_boundary.class, CssValueValidationClassV0::Invalid);
        assert_eq!(
            in_boundary.reason,
            CssValueValidationReasonV0::GrammarUnmatched
        );
    }

    #[test]
    fn pinned_registry_rows_are_all_accounted_for_by_the_grammar_parser() {
        const MIN_PINNED_REGISTRY_ENTRY_COUNT: usize = 1_700;

        let registry = spec_grammar_registry();
        let audit = audit_css_value_grammar_registry_v0(registry);
        assert_eq!(audit.total_entry_count, registry.total_entry_count());
        assert!(
            audit.total_entry_count >= MIN_PINNED_REGISTRY_ENTRY_COUNT,
            "pinned registry unexpectedly shrank below the audited coverage floor"
        );
        assert_eq!(audit.categories.len(), 5);
        assert_eq!(
            audit.parsed_entry_count + audit.missing_syntax_count + audit.grammar_defect_count,
            audit.total_entry_count
        );
        let properties = audit
            .categories
            .iter()
            .find(|category| category.category == "properties");
        assert_eq!(
            properties.map(|category| (
                category.entry_count,
                category.parsed_entry_count,
                category.missing_syntax_count,
                category.grammar_defect_count,
            )),
            Some((817, 812, 5, 0))
        );
        assert_eq!(
            (
                audit.parsed_entry_count,
                audit.missing_syntax_count,
                audit.grammar_defect_count,
            ),
            (1_529, 131, 57)
        );
    }

    #[test]
    fn matched_compounds_project_through_existing_typed_and_lattice_domains() {
        let border = match_and_type_standard_property_value_v0("border-top", "1px solid red");
        assert!(border.verdict.is_matched(), "{:?}", border.verdict);
        assert!(matches!(
            &border.abstract_value,
            AbstractCssValueV0::Exact {
                typed: Some(typed), ..
            } if matches!(
                typed.as_ref(),
                AbstractCssTypedValueV0::Compound { leaves } if leaves.len() == 3
            )
        ));
        assert!(matches!(
            border.projection.as_ref().map(|projection| projection.lattice.root()),
            Some(ValueNodeV0::List { items, .. }) if items.len() == 3
        ));

        let calc = match_and_type_css_value_grammar_v0(
            "calc( <length> '+' <length> )",
            "calc(1px + 2px)",
            spec_grammar_registry(),
            CssValueGrammarBudgetV0::default(),
        );
        assert!(calc.verdict.is_matched(), "{:?}", calc.verdict);
        assert!(matches!(
            calc.projection.as_ref().map(|projection| projection.lattice.root()),
            Some(ValueNodeV0::Function { name, arguments, .. })
                if *name == "calc" && arguments.len() == 3
        ));

        let font_families =
            match_and_type_standard_property_value_v0("font-family", "serif, sans-serif");
        assert!(
            font_families.verdict.is_matched(),
            "{:?}",
            font_families.verdict
        );
        assert!(matches!(
            font_families
                .projection
                .as_ref()
                .map(|projection| projection.lattice.root()),
            Some(ValueNodeV0::List { .. })
        ));
    }

    #[test]
    fn rejected_value_preserves_raw_bytes_and_carries_the_match_locus() {
        let source = "  1px nonsense red  ";
        let result = match_and_type_standard_property_value_v0("border-top", source);
        assert!(matches!(
            result.verdict,
            CssValueGrammarVerdictV0::Unmatched {
                grammar,
                locus,
            } if grammar == "<line-width> || <line-style> || <color>"
                && locus.start == 2
                && locus.end == source.len() - 2
        ));
        assert_eq!(
            result.abstract_value,
            AbstractCssValueV0::Raw {
                value: source.to_string(),
            }
        );
        assert!(result.projection.is_none());
    }

    #[test]
    fn validation_keeps_invalid_and_not_validatable_outcomes_distinct() {
        let invalid = validate_standard_property_value_v0("box-sizing", "inline-box");
        assert_eq!(invalid.class, CssValueValidationClassV0::Invalid);
        assert_eq!(invalid.reason, CssValueValidationReasonV0::GrammarUnmatched);

        let closed_world_mismatch = validate_standard_property_value_v0("z-index", "banana");
        assert_eq!(
            closed_world_mismatch.class,
            CssValueValidationClassV0::Invalid
        );
        assert_eq!(
            closed_world_mismatch.reason,
            CssValueValidationReasonV0::GrammarUnmatched
        );

        let defect = validate_registered_property_value_v0("<future-value>", "1px");
        assert_eq!(defect.class, CssValueValidationClassV0::NotValidatable);
        assert_eq!(defect.reason, CssValueValidationReasonV0::GrammarDefect);

        let budget_verdict = match_css_value_grammar_v0(
            "<calc-sum>",
            "calc(1px + 2px)",
            spec_grammar_registry(),
            CssValueGrammarBudgetV0 {
                max_reference_depth: 0,
                ..CssValueGrammarBudgetV0::default()
            },
        );
        let budget = adjudicate_css_value_validation("1px", budget_verdict);
        assert_eq!(budget.class, CssValueValidationClassV0::NotValidatable);
        assert_eq!(
            budget.reason,
            CssValueValidationReasonV0::MatchBudgetExhausted
        );

        let deferred = validate_standard_property_value_v0("width", "var(--width)");
        assert_eq!(deferred.class, CssValueValidationClassV0::NotValidatable);
        assert_eq!(
            deferred.reason,
            CssValueValidationReasonV0::DeferredSubstitution
        );
    }

    #[test]
    fn validation_distinguishes_negative_dimensions_from_vendor_identifiers() {
        let valid_negative = validate_standard_property_value_v0("margin", "-10px");
        assert_eq!(valid_negative.class, CssValueValidationClassV0::Valid);
        assert_eq!(
            valid_negative.reason,
            CssValueValidationReasonV0::GrammarMatched
        );
        assert!(valid_negative.verdict.is_matched());

        let invalid_negative = validate_standard_property_value_v0("margin", "-10px totally-bogus");
        assert_eq!(invalid_negative.class, CssValueValidationClassV0::Invalid);
        assert_eq!(
            invalid_negative.reason,
            CssValueValidationReasonV0::GrammarUnmatched
        );
        assert!(invalid_negative.verdict.is_definite_mismatch());

        let vendor_identifier =
            validate_standard_property_value_v0("box-sizing", "-webkit-border-box");
        assert_eq!(
            vendor_identifier.class,
            CssValueValidationClassV0::NotValidatable
        );
        assert_eq!(
            vendor_identifier.reason,
            CssValueValidationReasonV0::VendorExtension
        );
        assert!(vendor_identifier.verdict.is_definite_mismatch());
    }

    #[test]
    fn function_tokens_preserve_validation_boundaries() {
        let math_function = validate_standard_property_value_v0("width", "round(up, 101px, 10px)");
        assert_eq!(
            math_function.class,
            CssValueValidationClassV0::NotValidatable
        );
        assert_eq!(
            math_function.reason,
            CssValueValidationReasonV0::UnvalidatedStandardFunction
        );
        assert!(math_function.verdict.is_definite_mismatch());

        let quoted_text = validate_standard_property_value_v0("content", "\"var(\"");
        assert_eq!(quoted_text.class, CssValueValidationClassV0::Valid);
        assert_eq!(
            quoted_text.reason,
            CssValueValidationReasonV0::GrammarMatched
        );
        assert!(quoted_text.verdict.is_matched());

        let grid_function =
            validate_standard_property_value_v0("grid-template-columns", "minmax(101px, 1fr)");
        assert_eq!(grid_function.class, CssValueValidationClassV0::Valid);
        assert_eq!(
            grid_function.reason,
            CssValueValidationReasonV0::GrammarMatched
        );
    }

    #[test]
    fn recognized_functions_do_not_mask_adjacent_invalid_components() {
        for value in [
            "round(up, 101px, 10px)",
            "mod(10px, 3px)",
            "rem(10px, 3px)",
            "sin(45deg)",
            "pow(2, 3)",
            "sqrt(4)",
            "hypot(3px, 4px)",
            "abs(-10px)",
        ] {
            let validation = validate_standard_property_value_v0("width", value);
            assert_eq!(
                validation.class,
                CssValueValidationClassV0::NotValidatable,
                "{value} must remain non-definite until its function semantics are modeled"
            );
            assert_eq!(
                validation.reason,
                CssValueValidationReasonV0::UnvalidatedStandardFunction,
                "{value} must be attributed to the unvalidated standard-function channel"
            );
        }

        let adjacent_scalar =
            validate_standard_property_value_v0("margin", "round(1, 2) totally-bogus");
        assert_eq!(adjacent_scalar.class, CssValueValidationClassV0::Invalid);
        assert_eq!(
            adjacent_scalar.reason,
            CssValueValidationReasonV0::GrammarUnmatched
        );

        let unregistered_function =
            validate_standard_property_value_v0("width", "totally-unknown(1px)");
        assert_eq!(
            unregistered_function.class,
            CssValueValidationClassV0::NotValidatable
        );
        assert_eq!(
            unregistered_function.reason,
            CssValueValidationReasonV0::MatcherCoverageIncomplete
        );

        let compound_value =
            validate_standard_property_value_v0("margin", "round(up, 10px, 1px) auto");
        assert_eq!(
            compound_value.class,
            CssValueValidationClassV0::NotValidatable
        );
        assert_eq!(
            compound_value.reason,
            CssValueValidationReasonV0::UnvalidatedStandardFunction
        );
    }

    #[test]
    fn deferred_validation_uses_parsed_function_names() {
        for value in [
            "var(--width)",
            "env(safe-area-inset-top)",
            "attr(data-width type(<length>))",
        ] {
            let validation = validate_standard_property_value_v0("width", value);
            assert_eq!(
                validation.class,
                CssValueValidationClassV0::NotValidatable,
                "{value} must remain deferred"
            );
            assert_eq!(
                validation.reason,
                CssValueValidationReasonV0::DeferredSubstitution,
                "{value} must be attributed to an actual deferred function component"
            );
        }

        let unvalidated_outer_function =
            validate_standard_property_value_v0("width", "round(up, calc(101px), 10px)");
        assert_eq!(
            unvalidated_outer_function.class,
            CssValueValidationClassV0::NotValidatable
        );
        assert_eq!(
            unvalidated_outer_function.reason,
            CssValueValidationReasonV0::UnvalidatedStandardFunction
        );

        let similarly_named =
            validate_standard_property_value_v0("grid-template-columns", "minmax(101px, 1fr)");
        assert_eq!(
            similarly_named.reason,
            CssValueValidationReasonV0::GrammarMatched
        );
    }

    #[test]
    fn pinned_matcher_resolves_paint_math_and_grid_function_shapes() {
        for (property, value) in [
            ("fill", "#ff00aa"),
            ("stroke", "rgb(10 20 30)"),
            ("fill", "context-fill"),
            ("fill", "context-stroke"),
            ("stroke", "context-fill"),
            ("stroke", "context-stroke"),
            ("width", "calc(1px + 2px)"),
            ("width", "calc(100% - 8px)"),
            ("width", "calc(8px - 4px)"),
            ("width", "min(1px, 2px)"),
            ("width", "max(10%, 20%)"),
            ("width", "clamp(1px, 2px, 3px)"),
            ("opacity", "calc(0.4 + 0.1)"),
            ("animation-duration", "max(1s, 2s)"),
            ("rotate", "calc(10deg + 5deg)"),
            ("grid-template-columns", "minmax(101px, 1fr)"),
            ("grid-template-columns", "repeat(3, 1fr)"),
            ("grid-template-columns", "repeat(2, minmax(0, 1fr))"),
        ] {
            let validation = validate_standard_property_value_v0(property, value);
            assert_eq!(
                validation.class,
                CssValueValidationClassV0::Valid,
                "{property}: {value}: {validation:?}"
            );
        }

        for (property, value) in [
            ("width", "calc(1px + 2)"),
            ("width", "calc(1px * 2px)"),
            ("width", "calc(100% -8px)"),
            ("width", "calc(100%- 8px)"),
            ("opacity", "calc(1 + 1px)"),
            ("animation-duration", "min(1s, 2px)"),
        ] {
            let validation = validate_standard_property_value_v0(property, value);
            assert_ne!(
                validation.class,
                CssValueValidationClassV0::Valid,
                "dimensionally invalid math was accepted: {property}: {value}: {validation:?}"
            );
        }
    }

    #[test]
    fn css_tree_keyword_closure_regressions_are_valid_across_the_validator_adapter() {
        let validator = SpecStandardPropertyValueValidatorV0;
        let regressions = [
            ("fill", "context-fill"),
            ("fill", "context-stroke"),
            ("stroke", "context-fill"),
            ("stroke", "context-stroke"),
            ("zoom", "normal"),
            ("zoom", "reset"),
            ("baseline-shift", "baseline"),
            ("-webkit-mask", "border"),
            ("-webkit-mask", "content"),
            ("-webkit-mask", "padding"),
            ("-webkit-mask", "text"),
        ];
        assert_eq!(regressions.len(), 11);
        assert_eq!(
            regressions
                .iter()
                .map(|(_, value)| *value)
                .collect::<BTreeSet<_>>()
                .len(),
            9
        );
        for (property, value) in regressions {
            let validation = validate_standard_property_value_v0(property, value);
            assert_eq!(
                validation.class,
                CssValueValidationClassV0::Valid,
                "{property}: {value}: {validation:?}"
            );
            assert_eq!(
                validator
                    .validate_standard_property_value(&PropertyNameV0::standard(property), value,),
                CascadeStandardValueVerdictV0::Matched,
                "{property}: {value} must cross the cascade validator adapter as matched"
            );
        }
    }

    #[test]
    fn keyword_closure_certificate_binds_the_tested_pairs_and_nonempty_certification() {
        let authority = parse_closed_world_keyword_closure_certificate(
            CLOSED_WORLD_KEYWORD_CLOSURE_CERTIFICATE_SOURCE,
        );
        assert!(
            authority.is_some(),
            "the embedded keyword-closure certificate must pass its in-binary integrity checks"
        );
        let Some(authority) = authority else {
            return;
        };
        assert_eq!(authority.certified_properties.len(), 388);
        assert_eq!(authority.accepted_keywords_by_property.len(), 704);
        assert_eq!(
            authority
                .accepted_keywords_by_property
                .values()
                .map(BTreeSet::len)
                .sum::<usize>(),
            16_445
        );
        assert!(
            authority
                .accepted_keywords_by_property
                .get(&PropertyNameV0::canonical_standard_key("content"))
                .is_some_and(|keywords| keywords.contains("open-quote"))
        );

        let wrong_digest = serde_json::from_str::<serde_json::Value>(
            CLOSED_WORLD_KEYWORD_CLOSURE_CERTIFICATE_SOURCE,
        );
        assert!(wrong_digest.is_ok(), "embedded certificate JSON must parse");
        let Ok(mut wrong_digest) = wrong_digest else {
            return;
        };
        wrong_digest["acceptedPairDigest"] = serde_json::Value::String("0".repeat(64));
        let wrong_digest_source = serde_json::to_string(&wrong_digest);
        assert!(
            wrong_digest_source.is_ok(),
            "mutated certificate JSON must serialize"
        );
        let Ok(wrong_digest_source) = wrong_digest_source else {
            return;
        };
        assert!(
            parse_closed_world_keyword_closure_certificate(&wrong_digest_source).is_none(),
            "the accepted-pair digest must be checked by the in-binary loader"
        );

        let zero_sample = serde_json::from_str::<serde_json::Value>(
            CLOSED_WORLD_KEYWORD_CLOSURE_CERTIFICATE_SOURCE,
        );
        assert!(zero_sample.is_ok(), "embedded certificate JSON must parse");
        let Ok(mut zero_sample) = zero_sample else {
            return;
        };
        let zero_sample_property = zero_sample["propertyTests"].as_array().and_then(|entries| {
            entries.iter().find_map(|entry| {
                if entry["testedPairCount"].as_u64() == Some(0) {
                    entry["property"].as_str().map(str::to_owned)
                } else {
                    None
                }
            })
        });
        assert!(
            zero_sample_property.is_some(),
            "the exhaustive property table must include a zero-accepted property"
        );
        let Some(zero_sample_property) = zero_sample_property else {
            return;
        };
        let certified_properties = zero_sample["certifiedProperties"].as_array_mut();
        assert!(
            certified_properties.is_some(),
            "certified property list must be an array"
        );
        let Some(certified_properties) = certified_properties else {
            return;
        };
        certified_properties.push(serde_json::Value::String(zero_sample_property));
        let zero_sample_source = serde_json::to_string(&zero_sample);
        assert!(
            zero_sample_source.is_ok(),
            "mutated certificate JSON must serialize"
        );
        let Ok(zero_sample_source) = zero_sample_source else {
            return;
        };
        assert!(
            parse_closed_world_keyword_closure_certificate(&zero_sample_source).is_none(),
            "a property with zero tested pairs must never be certified"
        );
    }

    #[test]
    fn closed_world_builtin_domains_are_bound_to_the_css_tree_witness_manifest() {
        let manifest = closed_world_builtin_token_profiles();
        assert!(
            manifest.is_some(),
            "the css-tree builtin token witness manifest must parse"
        );
        let Some(manifest) = manifest else {
            return;
        };
        assert_eq!(manifest.profile_count, 33);

        let length = closed_world_builtin_profile("length");
        assert!(
            length.is_some(),
            "length must have a witnessed builtin profile"
        );
        let Some(length) = length else {
            return;
        };
        assert!(length.dimension.open);
        assert!(length.function_name.open);
        assert!(!length.number.open);
        assert_eq!(length.number.allowed, BTreeSet::from(["0".to_string()]));

        let unknown_css_tree_type = closed_world_builtin_profile("whole-value");
        assert!(
            unknown_css_tree_type.is_some(),
            "an unknown css-tree type must receive a default-open profile"
        );
        let Some(unknown_css_tree_type) = unknown_css_tree_type else {
            return;
        };
        assert!(
            CLOSED_WORLD_TOKEN_KINDS
                .iter()
                .all(|kind| unknown_css_tree_type.domain(*kind).open)
        );
        assert!(closed_world_builtin_profile("named-color").is_none());
    }

    #[test]
    fn incomplete_matcher_coverage_cannot_promote_unmatched_to_invalid() {
        let validation = adjudicate_css_value_validation_with_boundary(
            "fixture-value",
            CssValueGrammarVerdictV0::Unmatched {
                grammar: "<partially-modeled-type>".to_string(),
                locus: CssValueGrammarLocusV0 { start: 0, end: 13 },
            },
            SpecGrammarBoundaryClassificationV0::InBoundary,
            false,
            false,
        );
        assert_eq!(
            validation.class,
            CssValueValidationClassV0::NotValidatable,
            "MatcherCoverageIncomplete must remain non-definite"
        );
        assert_eq!(
            validation.reason,
            CssValueValidationReasonV0::MatcherCoverageIncomplete
        );
    }

    #[test]
    fn accepted_keyword_authority_prevents_oracle_valid_ident_rejection() {
        let validation = validate_standard_property_value_v0("content", "open-quote");
        assert_eq!(
            validation.class,
            CssValueValidationClassV0::NotValidatable,
            "an oracle-accepted matcher gap cannot become a definite rejection: {validation:?}"
        );
        assert_eq!(
            validation.reason,
            CssValueValidationReasonV0::MatcherCoverageIncomplete
        );
    }

    #[test]
    fn open_preprocessor_function_keeps_compound_ident_rejection_non_definite() {
        let validation =
            validate_standard_property_value_v0("box-shadow", "inset 0 0 0 2px fade(#0ea5e9, 24%)");
        assert_eq!(
            validation.class,
            CssValueValidationClassV0::NotValidatable,
            "an unevaluated preprocessor function must preserve uncertainty: {validation:?}"
        );
        assert_eq!(
            validation.reason,
            CssValueValidationReasonV0::MatcherCoverageIncomplete
        );
    }

    #[test]
    fn closed_world_token_kinds_certify_impossible_standard_values() {
        for (property, value) in [
            ("color", "12px"),
            ("color", "definitely-not-a-color"),
            ("width", "red"),
            ("border-top", "1px nonsense red"),
            ("fill", "bogusvalue"),
            ("z-index", "banana"),
            ("margin", "-10px totally-bogus"),
        ] {
            let validation = validate_standard_property_value_v0(property, value);
            assert_eq!(
                validation.class,
                CssValueValidationClassV0::Invalid,
                "{property}: {value}: {validation:?}"
            );
            assert_eq!(
                validation.reason,
                CssValueValidationReasonV0::GrammarUnmatched,
                "{property}: {value}: {validation:?}"
            );
        }
    }

    #[test]
    fn valid_declaration_corpus_covers_closed_ident_edges_without_definite_rejection() {
        let declarations = [
            ("color", "red"),
            ("color", "#ff00aa"),
            ("fill", "#f0f"),
            ("stroke", "#ff00ff"),
            ("width", "calc(10px + 2px)"),
            ("width", "min(10px, 20px)"),
            ("width", "max(10%, 20%)"),
            ("width", "clamp(1px, 2px, 3px)"),
            ("height", "calc(50% + 2px)"),
            ("margin", "calc(1rem + 2px)"),
            ("padding", "min(1rem, 2rem)"),
            ("row-gap", "clamp(1px, 2px, 3px)"),
            ("column-gap", "max(1%, 2%)"),
            ("gap", "clamp(1px, 2px, 3px)"),
            ("opacity", "calc(0.5 + 0.1)"),
            ("line-height", "min(1.2, 1.5)"),
            ("animation-duration", "calc(1s + 200ms)"),
            ("transition-duration", "max(1s, 2s)"),
            ("rotate", "calc(10deg + 5deg)"),
            ("grid-template-columns", "minmax(101px, 1fr)"),
            ("grid-template-columns", "repeat(3, 1fr)"),
            ("grid-template-columns", "repeat(2, minmax(0, 1fr))"),
            ("grid-template-columns", "1fr 2fr"),
            ("border-top", "1px solid red"),
            ("margin", "0 auto"),
            ("padding", "1px 2px"),
            ("display", "grid"),
            ("position", "absolute"),
            ("inset", "0"),
            ("top", "1px"),
            ("z-index", "2"),
            ("font-weight", "700"),
            ("font-size", "16px"),
            ("background-color", "rebeccapurple"),
            ("border-radius", "4px"),
            ("flex-grow", "1"),
            ("flex-shrink", "0"),
            ("order", "-1"),
            ("transform", "rotate(45deg)"),
            ("background-image", "linear-gradient(red, blue)"),
            ("color", "CanvasText"),
            ("fill", "context-fill"),
            ("stroke", "context-stroke"),
            ("zoom", "normal"),
            ("baseline-shift", "baseline"),
            ("-webkit-mask", "border"),
            ("-webkit-mask", "text"),
            ("content", "open-quote"),
        ];
        assert_eq!(declarations.len(), 48);
        let definite_rejections = declarations
            .iter()
            .filter_map(|(property, value)| {
                let property_name = PropertyNameV0::from_authored(*property);
                assert!(
                    !standard_property_closed_world_token_kind_mismatch(
                        &property_name,
                        value,
                        spec_grammar_registry(),
                    ),
                    "a valid declaration was outside the derived open/closed token profile: {property}: {value}"
                );
                let validation = validate_standard_property_value_v0(property, value);
                (validation.class == CssValueValidationClassV0::Invalid)
                    .then_some((*property, *value, validation))
            })
            .collect::<Vec<_>>();
        assert!(
            definite_rejections.is_empty(),
            "valid declarations were definitely rejected: {definite_rejections:?}"
        );
    }

    #[test]
    fn validation_consumer_policy_table_covers_every_live_consumer() {
        assert_eq!(CSS_VALUE_VALIDATION_CONSUMER_POLICIES_V0.len(), 5);
        assert_eq!(
            CSS_VALUE_VALIDATION_CONSUMER_POLICIES_V0
                .iter()
                .map(|policy| policy.consumer)
                .collect::<Vec<_>>(),
            vec![
                "checker.registeredPropertyTypeMismatch",
                "checker.invalidPropertyValue",
                "cascade.postSubstitutionStandardProperty",
                "scss.nativeCssFunctionParameter",
                "scss.nativeCssFunctionReturn",
            ]
        );
        for policy in CSS_VALUE_VALIDATION_CONSUMER_POLICIES_V0 {
            assert_eq!(policy.matched, "accept");
            assert!(matches!(policy.unmatched, "diagnostic" | "reject"));
            assert!(matches!(
                policy.forward_tier_unmatched,
                "not-applicable" | "not-validatable"
            ));
            assert!(matches!(policy.grammar_defect, "silent" | "unknown"));
            assert!(matches!(policy.budget_exhausted, "silent" | "unknown"));
        }
    }

    #[test]
    fn cascade_validator_adapter_preserves_spec_grammar_outcomes() {
        let validator = SpecStandardPropertyValueValidatorV0;

        assert_eq!(
            validator.validate_standard_property_value(&PropertyNameV0::standard("color"), "red"),
            CascadeStandardValueVerdictV0::Matched
        );
        assert_eq!(
            validator.validate_standard_property_value(&PropertyNameV0::standard("color"), "12px"),
            CascadeStandardValueVerdictV0::Unmatched
        );
        assert_eq!(
            validator.validate_standard_property_value(
                &PropertyNameV0::standard("box-sizing"),
                "inline-box",
            ),
            CascadeStandardValueVerdictV0::Unmatched
        );
        assert_eq!(
            validator.validate_standard_property_value(
                &PropertyNameV0::standard("color"),
                "var(--tone)",
            ),
            CascadeStandardValueVerdictV0::Unknown
        );
    }
}
