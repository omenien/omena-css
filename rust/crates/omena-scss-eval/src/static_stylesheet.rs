use std::collections::{BTreeMap, BTreeSet};

use omena_abstract_value::{AbstractCssValueV0, abstract_css_value_from_text};
use omena_parser::{LexedToken, ParsedVariableFact, ParsedVariableFactKind, StyleDialect, lex};
use omena_syntax::SyntaxKind;
use serde::Serialize;

use crate::{
    abstract_css_value_kind,
    scss_metadata::reduce_static_scss_metadata_with_context,
    static_loop_frames::parse_static_scss_each_loop_binding_frames,
    summarize_omena_scss_eval_oracle,
    value_eval::{
        reduce_static_numeric_value, reduce_static_scss_value,
        static_scss_bang_usage_is_comparison_only, static_scss_literal_truthiness,
    },
};

mod less_guard;
mod model;
mod oracle_corpus;
mod value_resolution_model;

use less_guard::{static_less_mixin_guard_depends_on_default, static_less_mixin_guard_matches};
use model::{
    StaticLessDetachedRulesetAccessor, StaticLessDetachedRulesetCall,
    StaticLessDetachedRulesetDeclaration, StaticLessMixinAccessor,
    StaticLessMixinAccessorRenderOutcome, StaticLessMixinAccessorRenderResult,
    StaticLessMixinBodyLocalDeclaration, StaticLessMixinCall, StaticLessMixinDeclaration,
    StaticLessMixinRenderContext, StaticLessMixinRenderOutcome, StaticLessMixinRenderResult,
    StaticScssFunctionArgument, StaticScssFunctionCall, StaticScssFunctionDeclaration,
    StaticScssFunctionLocalScope, StaticScssFunctionLocalVariable, StaticScssFunctionParameter,
    StaticScssFunctionResolutionContext, StaticScssFunctionReturnClause, StaticScssLoopHeader,
    StaticScssMixinBodyLocalDeclaration, StaticScssMixinDeclaration, StaticScssMixinIncludeCall,
    StaticScssMixinRenderResult, StaticStylesheetEvaluationEdit,
    StaticStylesheetPropertyDeclaration, StaticStylesheetScope,
    StaticStylesheetScopedVariableDeclaration, StaticStylesheetVariableDeclaration,
    StaticStylesheetVariableKind,
};
pub use oracle_corpus::{
    OmenaScssEvalStaticStylesheetOracleCorpusFixtureReportV0,
    OmenaScssEvalStaticStylesheetOracleCorpusReportV0, summarize_static_stylesheet_oracle_corpus,
};
use value_resolution_model::{
    StaticStylesheetAbstractResolution, StaticStylesheetResolutionOutcome,
    StaticStylesheetResolutionReason, raw_static_abstract_value, render_static_abstract_value,
    resolved_static_abstract_value, resolved_static_abstract_value_preserving_callable_raw,
    static_value_resolution_record, top_static_abstract_value,
};

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
    pub rendered_value: Option<String>,
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
    let mixin_declarations = collect_static_scss_mixin_declarations(style_source, tokens)?;
    if !variable_facts
        .iter()
        .any(|fact| fact.kind == ParsedVariableFactKind::ScssDeclaration)
        && function_declarations.is_empty()
        && mixin_declarations.is_empty()
    {
        return None;
    }
    let scopes = collect_static_stylesheet_scopes(style_source)?;
    let function_declaration_ranges =
        static_scss_function_declaration_ranges_from_declarations(function_declarations.as_slice());
    let mixin_declaration_ranges =
        static_scss_mixin_declaration_ranges_from_declarations(mixin_declarations.as_slice());
    let function_call_ranges =
        collect_static_scss_function_calls(style_source, tokens, function_declarations.as_slice())
            .map(|calls| {
                calls
                    .into_iter()
                    .map(|call| (call.start, call.end))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
    let declarations =
        collect_static_scss_variable_declarations(style_source, variable_facts, &scopes)?
            .into_iter()
            .filter(|declaration| {
                !static_stylesheet_position_is_inside_ranges(
                    declaration.declaration.span_start,
                    &function_declaration_ranges,
                ) && !static_stylesheet_position_is_inside_ranges(
                    declaration.declaration.span_start,
                    &mixin_declaration_ranges,
                ) && !static_stylesheet_position_is_inside_ranges(
                    declaration.declaration.span_start,
                    &function_call_ranges,
                )
            })
            .collect::<Vec<_>>();

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
        if static_stylesheet_position_is_scss_module_member_reference(style_source, reference_start)
        {
            continue;
        }
        if static_stylesheet_position_is_inside_scss_declaration(&declarations, reference_start)
            || static_stylesheet_position_is_inside_ranges(
                reference_start,
                &function_declaration_ranges,
            )
            || static_stylesheet_position_is_inside_ranges(
                reference_start,
                &mixin_declaration_ranges,
            )
            || static_stylesheet_position_is_inside_ranges(reference_start, &function_call_ranges)
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
            &mixin_declarations,
            &scopes,
            &declarations,
        )
    {
        edits.extend(function_edits);
        resolved_replacements.extend(function_replacements);
    }
    if let Some(mixin_edits) = collect_static_scss_mixin_evaluation_edits(
        style_source,
        tokens,
        &function_declarations,
        &mixin_declarations,
        &scopes,
        &declarations,
    ) {
        edits.extend(mixin_edits);
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
    let mixin_declarations = collect_static_less_mixin_declarations(style_source, tokens)?;
    let mixin_declaration_ranges =
        static_less_mixin_declaration_ranges_from_declarations(mixin_declarations.as_slice());
    let detached_rulesets =
        collect_static_less_detached_ruleset_declarations(style_source, tokens, &scopes)?;
    let detached_ruleset_ranges =
        static_less_detached_ruleset_ranges_from_declarations(detached_rulesets.as_slice());
    let detached_ruleset_calls = collect_static_less_detached_ruleset_calls(style_source, tokens)?;
    let detached_ruleset_call_ranges =
        static_less_detached_ruleset_ranges_from_calls(detached_ruleset_calls.as_slice());
    let detached_ruleset_accessors =
        collect_static_less_detached_ruleset_accessors(style_source, tokens)?;
    let detached_ruleset_accessor_ranges =
        static_less_detached_ruleset_ranges_from_accessors(detached_ruleset_accessors.as_slice());
    let mixin_calls = collect_static_less_mixin_calls(style_source, tokens).unwrap_or_default();
    let mixin_call_ranges = static_less_mixin_ranges_from_calls(mixin_calls.as_slice());
    let mixin_accessors = collect_static_less_mixin_accessors(style_source, tokens)?;
    let mixin_accessor_ranges = static_less_mixin_accessor_ranges_from_accessors(&mixin_accessors);
    let mut variable_excluded_ranges = mixin_declaration_ranges.clone();
    variable_excluded_ranges.extend(detached_ruleset_ranges.iter().copied());
    variable_excluded_ranges.extend(detached_ruleset_accessor_ranges.iter().copied());
    variable_excluded_ranges.extend(mixin_accessor_ranges.iter().copied());
    let declarations = collect_static_less_variable_declarations(
        style_source,
        variable_facts,
        &scopes,
        &variable_excluded_ranges,
    )?;
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
        let reference_end = parser_text_size_to_usize(fact.range.end().into());
        if static_stylesheet_variable_reference_is_named_argument_label(
            style_source,
            reference_start,
            reference_end,
        ) {
            continue;
        }
        if static_stylesheet_position_is_inside_scoped_declaration(&declarations, reference_start) {
            continue;
        }
        if static_stylesheet_position_is_inside_ranges(reference_start, &mixin_declaration_ranges) {
            continue;
        }
        if static_stylesheet_position_is_inside_ranges(reference_start, &detached_ruleset_ranges) {
            continue;
        }
        if static_stylesheet_position_is_inside_ranges(
            reference_start,
            &detached_ruleset_call_ranges,
        ) {
            continue;
        }
        if static_stylesheet_position_is_inside_ranges(
            reference_start,
            &detached_ruleset_accessor_ranges,
        ) {
            continue;
        }
        if static_stylesheet_position_is_inside_ranges(reference_start, &mixin_accessor_ranges) {
            continue;
        }
        let reference_scope_id = static_stylesheet_scope_for_position(&scopes, reference_start)?;
        if static_stylesheet_position_is_inside_ranges(reference_start, &mixin_call_ranges)
            && static_less_value_is_detached_ruleset_reference(
                fact.name.as_str(),
                reference_scope_id,
                &scopes,
                detached_rulesets.as_slice(),
            )
        {
            continue;
        }
        let mut stack = BTreeSet::new();
        let replacement = resolve_static_less_variable_value_in_scope(
            fact.name.as_str(),
            reference_scope_id,
            &scopes,
            &declarations,
            &mut stack,
        )?;
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
        if static_stylesheet_position_is_inside_ranges(reference_start, &mixin_declaration_ranges) {
            continue;
        }
        if static_stylesheet_position_is_inside_ranges(reference_start, &detached_ruleset_ranges) {
            continue;
        }
        if static_stylesheet_position_is_inside_ranges(
            reference_start,
            &detached_ruleset_accessor_ranges,
        ) {
            continue;
        }
        if static_stylesheet_position_is_inside_ranges(reference_start, &mixin_accessor_ranges) {
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
    edits.extend(collect_static_less_literal_value_edits(
        style_source,
        tokens,
        &declarations,
        &variable_excluded_ranges,
    )?);
    edits.extend(collect_static_less_detached_ruleset_evaluation_edits(
        style_source,
        &detached_rulesets,
        &detached_ruleset_calls,
        &mixin_declarations,
        &mixin_declaration_ranges,
        &scopes,
        &declarations,
    )?);
    edits.extend(
        collect_static_less_detached_ruleset_accessor_evaluation_edits(
            style_source,
            &detached_rulesets,
            &detached_ruleset_accessors,
            &mixin_declaration_ranges,
            &scopes,
            &declarations,
        )?,
    );
    edits.extend(collect_static_less_mixin_accessor_evaluation_edits(
        style_source,
        tokens,
        &mixin_declarations,
        &detached_rulesets,
        &scopes,
        &declarations,
        &detached_ruleset_ranges,
    )?);
    if let Some(mixin_edits) = collect_static_less_mixin_evaluation_edits(
        style_source,
        tokens,
        &mixin_declarations,
        &detached_rulesets,
        &scopes,
        &declarations,
        &detached_ruleset_ranges,
    ) {
        edits.extend(mixin_edits);
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
        rendered_value: render_static_abstract_value(&abstract_value),
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
    let mixin_declarations = collect_static_scss_mixin_declarations(style_source, tokens)?;
    let function_declaration_ranges =
        static_scss_function_declaration_ranges_from_declarations(function_declarations.as_slice());
    let mixin_declaration_ranges =
        static_scss_mixin_declaration_ranges_from_declarations(mixin_declarations.as_slice());
    let mut values = Vec::new();
    for fact in variable_facts {
        if fact.kind != ParsedVariableFactKind::ScssReference {
            continue;
        }
        let reference_start = parser_text_size_to_usize(fact.range.start().into());
        if static_stylesheet_position_is_scss_module_member_reference(style_source, reference_start)
        {
            continue;
        }
        if static_stylesheet_position_is_inside_scss_declaration(&declarations, reference_start)
            || static_stylesheet_position_is_inside_ranges(
                reference_start,
                &function_declaration_ranges,
            )
            || static_stylesheet_position_is_inside_ranges(
                reference_start,
                &mixin_declaration_ranges,
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
        &mixin_declarations,
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
    let mixin_declarations = collect_static_less_mixin_declarations(style_source, tokens)?;
    let mixin_declaration_ranges =
        static_less_mixin_declaration_ranges_from_declarations(mixin_declarations.as_slice());
    let detached_rulesets =
        collect_static_less_detached_ruleset_declarations(style_source, tokens, scopes)?;
    let detached_ruleset_ranges =
        static_less_detached_ruleset_ranges_from_declarations(detached_rulesets.as_slice());
    let detached_ruleset_calls = collect_static_less_detached_ruleset_calls(style_source, tokens)?;
    let detached_ruleset_call_ranges =
        static_less_detached_ruleset_ranges_from_calls(detached_ruleset_calls.as_slice());
    let detached_ruleset_accessors =
        collect_static_less_detached_ruleset_accessors(style_source, tokens)?;
    let detached_ruleset_accessor_ranges =
        static_less_detached_ruleset_ranges_from_accessors(detached_ruleset_accessors.as_slice());
    let mixin_calls = collect_static_less_mixin_calls(style_source, tokens).unwrap_or_default();
    let mixin_call_ranges = static_less_mixin_ranges_from_calls(mixin_calls.as_slice());
    let mixin_accessors = collect_static_less_mixin_accessors(style_source, tokens)?;
    let mixin_accessor_ranges = static_less_mixin_accessor_ranges_from_accessors(&mixin_accessors);
    let mut variable_excluded_ranges = mixin_declaration_ranges.clone();
    variable_excluded_ranges.extend(detached_ruleset_ranges.iter().copied());
    variable_excluded_ranges.extend(detached_ruleset_accessor_ranges.iter().copied());
    variable_excluded_ranges.extend(mixin_accessor_ranges.iter().copied());
    let declarations = collect_static_less_variable_declarations(
        style_source,
        variable_facts,
        scopes,
        &variable_excluded_ranges,
    )?;
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
        if static_stylesheet_position_is_inside_ranges(reference_start, &mixin_declaration_ranges) {
            continue;
        }
        if static_stylesheet_position_is_inside_ranges(reference_start, &detached_ruleset_ranges) {
            continue;
        }
        if static_stylesheet_position_is_inside_ranges(
            reference_start,
            &detached_ruleset_call_ranges,
        ) {
            continue;
        }
        if static_stylesheet_position_is_inside_ranges(
            reference_start,
            &detached_ruleset_accessor_ranges,
        ) {
            continue;
        }
        if static_stylesheet_position_is_inside_ranges(reference_start, &mixin_accessor_ranges) {
            continue;
        }
        let reference_scope_id = static_stylesheet_scope_for_position(scopes, reference_start)?;
        if static_stylesheet_position_is_inside_ranges(reference_start, &mixin_call_ranges)
            && static_less_value_is_detached_ruleset_reference(
                fact.name.as_str(),
                reference_scope_id,
                scopes,
                detached_rulesets.as_slice(),
            )
        {
            continue;
        }
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
        if static_stylesheet_position_is_inside_ranges(reference_start, &mixin_declaration_ranges) {
            continue;
        }
        if static_stylesheet_position_is_inside_ranges(reference_start, &detached_ruleset_ranges) {
            continue;
        }
        if static_stylesheet_position_is_inside_ranges(
            reference_start,
            &detached_ruleset_accessor_ranges,
        ) {
            continue;
        }
        if static_stylesheet_position_is_inside_ranges(reference_start, &mixin_accessor_ranges) {
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

fn collect_static_scss_function_evaluation_edits(
    source: &str,
    tokens: &[LexedToken],
    declarations: &[StaticScssFunctionDeclaration],
    mixin_declarations: &[StaticScssMixinDeclaration],
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
    for call in calls.iter().filter(|call| {
        !static_scss_function_call_is_inside_declaration_body(call, declarations)
            && !static_scss_function_call_is_inside_mixin_declaration_body(call, mixin_declarations)
    }) {
        let resolution = resolve_static_scss_function_call_abstract_value(
            call,
            declarations,
            mixin_declarations,
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
    extend_static_scss_used_function_dependencies(&mut used_declaration_names, declarations);

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
    mixin_declarations: &[StaticScssMixinDeclaration],
    scopes: &[StaticStylesheetScope],
    variable_declarations: &[StaticStylesheetScopedVariableDeclaration],
) -> Option<Vec<OmenaScssEvalStaticValueResolutionV0>> {
    let calls = collect_static_scss_function_calls(source, tokens, declarations)?;
    let values = calls
        .into_iter()
        .filter(|call| {
            !static_scss_function_call_is_inside_declaration_body(call, declarations)
                && !static_scss_function_call_is_inside_mixin_declaration_body(
                    call,
                    mixin_declarations,
                )
        })
        .map(|call| {
            let resolution = resolve_static_scss_function_call_abstract_value(
                &call,
                declarations,
                mixin_declarations,
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

fn collect_static_scss_mixin_evaluation_edits(
    source: &str,
    tokens: &[LexedToken],
    function_declarations: &[StaticScssFunctionDeclaration],
    mixin_declarations: &[StaticScssMixinDeclaration],
    scopes: &[StaticStylesheetScope],
    variable_declarations: &[StaticStylesheetScopedVariableDeclaration],
) -> Option<Vec<StaticStylesheetEvaluationEdit>> {
    let calls = collect_static_scss_mixin_include_calls(source, tokens, mixin_declarations)?;
    if calls.is_empty() {
        return Some(Vec::new());
    }

    let context = StaticScssFunctionResolutionContext {
        declarations: function_declarations,
        mixin_declarations,
        scopes,
        variable_declarations,
        active_functions: &BTreeSet::new(),
    };
    let mut edits = Vec::new();
    let mut used_declaration_names = BTreeSet::new();
    let mut used_function_declaration_names = BTreeSet::new();
    for call in calls.iter().filter(|call| {
        !static_scss_mixin_include_is_inside_declaration_body(call, mixin_declarations)
            && !static_scss_mixin_include_is_inside_function_declaration_body(
                call,
                function_declarations,
            )
    }) {
        let Some(declaration) = mixin_declarations.iter().find(|declaration| {
            canonical_static_scss_function_name(declaration.name.as_str())
                == canonical_static_scss_function_name(call.name.as_str())
        }) else {
            continue;
        };
        let rendered = render_static_scss_mixin_include_body(
            source,
            tokens,
            declaration,
            call,
            call.start,
            context,
        )?;
        used_declaration_names.extend(rendered.used_mixin_declaration_names);
        used_function_declaration_names.extend(rendered.used_function_declaration_names);
        edits.push(StaticStylesheetEvaluationEdit {
            start: call.start,
            end: call.end,
            replacement: rendered.body,
        });
    }

    for declaration in mixin_declarations.iter().filter(|declaration| {
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
    extend_static_scss_used_function_dependencies(
        &mut used_function_declaration_names,
        function_declarations,
    );
    for declaration in function_declarations.iter().filter(|declaration| {
        used_function_declaration_names.contains(&canonical_static_scss_function_name(
            declaration.name.as_str(),
        ))
    }) {
        edits.push(StaticStylesheetEvaluationEdit {
            start: declaration.span_start,
            end: declaration.span_end,
            replacement: String::new(),
        });
    }

    Some(edits)
}

fn collect_static_scss_resolved_function_names_in_mixin_body(
    source: &str,
    tokens: &[LexedToken],
    function_declarations: &[StaticScssFunctionDeclaration],
    mixin_declaration: &StaticScssMixinDeclaration,
    rendered_body: &str,
) -> Option<BTreeSet<String>> {
    let mut names = BTreeSet::new();
    for call in collect_static_scss_function_calls(source, tokens, function_declarations)?
        .into_iter()
        .filter(|call| {
            call.start >= mixin_declaration.body_start && call.start < mixin_declaration.body_end
        })
    {
        if !static_scss_function_value_contains_callable_to(rendered_body, call.name.as_str()) {
            names.insert(canonical_static_scss_function_name(call.name.as_str()));
        }
    }
    Some(names)
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
            source,
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
        let return_clauses = collect_static_scss_function_return_clauses(
            source,
            tokens,
            body_open_index + 1,
            body_close_index,
        )?;
        let local_variables = collect_static_scss_function_local_variables(
            source,
            tokens,
            body_open_index + 1,
            body_close_index,
        )?;
        if !static_scss_function_return_clauses_are_safe(return_clauses.as_slice()) {
            index = body_close_index + 1;
            continue;
        }

        declarations.push(StaticScssFunctionDeclaration {
            name: name_token.text.clone(),
            parameters,
            local_variables,
            return_clauses,
            span_start: static_stylesheet_token_start(&tokens[index]),
            span_end: static_stylesheet_token_end(&tokens[body_close_index]),
            body_start: static_stylesheet_token_end(&tokens[body_open_index]),
            body_end: static_stylesheet_token_start(&tokens[body_close_index]),
        });
        index = body_close_index + 1;
    }
    Some(declarations)
}

fn collect_static_scss_mixin_declarations(
    source: &str,
    tokens: &[LexedToken],
) -> Option<Vec<StaticScssMixinDeclaration>> {
    let mut declarations = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        if tokens[index].kind != SyntaxKind::AtKeyword
            || !tokens[index].text.eq_ignore_ascii_case("@mixin")
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
        let after_name_index = static_stylesheet_skip_trivia_tokens(tokens, name_index + 1);
        let (parameters, body_search_index) = if tokens
            .get(after_name_index)
            .is_some_and(|token| token.kind == SyntaxKind::LeftParen)
        {
            let parameter_close_index = static_stylesheet_matching_token_index(
                tokens,
                after_name_index,
                SyntaxKind::LeftParen,
                SyntaxKind::RightParen,
            )?;
            let parameters = collect_static_scss_function_parameters(
                source,
                tokens,
                after_name_index + 1,
                parameter_close_index,
            )?;
            (parameters, parameter_close_index + 1)
        } else {
            (Vec::new(), name_index + 1)
        };
        let Some(body_open_index) = static_stylesheet_next_token_kind_index(
            tokens,
            body_search_index,
            SyntaxKind::LeftBrace,
        ) else {
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
        declarations.push(StaticScssMixinDeclaration {
            name: name_token.text.clone(),
            parameters,
            span_start: static_stylesheet_token_start(&tokens[index]),
            span_end: static_stylesheet_token_end(&tokens[body_close_index]),
            body_start: static_stylesheet_token_end(&tokens[body_open_index]),
            body_end: static_stylesheet_token_start(&tokens[body_close_index]),
        });
        index = body_close_index + 1;
    }
    Some(declarations)
}

fn collect_static_scss_function_local_variables(
    source: &str,
    tokens: &[LexedToken],
    start: usize,
    end: usize,
) -> Option<Vec<StaticScssFunctionLocalVariable>> {
    let mut variables = Vec::new();
    let mut scope_stack = Vec::<StaticScssFunctionLocalScope>::new();
    let function_scope_start = tokens
        .get(start)
        .map(static_stylesheet_token_start)
        .or_else(|| tokens.get(end).map(static_stylesheet_token_start))?;
    let function_scope_end = tokens
        .get(end)
        .map(static_stylesheet_token_start)
        .unwrap_or(function_scope_start);
    let mut index = start;
    while index < end {
        while scope_stack
            .last()
            .is_some_and(|scope| index > scope.end_index)
        {
            scope_stack.pop();
        }
        match tokens[index].kind {
            SyntaxKind::LeftBrace => {
                let scope_end_index = static_stylesheet_matching_token_index(
                    tokens,
                    index,
                    SyntaxKind::LeftBrace,
                    SyntaxKind::RightBrace,
                )?;
                scope_stack.push(StaticScssFunctionLocalScope {
                    end_index: scope_end_index,
                    span_start: static_stylesheet_token_end(&tokens[index]),
                    span_end: static_stylesheet_token_start(&tokens[scope_end_index]),
                });
                index += 1;
            }
            SyntaxKind::RightBrace => {
                if scope_stack
                    .last()
                    .is_some_and(|scope| scope.end_index == index)
                {
                    scope_stack.pop();
                }
                index += 1;
            }
            SyntaxKind::ScssVariable => {
                let colon_index = static_stylesheet_skip_trivia_tokens(tokens, index + 1);
                if tokens
                    .get(colon_index)
                    .is_none_or(|token| token.kind != SyntaxKind::Colon)
                {
                    index += 1;
                    continue;
                }
                let value_end_index =
                    static_stylesheet_value_end_token_until(tokens, colon_index + 1, end)?;
                let name = canonical_static_scss_variable_name(tokens[index].text.as_str());
                if !static_stylesheet_variable_name_is_safe(name.as_str()) {
                    return None;
                }
                let value_start = static_stylesheet_token_end(&tokens[colon_index]);
                let value_end = static_stylesheet_token_start(&tokens[value_end_index]);
                let value = source.get(value_start..value_end)?.trim();
                let (scope_start, scope_end) = scope_stack
                    .last()
                    .map(|scope| (scope.span_start, scope.span_end))
                    .unwrap_or((function_scope_start, function_scope_end));
                variables.push(StaticScssFunctionLocalVariable {
                    name,
                    value: value.to_string(),
                    span_start: static_stylesheet_token_start(&tokens[index]),
                    scope_start,
                    scope_end,
                });
                index = value_end_index + 1;
            }
            _ => {
                index += 1;
            }
        }
    }
    Some(variables)
}

fn collect_static_scss_function_parameters(
    source: &str,
    tokens: &[LexedToken],
    start: usize,
    end: usize,
) -> Option<Vec<StaticScssFunctionParameter>> {
    let parameter_start = tokens.get(start).map(static_stylesheet_token_start)?;
    let parameter_end = tokens
        .get(end)
        .map(static_stylesheet_token_start)
        .unwrap_or(parameter_start);
    let parameter_text = source.get(parameter_start..parameter_end)?.trim();
    if parameter_text.is_empty() {
        return Some(Vec::new());
    }

    let mut parameters = Vec::new();
    let mut names = BTreeSet::new();
    let mut saw_default = false;
    for argument in split_static_scss_function_arguments(parameter_text)? {
        let parameter = parse_static_scss_function_parameter(argument)?;
        if parameter.default_value.is_some() {
            saw_default = true;
        } else if saw_default {
            return None;
        }
        if parameter.pattern_value.is_none() && !names.insert(parameter.name.clone()) {
            return None;
        }
        parameters.push(parameter);
    }
    Some(parameters)
}

fn parse_static_scss_function_parameter(
    argument: StaticScssFunctionArgument,
) -> Option<StaticScssFunctionParameter> {
    if let Some(name) = argument.name {
        return Some(StaticScssFunctionParameter {
            name,
            default_value: Some(argument.value),
            variadic: false,
            pattern_value: None,
        });
    }

    let name = argument.value.trim();
    let name = name.strip_prefix('$')?.trim();
    if !static_stylesheet_variable_name_is_safe(name) {
        return None;
    }
    Some(StaticScssFunctionParameter {
        name: canonical_static_scss_variable_name(name),
        default_value: None,
        variadic: false,
        pattern_value: None,
    })
}

fn collect_static_scss_function_return_clauses(
    source: &str,
    tokens: &[LexedToken],
    start: usize,
    end: usize,
) -> Option<Vec<StaticScssFunctionReturnClause>> {
    let clauses = collect_static_scss_function_return_clauses_in_range(
        source,
        tokens,
        start,
        end,
        &Vec::new(),
    )?;
    (!clauses.is_empty()).then_some(clauses)
}

fn collect_static_scss_function_return_clauses_in_range(
    source: &str,
    tokens: &[LexedToken],
    start: usize,
    end: usize,
    loop_headers: &[StaticScssLoopHeader],
) -> Option<Vec<StaticScssFunctionReturnClause>> {
    let mut clauses = Vec::new();
    let mut branch_conditions = Vec::<String>::new();
    let mut index = start;
    while index < end {
        let token = &tokens[index];
        if token.kind != SyntaxKind::AtKeyword {
            index += 1;
            continue;
        }
        if token.text.eq_ignore_ascii_case("@return") {
            let value_end_index = static_stylesheet_value_end_token_until(tokens, index + 1, end)?;
            let value = static_scss_return_value_text(source, tokens, index, value_end_index)?;
            clauses.push(StaticScssFunctionReturnClause {
                condition: None,
                value,
                span_start: static_stylesheet_token_start(token),
                loop_headers: loop_headers.to_vec(),
            });
            index = value_end_index + 1;
            branch_conditions.clear();
            continue;
        }
        if token.text.eq_ignore_ascii_case("@if") {
            let (condition, body_open_index, body_close_index) =
                static_scss_control_block_header_and_body(source, tokens, index, end)?;
            let return_clauses = collect_static_scss_function_return_clauses_in_range(
                source,
                tokens,
                body_open_index + 1,
                body_close_index,
                loop_headers,
            )?;
            clauses.extend(
                return_clauses
                    .into_iter()
                    .map(|return_clause| {
                        static_scss_return_clause_with_condition(return_clause, condition.as_str())
                    })
                    .collect::<Vec<_>>(),
            );
            branch_conditions.clear();
            branch_conditions.push(condition);
            index = body_close_index + 1;
            continue;
        }
        if token.text.eq_ignore_ascii_case("@else") {
            let (condition, body_open_index, body_close_index) =
                static_scss_control_block_header_and_body(source, tokens, index, end)?;
            let return_clauses = collect_static_scss_function_return_clauses_in_range(
                source,
                tokens,
                body_open_index + 1,
                body_close_index,
                loop_headers,
            )?;
            let branch_condition = if let Some(else_if_condition) =
                static_scss_else_if_condition(condition.as_str())
            {
                static_scss_branch_chain_condition(branch_conditions.as_slice(), else_if_condition)
            } else {
                static_scss_branch_chain_else_condition(branch_conditions.as_slice())?
            };
            clauses.extend(
                return_clauses
                    .into_iter()
                    .map(|return_clause| {
                        static_scss_return_clause_with_condition(
                            return_clause,
                            branch_condition.as_str(),
                        )
                    })
                    .collect::<Vec<_>>(),
            );
            if let Some(else_if_condition) = static_scss_else_if_condition(condition.as_str()) {
                branch_conditions.push(else_if_condition.to_string());
            } else {
                branch_conditions.clear();
            }
            index = body_close_index + 1;
            continue;
        }
        if static_scss_loop_at_keyword(token.text.as_str()).is_some() {
            let (header, body_open_index, body_close_index) =
                static_scss_control_block_header_and_body(source, tokens, index, end)?;
            let mut nested_loop_headers = loop_headers.to_vec();
            nested_loop_headers.push(StaticScssLoopHeader {
                text: format!("{} {}", token.text.trim(), header.trim()),
                span_start: static_stylesheet_token_start(token),
                body_start: static_stylesheet_token_end(&tokens[body_open_index]),
                body_end: static_stylesheet_token_start(&tokens[body_close_index]),
            });
            clauses.extend(collect_static_scss_function_return_clauses_in_range(
                source,
                tokens,
                body_open_index + 1,
                body_close_index,
                nested_loop_headers.as_slice(),
            )?);
            branch_conditions.clear();
            index = body_close_index + 1;
            continue;
        }
        index += 1;
    }
    Some(clauses)
}

fn static_scss_loop_at_keyword(keyword: &str) -> Option<&'static str> {
    if keyword.eq_ignore_ascii_case("@for") {
        Some("@for")
    } else if keyword.eq_ignore_ascii_case("@each") {
        Some("@each")
    } else if keyword.eq_ignore_ascii_case("@while") {
        Some("@while")
    } else {
        None
    }
}

fn static_scss_return_clause_with_condition(
    mut clause: StaticScssFunctionReturnClause,
    condition: &str,
) -> StaticScssFunctionReturnClause {
    clause.condition = Some(match clause.condition {
        Some(inner_condition) => format!("({condition}) and ({inner_condition})"),
        None => condition.to_string(),
    });
    clause
}

fn static_scss_return_value_text(
    source: &str,
    tokens: &[LexedToken],
    return_index: usize,
    value_end_index: usize,
) -> Option<String> {
    let value_start = static_stylesheet_token_end(&tokens[return_index]);
    let value_end = static_stylesheet_token_start(&tokens[value_end_index]);
    let value = source.get(value_start..value_end)?.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn static_scss_control_block_header_and_body(
    source: &str,
    tokens: &[LexedToken],
    control_index: usize,
    end: usize,
) -> Option<(String, usize, usize)> {
    let body_open_index =
        (control_index + 1..end).find(|index| tokens[*index].kind == SyntaxKind::LeftBrace)?;
    let body_close_index = static_stylesheet_matching_token_index(
        tokens,
        body_open_index,
        SyntaxKind::LeftBrace,
        SyntaxKind::RightBrace,
    )?;
    if body_close_index >= end {
        return None;
    }
    let header_start = static_stylesheet_token_end(&tokens[control_index]);
    let header_end = static_stylesheet_token_start(&tokens[body_open_index]);
    let header = source.get(header_start..header_end)?.trim().to_string();
    Some((header, body_open_index, body_close_index))
}

fn static_scss_else_if_condition(header: &str) -> Option<&str> {
    let trimmed = header.trim();
    let prefix = trimmed.get(..2)?;
    let rest = trimmed.get(2..)?;
    if !prefix.eq_ignore_ascii_case("if") || !rest.chars().next().is_some_and(char::is_whitespace) {
        return None;
    }
    Some(rest.trim()).filter(|condition| !condition.is_empty())
}

fn static_scss_branch_chain_condition(previous: &[String], current: &str) -> String {
    previous
        .iter()
        .map(|condition| format!("not ({condition})"))
        .chain(std::iter::once(current.to_string()))
        .collect::<Vec<_>>()
        .join(" and ")
}

fn static_scss_branch_chain_else_condition(previous: &[String]) -> Option<String> {
    (!previous.is_empty()).then(|| {
        previous
            .iter()
            .map(|condition| format!("not ({condition})"))
            .collect::<Vec<_>>()
            .join(" and ")
    })
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

fn collect_static_scss_mixin_include_calls(
    source: &str,
    tokens: &[LexedToken],
    declarations: &[StaticScssMixinDeclaration],
) -> Option<Vec<StaticScssMixinIncludeCall>> {
    let declaration_names = declarations
        .iter()
        .map(|declaration| canonical_static_scss_function_name(declaration.name.as_str()))
        .collect::<BTreeSet<_>>();
    let mut calls = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        let token = &tokens[index];
        if token.kind != SyntaxKind::AtKeyword || !token.text.eq_ignore_ascii_case("@include") {
            index += 1;
            continue;
        }
        let name_index = static_stylesheet_skip_trivia_tokens(tokens, index + 1);
        let name_token = tokens.get(name_index)?;
        if name_token.kind != SyntaxKind::Ident
            || !declaration_names.contains(&canonical_static_scss_function_name(
                name_token.text.as_str(),
            ))
        {
            index += 1;
            continue;
        }

        let after_name_index = static_stylesheet_skip_trivia_tokens(tokens, name_index + 1);
        let (arguments, after_arguments_index) = if tokens
            .get(after_name_index)
            .is_some_and(|candidate| candidate.kind == SyntaxKind::LeftParen)
        {
            let close_index = static_stylesheet_matching_token_index(
                tokens,
                after_name_index,
                SyntaxKind::LeftParen,
                SyntaxKind::RightParen,
            )?;
            let argument_text = source.get(
                static_stylesheet_token_end(&tokens[after_name_index])
                    ..static_stylesheet_token_start(&tokens[close_index]),
            )?;
            (
                split_static_scss_function_arguments(argument_text)?,
                static_stylesheet_skip_trivia_tokens(tokens, close_index + 1),
            )
        } else {
            (
                Vec::new(),
                static_stylesheet_skip_trivia_tokens(tokens, name_index + 1),
            )
        };
        let end_token = tokens.get(after_arguments_index)?;
        if end_token.kind != SyntaxKind::Semicolon {
            index += 1;
            continue;
        }
        calls.push(StaticScssMixinIncludeCall {
            name: name_token.text.clone(),
            start: static_stylesheet_token_start(token),
            end: static_stylesheet_token_end(end_token),
            arguments,
        });
        index = after_arguments_index + 1;
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

fn static_scss_function_call_is_inside_declaration_body(
    call: &StaticScssFunctionCall,
    declarations: &[StaticScssFunctionDeclaration],
) -> bool {
    declarations.iter().any(|declaration| {
        call.start >= declaration.body_start && call.start < declaration.body_end
    })
}

fn static_scss_function_call_is_inside_mixin_declaration_body(
    call: &StaticScssFunctionCall,
    declarations: &[StaticScssMixinDeclaration],
) -> bool {
    declarations.iter().any(|declaration| {
        call.start >= declaration.body_start && call.start < declaration.body_end
    })
}

fn static_scss_mixin_include_is_inside_declaration_body(
    call: &StaticScssMixinIncludeCall,
    declarations: &[StaticScssMixinDeclaration],
) -> bool {
    declarations.iter().any(|declaration| {
        call.start >= declaration.body_start && call.start < declaration.body_end
    })
}

fn static_scss_mixin_include_is_inside_function_declaration_body(
    call: &StaticScssMixinIncludeCall,
    declarations: &[StaticScssFunctionDeclaration],
) -> bool {
    declarations.iter().any(|declaration| {
        call.start >= declaration.body_start && call.start < declaration.body_end
    })
}

fn extend_static_scss_used_function_dependencies(
    used_declaration_names: &mut BTreeSet<String>,
    declarations: &[StaticScssFunctionDeclaration],
) {
    let declaration_names = declarations
        .iter()
        .map(|declaration| canonical_static_scss_function_name(declaration.name.as_str()))
        .collect::<BTreeSet<_>>();
    let mut changed = true;
    while changed {
        changed = false;
        for declaration in declarations {
            let declaration_name = canonical_static_scss_function_name(declaration.name.as_str());
            if !used_declaration_names.contains(&declaration_name) {
                continue;
            }
            for dependency_name in
                static_scss_function_dependency_names(declaration, &declaration_names)
            {
                if used_declaration_names.insert(dependency_name) {
                    changed = true;
                }
            }
        }
    }
}

fn static_scss_function_dependency_names(
    declaration: &StaticScssFunctionDeclaration,
    declaration_names: &BTreeSet<String>,
) -> Vec<String> {
    declaration
        .return_clauses
        .iter()
        .flat_map(|clause| {
            std::iter::once(clause.value.as_str()).chain(clause.condition.as_deref())
        })
        .chain(
            declaration
                .local_variables
                .iter()
                .map(|local_variable| local_variable.value.as_str()),
        )
        .flat_map(|value| static_scss_callable_names_in_value(value, declaration_names))
        .collect()
}

fn static_scss_callable_names_in_value(
    value: &str,
    declaration_names: &BTreeSet<String>,
) -> Vec<String> {
    let lexed = lex(value, StyleDialect::Scss);
    let tokens = lexed.tokens();
    tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            if token.kind != SyntaxKind::Ident || token.text.eq_ignore_ascii_case("if") {
                return None;
            }
            let canonical_name = canonical_static_scss_function_name(token.text.as_str());
            (declaration_names.contains(&canonical_name)
                && tokens
                    .get(static_stylesheet_skip_trivia_tokens(tokens, index + 1))
                    .is_some_and(|candidate| candidate.kind == SyntaxKind::LeftParen))
            .then_some(canonical_name)
        })
        .collect()
}

fn split_static_scss_function_arguments(
    arguments: &str,
) -> Option<Vec<StaticScssFunctionArgument>> {
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
                values.push(parse_static_scss_function_argument(
                    arguments.get(cursor..index)?.trim(),
                )?);
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
    values.push(parse_static_scss_function_argument(value)?);
    Some(values)
}

fn parse_static_scss_function_argument(value: &str) -> Option<StaticScssFunctionArgument> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if let Some((name, argument_value)) = split_static_scss_named_function_argument(value)? {
        if !static_stylesheet_variable_name_is_safe(name.as_str())
            || !static_scss_function_argument_is_safe(argument_value.as_str())
        {
            return None;
        }
        return Some(StaticScssFunctionArgument {
            name: Some(canonical_static_scss_variable_name(name.as_str())),
            value: argument_value,
        });
    }
    if !static_scss_function_argument_is_safe(value) {
        return None;
    }
    Some(StaticScssFunctionArgument {
        name: None,
        value: value.to_string(),
    })
}

fn split_static_scss_named_function_argument(value: &str) -> Option<Option<(String, String)>> {
    let colon_index = static_scss_top_level_colon_index(value)?;
    let Some(colon_index) = colon_index else {
        return Some(None);
    };
    let name = value.get(..colon_index)?.trim();
    let argument_value = value.get(colon_index + ':'.len_utf8()..)?.trim();
    let name = name.strip_prefix('$')?.trim();
    (!name.is_empty() && !argument_value.is_empty())
        .then(|| Some((name.to_string(), argument_value.to_string())))
}

fn static_scss_top_level_colon_index(value: &str) -> Option<Option<usize>> {
    let mut index = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
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
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.checked_sub(1)?,
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.checked_sub(1)?,
            ':' if paren_depth == 0 && bracket_depth == 0 => return Some(Some(index)),
            _ => {}
        }
        index += ch.len_utf8();
    }
    (quote.is_none() && paren_depth == 0 && bracket_depth == 0).then_some(None)
}

fn resolve_static_scss_function_call_abstract_value(
    call: &StaticScssFunctionCall,
    declarations: &[StaticScssFunctionDeclaration],
    mixin_declarations: &[StaticScssMixinDeclaration],
    scopes: &[StaticStylesheetScope],
    variable_declarations: &[StaticStylesheetScopedVariableDeclaration],
    fuel: usize,
) -> StaticStylesheetAbstractResolution {
    resolve_static_scss_function_call_abstract_value_with_stack(
        call,
        declarations,
        mixin_declarations,
        scopes,
        variable_declarations,
        fuel,
        &BTreeSet::new(),
    )
}

fn resolve_static_scss_function_call_abstract_value_with_stack(
    call: &StaticScssFunctionCall,
    declarations: &[StaticScssFunctionDeclaration],
    mixin_declarations: &[StaticScssMixinDeclaration],
    scopes: &[StaticStylesheetScope],
    variable_declarations: &[StaticStylesheetScopedVariableDeclaration],
    fuel: usize,
    active_functions: &BTreeSet<String>,
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
    let canonical_declaration_name = canonical_static_scss_function_name(declaration.name.as_str());
    if active_functions.contains(&canonical_declaration_name) {
        return top_static_abstract_value(StaticStylesheetResolutionReason::Cycle);
    }
    let mut next_active_functions = active_functions.clone();
    next_active_functions.insert(canonical_declaration_name);
    let context = StaticScssFunctionResolutionContext {
        declarations,
        mixin_declarations,
        scopes,
        variable_declarations,
        active_functions: &next_active_functions,
    };
    let Some(bound_arguments) = bind_static_scss_function_arguments(declaration, call) else {
        return raw_static_abstract_value(
            call.name.as_str(),
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        );
    };
    let mut argument_values = BTreeMap::new();
    for (parameter, argument) in bound_arguments {
        let resolution = resolve_static_scss_function_argument_abstract_value(
            argument.as_str(),
            &argument_values,
            call.start,
            fuel - 1,
            context,
        );
        let Some(rendered_value) = resolution.rendered_value else {
            return top_static_abstract_value(resolution.reason);
        };
        if resolution.outcome == StaticStylesheetResolutionOutcome::Top {
            return top_static_abstract_value(resolution.reason);
        }
        argument_values.insert(parameter, rendered_value);
    }

    resolve_static_scss_function_return_abstract_value(
        declaration,
        &argument_values,
        fuel - 1,
        context,
    )
}

fn bind_static_scss_function_local_variables_before(
    declaration: &StaticScssFunctionDeclaration,
    argument_values: &BTreeMap<String, String>,
    position: usize,
    fuel: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Result<BTreeMap<String, String>, StaticStylesheetAbstractResolution> {
    let mut bound_values = argument_values.clone();
    for local_variable in declaration.local_variables.iter().filter(|local_variable| {
        local_variable.span_start < position
            && local_variable.scope_start <= position
            && position < local_variable.scope_end
    }) {
        if static_scss_function_value_contains_callable_to(
            local_variable.value.as_str(),
            declaration.name.as_str(),
        ) {
            return Err(top_static_abstract_value(
                StaticStylesheetResolutionReason::Cycle,
            ));
        }
        let resolution = resolve_static_scss_function_value_with_bindings(
            local_variable.value.as_str(),
            &bound_values,
            local_variable.span_start,
            fuel,
            context,
        );
        if resolution.outcome == StaticStylesheetResolutionOutcome::Top {
            return Err(top_static_abstract_value(resolution.reason));
        }
        let Some(rendered_value) = resolution.rendered_value else {
            return Err(top_static_abstract_value(resolution.reason));
        };
        bound_values.insert(local_variable.name.clone(), rendered_value);
    }
    Ok(bound_values)
}

fn bind_static_scss_function_local_variables_in_range(
    declaration: &StaticScssFunctionDeclaration,
    argument_values: &BTreeMap<String, String>,
    range_start: usize,
    position: usize,
    fuel: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Result<BTreeMap<String, String>, StaticStylesheetAbstractResolution> {
    let mut bound_values = argument_values.clone();
    for local_variable in declaration.local_variables.iter().filter(|local_variable| {
        local_variable.span_start >= range_start
            && local_variable.span_start < position
            && local_variable.scope_start <= position
            && position < local_variable.scope_end
    }) {
        if static_scss_function_value_contains_callable_to(
            local_variable.value.as_str(),
            declaration.name.as_str(),
        ) {
            return Err(top_static_abstract_value(
                StaticStylesheetResolutionReason::Cycle,
            ));
        }
        let resolution = resolve_static_scss_function_value_with_bindings(
            local_variable.value.as_str(),
            &bound_values,
            local_variable.span_start,
            fuel,
            context,
        );
        if resolution.outcome == StaticStylesheetResolutionOutcome::Top {
            return Err(top_static_abstract_value(resolution.reason));
        }
        let Some(rendered_value) = resolution.rendered_value else {
            return Err(top_static_abstract_value(resolution.reason));
        };
        bound_values.insert(local_variable.name.clone(), rendered_value);
    }
    Ok(bound_values)
}

fn bind_static_scss_function_arguments(
    declaration: &StaticScssFunctionDeclaration,
    call: &StaticScssFunctionCall,
) -> Option<Vec<(String, String)>> {
    bind_static_scss_callable_arguments(&declaration.parameters, &call.arguments)
}

fn bind_static_scss_mixin_arguments(
    declaration: &StaticScssMixinDeclaration,
    call: &StaticScssMixinIncludeCall,
) -> Option<Vec<(String, String)>> {
    bind_static_scss_callable_arguments(&declaration.parameters, &call.arguments)
}

fn render_static_scss_mixin_include_body(
    source: &str,
    tokens: &[LexedToken],
    declaration: &StaticScssMixinDeclaration,
    call: &StaticScssMixinIncludeCall,
    call_position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<StaticScssMixinRenderResult> {
    let mut active_mixins = BTreeSet::new();
    render_static_scss_mixin_include_body_with_active(
        source,
        tokens,
        declaration,
        call,
        call_position,
        context,
        &mut active_mixins,
    )
}

fn render_static_scss_mixin_include_body_with_active(
    source: &str,
    tokens: &[LexedToken],
    declaration: &StaticScssMixinDeclaration,
    call: &StaticScssMixinIncludeCall,
    call_position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
    active_mixins: &mut BTreeSet<String>,
) -> Option<StaticScssMixinRenderResult> {
    let canonical_name = canonical_static_scss_function_name(declaration.name.as_str());
    if !active_mixins.insert(canonical_name.clone()) {
        return None;
    }
    let body = source.get(declaration.body_start..declaration.body_end)?;
    if !static_scss_mixin_body_is_static_declaration_subset(body) {
        return None;
    }
    let mut argument_values = BTreeMap::new();
    for (parameter, argument) in bind_static_scss_mixin_arguments(declaration, call)? {
        let resolution = resolve_static_scss_function_argument_abstract_value(
            argument.as_str(),
            &argument_values,
            call_position,
            STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
            context,
        );
        if resolution.outcome != StaticStylesheetResolutionOutcome::Resolved {
            return None;
        }
        let rendered_value = resolution.rendered_value?;
        argument_values.insert(parameter, rendered_value);
    }

    let body =
        render_static_scss_mixin_body_variables(body, call_position, &argument_values, context)?;
    let nested = render_static_scss_mixin_body_nested_includes(
        body.as_str(),
        source,
        tokens,
        call_position,
        context,
        active_mixins,
    )?;
    let body = resolve_static_scss_mixin_body_declaration_values(
        nested.body.as_str(),
        call_position,
        context,
    )?;
    let mut used_mixin_declaration_names = nested.used_mixin_declaration_names;
    let mut used_function_declaration_names = nested.used_function_declaration_names;
    used_mixin_declaration_names.insert(canonical_name.clone());
    used_function_declaration_names.extend(
        collect_static_scss_resolved_function_names_in_mixin_body(
            source,
            tokens,
            context.declarations,
            declaration,
            body.as_str(),
        )?,
    );
    active_mixins.remove(&canonical_name);
    Some(StaticScssMixinRenderResult {
        body,
        used_mixin_declaration_names,
        used_function_declaration_names,
    })
}

fn render_static_scss_mixin_body_nested_includes(
    body: &str,
    source: &str,
    tokens: &[LexedToken],
    call_position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
    active_mixins: &mut BTreeSet<String>,
) -> Option<StaticScssMixinRenderResult> {
    let body_lexed = lex(body, StyleDialect::Scss);
    let calls = collect_static_scss_mixin_include_calls(
        body,
        body_lexed.tokens(),
        context.mixin_declarations,
    )?;
    if calls.is_empty() {
        return Some(StaticScssMixinRenderResult {
            body: body.to_string(),
            used_mixin_declaration_names: BTreeSet::new(),
            used_function_declaration_names: BTreeSet::new(),
        });
    }

    let mut edits = Vec::new();
    let mut used_mixin_declaration_names = BTreeSet::new();
    let mut used_function_declaration_names = BTreeSet::new();
    for call in calls {
        let Some(declaration) = context.mixin_declarations.iter().find(|declaration| {
            canonical_static_scss_function_name(declaration.name.as_str())
                == canonical_static_scss_function_name(call.name.as_str())
        }) else {
            continue;
        };
        let rendered = render_static_scss_mixin_include_body_with_active(
            source,
            tokens,
            declaration,
            &call,
            call_position,
            context,
            active_mixins,
        )?;
        used_mixin_declaration_names.extend(rendered.used_mixin_declaration_names);
        used_function_declaration_names.extend(rendered.used_function_declaration_names);
        edits.push(StaticStylesheetEvaluationEdit {
            start: call.start,
            end: call.end,
            replacement: rendered.body,
        });
    }

    Some(StaticScssMixinRenderResult {
        body: apply_static_stylesheet_evaluation_edits(body, edits)?,
        used_mixin_declaration_names,
        used_function_declaration_names,
    })
}

fn render_static_scss_mixin_body_variables(
    body: &str,
    call_position: usize,
    argument_values: &BTreeMap<String, String>,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<String> {
    let local_declarations = collect_static_scss_mixin_body_local_declarations(body)?;
    let local_declaration_ranges = local_declarations
        .iter()
        .flat_map(|declaration| declaration.declaration.removal_spans.iter().copied())
        .collect::<Vec<_>>();
    let mut scoped_values = argument_values.clone();
    let mut edits = local_declarations
        .iter()
        .flat_map(|declaration| {
            declaration
                .declaration
                .removal_spans
                .iter()
                .map(|(start, end)| StaticStylesheetEvaluationEdit {
                    start: *start,
                    end: *end,
                    replacement: String::new(),
                })
        })
        .collect::<Vec<_>>();

    for local in &local_declarations {
        if local.declaration.is_default || local.declaration.is_global {
            return None;
        }
        let resolution = resolve_static_scss_function_value_with_bindings(
            local.declaration.value.as_str(),
            &scoped_values,
            call_position,
            STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
            context,
        );
        if resolution.outcome != StaticStylesheetResolutionOutcome::Resolved {
            return None;
        }
        scoped_values.insert(
            canonical_static_scss_variable_name(local.name.as_str()),
            resolution.rendered_value?,
        );
    }

    let references = collect_static_stylesheet_variable_references_with_options(
        body,
        StaticStylesheetVariableKind::Scss,
        true,
    )?;
    for reference in references {
        if static_stylesheet_position_is_inside_ranges(reference.start, &local_declaration_ranges) {
            continue;
        }
        let canonical_name = canonical_static_scss_variable_name(reference.name.as_str());
        let replacement = if let Some(value) = scoped_values.get(canonical_name.as_str()) {
            value.clone()
        } else {
            let mut stack = BTreeSet::new();
            resolve_static_scss_variable_value_at_position(
                reference.name.as_str(),
                call_position,
                context.scopes,
                context.variable_declarations,
                &mut stack,
            )?
        };
        edits.push(StaticStylesheetEvaluationEdit {
            start: reference.start,
            end: reference.end,
            replacement,
        });
    }
    apply_static_stylesheet_evaluation_edits(body, edits)
}

fn collect_static_scss_mixin_body_local_declarations(
    body: &str,
) -> Option<Vec<StaticScssMixinBodyLocalDeclaration>> {
    let facts = omena_parser::collect_style_facts(body, StyleDialect::Scss);
    let mut declarations = Vec::new();
    for fact in facts
        .variables
        .iter()
        .filter(|fact| fact.kind == ParsedVariableFactKind::ScssDeclaration)
    {
        let start = parser_text_size_to_usize(fact.range.start().into());
        let end = parser_text_size_to_usize(fact.range.end().into());
        if static_stylesheet_variable_reference_is_named_argument_label(body, start, end) {
            continue;
        }
        let declaration = extract_static_stylesheet_variable_declaration(
            body,
            start,
            end,
            StaticStylesheetVariableKind::Scss,
        )?;
        if !static_stylesheet_scss_declaration_value_is_removal_safe(&declaration.value) {
            return None;
        }
        declarations.push(StaticScssMixinBodyLocalDeclaration {
            name: fact.name.clone(),
            declaration,
        });
    }
    declarations.sort_by_key(|declaration| declaration.declaration.span_start);
    Some(declarations)
}

fn collect_static_less_mixin_evaluation_edits(
    source: &str,
    tokens: &[LexedToken],
    declarations: &[StaticLessMixinDeclaration],
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
    scopes: &[StaticStylesheetScope],
    variable_declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    excluded_call_ranges: &[(usize, usize)],
) -> Option<Vec<StaticStylesheetEvaluationEdit>> {
    let calls = collect_static_less_mixin_calls(source, tokens)?;
    if calls.is_empty() {
        return Some(Vec::new());
    }

    let declaration_ranges = static_less_mixin_declaration_ranges_from_declarations(declarations);
    let empty_captured_values = BTreeMap::new();
    let context = StaticLessMixinRenderContext {
        source,
        declarations,
        detached_ruleset_declarations,
        scopes,
        variable_declarations,
        captured_values: &empty_captured_values,
    };
    let mut edits = Vec::new();
    let mut used_declaration_names = BTreeSet::new();
    for call in calls.iter().filter(|call| {
        !static_stylesheet_position_is_inside_ranges(call.start, &declaration_ranges)
            && !static_stylesheet_position_is_inside_ranges(call.start, excluded_call_ranges)
    }) {
        let call_scope_id = static_stylesheet_scope_for_position(scopes, call.start)?;
        let mut active_mixins = BTreeSet::new();
        let Some(rendered) =
            render_static_less_mixin_call(call, call_scope_id, context, &mut active_mixins)?
        else {
            continue;
        };
        used_declaration_names.extend(rendered.used_declaration_names);
        edits.push(StaticStylesheetEvaluationEdit {
            start: call.start,
            end: call.end,
            replacement: rendered.body,
        });
    }

    for declaration in declarations.iter().filter(|declaration| {
        used_declaration_names
            .contains(&canonical_static_less_mixin_name(declaration.name.as_str()))
    }) {
        edits.push(StaticStylesheetEvaluationEdit {
            start: declaration.span_start,
            end: declaration.span_end,
            replacement: String::new(),
        });
    }
    Some(edits)
}

fn collect_static_less_mixin_accessor_evaluation_edits(
    source: &str,
    tokens: &[LexedToken],
    declarations: &[StaticLessMixinDeclaration],
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
    scopes: &[StaticStylesheetScope],
    variable_declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    excluded_ranges: &[(usize, usize)],
) -> Option<Vec<StaticStylesheetEvaluationEdit>> {
    let accessors = collect_static_less_mixin_accessors(source, tokens)?;
    if accessors.is_empty() {
        return Some(Vec::new());
    }

    let declaration_ranges = static_less_mixin_declaration_ranges_from_declarations(declarations);
    let empty_captured_values = BTreeMap::new();
    let context = StaticLessMixinRenderContext {
        source,
        declarations,
        detached_ruleset_declarations,
        scopes,
        variable_declarations,
        captured_values: &empty_captured_values,
    };
    let mut edits = Vec::new();
    let mut used_declaration_names = BTreeSet::new();
    for accessor in accessors.iter().filter(|accessor| {
        !static_stylesheet_position_is_inside_ranges(accessor.start, &declaration_ranges)
            && !static_stylesheet_position_is_inside_ranges(accessor.start, excluded_ranges)
    }) {
        let call_scope_id = static_stylesheet_scope_for_position(scopes, accessor.start)?;
        let rendered = render_static_less_mixin_accessor(accessor, call_scope_id, context)?;
        let rendered = rendered?;
        used_declaration_names.insert(rendered.used_declaration_name);
        edits.push(StaticStylesheetEvaluationEdit {
            start: accessor.start,
            end: accessor.end,
            replacement: rendered.value,
        });
    }

    for declaration in declarations.iter().filter(|declaration| {
        used_declaration_names
            .contains(&canonical_static_less_mixin_name(declaration.name.as_str()))
    }) {
        edits.push(StaticStylesheetEvaluationEdit {
            start: declaration.span_start,
            end: declaration.span_end,
            replacement: String::new(),
        });
    }
    Some(edits)
}

fn collect_static_less_detached_ruleset_evaluation_edits(
    source: &str,
    declarations: &[StaticLessDetachedRulesetDeclaration],
    calls: &[StaticLessDetachedRulesetCall],
    mixin_declarations: &[StaticLessMixinDeclaration],
    mixin_declaration_ranges: &[(usize, usize)],
    scopes: &[StaticStylesheetScope],
    variable_declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
) -> Option<Vec<StaticStylesheetEvaluationEdit>> {
    let declaration_ranges = static_less_detached_ruleset_ranges_from_declarations(declarations);
    let mut edits = declarations
        .iter()
        .map(|declaration| StaticStylesheetEvaluationEdit {
            start: declaration.span_start,
            end: declaration.span_end,
            replacement: String::new(),
        })
        .collect::<Vec<_>>();
    let mut used_mixin_declaration_names = BTreeSet::new();

    for call in calls.iter().filter(|call| {
        !static_stylesheet_position_is_inside_ranges(call.start, &declaration_ranges)
            && !static_stylesheet_position_is_inside_ranges(call.start, mixin_declaration_ranges)
    }) {
        let call_scope_id = static_stylesheet_scope_for_position(scopes, call.start)?;
        let declaration = find_static_less_detached_ruleset_declaration(
            call.name.as_str(),
            call_scope_id,
            scopes,
            declarations,
        )?;
        let replacement = render_static_less_detached_ruleset_body(
            source,
            declaration,
            call_scope_id,
            scopes,
            variable_declarations,
            mixin_declarations,
            declarations,
        )?;
        used_mixin_declaration_names.extend(replacement.used_declaration_names);
        edits.push(StaticStylesheetEvaluationEdit {
            start: call.start,
            end: call.end,
            replacement: replacement.body,
        });
    }
    for declaration in mixin_declarations.iter().filter(|declaration| {
        used_mixin_declaration_names
            .contains(&canonical_static_less_mixin_name(declaration.name.as_str()))
    }) {
        edits.push(StaticStylesheetEvaluationEdit {
            start: declaration.span_start,
            end: declaration.span_end,
            replacement: String::new(),
        });
    }
    Some(edits)
}

fn collect_static_less_detached_ruleset_accessor_evaluation_edits(
    source: &str,
    declarations: &[StaticLessDetachedRulesetDeclaration],
    accessors: &[StaticLessDetachedRulesetAccessor],
    mixin_declaration_ranges: &[(usize, usize)],
    scopes: &[StaticStylesheetScope],
    variable_declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
) -> Option<Vec<StaticStylesheetEvaluationEdit>> {
    if accessors.is_empty() {
        return Some(Vec::new());
    }

    let declaration_ranges = static_less_detached_ruleset_ranges_from_declarations(declarations);
    let mut edits = Vec::new();
    for accessor in accessors.iter().filter(|accessor| {
        !static_stylesheet_position_is_inside_ranges(accessor.start, &declaration_ranges)
            && !static_stylesheet_position_is_inside_ranges(
                accessor.start,
                mixin_declaration_ranges,
            )
    }) {
        let call_scope_id = static_stylesheet_scope_for_position(scopes, accessor.start)?;
        let declaration = find_static_less_detached_ruleset_declaration(
            accessor.name.as_str(),
            call_scope_id,
            scopes,
            declarations,
        )?;
        let replacement = render_static_less_detached_ruleset_accessor(
            source,
            declaration,
            accessor.member.as_str(),
            call_scope_id,
            scopes,
            variable_declarations,
            declarations,
        )?;
        edits.push(StaticStylesheetEvaluationEdit {
            start: accessor.start,
            end: accessor.end,
            replacement,
        });
    }
    Some(edits)
}

fn render_static_less_detached_ruleset_body(
    source: &str,
    declaration: &StaticLessDetachedRulesetDeclaration,
    call_scope_id: usize,
    scopes: &[StaticStylesheetScope],
    variable_declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    mixin_declarations: &[StaticLessMixinDeclaration],
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
) -> Option<StaticLessMixinRenderResult> {
    let body = source.get(declaration.body_start..declaration.body_end)?;
    if !static_less_mixin_body_is_static_declaration_subset(body) {
        return None;
    }
    let body_lexed = lex(body, StyleDialect::Less);
    if !collect_static_less_detached_ruleset_calls(body, body_lexed.tokens())?.is_empty() {
        return None;
    }
    let empty_arguments = BTreeMap::new();
    let empty_captured_values = BTreeMap::new();
    let body = render_static_less_mixin_body_variables(
        body,
        call_scope_id,
        &empty_arguments,
        &empty_captured_values,
        scopes,
        variable_declarations,
        detached_ruleset_declarations,
    )?;
    let context = StaticLessMixinRenderContext {
        source,
        declarations: mixin_declarations,
        detached_ruleset_declarations,
        scopes,
        variable_declarations,
        captured_values: &empty_captured_values,
    };
    let mut active_mixins = BTreeSet::new();
    let nested = render_static_less_mixin_body_nested_calls(
        body.as_str(),
        call_scope_id,
        context,
        &mut active_mixins,
    )?;
    let nested_lexed = lex(nested.body.as_str(), StyleDialect::Less);
    if !collect_static_less_mixin_calls(nested.body.as_str(), nested_lexed.tokens())?.is_empty()
        || !collect_static_less_detached_ruleset_calls(nested.body.as_str(), nested_lexed.tokens())?
            .is_empty()
    {
        return None;
    }
    Some(StaticLessMixinRenderResult {
        body: resolve_static_less_mixin_body_declaration_values(nested.body.as_str())?,
        used_declaration_names: nested.used_declaration_names,
    })
}

fn render_static_less_detached_ruleset_accessor(
    source: &str,
    declaration: &StaticLessDetachedRulesetDeclaration,
    member: &str,
    call_scope_id: usize,
    scopes: &[StaticStylesheetScope],
    variable_declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
) -> Option<String> {
    let body = source.get(declaration.body_start..declaration.body_end)?;
    if !static_less_mixin_body_is_static_declaration_subset(body) {
        return None;
    }
    let body_lexed = lex(body, StyleDialect::Less);
    if !collect_static_less_mixin_calls(body, body_lexed.tokens())?.is_empty()
        || !collect_static_less_detached_ruleset_calls(body, body_lexed.tokens())?.is_empty()
    {
        return None;
    }

    let empty_values = BTreeMap::new();
    let empty_mixin_declarations = [];
    let context = StaticLessMixinRenderContext {
        source,
        declarations: &empty_mixin_declarations,
        detached_ruleset_declarations,
        scopes,
        variable_declarations,
        captured_values: &empty_values,
    };
    let scoped_values = static_less_mixin_body_scoped_values(
        body,
        call_scope_id,
        &empty_values,
        &empty_values,
        scopes,
        variable_declarations,
        detached_ruleset_declarations,
    )?;
    if static_less_variable_name_is_safe(member) {
        return scoped_values.get(member).cloned();
    }
    static_less_body_property_value(body, member, &scoped_values, call_scope_id, context)
}

fn find_static_less_detached_ruleset_declaration<'a>(
    name: &str,
    mut scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &'a [StaticLessDetachedRulesetDeclaration],
) -> Option<&'a StaticLessDetachedRulesetDeclaration> {
    loop {
        if let Some(declaration) = declarations
            .iter()
            .rev()
            .find(|declaration| declaration.name == name && declaration.scope_id == scope_id)
        {
            return Some(declaration);
        }
        scope_id = scopes.get(scope_id)?.parent_id?;
    }
}

fn static_less_detached_ruleset_ranges_from_declarations(
    declarations: &[StaticLessDetachedRulesetDeclaration],
) -> Vec<(usize, usize)> {
    declarations
        .iter()
        .map(|declaration| (declaration.span_start, declaration.span_end))
        .collect()
}

fn static_less_detached_ruleset_ranges_from_calls(
    calls: &[StaticLessDetachedRulesetCall],
) -> Vec<(usize, usize)> {
    calls.iter().map(|call| (call.start, call.end)).collect()
}

fn static_less_detached_ruleset_ranges_from_accessors(
    accessors: &[StaticLessDetachedRulesetAccessor],
) -> Vec<(usize, usize)> {
    accessors
        .iter()
        .map(|accessor| (accessor.start, accessor.end))
        .collect()
}

fn static_less_mixin_ranges_from_calls(calls: &[StaticLessMixinCall]) -> Vec<(usize, usize)> {
    calls.iter().map(|call| (call.start, call.end)).collect()
}

fn static_less_mixin_accessor_ranges_from_accessors(
    accessors: &[StaticLessMixinAccessor],
) -> Vec<(usize, usize)> {
    accessors
        .iter()
        .map(|accessor| (accessor.start, accessor.end))
        .collect()
}

fn collect_static_less_mixin_declarations(
    source: &str,
    tokens: &[LexedToken],
) -> Option<Vec<StaticLessMixinDeclaration>> {
    let mut declarations = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        let Some((name, open_index)) = static_less_mixin_signature_at(tokens, index) else {
            index += 1;
            continue;
        };
        let close_index = static_stylesheet_matching_token_index(
            tokens,
            open_index,
            SyntaxKind::LeftParen,
            SyntaxKind::RightParen,
        )?;
        let Some(body_open_index) =
            static_stylesheet_next_token_kind_index(tokens, close_index + 1, SyntaxKind::LeftBrace)
        else {
            index += 1;
            continue;
        };
        let guard =
            static_less_mixin_header_guard_text(source, tokens, close_index + 1, body_open_index)?;
        let Some(body_close_index) = static_stylesheet_matching_token_index(
            tokens,
            body_open_index,
            SyntaxKind::LeftBrace,
            SyntaxKind::RightBrace,
        ) else {
            index += 1;
            continue;
        };
        let parameters =
            collect_static_less_mixin_parameters(source, tokens, open_index + 1, close_index)?;
        declarations.push(StaticLessMixinDeclaration {
            name,
            parameters,
            guard,
            span_start: static_stylesheet_token_start(&tokens[index]),
            span_end: static_stylesheet_token_end(&tokens[body_close_index]),
            body_start: static_stylesheet_token_end(&tokens[body_open_index]),
            body_end: static_stylesheet_token_start(&tokens[body_close_index]),
        });
        index = body_close_index + 1;
    }
    Some(declarations)
}

fn collect_static_less_mixin_calls(
    source: &str,
    tokens: &[LexedToken],
) -> Option<Vec<StaticLessMixinCall>> {
    let mut calls = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        if static_less_mixin_call_context_is_plain(tokens, index)
            && let Some((call, semicolon_index)) =
                static_less_namespace_mixin_call_at(source, tokens, index)
        {
            calls.push(call);
            index = semicolon_index + 1;
            continue;
        }
        let Some((name, open_index)) = static_less_mixin_signature_at(tokens, index) else {
            index += 1;
            continue;
        };
        if !static_less_mixin_call_context_is_plain(tokens, index) {
            index += 1;
            continue;
        }
        let close_index = static_stylesheet_matching_token_index(
            tokens,
            open_index,
            SyntaxKind::LeftParen,
            SyntaxKind::RightParen,
        )?;
        let Some((semicolon_index, important)) =
            static_less_mixin_call_semicolon_and_importance(source, tokens, close_index)
        else {
            index += 1;
            continue;
        };
        let arguments = split_static_less_mixin_arguments(source.get(
            static_stylesheet_token_end(&tokens[open_index])
                ..static_stylesheet_token_start(&tokens[close_index]),
        )?)?;
        calls.push(StaticLessMixinCall {
            namespace: None,
            namespace_arguments: Vec::new(),
            name,
            start: static_stylesheet_token_start(&tokens[index]),
            end: static_stylesheet_token_end(&tokens[semicolon_index]),
            important,
            arguments,
        });
        index = semicolon_index + 1;
    }
    calls.sort_by_key(|call| (call.start, call.end));
    Some(calls)
}

fn collect_static_less_mixin_accessors(
    source: &str,
    tokens: &[LexedToken],
) -> Option<Vec<StaticLessMixinAccessor>> {
    let mut accessors = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        let Some((name, open_index)) = static_less_mixin_signature_at(tokens, index) else {
            index += 1;
            continue;
        };
        let close_index = static_stylesheet_matching_token_index(
            tokens,
            open_index,
            SyntaxKind::LeftParen,
            SyntaxKind::RightParen,
        )?;
        let bracket_open_index = static_stylesheet_skip_trivia_tokens(tokens, close_index + 1);
        if tokens
            .get(bracket_open_index)
            .is_none_or(|token| token.kind != SyntaxKind::LeftBracket)
        {
            index += 1;
            continue;
        }
        let bracket_close_index = static_stylesheet_matching_token_index(
            tokens,
            bracket_open_index,
            SyntaxKind::LeftBracket,
            SyntaxKind::RightBracket,
        )?;
        let arguments = split_static_less_mixin_arguments(source.get(
            static_stylesheet_token_end(&tokens[open_index])
                ..static_stylesheet_token_start(&tokens[close_index]),
        )?)?;
        let member = static_less_mixin_accessor_member(source.get(
            static_stylesheet_token_end(&tokens[bracket_open_index])
                ..static_stylesheet_token_start(&tokens[bracket_close_index]),
        )?)?;
        accessors.push(StaticLessMixinAccessor {
            name,
            member,
            start: static_stylesheet_token_start(&tokens[index]),
            end: static_stylesheet_token_end(&tokens[bracket_close_index]),
            arguments,
        });
        index = bracket_close_index + 1;
    }
    accessors.sort_by_key(|accessor| (accessor.start, accessor.end));
    Some(accessors)
}

fn static_less_mixin_accessor_member(member: &str) -> Option<String> {
    let member = member.trim();
    if static_less_variable_name_is_safe(member) || static_stylesheet_property_name_is_safe(member)
    {
        return Some(member.to_string());
    }
    None
}

fn collect_static_less_detached_ruleset_declarations(
    source: &str,
    tokens: &[LexedToken],
    scopes: &[StaticStylesheetScope],
) -> Option<Vec<StaticLessDetachedRulesetDeclaration>> {
    let mut declarations = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        let token = tokens.get(index)?;
        if token.kind != SyntaxKind::LessVariable {
            index += 1;
            continue;
        }
        if !static_less_variable_name_is_safe(token.text.as_str()) {
            return None;
        }
        let colon_index = static_stylesheet_skip_trivia_tokens(tokens, index + 1);
        if tokens
            .get(colon_index)
            .is_none_or(|candidate| candidate.kind != SyntaxKind::Colon)
        {
            index += 1;
            continue;
        }
        let body_open_index = static_stylesheet_skip_trivia_tokens(tokens, colon_index + 1);
        if tokens
            .get(body_open_index)
            .is_none_or(|candidate| candidate.kind != SyntaxKind::LeftBrace)
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
        let semicolon_index = static_stylesheet_skip_trivia_tokens(tokens, body_close_index + 1);
        if tokens
            .get(semicolon_index)
            .is_none_or(|candidate| candidate.kind != SyntaxKind::Semicolon)
        {
            return None;
        }
        let body_start = static_stylesheet_token_end(&tokens[body_open_index]);
        let body_end = static_stylesheet_token_start(&tokens[body_close_index]);
        let body = source.get(body_start..body_end)?;
        if !static_less_mixin_body_is_static_declaration_subset(body) {
            return None;
        }
        let span_start = static_stylesheet_token_start(token);
        declarations.push(StaticLessDetachedRulesetDeclaration {
            name: token.text.clone(),
            scope_id: static_stylesheet_scope_for_position(scopes, span_start)?,
            span_start,
            span_end: static_stylesheet_token_end(&tokens[semicolon_index]),
            body_start,
            body_end,
        });
        index = semicolon_index + 1;
    }
    Some(declarations)
}

fn collect_static_less_detached_ruleset_calls(
    source: &str,
    tokens: &[LexedToken],
) -> Option<Vec<StaticLessDetachedRulesetCall>> {
    let mut calls = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        let Some((call, semicolon_index)) =
            static_less_detached_ruleset_call_at(source, tokens, index)
        else {
            index += 1;
            continue;
        };
        calls.push(call);
        index = semicolon_index + 1;
    }
    calls.sort_by_key(|call| (call.start, call.end));
    Some(calls)
}

fn collect_static_less_detached_ruleset_accessors(
    source: &str,
    tokens: &[LexedToken],
) -> Option<Vec<StaticLessDetachedRulesetAccessor>> {
    let mut accessors = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        let token = &tokens[index];
        if token.kind != SyntaxKind::LessVariable
            || !static_less_variable_name_is_safe(token.text.as_str())
        {
            index += 1;
            continue;
        }
        let bracket_open_index = static_stylesheet_skip_trivia_tokens(tokens, index + 1);
        if tokens
            .get(bracket_open_index)
            .is_none_or(|candidate| candidate.kind != SyntaxKind::LeftBracket)
        {
            index += 1;
            continue;
        }
        let bracket_close_index = static_stylesheet_matching_token_index(
            tokens,
            bracket_open_index,
            SyntaxKind::LeftBracket,
            SyntaxKind::RightBracket,
        )?;
        let member = static_less_mixin_accessor_member(source.get(
            static_stylesheet_token_end(&tokens[bracket_open_index])
                ..static_stylesheet_token_start(&tokens[bracket_close_index]),
        )?)?;
        accessors.push(StaticLessDetachedRulesetAccessor {
            name: token.text.clone(),
            member,
            start: static_stylesheet_token_start(token),
            end: static_stylesheet_token_end(&tokens[bracket_close_index]),
        });
        index = bracket_close_index + 1;
    }
    accessors.sort_by_key(|accessor| (accessor.start, accessor.end));
    Some(accessors)
}

fn static_less_detached_ruleset_call_at(
    source: &str,
    tokens: &[LexedToken],
    index: usize,
) -> Option<(StaticLessDetachedRulesetCall, usize)> {
    let token = tokens.get(index)?;
    if token.kind != SyntaxKind::LessVariable
        || !static_less_variable_name_is_safe(token.text.as_str())
        || !static_less_mixin_call_context_is_plain(tokens, index)
    {
        return None;
    }
    let open_index = static_stylesheet_skip_trivia_tokens(tokens, index + 1);
    if tokens
        .get(open_index)
        .is_none_or(|candidate| candidate.kind != SyntaxKind::LeftParen)
    {
        return None;
    }
    let close_index = static_stylesheet_matching_token_index(
        tokens,
        open_index,
        SyntaxKind::LeftParen,
        SyntaxKind::RightParen,
    )?;
    let argument_text = source.get(
        static_stylesheet_token_end(&tokens[open_index])
            ..static_stylesheet_token_start(&tokens[close_index]),
    )?;
    if !argument_text.trim().is_empty() {
        return None;
    }
    let semicolon_index = static_stylesheet_skip_trivia_tokens(tokens, close_index + 1);
    if tokens
        .get(semicolon_index)
        .is_none_or(|candidate| candidate.kind != SyntaxKind::Semicolon)
    {
        return None;
    }
    Some((
        StaticLessDetachedRulesetCall {
            name: token.text.clone(),
            start: static_stylesheet_token_start(token),
            end: static_stylesheet_token_end(&tokens[semicolon_index]),
        },
        semicolon_index,
    ))
}

fn static_less_namespace_mixin_call_at(
    source: &str,
    tokens: &[LexedToken],
    index: usize,
) -> Option<(StaticLessMixinCall, usize)> {
    let (namespace, after_namespace_index) = static_less_namespace_name_at(tokens, index)?;
    let namespace_arguments_index =
        static_stylesheet_skip_trivia_tokens(tokens, after_namespace_index);
    let (namespace_arguments, separator_index) = if tokens
        .get(namespace_arguments_index)
        .is_some_and(|token| token.kind == SyntaxKind::LeftParen)
    {
        let namespace_arguments_close_index = static_stylesheet_matching_token_index(
            tokens,
            namespace_arguments_index,
            SyntaxKind::LeftParen,
            SyntaxKind::RightParen,
        )?;
        let arguments = split_static_less_mixin_arguments(source.get(
            static_stylesheet_token_end(&tokens[namespace_arguments_index])
                ..static_stylesheet_token_start(&tokens[namespace_arguments_close_index]),
        )?)?;
        (
            arguments,
            static_stylesheet_skip_trivia_tokens(tokens, namespace_arguments_close_index + 1),
        )
    } else {
        (Vec::new(), namespace_arguments_index)
    };
    if tokens
        .get(separator_index)
        .is_none_or(|token| token.kind != SyntaxKind::GreaterThan)
    {
        return None;
    }
    let call_index = static_stylesheet_skip_trivia_tokens(tokens, separator_index + 1);
    let (name, open_index) = static_less_mixin_signature_at(tokens, call_index)?;
    let close_index = static_stylesheet_matching_token_index(
        tokens,
        open_index,
        SyntaxKind::LeftParen,
        SyntaxKind::RightParen,
    )?;
    let (semicolon_index, important) =
        static_less_mixin_call_semicolon_and_importance(source, tokens, close_index)?;
    let arguments = split_static_less_mixin_arguments(source.get(
        static_stylesheet_token_end(&tokens[open_index])
            ..static_stylesheet_token_start(&tokens[close_index]),
    )?)?;
    Some((
        StaticLessMixinCall {
            namespace: Some(namespace),
            namespace_arguments,
            name,
            start: static_stylesheet_token_start(&tokens[index]),
            end: static_stylesheet_token_end(&tokens[semicolon_index]),
            important,
            arguments,
        },
        semicolon_index,
    ))
}

fn static_less_namespace_name_at(tokens: &[LexedToken], index: usize) -> Option<(String, usize)> {
    let token = tokens.get(index)?;
    if token.kind == SyntaxKind::Hash {
        if !static_less_mixin_hash_name_is_safe(token.text.as_str()) {
            return None;
        }
        return Some((token.text.clone(), index + 1));
    }
    if token.kind != SyntaxKind::Dot {
        return None;
    }
    let name_index = static_stylesheet_skip_trivia_tokens(tokens, index + 1);
    let name_token = tokens.get(name_index)?;
    if !matches!(
        name_token.kind,
        SyntaxKind::Ident | SyntaxKind::CustomPropertyName
    ) || !static_less_mixin_name_part_is_safe(name_token.text.as_str())
    {
        return None;
    }
    Some((format!(".{}", name_token.text), name_index + 1))
}

fn static_less_mixin_call_semicolon_and_importance(
    source: &str,
    tokens: &[LexedToken],
    close_index: usize,
) -> Option<(usize, bool)> {
    let suffix_start = static_stylesheet_token_end(tokens.get(close_index)?);
    for (index, token) in tokens.iter().enumerate().skip(close_index + 1) {
        if token.kind != SyntaxKind::Semicolon {
            continue;
        }
        let suffix = source
            .get(suffix_start..static_stylesheet_token_start(token))?
            .trim();
        if suffix.is_empty() {
            return Some((index, false));
        }
        if suffix.eq_ignore_ascii_case("!important") {
            return Some((index, true));
        }
        return None;
    }
    None
}

fn static_less_mixin_signature_at(tokens: &[LexedToken], index: usize) -> Option<(String, usize)> {
    let token = tokens.get(index)?;
    if token.kind == SyntaxKind::Hash {
        if !static_less_mixin_hash_name_is_safe(token.text.as_str()) {
            return None;
        }
        let open_index = static_stylesheet_skip_trivia_tokens(tokens, index + 1);
        if tokens
            .get(open_index)
            .is_none_or(|token| token.kind != SyntaxKind::LeftParen)
        {
            return None;
        }
        return Some((token.text.clone(), open_index));
    }

    if token.kind != SyntaxKind::Dot {
        return None;
    }
    let name_index = static_stylesheet_skip_trivia_tokens(tokens, index + 1);
    let name_token = tokens.get(name_index)?;
    if !matches!(
        name_token.kind,
        SyntaxKind::Ident | SyntaxKind::CustomPropertyName
    ) || !static_less_mixin_name_part_is_safe(name_token.text.as_str())
    {
        return None;
    }
    let open_index = static_stylesheet_skip_trivia_tokens(tokens, name_index + 1);
    if tokens
        .get(open_index)
        .is_none_or(|token| token.kind != SyntaxKind::LeftParen)
    {
        return None;
    }
    Some((format!(".{}", name_token.text), open_index))
}

fn static_less_mixin_call_context_is_plain(tokens: &[LexedToken], index: usize) -> bool {
    tokens
        .get(..index)
        .and_then(|prefix| {
            prefix
                .iter()
                .rev()
                .find(|token| !static_stylesheet_token_is_trivia(token.kind))
        })
        .is_none_or(|token| matches!(token.kind, SyntaxKind::LeftBrace | SyntaxKind::Semicolon))
}

fn static_less_mixin_header_guard_text(
    source: &str,
    tokens: &[LexedToken],
    start: usize,
    end: usize,
) -> Option<Option<String>> {
    let first = static_stylesheet_skip_trivia_tokens(tokens, start);
    if first >= end {
        return Some(None);
    }
    let first_token = tokens.get(first)?;
    if first_token.kind != SyntaxKind::Ident || !first_token.text.eq_ignore_ascii_case("when") {
        return None;
    }
    let guard_start = static_stylesheet_token_start(first_token);
    let guard_end = tokens.get(end).map(static_stylesheet_token_start)?;
    source
        .get(guard_start..guard_end)
        .map(str::trim)
        .map(ToOwned::to_owned)
        .map(Some)
}

fn collect_static_less_mixin_parameters(
    source: &str,
    tokens: &[LexedToken],
    start: usize,
    end: usize,
) -> Option<Vec<StaticScssFunctionParameter>> {
    let parameter_start = tokens.get(start).map(static_stylesheet_token_start)?;
    let parameter_end = tokens
        .get(end)
        .map(static_stylesheet_token_start)
        .unwrap_or(parameter_start);
    let parameter_text = source.get(parameter_start..parameter_end)?.trim();
    let arguments = split_static_less_mixin_parameter_arguments(parameter_text)?;
    let mut parameters = Vec::new();
    let mut names = BTreeSet::new();
    let mut saw_default = false;
    let argument_count = arguments.len();
    for (index, argument) in arguments.into_iter().enumerate() {
        let parameter = parse_static_less_mixin_parameter(argument)?;
        if parameter.variadic && index + 1 != argument_count {
            return None;
        }
        if parameter.default_value.is_some() {
            saw_default = true;
        } else if saw_default && !parameter.variadic {
            return None;
        }
        if parameter.pattern_value.is_none() && !names.insert(parameter.name.clone()) {
            return None;
        }
        parameters.push(parameter);
    }
    Some(parameters)
}

fn parse_static_less_mixin_parameter(
    argument: StaticScssFunctionArgument,
) -> Option<StaticScssFunctionParameter> {
    if let Some(name) = argument.name {
        return Some(StaticScssFunctionParameter {
            name,
            default_value: Some(argument.value),
            variadic: false,
            pattern_value: None,
        });
    }
    let (name, variadic) = if let Some(name) = argument.value.strip_suffix("...") {
        (name.trim(), true)
    } else {
        (argument.value.as_str(), false)
    };
    if static_less_variable_name_is_safe(name) {
        return Some(StaticScssFunctionParameter {
            name: name.to_string(),
            default_value: None,
            variadic,
            pattern_value: None,
        });
    }
    (!variadic && static_less_mixin_argument_value_is_safe(argument.value.as_str())).then(|| {
        StaticScssFunctionParameter {
            name: String::new(),
            default_value: None,
            variadic: false,
            pattern_value: Some(argument.value),
        }
    })
}

fn split_static_less_mixin_arguments(arguments: &str) -> Option<Vec<StaticScssFunctionArgument>> {
    split_static_less_mixin_arguments_with_options(arguments, false)
}

fn split_static_less_mixin_parameter_arguments(
    arguments: &str,
) -> Option<Vec<StaticScssFunctionArgument>> {
    split_static_less_mixin_arguments_with_options(arguments, true)
}

fn split_static_less_mixin_arguments_with_options(
    arguments: &str,
    allow_rest_parameter: bool,
) -> Option<Vec<StaticScssFunctionArgument>> {
    let arguments = arguments.trim();
    if arguments.is_empty() {
        return Some(Vec::new());
    }
    let separator = if static_less_mixin_arguments_have_top_level_separator(arguments, ';')? {
        ';'
    } else {
        ','
    };
    split_static_less_mixin_arguments_with_separator(arguments, separator, allow_rest_parameter)
}

fn split_static_less_mixin_arguments_with_separator(
    arguments: &str,
    separator: char,
    allow_rest_parameter: bool,
) -> Option<Vec<StaticScssFunctionArgument>> {
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
            ch if ch == separator && paren_depth == 0 && bracket_depth == 0 => {
                values.push(parse_static_less_mixin_argument(
                    arguments.get(cursor..index)?.trim(),
                    allow_rest_parameter,
                )?);
                cursor = index + ch.len_utf8();
            }
            _ => {}
        }
        index += ch.len_utf8();
    }

    if quote.is_some() || paren_depth != 0 || bracket_depth != 0 {
        return None;
    }
    values.push(parse_static_less_mixin_argument(
        arguments.get(cursor..)?.trim(),
        allow_rest_parameter,
    )?);
    Some(values)
}

fn static_less_mixin_arguments_have_top_level_separator(
    arguments: &str,
    separator: char,
) -> Option<bool> {
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
            ch if ch == separator && paren_depth == 0 && bracket_depth == 0 => return Some(true),
            _ => {}
        }
        index += ch.len_utf8();
    }
    (quote.is_none() && paren_depth == 0 && bracket_depth == 0).then_some(false)
}

fn parse_static_less_mixin_argument(
    value: &str,
    allow_rest_parameter: bool,
) -> Option<StaticScssFunctionArgument> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if let Some(colon_index) = static_scss_top_level_colon_index(value)? {
        let name = value.get(..colon_index)?.trim();
        let argument_value = value.get(colon_index + ':'.len_utf8()..)?.trim();
        if !static_less_variable_name_is_safe(name)
            || !static_less_mixin_argument_value_is_safe(argument_value)
        {
            return None;
        }
        return Some(StaticScssFunctionArgument {
            name: Some(name.to_string()),
            value: argument_value.to_string(),
        });
    }
    if allow_rest_parameter
        && let Some(rest_name) = value.strip_suffix("...")
        && static_less_variable_name_is_safe(rest_name.trim())
    {
        return Some(StaticScssFunctionArgument {
            name: None,
            value: value.to_string(),
        });
    }
    static_less_mixin_argument_value_is_safe(value).then_some(StaticScssFunctionArgument {
        name: None,
        value: value.to_string(),
    })
}

fn render_static_less_mixin_body(
    declaration: &StaticLessMixinDeclaration,
    call: &StaticLessMixinCall,
    call_scope_id: usize,
    context: StaticLessMixinRenderContext<'_>,
    active_mixins: &mut BTreeSet<String>,
    default_matches: Option<bool>,
) -> Option<StaticLessMixinRenderOutcome> {
    let canonical_name = canonical_static_less_mixin_name(declaration.name.as_str());
    if !active_mixins.insert(canonical_name.clone()) {
        return None;
    }
    let body = context
        .source
        .get(declaration.body_start..declaration.body_end)?;
    if !static_less_mixin_body_is_static_declaration_subset(body) {
        return None;
    }
    let mut argument_values = BTreeMap::new();
    for (parameter, argument) in
        bind_static_scss_callable_arguments(&declaration.parameters, &call.arguments)?
    {
        let rendered_value = resolve_static_less_mixin_value_with_bindings(
            argument.as_str(),
            &argument_values,
            context.captured_values,
            call_scope_id,
            context.scopes,
            context.variable_declarations,
            context.detached_ruleset_declarations,
        )?;
        argument_values.insert(parameter, rendered_value);
    }
    if let Some(arguments_value) = static_less_mixin_arguments_value(call.arguments.as_slice()) {
        argument_values.insert("@arguments".to_string(), arguments_value);
    }
    if let Some(guard) = &declaration.guard
        && !static_less_mixin_guard_matches(
            guard,
            &argument_values,
            call_scope_id,
            context,
            default_matches,
        )?
    {
        active_mixins.remove(&canonical_name);
        return Some(StaticLessMixinRenderOutcome::GuardNotMatched);
    }
    let body = render_static_less_mixin_body_variables(
        body,
        call_scope_id,
        &argument_values,
        context.captured_values,
        context.scopes,
        context.variable_declarations,
        context.detached_ruleset_declarations,
    )?;
    let nested = render_static_less_mixin_body_nested_calls(
        body.as_str(),
        call_scope_id,
        context,
        active_mixins,
    )?;
    let body = resolve_static_less_mixin_body_declaration_values(nested.body.as_str())?;
    let body = if call.important {
        apply_static_less_mixin_call_importance(body.as_str())?
    } else {
        body
    };
    let mut used_declaration_names = nested.used_declaration_names;
    used_declaration_names.insert(canonical_name.clone());
    active_mixins.remove(&canonical_name);
    Some(StaticLessMixinRenderOutcome::Rendered(
        StaticLessMixinRenderResult {
            body,
            used_declaration_names,
        },
    ))
}

fn render_static_less_mixin_accessor(
    accessor: &StaticLessMixinAccessor,
    call_scope_id: usize,
    context: StaticLessMixinRenderContext<'_>,
) -> Option<Option<StaticLessMixinAccessorRenderResult>> {
    let canonical_accessor_name = canonical_static_less_mixin_name(accessor.name.as_str());
    let declarations = context
        .declarations
        .iter()
        .filter(|declaration| {
            canonical_static_less_mixin_name(declaration.name.as_str()) == canonical_accessor_name
        })
        .collect::<Vec<_>>();
    if declarations.is_empty() {
        return Some(None);
    }

    let call = StaticLessMixinCall {
        namespace: None,
        namespace_arguments: Vec::new(),
        name: accessor.name.clone(),
        start: accessor.start,
        end: accessor.end,
        important: false,
        arguments: accessor.arguments.clone(),
    };
    let mut rendered_values = Vec::new();
    for declaration in &declarations {
        if declaration
            .guard
            .as_deref()
            .is_some_and(static_less_mixin_guard_depends_on_default)
        {
            continue;
        }
        if !static_less_mixin_parameter_patterns_match(&declaration.parameters, &call.arguments) {
            continue;
        }
        match render_static_less_mixin_accessor_declaration(
            declaration,
            &call,
            accessor.member.as_str(),
            call_scope_id,
            context,
            None,
        )? {
            StaticLessMixinAccessorRenderOutcome::Rendered(rendered) => {
                rendered_values.push(rendered)
            }
            StaticLessMixinAccessorRenderOutcome::GuardNotMatched => {}
        }
    }

    let default_matches = Some(rendered_values.is_empty());
    for declaration in declarations.iter().filter(|declaration| {
        declaration
            .guard
            .as_deref()
            .is_some_and(static_less_mixin_guard_depends_on_default)
    }) {
        if !static_less_mixin_parameter_patterns_match(&declaration.parameters, &call.arguments) {
            continue;
        }
        match render_static_less_mixin_accessor_declaration(
            declaration,
            &call,
            accessor.member.as_str(),
            call_scope_id,
            context,
            default_matches,
        )? {
            StaticLessMixinAccessorRenderOutcome::Rendered(rendered) => {
                rendered_values.push(rendered)
            }
            StaticLessMixinAccessorRenderOutcome::GuardNotMatched => {}
        }
    }

    let mut rendered_values = rendered_values.into_iter();
    let rendered = rendered_values.next()?;
    rendered_values.next().is_none().then_some(Some(rendered))
}

fn render_static_less_mixin_accessor_declaration(
    declaration: &StaticLessMixinDeclaration,
    call: &StaticLessMixinCall,
    member: &str,
    call_scope_id: usize,
    context: StaticLessMixinRenderContext<'_>,
    default_matches: Option<bool>,
) -> Option<StaticLessMixinAccessorRenderOutcome> {
    let canonical_name = canonical_static_less_mixin_name(declaration.name.as_str());
    let body = context
        .source
        .get(declaration.body_start..declaration.body_end)?;
    if !static_less_mixin_body_is_static_declaration_subset(body) {
        return None;
    }
    let body_lexed = lex(body, StyleDialect::Less);
    if !collect_static_less_mixin_calls(body, body_lexed.tokens())?.is_empty()
        || !collect_static_less_detached_ruleset_calls(body, body_lexed.tokens())?.is_empty()
    {
        return None;
    }

    let mut argument_values = BTreeMap::new();
    for (parameter, argument) in
        bind_static_scss_callable_arguments(&declaration.parameters, &call.arguments)?
    {
        let rendered_value = resolve_static_less_mixin_value_with_bindings(
            argument.as_str(),
            &argument_values,
            context.captured_values,
            call_scope_id,
            context.scopes,
            context.variable_declarations,
            context.detached_ruleset_declarations,
        )?;
        argument_values.insert(parameter, rendered_value);
    }
    if let Some(arguments_value) = static_less_mixin_arguments_value(call.arguments.as_slice()) {
        argument_values.insert("@arguments".to_string(), arguments_value);
    }
    if let Some(guard) = &declaration.guard
        && !static_less_mixin_guard_matches(
            guard,
            &argument_values,
            call_scope_id,
            context,
            default_matches,
        )?
    {
        return Some(StaticLessMixinAccessorRenderOutcome::GuardNotMatched);
    }

    let scoped_values = static_less_mixin_body_scoped_values(
        body,
        call_scope_id,
        &argument_values,
        context.captured_values,
        context.scopes,
        context.variable_declarations,
        context.detached_ruleset_declarations,
    )?;
    let value = if static_less_variable_name_is_safe(member) {
        scoped_values.get(member)?.clone()
    } else {
        static_less_mixin_accessor_property_value(
            body,
            member,
            &scoped_values,
            call_scope_id,
            context,
        )?
    };
    Some(StaticLessMixinAccessorRenderOutcome::Rendered(
        StaticLessMixinAccessorRenderResult {
            value,
            used_declaration_name: canonical_name,
        },
    ))
}

fn static_less_mixin_body_scoped_values(
    body: &str,
    call_scope_id: usize,
    argument_values: &BTreeMap<String, String>,
    captured_values: &BTreeMap<String, String>,
    scopes: &[StaticStylesheetScope],
    variable_declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
) -> Option<BTreeMap<String, String>> {
    let local_declarations = collect_static_less_mixin_body_local_declarations(body)?;
    let mut scoped_values = argument_values.clone();
    for local in &local_declarations {
        let rendered_value = resolve_static_less_mixin_value_with_bindings(
            local.declaration.value.as_str(),
            &scoped_values,
            captured_values,
            call_scope_id,
            scopes,
            variable_declarations,
            detached_ruleset_declarations,
        )?;
        scoped_values.insert(local.name.clone(), rendered_value);
    }
    Some(scoped_values)
}

fn static_less_mixin_accessor_property_value(
    body: &str,
    member: &str,
    scoped_values: &BTreeMap<String, String>,
    call_scope_id: usize,
    context: StaticLessMixinRenderContext<'_>,
) -> Option<String> {
    static_less_body_property_value(body, member, scoped_values, call_scope_id, context)
}

fn static_less_body_property_value(
    body: &str,
    member: &str,
    scoped_values: &BTreeMap<String, String>,
    call_scope_id: usize,
    context: StaticLessMixinRenderContext<'_>,
) -> Option<String> {
    if !static_stylesheet_property_name_is_safe(member) {
        return None;
    }
    let body_lexed = lex(body, StyleDialect::Less);
    let body_scopes = collect_static_stylesheet_scopes(body)?;
    let property_declarations =
        collect_static_less_body_property_declarations(body, body_lexed.tokens(), &body_scopes)?;
    let declaration = find_static_less_property_declaration(
        format!("${member}").as_str(),
        0,
        &body_scopes,
        &property_declarations,
    )?;
    resolve_static_less_mixin_value_with_bindings(
        declaration.value.as_str(),
        scoped_values,
        context.captured_values,
        call_scope_id,
        context.scopes,
        context.variable_declarations,
        context.detached_ruleset_declarations,
    )
}

fn render_static_less_mixin_call(
    call: &StaticLessMixinCall,
    call_scope_id: usize,
    context: StaticLessMixinRenderContext<'_>,
    active_mixins: &mut BTreeSet<String>,
) -> Option<Option<StaticLessMixinRenderResult>> {
    if call.namespace.is_some() {
        return render_static_less_namespace_mixin_call(
            call,
            call_scope_id,
            context,
            active_mixins,
        );
    }
    let canonical_call_name = canonical_static_less_mixin_name(call.name.as_str());
    let mut saw_declaration = false;
    let mut rendered_bodies = Vec::new();
    let mut used_declaration_names = BTreeSet::new();
    let declarations = context
        .declarations
        .iter()
        .filter(|declaration| {
            canonical_static_less_mixin_name(declaration.name.as_str()) == canonical_call_name
        })
        .collect::<Vec<_>>();
    for declaration in &declarations {
        saw_declaration = true;
        if declaration
            .guard
            .as_deref()
            .is_some_and(static_less_mixin_guard_depends_on_default)
        {
            continue;
        }
        if !static_less_mixin_parameter_patterns_match(&declaration.parameters, &call.arguments) {
            continue;
        }
        match render_static_less_mixin_body(
            declaration,
            call,
            call_scope_id,
            context,
            active_mixins,
            None,
        )? {
            StaticLessMixinRenderOutcome::Rendered(rendered) => {
                used_declaration_names.extend(rendered.used_declaration_names);
                rendered_bodies.push(rendered.body);
            }
            StaticLessMixinRenderOutcome::GuardNotMatched => {}
        }
    }
    let default_matches = Some(rendered_bodies.is_empty());
    for declaration in declarations.iter().filter(|declaration| {
        declaration
            .guard
            .as_deref()
            .is_some_and(static_less_mixin_guard_depends_on_default)
    }) {
        if !static_less_mixin_parameter_patterns_match(&declaration.parameters, &call.arguments) {
            continue;
        }
        match render_static_less_mixin_body(
            declaration,
            call,
            call_scope_id,
            context,
            active_mixins,
            default_matches,
        )? {
            StaticLessMixinRenderOutcome::Rendered(rendered) => {
                used_declaration_names.extend(rendered.used_declaration_names);
                rendered_bodies.push(rendered.body);
            }
            StaticLessMixinRenderOutcome::GuardNotMatched => {}
        }
    }
    if !saw_declaration {
        return Some(None);
    }
    if rendered_bodies.is_empty() {
        return None;
    }
    Some(Some(StaticLessMixinRenderResult {
        body: rendered_bodies.join(" "),
        used_declaration_names,
    }))
}

fn render_static_less_namespace_mixin_call(
    call: &StaticLessMixinCall,
    call_scope_id: usize,
    context: StaticLessMixinRenderContext<'_>,
    active_mixins: &mut BTreeSet<String>,
) -> Option<Option<StaticLessMixinRenderResult>> {
    let namespace = call.namespace.as_ref()?;
    let canonical_namespace = canonical_static_less_mixin_name(namespace.as_str());
    let mut saw_namespace = false;
    let mut rendered_bodies = Vec::new();

    for declaration in context.declarations.iter().filter(|declaration| {
        canonical_static_less_mixin_name(declaration.name.as_str()) == canonical_namespace
    }) {
        saw_namespace = true;
        if declaration.parameters.is_empty() && !call.namespace_arguments.is_empty() {
            continue;
        }
        let mut namespace_argument_values = BTreeMap::new();
        let Some(bound_namespace_arguments) = bind_static_scss_callable_arguments(
            &declaration.parameters,
            call.namespace_arguments.as_slice(),
        ) else {
            continue;
        };
        for (parameter, argument) in bound_namespace_arguments {
            let rendered_value = resolve_static_less_mixin_value_with_bindings(
                argument.as_str(),
                &namespace_argument_values,
                context.captured_values,
                call_scope_id,
                context.scopes,
                context.variable_declarations,
                context.detached_ruleset_declarations,
            )?;
            namespace_argument_values.insert(parameter, rendered_value);
        }
        if let Some(guard) = &declaration.guard
            && !static_less_mixin_guard_matches(
                guard,
                &namespace_argument_values,
                call_scope_id,
                context,
                None,
            )?
        {
            continue;
        }
        if !active_mixins.insert(canonical_namespace.clone()) {
            return None;
        }
        let body = context
            .source
            .get(declaration.body_start..declaration.body_end)?;
        let body_lexed = lex(body, StyleDialect::Less);
        let nested_declarations =
            collect_static_less_mixin_declarations(body, body_lexed.tokens())?;
        let nested_context = StaticLessMixinRenderContext {
            source: body,
            declarations: nested_declarations.as_slice(),
            detached_ruleset_declarations: context.detached_ruleset_declarations,
            scopes: context.scopes,
            variable_declarations: context.variable_declarations,
            captured_values: &namespace_argument_values,
        };
        let nested_call = StaticLessMixinCall {
            namespace: None,
            namespace_arguments: Vec::new(),
            name: call.name.clone(),
            start: call.start,
            end: call.end,
            important: call.important,
            arguments: call.arguments.clone(),
        };
        if let Some(rendered) = render_static_less_mixin_call(
            &nested_call,
            call_scope_id,
            nested_context,
            active_mixins,
        )? {
            rendered_bodies.push(rendered.body);
        }
        active_mixins.remove(&canonical_namespace);
    }

    if !saw_namespace {
        return Some(None);
    }
    if rendered_bodies.is_empty() {
        return None;
    }
    Some(Some(StaticLessMixinRenderResult {
        body: rendered_bodies.join(" "),
        used_declaration_names: BTreeSet::from([canonical_namespace]),
    }))
}

fn static_less_mixin_arguments_value(arguments: &[StaticScssFunctionArgument]) -> Option<String> {
    arguments
        .iter()
        .map(|argument| {
            static_less_mixin_argument_value_is_safe(argument.value.as_str())
                .then(|| argument.value.clone())
        })
        .collect::<Option<Vec<_>>>()
        .map(|values| values.join(", "))
}

fn render_static_less_mixin_body_nested_calls(
    body: &str,
    call_scope_id: usize,
    context: StaticLessMixinRenderContext<'_>,
    active_mixins: &mut BTreeSet<String>,
) -> Option<StaticLessMixinRenderResult> {
    let body_lexed = lex(body, StyleDialect::Less);
    let body_tokens = body_lexed.tokens();
    let calls = collect_static_less_mixin_calls(body, body_tokens)?;
    let detached_calls = collect_static_less_detached_ruleset_calls(body, body_tokens)?;
    if calls.is_empty() && detached_calls.is_empty() {
        return Some(StaticLessMixinRenderResult {
            body: body.to_string(),
            used_declaration_names: BTreeSet::new(),
        });
    }

    let mut edits = Vec::new();
    let mut used_declaration_names = BTreeSet::new();
    for call in calls {
        let Some(rendered) =
            render_static_less_mixin_call(&call, call_scope_id, context, active_mixins)?
        else {
            continue;
        };
        used_declaration_names.extend(rendered.used_declaration_names);
        edits.push(StaticStylesheetEvaluationEdit {
            start: call.start,
            end: call.end,
            replacement: rendered.body,
        });
    }
    for call in detached_calls {
        let declaration = find_static_less_detached_ruleset_declaration(
            call.name.as_str(),
            call_scope_id,
            context.scopes,
            context.detached_ruleset_declarations,
        )?;
        let rendered = render_static_less_detached_ruleset_body(
            context.source,
            declaration,
            call_scope_id,
            context.scopes,
            context.variable_declarations,
            context.declarations,
            context.detached_ruleset_declarations,
        )?;
        used_declaration_names.extend(rendered.used_declaration_names);
        edits.push(StaticStylesheetEvaluationEdit {
            start: call.start,
            end: call.end,
            replacement: rendered.body,
        });
    }

    Some(StaticLessMixinRenderResult {
        body: apply_static_stylesheet_evaluation_edits(body, edits)?,
        used_declaration_names,
    })
}

fn render_static_less_mixin_body_variables(
    body: &str,
    call_scope_id: usize,
    argument_values: &BTreeMap<String, String>,
    captured_values: &BTreeMap<String, String>,
    scopes: &[StaticStylesheetScope],
    variable_declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
) -> Option<String> {
    let local_declarations = collect_static_less_mixin_body_local_declarations(body)?;
    let local_declaration_ranges = local_declarations
        .iter()
        .flat_map(|declaration| declaration.declaration.removal_spans.iter().copied())
        .collect::<Vec<_>>();
    let scoped_values = static_less_mixin_body_scoped_values(
        body,
        call_scope_id,
        argument_values,
        captured_values,
        scopes,
        variable_declarations,
        detached_ruleset_declarations,
    )?;
    let mut edits = local_declarations
        .iter()
        .flat_map(|declaration| {
            declaration
                .declaration
                .removal_spans
                .iter()
                .map(|(start, end)| StaticStylesheetEvaluationEdit {
                    start: *start,
                    end: *end,
                    replacement: String::new(),
                })
        })
        .collect::<Vec<_>>();

    let references =
        collect_static_stylesheet_variable_references(body, StaticStylesheetVariableKind::Less)?;
    for reference in references {
        if static_stylesheet_position_is_inside_ranges(reference.start, &local_declaration_ranges) {
            continue;
        }
        let replacement = if let Some(value) = scoped_values.get(reference.name.as_str()) {
            value.clone()
        } else if let Some(value) = captured_values.get(reference.name.as_str()) {
            value.clone()
        } else {
            let mut stack = BTreeSet::new();
            resolve_static_less_variable_value_in_scope(
                reference.name.as_str(),
                call_scope_id,
                scopes,
                variable_declarations,
                &mut stack,
            )?
        };
        edits.push(StaticStylesheetEvaluationEdit {
            start: reference.start,
            end: reference.end,
            replacement,
        });
    }
    apply_static_stylesheet_evaluation_edits(body, edits)
}

fn collect_static_less_mixin_body_local_declarations(
    body: &str,
) -> Option<Vec<StaticLessMixinBodyLocalDeclaration>> {
    let facts = omena_parser::collect_style_facts(body, StyleDialect::Less);
    let mut declarations = Vec::new();
    for fact in facts
        .variables
        .iter()
        .filter(|fact| fact.kind == ParsedVariableFactKind::LessDeclaration)
    {
        let start = parser_text_size_to_usize(fact.range.start().into());
        let end = parser_text_size_to_usize(fact.range.end().into());
        if static_stylesheet_variable_reference_is_named_argument_label(body, start, end) {
            continue;
        }
        let declaration = extract_static_stylesheet_variable_declaration(
            body,
            start,
            end,
            StaticStylesheetVariableKind::Less,
        )?;
        if !static_stylesheet_less_declaration_value_is_removal_safe(&declaration.value) {
            return None;
        }
        declarations.push(StaticLessMixinBodyLocalDeclaration {
            name: fact.name.clone(),
            declaration,
        });
    }
    declarations.sort_by_key(|declaration| declaration.declaration.span_start);
    Some(declarations)
}

fn resolve_static_less_mixin_value_with_bindings(
    value: &str,
    argument_values: &BTreeMap<String, String>,
    captured_values: &BTreeMap<String, String>,
    call_scope_id: usize,
    scopes: &[StaticStylesheetScope],
    variable_declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
) -> Option<String> {
    let references =
        collect_static_stylesheet_variable_references(value, StaticStylesheetVariableKind::Less)?;
    if references.is_empty() {
        return static_stylesheet_literal_value_is_safe(value)
            .then(|| reduce_static_less_value(value.to_string()));
    }
    if !static_stylesheet_composite_value_is_safe(value) {
        return None;
    }

    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    for reference in references {
        let replacement = if let Some(value) = argument_values.get(reference.name.as_str()) {
            value.clone()
        } else if let Some(value) = captured_values.get(reference.name.as_str()) {
            value.clone()
        } else if static_less_value_is_detached_ruleset_reference(
            reference.name.as_str(),
            call_scope_id,
            scopes,
            detached_ruleset_declarations,
        ) {
            reference.name.clone()
        } else {
            let mut stack = BTreeSet::new();
            resolve_static_less_variable_value_in_scope(
                reference.name.as_str(),
                call_scope_id,
                scopes,
                variable_declarations,
                &mut stack,
            )?
        };
        output.push_str(&value[cursor..reference.start]);
        output.push_str(&replacement);
        cursor = reference.end;
    }
    output.push_str(&value[cursor..]);
    if static_less_value_is_detached_ruleset_reference(
        output.trim(),
        call_scope_id,
        scopes,
        detached_ruleset_declarations,
    ) {
        return Some(output.trim().to_string());
    }
    static_stylesheet_literal_value_is_safe(output.as_str())
        .then(|| reduce_static_less_value(output))
}

fn static_less_value_is_detached_ruleset_reference(
    value: &str,
    call_scope_id: usize,
    scopes: &[StaticStylesheetScope],
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
) -> bool {
    let value = value.trim();
    value.starts_with('@')
        && find_static_less_detached_ruleset_declaration(
            value,
            call_scope_id,
            scopes,
            detached_ruleset_declarations,
        )
        .is_some()
}

fn resolve_static_less_mixin_body_declaration_values(body: &str) -> Option<String> {
    let value_ranges = collect_static_scss_mixin_body_declaration_value_ranges(body)?;
    let mut edits = Vec::new();
    for (start, end) in value_ranges {
        let value = body.get(start..end)?;
        let rendered_value = reduce_static_less_value(value.to_string());
        if rendered_value != value {
            edits.push(StaticStylesheetEvaluationEdit {
                start,
                end,
                replacement: rendered_value,
            });
        }
    }
    apply_static_stylesheet_evaluation_edits(body, edits)
}

fn apply_static_less_mixin_call_importance(body: &str) -> Option<String> {
    let mut output = String::new();
    let mut cursor = 0usize;
    for (index, ch) in body.char_indices() {
        if ch != ';' {
            continue;
        }
        let declaration = body.get(cursor..index)?.trim();
        if !declaration.is_empty() {
            if !output.is_empty() {
                output.push(' ');
            }
            if !static_scss_bang_usage_is_comparison_only(declaration) {
                return None;
            }
            output.push_str(declaration);
            output.push_str(" !important;");
        }
        cursor = index + ch.len_utf8();
    }
    body.get(cursor..)
        .is_some_and(|tail| tail.trim().is_empty())
        .then_some(output)
}

fn resolve_static_scss_mixin_body_declaration_values(
    body: &str,
    call_position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<String> {
    let value_ranges = collect_static_scss_mixin_body_declaration_value_ranges(body)?;
    let mut edits = Vec::new();
    let empty_arguments = BTreeMap::new();
    for (start, end) in value_ranges {
        let value = body.get(start..end)?;
        let resolution = resolve_static_scss_function_value_with_bindings(
            value,
            &empty_arguments,
            call_position,
            STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
            context,
        );
        if resolution.outcome == StaticStylesheetResolutionOutcome::Top {
            return None;
        }
        let rendered_value = resolution.rendered_value?;
        if rendered_value != value {
            edits.push(StaticStylesheetEvaluationEdit {
                start,
                end,
                replacement: rendered_value,
            });
        }
    }
    apply_static_stylesheet_evaluation_edits(body, edits)
}

fn collect_static_scss_mixin_body_declaration_value_ranges(
    body: &str,
) -> Option<Vec<(usize, usize)>> {
    let mut ranges = Vec::new();
    let mut statement_start = 0usize;
    let mut index = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;

    while index < body.len() {
        let ch = body[index..].chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = body[index..].chars().next() {
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
            ';' if paren_depth == 0 && bracket_depth == 0 => {
                collect_static_scss_mixin_body_statement_value_range(
                    body,
                    statement_start,
                    index,
                    &mut ranges,
                )?;
                statement_start = index + ch.len_utf8();
            }
            _ => {}
        }
        index += ch.len_utf8();
    }

    if quote.is_some() || paren_depth != 0 || bracket_depth != 0 {
        return None;
    }
    let trailing = body.get(statement_start..)?;
    trailing.trim().is_empty().then_some(ranges)
}

fn collect_static_scss_mixin_body_statement_value_range(
    body: &str,
    statement_start: usize,
    statement_end: usize,
    ranges: &mut Vec<(usize, usize)>,
) -> Option<()> {
    let statement = body.get(statement_start..statement_end)?;
    if statement.trim().is_empty() {
        return Some(());
    }
    let colon_index = static_scss_top_level_colon_index(statement)??;
    let mut value_start = statement_start + colon_index + ':'.len_utf8();
    let mut value_end = statement_end;
    while value_start < value_end {
        let ch = body[value_start..].chars().next()?;
        if !ch.is_ascii_whitespace() {
            break;
        }
        value_start += ch.len_utf8();
    }
    while value_start < value_end {
        let ch = body[..value_end].chars().next_back()?;
        if !ch.is_ascii_whitespace() {
            break;
        }
        value_end -= ch.len_utf8();
    }
    if value_start >= value_end {
        return None;
    }
    ranges.push((value_start, value_end));
    Some(())
}

fn bind_static_scss_callable_arguments(
    parameters: &[StaticScssFunctionParameter],
    arguments: &[StaticScssFunctionArgument],
) -> Option<Vec<(String, String)>> {
    let mut bindings = BTreeMap::<String, String>::new();
    let mut positional_index = 0usize;
    let mut saw_named_argument = false;

    for argument in arguments {
        if let Some(argument_name) = argument.name.as_ref() {
            saw_named_argument = true;
            if !parameters.iter().any(|parameter| {
                parameter.pattern_value.is_none() && parameter.name == *argument_name
            }) || bindings
                .insert(argument_name.clone(), argument.value.clone())
                .is_some()
            {
                return None;
            }
            continue;
        }

        if saw_named_argument {
            return None;
        }
        let parameter = parameters.get(positional_index)?;
        if let Some(pattern_value) = parameter.pattern_value.as_deref() {
            if !static_less_mixin_pattern_argument_matches(pattern_value, argument.value.as_str()) {
                return None;
            }
            positional_index += 1;
            continue;
        }
        if parameter.variadic {
            bindings
                .entry(parameter.name.clone())
                .and_modify(|value| {
                    value.push_str(", ");
                    value.push_str(argument.value.as_str());
                })
                .or_insert_with(|| argument.value.clone());
            continue;
        }
        if bindings
            .insert(parameter.name.clone(), argument.value.clone())
            .is_some()
        {
            return None;
        }
        positional_index += 1;
    }

    for (index, parameter) in parameters.iter().enumerate() {
        if parameter.pattern_value.is_some() {
            if index >= positional_index {
                return None;
            }
            continue;
        }
        if bindings.contains_key(parameter.name.as_str()) {
            continue;
        }
        if parameter.variadic {
            return None;
        }
        let default_value = parameter.default_value.as_ref()?;
        bindings.insert(parameter.name.clone(), default_value.clone());
    }

    parameters
        .iter()
        .filter(|parameter| parameter.pattern_value.is_none())
        .map(|parameter| {
            bindings
                .remove(parameter.name.as_str())
                .map(|value| (parameter.name.clone(), value))
        })
        .collect::<Option<Vec<_>>>()
}

fn static_less_mixin_pattern_argument_matches(pattern_value: &str, argument_value: &str) -> bool {
    pattern_value.trim() == argument_value.trim()
}

fn static_less_mixin_parameter_patterns_match(
    parameters: &[StaticScssFunctionParameter],
    arguments: &[StaticScssFunctionArgument],
) -> bool {
    let mut positional_index = 0usize;
    let mut saw_named_argument = false;
    for argument in arguments {
        if argument.name.is_some() {
            saw_named_argument = true;
            continue;
        }
        if saw_named_argument {
            return true;
        }
        let Some(parameter) = parameters.get(positional_index) else {
            return true;
        };
        if let Some(pattern_value) = parameter.pattern_value.as_deref()
            && !static_less_mixin_pattern_argument_matches(pattern_value, argument.value.as_str())
        {
            return false;
        }
        positional_index += 1;
        if parameter.variadic {
            return true;
        }
    }
    parameters
        .iter()
        .enumerate()
        .all(|(index, parameter)| parameter.pattern_value.is_none() || index < positional_index)
}

fn resolve_static_scss_function_argument_abstract_value(
    argument: &str,
    argument_values: &BTreeMap<String, String>,
    call_position: usize,
    fuel: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> StaticStylesheetAbstractResolution {
    resolve_static_scss_function_value_with_bindings(
        argument,
        argument_values,
        call_position,
        fuel,
        context,
    )
}

fn resolve_static_scss_function_return_abstract_value(
    declaration: &StaticScssFunctionDeclaration,
    argument_values: &BTreeMap<String, String>,
    fuel: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> StaticStylesheetAbstractResolution {
    for clause in &declaration.return_clauses {
        if !clause.loop_headers.is_empty() {
            match resolve_static_scss_loop_return_clause(
                declaration,
                clause,
                argument_values,
                fuel,
                context,
            ) {
                StaticScssLoopReturnResolution::Active(resolution) => return resolution,
                StaticScssLoopReturnResolution::Inactive => continue,
                StaticScssLoopReturnResolution::Unknown(reason) => {
                    return top_static_abstract_value(reason);
                }
            }
        }
        let argument_values = match bind_static_scss_function_local_variables_before(
            declaration,
            argument_values,
            clause.span_start,
            fuel,
            context,
        ) {
            Ok(argument_values) => argument_values,
            Err(resolution) => return resolution,
        };
        let Some(condition) = clause.condition.as_ref() else {
            return resolve_static_scss_function_value_with_bindings(
                clause.value.as_str(),
                &argument_values,
                clause.span_start,
                fuel,
                context,
            );
        };
        let condition_resolution = resolve_static_scss_function_value_with_bindings(
            condition.as_str(),
            &argument_values,
            clause.span_start,
            fuel,
            context,
        );
        if condition_resolution.outcome == StaticStylesheetResolutionOutcome::Top {
            return top_static_abstract_value(condition_resolution.reason);
        }
        let Some(condition_value) = condition_resolution.rendered_value else {
            return top_static_abstract_value(condition_resolution.reason);
        };
        let Some(truthy) = static_scss_literal_truthiness(condition_value.as_str()) else {
            return top_static_abstract_value(StaticStylesheetResolutionReason::UnsupportedDynamic);
        };
        if truthy {
            return resolve_static_scss_function_value_with_bindings(
                clause.value.as_str(),
                &argument_values,
                clause.span_start,
                fuel,
                context,
            );
        }
    }
    top_static_abstract_value(StaticStylesheetResolutionReason::UnsupportedDynamic)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum StaticScssLoopReturnResolution {
    Active(StaticStylesheetAbstractResolution),
    Inactive,
    Unknown(StaticStylesheetResolutionReason),
}

fn resolve_static_scss_loop_return_clause(
    declaration: &StaticScssFunctionDeclaration,
    clause: &StaticScssFunctionReturnClause,
    argument_values: &BTreeMap<String, String>,
    fuel: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> StaticScssLoopReturnResolution {
    let Some(frames) = static_scss_loop_binding_frames_for_headers(
        declaration,
        clause.loop_headers.as_slice(),
        argument_values,
        fuel,
        context,
    ) else {
        return StaticScssLoopReturnResolution::Unknown(
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        );
    };
    if frames.is_empty() {
        return StaticScssLoopReturnResolution::Inactive;
    }

    for frame in frames {
        let mut frame_values = argument_values.clone();
        for (name, value) in frame {
            frame_values.insert(canonical_static_scss_variable_name(name.as_str()), value);
        }
        let loop_body_start = clause
            .loop_headers
            .last()
            .map(|header| header.body_start)
            .unwrap_or(declaration.body_start);
        let frame_values = match bind_static_scss_function_local_variables_in_range(
            declaration,
            &frame_values,
            loop_body_start,
            clause.span_start,
            fuel,
            context,
        ) {
            Ok(frame_values) => frame_values,
            Err(resolution) => return StaticScssLoopReturnResolution::Unknown(resolution.reason),
        };
        let active = match static_scss_return_clause_is_active(clause, &frame_values, fuel, context)
        {
            Ok(active) => active,
            Err(resolution) => return StaticScssLoopReturnResolution::Unknown(resolution.reason),
        };
        if !active {
            continue;
        }
        return StaticScssLoopReturnResolution::Active(
            resolve_static_scss_function_value_with_bindings(
                clause.value.as_str(),
                &frame_values,
                clause.span_start,
                fuel,
                context,
            ),
        );
    }

    StaticScssLoopReturnResolution::Inactive
}

fn static_scss_return_clause_is_active(
    clause: &StaticScssFunctionReturnClause,
    argument_values: &BTreeMap<String, String>,
    fuel: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Result<bool, StaticStylesheetAbstractResolution> {
    let Some(condition) = clause.condition.as_ref() else {
        return Ok(true);
    };
    let condition_resolution = resolve_static_scss_function_value_with_bindings(
        condition.as_str(),
        argument_values,
        clause.span_start,
        fuel,
        context,
    );
    if condition_resolution.outcome == StaticStylesheetResolutionOutcome::Top {
        return Err(top_static_abstract_value(condition_resolution.reason));
    }
    let Some(condition_value) = condition_resolution.rendered_value else {
        return Err(top_static_abstract_value(condition_resolution.reason));
    };
    let Some(truthy) = static_scss_literal_truthiness(condition_value.as_str()) else {
        return Err(top_static_abstract_value(
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        ));
    };
    Ok(truthy)
}

fn static_scss_loop_binding_frames_for_headers(
    declaration: &StaticScssFunctionDeclaration,
    headers: &[StaticScssLoopHeader],
    argument_values: &BTreeMap<String, String>,
    fuel: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<Vec<Vec<(String, String)>>> {
    if headers.is_empty() {
        return None;
    }

    let mut frames = vec![Vec::<(String, String)>::new()];
    for header in headers {
        let mut next_frames = Vec::new();
        for frame in frames {
            let mut frame_values = argument_values.clone();
            for (name, value) in &frame {
                frame_values.insert(
                    canonical_static_scss_variable_name(name.as_str()),
                    value.clone(),
                );
            }
            let frame_values = bind_static_scss_function_local_variables_before(
                declaration,
                &frame_values,
                header.span_start,
                fuel,
                context,
            )
            .ok()?;
            let header_frames =
                static_scss_loop_binding_frames(declaration, header, &frame_values, fuel, context)?;
            for header_frame in header_frames {
                let mut combined = frame.clone();
                combined.extend(header_frame);
                next_frames.push(combined);
                if next_frames.len() > 64 {
                    return None;
                }
            }
        }
        frames = next_frames;
    }

    Some(frames)
}

fn static_scss_loop_binding_frames(
    declaration: &StaticScssFunctionDeclaration,
    header: &StaticScssLoopHeader,
    argument_values: &BTreeMap<String, String>,
    fuel: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<Vec<Vec<(String, String)>>> {
    let header_text = header.text.as_str();
    let position = header.span_start;
    let trimmed = header_text.trim_start();
    if trimmed.to_ascii_lowercase().starts_with("@for") {
        return static_scss_for_loop_binding_frames(
            trimmed,
            argument_values,
            fuel,
            context,
            position,
        );
    }
    if trimmed.to_ascii_lowercase().starts_with("@each") {
        return parse_static_scss_each_loop_binding_frames(trimmed, |source| {
            let resolution = resolve_static_scss_function_value_with_bindings(
                source,
                argument_values,
                position,
                fuel,
                context,
            );
            if resolution.outcome == StaticStylesheetResolutionOutcome::Top {
                return None;
            }
            resolution.rendered_value
        });
    }
    if trimmed.to_ascii_lowercase().starts_with("@while") {
        return static_scss_while_loop_binding_frames(
            declaration,
            header,
            argument_values,
            fuel,
            context,
        );
    }
    None
}

fn static_scss_for_loop_binding_frames(
    header: &str,
    argument_values: &BTreeMap<String, String>,
    fuel: usize,
    context: StaticScssFunctionResolutionContext<'_>,
    position: usize,
) -> Option<Vec<Vec<(String, String)>>> {
    let parts = header.split_whitespace().collect::<Vec<_>>();
    let binding = parts.get(1)?.trim();
    if !binding.starts_with('$') {
        return None;
    }
    let from_index = parts
        .iter()
        .position(|part| part.eq_ignore_ascii_case("from"))?;
    let to_index = parts
        .iter()
        .position(|part| part.eq_ignore_ascii_case("to") || part.eq_ignore_ascii_case("through"))?;
    let includes_end = parts[to_index].eq_ignore_ascii_case("through");
    let start = parse_static_scss_for_loop_bound(
        parts.get(from_index + 1)?,
        argument_values,
        fuel,
        context,
        position,
    )?;
    let end = parse_static_scss_for_loop_bound(
        parts.get(to_index + 1)?,
        argument_values,
        fuel,
        context,
        position,
    )?;
    if start > end {
        return None;
    }
    let value_count = if includes_end {
        i64::from(end) - i64::from(start) + 1
    } else {
        i64::from(end) - i64::from(start)
    };
    if !(0..=64).contains(&value_count) {
        return None;
    }
    if value_count == 0 {
        return Some(Vec::new());
    }
    let last = if includes_end {
        end
    } else {
        end.saturating_sub(1)
    };
    Some(
        (start..=last)
            .map(|value| vec![(binding.to_string(), value.to_string())])
            .collect(),
    )
}

fn parse_static_scss_for_loop_bound(
    value: &str,
    argument_values: &BTreeMap<String, String>,
    fuel: usize,
    context: StaticScssFunctionResolutionContext<'_>,
    position: usize,
) -> Option<i32> {
    let resolution = resolve_static_scss_function_value_with_bindings(
        value,
        argument_values,
        position,
        fuel,
        context,
    );
    resolution.rendered_value?.parse::<i32>().ok()
}

fn static_scss_while_loop_binding_frames(
    declaration: &StaticScssFunctionDeclaration,
    header: &StaticScssLoopHeader,
    argument_values: &BTreeMap<String, String>,
    fuel: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<Vec<Vec<(String, String)>>> {
    let condition = static_scss_while_condition(header.text.as_str())?;
    if header.body_start >= header.body_end {
        return None;
    }
    let mut frames = Vec::new();
    let mut current_values = argument_values.clone();
    let body_end_position = header.body_end.saturating_sub(1);

    for _ in 0..64 {
        let active = static_scss_while_condition_is_active(
            condition,
            &current_values,
            header.span_start,
            fuel,
            context,
        )?;
        if !active {
            return Some(frames);
        }
        frames.push(
            current_values
                .iter()
                .map(|(name, value)| (name.clone(), value.clone()))
                .collect(),
        );
        let next_values = bind_static_scss_function_local_variables_in_range(
            declaration,
            &current_values,
            header.body_start,
            body_end_position,
            fuel,
            context,
        )
        .ok()?;
        if next_values == current_values {
            return None;
        }
        current_values = next_values;
    }

    None
}

fn static_scss_while_condition(header: &str) -> Option<&str> {
    let trimmed = header.trim_start();
    let keyword = trimmed.get(.."@while".len())?;
    if !keyword.eq_ignore_ascii_case("@while") {
        return None;
    }
    Some(trimmed.get("@while".len()..)?.trim()).filter(|condition| !condition.is_empty())
}

fn static_scss_while_condition_is_active(
    condition: &str,
    argument_values: &BTreeMap<String, String>,
    position: usize,
    fuel: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<bool> {
    let resolution = resolve_static_scss_function_value_with_bindings(
        condition,
        argument_values,
        position,
        fuel,
        context,
    );
    if resolution.outcome == StaticStylesheetResolutionOutcome::Top {
        return None;
    }
    let condition_value = resolution.rendered_value?;
    static_scss_literal_truthiness(condition_value.as_str())
}

fn static_scss_function_return_clauses_are_safe(
    clauses: &[StaticScssFunctionReturnClause],
) -> bool {
    !clauses.is_empty()
        && clauses.iter().all(|clause| {
            static_stylesheet_composite_value_is_safe(clause.value.as_str())
                && clause
                    .condition
                    .as_deref()
                    .is_none_or(static_stylesheet_composite_value_is_safe)
                && clause
                    .loop_headers
                    .iter()
                    .all(|header| static_stylesheet_composite_value_is_safe(header.text.as_str()))
        })
}

fn resolve_static_scss_function_value_with_bindings(
    value: &str,
    argument_values: &BTreeMap<String, String>,
    fallback_position: usize,
    fuel: usize,
    context: StaticScssFunctionResolutionContext<'_>,
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
        return resolve_static_scss_known_function_calls_in_value(
            value,
            argument_values,
            fallback_position,
            fuel,
            context,
        );
    }

    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    let mut stack = BTreeSet::new();
    for reference in references {
        let canonical_name = canonical_static_scss_variable_name(reference.name.as_str());
        let resolved = if let Some(argument_value) = argument_values.get(&canonical_name) {
            evaluate_static_scss_function_output_value(argument_value.as_str())
        } else {
            resolve_static_scss_variable_abstract_value_at_position(
                reference.name.as_str(),
                fallback_position,
                context.scopes,
                context.variable_declarations,
                &mut stack,
                fuel,
            )
        };
        let Some(rendered_value) = resolved.rendered_value else {
            return top_static_abstract_value(resolved.reason);
        };
        output.push_str(&value[cursor..reference.start]);
        output.push_str(&rendered_value);
        cursor = reference.end;
    }
    output.push_str(&value[cursor..]);
    resolve_static_scss_known_function_calls_in_value(
        output.as_str(),
        argument_values,
        fallback_position,
        fuel,
        context,
    )
}

fn resolve_static_scss_known_function_calls_in_value(
    value: &str,
    argument_values: &BTreeMap<String, String>,
    position: usize,
    fuel: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> StaticStylesheetAbstractResolution {
    if fuel == 0 {
        return top_static_abstract_value(StaticStylesheetResolutionReason::FuelExhausted);
    }
    let declaration_names = context
        .declarations
        .iter()
        .map(|declaration| canonical_static_scss_function_name(declaration.name.as_str()))
        .collect::<BTreeSet<_>>();
    let lexed = lex(value, StyleDialect::Scss);
    let tokens = lexed.tokens();
    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    let mut index = 0usize;
    let mut replaced_any = false;

    while index < tokens.len() {
        let token = &tokens[index];
        if token.kind != SyntaxKind::Ident || token.text.eq_ignore_ascii_case("if") {
            index += 1;
            continue;
        }
        let canonical_name = canonical_static_scss_function_name(token.text.as_str());
        if !declaration_names.contains(&canonical_name) {
            index += 1;
            continue;
        }
        let open_index = static_stylesheet_skip_trivia_tokens(tokens, index + 1);
        if tokens
            .get(open_index)
            .is_none_or(|candidate| candidate.kind != SyntaxKind::LeftParen)
        {
            index += 1;
            continue;
        }
        let Some(close_index) = static_stylesheet_matching_token_index(
            tokens,
            open_index,
            SyntaxKind::LeftParen,
            SyntaxKind::RightParen,
        ) else {
            return raw_static_abstract_value(
                value,
                StaticStylesheetResolutionReason::UnsupportedDynamic,
            );
        };
        let call_start = static_stylesheet_token_start(token);
        let call_end = static_stylesheet_token_end(&tokens[close_index]);
        let Some(argument_text) = value.get(
            static_stylesheet_token_end(&tokens[open_index])
                ..static_stylesheet_token_start(&tokens[close_index]),
        ) else {
            return raw_static_abstract_value(
                value,
                StaticStylesheetResolutionReason::UnsupportedDynamic,
            );
        };
        let Some(arguments) = split_static_scss_function_arguments(argument_text) else {
            return raw_static_abstract_value(
                value,
                StaticStylesheetResolutionReason::UnsupportedDynamic,
            );
        };
        let nested_call = StaticScssFunctionCall {
            name: token.text.clone(),
            start: usize::MAX,
            end: usize::MAX,
            arguments,
        };
        let resolution = resolve_static_scss_function_call_abstract_value_with_stack(
            &nested_call,
            context.declarations,
            context.mixin_declarations,
            context.scopes,
            context.variable_declarations,
            fuel - 1,
            context.active_functions,
        );
        if resolution.outcome == StaticStylesheetResolutionOutcome::Top {
            return top_static_abstract_value(resolution.reason);
        }
        if resolution.outcome == StaticStylesheetResolutionOutcome::Raw {
            return raw_static_abstract_value(value, resolution.reason);
        }
        let Some(rendered_value) = resolution.rendered_value else {
            return top_static_abstract_value(resolution.reason);
        };
        output.push_str(&value[cursor..call_start]);
        output.push_str(rendered_value.as_str());
        cursor = call_end;
        replaced_any = true;
        index = close_index + 1;
    }

    if !replaced_any {
        return evaluate_static_scss_function_output_value_with_context(
            value,
            argument_values,
            position,
            context,
        );
    }
    output.push_str(&value[cursor..]);
    evaluate_static_scss_function_output_value_with_context(
        output.as_str(),
        argument_values,
        position,
        context,
    )
}

fn evaluate_static_scss_function_output_value_with_context(
    value: &str,
    argument_values: &BTreeMap<String, String>,
    position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> StaticStylesheetAbstractResolution {
    let reduced_context_value = reduce_static_scss_metadata_with_function_context(
        value,
        argument_values,
        position,
        context,
    )
    .unwrap_or_else(|| value.to_string());
    evaluate_static_scss_function_output_value(reduced_context_value.as_str())
}

fn evaluate_static_scss_function_output_value(value: &str) -> StaticStylesheetAbstractResolution {
    if !static_stylesheet_composite_value_is_safe(value) {
        return raw_static_abstract_value(
            value,
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        );
    }
    let rendered_value = reduce_static_scss_value(value.to_string());
    let abstract_value = abstract_css_value_from_text(rendered_value.as_str());
    if matches!(abstract_value, AbstractCssValueV0::Raw { .. })
        && static_scss_function_value_contains_any_callable(rendered_value.as_str())
    {
        return raw_static_abstract_value(
            value,
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        );
    }
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
        rendered_value: Some(rendered_value),
        abstract_value,
        outcome,
        reason,
    }
}

fn reduce_static_scss_metadata_with_function_context(
    value: &str,
    argument_values: &BTreeMap<String, String>,
    position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<String> {
    reduce_static_scss_metadata_with_context(
        value,
        |name| {
            static_scss_visible_function_declaration_exists(name, position, context).then_some(true)
        },
        |name| {
            static_scss_visible_mixin_declaration_exists(name, position, context).then_some(true)
        },
        |name| {
            Some(static_scss_visible_variable_exists(
                name,
                position,
                argument_values,
                context,
            ))
        },
        |name| {
            Some(static_scss_visible_global_variable_exists(
                name, position, context,
            ))
        },
    )
}

fn static_scss_visible_function_declaration_exists(
    name: &str,
    position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> bool {
    context.declarations.iter().any(|declaration| {
        declaration.span_start <= position
            && canonical_static_scss_function_name(declaration.name.as_str()) == name
    })
}

fn static_scss_visible_mixin_declaration_exists(
    name: &str,
    position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> bool {
    context.mixin_declarations.iter().any(|declaration| {
        declaration.span_start <= position
            && canonical_static_scss_function_name(declaration.name.as_str()) == name
    })
}

fn static_scss_visible_variable_exists(
    name: &str,
    position: usize,
    argument_values: &BTreeMap<String, String>,
    context: StaticScssFunctionResolutionContext<'_>,
) -> bool {
    argument_values.contains_key(canonical_static_scss_variable_name(name).as_str())
        || static_stylesheet_scope_for_position(context.scopes, position)
            .and_then(|scope_id| {
                find_static_scss_variable_declaration(
                    name,
                    scope_id,
                    position,
                    context.scopes,
                    context.variable_declarations,
                )
            })
            .is_some()
}

fn static_scss_visible_global_variable_exists(
    name: &str,
    position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> bool {
    find_static_scss_variable_declaration_in_scope(
        name,
        0,
        position,
        context.scopes,
        context.variable_declarations,
    )
    .is_some()
}

fn static_scss_function_value_contains_any_callable(value: &str) -> bool {
    let lexed = lex(value, StyleDialect::Scss);
    let tokens = lexed.tokens();
    tokens.iter().enumerate().any(|(index, token)| {
        token.kind == SyntaxKind::Ident
            && !token.text.eq_ignore_ascii_case("if")
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
    let mixin_declaration_ranges = collect_static_scss_mixin_declaration_ranges(source);
    for fact in variable_facts {
        if fact.kind != ParsedVariableFactKind::ScssDeclaration {
            continue;
        }
        let start = parser_text_size_to_usize(fact.range.start().into());
        let end = parser_text_size_to_usize(fact.range.end().into());
        if static_stylesheet_variable_reference_is_named_argument_label(source, start, end) {
            continue;
        }
        if static_stylesheet_position_is_inside_ranges(start, &module_rule_ranges)
            || static_stylesheet_position_is_inside_ranges(start, &function_declaration_ranges)
            || static_stylesheet_position_is_inside_ranges(start, &mixin_declaration_ranges)
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
    collect_static_scss_block_at_rule_ranges(source, "@function")
}

fn collect_static_scss_mixin_declaration_ranges(source: &str) -> Vec<(usize, usize)> {
    collect_static_scss_block_at_rule_ranges(source, "@mixin")
}

fn collect_static_scss_block_at_rule_ranges(
    source: &str,
    at_rule_name: &str,
) -> Vec<(usize, usize)> {
    let lexed = lex(source, StyleDialect::Scss);
    let tokens = lexed.tokens();
    let mut ranges = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        if tokens[index].kind != SyntaxKind::AtKeyword
            || !tokens[index].text.eq_ignore_ascii_case(at_rule_name)
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

fn static_scss_mixin_declaration_ranges_from_declarations(
    declarations: &[StaticScssMixinDeclaration],
) -> Vec<(usize, usize)> {
    declarations
        .iter()
        .map(|declaration| (declaration.span_start, declaration.span_end))
        .collect()
}

fn static_less_mixin_declaration_ranges_from_declarations(
    declarations: &[StaticLessMixinDeclaration],
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
    excluded_ranges: &[(usize, usize)],
) -> Option<BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>> {
    let mut declarations = BTreeMap::<(usize, String), StaticStylesheetVariableDeclaration>::new();
    for fact in variable_facts {
        if fact.kind != ParsedVariableFactKind::LessDeclaration {
            continue;
        }
        let start = parser_text_size_to_usize(fact.range.start().into());
        let end = parser_text_size_to_usize(fact.range.end().into());
        if static_stylesheet_variable_reference_is_named_argument_label(source, start, end) {
            continue;
        }
        if static_stylesheet_position_is_inside_ranges(start, excluded_ranges) {
            continue;
        }
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
    collect_static_less_property_declarations_with_body_start(source, tokens, scopes, false)
}

fn collect_static_less_body_property_declarations(
    source: &str,
    tokens: &[LexedToken],
    scopes: &[StaticStylesheetScope],
) -> Option<BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>> {
    collect_static_less_property_declarations_with_body_start(source, tokens, scopes, true)
}

fn collect_static_less_property_declarations_with_body_start(
    source: &str,
    tokens: &[LexedToken],
    scopes: &[StaticStylesheetScope],
    allow_body_start: bool,
) -> Option<BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>> {
    let mut declarations = BTreeMap::<(usize, String), StaticStylesheetPropertyDeclaration>::new();
    let mut index = 0usize;
    while index < tokens.len() {
        if !matches!(
            tokens[index].kind,
            SyntaxKind::Ident | SyntaxKind::CustomPropertyName
        ) || !static_stylesheet_property_name_is_safe(tokens[index].text.as_str())
            || !(static_stylesheet_previous_token_starts_declaration(tokens, index)
                || (allow_body_start
                    && static_stylesheet_previous_token_is_body_start(tokens, index)))
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
        let metadata_reduced_value = reduce_static_scss_metadata_with_variable_context(
            value,
            position,
            scopes,
            declarations,
        )
        .unwrap_or_else(|| value.to_string());
        let reduced = reduce_static_scss_value(metadata_reduced_value.clone());
        if static_stylesheet_literal_value_is_safe(reduced.as_str()) {
            return resolved_static_abstract_value_preserving_callable_raw(value, reduced.as_str());
        }
        return raw_static_abstract_value(
            metadata_reduced_value.as_str(),
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
    let output = reduce_static_scss_metadata_with_variable_context(
        output.as_str(),
        position,
        scopes,
        declarations,
    )
    .unwrap_or(output);
    let reduced_output = reduce_static_scss_value(output.clone());
    resolved_static_abstract_value_preserving_callable_raw(output.as_str(), reduced_output.as_str())
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

fn reduce_static_scss_metadata_with_variable_context(
    value: &str,
    position: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &[StaticStylesheetScopedVariableDeclaration],
) -> Option<String> {
    reduce_static_scss_metadata_with_context(
        value,
        |_| None,
        |_| None,
        |name| {
            Some(
                static_stylesheet_scope_for_position(scopes, position)
                    .and_then(|scope_id| {
                        find_static_scss_variable_declaration(
                            name,
                            scope_id,
                            position,
                            scopes,
                            declarations,
                        )
                    })
                    .is_some(),
            )
        },
        |name| {
            Some(
                find_static_scss_variable_declaration_in_scope(
                    name,
                    0,
                    position,
                    scopes,
                    declarations,
                )
                .is_some(),
            )
        },
    )
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
        let value = reduce_static_scss_metadata_with_variable_context(
            value,
            position,
            scopes,
            declarations,
        )
        .unwrap_or_else(|| value.to_string());
        let reduced = reduce_static_scss_value(value);
        return static_stylesheet_literal_value_is_safe(reduced.as_str()).then_some(reduced);
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
    let output = reduce_static_scss_metadata_with_variable_context(
        output.as_str(),
        position,
        scopes,
        declarations,
    )
    .unwrap_or(output);
    Some(reduce_static_scss_value(output))
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
            reduce_static_less_value(rendered_value.to_string()).as_str(),
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
            return resolved_static_abstract_value(
                reduce_static_less_value(value.to_string()).as_str(),
            );
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
    resolved_static_abstract_value(reduce_static_less_value(output).as_str())
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
    resolved.map(reduce_static_less_value)
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
        return static_stylesheet_literal_value_is_safe(value)
            .then(|| reduce_static_less_value(value.to_string()));
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
    Some(reduce_static_less_value(output))
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
    if let Some(rendered_value) = resolved.rendered_value.as_deref() {
        return resolved_static_abstract_value(
            reduce_static_less_value(rendered_value.to_string()).as_str(),
        );
    }
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
            return resolved_static_abstract_value(
                reduce_static_less_value(value.to_string()).as_str(),
            );
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
    resolved_static_abstract_value(reduce_static_less_value(output).as_str())
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
    resolved.map(reduce_static_less_value)
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
        return static_stylesheet_literal_value_is_safe(value)
            .then(|| reduce_static_less_value(value.to_string()));
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
    Some(reduce_static_less_value(output))
}

fn collect_static_less_literal_value_edits(
    style_source: &str,
    tokens: &[LexedToken],
    declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    mixin_declaration_ranges: &[(usize, usize)],
) -> Option<Vec<StaticStylesheetEvaluationEdit>> {
    let declaration_removal_ranges = declarations
        .values()
        .flat_map(|declaration| declaration.removal_spans.iter().copied())
        .collect::<Vec<_>>();
    let mut edits = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::LessEscapedString {
            continue;
        }
        let start = static_stylesheet_token_start(token);
        if static_stylesheet_position_is_inside_ranges(start, &declaration_removal_ranges)
            || static_stylesheet_position_is_inside_ranges(start, mixin_declaration_ranges)
            || !static_less_escaped_string_token_is_declaration_value(tokens, index)
        {
            continue;
        }
        let end = static_stylesheet_token_end(token);
        let value = style_source.get(start..end)?;
        if !static_stylesheet_literal_value_is_safe(value) {
            continue;
        }
        let replacement = reduce_static_less_value(value.to_string());
        if replacement != value {
            edits.push(StaticStylesheetEvaluationEdit {
                start,
                end,
                replacement,
            });
        }
    }
    Some(edits)
}

fn static_less_escaped_string_token_is_declaration_value(
    tokens: &[LexedToken],
    token_index: usize,
) -> bool {
    let mut index = token_index;
    while index > 0 {
        index -= 1;
        let kind = tokens[index].kind;
        if static_stylesheet_token_is_trivia(kind) {
            continue;
        }
        match kind {
            SyntaxKind::Colon => return true,
            SyntaxKind::LeftBrace | SyntaxKind::RightBrace | SyntaxKind::Semicolon => return false,
            _ => {}
        }
    }
    false
}

fn reduce_static_less_value(value: String) -> String {
    let value = reduce_static_numeric_value(value);
    reduce_static_less_escaped_string_value(value.as_str()).unwrap_or(value)
}

fn reduce_static_less_escaped_string_value(value: &str) -> Option<String> {
    let trimmed = value.trim();
    let rest = trimmed.strip_prefix('~')?;
    let quote = rest.chars().next()?;
    if !matches!(quote, '"' | '\'') {
        return None;
    }

    let mut output = String::new();
    let mut index = quote.len_utf8();
    while index < rest.len() {
        let ch = rest[index..].chars().next()?;
        if matches!(ch, '\n' | '\r' | '\u{000c}') {
            return None;
        }
        if ch == quote {
            return (index + ch.len_utf8() == rest.len()).then_some(output);
        }
        if ch == '\\' {
            index += ch.len_utf8();
            let escaped = rest[index..].chars().next()?;
            if matches!(escaped, '\n' | '\r' | '\u{000c}') {
                return None;
            }
            output.push(escaped);
            index += escaped.len_utf8();
            continue;
        }
        output.push(ch);
        index += ch.len_utf8();
    }
    None
}

fn static_stylesheet_less_declaration_value_is_removal_safe(value: &str) -> bool {
    !value.chars().any(|ch| matches!(ch, '{' | '}' | ';' | '!'))
}

fn static_stylesheet_scss_declaration_value_is_removal_safe(value: &str) -> bool {
    !value.chars().any(|ch| matches!(ch, '{' | '}' | ';'))
        && static_scss_bang_usage_is_comparison_only(value)
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

fn static_stylesheet_previous_token_is_body_start(tokens: &[LexedToken], index: usize) -> bool {
    tokens[..index]
        .iter()
        .rev()
        .all(|token| static_stylesheet_token_is_trivia(token.kind))
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
            .any(|ch| matches!(ch, '{' | '}' | ';' | '$' | '@'))
        && static_scss_bang_usage_is_comparison_only(value)
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

fn static_less_mixin_name_part_is_safe(name: &str) -> bool {
    static_stylesheet_property_name_is_safe(name)
}

fn static_less_mixin_hash_name_is_safe(name: &str) -> bool {
    name.strip_prefix('#')
        .is_some_and(static_stylesheet_property_name_is_safe)
}

fn static_less_variable_name_is_safe(name: &str) -> bool {
    name.strip_prefix('@')
        .is_some_and(static_stylesheet_variable_name_is_safe)
}

fn static_scss_function_argument_is_safe(value: &str) -> bool {
    !value.is_empty()
        && !value.contains("...")
        && !value.chars().any(|ch| matches!(ch, '{' | '}' | ';' | ':'))
        && static_scss_bang_usage_is_comparison_only(value)
}

fn static_less_mixin_argument_value_is_safe(value: &str) -> bool {
    !value.is_empty()
        && !value.contains("...")
        && !value.chars().any(|ch| matches!(ch, '{' | '}' | ';'))
}

fn static_scss_mixin_body_is_static_declaration_subset(body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    !body.chars().any(|ch| matches!(ch, '{' | '}'))
        && !lower.contains("@content")
        && !lower.contains("@mixin")
        && !lower.contains("@function")
        && !lower.contains("@return")
        && !lower.contains("@if")
        && !lower.contains("@for")
        && !lower.contains("@each")
        && !lower.contains("@while")
}

fn static_less_mixin_body_is_static_declaration_subset(body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    !body.chars().any(|ch| matches!(ch, '{' | '}'))
        && !lower.contains("when")
        && !lower.contains(":extend")
        && !lower.contains("@plugin")
        && !lower.contains("@import")
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

fn canonical_static_less_mixin_name(name: &str) -> String {
    name.trim().to_string()
}

pub fn static_scss_variable_names_equal(left: &str, right: &str) -> bool {
    canonical_static_scss_variable_name(left) == canonical_static_scss_variable_name(right)
}

fn static_stylesheet_composite_value_is_safe(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && !value.chars().any(|ch| matches!(ch, '{' | '}' | ';'))
        && static_scss_bang_usage_is_comparison_only(value)
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
    collect_static_stylesheet_variable_references_with_options(value, variable_kind, false)
}

fn collect_static_stylesheet_variable_references_with_options(
    value: &str,
    variable_kind: StaticStylesheetVariableKind,
    allow_scss_include_at_keyword: bool,
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
            if allow_scss_include_at_keyword
                && variable_kind == StaticStylesheetVariableKind::Scss
                && static_scss_at_keyword_prefix_is_include(value, index)
            {
                index += "@include".len();
                continue;
            }
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
        if static_stylesheet_variable_reference_is_named_argument_label(value, index, name_end) {
            index = name_end;
            continue;
        }
        if variable_kind == StaticStylesheetVariableKind::Scss
            && static_stylesheet_position_is_scss_module_member_reference(value, index)
        {
            index = name_end;
            continue;
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

fn static_stylesheet_position_is_scss_module_member_reference(value: &str, start: usize) -> bool {
    value
        .get(..start)
        .and_then(|prefix| prefix.chars().next_back())
        .is_some_and(|ch| ch == '.')
}

fn static_scss_at_keyword_prefix_is_include(value: &str, index: usize) -> bool {
    let Some(candidate) = value.get(index..index + "@include".len()) else {
        return false;
    };
    if !candidate.eq_ignore_ascii_case("@include") {
        return false;
    }
    value
        .get(index + "@include".len()..)
        .and_then(|suffix| suffix.chars().next())
        .is_some_and(|ch| ch.is_ascii_whitespace())
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

fn static_stylesheet_variable_reference_is_named_argument_label(
    value: &str,
    start: usize,
    mut index: usize,
) -> bool {
    if !value
        .get(..start)
        .and_then(|prefix| {
            prefix
                .chars()
                .rev()
                .find(|candidate| !candidate.is_ascii_whitespace())
        })
        .is_some_and(|ch| matches!(ch, '(' | ','))
    {
        return false;
    }
    while index < value.len() {
        let Some(ch) = value[index..].chars().next() else {
            return false;
        };
        if ch == ':' {
            return true;
        }
        if !ch.is_ascii_whitespace() {
            return false;
        }
        index += ch.len_utf8();
    }
    false
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
    edits.dedup_by(|left, right| {
        left.start == right.start && left.end == right.end && left.replacement == right.replacement
    });
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
    fn static_stylesheet_oracle_corpus_preserves_legacy_output() {
        let report = summarize_static_stylesheet_oracle_corpus();

        assert_eq!(
            report.product,
            "omena-scss-eval.static-stylesheet-oracle-corpus"
        );
        assert_eq!(report.mode, "oracleOnly");
        assert_eq!(report.value_type, "AbstractCssValueV0");
        assert_eq!(report.product_output_source, "legacyEvaluatedCss");
        assert_eq!(report.fixture_count, 12);
        assert_eq!(report.scss_fixture_count, 6);
        assert_eq!(report.less_fixture_count, 6);
        assert_eq!(report.evaluated_fixture_count, report.fixture_count);
        assert_eq!(report.missing_evaluation_count, 0);
        assert_eq!(report.divergence_count, 0);
        assert!(report.native_replacement_count > 0);
        assert!(report.native_value_reference_count > 0);
        assert!(report.native_resolved_value_count > 0);
        assert_eq!(report.native_top_value_count, 0);
        assert!(report.all_legacy_declaration_values_preserved);
        assert!(
            report
                .fixtures
                .iter()
                .all(|fixture| fixture.legacy_output_consumed_until_cutover)
        );
    }

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
    fn static_less_evaluation_reduces_escaped_string_variable_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "@filter: ~\"alpha(opacity=50)\"; .button { filter: @filter; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.resolved_replacements[0].text, "alpha(opacity=50)");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "raw");
        assert_eq!(
            report.value_resolution.values[0].rendered_value.as_deref(),
            Some("alpha(opacity=50)")
        );
        assert!(report.evaluated_css.contains("filter: alpha(opacity=50)"));
        assert!(!report.evaluated_css.contains("~\"alpha"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_expands_static_mixin_calls() {
        let report = derive_static_stylesheet_module_evaluation(
            "@brand: red; .tone(@color, @gap: 1px) { color: @color; margin: @gap; padding: @brand; } .button { .tone(blue, 2px); }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains(".tone(@color"));
        assert!(!report.evaluated_css.contains(".tone(blue"));
        assert!(report.evaluated_css.contains("color: blue"));
        assert!(report.evaluated_css.contains("margin: 2px"));
        assert!(report.evaluated_css.contains("padding: red"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_expands_hash_mixin_calls() {
        let report = derive_static_stylesheet_module_evaluation(
            "#tone(@color, @gap: 1px) { color: @color; margin: @gap; } .button { #tone(red, 2px); }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains("#tone(@color"));
        assert!(!report.evaluated_css.contains("#tone(red"));
        assert!(report.evaluated_css.contains("color: red"));
        assert!(report.evaluated_css.contains("margin: 2px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_expands_mixin_declaration_accessors() {
        let report = derive_static_stylesheet_module_evaluation(
            ".tokens(@color, @gap: 1px) { @result: @color; width: @gap; } .button { color: .tokens(red)[@result]; margin: .tokens(red, 2px)[width]; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains(".tokens(@color"));
        assert!(!report.evaluated_css.contains(".tokens(red)[@result]"));
        assert!(!report.evaluated_css.contains(".tokens(red, 2px)[width]"));
        assert!(report.evaluated_css.contains("color: red"));
        assert!(report.evaluated_css.contains("margin: 2px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_keeps_unknown_mixin_accessor_members_planned_only() {
        let report = derive_static_stylesheet_module_evaluation(
            ".tokens(@color) { @result: @color; } .button { color: .tokens(red)[@missing]; }",
            StyleDialect::Less,
        );

        assert!(report.is_none());
    }

    #[test]
    fn static_less_evaluation_expands_namespace_mixin_access() {
        let report = derive_static_stylesheet_module_evaluation(
            "#bundle() { .rounded(@radius) { border-radius: @radius; } } .button { #bundle > .rounded(2px); }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains("#bundle()"));
        assert!(!report.evaluated_css.contains("#bundle > .rounded"));
        assert!(report.evaluated_css.contains("border-radius: 2px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_expands_parameterized_namespace_mixin_access() {
        let report = derive_static_stylesheet_module_evaluation(
            "#bundle(@color) { .tone() { color: @color; } } .button { #bundle(red) > .tone(); }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains("#bundle(@color"));
        assert!(!report.evaluated_css.contains("#bundle(red) > .tone"));
        assert!(report.evaluated_css.contains("color: red"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_expands_guarded_namespace_mixin_access() {
        let report = derive_static_stylesheet_module_evaluation(
            "#bundle() when (iscolor(red)) { .tone() { color: red; } } .button { #bundle > .tone(); }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains("#bundle()"));
        assert!(!report.evaluated_css.contains("#bundle > .tone"));
        assert!(report.evaluated_css.contains("color: red"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_keeps_false_guarded_namespace_mixin_access_planned_only() {
        let report = derive_static_stylesheet_module_evaluation(
            "#bundle() when (iscolor(1px)) { .tone() { color: red; } } .button { #bundle > .tone(); }",
            StyleDialect::Less,
        );

        assert!(report.is_none());
    }

    #[test]
    fn static_less_evaluation_expands_detached_ruleset_calls() {
        let report = derive_static_stylesheet_module_evaluation(
            "@brand: red; @rules: { color: @brand; margin: 1px; }; .button { @rules(); }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains("@rules:"));
        assert!(!report.evaluated_css.contains("@rules();"));
        assert!(report.evaluated_css.contains("color: red"));
        assert!(report.evaluated_css.contains("margin: 1px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_expands_ruleset_guarded_mixin_arguments() {
        let report = derive_static_stylesheet_module_evaluation(
            ".apply(@block) when (isruleset(@block)) { @block(); } @rules: { color: red; margin: 1px; }; .button { .apply(@rules); }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains(".apply(@block"));
        assert!(!report.evaluated_css.contains("@rules:"));
        assert!(!report.evaluated_css.contains(".apply(@rules"));
        assert!(!report.evaluated_css.contains("@block();"));
        assert!(report.evaluated_css.contains("color: red"));
        assert!(report.evaluated_css.contains("margin: 1px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_keeps_false_ruleset_guarded_mixins_planned_only() {
        let report = derive_static_stylesheet_module_evaluation(
            ".apply(@block) when (isruleset(@block)) { @block(); } .button { .apply(red); }",
            StyleDialect::Less,
        );

        assert!(report.is_none());
    }

    #[test]
    fn static_less_evaluation_expands_detached_ruleset_accessors() {
        let report = derive_static_stylesheet_module_evaluation(
            "@brand: red; @tokens: { primary: @brand; @gap: 2px; }; .button { color: @tokens[primary]; margin: @tokens[@gap]; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains("@tokens:"));
        assert!(!report.evaluated_css.contains("@tokens[primary]"));
        assert!(!report.evaluated_css.contains("@tokens[@gap]"));
        assert!(report.evaluated_css.contains("color: red"));
        assert!(report.evaluated_css.contains("margin: 2px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_keeps_unknown_detached_ruleset_accessor_members_planned_only() {
        let report = derive_static_stylesheet_module_evaluation(
            "@tokens: { primary: red; }; .button { color: @tokens[missing]; }",
            StyleDialect::Less,
        );

        assert!(report.is_none());
    }

    #[test]
    fn static_less_evaluation_expands_scoped_detached_ruleset_calls() {
        let report = derive_static_stylesheet_module_evaluation(
            "@rules: { color: red; }; .scope { @rules: { color: blue; }; .button { @rules(); } }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains("@rules:"));
        assert!(report.evaluated_css.contains("color: blue"));
        assert!(!report.evaluated_css.contains("color: red"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_expands_detached_rulesets_with_mixin_calls() {
        let report = derive_static_stylesheet_module_evaluation(
            ".rounded() { border-radius: 2px; } @rules: { .rounded(); }; .button { @rules(); }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains(".rounded()"));
        assert!(!report.evaluated_css.contains("@rules:"));
        assert!(!report.evaluated_css.contains("@rules();"));
        assert!(report.evaluated_css.contains("border-radius: 2px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_keeps_unknown_detached_ruleset_mixin_calls_planned_only() {
        let report = derive_static_stylesheet_module_evaluation(
            "@rules: { .unknown(); }; .button { @rules(); }",
            StyleDialect::Less,
        );

        assert!(report.is_none());
    }

    #[test]
    fn static_less_evaluation_keeps_unbound_parameterized_namespace_mixin_access_planned_only() {
        let report = derive_static_stylesheet_module_evaluation(
            "#bundle(@color) { .tone() { color: @color; } } .button { #bundle > .tone(); }",
            StyleDialect::Less,
        );

        assert!(report.is_none());
    }

    #[test]
    fn static_less_evaluation_reduces_escaped_string_mixin_arguments() {
        let report = derive_static_stylesheet_module_evaluation(
            ".legacy(@value) { filter: @value; } .button { .legacy(~\"alpha(opacity=50)\"); }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains(".legacy(@value"));
        assert!(!report.evaluated_css.contains(".legacy(~\"alpha"));
        assert!(report.evaluated_css.contains("filter: alpha(opacity=50)"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_expands_semicolon_separated_mixin_calls() {
        let report = derive_static_stylesheet_module_evaluation(
            ".shadow(@value; @color: red) { box-shadow: @value; color: @color; } .button { .shadow(1px, 2px, 3px; blue); }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains(".shadow(@value"));
        assert!(!report.evaluated_css.contains(".shadow(1px"));
        assert!(report.evaluated_css.contains("box-shadow: 1px, 2px, 3px"));
        assert!(report.evaluated_css.contains("color: blue"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_expands_variadic_mixin_arguments() {
        let report = derive_static_stylesheet_module_evaluation(
            ".shadow(@color; @rest...) { color: @color; box-shadow: @rest; trace: @arguments; } .button { .shadow(red; 1px, 2px, 3px); }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains(".shadow(@color"));
        assert!(!report.evaluated_css.contains(".shadow(red"));
        assert!(report.evaluated_css.contains("color: red"));
        assert!(report.evaluated_css.contains("box-shadow: 1px, 2px, 3px"));
        assert!(report.evaluated_css.contains("trace: red, 1px, 2px, 3px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_expands_literal_pattern_mixins() {
        let report = derive_static_stylesheet_module_evaluation(
            ".tone(dark, @color) { color: @color; background: black; } .tone(light, @color) { color: @color; background: white; } .button { .tone(dark, red); }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains(".tone(dark"));
        assert!(!report.evaluated_css.contains(".tone(light"));
        assert!(report.evaluated_css.contains("color: red"));
        assert!(report.evaluated_css.contains("background: black"));
        assert!(!report.evaluated_css.contains("background: white"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_keeps_unmatched_literal_pattern_mixins_planned_only() {
        let report = derive_static_stylesheet_module_evaluation(
            ".tone(dark, @color) { color: @color; background: black; } .button { .tone(light, red); }",
            StyleDialect::Less,
        );

        assert!(report.is_none());
    }

    #[test]
    fn static_less_evaluation_does_not_expand_variadic_tokens_in_calls() {
        let report = derive_static_stylesheet_module_evaluation(
            "@gap: 1px; .space(@value) { margin: @value; } .button { .space(@gap...); }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains(".space(1px...)"));
        assert!(!report.evaluated_css.contains("margin: 1px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_expands_important_mixin_calls() {
        let report = derive_static_stylesheet_module_evaluation(
            ".tone(@color, @gap: 1px) { color: @color; margin: @gap; } .button { .tone(red, 2px) !important; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains(".tone(@color"));
        assert!(!report.evaluated_css.contains(".tone(red"));
        assert!(report.evaluated_css.contains("color: red !important"));
        assert!(report.evaluated_css.contains("margin: 2px !important"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_does_not_expand_unknown_mixin_call_suffixes() {
        let report = derive_static_stylesheet_module_evaluation(
            ".tone(@color) { color: @color; } .button { .tone(red) !default; }",
            StyleDialect::Less,
        );

        assert!(report.is_none());
    }

    #[test]
    fn static_less_evaluation_expands_named_and_default_mixin_arguments() {
        let report = derive_static_stylesheet_module_evaluation(
            ".tone(@color: red, @gap: 1px, @double: 4px) { color: @color; margin: @gap; padding: @double; } .button { .tone(@gap: 2px, @color: blue); }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains(".tone(@color"));
        assert!(!report.evaluated_css.contains(".tone(@gap"));
        assert!(report.evaluated_css.contains("color: blue"));
        assert!(report.evaluated_css.contains("margin: 2px"));
        assert!(report.evaluated_css.contains("padding: 4px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_expands_mixin_local_variables() {
        let report = derive_static_stylesheet_module_evaluation(
            ".tone(@gap) { @space: (@gap * 2); margin: @space; } .button { .tone(2px); }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains("@space"));
        assert!(!report.evaluated_css.contains(".tone(@gap"));
        assert!(!report.evaluated_css.contains(".tone(2px"));
        assert!(report.evaluated_css.contains("margin: 4px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_expands_nested_static_mixin_calls() {
        let report = derive_static_stylesheet_module_evaluation(
            ".spacing(@gap) { margin: @gap; } .tone(@gap, @color: red) { .spacing(@gap); color: @color; } .button { .tone(2px, blue); }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains(".spacing(@gap"));
        assert!(!report.evaluated_css.contains(".tone(@gap"));
        assert!(!report.evaluated_css.contains(".spacing(2px"));
        assert!(!report.evaluated_css.contains(".tone(2px"));
        assert!(report.evaluated_css.contains("margin: 2px"));
        assert!(report.evaluated_css.contains("color: blue"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_skips_recursive_nested_mixin_calls() {
        let report = derive_static_stylesheet_module_evaluation(
            ".again() { .again(); } .button { .again(); }",
            StyleDialect::Less,
        );

        assert!(report.is_none());
    }

    #[test]
    fn static_less_evaluation_expands_static_guarded_mixin_calls() {
        let report = derive_static_stylesheet_module_evaluation(
            ".tone(@color) when (iscolor(@color)) { color: @color; } .button { .tone(red); }",
            StyleDialect::Less,
        );

        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains(".tone(@color"));
        assert!(!report.evaluated_css.contains(".tone(red"));
        assert!(report.evaluated_css.contains("color: red"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_treats_oklab_values_as_static_colors() {
        let report = derive_static_stylesheet_module_evaluation(
            ".tone(@color) when (iscolor(@color)) { color: @color; } .button { .tone(oklab(1 0 0)); }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains(".tone(@color"));
        assert!(!report.evaluated_css.contains(".tone(oklab"));
        assert!(report.evaluated_css.contains("color: oklab(1 0 0)"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_treats_rgb_values_as_static_colors() {
        let report = derive_static_stylesheet_module_evaluation(
            ".tone(@color) when (iscolor(@color)) { color: @color; } .button { .tone(rgb(127.5, 0, 127.5)); }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains(".tone(@color"));
        assert!(!report.evaluated_css.contains(".tone(rgb"));
        assert!(report.evaluated_css.contains("color: rgb(127.5, 0, 127.5)"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_expands_numeric_guarded_mixin_calls() {
        let report = derive_static_stylesheet_module_evaluation(
            ".space(@gap) when (isnumber(@gap)) { margin: @gap; } .button { .space(2px); }",
            StyleDialect::Less,
        );

        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains(".space(@gap"));
        assert!(!report.evaluated_css.contains(".space(2px"));
        assert!(report.evaluated_css.contains("margin: 2px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_expands_type_guarded_mixin_calls() {
        let report = derive_static_stylesheet_module_evaluation(
            r#".space(@gap) when (ispixel(@gap)) { margin: @gap; }
.ratio(@value) when (ispercentage(@value)) { width: @value; }
.font(@family) when (isstring(@family)) { font-family: @family; }
.display(@value) when (iskeyword(@value)) { display: @value; }
.asset(@value) when (isurl(@value)) { background-image: @value; }
.unit(@gap) when (isunit(@gap, "rem")) { padding: @gap; }
.button { .space(2px); .ratio(50%); .font("Roboto"); .display(block); .asset(url("./icon.svg")); .unit(1rem); }"#,
            StyleDialect::Less,
        );

        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains("margin: 2px"));
        assert!(report.evaluated_css.contains("width: 50%"));
        assert!(report.evaluated_css.contains(r#"font-family: "Roboto""#));
        assert!(report.evaluated_css.contains("display: block"));
        assert!(report.evaluated_css.contains("padding: 1rem"));
        assert!(
            report
                .evaluated_css
                .contains(r#"background-image: url("./icon.svg")"#)
        );
        assert!(!report.evaluated_css.contains(".space(2px"));
        assert!(!report.evaluated_css.contains(".asset(url"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_expands_comparison_guarded_mixin_calls() {
        let report = derive_static_stylesheet_module_evaluation(
            r#".space(@gap) when (@gap > 1px) { margin: @gap; }
.tone(@color) when (@color = red) { color: @color; }
.combo(@gap, @color) when (@gap >= 2px) and (iscolor(@color)) { padding: @gap; border-color: @color; }
.inverse(@gap) when not (@gap < 2px) { inset: @gap; }
.fallback(@name) when (@name = primary), (@name = secondary) { content: @name; }
.button { .space(2px); .tone(red); .combo(2px, blue); .inverse(2px); .fallback(secondary); }"#,
            StyleDialect::Less,
        );

        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains("margin: 2px"));
        assert!(report.evaluated_css.contains("color: red"));
        assert!(report.evaluated_css.contains("padding: 2px"));
        assert!(report.evaluated_css.contains("border-color: blue"));
        assert!(report.evaluated_css.contains("inset: 2px"));
        assert!(report.evaluated_css.contains("content: secondary"));
        assert!(!report.evaluated_css.contains(".space(2px"));
        assert!(!report.evaluated_css.contains(".fallback(secondary"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_expands_multiple_matching_guarded_mixins() {
        let report = derive_static_stylesheet_module_evaluation(
            r#".tone(@color) when (@color = blue) { outline-color: blue; }
.tone(@color) when (@color = red) { color: @color; }
.tone(@color) when (iscolor(@color)) { border-color: @color; }
.button { .tone(red); }"#,
            StyleDialect::Less,
        );

        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains("outline-color: blue"));
        assert!(report.evaluated_css.contains("color: red"));
        assert!(report.evaluated_css.contains("border-color: red"));
        assert!(!report.evaluated_css.contains(".tone(@color"));
        assert!(!report.evaluated_css.contains(".tone(red"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_expands_default_guarded_mixins() {
        let red_report = derive_static_stylesheet_module_evaluation(
            r#".tone(@color) when (@color = red) { color: @color; }
.tone(@color) when (default()) and (iscolor(@color)) { color: gray; }
.button { .tone(red); }"#,
            StyleDialect::Less,
        );
        assert!(red_report.is_some());
        let Some(red_report) = red_report else {
            return;
        };

        assert!(red_report.evaluated_css.contains("color: red"));
        assert!(!red_report.evaluated_css.contains("color: gray"));
        assert!(!red_report.evaluated_css.contains(".tone(@color"));
        assert!(!red_report.evaluated_css.contains(".tone(red"));
        assert!(red_report.oracle.all_legacy_declaration_values_preserved);

        let blue_report = derive_static_stylesheet_module_evaluation(
            r#".tone(@color) when (@color = red) { color: @color; }
.tone(@color) when (default()) and (iscolor(@color)) { color: gray; }
.button { .tone(blue); }"#,
            StyleDialect::Less,
        );
        assert!(blue_report.is_some());
        let Some(blue_report) = blue_report else {
            return;
        };

        assert!(blue_report.evaluated_css.contains("color: gray"));
        assert!(!blue_report.evaluated_css.contains("color: blue"));
        assert!(!blue_report.evaluated_css.contains(".tone(@color"));
        assert!(!blue_report.evaluated_css.contains(".tone(blue"));
        assert!(blue_report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_keeps_false_guarded_mixins_planned_only() {
        let report = derive_static_stylesheet_module_evaluation(
            ".tone(@value) when (iscolor(@value)) { color: @value; } .button { .tone(1px); }",
            StyleDialect::Less,
        );

        assert!(report.is_none());
    }

    #[test]
    fn static_less_evaluation_keeps_false_comparison_guarded_mixins_planned_only() {
        let report = derive_static_stylesheet_module_evaluation(
            ".space(@gap) when (@gap > 2px) { margin: @gap; } .button { .space(1px); }",
            StyleDialect::Less,
        );

        assert!(report.is_none());
    }

    #[test]
    fn static_less_evaluation_keeps_false_unit_guarded_mixins_planned_only() {
        let report = derive_static_stylesheet_module_evaluation(
            ".space(@gap) when (ispixel(@gap)) { margin: @gap; } .button { .space(2em); }",
            StyleDialect::Less,
        );

        assert!(report.is_none());
    }

    #[test]
    fn static_less_evaluation_keeps_false_isunit_guarded_mixins_planned_only() {
        let report = derive_static_stylesheet_module_evaluation(
            r#".space(@gap) when (isunit(@gap, "px")) { margin: @gap; } .button { .space(2em); }"#,
            StyleDialect::Less,
        );

        assert!(report.is_none());
    }

    #[test]
    fn static_scss_evaluation_uses_value_lattice_numeric_reduction() {
        let report = derive_static_stylesheet_module_evaluation(
            "$gap: (1px + 2px); .button { margin: $gap; }",
            StyleDialect::Scss,
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
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_bare_numeric_expressions() {
        let report = derive_static_stylesheet_module_evaluation(
            "$gap: 1px + 2px; .button { margin: $gap; }",
            StyleDialect::Scss,
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
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_calc_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "$gap: calc(1px + 2px); .button { margin: $gap; }",
            StyleDialect::Scss,
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
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_numeric_builtin_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "$gap: min(10px, 4px); $offset: clamp(1px, 3px, 2px); .button { margin: $gap; padding: $offset; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        let replacements = report
            .resolved_replacements
            .iter()
            .map(|replacement| replacement.text.as_str())
            .collect::<Vec<_>>();
        assert!(replacements.contains(&"4px"));
        assert!(replacements.contains(&"2px"));
        assert!(
            report
                .resolved_replacements
                .iter()
                .all(|replacement| replacement.abstract_value_kind == "exact")
        );
        assert!(report.evaluated_css.contains("margin: 4px"));
        assert!(report.evaluated_css.contains("padding: 2px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_if_function_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "$gap: if(false, 1px, 2px); .button { margin: $gap; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.resolved_replacements[0].text, "2px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { margin: 2px; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_nth_function_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "$gap: nth(1px 2px 3px, 2); $pad: list.nth((4px, 5px, 6px), -1); .button { margin: $gap; padding: $pad; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        let replacements = report
            .resolved_replacements
            .iter()
            .map(|replacement| replacement.text.as_str())
            .collect::<Vec<_>>();
        assert!(replacements.contains(&"2px"));
        assert!(replacements.contains(&"6px"));
        assert!(report.evaluated_css.contains("margin: 2px"));
        assert!(report.evaluated_css.contains("padding: 6px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_map_get_function_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "$gap: map-get((default: 2px, dense: 1px), default); $tone: map.get((primary: red, secondary: blue), secondary); .button { margin: $gap; color: $tone; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        let replacements = report
            .resolved_replacements
            .iter()
            .map(|replacement| replacement.text.as_str())
            .collect::<Vec<_>>();
        assert!(replacements.contains(&"2px"));
        assert!(replacements.contains(&"blue"));
        assert!(report.evaluated_css.contains("margin: 2px"));
        assert!(report.evaluated_css.contains("color: blue"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_nested_static_map_get_and_has_key_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "$weight: map.get((font: (weights: (regular: 400, medium: 500))), font, weights, medium); $tone: map-get((theme: (primary: red)), theme, primary); $has: if(map.has-key((theme: (primary: red)), theme, primary), 1px, 2px); $missing: if(map-has-key((theme: (primary: red)), theme, missing), 3px, 4px); .button { font-weight: $weight; color: $tone; margin: $has; padding: $missing; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        let replacements = report
            .resolved_replacements
            .iter()
            .map(|replacement| replacement.text.as_str())
            .collect::<Vec<_>>();
        assert!(replacements.contains(&"500"));
        assert!(replacements.contains(&"red"));
        assert!(replacements.contains(&"1px"));
        assert!(replacements.contains(&"4px"));
        assert!(report.evaluated_css.contains("font-weight: 500"));
        assert!(report.evaluated_css.contains("color: red"));
        assert!(report.evaluated_css.contains("margin: 1px"));
        assert!(report.evaluated_css.contains("padding: 4px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_collection_size_and_search_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "$count: list.length((1px, 2px, 3px)); $position: index(red blue green, green); .button { z-index: $count; order: $position; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        let replacements = report
            .resolved_replacements
            .iter()
            .map(|replacement| replacement.text.as_str())
            .collect::<Vec<_>>();
        assert!(replacements.contains(&"3"));
        assert!(report.evaluated_css.contains("z-index: 3"));
        assert!(report.evaluated_css.contains("order: 3"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_list_metadata_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "$separator: list.separator((1px, 2px)); $legacy-separator: list-separator(1px 2px); $space: if(list-separator(1px 2px) == \"space\", 1px, 2px); $bracketed: if(list.is-bracketed([1px 2px]), 3px, 4px); $legacy-bracketed: if(is-bracketed([1px 2px]), 5px, 6px); .button { content: $separator; quotes: $legacy-separator; margin: $space; padding: $bracketed; inset: $legacy-bracketed; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        let replacements = report
            .resolved_replacements
            .iter()
            .map(|replacement| replacement.text.as_str())
            .collect::<Vec<_>>();
        assert!(replacements.contains(&"\"comma\""));
        assert!(replacements.contains(&"\"space\""));
        assert!(replacements.contains(&"1px"));
        assert!(replacements.contains(&"3px"));
        assert!(replacements.contains(&"5px"));
        assert!(report.evaluated_css.contains("content: \"comma\""));
        assert!(report.evaluated_css.contains("quotes: \"space\""));
        assert!(report.evaluated_css.contains("margin: 1px"));
        assert!(report.evaluated_css.contains("padding: 3px"));
        assert!(report.evaluated_css.contains("inset: 5px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_string_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "$family: string.quote(Demo); $style: unquote(\"serif\"); $length: string.length(\"Helvetica Neue\"); $position: str-index(\"Helvetica Neue\", \"Neue\"); $slice: string.slice(\"Helvetica Neue\", 1, -6); $inserted: string.insert(\"Roboto Bold\", \" Mono\", 7); $upper: to-upper-case(sans-serif); $lower: string.to-lower-case(\"BOLD\"); .button { font-family: $family, $style; z-index: $length; order: $position; content: $slice; src: $inserted; text-transform: $upper; font-style: $lower; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        let rendered_values = report
            .resolved_replacements
            .iter()
            .filter_map(|replacement| replacement.rendered_value.as_deref())
            .collect::<Vec<_>>();
        assert!(rendered_values.contains(&"\"Demo\""));
        assert!(rendered_values.contains(&"serif"));
        assert!(rendered_values.contains(&"14"));
        assert!(rendered_values.contains(&"11"));
        assert!(rendered_values.contains(&"\"Helvetica\""));
        assert!(rendered_values.contains(&"\"Roboto Mono Bold\""));
        assert!(rendered_values.contains(&"SANS-SERIF"));
        assert!(rendered_values.contains(&"\"bold\""));
        assert!(
            report
                .evaluated_css
                .contains("font-family: \"Demo\", serif")
        );
        assert!(report.evaluated_css.contains("z-index: 14"));
        assert!(report.evaluated_css.contains("order: 11"));
        assert!(report.evaluated_css.contains("content: \"Helvetica\""));
        assert!(report.evaluated_css.contains("src: \"Roboto Mono Bold\""));
        assert!(report.evaluated_css.contains("text-transform: SANS-SERIF"));
        assert!(report.evaluated_css.contains("font-style: \"bold\""));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_map_has_key_conditions() {
        let report = derive_static_stylesheet_module_evaluation(
            "$gap: if(map.has-key((default: 2px, dense: 1px), dense), 1px, 2px); $pad: if(map-has-key((default: 2px), missing), 3px, 4px); .button { margin: $gap; padding: $pad; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        let replacements = report
            .resolved_replacements
            .iter()
            .map(|replacement| replacement.text.as_str())
            .collect::<Vec<_>>();
        assert!(replacements.contains(&"1px"));
        assert!(replacements.contains(&"4px"));
        assert!(report.evaluated_css.contains("margin: 1px"));
        assert!(report.evaluated_css.contains("padding: 4px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_map_key_and_value_lists() {
        let report = derive_static_stylesheet_module_evaluation(
            "$key-count: list.length(map.keys((default: 1px, dense: 2px))); $first-value: list.nth(map.values((default: 1px, dense: 2px)), 1); $legacy-key-count: length(map-keys((primary: red, secondary: blue))); $legacy-value: nth(map-values((primary: red, secondary: blue)), 2); .button { z-index: $key-count; margin: $first-value; order: $legacy-key-count; color: $legacy-value; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        let replacements = report
            .resolved_replacements
            .iter()
            .map(|replacement| replacement.text.as_str())
            .collect::<Vec<_>>();
        assert!(replacements.contains(&"2"));
        assert!(replacements.contains(&"1px"));
        assert!(replacements.contains(&"blue"));
        assert!(report.evaluated_css.contains("z-index: 2"));
        assert!(report.evaluated_css.contains("margin: 1px"));
        assert!(report.evaluated_css.contains("order: 2"));
        assert!(report.evaluated_css.contains("color: blue"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_map_merge_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "$gap: map.get(map.merge((default: 1px, dense: 2px), (dense: 3px, compact: 4px)), dense); $count: list.length(map.keys(map-merge((default: 1px), (compact: 4px)))); .button { margin: $gap; z-index: $count; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        let replacements = report
            .resolved_replacements
            .iter()
            .map(|replacement| replacement.text.as_str())
            .collect::<Vec<_>>();
        assert!(replacements.contains(&"3px"));
        assert!(replacements.contains(&"2"));
        assert!(report.evaluated_css.contains("margin: 3px"));
        assert!(report.evaluated_css.contains("z-index: 2"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_nested_static_map_merge_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "$gap: map.get(map.merge((theme: (spacing: (sm: 4px))), theme, spacing, (md: 8px)), theme, spacing, md); $count: list.length(map.keys(map.merge((), theme, colors, (primary: red, secondary: blue)))); .button { margin: $gap; z-index: $count; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        let replacements = report
            .resolved_replacements
            .iter()
            .map(|replacement| replacement.text.as_str())
            .collect::<Vec<_>>();
        assert!(replacements.contains(&"8px"));
        assert!(replacements.contains(&"1"));
        assert!(report.evaluated_css.contains("margin: 8px"));
        assert!(report.evaluated_css.contains("z-index: 1"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_map_deep_merge_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "$merged: map.deep-merge((theme: (spacing: (sm: 4px), tone: blue)), (theme: (spacing: (md: 8px), tone: red))); $gap: map.get($merged, theme, spacing, md); $old: map.get($merged, theme, spacing, sm); $tone: map.get($merged, theme, tone); $count: list.length(map.keys(map.get($merged, theme, spacing))); .button { margin: $gap; padding: $old; color: $tone; z-index: $count; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        let replacements = report
            .resolved_replacements
            .iter()
            .map(|replacement| replacement.text.as_str())
            .collect::<Vec<_>>();
        assert!(replacements.contains(&"8px"));
        assert!(replacements.contains(&"4px"));
        assert!(replacements.contains(&"red"));
        assert!(replacements.contains(&"2"));
        assert!(report.evaluated_css.contains("margin: 8px"));
        assert!(report.evaluated_css.contains("padding: 4px"));
        assert!(report.evaluated_css.contains("color: red"));
        assert!(report.evaluated_css.contains("z-index: 2"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_map_remove_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "$gap: map.get(map.remove((default: 1px, dense: 2px, compact: 4px), dense, missing), compact); $count: list.length(map.keys(map-remove((default: 1px, dense: 2px), default, dense))); .button { margin: $gap; z-index: $count; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        let replacements = report
            .resolved_replacements
            .iter()
            .map(|replacement| replacement.text.as_str())
            .collect::<Vec<_>>();
        assert!(replacements.contains(&"4px"));
        assert!(replacements.contains(&"0"));
        assert!(report.evaluated_css.contains("margin: 4px"));
        assert!(report.evaluated_css.contains("z-index: 0"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_nested_static_map_deep_remove_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "$gap: map.get(map.deep-remove((theme: (spacing: (sm: 4px, md: 8px))), theme, spacing, sm), theme, spacing, md); $count: list.length(map.keys(map.deep-remove((theme: (colors: (primary: red, secondary: blue))), theme, colors, primary))); $tone: map.get(map.deep-remove((theme: blue), theme, colors, primary), theme); .button { margin: $gap; z-index: $count; color: $tone; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        let replacements = report
            .resolved_replacements
            .iter()
            .map(|replacement| replacement.text.as_str())
            .collect::<Vec<_>>();
        assert!(replacements.contains(&"8px"));
        assert!(replacements.contains(&"1"));
        assert!(replacements.contains(&"blue"));
        assert!(report.evaluated_css.contains("margin: 8px"));
        assert!(report.evaluated_css.contains("z-index: 1"));
        assert!(report.evaluated_css.contains("color: blue"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_map_set_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "$weight: map.get(map.set((regular: 400, medium: 500), regular, 300), regular); $count: list.length(map.keys(map.set((), compact, 4px))); .button { font-weight: $weight; z-index: $count; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        let replacements = report
            .resolved_replacements
            .iter()
            .map(|replacement| replacement.text.as_str())
            .collect::<Vec<_>>();
        assert!(replacements.contains(&"300"));
        assert!(replacements.contains(&"1"));
        assert!(report.evaluated_css.contains("font-weight: 300"));
        assert!(report.evaluated_css.contains("z-index: 1"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_nested_static_map_set_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "$tone: map.get(map.set((theme: blue), theme, colors, primary, red), theme, colors, primary); $gap: map.get(map.set((theme: (spacing: (sm: 4px))), theme, spacing, md, 8px), theme, spacing, md); .button { color: $tone; margin: $gap; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        let replacements = report
            .resolved_replacements
            .iter()
            .map(|replacement| replacement.text.as_str())
            .collect::<Vec<_>>();
        assert!(replacements.contains(&"red"));
        assert!(replacements.contains(&"8px"));
        assert!(report.evaluated_css.contains("color: red"));
        assert!(report.evaluated_css.contains("margin: 8px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_math_numeric_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "$gap: math.div(6px, 3); $ratio: percentage(.25); $math-ratio: math.percentage(.5); $pad: if(math.is-unitless(2), 1px, 2px); $border: if(unitless(2px), 3px, 4px); $unit: math.unit(2px); $unitless-name: unit(2); $compatible: if(math.compatible(1px, 2px), 5px, 6px); $global-compatible: if(comparable(1, 1px), 7px, 8px); .button { margin: $gap; width: $ratio; max-width: $math-ratio; padding: $pad; border-width: $border; content: $unit; quotes: $unitless-name; outline-width: $compatible; min-width: $global-compatible; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        let replacements = report
            .resolved_replacements
            .iter()
            .map(|replacement| replacement.text.as_str())
            .collect::<Vec<_>>();
        assert!(replacements.contains(&"2px"));
        assert!(replacements.contains(&"25%"));
        assert!(replacements.contains(&"50%"));
        assert!(replacements.contains(&"1px"));
        assert!(replacements.contains(&"4px"));
        assert!(replacements.contains(&"\"px\""));
        assert!(replacements.contains(&"\"\""));
        assert!(replacements.contains(&"5px"));
        assert!(replacements.contains(&"8px"));
        assert!(report.evaluated_css.contains("margin: 2px"));
        assert!(report.evaluated_css.contains("width: 25%"));
        assert!(report.evaluated_css.contains("max-width: 50%"));
        assert!(report.evaluated_css.contains("padding: 1px"));
        assert!(report.evaluated_css.contains("border-width: 4px"));
        assert!(report.evaluated_css.contains("content: \"px\""));
        assert!(report.evaluated_css.contains("quotes: \"\""));
        assert!(report.evaluated_css.contains("outline-width: 5px"));
        assert!(report.evaluated_css.contains("min-width: 8px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_namespaced_math_aliases() {
        let report = derive_static_stylesheet_module_evaluation(
            "$gap: math.max(1px, 3px); $pad: math.min(4px, 2px); $offset: math.abs(-2px); $width: math.clamp(1px, 5px, 3px); .button { margin: $gap; padding: $pad; inset: $offset; width: $width; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        let replacements = report
            .resolved_replacements
            .iter()
            .map(|replacement| replacement.text.as_str())
            .collect::<Vec<_>>();
        assert!(replacements.contains(&"3px"));
        assert!(replacements.contains(&"2px"));
        assert!(report.evaluated_css.contains("margin: 3px"));
        assert!(report.evaluated_css.contains("padding: 2px"));
        assert!(report.evaluated_css.contains("inset: 2px"));
        assert!(report.evaluated_css.contains("width: 3px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_legacy_rounding_aliases() {
        let report = derive_static_stylesheet_module_evaluation(
            "$ceil: ceil(1.2px); $floor: floor(1.8px); $round: round(1.5px); .button { top: $ceil; bottom: $floor; left: $round; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        let replacements = report
            .resolved_replacements
            .iter()
            .map(|replacement| replacement.text.as_str())
            .collect::<Vec<_>>();
        assert!(replacements.contains(&"2px"));
        assert!(replacements.contains(&"1px"));
        assert!(report.evaluated_css.contains("top: 2px"));
        assert!(report.evaluated_css.contains("bottom: 1px"));
        assert!(report.evaluated_css.contains("left: 2px"));
        assert_eq!(report.value_resolution.reference_count, 3);
        assert_eq!(report.value_resolution.resolved_count, 3);
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_extended_namespaced_math_aliases() {
        let report = derive_static_stylesheet_module_evaluation(
            "$sign: math.sign(-2px); $ceil: math.ceil(1.2px); $floor: math.floor(1.8px); $round: math.round(1.5px); $mod: math.mod(7px, 3px); $rem: math.rem(8px, 3px); $hypot: math.hypot(3px, 4px); $sqrt: math.sqrt(9); $pow: math.pow(2, 3); $exp: math.exp(0); $log: math.log(8, 2); .button { z-index: $sign; margin: $mod; padding: $rem; width: $hypot; opacity: $sqrt; order: $pow; flex-grow: $exp; flex-shrink: $log; top: $ceil; bottom: $floor; left: $round; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        let replacements = report
            .resolved_replacements
            .iter()
            .map(|replacement| replacement.text.as_str())
            .collect::<Vec<_>>();
        assert!(replacements.contains(&"-1"));
        assert!(replacements.contains(&"1px"));
        assert!(replacements.contains(&"2px"));
        assert!(replacements.contains(&"5px"));
        assert!(replacements.contains(&"3"));
        assert!(replacements.contains(&"8"));
        assert!(report.evaluated_css.contains("z-index: -1"));
        assert!(report.evaluated_css.contains("margin: 1px"));
        assert!(report.evaluated_css.contains("padding: 2px"));
        assert!(report.evaluated_css.contains("width: 5px"));
        assert!(report.evaluated_css.contains("opacity: 3"));
        assert!(report.evaluated_css.contains("order: 8"));
        assert!(report.evaluated_css.contains("flex-grow: 1"));
        assert!(report.evaluated_css.contains("flex-shrink: 3"));
        assert!(report.evaluated_css.contains("top: 2px"));
        assert!(report.evaluated_css.contains("bottom: 1px"));
        assert!(report.evaluated_css.contains("left: 2px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_namespaced_math_trig_aliases() {
        let report = derive_static_stylesheet_module_evaluation(
            "$sin: math.sin(30deg); $cos: math.cos(60deg); $tan: math.tan(45deg); $asin: math.asin(.5); $acos: math.acos(.5); $atan: math.atan(1); $atan2: math.atan2(1px, 1px); .button { opacity: $sin; flex-grow: $cos; flex-shrink: $tan; rotate: $asin; offset-rotate: $acos; --atan: $atan; --atan2: $atan2; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        let replacements = report
            .resolved_replacements
            .iter()
            .map(|replacement| replacement.text.as_str())
            .collect::<Vec<_>>();
        assert!(replacements.contains(&"0.5"));
        assert!(replacements.contains(&"1"));
        assert!(replacements.contains(&"30deg"));
        assert!(replacements.contains(&"60deg"));
        assert!(replacements.contains(&"45deg"));
        assert!(report.evaluated_css.contains("opacity: 0.5"));
        assert!(report.evaluated_css.contains("flex-grow: 0.5"));
        assert!(report.evaluated_css.contains("flex-shrink: 1"));
        assert!(report.evaluated_css.contains("rotate: 30deg"));
        assert!(report.evaluated_css.contains("offset-rotate: 60deg"));
        assert!(report.evaluated_css.contains("--atan: 45deg"));
        assert!(report.evaluated_css.contains("--atan2: 45deg"));
        assert_eq!(report.value_resolution.reference_count, 7);
        assert_eq!(report.value_resolution.resolved_count, 7);
        assert_eq!(report.value_resolution.raw_count, 0);
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_namespaced_math_constants() {
        let report = derive_static_stylesheet_module_evaluation(
            "$pi: math.$pi; $e: math.$e; $epsilon: math.$epsilon; $max-safe: math.$max-safe-integer; $min-safe: math.$min-safe-integer; .button { --pi: $pi; --e: $e; --epsilon: $epsilon; z-index: $max-safe; order: $min-safe; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        let replacements = report
            .resolved_replacements
            .iter()
            .map(|replacement| replacement.text.as_str())
            .collect::<Vec<_>>();
        assert!(replacements.contains(&"3.1415926536"));
        assert!(replacements.contains(&"2.7182818285"));
        assert!(replacements.contains(&"0"));
        assert!(replacements.contains(&"9007199254740991"));
        assert!(replacements.contains(&"-9007199254740991"));
        assert!(report.evaluated_css.contains("--pi: 3.1415926536"));
        assert!(report.evaluated_css.contains("--e: 2.7182818285"));
        assert!(report.evaluated_css.contains("--epsilon: 0"));
        assert!(report.evaluated_css.contains("z-index: 9007199254740991"));
        assert!(report.evaluated_css.contains("order: -9007199254740991"));
        assert_eq!(report.value_resolution.reference_count, 5);
        assert_eq!(report.value_resolution.resolved_count, 5);
        assert_eq!(report.value_resolution.raw_count, 0);
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_does_not_treat_math_constants_as_variable_dependencies() {
        let report = summarize_static_stylesheet_value_resolution(
            "$pi: math.$pi; .button { --pi: $pi; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.reference_count, 1);
        assert_eq!(report.resolved_count, 1);
        assert_eq!(report.values[0].source_text, "$pi");
        assert_eq!(report.values[0].rendered_value.as_deref(), Some("3.141593"));
    }

    #[test]
    fn static_scss_evaluation_reduces_math_constant_function_arguments() {
        let report = derive_static_stylesheet_module_evaluation(
            "$unitless: if(math.is-unitless(math.$pi), 1px, 2px); $unit-ok: if(math.unit(math.$pi) == \"\", 5px, 6px); $compatible: if(math.compatible(math.$pi, 1), 3px, 4px); $sin: math.sin(math.$pi); .button { padding: $unitless; border-width: $unit-ok; margin: $compatible; opacity: $sin; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains("padding: 1px"));
        assert!(report.evaluated_css.contains("border-width: 5px"));
        assert!(report.evaluated_css.contains("margin: 3px"));
        assert!(report.evaluated_css.contains("opacity: 0"));
        assert_eq!(report.value_resolution.reference_count, 4);
        assert_eq!(report.value_resolution.resolved_count, 4);
        assert_eq!(report.value_resolution.raw_count, 0);
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_keeps_unsupported_namespaced_math_trig_raw() {
        let report = derive_static_stylesheet_module_evaluation(
            "$bad-angle: math.sin(1px); $bad-inverse: math.asin(2); .button { width: $bad-angle; height: $bad-inverse; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        assert_eq!(report.replacement_count, 2);
        let replacements = report
            .resolved_replacements
            .iter()
            .map(|replacement| replacement.text.as_str())
            .collect::<Vec<_>>();
        assert!(replacements.contains(&"math.sin(1px)"));
        assert!(replacements.contains(&"math.asin(2)"));
        assert_eq!(report.value_resolution.raw_count, 2);
        assert!(report.evaluated_css.contains("width: math.sin(1px)"));
        assert!(report.evaluated_css.contains("height: math.asin(2)"));
    }

    #[test]
    fn static_scss_evaluation_keeps_unsupported_namespaced_math_raw() {
        let report = derive_static_stylesheet_module_evaluation(
            "$gap: math.random(); .button { margin: $gap; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].text, "math.random()");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "raw");
        assert!(report.evaluated_css.contains("margin: math.random()"));

        let resolution = summarize_static_stylesheet_value_resolution(
            "$gap: math.random(); .button { margin: $gap; }",
            StyleDialect::Scss,
        );
        assert!(resolution.is_some());
        let Some(resolution) = resolution else {
            return;
        };

        assert_eq!(resolution.reference_count, 1);
        assert_eq!(resolution.raw_count, 1);
        assert_eq!(resolution.values[0].source_text, "$gap");
        assert_eq!(
            resolution.values[0].rendered_value.as_deref(),
            Some("math.random()")
        );
        assert_eq!(resolution.values[0].outcome, "raw");
        assert_eq!(resolution.values[0].reason, "unsupportedDynamic");
    }

    #[test]
    fn static_scss_evaluation_reduces_nested_static_list_conditions_in_order() {
        let report = derive_static_stylesheet_module_evaluation(
            "$count: list.length(if(false, 1px 2px, 3px 4px 5px)); .button { z-index: $count; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(
            report
                .resolved_replacements
                .iter()
                .any(|replacement| { replacement.name == "$count" && replacement.text == "3" })
        );
        assert!(report.evaluated_css.contains("z-index: 3"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_if_not_conditions() {
        let report = derive_static_stylesheet_module_evaluation(
            "$gap: if(not true, 1px, 2px); .button { margin: $gap; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.resolved_replacements[0].text, "2px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { margin: 2px; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_if_boolean_conditions() {
        let and_report = derive_static_stylesheet_module_evaluation(
            "$gap: if(false and true, 1px, 2px); .button { margin: $gap; }",
            StyleDialect::Scss,
        );
        assert!(and_report.is_some());
        let Some(and_report) = and_report else {
            return;
        };
        assert_eq!(and_report.resolved_replacements[0].text, "2px");
        assert!(
            and_report
                .evaluated_css
                .contains(".button { margin: 2px; }")
        );
        assert!(and_report.oracle.all_legacy_declaration_values_preserved);

        let or_report = derive_static_stylesheet_module_evaluation(
            "$gap: if(false or true, 1px, 2px); .button { margin: $gap; }",
            StyleDialect::Scss,
        );
        assert!(or_report.is_some());
        let Some(or_report) = or_report else {
            return;
        };
        assert_eq!(or_report.resolved_replacements[0].text, "1px");
        assert!(or_report.evaluated_css.contains(".button { margin: 1px; }"));
        assert!(or_report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_if_equality_conditions() {
        let report = derive_static_stylesheet_module_evaluation(
            "$gap: if(1px == 2px, 1px, 2px); .button { margin: $gap; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.resolved_replacements[0].text, "2px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { margin: 2px; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_if_inequality_conditions() {
        let report = derive_static_stylesheet_module_evaluation(
            "$gap: if(1px != 2px, 1px, 2px); .button { margin: $gap; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.resolved_replacements[0].text, "1px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { margin: 1px; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_if_numeric_ordering_conditions() {
        let report = derive_static_stylesheet_module_evaluation(
            "$gap: if(3px > 2px, 1px, 2px); .button { margin: $gap; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.resolved_replacements[0].text, "1px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { margin: 1px; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_if_zero_numeric_ordering_conditions() {
        let report = derive_static_stylesheet_module_evaluation(
            "$gap: if(0px >= 0, 1px, 2px); .button { margin: $gap; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.resolved_replacements[0].text, "1px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { margin: 1px; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_stylesheet_bang_safety_only_allows_comparisons() {
        assert!(static_scss_bang_usage_is_comparison_only(
            "if(1px != 2px, 1px, 2px)"
        ));
        assert!(!static_scss_bang_usage_is_comparison_only("1px !important"));
    }

    #[test]
    fn static_scss_evaluation_reduces_parenthesized_if_conditions() {
        let report = derive_static_stylesheet_module_evaluation(
            "$gap: if((false or true), 1px, 2px); .button { margin: $gap; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.resolved_replacements[0].text, "1px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { margin: 1px; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_reduces_numeric_builtin_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "@gap: max(1px, 2px); .button { margin: @gap; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.resolved_replacements[0].text, "2px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert_eq!(
            report.value_resolution.values[0].rendered_value.as_deref(),
            Some("2px")
        );
        assert!(report.evaluated_css.contains("margin: 2px"));
    }

    #[test]
    fn static_less_evaluation_reduces_property_variable_numeric_values() {
        let report = derive_static_stylesheet_module_evaluation(
            ".button { margin: (1px + 2px); padding: $margin; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "$margin");
        assert_eq!(report.resolved_replacements[0].text, "3px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert_eq!(
            report.value_resolution.values[0].rendered_value.as_deref(),
            Some("3px")
        );
        assert!(report.evaluated_css.contains("padding: 3px"));
    }

    #[test]
    fn static_less_evaluation_reduces_property_variable_escaped_string_values() {
        let report = derive_static_stylesheet_module_evaluation(
            ".button { filter: ~\"alpha(opacity=50)\"; background: $filter; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "$filter");
        assert_eq!(report.resolved_replacements[0].text, "alpha(opacity=50)");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "raw");
        assert_eq!(
            report.value_resolution.values[0].rendered_value.as_deref(),
            Some("alpha(opacity=50)")
        );
        assert!(
            report
                .evaluated_css
                .contains("background: alpha(opacity=50)")
        );
        assert!(!report.evaluated_css.contains("~\"alpha"));
    }

    #[test]
    fn static_value_resolution_keeps_irreducible_numeric_functions_raw() {
        let report = summarize_static_stylesheet_value_resolution(
            "$gap: min(1px, 2rem); .button { margin: $gap; }",
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
            Some("min(1px, 2rem)")
        );
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
    fn static_scss_evaluation_reduces_function_numeric_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function double($value) { @return ($value + $value); } .button { margin: double(2px); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:double");
        assert_eq!(report.resolved_replacements[0].text, "4px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { margin: 4px; }"));
        assert_eq!(
            report.value_resolution.values[0].rendered_value.as_deref(),
            Some("4px")
        );
    }

    #[test]
    fn static_scss_evaluation_resolves_named_function_arguments() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function pair($left, $right) { @return $left + $right; } .button { margin: pair($right: 2px, $left: 1px); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:pair");
        assert_eq!(report.resolved_replacements[0].text, "3px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { margin: 3px; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_rejects_positional_arguments_after_named_arguments() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function pair($left, $right) { @return $left + $right; } .button { margin: pair($left: 1px, 2px); }",
            StyleDialect::Scss,
        );
        assert!(report.is_none());

        let resolution = summarize_static_stylesheet_value_resolution(
            "@function pair($left, $right) { @return $left + $right; } .button { margin: pair($left: 1px, 2px); }",
            StyleDialect::Scss,
        );
        assert!(resolution.is_some());
        let Some(resolution) = resolution else {
            return;
        };

        assert_eq!(resolution.raw_count, 1);
        assert_eq!(resolution.unsupported_dynamic_count, 1);
    }

    #[test]
    fn static_scss_evaluation_resolves_function_default_arguments() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function offset($value: 1px, $extra: 2px) { @return $value + $extra; } .button { margin: offset(); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:offset");
        assert_eq!(report.resolved_replacements[0].text, "3px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(!report.evaluated_css.contains("@function"));
        assert!(report.evaluated_css.contains(".button { margin: 3px; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_resolves_named_arguments_with_default_tail() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function pair($left, $right: 2px) { @return $left + $right; } .button { margin: pair($left: 1px); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:pair");
        assert_eq!(report.resolved_replacements[0].text, "3px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { margin: 3px; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_resolves_default_arguments_from_prior_parameters() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function offset($value, $extra: $value + 1px) { @return $extra; } .button { margin: offset(2px); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:offset");
        assert_eq!(report.resolved_replacements[0].text, "3px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { margin: 3px; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_if_function_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function choose($condition) { @return if($condition, 1px, 2px) + 1px; } .button { margin: choose(true); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:choose");
        assert_eq!(report.resolved_replacements[0].text, "2px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { margin: 2px; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_function_if_not_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function choose($condition) { @return if(not $condition, 1px, 2px); } .button { margin: choose(true); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:choose");
        assert_eq!(report.resolved_replacements[0].text, "2px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { margin: 2px; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_function_boolean_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function choose($condition) { @return if($condition and false, 1px, 2px); } .button { margin: choose(true); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:choose");
        assert_eq!(report.resolved_replacements[0].text, "2px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { margin: 2px; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_function_equality_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function choose($value) { @return if($value == 2px, 1px, 2px); } .button { margin: choose(2px); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:choose");
        assert_eq!(report.resolved_replacements[0].text, "1px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { margin: 1px; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_function_inequality_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function choose($value) { @return if($value != 2px, 1px, 2px); } .button { margin: choose(3px); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:choose");
        assert_eq!(report.resolved_replacements[0].text, "1px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { margin: 1px; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_function_numeric_ordering_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function choose($value) { @return if($value <= 2px, 1px, 2px); } .button { margin: choose(2px); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:choose");
        assert_eq!(report.resolved_replacements[0].text, "1px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { margin: 1px; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_ignores_inactive_if_branch_callables() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function choose() { @return if(false, min(1px, 2px), 3px) + 1px; } .button { margin: choose(); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:choose");
        assert_eq!(report.resolved_replacements[0].text, "4px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { margin: 4px; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_function_bare_numeric_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function double($value) { @return $value * 2; } .button { margin: double(2px); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:double");
        assert_eq!(report.resolved_replacements[0].text, "4px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { margin: 4px; }"));
        assert_eq!(
            report.value_resolution.values[0].rendered_value.as_deref(),
            Some("4px")
        );
    }

    #[test]
    fn static_scss_evaluation_resolves_function_local_variables() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function offset($base) { $next: $base + 1px; @return $next + 1px; } .button { margin: offset(2px); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:offset");
        assert_eq!(report.resolved_replacements[0].text, "4px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { margin: 4px; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_resolves_function_local_variable_chains() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function scale($base) { $next: $base + 1px; $double: $next * 2; @return $double; } .button { margin: scale(2px); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:scale");
        assert_eq!(report.resolved_replacements[0].text, "6px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { margin: 6px; }"));
        assert_eq!(
            report.value_resolution.values[0].rendered_value.as_deref(),
            Some("6px")
        );
    }

    #[test]
    fn static_scss_evaluation_resolves_local_variables_after_prior_branch() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function pick($enabled) { @if $enabled { @return 3px; } $after: 1px + 1px; @return $after; } .button { margin: pick(false); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:pick");
        assert_eq!(report.resolved_replacements[0].text, "2px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { margin: 2px; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_resolves_branch_local_variables() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function pick($enabled) { @if $enabled { $inside: 1px + 1px; @return $inside; } @return 1px; } .button { margin: pick(true); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:pick");
        assert_eq!(report.resolved_replacements[0].text, "2px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { margin: 2px; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_does_not_leak_sibling_branch_local_variables() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function pick($enabled) { @if $enabled { @return $other; } @else { $other: 1px; @return $other; } } .button { margin: pick(true); }",
            StyleDialect::Scss,
        );
        assert!(report.is_none());
    }

    #[test]
    fn static_scss_evaluation_skips_future_local_variable_replacements() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function pick($enabled) { @if $enabled { @return $after; } $after: 1px; @return $after; } .button { margin: pick(true); }",
            StyleDialect::Scss,
        );
        assert!(report.is_none());
    }

    #[test]
    fn static_scss_evaluation_ignores_future_unsafe_local_variables() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function pick($enabled) { @if $enabled { @return 2px; } $after: 1px !global; @return $after; } .button { margin: pick(true); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:pick");
        assert_eq!(report.resolved_replacements[0].text, "2px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { margin: 2px; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_resolves_composed_same_file_function_calls() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function inc($value) { @return $value + 1px; } @function gap($value) { @return inc($value) + 1px; } .button { margin: gap(2px); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:gap");
        assert_eq!(report.resolved_replacements[0].text, "4px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(!report.evaluated_css.contains("@function"));
        assert!(report.evaluated_css.contains(".button { margin: 4px; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_resolves_local_values_with_same_file_function_calls() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function inc($value) { @return $value + 1px; } @function gap($value) { $next: inc($value); @return $next + 1px; } .button { margin: gap(2px); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:gap");
        assert_eq!(report.resolved_replacements[0].text, "4px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(!report.evaluated_css.contains("@function"));
        assert!(report.evaluated_css.contains(".button { margin: 4px; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_resolves_static_if_function_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function tone($enabled) { @if $enabled { @return red; } @return blue; } .button { color: tone(true); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:tone");
        assert_eq!(report.resolved_replacements[0].text, "red");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(!report.evaluated_css.contains("@function"));
        assert!(report.evaluated_css.contains(".button { color: red; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_resolves_static_else_function_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function tone($enabled) { @if $enabled { @return red; } @else { @return blue; } } .button { color: tone(false); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:tone");
        assert_eq!(report.resolved_replacements[0].text, "blue");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { color: blue; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_resolves_static_else_if_function_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function tone($first, $second) { @if $first { @return red; } @else if $second { @return green; } @else { @return blue; } } .button { color: tone(false, true); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:tone");
        assert_eq!(report.resolved_replacements[0].text, "green");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { color: green; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_resolves_static_for_loop_function_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function pick($target) { @for $i from 1 through 3 { @if $i == $target { @return $i + 1; } } @return 0; } .button { z-index: pick(2); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:pick");
        assert_eq!(report.resolved_replacements[0].text, "3");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { z-index: 3; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_resolves_nested_static_for_loop_function_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function collect($target) { @for $i from 1 through 2 { @for $j from 1 through 2 { @if $i == $target { @return $i + $j; } } } @return 0; } .button { z-index: collect(2); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:collect");
        assert_eq!(report.resolved_replacements[0].text, "3");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { z-index: 3; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_continues_after_inactive_static_for_loop_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function pick($target) { @for $i from 1 through 3 { @if $i == $target { @return $i + 1px; } } @return 0px; } .button { margin: pick(4); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:pick");
        assert_eq!(report.resolved_replacements[0].text, "0px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { margin: 0px; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_resolves_static_each_single_loop_function_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function first-tone() { @each $tone in red, blue { @return $tone; } } .button { color: first-tone(); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:first-tone");
        assert_eq!(report.resolved_replacements[0].text, "red");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { color: red; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_resolves_static_each_map_loop_function_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function tone($target) { @each $name, $tone in (primary: red, secondary: blue) { @if $name == $target { @return $tone; } } @return black; } .button { color: tone(secondary); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:tone");
        assert_eq!(report.resolved_replacements[0].text, "blue");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { color: blue; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_resolves_static_each_tuple_loop_function_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function icon-size($target) { $pairs: (save, 16px), (cancel, 24px); @each $icon, $size in $pairs { @if $icon == $target { @return $size; } } @return 0px; } .button { width: icon-size(cancel); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:icon-size");
        assert_eq!(report.resolved_replacements[0].text, "24px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { width: 24px; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_resolves_static_while_loop_function_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function pick() { $i: 0; @while $i < 3 { @if $i == 2 { @return $i + 1; } $i: $i + 1; } @return 0; } .button { z-index: pick(); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:pick");
        assert_eq!(report.resolved_replacements[0].text, "3");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { z-index: 3; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_uses_arguments_in_static_while_loop_function_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function pick($target) { $i: 0; @while $i < 3 { @if $i == $target { @return $i + 1; } $i: $i + 1; } @return 0; } .button { z-index: pick(2); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:pick");
        assert_eq!(report.resolved_replacements[0].text, "3");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { z-index: 3; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_continues_after_inactive_static_while_loop_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function pick() { $i: 0; @while $i < 2 { @if $i == 5 { @return $i; } $i: $i + 1; } @return 9; } .button { z-index: pick(); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "function:pick");
        assert_eq!(report.resolved_replacements[0].text, "9");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { z-index: 9; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_keeps_dynamic_if_function_returns_top() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function tone($enabled) { @if $enabled { @return red; } @else { @return blue; } } .button { color: tone(var(--enabled)); }",
            StyleDialect::Scss,
        );
        assert!(report.is_none());

        let resolution = summarize_static_stylesheet_value_resolution(
            "@function tone($enabled) { @if $enabled { @return red; } @else { @return blue; } } .button { color: tone(var(--enabled)); }",
            StyleDialect::Scss,
        );
        assert!(resolution.is_some());
        let Some(resolution) = resolution else {
            return;
        };

        assert_eq!(resolution.reference_count, 1);
        assert_eq!(resolution.top_count, 1);
        assert_eq!(resolution.unsupported_dynamic_count, 1);
        assert_eq!(resolution.values[0].outcome, "top");
        assert_eq!(resolution.values[0].reason, "unsupportedDynamic");
    }

    #[test]
    fn static_scss_evaluation_reports_indirect_recursive_function_calls_as_top() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function a($value) { @return b($value); } @function b($value) { @return a($value); } .button { color: a(red); }",
            StyleDialect::Scss,
        );
        assert!(report.is_none());

        let resolution = summarize_static_stylesheet_value_resolution(
            "@function a($value) { @return b($value); } @function b($value) { @return a($value); } .button { color: a(red); }",
            StyleDialect::Scss,
        );
        assert!(resolution.is_some());
        let Some(resolution) = resolution else {
            return;
        };

        assert_eq!(resolution.reference_count, 1);
        assert_eq!(resolution.top_count, 1);
        assert_eq!(resolution.cycle_count, 1);
        assert_eq!(resolution.values[0].outcome, "top");
        assert_eq!(resolution.values[0].reason, "cycle");
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

        assert_eq!(resolution.reference_count, 1);
        assert_eq!(resolution.top_count, 1);
        assert_eq!(resolution.cycle_count, 1);
        assert!(
            resolution
                .values
                .iter()
                .all(|value| value.outcome == "top" && value.reason == "cycle")
        );
    }

    #[test]
    fn static_scss_evaluation_reduces_static_list_constructor_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "$items: list.append(1px 2px, 3px); $item-count: list.length($items); $third-item: list.nth($items, 3); $joined: list.join((red, blue), (green, yellow), $separator: comma); $joined-third: list.nth($joined, 3); $set: list.set-nth(4px 5px 6px, -1, 8px); $set-tail: list.nth($set, -1); $zipped: list.zip(1px 2px, solid dashed); $second-pair: list.nth($zipped, 2); .button { z-index: $item-count; margin: $third-item; color: $joined-third; padding: $set-tail; border: $second-pair; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains("z-index: 3"));
        assert!(report.evaluated_css.contains("margin: 3px"));
        assert!(report.evaluated_css.contains("color: green"));
        assert!(report.evaluated_css.contains("padding: 8px"));
        assert!(report.evaluated_css.contains("border: 2px dashed"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_slash_list_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "$stroke: list.slash(1px, solid, red); $separator: list.separator($stroke); $middle: list.nth($stroke, 2); .button { font: $stroke; content: $separator; outline-style: $middle; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains("font: 1px / solid / red"));
        assert!(report.evaluated_css.contains("content: \"slash\""));
        assert!(report.evaluated_css.contains("outline-style: solid"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_function_comparison_operands() {
        let report = derive_static_stylesheet_module_evaluation(
            "$stroke: list.slash(1px, solid, red); $kind: if(meta.type-of($stroke) == list and list.separator($stroke) == \"slash\" and hue(#808000) == 60deg, 1px, 2px); .button { margin: $kind; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains("margin: 1px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_type_metadata_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "$gap: 2px; $tone: red; $transparent-tone: rgba($tone, .5); $mixed-tone: color.mix(red, blue); $red-channel: color.channel($mixed-tone, \"red\", $space: rgb); $legacy-red-channel: red($tone); $relative-tone: oklab(1 0 0); $items: 1px 2px; $config: (dense: true); $kind: if(meta.type-of($gap) == number and type-of($tone) == color and meta.type-of($transparent-tone) == color and meta.type-of($mixed-tone) == color and meta.type-of($red-channel) == number and meta.type-of($legacy-red-channel) == number and meta.type-of($relative-tone) == color and meta.type-of($items) == list and type-of($config) == map and feature-exists(\"at-error\") and meta.feature-exists(custom-property) and not meta.feature-exists(\"unknown\"), 1px, 2px); .button { margin: $kind; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains("margin: 1px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_inspect_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "$tone: meta.inspect(red); $gap: inspect(2px); .button { color: $tone; margin: $gap; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains("color: red"));
        assert!(report.evaluated_css.contains("margin: 2px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_calculation_metadata_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "$name: meta.calc-name(clamp(1px, 2px, 3px)); $args: meta.calc-args(clamp(1px, 2px, 3px)); $kind: meta.type-of(calc(100% - 1px)); $gap: if($name == \"clamp\" and $kind == calculation and list.length($args) == 3 and list.nth($args, 2) == 2px, 1px, 2px); .button { margin: $gap; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains("margin: 1px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_function_metadata_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function present() { @return 1px; } @function gate() { @return if(meta.function-exists(\"present\") and function-exists(\"scale-color\") and function-exists(\"hue\") and not function-exists(\"not-defined-here\"), present(), 2px); } .button { margin: gate(); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains("margin: 1px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_preserves_function_exists_declaration_order() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function gate() { @return if(function-exists(\"later\"), 2px, 1px); } @function later() { @return 2px; } .button { margin: gate(); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains("margin: 1px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_variable_metadata_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "$global-gap: 1px; $kind: if(variable-exists(\"global-gap\") and meta.global-variable-exists(\"global-gap\") and not global-variable-exists(\"missing\"), 1px, 2px); @function gate($local-gap) { $inner-gap: 2px; @return if(meta.variable-exists(\"local-gap\") and variable-exists(\"inner-gap\") and global-variable-exists(\"global-gap\") and not global-variable-exists(\"inner-gap\"), $global-gap, 4px); } .button { margin: $kind; padding: gate(3px); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains("margin: 1px"));
        assert!(report.evaluated_css.contains("padding: 1px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_mixin_metadata_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "@mixin present { color: red; } @function gate() { @return if(meta.mixin-exists(\"present\") and not mixin-exists(\"not-defined-here\"), 1px, 2px); } .button { margin: gate(); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains("margin: 1px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_preserves_mixin_exists_declaration_order() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function gate() { @return if(mixin-exists(\"later\"), 2px, 1px); } @mixin later { color: red; } .button { margin: gate(); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains("margin: 1px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_expands_static_mixin_includes() {
        let report = derive_static_stylesheet_module_evaluation(
            "$brand: red; @mixin tone($color, $gap: 1px) { color: $color; margin: $gap; padding: $brand; } .button { @include tone(blue, 2px); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains("@mixin"));
        assert!(!report.evaluated_css.contains("@include"));
        assert!(report.evaluated_css.contains("color: blue"));
        assert!(report.evaluated_css.contains("margin: 2px"));
        assert!(report.evaluated_css.contains("padding: red"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_expands_mixin_includes_with_static_function_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function double($value) { @return $value * 2; } @mixin tone($gap) { margin: double($gap); color: red; } .button { @include tone(2px); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains("@function"));
        assert!(!report.evaluated_css.contains("@mixin"));
        assert!(!report.evaluated_css.contains("@include"));
        assert!(report.evaluated_css.contains("margin: 4px"));
        assert!(report.evaluated_css.contains("color: red"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_expands_nested_static_mixin_includes() {
        let report = derive_static_stylesheet_module_evaluation(
            "@mixin spacing($gap) { margin: $gap; } @mixin tone($gap, $color: red) { @include spacing($gap); color: $color; } .button { @include tone(2px, blue); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains("@mixin"));
        assert!(!report.evaluated_css.contains("@include"));
        assert!(report.evaluated_css.contains("margin: 2px"));
        assert!(report.evaluated_css.contains("color: blue"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_expands_mixin_local_variables() {
        let report = derive_static_stylesheet_module_evaluation(
            "@mixin tone($gap) { $space: $gap * 2; $color: if($space == 4px, blue, red); margin: $space; color: $color; } .button { @include tone(2px); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains("$space"));
        assert!(!report.evaluated_css.contains("$color"));
        assert!(!report.evaluated_css.contains("@mixin"));
        assert!(!report.evaluated_css.contains("@include"));
        assert!(report.evaluated_css.contains("margin: 4px"));
        assert!(report.evaluated_css.contains("color: blue"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_skips_dynamic_mixin_local_variables() {
        let report = derive_static_stylesheet_module_evaluation(
            "@mixin tone { $space: meta.inspect((a: b)); margin: $space; } .button { @include tone; }",
            StyleDialect::Scss,
        );

        assert!(report.is_none());
    }

    #[test]
    fn static_scss_evaluation_skips_recursive_nested_mixin_includes() {
        let report = derive_static_stylesheet_module_evaluation(
            "@mixin a { @include b; } @mixin b { @include a; } .button { @include a; }",
            StyleDialect::Scss,
        );

        assert!(report.is_none());
    }

    #[test]
    fn static_scss_evaluation_expands_hyphen_underscore_mixin_includes() {
        let report = derive_static_stylesheet_module_evaluation(
            "@mixin tone_color($color) { color: $color; } .button { @include tone-color(green); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains("@mixin"));
        assert!(!report.evaluated_css.contains("@include"));
        assert!(report.evaluated_css.contains("color: green"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_function_list_constructor_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function tail($list) { @return list.nth(list.append($list, 3px), 3); } .button { margin: tail(1px 2px); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.resolved_replacements[0].name, "function:tail");
        assert_eq!(report.resolved_replacements[0].text, "3px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { margin: 3px; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_sass_color_mix_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function tone() { @return color.mix(red, blue); } .button { color: tone(); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.resolved_replacements[0].name, "function:tone");
        assert_eq!(report.resolved_replacements[0].text, "rgb(127.5, 0, 127.5)");
        assert_eq!(
            report.resolved_replacements[0].rendered_value.as_deref(),
            Some("purple")
        );
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(
            report
                .evaluated_css
                .contains(".button { color: rgb(127.5, 0, 127.5); }")
        );
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_color_channel_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function tone-channel() { @return color.channel(color.mix(red, blue), \"red\", rgb); } .button { z-index: tone-channel(); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(
            report.resolved_replacements[0].name,
            "function:tone-channel"
        );
        assert_eq!(report.resolved_replacements[0].text, "127.5");
        assert_eq!(
            report.resolved_replacements[0].rendered_value.as_deref(),
            Some("127.5")
        );
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { z-index: 127.5; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_hsl_color_channel_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function tone-channel() { @return hue(#808000); } .button { --hue: tone-channel(); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(
            report.resolved_replacements[0].name,
            "function:tone-channel"
        );
        assert_eq!(report.resolved_replacements[0].text, "60deg");
        assert_eq!(
            report.resolved_replacements[0].rendered_value.as_deref(),
            Some("60deg")
        );
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { --hue: 60deg; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_static_hsl_color_transform_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function tone() { @return adjust-hue(red, 120deg); } .button { color: tone(); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.resolved_replacements[0].name, "function:tone");
        assert_eq!(report.resolved_replacements[0].text, "#0f0");
        assert_eq!(
            report.resolved_replacements[0].rendered_value.as_deref(),
            Some("#0f0")
        );
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { color: #0f0; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_legacy_global_color_function_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function tone-channel() { @return red(mix(red, blue)); } .button { z-index: tone-channel(); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(
            report.resolved_replacements[0].name,
            "function:tone-channel"
        );
        assert_eq!(report.resolved_replacements[0].text, "127.5");
        assert_eq!(
            report.resolved_replacements[0].rendered_value.as_deref(),
            Some("127.5")
        );
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains(".button { z-index: 127.5; }"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_sass_rgb_color_constructor_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function tone() { @return rgba(red, .5); } .button { color: tone(); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.resolved_replacements[0].name, "function:tone");
        assert_eq!(report.resolved_replacements[0].text, "rgba(255, 0, 0, 0.5)");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(
            report
                .evaluated_css
                .contains(".button { color: rgba(255, 0, 0, 0.5); }")
        );
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_sass_hsl_color_constructor_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function tone() { @return hsl(180, 100%, 50%); } @function overlay() { @return hsla(120, 100%, 50%, .5); } .button { color: tone(); background: overlay(); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.resolved_replacements[0].name, "function:tone");
        assert_eq!(report.resolved_replacements[0].text, "#0ff");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert_eq!(report.resolved_replacements[1].name, "function:overlay");
        assert_eq!(report.resolved_replacements[1].text, "rgba(0, 255, 0, 0.5)");
        assert_eq!(report.resolved_replacements[1].abstract_value_kind, "exact");
        assert!(
            report
                .evaluated_css
                .contains(".button { color: #0ff; background: rgba(0, 255, 0, 0.5); }")
        );
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reduces_sass_opacity_color_returns() {
        let report = derive_static_stylesheet_module_evaluation(
            "@function tone() { @return transparentize(red, .25); } .button { color: tone(); }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.resolved_replacements[0].name, "function:tone");
        assert_eq!(
            report.resolved_replacements[0].text,
            "rgba(255, 0, 0, 0.75)"
        );
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(
            report
                .evaluated_css
                .contains(".button { color: rgba(255, 0, 0, 0.75); }")
        );
        assert!(report.oracle.all_legacy_declaration_values_preserved);
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
            "$tone: color.mix(rgba(red, .5), blue); .button { color: $tone; }",
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
            Some("color.mix(rgba(red, .5), blue)")
        );
    }

    #[test]
    fn static_value_resolution_keeps_nested_opacity_helpers_raw() {
        let report = summarize_static_stylesheet_value_resolution(
            "$tone: color.mix(transparentize(red, .25), blue); .button { color: $tone; }",
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
            Some("color.mix(transparentize(red, .25), blue)")
        );
    }

    #[test]
    fn static_value_resolution_keeps_percent_opacity_amounts_raw() {
        let report = summarize_static_stylesheet_value_resolution(
            "$tone: transparentize(red, 25%); .button { color: $tone; }",
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
            Some("transparentize(red, 25%)")
        );
    }

    #[test]
    fn static_value_resolution_emits_exact_static_sass_color_mix_values() {
        let report = summarize_static_stylesheet_value_resolution(
            "$tone: color.mix(red, blue); $weighted: color.mix(rgb(255 0 0), blue, $weight: 25%); .button { color: $tone; border-color: $weighted; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.reference_count, 2);
        assert_eq!(report.resolved_count, 2);
        assert_eq!(report.raw_count, 0);
        assert!(
            report
                .values
                .iter()
                .all(|value| value.abstract_value_kind == "exact")
        );
        let rendered_values = report
            .values
            .iter()
            .filter_map(|value| value.rendered_value.as_deref())
            .collect::<Vec<_>>();
        assert!(rendered_values.contains(&"purple"));
        assert!(rendered_values.contains(&"#4000bf"));
    }

    #[test]
    fn static_value_resolution_emits_exact_static_color_channel_values() {
        let report = summarize_static_stylesheet_value_resolution(
            "$red: color.channel(color.mix(red, blue), \"red\", $space: rgb); $alpha: color.alpha(rgba(255, 0, 0, .5)); $opacity: color.opacity(rgba(red, .5)); $hue: color.channel(#808000, \"hue\", $space: hsl); $saturation: saturation(#808000); $lightness: color.lightness(#808000); .button { z-index: $red; opacity: $alpha; flex-grow: $opacity; width: $hue; height: $saturation; margin: $lightness; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.reference_count, 6);
        assert_eq!(report.resolved_count, 6);
        assert_eq!(report.raw_count, 0);
        let rendered_values = report
            .values
            .iter()
            .filter_map(|value| value.rendered_value.as_deref())
            .collect::<Vec<_>>();
        assert!(rendered_values.contains(&"127.5"));
        assert!(rendered_values.contains(&"0.5"));
        assert!(rendered_values.contains(&"60deg"));
        assert!(rendered_values.contains(&"100%"));
        assert!(rendered_values.contains(&"25.098039%"));
    }

    #[test]
    fn static_value_resolution_emits_exact_static_hsl_color_transform_values() {
        let report = summarize_static_stylesheet_value_resolution(
            "$adjusted: adjust-hue($color: red, $degrees: 120deg); $complement: color.complement(red); $light: lighten(#808000, 10%); $dark: darken(#808000, 10%); $sat: saturate(#808000, 10%); $desat: desaturate(#808000, 10%); $gray: grayscale(red); $invert: color.invert(red, $weight: 25%); $scaled: color.scale(#808000, $lightness: 50%); $changed: color.change(#808000, $lightness: 50%); .button { color: $adjusted; background: $complement; border-color: $light; outline-color: $dark; caret-color: $sat; text-decoration-color: $desat; column-rule-color: $gray; accent-color: $invert; fill: $scaled; stroke: $changed; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.reference_count, 10);
        assert_eq!(report.resolved_count, 10);
        assert_eq!(report.raw_count, 0);
        assert!(
            report
                .values
                .iter()
                .all(|value| value.abstract_value_kind == "exact")
        );
        let rendered_values = report
            .values
            .iter()
            .filter_map(|value| value.rendered_value.as_deref())
            .collect::<Vec<_>>();
        assert_eq!(
            rendered_values
                .iter()
                .filter(|value| **value == "#0ff")
                .count(),
            1
        );
        assert!(rendered_values.contains(&"#0f0"));
        assert!(rendered_values.contains(&"#b3b300"));
        assert!(rendered_values.contains(&"#4d4d00"));
        assert!(rendered_values.contains(&"olive"));
        assert!(rendered_values.contains(&"#7a7a06"));
        assert!(rendered_values.contains(&"gray"));
        assert!(rendered_values.contains(&"#bf4040"));
        assert!(rendered_values.contains(&"#ffff40"));
        assert!(rendered_values.contains(&"#ff0"));
    }

    #[test]
    fn static_value_resolution_emits_exact_legacy_global_color_values() {
        let report = summarize_static_stylesheet_value_resolution(
            "$red: red(mix(red, blue)); $green: green(rgb(127.5, 10, 20)); $blue: blue(blue); $alpha: alpha(rgba(255, 0, 0, .5)); .button { z-index: $red; --g: $green; --b: $blue; opacity: $alpha; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.reference_count, 4);
        assert_eq!(report.resolved_count, 4);
        assert_eq!(report.raw_count, 0);
        let rendered_values = report
            .values
            .iter()
            .filter_map(|value| value.rendered_value.as_deref())
            .collect::<Vec<_>>();
        assert!(rendered_values.contains(&"127.5"));
        assert!(rendered_values.contains(&"10"));
        assert!(rendered_values.contains(&"255"));
        assert!(rendered_values.contains(&"0.5"));
    }

    #[test]
    fn static_value_resolution_emits_exact_sass_rgb_color_constructor_values() {
        let report = summarize_static_stylesheet_value_resolution(
            "$transparent: rgba(red, .5); $opaque: rgb(red, 1); .button { color: $transparent; border-color: $opaque; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.reference_count, 2);
        assert_eq!(report.resolved_count, 2);
        assert_eq!(report.raw_count, 0);
        let rendered_values = report
            .values
            .iter()
            .filter_map(|value| value.rendered_value.as_deref())
            .collect::<Vec<_>>();
        assert!(rendered_values.contains(&"#ff000080"));
        assert!(rendered_values.contains(&"red"));
    }

    #[test]
    fn static_value_resolution_emits_exact_sass_hsl_color_constructor_values() {
        let report = summarize_static_stylesheet_value_resolution(
            "$tone: hsl(180, 100%, 50%); $overlay: hsla(120, 100%, 50%, .5); .button { color: $tone; background: $overlay; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.reference_count, 2);
        assert_eq!(report.resolved_count, 2);
        assert_eq!(report.raw_count, 0);
        let rendered_values = report
            .values
            .iter()
            .filter_map(|value| value.rendered_value.as_deref())
            .collect::<Vec<_>>();
        assert!(rendered_values.contains(&"#0ff"));
        assert!(rendered_values.contains(&"#00ff0080"));
    }

    #[test]
    fn static_value_resolution_emits_exact_sass_opacity_color_values() {
        let report = summarize_static_stylesheet_value_resolution(
            "$transparent: transparentize(red, .25); $faded: fade-in(rgba(red, .5), .25); $opaque: opacify(rgba(red, .5), .25); $adjusted: color.adjust(red, $alpha: -.25); $changed: color.change(red, $alpha: .5); $scaled: color.scale(rgba(red, .5), $alpha: -50%); .button { color: $transparent; background: $faded; border-color: $opaque; outline-color: $adjusted; caret-color: $changed; text-decoration-color: $scaled; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.reference_count, 6);
        assert_eq!(report.resolved_count, 6);
        assert_eq!(report.raw_count, 0);
        assert!(
            report
                .values
                .iter()
                .all(|value| value.abstract_value_kind == "exact")
        );
        let rendered_values = report
            .values
            .iter()
            .filter_map(|value| value.rendered_value.as_deref())
            .collect::<Vec<_>>();
        assert_eq!(
            rendered_values
                .iter()
                .filter(|value| **value == "#ff0000bf")
                .count(),
            4
        );
        assert!(rendered_values.contains(&"#ff000080"));
        assert!(rendered_values.contains(&"#ff000040"));
    }

    #[test]
    fn static_value_resolution_emits_exact_nested_sass_color_helper_values() {
        let report = summarize_static_stylesheet_value_resolution(
            "$tone: list.nth(list.append(1px, transparentize(red, .25)), 2); $scaled: list.nth(list.append(1px, color.scale(#808000, $lightness: 50%)), 2); $opacity: list.nth(list.append(1px, color.opacity(rgba(red, .5))), 2); .button { color: $tone; background: $scaled; opacity: $opacity; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.reference_count, 3);
        assert_eq!(report.resolved_count, 3);
        assert_eq!(report.raw_count, 0);
        assert!(
            report
                .values
                .iter()
                .all(|value| value.abstract_value_kind == "exact")
        );
        let rendered_values = report
            .values
            .iter()
            .filter_map(|value| value.rendered_value.as_deref())
            .collect::<Vec<_>>();
        assert!(rendered_values.contains(&"#ff0000bf"));
        assert!(rendered_values.contains(&"#ffff40"));
        assert!(rendered_values.contains(&"0.5"));
    }

    #[test]
    fn static_scss_evaluation_preserves_css_rgba_constructor_text() {
        let report = derive_static_stylesheet_module_evaluation(
            "$transparent: rgba(255, 0, 0, .5); .button { color: $transparent; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(
            report
                .evaluated_css
                .contains(".button { color: rgba(255, 0, 0, .5); }")
        );
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_value_resolution_keeps_css_filter_alpha_raw() {
        let report = summarize_static_stylesheet_value_resolution(
            "$filter: alpha(opacity=50); .button { filter: $filter; }",
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
            Some("alpha(opacity=50)")
        );
    }

    #[test]
    fn static_value_resolution_keeps_unspecified_hsl_color_channels_raw() {
        let report = summarize_static_stylesheet_value_resolution(
            "$hue: color.channel(red, \"hue\"); .button { z-index: $hue; }",
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
            Some("color.channel(red, \"hue\")")
        );
    }

    #[test]
    fn static_value_resolution_emits_exact_ie_hex_str_values() {
        let report = summarize_static_stylesheet_value_resolution(
            "$legacy: ie-hex-str(rgba(red, .5)); .button { color: $legacy; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.reference_count, 1);
        assert_eq!(report.resolved_count, 1);
        assert_eq!(report.raw_count, 0);
        assert_eq!(report.unsupported_dynamic_count, 0);
        assert_eq!(report.values[0].outcome, "resolved");
        assert_eq!(report.values[0].reason, "resolved");
        assert_eq!(
            report.values[0].rendered_value.as_deref(),
            Some("#80ff0000")
        );
    }

    #[test]
    fn static_value_resolution_emits_exact_static_inspect_values() {
        let report = summarize_static_stylesheet_value_resolution(
            "$tone: meta.inspect(red); $gap: inspect(2px); .button { color: $tone; margin: $gap; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.reference_count, 2);
        assert_eq!(report.resolved_count, 2);
        assert_eq!(report.raw_count, 0);
        assert_eq!(report.unsupported_dynamic_count, 0);
        let rendered_values = report
            .values
            .iter()
            .filter_map(|value| value.rendered_value.as_deref())
            .collect::<Vec<_>>();
        assert!(rendered_values.contains(&"red"));
        assert!(rendered_values.contains(&"2px"));
    }

    #[test]
    fn static_value_resolution_emits_exact_static_color_values() {
        let report = summarize_static_stylesheet_value_resolution(
            "$tone: color-mix(in srgb, red 50%, blue 50%); .button { color: $tone; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.reference_count, 1);
        assert_eq!(report.resolved_count, 1);
        assert_eq!(report.raw_count, 0);
        assert_eq!(report.values[0].outcome, "resolved");
        assert_eq!(report.values[0].abstract_value_kind, "exact");
        assert_eq!(report.values[0].rendered_value.as_deref(), Some("purple"));
    }

    #[test]
    fn static_scss_evaluation_reports_exact_color_replacements_without_cutover() {
        let report = derive_static_stylesheet_module_evaluation(
            "$tone: color-mix(in srgb, red 50%, blue 50%); .button { color: $tone; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(
            report.resolved_replacements[0].text,
            "color-mix(in srgb, red 50%, blue 50%)"
        );
        assert_eq!(
            report.resolved_replacements[0].rendered_value.as_deref(),
            Some("purple")
        );
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(
            report
                .evaluated_css
                .contains("color-mix(in srgb, red 50%, blue 50%)")
        );
        assert!(!report.evaluated_css.contains("color: purple"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_reports_exact_sass_color_mix_replacements() {
        let report = derive_static_stylesheet_module_evaluation(
            "$tone: color.mix(red, blue); .button { color: $tone; }",
            StyleDialect::Scss,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].text, "rgb(127.5, 0, 127.5)");
        assert_eq!(
            report.resolved_replacements[0].rendered_value.as_deref(),
            Some("purple")
        );
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert!(report.evaluated_css.contains("color: rgb(127.5, 0, 127.5)"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
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
