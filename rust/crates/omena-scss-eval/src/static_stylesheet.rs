use std::collections::{BTreeMap, BTreeSet};

use omena_abstract_value::{AbstractCssValueV0, abstract_css_value_from_text};
use omena_parser::{LexedToken, ParsedVariableFact, ParsedVariableFactKind, StyleDialect, lex};
use omena_syntax::SyntaxKind;
use omena_value_lattice::number::reduce_static_numeric_expression;
use serde::Serialize;

use crate::{abstract_css_value_kind, summarize_omena_scss_eval_oracle};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalStaticStylesheetEvaluationV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub evaluator: &'static str,
    pub dialect: &'static str,
    pub evaluated_css: String,
    pub replacement_count: usize,
    pub resolved_replacements: Vec<OmenaScssEvalResolvedReplacementV0>,
    pub value_resolution: OmenaScssEvalStaticValueResolutionReportV0,
    pub oracle: crate::OmenaScssEvalOracleReportV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalResolvedReplacementV0 {
    pub name: String,
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub abstract_value: AbstractCssValueV0,
    pub abstract_value_kind: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalStaticValueResolutionReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub mode: &'static str,
    pub dialect: &'static str,
    pub fuel_limit: usize,
    pub reference_count: usize,
    pub resolved_count: usize,
    pub raw_count: usize,
    pub top_count: usize,
    pub cycle_count: usize,
    pub fuel_exhausted_count: usize,
    pub unresolved_reference_count: usize,
    pub unsupported_dynamic_count: usize,
    pub values: Vec<OmenaScssEvalStaticValueResolutionV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalStaticValueResolutionV0 {
    pub name: String,
    pub start: usize,
    pub end: usize,
    pub source_text: String,
    pub rendered_value: Option<String>,
    pub abstract_value: AbstractCssValueV0,
    pub abstract_value_kind: &'static str,
    pub outcome: &'static str,
    pub reason: &'static str,
}

const STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT: usize = 128;

pub fn derive_static_stylesheet_module_evaluation(
    style_source: &str,
    dialect: StyleDialect,
) -> Option<OmenaScssEvalStaticStylesheetEvaluationV0> {
    let variable_kind = StaticStylesheetVariableKind::for_dialect(dialect)?;
    let facts = omena_parser::collect_style_facts(style_source, dialect);
    let variable_facts = facts.variables.as_slice();
    match variable_kind {
        StaticStylesheetVariableKind::Scss => {
            derive_static_scss_stylesheet_module_evaluation(style_source, dialect, variable_facts)
        }
        StaticStylesheetVariableKind::Less => {
            derive_static_less_stylesheet_module_evaluation(style_source, variable_facts)
        }
    }
}

pub fn summarize_static_stylesheet_value_resolution(
    style_source: &str,
    dialect: StyleDialect,
) -> Option<OmenaScssEvalStaticValueResolutionReportV0> {
    let variable_kind = StaticStylesheetVariableKind::for_dialect(dialect)?;
    let facts = omena_parser::collect_style_facts(style_source, dialect);
    let scopes = collect_static_stylesheet_scopes(style_source)?;
    let values = match variable_kind {
        StaticStylesheetVariableKind::Scss => {
            summarize_static_scss_value_resolution_values(style_source, &facts.variables, &scopes)?
        }
        StaticStylesheetVariableKind::Less => {
            summarize_static_less_value_resolution_values(style_source, &facts.variables, &scopes)?
        }
    };
    Some(build_static_value_resolution_report(
        dialect_label(dialect),
        values,
    ))
}

pub fn derive_static_scss_stylesheet_module_variable_exports(
    style_source: &str,
) -> BTreeMap<String, String> {
    let facts = omena_parser::collect_style_facts(style_source, StyleDialect::Scss);
    let scopes = match collect_static_stylesheet_scopes(style_source) {
        Some(scopes) => scopes,
        None => return BTreeMap::new(),
    };
    let declarations =
        match collect_static_scss_variable_declarations(style_source, &facts.variables, &scopes) {
            Some(declarations) => declarations,
            None => return BTreeMap::new(),
        };

    let mut exports = BTreeMap::new();
    for declaration in declarations
        .iter()
        .filter(|declaration| declaration.scope_id == 0)
    {
        let Some(public_name) = static_scss_public_module_variable_name(declaration.name.as_str())
        else {
            continue;
        };
        let mut stack = BTreeSet::new();
        if let Some(value) = resolve_static_scss_variable_value_in_scope(
            declaration.name.as_str(),
            0,
            usize::MAX,
            &scopes,
            &declarations,
            &mut stack,
        ) {
            exports.insert(public_name, value);
        }
    }
    exports
}

pub fn derive_static_scss_stylesheet_module_configurable_variable_names(
    style_source: &str,
) -> BTreeSet<String> {
    let facts = omena_parser::collect_style_facts(style_source, StyleDialect::Scss);
    let scopes = match collect_static_stylesheet_scopes(style_source) {
        Some(scopes) => scopes,
        None => return BTreeSet::new(),
    };
    let declarations =
        match collect_static_scss_variable_declarations(style_source, &facts.variables, &scopes) {
            Some(declarations) => declarations,
            None => return BTreeSet::new(),
        };

    declarations
        .iter()
        .filter(|declaration| declaration.scope_id == 0)
        .filter(|declaration| declaration.declaration.is_default)
        .filter_map(|declaration| {
            static_scss_public_module_variable_name(declaration.name.as_str())
        })
        .collect()
}

fn derive_static_scss_stylesheet_module_evaluation(
    style_source: &str,
    dialect: StyleDialect,
    variable_facts: &[ParsedVariableFact],
) -> Option<OmenaScssEvalStaticStylesheetEvaluationV0> {
    let lexed = lex(style_source, dialect);
    let tokens = lexed.tokens();
    let function_declarations = collect_static_scss_function_declarations(style_source, tokens)?;
    if !variable_facts
        .iter()
        .any(|fact| fact.kind == ParsedVariableFactKind::ScssDeclaration)
        && function_declarations.is_empty()
    {
        return None;
    }
    let scopes = collect_static_stylesheet_scopes(style_source)?;
    let declarations =
        collect_static_scss_variable_declarations(style_source, variable_facts, &scopes)?;
    let function_declaration_ranges =
        static_scss_function_declaration_ranges_from_declarations(function_declarations.as_slice());

    let mut edits = Vec::new();
    let mut resolved_replacements = Vec::new();
    for declaration in &declarations {
        for (start, end) in &declaration.removal_spans {
            edits.push(StaticStylesheetEvaluationEdit {
                start: *start,
                end: *end,
                replacement: String::new(),
            });
        }
    }
    for fact in variable_facts {
        if fact.kind != ParsedVariableFactKind::ScssReference {
            continue;
        }
        let reference_start = parser_text_size_to_usize(fact.range.start().into());
        if static_stylesheet_position_is_inside_scss_declaration(&declarations, reference_start)
            || static_stylesheet_position_is_inside_ranges(
                reference_start,
                &function_declaration_ranges,
            )
        {
            continue;
        }
        let mut stack = BTreeSet::new();
        let replacement = resolve_static_scss_variable_value_at_position(
            fact.name.as_str(),
            reference_start,
            &scopes,
            &declarations,
            &mut stack,
        )?;
        let reference_end = parser_text_size_to_usize(fact.range.end().into());
        resolved_replacements.push(resolved_replacement_value(
            fact.name.as_str(),
            reference_start,
            reference_end,
            replacement.as_str(),
        ));
        edits.push(StaticStylesheetEvaluationEdit {
            start: reference_start,
            end: reference_end,
            replacement,
        });
    }
    if let Some((function_edits, function_replacements)) =
        collect_static_scss_function_evaluation_edits(
            style_source,
            tokens,
            &function_declarations,
            &scopes,
            &declarations,
        )
    {
        edits.extend(function_edits);
        resolved_replacements.extend(function_replacements);
    }

    let evaluated_css = apply_static_stylesheet_evaluation_edits(style_source, edits)?;
    if evaluated_css == style_source {
        return None;
    }
    build_static_stylesheet_evaluation_report(
        style_source,
        dialect,
        StaticStylesheetVariableKind::Scss,
        evaluated_css,
        resolved_replacements,
    )
}

fn derive_static_less_stylesheet_module_evaluation(
    style_source: &str,
    variable_facts: &[ParsedVariableFact],
) -> Option<OmenaScssEvalStaticStylesheetEvaluationV0> {
    let scopes = collect_static_stylesheet_scopes(style_source)?;
    let lexed = lex(style_source, StyleDialect::Less);
    let tokens = lexed.tokens();
    let declarations =
        collect_static_less_variable_declarations(style_source, variable_facts, &scopes)?;
    let property_declarations =
        collect_static_less_property_declarations(style_source, tokens, &scopes)?;

    let mut edits = Vec::new();
    let mut resolved_replacements = Vec::new();
    for declaration in declarations.values() {
        for (start, end) in &declaration.removal_spans {
            edits.push(StaticStylesheetEvaluationEdit {
                start: *start,
                end: *end,
                replacement: String::new(),
            });
        }
    }
    for fact in variable_facts {
        if fact.kind != ParsedVariableFactKind::LessReference {
            continue;
        }
        let reference_start = parser_text_size_to_usize(fact.range.start().into());
        if static_stylesheet_position_is_inside_scoped_declaration(&declarations, reference_start) {
            continue;
        }
        let reference_scope_id = static_stylesheet_scope_for_position(&scopes, reference_start)?;
        let mut stack = BTreeSet::new();
        let replacement = resolve_static_less_variable_value_in_scope(
            fact.name.as_str(),
            reference_scope_id,
            &scopes,
            &declarations,
            &mut stack,
        )?;
        let reference_end = parser_text_size_to_usize(fact.range.end().into());
        resolved_replacements.push(resolved_replacement_value(
            fact.name.as_str(),
            reference_start,
            reference_end,
            replacement.as_str(),
        ));
        edits.push(StaticStylesheetEvaluationEdit {
            start: reference_start,
            end: reference_end,
            replacement,
        });
    }
    for token in tokens {
        if token.kind != SyntaxKind::LessPropertyVariableToken {
            continue;
        }
        let reference_start = static_stylesheet_token_start(token);
        if static_stylesheet_position_is_inside_scoped_declaration(&declarations, reference_start) {
            continue;
        }
        let reference_scope_id = static_stylesheet_scope_for_position(&scopes, reference_start)?;
        let mut stack = BTreeSet::new();
        let replacement = resolve_static_less_property_value_in_scope(
            token.text.as_str(),
            reference_scope_id,
            &scopes,
            &property_declarations,
            &mut stack,
        )?;
        resolved_replacements.push(resolved_replacement_value(
            token.text.as_str(),
            reference_start,
            static_stylesheet_token_end(token),
            replacement.as_str(),
        ));
        edits.push(StaticStylesheetEvaluationEdit {
            start: reference_start,
            end: static_stylesheet_token_end(token),
            replacement,
        });
    }

    let evaluated_css = apply_static_stylesheet_evaluation_edits(style_source, edits)?;
    if evaluated_css == style_source {
        return None;
    }
    build_static_stylesheet_evaluation_report(
        style_source,
        StyleDialect::Less,
        StaticStylesheetVariableKind::Less,
        evaluated_css,
        resolved_replacements,
    )
}

fn build_static_stylesheet_evaluation_report(
    style_source: &str,
    dialect: StyleDialect,
    variable_kind: StaticStylesheetVariableKind,
    evaluated_css: String,
    resolved_replacements: Vec<OmenaScssEvalResolvedReplacementV0>,
) -> Option<OmenaScssEvalStaticStylesheetEvaluationV0> {
    let value_resolution = summarize_static_stylesheet_value_resolution(style_source, dialect)?;
    let oracle = summarize_omena_scss_eval_oracle(style_source, dialect, evaluated_css.as_str());
    if !oracle.all_legacy_declaration_values_preserved {
        return None;
    }
    Some(OmenaScssEvalStaticStylesheetEvaluationV0 {
        schema_version: "0",
        product: "omena-scss-eval.static-stylesheet-evaluation",
        evaluator: variable_kind.evaluator_label(),
        dialect: dialect_label(dialect),
        replacement_count: resolved_replacements.len(),
        resolved_replacements,
        value_resolution,
        evaluated_css,
        oracle,
    })
}

fn resolved_replacement_value(
    name: &str,
    start: usize,
    end: usize,
    text: &str,
) -> OmenaScssEvalResolvedReplacementV0 {
    let abstract_value = abstract_css_value_from_text(text);
    OmenaScssEvalResolvedReplacementV0 {
        name: name.to_string(),
        start,
        end,
        text: text.to_string(),
        abstract_value_kind: abstract_css_value_kind(&abstract_value),
        abstract_value,
    }
}

fn build_static_value_resolution_report(
    dialect: &'static str,
    values: Vec<OmenaScssEvalStaticValueResolutionV0>,
) -> OmenaScssEvalStaticValueResolutionReportV0 {
    let resolved_count = values
        .iter()
        .filter(|value| value.outcome == "resolved")
        .count();
    let raw_count = values
        .iter()
        .filter(|value| matches!(value.abstract_value, AbstractCssValueV0::Raw { .. }))
        .count();
    let top_count = values
        .iter()
        .filter(|value| matches!(value.abstract_value, AbstractCssValueV0::Top))
        .count();
    let cycle_count = values
        .iter()
        .filter(|value| value.reason == "cycle")
        .count();
    let fuel_exhausted_count = values
        .iter()
        .filter(|value| value.reason == "fuelExhausted")
        .count();
    let unresolved_reference_count = values
        .iter()
        .filter(|value| value.reason == "unresolvedReference")
        .count();
    let unsupported_dynamic_count = values
        .iter()
        .filter(|value| value.reason == "unsupportedDynamic")
        .count();
    OmenaScssEvalStaticValueResolutionReportV0 {
        schema_version: "0",
        product: "omena-scss-eval.static-value-resolution",
        mode: "oracleOnly",
        dialect,
        fuel_limit: STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
        reference_count: values.len(),
        resolved_count,
        raw_count,
        top_count,
        cycle_count,
        fuel_exhausted_count,
        unresolved_reference_count,
        unsupported_dynamic_count,
        values,
    }
}

fn summarize_static_scss_value_resolution_values(
    style_source: &str,
    variable_facts: &[ParsedVariableFact],
    scopes: &[StaticStylesheetScope],
) -> Option<Vec<OmenaScssEvalStaticValueResolutionV0>> {
    let declarations =
        collect_static_scss_variable_declarations(style_source, variable_facts, scopes)?;
    let lexed = lex(style_source, StyleDialect::Scss);
    let tokens = lexed.tokens();
    let function_declarations = collect_static_scss_function_declarations(style_source, tokens)?;
    let function_declaration_ranges =
        static_scss_function_declaration_ranges_from_declarations(function_declarations.as_slice());
    let mut values = Vec::new();
    for fact in variable_facts {
        if fact.kind != ParsedVariableFactKind::ScssReference {
            continue;
        }
        let reference_start = parser_text_size_to_usize(fact.range.start().into());
        if static_stylesheet_position_is_inside_scss_declaration(&declarations, reference_start)
            || static_stylesheet_position_is_inside_ranges(
                reference_start,
                &function_declaration_ranges,
            )
        {
            continue;
        }
        let reference_end = parser_text_size_to_usize(fact.range.end().into());
        let mut stack = BTreeSet::new();
        let resolution = resolve_static_scss_variable_abstract_value_at_position(
            fact.name.as_str(),
            reference_start,
            scopes,
            &declarations,
            &mut stack,
            STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
        );
        values.push(static_value_resolution_record(
            fact.name.as_str(),
            reference_start,
            reference_end,
            style_source
                .get(reference_start..reference_end)
                .unwrap_or(""),
            resolution,
        ));
    }
    values.extend(collect_static_scss_function_value_resolution_values(
        style_source,
        tokens,
        &function_declarations,
        scopes,
        &declarations,
    )?);
    Some(values)
}

fn summarize_static_less_value_resolution_values(
    style_source: &str,
    variable_facts: &[ParsedVariableFact],
    scopes: &[StaticStylesheetScope],
) -> Option<Vec<OmenaScssEvalStaticValueResolutionV0>> {
    let lexed = lex(style_source, StyleDialect::Less);
    let tokens = lexed.tokens();
    let declarations =
        collect_static_less_variable_declarations(style_source, variable_facts, scopes)?;
    let property_declarations =
        collect_static_less_property_declarations(style_source, tokens, scopes)?;
    let mut values = Vec::new();
    for fact in variable_facts {
        if fact.kind != ParsedVariableFactKind::LessReference {
            continue;
        }
        let reference_start = parser_text_size_to_usize(fact.range.start().into());
        if static_stylesheet_position_is_inside_scoped_declaration(&declarations, reference_start) {
            continue;
        }
        let reference_scope_id = static_stylesheet_scope_for_position(scopes, reference_start)?;
        let reference_end = parser_text_size_to_usize(fact.range.end().into());
        let mut stack = BTreeSet::new();
        let resolution = resolve_static_less_variable_abstract_value_in_scope(
            fact.name.as_str(),
            reference_scope_id,
            scopes,
            &declarations,
            &mut stack,
            STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
        );
        values.push(static_value_resolution_record(
            fact.name.as_str(),
            reference_start,
            reference_end,
            style_source
                .get(reference_start..reference_end)
                .unwrap_or(""),
            resolution,
        ));
    }
    for token in tokens {
        if token.kind != SyntaxKind::LessPropertyVariableToken {
            continue;
        }
        let reference_start = static_stylesheet_token_start(token);
        if static_stylesheet_position_is_inside_scoped_declaration(&declarations, reference_start) {
            continue;
        }
        let reference_scope_id = static_stylesheet_scope_for_position(scopes, reference_start)?;
        let mut stack = BTreeSet::new();
        let resolution = resolve_static_less_property_abstract_value_in_scope(
            token.text.as_str(),
            reference_scope_id,
            scopes,
            &property_declarations,
            &mut stack,
            STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
        );
        values.push(static_value_resolution_record(
            token.text.as_str(),
            reference_start,
            static_stylesheet_token_end(token),
            token.text.as_str(),
            resolution,
        ));
    }
    Some(values)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticStylesheetResolutionOutcome {
    Resolved,
    Raw,
    Top,
}

impl StaticStylesheetResolutionOutcome {
    fn label(self) -> &'static str {
        match self {
            Self::Resolved => "resolved",
            Self::Raw => "raw",
            Self::Top => "top",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticStylesheetResolutionReason {
    Resolved,
    Cycle,
    FuelExhausted,
    UnresolvedReference,
    UnsupportedDynamic,
}

impl StaticStylesheetResolutionReason {
    fn label(self) -> &'static str {
        match self {
            Self::Resolved => "resolved",
            Self::Cycle => "cycle",
            Self::FuelExhausted => "fuelExhausted",
            Self::UnresolvedReference => "unresolvedReference",
            Self::UnsupportedDynamic => "unsupportedDynamic",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StaticStylesheetAbstractResolution {
    rendered_value: Option<String>,
    abstract_value: AbstractCssValueV0,
    outcome: StaticStylesheetResolutionOutcome,
    reason: StaticStylesheetResolutionReason,
}

fn static_value_resolution_record(
    name: &str,
    start: usize,
    end: usize,
    source_text: &str,
    resolution: StaticStylesheetAbstractResolution,
) -> OmenaScssEvalStaticValueResolutionV0 {
    OmenaScssEvalStaticValueResolutionV0 {
        name: name.to_string(),
        start,
        end,
        source_text: source_text.to_string(),
        rendered_value: resolution.rendered_value,
        abstract_value_kind: abstract_css_value_kind(&resolution.abstract_value),
        abstract_value: resolution.abstract_value,
        outcome: resolution.outcome.label(),
        reason: resolution.reason.label(),
    }
}

fn resolved_static_abstract_value(text: &str) -> StaticStylesheetAbstractResolution {
    let abstract_value = abstract_css_value_from_text(text);
    let rendered_value = render_static_abstract_value(&abstract_value);
    let outcome = if matches!(abstract_value, AbstractCssValueV0::Raw { .. }) {
        StaticStylesheetResolutionOutcome::Raw
    } else {
        StaticStylesheetResolutionOutcome::Resolved
    };
    let reason = if outcome == StaticStylesheetResolutionOutcome::Raw {
        StaticStylesheetResolutionReason::UnsupportedDynamic
    } else {
        StaticStylesheetResolutionReason::Resolved
    };
    StaticStylesheetAbstractResolution {
        rendered_value,
        abstract_value,
        outcome,
        reason,
    }
}

fn raw_static_abstract_value(
    text: &str,
    reason: StaticStylesheetResolutionReason,
) -> StaticStylesheetAbstractResolution {
    StaticStylesheetAbstractResolution {
        rendered_value: Some(text.to_string()),
        abstract_value: AbstractCssValueV0::Raw {
            value: text.to_string(),
        },
        outcome: StaticStylesheetResolutionOutcome::Raw,
        reason,
    }
}

fn top_static_abstract_value(
    reason: StaticStylesheetResolutionReason,
) -> StaticStylesheetAbstractResolution {
    StaticStylesheetAbstractResolution {
        rendered_value: None,
        abstract_value: AbstractCssValueV0::Top,
        outcome: StaticStylesheetResolutionOutcome::Top,
        reason,
    }
}

fn render_static_abstract_value(value: &AbstractCssValueV0) -> Option<String> {
    match value {
        AbstractCssValueV0::Bottom => Some(String::new()),
        AbstractCssValueV0::Exact { value } | AbstractCssValueV0::Raw { value } => {
            Some(value.clone())
        }
        AbstractCssValueV0::FiniteSet { values } => values.first().cloned(),
        AbstractCssValueV0::Top => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticStylesheetVariableKind {
    Scss,
    Less,
}

impl StaticStylesheetVariableKind {
    fn for_dialect(dialect: StyleDialect) -> Option<Self> {
        match dialect {
            StyleDialect::Scss | StyleDialect::Sass => Some(Self::Scss),
            StyleDialect::Less => Some(Self::Less),
            StyleDialect::Css => None,
        }
    }

    fn evaluator_label(self) -> &'static str {
        match self {
            Self::Scss => "omena-query-static-scss-variable-evaluator",
            Self::Less => "omena-query-static-less-variable-evaluator",
        }
    }

    fn reference_prefix(self) -> char {
        match self {
            Self::Scss => '$',
            Self::Less => '@',
        }
    }
}

#[derive(Debug, Clone)]
struct StaticStylesheetVariableDeclaration {
    value: String,
    span_start: usize,
    span_end: usize,
    removal_spans: Vec<(usize, usize)>,
    is_default: bool,
    is_global: bool,
}

#[derive(Debug, Clone)]
struct StaticStylesheetScopedVariableDeclaration {
    name: String,
    scope_id: usize,
    declaration: StaticStylesheetVariableDeclaration,
    removal_spans: Vec<(usize, usize)>,
}

#[derive(Debug, Clone)]
struct StaticStylesheetEvaluationEdit {
    start: usize,
    end: usize,
    replacement: String,
}

#[derive(Debug, Clone)]
struct StaticStylesheetPropertyDeclaration {
    value: String,
}

#[derive(Debug, Clone)]
struct StaticScssFunctionDeclaration {
    name: String,
    parameters: Vec<String>,
    return_value: String,
    span_start: usize,
    span_end: usize,
    body_start: usize,
    body_end: usize,
}

#[derive(Debug, Clone)]
struct StaticScssFunctionCall {
    name: String,
    start: usize,
    end: usize,
    arguments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StaticStylesheetScope {
    parent_id: Option<usize>,
    body_start: usize,
    end: usize,
}

fn collect_static_scss_function_evaluation_edits(
    source: &str,
    tokens: &[LexedToken],
    declarations: &[StaticScssFunctionDeclaration],
    scopes: &[StaticStylesheetScope],
    variable_declarations: &[StaticStylesheetScopedVariableDeclaration],
) -> Option<(
    Vec<StaticStylesheetEvaluationEdit>,
    Vec<OmenaScssEvalResolvedReplacementV0>,
)> {
    let calls = collect_static_scss_function_calls(source, tokens, declarations)?;
    if calls.is_empty() {
        return Some((Vec::new(), Vec::new()));
    }

    let mut edits = Vec::new();
    let mut replacements = Vec::new();
    let mut used_declaration_names = BTreeSet::new();
    for call in &calls {
        let resolution = resolve_static_scss_function_call_abstract_value(
            call,
            declarations,
            scopes,
            variable_declarations,
            STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
        );
        if resolution.outcome != StaticStylesheetResolutionOutcome::Resolved {
            return None;
        }
        let rendered_value = resolution.rendered_value?;
        used_declaration_names.insert(canonical_static_scss_function_name(call.name.as_str()));
        replacements.push(resolved_replacement_value(
            format!("function:{}", call.name).as_str(),
            call.start,
            call.end,
            rendered_value.as_str(),
        ));
        edits.push(StaticStylesheetEvaluationEdit {
            start: call.start,
            end: call.end,
            replacement: rendered_value,
        });
    }

    for declaration in declarations.iter().filter(|declaration| {
        used_declaration_names.contains(&canonical_static_scss_function_name(
            declaration.name.as_str(),
        ))
    }) {
        edits.push(StaticStylesheetEvaluationEdit {
            start: declaration.span_start,
            end: declaration.span_end,
            replacement: String::new(),
        });
    }

    Some((edits, replacements))
}

fn collect_static_scss_function_value_resolution_values(
    source: &str,
    tokens: &[LexedToken],
    declarations: &[StaticScssFunctionDeclaration],
    scopes: &[StaticStylesheetScope],
    variable_declarations: &[StaticStylesheetScopedVariableDeclaration],
) -> Option<Vec<OmenaScssEvalStaticValueResolutionV0>> {
    let calls = collect_static_scss_function_calls(source, tokens, declarations)?;
    let values = calls
        .into_iter()
        .map(|call| {
            let resolution = resolve_static_scss_function_call_abstract_value(
                &call,
                declarations,
                scopes,
                variable_declarations,
                STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
            );
            static_value_resolution_record(
                format!("function:{}", call.name).as_str(),
                call.start,
                call.end,
                source.get(call.start..call.end).unwrap_or(""),
                resolution,
            )
        })
        .collect();
    Some(values)
}

fn collect_static_scss_function_declarations(
    source: &str,
    tokens: &[LexedToken],
) -> Option<Vec<StaticScssFunctionDeclaration>> {
    let mut declarations = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        if tokens[index].kind != SyntaxKind::AtKeyword
            || !tokens[index].text.eq_ignore_ascii_case("@function")
        {
            index += 1;
            continue;
        }

        let name_index = static_stylesheet_skip_trivia_tokens(tokens, index + 1);
        let name_token = tokens.get(name_index)?;
        if name_token.kind != SyntaxKind::Ident
            || !static_scss_callable_name_is_safe(name_token.text.as_str())
        {
            index += 1;
            continue;
        }

        let parameter_open_index = static_stylesheet_skip_trivia_tokens(tokens, name_index + 1);
        if tokens
            .get(parameter_open_index)
            .is_none_or(|token| token.kind != SyntaxKind::LeftParen)
        {
            index += 1;
            continue;
        }
        let parameter_close_index = static_stylesheet_matching_token_index(
            tokens,
            parameter_open_index,
            SyntaxKind::LeftParen,
            SyntaxKind::RightParen,
        )?;
        let parameters = collect_static_scss_function_parameters(
            tokens,
            parameter_open_index + 1,
            parameter_close_index,
        )?;

        let body_open_index =
            static_stylesheet_skip_trivia_tokens(tokens, parameter_close_index + 1);
        if tokens
            .get(body_open_index)
            .is_none_or(|token| token.kind != SyntaxKind::LeftBrace)
        {
            index += 1;
            continue;
        }
        let body_close_index = static_stylesheet_matching_token_index(
            tokens,
            body_open_index,
            SyntaxKind::LeftBrace,
            SyntaxKind::RightBrace,
        )?;
        let return_value = collect_static_scss_function_return_value(
            source,
            tokens,
            body_open_index + 1,
            body_close_index,
        )?;
        if !static_stylesheet_composite_value_is_safe(return_value.as_str()) {
            index = body_close_index + 1;
            continue;
        }

        declarations.push(StaticScssFunctionDeclaration {
            name: name_token.text.clone(),
            parameters,
            return_value,
            span_start: static_stylesheet_token_start(&tokens[index]),
            span_end: static_stylesheet_token_end(&tokens[body_close_index]),
            body_start: static_stylesheet_token_end(&tokens[body_open_index]),
            body_end: static_stylesheet_token_start(&tokens[body_close_index]),
        });
        index = body_close_index + 1;
    }
    Some(declarations)
}

fn collect_static_scss_function_parameters(
    tokens: &[LexedToken],
    start: usize,
    end: usize,
) -> Option<Vec<String>> {
    let mut parameters = Vec::new();
    let mut expect_parameter = true;
    let mut saw_token = false;
    for token in &tokens[start..end] {
        if static_stylesheet_token_is_trivia(token.kind) {
            continue;
        }
        saw_token = true;
        if expect_parameter {
            if token.kind != SyntaxKind::ScssVariable {
                return None;
            }
            parameters.push(canonical_static_scss_variable_name(token.text.as_str()));
            expect_parameter = false;
            continue;
        }
        if token.kind != SyntaxKind::Comma {
            return None;
        }
        expect_parameter = true;
    }
    if saw_token && expect_parameter {
        return None;
    }
    Some(parameters)
}

fn collect_static_scss_function_return_value(
    source: &str,
    tokens: &[LexedToken],
    start: usize,
    end: usize,
) -> Option<String> {
    let mut values = Vec::new();
    let mut index = start;
    while index < end {
        if tokens[index].kind != SyntaxKind::AtKeyword
            || !tokens[index].text.eq_ignore_ascii_case("@return")
        {
            index += 1;
            continue;
        }
        let value_end_index = static_stylesheet_value_end_token_until(tokens, index + 1, end)?;
        let value_start = static_stylesheet_token_end(&tokens[index]);
        let value_end = static_stylesheet_token_start(&tokens[value_end_index]);
        let value = source.get(value_start..value_end)?.trim();
        if value.is_empty() {
            return None;
        }
        values.push(value.to_string());
        index = value_end_index + 1;
    }
    (values.len() == 1).then(|| values.remove(0))
}

fn collect_static_scss_function_calls(
    source: &str,
    tokens: &[LexedToken],
    declarations: &[StaticScssFunctionDeclaration],
) -> Option<Vec<StaticScssFunctionCall>> {
    let declaration_names = declarations
        .iter()
        .map(|declaration| canonical_static_scss_function_name(declaration.name.as_str()))
        .collect::<BTreeSet<_>>();
    let mut calls = Vec::new();
    for (name_index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::Ident
            || !declaration_names
                .contains(&canonical_static_scss_function_name(token.text.as_str()))
            || static_scss_function_position_is_inside_declaration_header(
                declarations,
                static_stylesheet_token_start(token),
            )
        {
            continue;
        }
        let open_index = static_stylesheet_skip_trivia_tokens(tokens, name_index + 1);
        if tokens
            .get(open_index)
            .is_none_or(|token| token.kind != SyntaxKind::LeftParen)
        {
            continue;
        }
        let close_index = static_stylesheet_matching_token_index(
            tokens,
            open_index,
            SyntaxKind::LeftParen,
            SyntaxKind::RightParen,
        )?;
        let arguments = split_static_scss_function_arguments(source.get(
            static_stylesheet_token_end(&tokens[open_index])
                ..static_stylesheet_token_start(&tokens[close_index]),
        )?)?;
        calls.push(StaticScssFunctionCall {
            name: token.text.clone(),
            start: static_stylesheet_token_start(token),
            end: static_stylesheet_token_end(&tokens[close_index]),
            arguments,
        });
    }
    calls.sort_by_key(|call| (call.start, call.end));
    Some(calls)
}

fn static_scss_function_position_is_inside_declaration_header(
    declarations: &[StaticScssFunctionDeclaration],
    position: usize,
) -> bool {
    declarations
        .iter()
        .any(|declaration| position >= declaration.span_start && position < declaration.body_start)
}

fn split_static_scss_function_arguments(arguments: &str) -> Option<Vec<String>> {
    let arguments = arguments.trim();
    if arguments.is_empty() {
        return Some(Vec::new());
    }

    let mut values = Vec::new();
    let mut cursor = 0usize;
    let mut index = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;
    while index < arguments.len() {
        let ch = arguments[index..].chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = arguments[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }
        if matches!(ch, '"' | '\'') {
            quote = Some(ch);
            index += ch.len_utf8();
            continue;
        }
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.checked_sub(1)?,
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.checked_sub(1)?,
            ',' if paren_depth == 0 && bracket_depth == 0 => {
                let value = arguments.get(cursor..index)?.trim();
                if !static_scss_function_argument_is_safe(value) {
                    return None;
                }
                values.push(value.to_string());
                cursor = index + ch.len_utf8();
            }
            _ => {}
        }
        index += ch.len_utf8();
    }

    if quote.is_some() || paren_depth != 0 || bracket_depth != 0 {
        return None;
    }
    let value = arguments.get(cursor..)?.trim();
    if !static_scss_function_argument_is_safe(value) {
        return None;
    }
    values.push(value.to_string());
    Some(values)
}

fn resolve_static_scss_function_call_abstract_value(
    call: &StaticScssFunctionCall,
    declarations: &[StaticScssFunctionDeclaration],
    scopes: &[StaticStylesheetScope],
    variable_declarations: &[StaticStylesheetScopedVariableDeclaration],
    fuel: usize,
) -> StaticStylesheetAbstractResolution {
    if fuel == 0 {
        return top_static_abstract_value(StaticStylesheetResolutionReason::FuelExhausted);
    }
    let Some(declaration) = declarations.iter().find(|declaration| {
        canonical_static_scss_function_name(declaration.name.as_str())
            == canonical_static_scss_function_name(call.name.as_str())
    }) else {
        return top_static_abstract_value(StaticStylesheetResolutionReason::UnresolvedReference);
    };
    if call.start >= declaration.body_start && call.start < declaration.body_end {
        return top_static_abstract_value(StaticStylesheetResolutionReason::Cycle);
    }
    if declaration.parameters.len() != call.arguments.len() {
        return raw_static_abstract_value(
            call.name.as_str(),
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        );
    }
    if static_scss_function_value_contains_callable_to(
        declaration.return_value.as_str(),
        declaration.name.as_str(),
    ) {
        return top_static_abstract_value(StaticStylesheetResolutionReason::Cycle);
    }

    let mut argument_values = BTreeMap::new();
    for (parameter, argument) in declaration.parameters.iter().zip(&call.arguments) {
        let resolution = resolve_static_scss_function_argument_abstract_value(
            argument.as_str(),
            call.start,
            scopes,
            variable_declarations,
            fuel - 1,
        );
        let Some(rendered_value) = resolution.rendered_value else {
            return top_static_abstract_value(resolution.reason);
        };
        if resolution.outcome == StaticStylesheetResolutionOutcome::Top {
            return top_static_abstract_value(resolution.reason);
        }
        argument_values.insert(parameter.clone(), rendered_value);
    }
    resolve_static_scss_function_return_abstract_value(
        declaration,
        &argument_values,
        scopes,
        variable_declarations,
        fuel - 1,
    )
}

fn resolve_static_scss_function_argument_abstract_value(
    argument: &str,
    call_position: usize,
    scopes: &[StaticStylesheetScope],
    variable_declarations: &[StaticStylesheetScopedVariableDeclaration],
    fuel: usize,
) -> StaticStylesheetAbstractResolution {
    let mut abstract_stack = BTreeSet::new();
    let mut resolution = resolve_static_scss_variable_abstract_value_text(
        argument,
        call_position,
        scopes,
        variable_declarations,
        &mut abstract_stack,
        fuel,
    );
    if resolution.outcome == StaticStylesheetResolutionOutcome::Top {
        return resolution;
    }
    let mut text_stack = BTreeSet::new();
    if let Some(rendered_value) = resolve_static_scss_variable_value_text(
        argument,
        call_position,
        scopes,
        variable_declarations,
        &mut text_stack,
    ) {
        resolution.rendered_value = Some(rendered_value);
    }
    resolution
}

fn resolve_static_scss_function_return_abstract_value(
    declaration: &StaticScssFunctionDeclaration,
    argument_values: &BTreeMap<String, String>,
    scopes: &[StaticStylesheetScope],
    variable_declarations: &[StaticStylesheetScopedVariableDeclaration],
    fuel: usize,
) -> StaticStylesheetAbstractResolution {
    let Some(references) = collect_static_stylesheet_variable_references(
        declaration.return_value.as_str(),
        StaticStylesheetVariableKind::Scss,
    ) else {
        return raw_static_abstract_value(
            declaration.return_value.as_str(),
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        );
    };
    if references.is_empty() {
        return evaluate_static_scss_function_output_value(declaration.return_value.as_str());
    }

    let mut output = String::with_capacity(declaration.return_value.len());
    let mut cursor = 0usize;
    let mut stack = BTreeSet::new();
    for reference in references {
        let canonical_name = canonical_static_scss_variable_name(reference.name.as_str());
        let resolved = if let Some(argument_value) = argument_values.get(&canonical_name) {
            evaluate_static_scss_function_output_value(argument_value.as_str())
        } else {
            resolve_static_scss_variable_abstract_value_at_position(
                reference.name.as_str(),
                declaration.span_start,
                scopes,
                variable_declarations,
                &mut stack,
                fuel,
            )
        };
        let Some(rendered_value) = resolved.rendered_value else {
            return top_static_abstract_value(resolved.reason);
        };
        output.push_str(&declaration.return_value[cursor..reference.start]);
        output.push_str(&rendered_value);
        cursor = reference.end;
    }
    output.push_str(&declaration.return_value[cursor..]);
    evaluate_static_scss_function_output_value(output.as_str())
}

fn evaluate_static_scss_function_output_value(value: &str) -> StaticStylesheetAbstractResolution {
    if !static_stylesheet_composite_value_is_safe(value) {
        return raw_static_abstract_value(
            value,
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        );
    }
    if static_scss_function_value_contains_any_callable(value) {
        return raw_static_abstract_value(
            value,
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        );
    }
    let abstract_value = abstract_css_value_from_text(value);
    let outcome = if matches!(abstract_value, AbstractCssValueV0::Raw { .. }) {
        StaticStylesheetResolutionOutcome::Raw
    } else {
        StaticStylesheetResolutionOutcome::Resolved
    };
    let reason = if outcome == StaticStylesheetResolutionOutcome::Raw {
        StaticStylesheetResolutionReason::UnsupportedDynamic
    } else {
        StaticStylesheetResolutionReason::Resolved
    };
    StaticStylesheetAbstractResolution {
        rendered_value: Some(value.to_string()),
        abstract_value,
        outcome,
        reason,
    }
}

fn static_scss_function_value_contains_any_callable(value: &str) -> bool {
    let lexed = lex(value, StyleDialect::Scss);
    let tokens = lexed.tokens();
    tokens.iter().enumerate().any(|(index, token)| {
        token.kind == SyntaxKind::Ident
            && tokens
                .get(static_stylesheet_skip_trivia_tokens(tokens, index + 1))
                .is_some_and(|candidate| candidate.kind == SyntaxKind::LeftParen)
    })
}

fn static_scss_function_value_contains_callable_to(value: &str, name: &str) -> bool {
    let canonical_name = canonical_static_scss_function_name(name);
    let lexed = lex(value, StyleDialect::Scss);
    let tokens = lexed.tokens();
    tokens.iter().enumerate().any(|(index, token)| {
        token.kind == SyntaxKind::Ident
            && canonical_static_scss_function_name(token.text.as_str()) == canonical_name
            && tokens
                .get(static_stylesheet_skip_trivia_tokens(tokens, index + 1))
                .is_some_and(|candidate| candidate.kind == SyntaxKind::LeftParen)
    })
}

fn static_stylesheet_matching_token_index(
    tokens: &[LexedToken],
    start: usize,
    left: SyntaxKind,
    right: SyntaxKind,
) -> Option<usize> {
    if tokens.get(start)?.kind != left {
        return None;
    }
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(start) {
        if token.kind == left {
            depth += 1;
        } else if token.kind == right {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

fn collect_static_scss_variable_declarations(
    source: &str,
    variable_facts: &[ParsedVariableFact],
    scopes: &[StaticStylesheetScope],
) -> Option<Vec<StaticStylesheetScopedVariableDeclaration>> {
    let mut declarations = Vec::new();
    let module_rule_ranges = collect_static_scss_module_rule_ranges(source);
    let function_declaration_ranges = collect_static_scss_function_declaration_ranges(source);
    for fact in variable_facts {
        if fact.kind != ParsedVariableFactKind::ScssDeclaration {
            continue;
        }
        let start = parser_text_size_to_usize(fact.range.start().into());
        let end = parser_text_size_to_usize(fact.range.end().into());
        if static_stylesheet_position_is_inside_ranges(start, &module_rule_ranges)
            || static_stylesheet_position_is_inside_ranges(start, &function_declaration_ranges)
        {
            continue;
        }
        let scope_id = static_stylesheet_scope_for_position(scopes, start)?;
        let declaration = extract_static_stylesheet_variable_declaration(
            source,
            start,
            end,
            StaticStylesheetVariableKind::Scss,
        )?;
        if !static_stylesheet_scss_declaration_value_is_removal_safe(&declaration.value) {
            return None;
        }
        declarations.push(StaticStylesheetScopedVariableDeclaration {
            name: fact.name.clone(),
            scope_id: if declaration.is_global { 0 } else { scope_id },
            removal_spans: declaration.removal_spans.clone(),
            declaration,
        });
    }
    declarations.sort_by_key(|declaration| declaration.declaration.span_start);
    Some(declarations)
}

fn collect_static_scss_function_declaration_ranges(source: &str) -> Vec<(usize, usize)> {
    let lexed = lex(source, StyleDialect::Scss);
    let tokens = lexed.tokens();
    let mut ranges = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        if tokens[index].kind != SyntaxKind::AtKeyword
            || !tokens[index].text.eq_ignore_ascii_case("@function")
        {
            index += 1;
            continue;
        }
        let Some(body_open_index) =
            static_stylesheet_next_token_kind_index(tokens, index + 1, SyntaxKind::LeftBrace)
        else {
            index += 1;
            continue;
        };
        let Some(body_close_index) = static_stylesheet_matching_token_index(
            tokens,
            body_open_index,
            SyntaxKind::LeftBrace,
            SyntaxKind::RightBrace,
        ) else {
            index += 1;
            continue;
        };
        ranges.push((
            static_stylesheet_token_start(&tokens[index]),
            static_stylesheet_token_end(&tokens[body_close_index]),
        ));
        index = body_close_index + 1;
    }
    ranges
}

fn static_scss_function_declaration_ranges_from_declarations(
    declarations: &[StaticScssFunctionDeclaration],
) -> Vec<(usize, usize)> {
    declarations
        .iter()
        .map(|declaration| (declaration.span_start, declaration.span_end))
        .collect()
}

fn collect_static_scss_module_rule_ranges(source: &str) -> Vec<(usize, usize)> {
    let lexed = lex(source, StyleDialect::Scss);
    let tokens = lexed.tokens();
    let mut ranges = Vec::new();
    let mut depth = 0usize;
    let mut index = 0usize;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
            SyntaxKind::AtKeyword
                if depth == 0
                    && matches!(
                        tokens[index].text.to_ascii_lowercase().as_str(),
                        "@use" | "@forward"
                    ) =>
            {
                let Some(end_index) = static_stylesheet_scss_module_rule_semicolon(tokens, index)
                else {
                    index += 1;
                    continue;
                };
                ranges.push((
                    static_stylesheet_token_start(&tokens[index]),
                    static_stylesheet_token_end(&tokens[end_index]),
                ));
                index = end_index + 1;
                continue;
            }
            _ => {}
        }
        index += 1;
    }

    ranges
}

fn static_stylesheet_next_token_kind_index(
    tokens: &[LexedToken],
    mut index: usize,
    kind: SyntaxKind,
) -> Option<usize> {
    while index < tokens.len() {
        match tokens[index].kind {
            candidate if candidate == kind => return Some(index),
            SyntaxKind::Semicolon | SyntaxKind::RightBrace => return None,
            _ => index += 1,
        }
    }
    None
}

fn static_stylesheet_scss_module_rule_semicolon(
    tokens: &[LexedToken],
    at_rule_index: usize,
) -> Option<usize> {
    let mut index = at_rule_index + 1;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::Semicolon => return Some(index),
            SyntaxKind::LeftBrace | SyntaxKind::RightBrace => return None,
            _ => index += 1,
        }
    }
    None
}

fn static_stylesheet_position_is_inside_ranges(position: usize, ranges: &[(usize, usize)]) -> bool {
    ranges
        .iter()
        .any(|(start, end)| *start <= position && position < *end)
}

fn collect_static_less_variable_declarations(
    source: &str,
    variable_facts: &[ParsedVariableFact],
    scopes: &[StaticStylesheetScope],
) -> Option<BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>> {
    let mut declarations = BTreeMap::<(usize, String), StaticStylesheetVariableDeclaration>::new();
    for fact in variable_facts {
        if fact.kind != ParsedVariableFactKind::LessDeclaration {
            continue;
        }
        let start = parser_text_size_to_usize(fact.range.start().into());
        let end = parser_text_size_to_usize(fact.range.end().into());
        let scope_id = static_stylesheet_scope_for_position(scopes, start)?;
        let declaration = extract_static_stylesheet_variable_declaration(
            source,
            start,
            end,
            StaticStylesheetVariableKind::Less,
        )?;
        if !static_stylesheet_less_declaration_value_is_removal_safe(&declaration.value) {
            return None;
        }
        let key = (scope_id, fact.name.clone());
        if let Some(previous) = declarations.get_mut(&key) {
            merge_static_stylesheet_duplicate_declaration(
                previous,
                declaration,
                StaticStylesheetVariableKind::Less,
            )?;
            continue;
        }
        declarations.insert(key, declaration);
    }
    Some(declarations)
}

fn collect_static_less_property_declarations(
    source: &str,
    tokens: &[LexedToken],
    scopes: &[StaticStylesheetScope],
) -> Option<BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>> {
    let mut declarations = BTreeMap::<(usize, String), StaticStylesheetPropertyDeclaration>::new();
    let mut index = 0usize;
    while index < tokens.len() {
        if !matches!(
            tokens[index].kind,
            SyntaxKind::Ident | SyntaxKind::CustomPropertyName
        ) || !static_stylesheet_property_name_is_safe(tokens[index].text.as_str())
            || !static_stylesheet_previous_token_starts_declaration(tokens, index)
        {
            index += 1;
            continue;
        }

        let colon_index = static_stylesheet_skip_trivia_tokens(tokens, index + 1);
        if tokens
            .get(colon_index)
            .is_none_or(|token| token.kind != SyntaxKind::Colon)
        {
            index += 1;
            continue;
        }

        let value_start_index = colon_index + 1;
        let value_end_index =
            static_stylesheet_declaration_value_end_token(tokens, value_start_index)?;
        let value_start = static_stylesheet_token_end(&tokens[colon_index]);
        let value_end = static_stylesheet_token_start(&tokens[value_end_index]);
        let value = source.get(value_start..value_end)?.trim().to_string();
        if value.is_empty() || !static_stylesheet_property_value_is_removal_safe(&value) {
            return None;
        }

        let scope_id = static_stylesheet_scope_for_position(
            scopes,
            static_stylesheet_token_start(&tokens[index]),
        )?;
        declarations.insert(
            (scope_id, format!("${}", tokens[index].text)),
            StaticStylesheetPropertyDeclaration { value },
        );
        index = value_end_index + 1;
    }
    Some(declarations)
}

fn collect_static_stylesheet_scopes(source: &str) -> Option<Vec<StaticStylesheetScope>> {
    let mut scopes = vec![StaticStylesheetScope {
        parent_id: None,
        body_start: 0,
        end: source.len(),
    }];
    let mut stack = vec![0usize];
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    let bytes = source.as_bytes();

    while index < source.len() {
        let ch = source[index..].chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = source[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        if matches!(ch, '"' | '\'') {
            quote = Some(ch);
            index += ch.len_utf8();
            continue;
        }
        if bytes.get(index..index + 2) == Some(b"/*") {
            let end = source.get(index + 2..)?.find("*/")?;
            index += end + 4;
            continue;
        }
        if bytes.get(index..index + 2) == Some(b"//") {
            let line_end = source
                .get(index + 2..)?
                .find('\n')
                .map(|offset| index + 2 + offset)
                .unwrap_or(source.len());
            index = line_end;
            continue;
        }

        match ch {
            '{' => {
                let parent_id = *stack.last()?;
                let scope_id = scopes.len();
                scopes.push(StaticStylesheetScope {
                    parent_id: Some(parent_id),
                    body_start: index + ch.len_utf8(),
                    end: source.len(),
                });
                stack.push(scope_id);
            }
            '}' => {
                let scope_id = stack.pop()?;
                if scope_id == 0 {
                    return None;
                }
                scopes.get_mut(scope_id)?.end = index;
            }
            _ => {}
        }
        index += ch.len_utf8();
    }

    (stack.len() == 1).then_some(scopes)
}

fn static_stylesheet_scope_for_position(
    scopes: &[StaticStylesheetScope],
    position: usize,
) -> Option<usize> {
    scopes
        .iter()
        .enumerate()
        .rev()
        .find_map(|(scope_id, scope)| {
            (position >= scope.body_start && position < scope.end).then_some(scope_id)
        })
}

fn resolve_static_scss_variable_abstract_value_at_position(
    name: &str,
    position: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &[StaticStylesheetScopedVariableDeclaration],
    stack: &mut BTreeSet<(usize, String, usize)>,
    fuel: usize,
) -> StaticStylesheetAbstractResolution {
    let Some(scope_id) = static_stylesheet_scope_for_position(scopes, position) else {
        return top_static_abstract_value(StaticStylesheetResolutionReason::UnresolvedReference);
    };
    resolve_static_scss_variable_abstract_value_in_scope(
        name,
        scope_id,
        position,
        scopes,
        declarations,
        stack,
        fuel,
    )
}

fn resolve_static_scss_variable_abstract_value_in_scope(
    name: &str,
    scope_id: usize,
    position: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &[StaticStylesheetScopedVariableDeclaration],
    stack: &mut BTreeSet<(usize, String, usize)>,
    fuel: usize,
) -> StaticStylesheetAbstractResolution {
    if fuel == 0 {
        return top_static_abstract_value(StaticStylesheetResolutionReason::FuelExhausted);
    }
    let Some(declaration) =
        find_static_scss_variable_declaration(name, scope_id, position, scopes, declarations)
    else {
        return top_static_abstract_value(StaticStylesheetResolutionReason::UnresolvedReference);
    };
    let stack_key = (
        declaration.scope_id,
        canonical_static_scss_variable_name(name),
        declaration.declaration.span_start,
    );
    if !stack.insert(stack_key.clone()) {
        return top_static_abstract_value(StaticStylesheetResolutionReason::Cycle);
    }
    let resolved = resolve_static_scss_variable_abstract_value_text(
        declaration.declaration.value.trim(),
        declaration.declaration.span_start,
        scopes,
        declarations,
        stack,
        fuel - 1,
    );
    stack.remove(&stack_key);
    resolved
}

fn resolve_static_scss_variable_abstract_value_text(
    value: &str,
    position: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &[StaticStylesheetScopedVariableDeclaration],
    stack: &mut BTreeSet<(usize, String, usize)>,
    fuel: usize,
) -> StaticStylesheetAbstractResolution {
    let Some(references) =
        collect_static_stylesheet_variable_references(value, StaticStylesheetVariableKind::Scss)
    else {
        return raw_static_abstract_value(
            value,
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        );
    };
    if references.is_empty() {
        if static_stylesheet_literal_value_is_safe(value) {
            return resolved_static_abstract_value(value);
        }
        return raw_static_abstract_value(
            value,
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        );
    }
    if !static_stylesheet_composite_value_is_safe(value) {
        return raw_static_abstract_value(
            value,
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        );
    }

    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    for reference in references {
        let resolved = resolve_static_scss_variable_abstract_value_at_position(
            reference.name.as_str(),
            position,
            scopes,
            declarations,
            stack,
            fuel,
        );
        let Some(rendered_value) = resolved.rendered_value else {
            return top_static_abstract_value(resolved.reason);
        };
        output.push_str(&value[cursor..reference.start]);
        output.push_str(&rendered_value);
        cursor = reference.end;
    }
    output.push_str(&value[cursor..]);
    resolved_static_abstract_value(output.as_str())
}

fn resolve_static_scss_variable_value_at_position(
    name: &str,
    position: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &[StaticStylesheetScopedVariableDeclaration],
    stack: &mut BTreeSet<(usize, String, usize)>,
) -> Option<String> {
    let scope_id = static_stylesheet_scope_for_position(scopes, position)?;
    resolve_static_scss_variable_value_in_scope(
        name,
        scope_id,
        position,
        scopes,
        declarations,
        stack,
    )
}

fn resolve_static_scss_variable_value_in_scope(
    name: &str,
    scope_id: usize,
    position: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &[StaticStylesheetScopedVariableDeclaration],
    stack: &mut BTreeSet<(usize, String, usize)>,
) -> Option<String> {
    let stack_key = (
        scope_id,
        canonical_static_scss_variable_name(name),
        position,
    );
    if !stack.insert(stack_key.clone()) {
        return None;
    }
    let declaration =
        find_static_scss_variable_declaration(name, scope_id, position, scopes, declarations)?;
    let resolved = resolve_static_scss_variable_value_text(
        declaration.declaration.value.trim(),
        declaration.declaration.span_start,
        scopes,
        declarations,
        stack,
    );
    stack.remove(&stack_key);
    resolved
}

fn find_static_scss_variable_declaration<'a>(
    name: &str,
    mut scope_id: usize,
    position: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &'a [StaticStylesheetScopedVariableDeclaration],
) -> Option<&'a StaticStylesheetScopedVariableDeclaration> {
    loop {
        if let Some(declaration) = find_static_scss_variable_declaration_in_scope(
            name,
            scope_id,
            position,
            scopes,
            declarations,
        ) {
            return Some(declaration);
        }
        scope_id = scopes.get(scope_id)?.parent_id?;
    }
}

fn find_static_scss_variable_declaration_in_scope<'a>(
    name: &str,
    scope_id: usize,
    position: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &'a [StaticStylesheetScopedVariableDeclaration],
) -> Option<&'a StaticStylesheetScopedVariableDeclaration> {
    let mut active = None;
    for declaration in declarations.iter().filter(|declaration| {
        static_scss_variable_names_equal(&declaration.name, name)
            && declaration.scope_id == scope_id
            && declaration.declaration.span_end <= position
    }) {
        if declaration.declaration.is_default {
            let has_visible_value = active.is_some()
                || scopes
                    .get(scope_id)
                    .and_then(|scope| scope.parent_id)
                    .and_then(|parent_scope_id| {
                        find_static_scss_variable_declaration(
                            name,
                            parent_scope_id,
                            declaration.declaration.span_start,
                            scopes,
                            declarations,
                        )
                    })
                    .is_some();
            if !has_visible_value {
                active = Some(declaration);
            }
            continue;
        }
        active = Some(declaration);
    }
    active
}

fn resolve_static_scss_variable_value_text(
    value: &str,
    position: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &[StaticStylesheetScopedVariableDeclaration],
    stack: &mut BTreeSet<(usize, String, usize)>,
) -> Option<String> {
    let references =
        collect_static_stylesheet_variable_references(value, StaticStylesheetVariableKind::Scss)?;
    if references.is_empty() {
        return static_stylesheet_literal_value_is_safe(value).then(|| value.to_string());
    }
    if !static_stylesheet_composite_value_is_safe(value) {
        return None;
    }

    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    for reference in references {
        let resolved = resolve_static_scss_variable_value_at_position(
            reference.name.as_str(),
            position,
            scopes,
            declarations,
            stack,
        )?;
        output.push_str(&value[cursor..reference.start]);
        output.push_str(&resolved);
        cursor = reference.end;
    }
    output.push_str(&value[cursor..]);
    Some(output)
}

fn resolve_static_less_variable_abstract_value_in_scope(
    name: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    stack: &mut BTreeSet<(usize, String)>,
    fuel: usize,
) -> StaticStylesheetAbstractResolution {
    if fuel == 0 {
        return top_static_abstract_value(StaticStylesheetResolutionReason::FuelExhausted);
    }
    let Some(declaration) =
        find_static_less_variable_declaration(name, scope_id, scopes, declarations)
    else {
        return top_static_abstract_value(StaticStylesheetResolutionReason::UnresolvedReference);
    };
    let stack_key = (scope_id, name.to_string());
    if !stack.insert(stack_key.clone()) {
        return top_static_abstract_value(StaticStylesheetResolutionReason::Cycle);
    }
    let resolved = resolve_static_less_variable_abstract_value_text(
        declaration.value.trim(),
        scope_id,
        scopes,
        declarations,
        stack,
        fuel - 1,
    );
    stack.remove(&stack_key);
    if let Some(rendered_value) = resolved.rendered_value.as_deref() {
        return resolved_static_abstract_value(
            reduce_static_less_parenthesized_numeric_value(rendered_value.to_string()).as_str(),
        );
    }
    resolved
}

fn resolve_static_less_variable_abstract_value_text(
    value: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    stack: &mut BTreeSet<(usize, String)>,
    fuel: usize,
) -> StaticStylesheetAbstractResolution {
    let Some(references) =
        collect_static_stylesheet_variable_references(value, StaticStylesheetVariableKind::Less)
    else {
        return raw_static_abstract_value(
            value,
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        );
    };
    if references.is_empty() {
        if static_stylesheet_literal_value_is_safe(value) {
            return resolved_static_abstract_value(value);
        }
        return raw_static_abstract_value(
            value,
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        );
    }
    if !static_stylesheet_composite_value_is_safe(value) {
        return raw_static_abstract_value(
            value,
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        );
    }

    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    for reference in references {
        let resolved = resolve_static_less_variable_abstract_value_in_scope(
            reference.name.as_str(),
            scope_id,
            scopes,
            declarations,
            stack,
            fuel,
        );
        let Some(rendered_value) = resolved.rendered_value else {
            return top_static_abstract_value(resolved.reason);
        };
        output.push_str(&value[cursor..reference.start]);
        output.push_str(&rendered_value);
        cursor = reference.end;
    }
    output.push_str(&value[cursor..]);
    resolved_static_abstract_value(output.as_str())
}

fn resolve_static_less_variable_value_in_scope(
    name: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    stack: &mut BTreeSet<(usize, String)>,
) -> Option<String> {
    let stack_key = (scope_id, name.to_string());
    if !stack.insert(stack_key.clone()) {
        return None;
    }
    let declaration = find_static_less_variable_declaration(name, scope_id, scopes, declarations)?;
    let resolved = resolve_static_less_variable_value_text(
        declaration.value.trim(),
        scope_id,
        scopes,
        declarations,
        stack,
    );
    stack.remove(&stack_key);
    resolved.map(reduce_static_less_parenthesized_numeric_value)
}

fn find_static_less_variable_declaration<'a>(
    name: &str,
    mut scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &'a BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
) -> Option<&'a StaticStylesheetVariableDeclaration> {
    loop {
        if let Some(declaration) = declarations.get(&(scope_id, name.to_string())) {
            return Some(declaration);
        }
        scope_id = scopes.get(scope_id)?.parent_id?;
    }
}

fn resolve_static_less_variable_value_text(
    value: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    stack: &mut BTreeSet<(usize, String)>,
) -> Option<String> {
    let references =
        collect_static_stylesheet_variable_references(value, StaticStylesheetVariableKind::Less)?;
    if references.is_empty() {
        return static_stylesheet_literal_value_is_safe(value).then(|| value.to_string());
    }
    if !static_stylesheet_composite_value_is_safe(value) {
        return None;
    }

    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    for reference in references {
        let resolved = resolve_static_less_variable_value_in_scope(
            reference.name.as_str(),
            scope_id,
            scopes,
            declarations,
            stack,
        )?;
        output.push_str(&value[cursor..reference.start]);
        output.push_str(&resolved);
        cursor = reference.end;
    }
    output.push_str(&value[cursor..]);
    Some(output)
}

fn resolve_static_less_property_abstract_value_in_scope(
    name: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    stack: &mut BTreeSet<(usize, String)>,
    fuel: usize,
) -> StaticStylesheetAbstractResolution {
    if fuel == 0 {
        return top_static_abstract_value(StaticStylesheetResolutionReason::FuelExhausted);
    }
    let Some(declaration) =
        find_static_less_property_declaration(name, scope_id, scopes, declarations)
    else {
        return top_static_abstract_value(StaticStylesheetResolutionReason::UnresolvedReference);
    };
    let stack_key = (scope_id, name.to_string());
    if !stack.insert(stack_key.clone()) {
        return top_static_abstract_value(StaticStylesheetResolutionReason::Cycle);
    }
    let resolved = resolve_static_less_property_abstract_value_text(
        declaration.value.trim(),
        scope_id,
        scopes,
        declarations,
        stack,
        fuel - 1,
    );
    stack.remove(&stack_key);
    resolved
}

fn resolve_static_less_property_abstract_value_text(
    value: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    stack: &mut BTreeSet<(usize, String)>,
    fuel: usize,
) -> StaticStylesheetAbstractResolution {
    let Some(references) =
        collect_static_stylesheet_variable_references(value, StaticStylesheetVariableKind::Scss)
    else {
        return raw_static_abstract_value(
            value,
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        );
    };
    if references.is_empty() {
        if static_stylesheet_literal_value_is_safe(value) {
            return resolved_static_abstract_value(value);
        }
        return raw_static_abstract_value(
            value,
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        );
    }
    if !static_stylesheet_composite_value_is_safe(value) {
        return raw_static_abstract_value(
            value,
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        );
    }

    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    for reference in references {
        let resolved = resolve_static_less_property_abstract_value_in_scope(
            reference.name.as_str(),
            scope_id,
            scopes,
            declarations,
            stack,
            fuel,
        );
        let Some(rendered_value) = resolved.rendered_value else {
            return top_static_abstract_value(resolved.reason);
        };
        output.push_str(&value[cursor..reference.start]);
        output.push_str(&rendered_value);
        cursor = reference.end;
    }
    output.push_str(&value[cursor..]);
    resolved_static_abstract_value(output.as_str())
}

fn resolve_static_less_property_value_in_scope(
    name: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    stack: &mut BTreeSet<(usize, String)>,
) -> Option<String> {
    let stack_key = (scope_id, name.to_string());
    if !stack.insert(stack_key.clone()) {
        return None;
    }
    let declaration = find_static_less_property_declaration(name, scope_id, scopes, declarations)?;
    let resolved = resolve_static_less_property_value_text(
        declaration.value.trim(),
        scope_id,
        scopes,
        declarations,
        stack,
    );
    stack.remove(&stack_key);
    resolved
}

fn find_static_less_property_declaration<'a>(
    name: &str,
    mut scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &'a BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
) -> Option<&'a StaticStylesheetPropertyDeclaration> {
    loop {
        if let Some(declaration) = declarations.get(&(scope_id, name.to_string())) {
            return Some(declaration);
        }
        scope_id = scopes.get(scope_id)?.parent_id?;
    }
}

fn resolve_static_less_property_value_text(
    value: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    stack: &mut BTreeSet<(usize, String)>,
) -> Option<String> {
    let references =
        collect_static_stylesheet_variable_references(value, StaticStylesheetVariableKind::Scss)?;
    if references.is_empty() {
        return static_stylesheet_literal_value_is_safe(value).then(|| value.to_string());
    }
    if !static_stylesheet_composite_value_is_safe(value) {
        return None;
    }

    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    for reference in references {
        let resolved = resolve_static_less_property_value_in_scope(
            reference.name.as_str(),
            scope_id,
            scopes,
            declarations,
            stack,
        )?;
        output.push_str(&value[cursor..reference.start]);
        output.push_str(&resolved);
        cursor = reference.end;
    }
    output.push_str(&value[cursor..]);
    Some(output)
}

fn reduce_static_less_parenthesized_numeric_value(value: String) -> String {
    let trimmed = value.trim();
    let Some(inner) = trimmed
        .strip_prefix('(')
        .and_then(|without_left| without_left.strip_suffix(')'))
    else {
        return value;
    };
    reduce_static_numeric_expression(inner.trim()).unwrap_or(value)
}

fn static_stylesheet_less_declaration_value_is_removal_safe(value: &str) -> bool {
    !value.chars().any(|ch| matches!(ch, '{' | '}' | ';' | '!'))
}

fn static_stylesheet_scss_declaration_value_is_removal_safe(value: &str) -> bool {
    !value.chars().any(|ch| matches!(ch, '{' | '}' | ';' | '!'))
}

fn static_stylesheet_property_name_is_safe(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
}

fn static_stylesheet_property_value_is_removal_safe(value: &str) -> bool {
    !value.chars().any(|ch| matches!(ch, '{' | '}' | ';' | '!'))
}

fn static_stylesheet_previous_token_starts_declaration(
    tokens: &[LexedToken],
    index: usize,
) -> bool {
    tokens[..index]
        .iter()
        .rev()
        .find(|token| !static_stylesheet_token_is_trivia(token.kind))
        .is_some_and(|token| matches!(token.kind, SyntaxKind::LeftBrace | SyntaxKind::Semicolon))
}

fn static_stylesheet_declaration_value_end_token(
    tokens: &[LexedToken],
    index: usize,
) -> Option<usize> {
    static_stylesheet_value_end_token_until(tokens, index, tokens.len())
}

fn static_stylesheet_value_end_token_until(
    tokens: &[LexedToken],
    mut index: usize,
    end: usize,
) -> Option<usize> {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.checked_sub(1)?,
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.checked_sub(1)?,
            SyntaxKind::Semicolon | SyntaxKind::RightBrace
                if paren_depth == 0 && bracket_depth == 0 =>
            {
                return Some(index);
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn static_stylesheet_skip_trivia_tokens(tokens: &[LexedToken], mut index: usize) -> usize {
    while tokens
        .get(index)
        .is_some_and(|token| static_stylesheet_token_is_trivia(token.kind))
    {
        index += 1;
    }
    index
}

fn static_stylesheet_token_is_trivia(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Whitespace
            | SyntaxKind::LineComment
            | SyntaxKind::BlockComment
            | SyntaxKind::ScssSilentComment
    )
}

fn extract_static_stylesheet_variable_declaration(
    source: &str,
    variable_start: usize,
    variable_end: usize,
    variable_kind: StaticStylesheetVariableKind,
) -> Option<StaticStylesheetVariableDeclaration> {
    let after_name = source.get(variable_end..)?;
    let colon_offset = after_name.find(':')?;
    let value_start = variable_end + colon_offset + 1;
    let terminator_offset = source.get(value_start..)?.find(';')?;
    let span_end = value_start + terminator_offset + 1;
    let (value, is_default, is_global) = parse_static_stylesheet_declaration_value(
        source.get(value_start..span_end - 1)?,
        variable_kind,
    );
    Some(StaticStylesheetVariableDeclaration {
        value,
        span_start: variable_start,
        span_end,
        removal_spans: vec![(variable_start, span_end)],
        is_default,
        is_global,
    })
}

fn parse_static_stylesheet_declaration_value(
    value: &str,
    variable_kind: StaticStylesheetVariableKind,
) -> (String, bool, bool) {
    let mut value = value.trim();
    let mut is_default = false;
    let mut is_global = false;
    if variable_kind == StaticStylesheetVariableKind::Scss {
        loop {
            if let Some(before_flag) = value.strip_suffix("!default")
                && before_flag
                    .chars()
                    .next_back()
                    .is_some_and(char::is_whitespace)
            {
                is_default = true;
                value = before_flag.trim_end();
                continue;
            }
            if let Some(before_flag) = value.strip_suffix("!global")
                && before_flag
                    .chars()
                    .next_back()
                    .is_some_and(char::is_whitespace)
            {
                is_global = true;
                value = before_flag.trim_end();
                continue;
            }
            break;
        }
    }
    (value.to_string(), is_default, is_global)
}

fn merge_static_stylesheet_duplicate_declaration(
    previous: &mut StaticStylesheetVariableDeclaration,
    declaration: StaticStylesheetVariableDeclaration,
    variable_kind: StaticStylesheetVariableKind,
) -> Option<()> {
    match variable_kind {
        StaticStylesheetVariableKind::Less => {
            let mut removal_spans = previous.removal_spans.clone();
            removal_spans.extend(declaration.removal_spans.iter().copied());
            *previous = StaticStylesheetVariableDeclaration {
                removal_spans,
                ..declaration
            };
            Some(())
        }
        StaticStylesheetVariableKind::Scss if declaration.is_default => {
            previous
                .removal_spans
                .extend(declaration.removal_spans.iter().copied());
            Some(())
        }
        StaticStylesheetVariableKind::Scss if previous.is_default => {
            let mut removal_spans = previous.removal_spans.clone();
            removal_spans.extend(declaration.removal_spans.iter().copied());
            *previous = StaticStylesheetVariableDeclaration {
                removal_spans,
                ..declaration
            };
            Some(())
        }
        StaticStylesheetVariableKind::Scss => None,
    }
}

fn static_stylesheet_literal_value_is_safe(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && !value
            .chars()
            .any(|ch| matches!(ch, '{' | '}' | ';' | '$' | '@' | '!'))
}

fn static_stylesheet_variable_name_is_safe(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}

fn static_scss_callable_name_is_safe(name: &str) -> bool {
    static_stylesheet_variable_name_is_safe(name)
}

fn static_scss_function_argument_is_safe(value: &str) -> bool {
    !value.is_empty()
        && !value.contains("...")
        && !value
            .chars()
            .any(|ch| matches!(ch, '{' | '}' | ';' | '!' | ':'))
}

fn static_scss_public_module_variable_name(name: &str) -> Option<String> {
    let bare_name = name.strip_prefix('$')?;
    if bare_name.starts_with('-') || bare_name.starts_with('_') || bare_name.is_empty() {
        return None;
    }
    Some(canonical_static_scss_variable_name(bare_name))
}

pub fn canonical_static_scss_variable_name(name: &str) -> String {
    name.trim()
        .strip_prefix('$')
        .unwrap_or_else(|| name.trim())
        .replace('_', "-")
}

fn canonical_static_scss_function_name(name: &str) -> String {
    name.trim().replace('_', "-")
}

pub fn static_scss_variable_names_equal(left: &str, right: &str) -> bool {
    canonical_static_scss_variable_name(left) == canonical_static_scss_variable_name(right)
}

fn static_stylesheet_composite_value_is_safe(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty() && !value.chars().any(|ch| matches!(ch, '{' | '}' | ';' | '!'))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StaticStylesheetVariableReference {
    name: String,
    start: usize,
    end: usize,
}

fn collect_static_stylesheet_variable_references(
    value: &str,
    variable_kind: StaticStylesheetVariableKind,
) -> Option<Vec<StaticStylesheetVariableReference>> {
    let prefix = variable_kind.reference_prefix();
    let other_prefix = match variable_kind {
        StaticStylesheetVariableKind::Scss => '@',
        StaticStylesheetVariableKind::Less => '$',
    };
    let mut references = Vec::new();
    let mut index = 0usize;
    let mut quote: Option<char> = None;

    while index < value.len() {
        let ch = value[index..].chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = value[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        if matches!(ch, '"' | '\'') {
            quote = Some(ch);
            index += ch.len_utf8();
            continue;
        }
        if ch == other_prefix {
            return None;
        }
        if ch != prefix {
            index += ch.len_utf8();
            continue;
        }

        let name_start = index + ch.len_utf8();
        let name_end = static_stylesheet_variable_name_end(value, name_start);
        if name_end == name_start {
            return None;
        }
        let bare_name = &value[name_start..name_end];
        if !static_stylesheet_variable_name_is_safe(bare_name) {
            return None;
        }
        references.push(StaticStylesheetVariableReference {
            name: value[index..name_end].to_string(),
            start: index,
            end: name_end,
        });
        index = name_end;
    }

    Some(references)
}

fn static_stylesheet_variable_name_end(value: &str, mut index: usize) -> usize {
    while index < value.len() {
        let Some(ch) = value[index..].chars().next() else {
            break;
        };
        if !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '-') {
            break;
        }
        index += ch.len_utf8();
    }
    index
}

fn static_stylesheet_position_is_inside_scoped_declaration(
    declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    position: usize,
) -> bool {
    declarations.values().any(|declaration| {
        declaration
            .removal_spans
            .iter()
            .any(|(start, end)| position >= *start && position < *end)
    })
}

fn static_stylesheet_position_is_inside_scss_declaration(
    declarations: &[StaticStylesheetScopedVariableDeclaration],
    position: usize,
) -> bool {
    declarations.iter().any(|declaration| {
        declaration
            .removal_spans
            .iter()
            .any(|(start, end)| position >= *start && position < *end)
    })
}

fn apply_static_stylesheet_evaluation_edits(
    source: &str,
    mut edits: Vec<StaticStylesheetEvaluationEdit>,
) -> Option<String> {
    edits.sort_by_key(|edit| edit.start);
    let mut previous_end = 0usize;
    for edit in &edits {
        if edit.start < previous_end || edit.start > edit.end || edit.end > source.len() {
            return None;
        }
        previous_end = edit.end;
    }

    let mut output = source.to_string();
    for edit in edits.into_iter().rev() {
        output.replace_range(edit.start..edit.end, edit.replacement.as_str());
    }
    Some(output)
}

fn parser_text_size_to_usize(value: u32) -> usize {
    value as usize
}

fn static_stylesheet_token_start(token: &LexedToken) -> usize {
    parser_text_size_to_usize(token.range.start().into())
}

fn static_stylesheet_token_end(token: &LexedToken) -> usize {
    parser_text_size_to_usize(token.range.end().into())
}

fn dialect_label(dialect: StyleDialect) -> &'static str {
    match dialect {
        StyleDialect::Css => "css",
        StyleDialect::Scss => "scss",
        StyleDialect::Sass => "sass",
        StyleDialect::Less => "less",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write as _;

    #[test]
    fn static_scss_evaluation_emits_abstract_replacement_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "$gap: 0px; .button { margin: $gap; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(
            report.evaluator,
            "omena-query-static-scss-variable-evaluator"
        );
        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert_eq!(report.resolved_replacements[0].text, "0px");
        assert_eq!(report.value_resolution.resolved_count, 1);
        assert!(report.evaluated_css.contains("margin: 0px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_uses_value_lattice_numeric_reduction() {
        let report = derive_static_stylesheet_module_evaluation(
            "@gap: (1px + 2px); .button { margin: @gap; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.resolved_replacements[0].text, "3px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert_eq!(
            report.value_resolution.values[0].rendered_value.as_deref(),
            Some("3px")
        );
        assert!(report.evaluated_css.contains("margin: 3px"));
    }

    #[test]
    fn static_scss_evaluation_resolves_same_file_function_calls() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function gap($value) { @return $value; } .button { margin: gap(0px); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:gap");
        assert_eq!(report.resolved_replacements[0].text, "0px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(!report.evaluated_css.contains("@function"));
        assert!(report.evaluated_css.contains(".button { margin: 0px; }"));
        assert_eq!(report.value_resolution.reference_count, 1);
        assert_eq!(report.value_resolution.resolved_count, 1);
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_skips_recursive_function_calls() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function loop($value) { @return loop($value); } .button { color: loop(red); }",
            StyleDialect::Scss,
        );
        assert!(report.is_none());

        let resolution = summarize_static_stylesheet_value_resolution(
            "@function loop($value) { @return loop($value); } .button { color: loop(red); }",
            StyleDialect::Scss,
        );
        assert!(resolution.is_some());
        let Some(resolution) = resolution else {
            return;
        };

        assert_eq!(resolution.reference_count, 2);
        assert_eq!(resolution.top_count, 2);
        assert_eq!(resolution.cycle_count, 2);
        assert!(
            resolution
                .values
                .iter()
                .all(|value| value.outcome == "top" && value.reason == "cycle")
        );
    }

    #[test]
    fn static_value_resolution_reports_unresolved_references_as_top() {
        let report = summarize_static_stylesheet_value_resolution(
            ".button { color: $missing; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.reference_count, 1);
        assert_eq!(report.top_count, 1);
        assert_eq!(report.unresolved_reference_count, 1);
        assert_eq!(report.values[0].outcome, "top");
        assert_eq!(report.values[0].reason, "unresolvedReference");
        assert_eq!(report.values[0].rendered_value, None);
    }

    #[test]
    fn static_value_resolution_reports_cycles_as_top() {
        let report = summarize_static_stylesheet_value_resolution(
            "@a: @b; @b: @a; .button { color: @a; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.reference_count, 1);
        assert_eq!(report.top_count, 1);
        assert_eq!(report.cycle_count, 1);
        assert_eq!(report.values[0].outcome, "top");
        assert_eq!(report.values[0].reason, "cycle");
    }

    #[test]
    fn static_value_resolution_keeps_dynamic_values_raw() {
        let report = summarize_static_stylesheet_value_resolution(
            "$tone: color.mix(red, blue); .button { color: $tone; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.reference_count, 1);
        assert_eq!(report.raw_count, 1);
        assert_eq!(report.unsupported_dynamic_count, 1);
        assert_eq!(report.values[0].outcome, "raw");
        assert_eq!(report.values[0].reason, "unsupportedDynamic");
        assert_eq!(
            report.values[0].rendered_value.as_deref(),
            Some("color.mix(red, blue)")
        );
    }

    #[test]
    fn static_value_resolution_reports_fuel_exhaustion_as_top() {
        let mut source = String::new();
        for index in 0..130 {
            let _ = write!(source, "@v{index}: @v{}; ", index + 1);
        }
        source.push_str("@v130: 1px; .button { width: @v0; }");

        let report = summarize_static_stylesheet_value_resolution(&source, StyleDialect::Less);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.reference_count, 1);
        assert_eq!(report.top_count, 1);
        assert_eq!(report.fuel_exhausted_count, 1);
        assert_eq!(report.values[0].outcome, "top");
        assert_eq!(report.values[0].reason, "fuelExhausted");
    }
}
