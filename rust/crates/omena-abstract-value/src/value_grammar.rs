use std::collections::{BTreeSet, HashMap};

use omena_spec_audit::{SpecGrammarRegistryV0, spec_grammar_registry};
use omena_value_lattice::{
    CssValueComponentKindV0, CssValueComponentV0, DeclarationValueLensV0, ValueNodeV0,
    css_value_component_stream, declaration_value_lens, parse_numeric_value_with_unit,
};
use serde::{Deserialize, Serialize};

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
    pub grammar_defect: &'static str,
    pub budget_exhausted: &'static str,
}

pub const CSS_VALUE_VALIDATION_CONSUMER_POLICIES_V0: [CssValueValidationConsumerPolicyV0; 4] = [
    CssValueValidationConsumerPolicyV0 {
        consumer: "checker.registeredPropertyTypeMismatch",
        matched: "accept",
        unmatched: "diagnostic",
        grammar_defect: "silent",
        budget_exhausted: "silent",
    },
    CssValueValidationConsumerPolicyV0 {
        consumer: "checker.invalidPropertyValue",
        matched: "accept",
        unmatched: "diagnostic",
        grammar_defect: "silent",
        budget_exhausted: "silent",
    },
    CssValueValidationConsumerPolicyV0 {
        consumer: "scss.nativeCssFunctionParameter",
        matched: "accept",
        unmatched: "reject",
        grammar_defect: "unknown",
        budget_exhausted: "unknown",
    },
    CssValueValidationConsumerPolicyV0 {
        consumer: "scss.nativeCssFunctionReturn",
        matched: "accept",
        unmatched: "reject",
        grammar_defect: "unknown",
        budget_exhausted: "unknown",
    },
];

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
    let registry = spec_grammar_registry();
    let Some(entry) = registry.entry("properties", property) else {
        return grammar_defect(
            "",
            0,
            "unknownProperty",
            format!("property {property:?} is absent from the pinned registry"),
        );
    };
    let Some(grammar) = entry.syntax.as_deref() else {
        return grammar_defect(
            "",
            0,
            "missingPropertyGrammar",
            format!("property {property:?} has no syntax in the pinned registry"),
        );
    };
    if matches!(
        classify_registered_property_declared_value_v0(value),
        DeclaredValueKindV0::CssWide
    ) {
        return CssValueGrammarVerdictV0::Matched {
            grammar: grammar.to_string(),
            consumed_components: 1,
        };
    }
    match_css_value_grammar_v0(grammar, value, registry, CssValueGrammarBudgetV0::default())
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
    adjudicate_css_value_validation(value, match_standard_property_value_v0(property, value))
}

pub fn validate_registered_property_value_v0(syntax: &str, value: &str) -> CssValueValidationV0 {
    adjudicate_css_value_validation(value, match_registered_property_value_v0(syntax, value))
}

fn adjudicate_css_value_validation(
    value: &str,
    verdict: CssValueGrammarVerdictV0,
) -> CssValueValidationV0 {
    let (class, reason) = if contains_deferred_css_value(value) {
        (
            CssValueValidationClassV0::NotValidatable,
            CssValueValidationReasonV0::DeferredSubstitution,
        )
    } else if value.trim_start().starts_with('-') {
        (
            CssValueValidationClassV0::NotValidatable,
            CssValueValidationReasonV0::VendorExtension,
        )
    } else {
        match verdict {
            CssValueGrammarVerdictV0::Matched { .. } => (
                CssValueValidationClassV0::Valid,
                CssValueValidationReasonV0::GrammarMatched,
            ),
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

fn contains_deferred_css_value(value: &str) -> bool {
    let compact = value
        .chars()
        .filter(|character| !character.is_ascii_whitespace())
        .flat_map(char::to_lowercase)
        .collect::<String>();
    ["var(", "env(", "attr(", "calc(", "min(", "max(", "clamp("]
        .iter()
        .any(|function| compact.contains(function))
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
    let locus = component_locus(components);
    let mut context = MatchContext {
        registry,
        budget,
        match_steps: 0,
        first_stop: None,
        grammar_cache: HashMap::new(),
    };
    let ends = context.match_expression(&expression, components, 0, 0);
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
            let Some(relative_end) = rest.find('}') else {
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
            name: property.to_ascii_lowercase(),
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
    grammar_cache: HashMap<(ReferenceCategory, String), Result<VdsExpression, VdsParseError>>,
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
        let mut positions = BTreeSet::from([position]);
        for expression in expressions {
            let mut next = BTreeSet::new();
            for position in positions {
                next.extend(self.match_expression(
                    expression,
                    components,
                    position,
                    reference_depth,
                ));
            }
            positions = self.cap_states(next, None);
            if positions.is_empty() {
                break;
            }
        }
        positions
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
                    if end > current {
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
        if let Some(positions) = match_builtin_reference(reference, components, position) {
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
        let parsed = self
            .grammar_cache
            .entry(key)
            .or_insert_with(|| VdsParser::new(source).parse())
            .clone();
        let expression = match parsed {
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
                &expression,
                components,
                position,
                reference_depth + 1,
            );
        }
        self.match_expression(&expression, components, position, reference_depth + 1)
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

fn match_builtin_reference(
    reference: &VdsReference,
    components: &[CssValueComponentV0],
    position: usize,
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
    let kind = classify_registered_property_declared_value_v0(component.text.as_str());
    let accepted = match reference.name.as_str() {
        "number" | "number-token" => {
            matches!(
                kind,
                DeclaredValueKindV0::Number | DeclaredValueKindV0::Integer
            )
        }
        "integer" => matches!(kind, DeclaredValueKindV0::Integer),
        "length" => matches!(
            kind,
            DeclaredValueKindV0::Dimension(DeclaredNumericTypeV0::Length)
        ),
        "percentage" | "percentage-token" => matches!(
            kind,
            DeclaredValueKindV0::Dimension(DeclaredNumericTypeV0::Percentage)
        ),
        "length-percentage" => matches!(
            kind,
            DeclaredValueKindV0::Dimension(
                DeclaredNumericTypeV0::Length | DeclaredNumericTypeV0::Percentage
            )
        ),
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
        "hex-color" => matches!(kind, DeclaredValueKindV0::HexColor),
        "named-color" => matches!(kind, DeclaredValueKindV0::ColorKeyword(_)),
        "custom-ident" => {
            matches!(component.kind, CssValueComponentKindV0::Ident)
                && !matches!(kind, DeclaredValueKindV0::CssWide)
        }
        "ident" | "ident-token" => matches!(component.kind, CssValueComponentKindV0::Ident),
        "dashed-ident" | "custom-property-name" => {
            matches!(component.kind, CssValueComponentKindV0::Ident)
                && component.text.starts_with("--")
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
    use omena_spec_audit::spec_grammar_registry;
    use omena_value_lattice::ValueNodeV0;

    use super::{
        CSS_VALUE_VALIDATION_CONSUMER_POLICIES_V0, CssValueGrammarBudgetKindV0,
        CssValueGrammarBudgetV0, CssValueGrammarVerdictV0, CssValueValidationClassV0,
        CssValueValidationReasonV0, adjudicate_css_value_validation,
        audit_css_value_grammar_registry_v0, match_and_type_css_value_grammar_v0,
        match_and_type_standard_property_value_v0, match_css_value_grammar_v0,
        match_standard_property_value_v0, validate_registered_property_value_v0,
        validate_standard_property_value_v0,
    };
    use crate::{AbstractCssTypedValueV0, AbstractCssValueV0};

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
    fn pinned_registry_rows_are_all_accounted_for_by_the_grammar_parser() {
        let audit = audit_css_value_grammar_registry_v0(spec_grammar_registry());
        assert_eq!(audit.total_entry_count, 1_717);
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
            Some((815, 810, 5, 0))
        );
        assert_eq!(
            (
                audit.parsed_entry_count,
                audit.missing_syntax_count,
                audit.grammar_defect_count,
            ),
            (1_528, 132, 57)
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
        let invalid = validate_standard_property_value_v0("border-top", "1px nonsense red");
        assert_eq!(invalid.class, CssValueValidationClassV0::Invalid);
        assert_eq!(invalid.reason, CssValueValidationReasonV0::GrammarUnmatched);

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
    fn validation_consumer_policy_table_covers_every_live_consumer() {
        assert_eq!(CSS_VALUE_VALIDATION_CONSUMER_POLICIES_V0.len(), 4);
        assert_eq!(
            CSS_VALUE_VALIDATION_CONSUMER_POLICIES_V0
                .iter()
                .map(|policy| policy.consumer)
                .collect::<Vec<_>>(),
            vec![
                "checker.registeredPropertyTypeMismatch",
                "checker.invalidPropertyValue",
                "scss.nativeCssFunctionParameter",
                "scss.nativeCssFunctionReturn",
            ]
        );
        for policy in CSS_VALUE_VALIDATION_CONSUMER_POLICIES_V0 {
            assert_eq!(policy.matched, "accept");
            assert!(matches!(policy.unmatched, "diagnostic" | "reject"));
            assert!(matches!(policy.grammar_defect, "silent" | "unknown"));
            assert!(matches!(policy.budget_exhausted, "silent" | "unknown"));
        }
    }
}
