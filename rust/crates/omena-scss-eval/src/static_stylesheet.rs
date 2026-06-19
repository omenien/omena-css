use std::collections::{BTreeMap, BTreeSet};

use omena_abstract_value::{AbstractCssValueV0, abstract_css_value_from_text};
use omena_parser::{LexedToken, ParsedVariableFact, ParsedVariableFactKind, StyleDialect, lex};
use omena_syntax::SyntaxKind;
use omena_value_lattice::{
    NumericValueV0, SrgbColor, StaticSrgbColorWithAlpha, format_css_number,
    parse_basic_named_srgb_color, parse_color_function_value, parse_color_mix_value,
    parse_numeric_value_with_unit, parse_oklab_oklch_value, parse_reducible_ceil_value,
    parse_reducible_floor_value, parse_static_hsl_function_color_with_alpha,
    parse_static_hwb_function_color_with_alpha, parse_static_rgb_function_color_with_alpha,
    parse_static_srgb_color_with_alpha, parse_whole_function_value_arguments,
    parse_whole_function_value_inner, split_top_level_whitespace_value_components_owned,
    substitute_static_css_function_references_in_value_until_stable,
};
use serde::Serialize;

use crate::{
    abstract_css_value_kind, abstract_css_value_reflected_in_legacy_css,
    scss_metadata::reduce_static_scss_metadata_with_context,
    static_loop_frames::parse_static_scss_each_loop_binding_frames,
    summarize_omena_scss_eval_oracle,
    value_eval::{
        reduce_static_less_numeric_value, reduce_static_scss_value,
        static_scss_bang_usage_is_comparison_only, static_scss_literal_truthiness,
    },
};

mod less_guard;
mod model;
mod oracle_corpus;
mod value_resolution_model;

use less_guard::{
    static_less_guard_unit_text, static_less_guard_value_has_unit,
    static_less_guard_value_is_color, static_less_guard_value_is_keyword,
    static_less_guard_value_is_number, static_less_guard_value_is_string,
    static_less_guard_value_is_url, static_less_mixin_guard_depends_on_default,
    static_less_mixin_guard_matches, static_less_value_condition_matches,
};
use model::{
    StaticLessBodyPropertyValueOutcome, StaticLessDetachedRulesetAccessor,
    StaticLessDetachedRulesetAccessorEvaluationEdits,
    StaticLessDetachedRulesetAccessorRenderOutcome, StaticLessDetachedRulesetCall,
    StaticLessDetachedRulesetCallRenderOutcome, StaticLessDetachedRulesetDeclaration,
    StaticLessDetachedRulesetEvaluationEdits, StaticLessMixinAccessor,
    StaticLessMixinAccessorCallRenderOutcome, StaticLessMixinAccessorEvaluationEdits,
    StaticLessMixinAccessorRenderOutcome, StaticLessMixinAccessorRenderResult,
    StaticLessMixinBodyLocalDeclaration, StaticLessMixinCall, StaticLessMixinCallRenderOutcome,
    StaticLessMixinDeclaration, StaticLessMixinEvaluationEdits, StaticLessMixinRenderContext,
    StaticLessMixinRenderOutcome, StaticLessMixinRenderResult, StaticLessResolvedValue,
    StaticScssFunctionArgument, StaticScssFunctionCall, StaticScssFunctionDeclaration,
    StaticScssFunctionEvaluationEdits, StaticScssFunctionLocalScope,
    StaticScssFunctionLocalVariable, StaticScssFunctionParameter,
    StaticScssFunctionResolutionContext, StaticScssFunctionReturnClause, StaticScssLoopHeader,
    StaticScssMixinBodyLocalDeclaration, StaticScssMixinDeclaration,
    StaticScssMixinEvaluationEdits, StaticScssMixinIncludeCall, StaticScssMixinRenderResult,
    StaticStylesheetEvaluationEdit, StaticStylesheetPropertyDeclaration, StaticStylesheetScope,
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
    pub native_replacement_legacy_reflection_count: usize,
    pub native_replacement_legacy_unreflected_count: usize,
    pub native_edit_count: usize,
    pub native_value_edit_count: usize,
    pub native_structural_edit_count: usize,
    pub native_edit_output_matches_evaluated_css: bool,
    pub resolved_replacements: Vec<OmenaScssEvalResolvedReplacementV0>,
    pub native_edits: Vec<OmenaScssEvalStaticStylesheetNativeEditV0>,
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
pub struct OmenaScssEvalStaticStylesheetNativeEditV0 {
    pub start: usize,
    pub end: usize,
    pub replacement: String,
    pub edit_kind: &'static str,
    pub abstract_value: Option<AbstractCssValueV0>,
    pub abstract_value_kind: Option<&'static str>,
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
        let Some(replacement) = resolve_static_scss_variable_value_at_position(
            fact.name.as_str(),
            reference_start,
            &scopes,
            &declarations,
            &mut stack,
        ) else {
            return build_static_stylesheet_preserved_evaluation_report_if_explained(
                style_source,
                dialect,
                StaticStylesheetVariableKind::Scss,
            );
        };
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
    let mut preserved_scss_evaluation_count = 0usize;
    if let Some(function_edits) = collect_static_scss_function_evaluation_edits(
        style_source,
        tokens,
        &function_declarations,
        &mixin_declarations,
        &scopes,
        &declarations,
    ) {
        if function_edits.preserved_raw_call_count > 0 {
            return build_static_stylesheet_preserved_evaluation_report_if_explained(
                style_source,
                dialect,
                StaticStylesheetVariableKind::Scss,
            );
        }
        edits.extend(function_edits.edits);
        resolved_replacements.extend(function_edits.replacements);
    }
    if let Some(mixin_edits) = collect_static_scss_mixin_evaluation_edits(
        style_source,
        tokens,
        &function_declarations,
        &mixin_declarations,
        &scopes,
        &declarations,
    ) {
        preserved_scss_evaluation_count += mixin_edits.preserved_raw_include_count;
        edits.extend(mixin_edits.edits);
    }

    let evaluated_css = apply_static_stylesheet_evaluation_edits(style_source, edits.clone())?;
    if evaluated_css == style_source && preserved_scss_evaluation_count == 0 {
        return None;
    }
    build_static_stylesheet_evaluation_report(
        style_source,
        dialect,
        StaticStylesheetVariableKind::Scss,
        evaluated_css,
        edits,
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
    let mut preserved_less_evaluation_count = 0usize;
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
            &property_declarations,
            detached_rulesets.as_slice(),
            &mut stack,
        )?;
        let replacement = replacement.text;
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
        let replacement = replacement.text;
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
    let detached_ruleset_accessor_evaluation_edits =
        collect_static_less_detached_ruleset_accessor_evaluation_edits(
            style_source,
            &detached_rulesets,
            &detached_ruleset_accessors,
            &mixin_declaration_ranges,
            &scopes,
            &declarations,
            &property_declarations,
        )?;
    preserved_less_evaluation_count +=
        detached_ruleset_accessor_evaluation_edits.preserved_raw_accessor_count;
    let detached_ruleset_evaluation_edits = collect_static_less_detached_ruleset_evaluation_edits(
        style_source,
        &detached_rulesets,
        &detached_ruleset_calls,
        &mixin_declarations,
        &mixin_declaration_ranges,
        &detached_ruleset_accessor_evaluation_edits.preserved_declaration_keys,
        &scopes,
        &declarations,
        &property_declarations,
    )?;
    preserved_less_evaluation_count += detached_ruleset_evaluation_edits.preserved_raw_call_count;
    edits.extend(detached_ruleset_evaluation_edits.edits);
    edits.extend(detached_ruleset_accessor_evaluation_edits.edits);
    let accessor_evaluation_edits = collect_static_less_mixin_accessor_evaluation_edits(
        style_source,
        tokens,
        &mixin_declarations,
        &detached_rulesets,
        &scopes,
        &declarations,
        &property_declarations,
        &detached_ruleset_ranges,
    )?;
    preserved_less_evaluation_count += accessor_evaluation_edits.preserved_raw_accessor_count;
    edits.extend(accessor_evaluation_edits.edits);
    if let Some(mixin_evaluation_edits) = collect_static_less_mixin_evaluation_edits(
        style_source,
        tokens,
        &mixin_declarations,
        &detached_rulesets,
        &scopes,
        &declarations,
        &property_declarations,
        &detached_ruleset_ranges,
    ) {
        preserved_less_evaluation_count +=
            mixin_evaluation_edits.preserved_non_rendering_call_count;
        edits.extend(mixin_evaluation_edits.edits);
    }

    let evaluated_css = apply_static_stylesheet_evaluation_edits(style_source, edits.clone())?;
    if evaluated_css == style_source && preserved_less_evaluation_count == 0 {
        return None;
    }
    build_static_stylesheet_evaluation_report(
        style_source,
        StyleDialect::Less,
        StaticStylesheetVariableKind::Less,
        evaluated_css,
        edits,
        resolved_replacements,
    )
}

fn build_static_stylesheet_evaluation_report(
    style_source: &str,
    dialect: StyleDialect,
    variable_kind: StaticStylesheetVariableKind,
    evaluated_css: String,
    native_edit_source: Vec<StaticStylesheetEvaluationEdit>,
    resolved_replacements: Vec<OmenaScssEvalResolvedReplacementV0>,
) -> Option<OmenaScssEvalStaticStylesheetEvaluationV0> {
    let value_resolution = summarize_static_stylesheet_value_resolution(style_source, dialect)?;
    build_static_stylesheet_evaluation_report_with_value_resolution(
        style_source,
        dialect,
        variable_kind,
        evaluated_css,
        native_edit_source,
        resolved_replacements,
        value_resolution,
    )
}

fn build_static_stylesheet_preserved_evaluation_report_if_explained(
    style_source: &str,
    dialect: StyleDialect,
    variable_kind: StaticStylesheetVariableKind,
) -> Option<OmenaScssEvalStaticStylesheetEvaluationV0> {
    let value_resolution = summarize_static_stylesheet_value_resolution(style_source, dialect)?;
    if value_resolution.raw_count == 0 && value_resolution.top_count == 0 {
        return None;
    }
    build_static_stylesheet_evaluation_report_with_value_resolution(
        style_source,
        dialect,
        variable_kind,
        style_source.to_string(),
        Vec::new(),
        Vec::new(),
        value_resolution,
    )
}

fn build_static_stylesheet_evaluation_report_with_value_resolution(
    style_source: &str,
    dialect: StyleDialect,
    variable_kind: StaticStylesheetVariableKind,
    evaluated_css: String,
    native_edit_source: Vec<StaticStylesheetEvaluationEdit>,
    resolved_replacements: Vec<OmenaScssEvalResolvedReplacementV0>,
    value_resolution: OmenaScssEvalStaticValueResolutionReportV0,
) -> Option<OmenaScssEvalStaticStylesheetEvaluationV0> {
    let oracle = summarize_omena_scss_eval_oracle(style_source, dialect, evaluated_css.as_str());
    if !oracle.all_legacy_declaration_values_preserved {
        return None;
    }
    let native_replacement_legacy_reflection_count =
        count_native_replacements_reflected_in_legacy_css(
            resolved_replacements.as_slice(),
            evaluated_css.as_str(),
            dialect,
        );
    let native_replacement_legacy_unreflected_count = resolved_replacements
        .len()
        .saturating_sub(native_replacement_legacy_reflection_count);
    let normalized_native_edit_source =
        normalize_static_stylesheet_evaluation_edits(style_source, native_edit_source)?;
    let native_edit_output = apply_normalized_static_stylesheet_evaluation_edits(
        style_source,
        &normalized_native_edit_source,
    );
    let native_edit_output_matches_evaluated_css = native_edit_output == evaluated_css;
    let native_edits = build_static_stylesheet_native_edits(
        normalized_native_edit_source,
        resolved_replacements.as_slice(),
    );
    let native_value_edit_count = native_edits
        .iter()
        .filter(|edit| edit.edit_kind == "valueReplacement")
        .count();
    let native_structural_edit_count = native_edits.len().saturating_sub(native_value_edit_count);
    Some(OmenaScssEvalStaticStylesheetEvaluationV0 {
        schema_version: "0",
        product: "omena-scss-eval.static-stylesheet-evaluation",
        evaluator: variable_kind.evaluator_label(),
        dialect: dialect_label(dialect),
        replacement_count: resolved_replacements.len(),
        native_replacement_legacy_reflection_count,
        native_replacement_legacy_unreflected_count,
        native_edit_count: native_edits.len(),
        native_value_edit_count,
        native_structural_edit_count,
        native_edit_output_matches_evaluated_css,
        resolved_replacements,
        native_edits,
        value_resolution,
        evaluated_css,
        oracle,
    })
}

fn count_native_replacements_reflected_in_legacy_css(
    replacements: &[OmenaScssEvalResolvedReplacementV0],
    evaluated_css: &str,
    dialect: StyleDialect,
) -> usize {
    replacements
        .iter()
        .filter(|replacement| {
            replacement
                .rendered_value
                .as_deref()
                .is_some_and(|rendered| {
                    abstract_css_value_reflected_in_legacy_css(
                        evaluated_css,
                        dialect,
                        rendered,
                        &replacement.abstract_value,
                    )
                })
        })
        .count()
}

fn build_static_stylesheet_native_edits(
    edits: Vec<StaticStylesheetEvaluationEdit>,
    replacements: &[OmenaScssEvalResolvedReplacementV0],
) -> Vec<OmenaScssEvalStaticStylesheetNativeEditV0> {
    edits
        .into_iter()
        .map(|edit| {
            let value_replacement =
                native_edit_value_replacement_for_static_edit(&edit, replacements);
            let edit_kind = value_replacement
                .map(|_| "valueReplacement")
                .unwrap_or_else(|| {
                    if edit.replacement.is_empty() {
                        "structuralRemoval"
                    } else {
                        "structuralReplacement"
                    }
                });
            OmenaScssEvalStaticStylesheetNativeEditV0 {
                start: edit.start,
                end: edit.end,
                replacement: edit.replacement,
                edit_kind,
                abstract_value: value_replacement
                    .map(|replacement| replacement.abstract_value.clone()),
                abstract_value_kind: value_replacement
                    .map(|replacement| replacement.abstract_value_kind),
            }
        })
        .collect()
}

fn native_edit_value_replacement_for_static_edit<'a>(
    edit: &StaticStylesheetEvaluationEdit,
    replacements: &'a [OmenaScssEvalResolvedReplacementV0],
) -> Option<&'a OmenaScssEvalResolvedReplacementV0> {
    replacements.iter().find(|replacement| {
        replacement.start == edit.start
            && replacement.end == edit.end
            && (replacement.text == edit.replacement
                || replacement.rendered_value.as_deref() == Some(edit.replacement.as_str()))
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
        let mut stack = BTreeSet::new();
        let resolution = resolve_static_less_variable_abstract_value_in_scope(
            fact.name.as_str(),
            reference_scope_id,
            scopes,
            &declarations,
            &property_declarations,
            detached_rulesets.as_slice(),
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
) -> Option<StaticScssFunctionEvaluationEdits> {
    let calls = collect_static_scss_function_calls(source, tokens, declarations)?;
    if calls.is_empty() {
        return Some(StaticScssFunctionEvaluationEdits {
            edits: Vec::new(),
            replacements: Vec::new(),
            preserved_raw_call_count: 0,
        });
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
        if resolution.outcome == StaticStylesheetResolutionOutcome::Top
            && resolution.reason != StaticStylesheetResolutionReason::UnresolvedReference
        {
            return Some(StaticScssFunctionEvaluationEdits {
                edits: Vec::new(),
                replacements: Vec::new(),
                preserved_raw_call_count: 1,
            });
        }
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

    Some(StaticScssFunctionEvaluationEdits {
        edits,
        replacements,
        preserved_raw_call_count: 0,
    })
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
) -> Option<StaticScssMixinEvaluationEdits> {
    let calls = collect_static_scss_mixin_include_calls(source, tokens, mixin_declarations)?;
    if calls.is_empty() {
        return Some(StaticScssMixinEvaluationEdits {
            edits: Vec::new(),
            preserved_raw_include_count: 0,
        });
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
    let mut preserved_declaration_names = BTreeSet::new();
    let mut used_function_declaration_names = BTreeSet::new();
    let mut preserved_raw_include_count = 0usize;
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
        let Some(rendered) = render_static_scss_mixin_include_body(
            source,
            tokens,
            declaration,
            call,
            call.start,
            context,
        ) else {
            preserved_raw_include_count += 1;
            preserved_declaration_names.insert(canonical_static_scss_function_name(
                declaration.name.as_str(),
            ));
            continue;
        };
        used_declaration_names.extend(rendered.used_mixin_declaration_names);
        used_function_declaration_names.extend(rendered.used_function_declaration_names);
        edits.push(StaticStylesheetEvaluationEdit {
            start: call.start,
            end: call.end,
            replacement: rendered.body,
        });
    }

    for declaration in mixin_declarations.iter().filter(|declaration| {
        let canonical_name = canonical_static_scss_function_name(declaration.name.as_str());
        used_declaration_names.contains(&canonical_name)
            && !preserved_declaration_names.contains(&canonical_name)
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

    Some(StaticScssMixinEvaluationEdits {
        edits,
        preserved_raw_include_count,
    })
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
        false,
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

#[allow(clippy::too_many_arguments)]
fn collect_static_less_mixin_evaluation_edits(
    source: &str,
    tokens: &[LexedToken],
    declarations: &[StaticLessMixinDeclaration],
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
    scopes: &[StaticStylesheetScope],
    variable_declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    property_declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    excluded_call_ranges: &[(usize, usize)],
) -> Option<StaticLessMixinEvaluationEdits> {
    let calls = collect_static_less_mixin_calls(source, tokens)?;
    let unsupported_suffix_ranges =
        collect_static_less_unsupported_mixin_call_suffix_ranges(source, tokens)?;
    if calls.is_empty() && unsupported_suffix_ranges.is_empty() {
        return Some(StaticLessMixinEvaluationEdits {
            edits: Vec::new(),
            preserved_non_rendering_call_count: 0,
        });
    }

    let declaration_ranges = static_less_mixin_declaration_ranges_from_declarations(declarations);
    let empty_captured_values = BTreeMap::new();
    let context = StaticLessMixinRenderContext {
        source,
        declarations,
        detached_ruleset_declarations,
        scopes,
        variable_declarations,
        property_declarations,
        captured_values: &empty_captured_values,
    };
    let mut edits = Vec::new();
    let mut preserved_non_rendering_call_count = 0usize;
    let mut used_declaration_names = BTreeSet::new();
    preserved_non_rendering_call_count += unsupported_suffix_ranges
        .iter()
        .filter(|(start, _)| {
            !static_stylesheet_position_is_inside_ranges(*start, &declaration_ranges)
                && !static_stylesheet_position_is_inside_ranges(*start, excluded_call_ranges)
        })
        .count();
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
        match rendered {
            StaticLessMixinCallRenderOutcome::Rendered(rendered) => {
                used_declaration_names.extend(rendered.used_declaration_names);
                edits.push(StaticStylesheetEvaluationEdit {
                    start: call.start,
                    end: call.end,
                    replacement: rendered.body,
                });
            }
            StaticLessMixinCallRenderOutcome::PreservedNoOutput => {
                preserved_non_rendering_call_count += 1;
            }
        }
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
    Some(StaticLessMixinEvaluationEdits {
        edits,
        preserved_non_rendering_call_count,
    })
}

#[allow(clippy::too_many_arguments)]
fn collect_static_less_mixin_accessor_evaluation_edits(
    source: &str,
    tokens: &[LexedToken],
    declarations: &[StaticLessMixinDeclaration],
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
    scopes: &[StaticStylesheetScope],
    variable_declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    property_declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    excluded_ranges: &[(usize, usize)],
) -> Option<StaticLessMixinAccessorEvaluationEdits> {
    let accessors = collect_static_less_mixin_accessors(source, tokens)?;
    if accessors.is_empty() {
        return Some(StaticLessMixinAccessorEvaluationEdits {
            edits: Vec::new(),
            preserved_raw_accessor_count: 0,
        });
    }

    let declaration_ranges = static_less_mixin_declaration_ranges_from_declarations(declarations);
    let empty_captured_values = BTreeMap::new();
    let context = StaticLessMixinRenderContext {
        source,
        declarations,
        detached_ruleset_declarations,
        scopes,
        variable_declarations,
        property_declarations,
        captured_values: &empty_captured_values,
    };
    let mut edits = Vec::new();
    let mut preserved_raw_accessor_count = 0usize;
    let mut used_declaration_names = BTreeSet::new();
    for accessor in accessors.iter().filter(|accessor| {
        !static_stylesheet_position_is_inside_ranges(accessor.start, &declaration_ranges)
            && !static_stylesheet_position_is_inside_ranges(accessor.start, excluded_ranges)
    }) {
        let call_scope_id = static_stylesheet_scope_for_position(scopes, accessor.start)?;
        let rendered = render_static_less_mixin_accessor(accessor, call_scope_id, context)?;
        let Some(rendered) = rendered else {
            continue;
        };
        match rendered {
            StaticLessMixinAccessorCallRenderOutcome::Rendered(rendered) => {
                used_declaration_names.insert(rendered.used_declaration_name);
                edits.push(StaticStylesheetEvaluationEdit {
                    start: accessor.start,
                    end: accessor.end,
                    replacement: rendered.value,
                });
            }
            StaticLessMixinAccessorCallRenderOutcome::PreservedRaw => {
                preserved_raw_accessor_count += 1;
            }
        }
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
    Some(StaticLessMixinAccessorEvaluationEdits {
        edits,
        preserved_raw_accessor_count,
    })
}

#[allow(clippy::too_many_arguments)]
fn collect_static_less_detached_ruleset_evaluation_edits(
    source: &str,
    declarations: &[StaticLessDetachedRulesetDeclaration],
    calls: &[StaticLessDetachedRulesetCall],
    mixin_declarations: &[StaticLessMixinDeclaration],
    mixin_declaration_ranges: &[(usize, usize)],
    preserved_declaration_keys: &BTreeSet<(usize, String)>,
    scopes: &[StaticStylesheetScope],
    variable_declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    property_declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
) -> Option<StaticLessDetachedRulesetEvaluationEdits> {
    let declaration_ranges = static_less_detached_ruleset_ranges_from_declarations(declarations);
    let mut edits = Vec::new();
    let mut used_mixin_declaration_names = BTreeSet::new();
    let mut call_preserved_declaration_keys = BTreeSet::new();
    let mut preserved_raw_call_count = 0usize;

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
            property_declarations,
            mixin_declarations,
            declarations,
        )?;
        match replacement {
            StaticLessDetachedRulesetCallRenderOutcome::Rendered(replacement) => {
                used_mixin_declaration_names.extend(replacement.used_declaration_names);
                edits.push(StaticStylesheetEvaluationEdit {
                    start: call.start,
                    end: call.end,
                    replacement: replacement.body,
                });
            }
            StaticLessDetachedRulesetCallRenderOutcome::PreservedRaw => {
                preserved_raw_call_count += 1;
                call_preserved_declaration_keys
                    .insert((declaration.scope_id, declaration.name.clone()));
            }
        }
    }
    for declaration in declarations.iter().filter(|declaration| {
        !preserved_declaration_keys.contains(&(declaration.scope_id, declaration.name.clone()))
            && !call_preserved_declaration_keys
                .contains(&(declaration.scope_id, declaration.name.clone()))
    }) {
        edits.push(StaticStylesheetEvaluationEdit {
            start: declaration.span_start,
            end: declaration.span_end,
            replacement: String::new(),
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
    Some(StaticLessDetachedRulesetEvaluationEdits {
        edits,
        preserved_raw_call_count,
    })
}

fn collect_static_less_detached_ruleset_accessor_evaluation_edits(
    source: &str,
    declarations: &[StaticLessDetachedRulesetDeclaration],
    accessors: &[StaticLessDetachedRulesetAccessor],
    mixin_declaration_ranges: &[(usize, usize)],
    scopes: &[StaticStylesheetScope],
    variable_declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    property_declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
) -> Option<StaticLessDetachedRulesetAccessorEvaluationEdits> {
    if accessors.is_empty() {
        return Some(StaticLessDetachedRulesetAccessorEvaluationEdits {
            edits: Vec::new(),
            preserved_raw_accessor_count: 0,
            preserved_declaration_keys: BTreeSet::new(),
        });
    }

    let declaration_ranges = static_less_detached_ruleset_ranges_from_declarations(declarations);
    let mut edits = Vec::new();
    let mut preserved_raw_accessor_count = 0usize;
    let mut preserved_declaration_keys = BTreeSet::new();
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
            property_declarations,
            declarations,
        )?;
        match replacement {
            StaticLessDetachedRulesetAccessorRenderOutcome::Rendered(replacement) => {
                edits.push(StaticStylesheetEvaluationEdit {
                    start: accessor.start,
                    end: accessor.end,
                    replacement,
                });
            }
            StaticLessDetachedRulesetAccessorRenderOutcome::PreservedRaw => {
                preserved_raw_accessor_count += 1;
                preserved_declaration_keys.insert((declaration.scope_id, declaration.name.clone()));
            }
        }
    }
    Some(StaticLessDetachedRulesetAccessorEvaluationEdits {
        edits,
        preserved_raw_accessor_count,
        preserved_declaration_keys,
    })
}

#[allow(clippy::too_many_arguments)]
fn render_static_less_detached_ruleset_body(
    source: &str,
    declaration: &StaticLessDetachedRulesetDeclaration,
    call_scope_id: usize,
    scopes: &[StaticStylesheetScope],
    variable_declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    property_declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    mixin_declarations: &[StaticLessMixinDeclaration],
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
) -> Option<StaticLessDetachedRulesetCallRenderOutcome> {
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
        property_declarations,
        detached_ruleset_declarations,
    )?;
    let context = StaticLessMixinRenderContext {
        source,
        declarations: mixin_declarations,
        detached_ruleset_declarations,
        scopes,
        variable_declarations,
        property_declarations,
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
        return Some(StaticLessDetachedRulesetCallRenderOutcome::PreservedRaw);
    }
    Some(StaticLessDetachedRulesetCallRenderOutcome::Rendered(
        StaticLessMixinRenderResult {
            body: resolve_static_less_mixin_body_declaration_values(nested.body.as_str())?,
            used_declaration_names: nested.used_declaration_names,
        },
    ))
}

#[allow(clippy::too_many_arguments)]
fn render_static_less_detached_ruleset_accessor(
    source: &str,
    declaration: &StaticLessDetachedRulesetDeclaration,
    member: &str,
    call_scope_id: usize,
    scopes: &[StaticStylesheetScope],
    variable_declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    property_declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
) -> Option<StaticLessDetachedRulesetAccessorRenderOutcome> {
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
        property_declarations,
        captured_values: &empty_values,
    };
    let scoped_values = static_less_mixin_body_scoped_values(
        body,
        call_scope_id,
        &empty_values,
        &empty_values,
        scopes,
        variable_declarations,
        property_declarations,
        detached_ruleset_declarations,
    )?;
    if static_less_variable_name_is_safe(member) {
        return Some(match scoped_values.get(member) {
            Some(value) => StaticLessDetachedRulesetAccessorRenderOutcome::Rendered(value.clone()),
            None => StaticLessDetachedRulesetAccessorRenderOutcome::PreservedRaw,
        });
    }
    Some(
        match static_less_body_property_value(body, member, &scoped_values, call_scope_id, context)?
        {
            StaticLessBodyPropertyValueOutcome::Resolved(value) => {
                StaticLessDetachedRulesetAccessorRenderOutcome::Rendered(value)
            }
            StaticLessBodyPropertyValueOutcome::MemberNotFound => {
                StaticLessDetachedRulesetAccessorRenderOutcome::PreservedRaw
            }
        },
    )
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

fn collect_static_less_unsupported_mixin_call_suffix_ranges(
    source: &str,
    tokens: &[LexedToken],
) -> Option<Vec<(usize, usize)>> {
    let mut ranges = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        if static_less_mixin_call_context_is_plain(tokens, index)
            && let Some(((start, end), semicolon_index)) =
                static_less_namespace_unsupported_mixin_call_suffix_at(source, tokens, index)
        {
            ranges.push((start, end));
            index = semicolon_index + 1;
            continue;
        }
        let Some((_, open_index)) = static_less_mixin_signature_at(tokens, index) else {
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
        let Some((semicolon_index, suffix)) =
            static_less_mixin_call_semicolon_suffix(source, tokens, close_index)
        else {
            index += 1;
            continue;
        };
        if !static_less_mixin_call_suffix_is_supported(suffix) {
            ranges.push((
                static_stylesheet_token_start(&tokens[index]),
                static_stylesheet_token_end(&tokens[semicolon_index]),
            ));
        }
        index = semicolon_index + 1;
    }
    ranges.sort();
    Some(ranges)
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

fn static_less_namespace_unsupported_mixin_call_suffix_at(
    source: &str,
    tokens: &[LexedToken],
    index: usize,
) -> Option<((usize, usize), usize)> {
    let (_, after_namespace_index) = static_less_namespace_name_at(tokens, index)?;
    let namespace_arguments_index =
        static_stylesheet_skip_trivia_tokens(tokens, after_namespace_index);
    let separator_index = if tokens
        .get(namespace_arguments_index)
        .is_some_and(|token| token.kind == SyntaxKind::LeftParen)
    {
        let namespace_arguments_close_index = static_stylesheet_matching_token_index(
            tokens,
            namespace_arguments_index,
            SyntaxKind::LeftParen,
            SyntaxKind::RightParen,
        )?;
        static_stylesheet_skip_trivia_tokens(tokens, namespace_arguments_close_index + 1)
    } else {
        namespace_arguments_index
    };
    if tokens
        .get(separator_index)
        .is_none_or(|token| token.kind != SyntaxKind::GreaterThan)
    {
        return None;
    }
    let call_index = static_stylesheet_skip_trivia_tokens(tokens, separator_index + 1);
    let (_, open_index) = static_less_mixin_signature_at(tokens, call_index)?;
    let close_index = static_stylesheet_matching_token_index(
        tokens,
        open_index,
        SyntaxKind::LeftParen,
        SyntaxKind::RightParen,
    )?;
    let (semicolon_index, suffix) =
        static_less_mixin_call_semicolon_suffix(source, tokens, close_index)?;
    (!static_less_mixin_call_suffix_is_supported(suffix)).then_some((
        (
            static_stylesheet_token_start(&tokens[index]),
            static_stylesheet_token_end(&tokens[semicolon_index]),
        ),
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
    let (index, suffix) = static_less_mixin_call_semicolon_suffix(source, tokens, close_index)?;
    if suffix.is_empty() {
        return Some((index, false));
    }
    if suffix.eq_ignore_ascii_case("!important") {
        return Some((index, true));
    }
    None
}

fn static_less_mixin_call_semicolon_suffix<'a>(
    source: &'a str,
    tokens: &[LexedToken],
    close_index: usize,
) -> Option<(usize, &'a str)> {
    let suffix_start = static_stylesheet_token_end(tokens.get(close_index)?);
    for (index, token) in tokens.iter().enumerate().skip(close_index + 1) {
        if token.kind != SyntaxKind::Semicolon {
            continue;
        }
        let suffix = source
            .get(suffix_start..static_stylesheet_token_start(token))?
            .trim();
        return Some((index, suffix));
    }
    None
}

fn static_less_mixin_call_suffix_is_supported(suffix: &str) -> bool {
    suffix.is_empty() || suffix.eq_ignore_ascii_case("!important")
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
        return Some(StaticLessMixinRenderOutcome::GuardUnknown);
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
            context.property_declarations,
            None,
            context.detached_ruleset_declarations,
        )?;
        argument_values.insert(parameter, rendered_value);
    }
    if let Some(arguments_value) = static_less_mixin_arguments_value(call.arguments.as_slice()) {
        argument_values.insert("@arguments".to_string(), arguments_value);
    }
    if let Some(guard) = &declaration.guard {
        match static_less_mixin_guard_matches(
            guard,
            &argument_values,
            call_scope_id,
            call.start,
            context,
            default_matches,
        ) {
            Some(true) => {}
            Some(false) => {
                active_mixins.remove(&canonical_name);
                return Some(StaticLessMixinRenderOutcome::GuardNotMatched);
            }
            None => {
                active_mixins.remove(&canonical_name);
                return Some(StaticLessMixinRenderOutcome::GuardUnknown);
            }
        }
    }
    let body = render_static_less_mixin_body_variables(
        body,
        call_scope_id,
        &argument_values,
        context.captured_values,
        context.scopes,
        context.variable_declarations,
        context.property_declarations,
        context.detached_ruleset_declarations,
    )?;
    let nested = render_static_less_mixin_body_nested_calls(
        body.as_str(),
        call_scope_id,
        context,
        active_mixins,
    )?;
    let nested_lexed = lex(nested.body.as_str(), StyleDialect::Less);
    if !collect_static_less_mixin_calls(nested.body.as_str(), nested_lexed.tokens())?.is_empty()
        || !collect_static_less_detached_ruleset_calls(nested.body.as_str(), nested_lexed.tokens())?
            .is_empty()
    {
        active_mixins.remove(&canonical_name);
        return Some(StaticLessMixinRenderOutcome::GuardUnknown);
    }
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
) -> Option<Option<StaticLessMixinAccessorCallRenderOutcome>> {
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
    let mut saw_parameter_match = false;
    let mut saw_guard_not_matched = false;
    let mut saw_member_not_found = false;
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
        saw_parameter_match = true;
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
            StaticLessMixinAccessorRenderOutcome::GuardNotMatched => {
                saw_guard_not_matched = true;
            }
            StaticLessMixinAccessorRenderOutcome::GuardUnknown => {
                saw_guard_not_matched = true;
            }
            StaticLessMixinAccessorRenderOutcome::MemberNotFound => {
                saw_member_not_found = true;
            }
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
        saw_parameter_match = true;
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
            StaticLessMixinAccessorRenderOutcome::GuardNotMatched => {
                saw_guard_not_matched = true;
            }
            StaticLessMixinAccessorRenderOutcome::GuardUnknown => {
                saw_guard_not_matched = true;
            }
            StaticLessMixinAccessorRenderOutcome::MemberNotFound => {
                saw_member_not_found = true;
            }
        }
    }

    let mut rendered_values = rendered_values.into_iter();
    let Some(rendered) = rendered_values.next() else {
        if saw_parameter_match && (saw_member_not_found || saw_guard_not_matched) {
            return Some(Some(StaticLessMixinAccessorCallRenderOutcome::PreservedRaw));
        }
        return Some(None);
    };
    rendered_values.next().is_none().then_some(Some(
        StaticLessMixinAccessorCallRenderOutcome::Rendered(rendered),
    ))
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
            context.property_declarations,
            None,
            context.detached_ruleset_declarations,
        )?;
        argument_values.insert(parameter, rendered_value);
    }
    if let Some(arguments_value) = static_less_mixin_arguments_value(call.arguments.as_slice()) {
        argument_values.insert("@arguments".to_string(), arguments_value);
    }
    if let Some(guard) = &declaration.guard {
        match static_less_mixin_guard_matches(
            guard,
            &argument_values,
            call_scope_id,
            call.start,
            context,
            default_matches,
        ) {
            Some(true) => {}
            Some(false) => return Some(StaticLessMixinAccessorRenderOutcome::GuardNotMatched),
            None => return Some(StaticLessMixinAccessorRenderOutcome::GuardUnknown),
        }
    }

    let scoped_values = static_less_mixin_body_scoped_values(
        body,
        call_scope_id,
        &argument_values,
        context.captured_values,
        context.scopes,
        context.variable_declarations,
        context.property_declarations,
        context.detached_ruleset_declarations,
    )?;
    let value = if static_less_variable_name_is_safe(member) {
        let Some(value) = scoped_values.get(member) else {
            return Some(StaticLessMixinAccessorRenderOutcome::MemberNotFound);
        };
        value.clone()
    } else {
        match static_less_mixin_accessor_property_value(
            body,
            member,
            &scoped_values,
            call_scope_id,
            context,
        )? {
            StaticLessBodyPropertyValueOutcome::Resolved(value) => value,
            StaticLessBodyPropertyValueOutcome::MemberNotFound => {
                return Some(StaticLessMixinAccessorRenderOutcome::MemberNotFound);
            }
        }
    };
    Some(StaticLessMixinAccessorRenderOutcome::Rendered(
        StaticLessMixinAccessorRenderResult {
            value,
            used_declaration_name: canonical_name,
        },
    ))
}

#[allow(clippy::too_many_arguments)]
fn static_less_mixin_body_scoped_values(
    body: &str,
    call_scope_id: usize,
    argument_values: &BTreeMap<String, String>,
    captured_values: &BTreeMap<String, String>,
    scopes: &[StaticStylesheetScope],
    variable_declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    property_declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
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
            property_declarations,
            None,
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
) -> Option<StaticLessBodyPropertyValueOutcome> {
    static_less_body_property_value(body, member, scoped_values, call_scope_id, context)
}

fn static_less_body_property_value(
    body: &str,
    member: &str,
    scoped_values: &BTreeMap<String, String>,
    call_scope_id: usize,
    context: StaticLessMixinRenderContext<'_>,
) -> Option<StaticLessBodyPropertyValueOutcome> {
    if !static_stylesheet_property_name_is_safe(member) {
        return None;
    }
    let body_lexed = lex(body, StyleDialect::Less);
    let body_scopes = collect_static_stylesheet_scopes(body)?;
    let property_declarations =
        collect_static_less_body_property_declarations(body, body_lexed.tokens(), &body_scopes)?;
    let Some(declaration) = find_static_less_property_declaration(
        format!("${member}").as_str(),
        0,
        &body_scopes,
        &property_declarations,
    ) else {
        return Some(StaticLessBodyPropertyValueOutcome::MemberNotFound);
    };
    let resolved = resolve_static_less_mixin_value_with_bindings(
        declaration.value.as_str(),
        scoped_values,
        context.captured_values,
        call_scope_id,
        context.scopes,
        context.variable_declarations,
        context.property_declarations,
        None,
        context.detached_ruleset_declarations,
    )?;
    Some(StaticLessBodyPropertyValueOutcome::Resolved(resolved))
}

fn render_static_less_mixin_call(
    call: &StaticLessMixinCall,
    call_scope_id: usize,
    context: StaticLessMixinRenderContext<'_>,
    active_mixins: &mut BTreeSet<String>,
) -> Option<Option<StaticLessMixinCallRenderOutcome>> {
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
    let mut saw_parameter_match = false;
    let mut saw_guard_not_matched = false;
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
        saw_parameter_match = true;
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
            StaticLessMixinRenderOutcome::GuardNotMatched => {
                saw_guard_not_matched = true;
            }
            StaticLessMixinRenderOutcome::GuardUnknown => {
                saw_guard_not_matched = true;
            }
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
        saw_parameter_match = true;
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
            StaticLessMixinRenderOutcome::GuardNotMatched => {
                saw_guard_not_matched = true;
            }
            StaticLessMixinRenderOutcome::GuardUnknown => {
                saw_guard_not_matched = true;
            }
        }
    }
    if !saw_declaration {
        return Some(None);
    }
    if rendered_bodies.is_empty() {
        if !saw_parameter_match || saw_guard_not_matched {
            return Some(Some(StaticLessMixinCallRenderOutcome::PreservedNoOutput));
        }
        return None;
    }
    Some(Some(StaticLessMixinCallRenderOutcome::Rendered(
        StaticLessMixinRenderResult {
            body: rendered_bodies.join(" "),
            used_declaration_names,
        },
    )))
}

fn render_static_less_namespace_mixin_call(
    call: &StaticLessMixinCall,
    call_scope_id: usize,
    context: StaticLessMixinRenderContext<'_>,
    active_mixins: &mut BTreeSet<String>,
) -> Option<Option<StaticLessMixinCallRenderOutcome>> {
    let namespace = call.namespace.as_ref()?;
    let canonical_namespace = canonical_static_less_mixin_name(namespace.as_str());
    let mut saw_namespace = false;
    let mut saw_parameter_match = false;
    let mut saw_guard_not_matched = false;
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
        saw_parameter_match = true;
        for (parameter, argument) in bound_namespace_arguments {
            let rendered_value = resolve_static_less_mixin_value_with_bindings(
                argument.as_str(),
                &namespace_argument_values,
                context.captured_values,
                call_scope_id,
                context.scopes,
                context.variable_declarations,
                context.property_declarations,
                None,
                context.detached_ruleset_declarations,
            )?;
            namespace_argument_values.insert(parameter, rendered_value);
        }
        if let Some(guard) = &declaration.guard {
            match static_less_mixin_guard_matches(
                guard,
                &namespace_argument_values,
                call_scope_id,
                call.start,
                context,
                None,
            ) {
                Some(true) => {}
                Some(false) | None => {
                    saw_guard_not_matched = true;
                    continue;
                }
            }
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
            property_declarations: context.property_declarations,
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
            match rendered {
                StaticLessMixinCallRenderOutcome::Rendered(rendered) => {
                    rendered_bodies.push(rendered.body);
                }
                StaticLessMixinCallRenderOutcome::PreservedNoOutput => {}
            }
        }
        active_mixins.remove(&canonical_namespace);
    }

    if !saw_namespace {
        return Some(None);
    }
    if rendered_bodies.is_empty() {
        if !saw_parameter_match || saw_guard_not_matched {
            return Some(Some(StaticLessMixinCallRenderOutcome::PreservedNoOutput));
        }
        return None;
    }
    Some(Some(StaticLessMixinCallRenderOutcome::Rendered(
        StaticLessMixinRenderResult {
            body: rendered_bodies.join(" "),
            used_declaration_names: BTreeSet::from([canonical_namespace]),
        },
    )))
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
        match rendered {
            StaticLessMixinCallRenderOutcome::Rendered(rendered) => {
                used_declaration_names.extend(rendered.used_declaration_names);
                edits.push(StaticStylesheetEvaluationEdit {
                    start: call.start,
                    end: call.end,
                    replacement: rendered.body,
                });
            }
            StaticLessMixinCallRenderOutcome::PreservedNoOutput => {}
        }
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
            context.property_declarations,
            context.declarations,
            context.detached_ruleset_declarations,
        )?;
        match rendered {
            StaticLessDetachedRulesetCallRenderOutcome::Rendered(rendered) => {
                used_declaration_names.extend(rendered.used_declaration_names);
                edits.push(StaticStylesheetEvaluationEdit {
                    start: call.start,
                    end: call.end,
                    replacement: rendered.body,
                });
            }
            StaticLessDetachedRulesetCallRenderOutcome::PreservedRaw => {}
        }
    }

    Some(StaticLessMixinRenderResult {
        body: apply_static_stylesheet_evaluation_edits(body, edits)?,
        used_declaration_names,
    })
}

#[allow(clippy::too_many_arguments)]
fn render_static_less_mixin_body_variables(
    body: &str,
    call_scope_id: usize,
    argument_values: &BTreeMap<String, String>,
    captured_values: &BTreeMap<String, String>,
    scopes: &[StaticStylesheetScope],
    variable_declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    property_declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
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
        property_declarations,
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

    let references = collect_static_stylesheet_variable_references_with_options(
        body,
        StaticStylesheetVariableKind::Less,
        false,
        true,
    )?;
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
                property_declarations,
                detached_ruleset_declarations,
                &mut stack,
            )?
            .text
        };
        edits.push(StaticStylesheetEvaluationEdit {
            start: reference.start,
            end: reference.end,
            replacement,
        });
    }
    let body_lexed = lex(body, StyleDialect::Less);
    for token in body_lexed.tokens() {
        if token.kind != SyntaxKind::LessPropertyVariableToken {
            continue;
        }
        let reference_start = static_stylesheet_token_start(token);
        let mut stack = BTreeSet::new();
        let replacement = resolve_static_less_property_value_in_scope(
            token.text.as_str(),
            call_scope_id,
            scopes,
            property_declarations,
            &mut stack,
        )?
        .text;
        edits.push(StaticStylesheetEvaluationEdit {
            start: reference_start,
            end: static_stylesheet_token_end(token),
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

#[allow(clippy::too_many_arguments)]
fn resolve_static_less_mixin_value_with_bindings(
    value: &str,
    argument_values: &BTreeMap<String, String>,
    captured_values: &BTreeMap<String, String>,
    call_scope_id: usize,
    scopes: &[StaticStylesheetScope],
    variable_declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    property_declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    property_reference_position: Option<usize>,
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
) -> Option<String> {
    let references = collect_static_stylesheet_variable_references_with_options(
        value,
        StaticStylesheetVariableKind::Less,
        false,
        true,
    )?;
    let property_references = collect_static_less_property_variable_references(value)?;
    if references.is_empty() && property_references.is_empty() {
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
                property_declarations,
                detached_ruleset_declarations,
                &mut stack,
            )?
            .text
        };
        output.push_str(&value[cursor..reference.start]);
        output.push_str(&replacement);
        cursor = reference.end;
    }
    output.push_str(&value[cursor..]);
    let property_references = collect_static_less_property_variable_references(output.as_str())?;
    if !property_references.is_empty() {
        let mut property_stack = BTreeSet::new();
        output = resolve_static_less_property_value_text_with_position(
            output.as_str(),
            call_scope_id,
            scopes,
            property_declarations,
            &mut property_stack,
            property_reference_position,
        )?
        .text;
    }
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
            StaticStylesheetPropertyDeclaration {
                span_start: static_stylesheet_token_start(&tokens[index]),
                value,
            },
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

#[allow(clippy::too_many_arguments)]
fn resolve_static_less_variable_abstract_value_in_scope(
    name: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    property_declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
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
        declaration.span_start,
        scopes,
        declarations,
        property_declarations,
        detached_ruleset_declarations,
        stack,
        fuel - 1,
    );
    stack.remove(&stack_key);
    resolved
}

#[allow(clippy::too_many_arguments)]
fn resolve_static_less_variable_abstract_value_text(
    value: &str,
    scope_id: usize,
    reference_position: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    property_declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
    stack: &mut BTreeSet<(usize, String)>,
    fuel: usize,
) -> StaticStylesheetAbstractResolution {
    if let Some(value) = parse_static_less_isdefined_value_with_context(
        value,
        scope_id,
        reference_position,
        scopes,
        declarations,
        property_declarations,
        detached_ruleset_declarations,
    ) {
        return resolved_static_abstract_value(value.as_str());
    }
    if let Some(value) = parse_static_less_isruleset_value_with_context(
        value,
        scope_id,
        scopes,
        detached_ruleset_declarations,
    ) {
        return resolved_static_abstract_value(value.as_str());
    }
    let Some(references) = collect_static_stylesheet_variable_references_with_options(
        value,
        StaticStylesheetVariableKind::Less,
        false,
        true,
    ) else {
        return raw_static_abstract_value(
            value,
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        );
    };
    let Some(property_references) = collect_static_less_property_variable_references(value) else {
        return raw_static_abstract_value(
            value,
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        );
    };
    if references.is_empty() && property_references.is_empty() {
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
            property_declarations,
            detached_ruleset_declarations,
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
    if !property_references.is_empty() {
        let mut property_stack = BTreeSet::new();
        let resolved = resolve_static_less_property_references_in_value(
            output.as_str(),
            scope_id,
            scopes,
            property_declarations,
            &mut property_stack,
        );
        let Some(rendered_value) = resolved.rendered_value else {
            return top_static_abstract_value(resolved.reason);
        };
        output = rendered_value;
    }
    resolved_static_abstract_value(reduce_static_less_value(output).as_str())
}

fn resolve_static_less_variable_value_in_scope(
    name: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    property_declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
    stack: &mut BTreeSet<(usize, String)>,
) -> Option<StaticLessResolvedValue> {
    let stack_key = (scope_id, name.to_string());
    if !stack.insert(stack_key.clone()) {
        return None;
    }
    let declaration = find_static_less_variable_declaration(name, scope_id, scopes, declarations)?;
    let resolved = resolve_static_less_variable_value_text(
        declaration.value.trim(),
        scope_id,
        declaration.span_start,
        scopes,
        declarations,
        property_declarations,
        detached_ruleset_declarations,
        stack,
    );
    stack.remove(&stack_key);
    resolved.map(|resolved| {
        if resolved.escaped {
            resolved
        } else {
            reduce_static_less_value_with_escape_flag(resolved.text)
        }
    })
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

#[allow(clippy::too_many_arguments)]
fn resolve_static_less_variable_value_text(
    value: &str,
    scope_id: usize,
    reference_position: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    property_declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
    stack: &mut BTreeSet<(usize, String)>,
) -> Option<StaticLessResolvedValue> {
    if let Some(value) = parse_static_less_isdefined_value_with_context(
        value,
        scope_id,
        reference_position,
        scopes,
        declarations,
        property_declarations,
        detached_ruleset_declarations,
    ) {
        return Some(StaticLessResolvedValue {
            text: value,
            escaped: false,
        });
    }
    if let Some(value) = parse_static_less_isruleset_value_with_context(
        value,
        scope_id,
        scopes,
        detached_ruleset_declarations,
    ) {
        return Some(StaticLessResolvedValue {
            text: value,
            escaped: false,
        });
    }
    let references = collect_static_stylesheet_variable_references_with_options(
        value,
        StaticStylesheetVariableKind::Less,
        false,
        true,
    )?;
    let property_references = collect_static_less_property_variable_references(value)?;
    if references.is_empty() && property_references.is_empty() {
        if let Some(preserved) = preserve_static_less_dynamic_escaped_string_value(value) {
            return Some(preserved);
        }
        return static_stylesheet_literal_value_is_safe(value)
            .then(|| reduce_static_less_value_with_escape_flag(value.to_string()));
    }
    if !static_stylesheet_composite_value_is_safe(value) {
        return None;
    }

    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    let mut escaped = false;
    for reference in references {
        let resolved = resolve_static_less_variable_value_in_scope(
            reference.name.as_str(),
            scope_id,
            scopes,
            declarations,
            property_declarations,
            detached_ruleset_declarations,
            stack,
        )?;
        escaped |= resolved.escaped;
        output.push_str(&value[cursor..reference.start]);
        output.push_str(&resolved.text);
        cursor = reference.end;
    }
    output.push_str(&value[cursor..]);
    if !property_references.is_empty() {
        let mut property_stack = BTreeSet::new();
        let resolved = resolve_static_less_property_value_text(
            output.as_str(),
            scope_id,
            scopes,
            property_declarations,
            &mut property_stack,
        )?;
        escaped |= resolved.escaped;
        output = resolved.text;
    }
    Some(if escaped {
        StaticLessResolvedValue {
            text: output,
            escaped,
        }
    } else {
        reduce_static_less_value_with_escape_flag(output)
    })
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
) -> Option<StaticLessResolvedValue> {
    resolve_static_less_property_value_in_scope_with_position(
        name,
        scope_id,
        scopes,
        declarations,
        stack,
        None,
    )
}

fn resolve_static_less_property_value_in_scope_with_position(
    name: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    stack: &mut BTreeSet<(usize, String)>,
    reference_position: Option<usize>,
) -> Option<StaticLessResolvedValue> {
    let stack_key = (scope_id, name.to_string());
    if !stack.insert(stack_key.clone()) {
        return None;
    }
    let declaration = match reference_position {
        Some(position) => find_static_less_property_declaration_before(
            name,
            scope_id,
            scopes,
            declarations,
            position,
        ),
        None => find_static_less_property_declaration(name, scope_id, scopes, declarations),
    }?;
    let resolved = resolve_static_less_property_value_text_with_position(
        declaration.value.trim(),
        scope_id,
        scopes,
        declarations,
        stack,
        reference_position,
    );
    stack.remove(&stack_key);
    resolved.map(|resolved| {
        if resolved.escaped {
            resolved
        } else {
            reduce_static_less_value_with_escape_flag(resolved.text)
        }
    })
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

fn find_static_less_property_declaration_before<'a>(
    name: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &'a BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    reference_position: usize,
) -> Option<&'a StaticStylesheetPropertyDeclaration> {
    find_static_less_property_declaration(name, scope_id, scopes, declarations)
        .filter(|declaration| declaration.span_start < reference_position)
}

fn resolve_static_less_property_value_text(
    value: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    stack: &mut BTreeSet<(usize, String)>,
) -> Option<StaticLessResolvedValue> {
    resolve_static_less_property_value_text_with_position(
        value,
        scope_id,
        scopes,
        declarations,
        stack,
        None,
    )
}

fn resolve_static_less_property_value_text_with_position(
    value: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    stack: &mut BTreeSet<(usize, String)>,
    reference_position: Option<usize>,
) -> Option<StaticLessResolvedValue> {
    let references =
        collect_static_stylesheet_variable_references(value, StaticStylesheetVariableKind::Scss)?;
    if references.is_empty() {
        if let Some(preserved) = preserve_static_less_dynamic_escaped_string_value(value) {
            return Some(preserved);
        }
        return static_stylesheet_literal_value_is_safe(value)
            .then(|| reduce_static_less_value_with_escape_flag(value.to_string()));
    }
    if !static_stylesheet_composite_value_is_safe(value) {
        return None;
    }

    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    let mut escaped = false;
    for reference in references {
        let resolved = resolve_static_less_property_value_in_scope_with_position(
            reference.name.as_str(),
            scope_id,
            scopes,
            declarations,
            stack,
            reference_position,
        )?;
        escaped |= resolved.escaped;
        output.push_str(&value[cursor..reference.start]);
        output.push_str(&resolved.text);
        cursor = reference.end;
    }
    output.push_str(&value[cursor..]);
    Some(if escaped {
        StaticLessResolvedValue {
            text: output,
            escaped,
        }
    } else {
        reduce_static_less_value_with_escape_flag(output)
    })
}

fn resolve_static_less_property_references_in_value(
    value: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    stack: &mut BTreeSet<(usize, String)>,
) -> StaticStylesheetAbstractResolution {
    let Some(references) = collect_static_less_property_variable_references(value) else {
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
            STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
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
    reduce_static_less_value_with_escape_flag(value).text
}

fn reduce_static_less_value_with_escape_flag(value: String) -> StaticLessResolvedValue {
    if let Some(escaped) = parse_static_less_escape_value(value.as_str())
        .or_else(|| reduce_static_less_escaped_string_value(value.as_str()))
    {
        return StaticLessResolvedValue {
            text: escaped,
            escaped: true,
        };
    }
    if let Some(encoded) = parse_static_less_url_escape_value(value.as_str()) {
        return StaticLessResolvedValue {
            text: encoded,
            escaped: false,
        };
    }
    let value = reduce_static_less_numeric_value(value);
    let text = substitute_static_css_function_references_in_value_until_stable(
        value.as_str(),
        &[
            ("unit", parse_static_less_unit_value),
            ("get-unit", parse_static_less_get_unit_value),
            ("convert", parse_static_less_convert_value),
            ("if", parse_static_less_if_value),
            ("boolean", parse_static_less_boolean_value),
            ("percentage", parse_static_less_percentage_value),
            ("red", parse_static_less_red_value),
            ("green", parse_static_less_green_value),
            ("blue", parse_static_less_blue_value),
            ("alpha", parse_static_less_alpha_value),
            ("hue", parse_static_less_hue_value),
            ("saturation", parse_static_less_saturation_value),
            ("lightness", parse_static_less_lightness_value),
            ("hsv", parse_static_less_hsv_value),
            ("hsva", parse_static_less_hsva_value),
            ("hsvhue", parse_static_less_hsvhue_value),
            ("hsvsaturation", parse_static_less_hsvsaturation_value),
            ("hsvvalue", parse_static_less_hsvvalue_value),
            ("luma", parse_static_less_luma_value),
            ("luminance", parse_static_less_luminance_value),
            ("contrast", parse_static_less_contrast_value),
            ("color", parse_static_less_color_value),
            ("argb", parse_static_less_argb_value),
            ("fade", parse_static_less_fade_value),
            ("fadein", parse_static_less_fadein_value),
            ("fadeout", parse_static_less_fadeout_value),
            ("mix", parse_static_less_mix_value),
            ("tint", parse_static_less_tint_value),
            ("shade", parse_static_less_shade_value),
            ("multiply", parse_static_less_multiply_value),
            ("screen", parse_static_less_screen_value),
            ("overlay", parse_static_less_overlay_value),
            ("softlight", parse_static_less_softlight_value),
            ("hardlight", parse_static_less_hardlight_value),
            ("difference", parse_static_less_difference_value),
            ("exclusion", parse_static_less_exclusion_value),
            ("average", parse_static_less_average_value),
            ("negation", parse_static_less_negation_value),
            ("lighten", parse_static_less_lighten_value),
            ("darken", parse_static_less_darken_value),
            ("saturate", parse_static_less_saturate_value),
            ("desaturate", parse_static_less_desaturate_value),
            ("spin", parse_static_less_spin_value),
            ("greyscale", parse_static_less_greyscale_value),
            ("ceil", parse_reducible_ceil_value),
            ("floor", parse_reducible_floor_value),
            ("round", parse_static_less_round_value),
            ("pi", parse_static_less_pi_value),
            ("sin", parse_static_less_sin_value),
            ("cos", parse_static_less_cos_value),
            ("tan", parse_static_less_tan_value),
            ("asin", parse_static_less_asin_value),
            ("acos", parse_static_less_acos_value),
            ("atan", parse_static_less_atan_value),
            ("isnumber", parse_static_less_isnumber_value),
            ("iscolor", parse_static_less_iscolor_value),
            ("isstring", parse_static_less_isstring_value),
            ("iskeyword", parse_static_less_iskeyword_value),
            ("isurl", parse_static_less_isurl_value),
            ("isdefined", parse_static_less_isdefined_value),
            ("isruleset", parse_static_less_isruleset_value),
            ("ispixel", parse_static_less_ispixel_value),
            ("ispercentage", parse_static_less_ispercentage_value),
            ("isem", parse_static_less_isem_value),
            ("isunit", parse_static_less_isunit_value),
            ("length", parse_static_less_length_value),
            ("extract", parse_static_less_extract_value),
            ("range", parse_static_less_range_value),
            ("replace", parse_static_less_replace_value),
            ("%", parse_static_less_format_value),
        ],
    )
    .unwrap_or(value);
    let text = parse_static_less_rgb_color_value(text.as_str()).unwrap_or(text);
    StaticLessResolvedValue {
        text,
        escaped: false,
    }
}

fn parse_static_less_unit_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "unit")?;
    match arguments.as_slice() {
        [number] => {
            let parsed = parse_numeric_value_with_unit(number.trim())?;
            Some(format_static_less_number(parsed.value))
        }
        [number, unit] => {
            let parsed = parse_numeric_value_with_unit(number.trim())?;
            let unit = parse_static_less_unit_argument(unit.trim())?;
            Some(format!(
                "{}{}",
                format_static_less_number(parsed.value),
                unit
            ))
        }
        _ => None,
    }
}

fn parse_static_less_get_unit_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "get-unit")?;
    let [number] = arguments.as_slice() else {
        return None;
    };
    let parsed = parse_numeric_value_with_unit(number.trim())?;
    Some(parsed.unit.to_string())
}

fn parse_static_less_convert_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "convert")?;
    let [number, target_unit] = arguments.as_slice() else {
        return None;
    };
    let parsed = parse_numeric_value_with_unit(number.trim())?;
    let target_unit = parse_static_less_convert_unit_argument(target_unit.trim())?;
    let original = || {
        format!(
            "{}{}",
            format_static_less_channel_number(parsed.value),
            parsed.unit
        )
    };
    let Some(source_unit) = static_less_convertible_unit(parsed.unit) else {
        return Some(original());
    };
    let Some(target_unit) = static_less_convertible_unit(target_unit.as_str()) else {
        return Some(original());
    };
    if source_unit.family != target_unit.family {
        return Some(original());
    }
    let converted = parsed.value * source_unit.base_factor / target_unit.base_factor;
    Some(format!(
        "{}{}",
        format_static_less_channel_number(converted),
        target_unit.unit
    ))
}

fn parse_static_less_percentage_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "percentage")?;
    let [number] = arguments.as_slice() else {
        return None;
    };
    let parsed = parse_numeric_value_with_unit(number.trim())?;
    Some(format!(
        "{}%",
        format_static_less_number(parsed.value * 100.0)
    ))
}

fn parse_static_less_if_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "if")?;
    let [condition, truthy, falsey] = arguments.as_slice() else {
        return None;
    };
    Some(
        if static_less_value_condition_matches(condition.trim())? {
            truthy
        } else {
            falsey
        }
        .trim()
        .to_string(),
    )
}

fn parse_static_less_boolean_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "boolean")?;
    let [condition] = arguments.as_slice() else {
        return None;
    };
    Some(static_less_value_condition_matches(condition.trim())?.to_string())
}

fn parse_static_less_rgb_color_value(value: &str) -> Option<String> {
    let color = parse_static_rgb_function_color_with_alpha(value)?;
    Some(format_static_less_color_with_alpha(
        color,
        color.alpha.unwrap_or(1.0),
    ))
}

fn parse_static_less_red_value(value: &str) -> Option<String> {
    parse_static_less_color_channel_value(value, "red", |color| f64::from(color.color.red))
}

fn parse_static_less_green_value(value: &str) -> Option<String> {
    parse_static_less_color_channel_value(value, "green", |color| f64::from(color.color.green))
}

fn parse_static_less_blue_value(value: &str) -> Option<String> {
    parse_static_less_color_channel_value(value, "blue", |color| f64::from(color.color.blue))
}

fn parse_static_less_alpha_value(value: &str) -> Option<String> {
    parse_static_less_color_channel_value(value, "alpha", |color| color.alpha.unwrap_or(1.0))
}

fn parse_static_less_hue_value(value: &str) -> Option<String> {
    parse_static_less_hsl_channel_value(value, "hue", |channels| channels.hue)
}

fn parse_static_less_saturation_value(value: &str) -> Option<String> {
    parse_static_less_hsl_channel_value(value, "saturation", |channels| channels.saturation)
        .map(|value| format!("{value}%"))
}

fn parse_static_less_lightness_value(value: &str) -> Option<String> {
    parse_static_less_hsl_channel_value(value, "lightness", |channels| channels.lightness)
        .map(|value| format!("{value}%"))
}

fn parse_static_less_hsv_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "hsv")?;
    let [hue, saturation, value] = arguments.as_slice() else {
        return None;
    };
    parse_static_less_hsv_color(hue.trim(), saturation.trim(), value.trim(), "1")
}

fn parse_static_less_hsva_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "hsva")?;
    let [hue, saturation, value, alpha] = arguments.as_slice() else {
        return None;
    };
    parse_static_less_hsv_color(hue.trim(), saturation.trim(), value.trim(), alpha.trim())
}

fn parse_static_less_hsvhue_value(value: &str) -> Option<String> {
    parse_static_less_hsv_channel_value(value, "hsvhue", |channels| channels.hue)
}

fn parse_static_less_hsvsaturation_value(value: &str) -> Option<String> {
    parse_static_less_hsv_channel_value(value, "hsvsaturation", |channels| channels.saturation)
        .map(|value| format!("{value}%"))
}

fn parse_static_less_hsvvalue_value(value: &str) -> Option<String> {
    parse_static_less_hsv_channel_value(value, "hsvvalue", |channels| channels.value)
        .map(|value| format!("{value}%"))
}

fn parse_static_less_luma_value(value: &str) -> Option<String> {
    parse_static_less_luma_or_luminance_value(value, "luma", static_less_luma)
}

fn parse_static_less_luminance_value(value: &str) -> Option<String> {
    parse_static_less_luma_or_luminance_value(value, "luminance", static_less_luminance)
}

fn parse_static_less_contrast_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "contrast")?;
    let (color, dark, light, threshold) = match arguments.as_slice() {
        [color] => (color.as_str(), None, None, None),
        [color, dark] => (color.as_str(), Some(dark.as_str()), None, None),
        [color, dark, light] => (
            color.as_str(),
            Some(dark.as_str()),
            Some(light.as_str()),
            None,
        ),
        [color, dark, light, threshold] => (
            color.as_str(),
            Some(dark.as_str()),
            Some(light.as_str()),
            Some(threshold.as_str()),
        ),
        _ => return None,
    };
    let color = parse_static_less_color_argument(color.trim())?;
    let mut dark = match dark {
        Some(dark) => parse_static_less_color_argument(dark.trim())?,
        None => static_less_opaque_srgb_color(0, 0, 0),
    };
    let mut light = match light {
        Some(light) => parse_static_less_color_argument(light.trim())?,
        None => static_less_opaque_srgb_color(255, 255, 255),
    };
    if static_less_luma(dark.color) > static_less_luma(light.color) {
        std::mem::swap(&mut dark, &mut light);
    }
    let threshold = threshold
        .map(|threshold| parse_static_less_threshold_number(threshold.trim()))
        .unwrap_or(Some(0.43))?;
    let selected = if static_less_luma(color.color) < threshold {
        light
    } else {
        dark
    };
    Some(format_static_less_color_with_alpha(
        selected,
        selected.alpha.unwrap_or(1.0),
    ))
}

fn parse_static_less_color_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "color")?;
    let [color] = arguments.as_slice() else {
        return None;
    };
    let color = color.trim();
    if let Some(hex) = parse_static_less_quoted_hex_color_literal(color) {
        return Some(hex);
    }
    if let Some(named) = static_less_quoted_string_contents(color)
        .as_deref()
        .and_then(parse_basic_named_srgb_color)
    {
        return Some(format_static_less_color_with_alpha(
            StaticSrgbColorWithAlpha {
                color: named,
                alpha: None,
            },
            1.0,
        ));
    }
    let color = parse_static_less_color_argument(color)?;
    Some(format_static_less_color_with_alpha(
        color,
        color.alpha.unwrap_or(1.0),
    ))
}

fn parse_static_less_argb_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "argb")?;
    let [color] = arguments.as_slice() else {
        return None;
    };
    let color = parse_static_less_color_argument(color.trim())?;
    Some(format!(
        "#{:02x}{:02x}{:02x}{:02x}",
        static_less_alpha_byte(color.alpha.unwrap_or(1.0)),
        color.color.red,
        color.color.green,
        color.color.blue
    ))
}

fn parse_static_less_fade_value(value: &str) -> Option<String> {
    parse_static_less_alpha_transform_value(value, "fade", |_, amount, _| amount)
}

fn parse_static_less_fadein_value(value: &str) -> Option<String> {
    parse_static_less_alpha_transform_value(value, "fadein", |current, amount, mode| {
        current + static_less_unit_interval_delta(current, amount, mode)
    })
}

fn parse_static_less_fadeout_value(value: &str) -> Option<String> {
    parse_static_less_alpha_transform_value(value, "fadeout", |current, amount, mode| {
        current - static_less_unit_interval_delta(current, amount, mode)
    })
}

fn parse_static_less_mix_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "mix")?;
    let (first, second, weight) = match arguments.as_slice() {
        [first, second] => (first.as_str(), second.as_str(), 50.0),
        [first, second, weight] => (
            first.as_str(),
            second.as_str(),
            parse_static_less_percentage_points(weight.trim())?,
        ),
        _ => return None,
    };
    let first = parse_static_less_color_argument(first.trim())?;
    let second = parse_static_less_color_argument(second.trim())?;
    Some(format_static_less_mixed_color(first, second, weight))
}

fn parse_static_less_tint_value(value: &str) -> Option<String> {
    parse_static_less_tone_mix_value(value, "tint", "white")
}

fn parse_static_less_shade_value(value: &str) -> Option<String> {
    parse_static_less_tone_mix_value(value, "shade", "black")
}

fn parse_static_less_tone_mix_value(
    value: &str,
    function_name: &str,
    base_color: &str,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [color, weight] = arguments.as_slice() else {
        return None;
    };
    let base_color = parse_static_less_color_argument(base_color)?;
    let color = parse_static_less_color_argument(color.trim())?;
    let weight = parse_static_less_percentage_points(weight.trim())?;
    Some(format_static_less_mixed_color(base_color, color, weight))
}

fn parse_static_less_multiply_value(value: &str) -> Option<String> {
    parse_static_less_blend_value(value, "multiply", static_less_multiply_value)
}

fn parse_static_less_screen_value(value: &str) -> Option<String> {
    parse_static_less_blend_value(value, "screen", static_less_screen_value)
}

fn parse_static_less_overlay_value(value: &str) -> Option<String> {
    parse_static_less_blend_value(value, "overlay", static_less_overlay_value)
}

fn parse_static_less_softlight_value(value: &str) -> Option<String> {
    parse_static_less_blend_value(value, "softlight", static_less_softlight_value)
}

fn parse_static_less_hardlight_value(value: &str) -> Option<String> {
    parse_static_less_blend_value(value, "hardlight", |backdrop, source| {
        static_less_overlay_value(source, backdrop)
    })
}

fn parse_static_less_difference_value(value: &str) -> Option<String> {
    parse_static_less_blend_value(value, "difference", |backdrop, source| {
        (backdrop - source).abs()
    })
}

fn parse_static_less_exclusion_value(value: &str) -> Option<String> {
    parse_static_less_blend_value(value, "exclusion", static_less_exclusion_value)
}

fn parse_static_less_average_value(value: &str) -> Option<String> {
    parse_static_less_blend_value(value, "average", static_less_average_value)
}

fn parse_static_less_negation_value(value: &str) -> Option<String> {
    parse_static_less_blend_value(value, "negation", static_less_negation_value)
}

fn parse_static_less_blend_value(
    value: &str,
    function_name: &str,
    blend_channel: impl Fn(f64, f64) -> f64,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [first, second] = arguments.as_slice() else {
        return None;
    };
    let first = parse_static_less_color_argument(first.trim())?;
    let second = parse_static_less_color_argument(second.trim())?;
    Some(format_static_less_blended_color(
        first,
        second,
        blend_channel,
    ))
}

fn parse_static_less_lighten_value(value: &str) -> Option<String> {
    parse_static_less_hsl_amount_transform_value(value, "lighten", |mut channels, amount, mode| {
        channels.lightness = (channels.lightness
            + static_less_channel_delta(channels.lightness, amount, mode))
        .clamp(0.0, 100.0);
        channels
    })
}

fn parse_static_less_darken_value(value: &str) -> Option<String> {
    parse_static_less_hsl_amount_transform_value(value, "darken", |mut channels, amount, mode| {
        channels.lightness = (channels.lightness
            - static_less_channel_delta(channels.lightness, amount, mode))
        .clamp(0.0, 100.0);
        channels
    })
}

fn parse_static_less_saturate_value(value: &str) -> Option<String> {
    parse_static_less_hsl_amount_transform_value(value, "saturate", |mut channels, amount, mode| {
        channels.saturation = (channels.saturation
            + static_less_channel_delta(channels.saturation, amount, mode))
        .clamp(0.0, 100.0);
        channels
    })
}

fn parse_static_less_desaturate_value(value: &str) -> Option<String> {
    parse_static_less_hsl_amount_transform_value(
        value,
        "desaturate",
        |mut channels, amount, mode| {
            channels.saturation = (channels.saturation
                - static_less_channel_delta(channels.saturation, amount, mode))
            .clamp(0.0, 100.0);
            channels
        },
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticLessColorTransformMode {
    Absolute,
    Relative,
}

fn static_less_channel_delta(current: f64, amount: f64, mode: StaticLessColorTransformMode) -> f64 {
    match mode {
        StaticLessColorTransformMode::Absolute => amount,
        StaticLessColorTransformMode::Relative => current * amount / 100.0,
    }
}

fn static_less_unit_interval_delta(
    current: f64,
    amount: f64,
    mode: StaticLessColorTransformMode,
) -> f64 {
    match mode {
        StaticLessColorTransformMode::Absolute => amount,
        StaticLessColorTransformMode::Relative => current * amount,
    }
}

fn parse_static_less_color_transform_arguments(
    value: &str,
    function_name: &str,
) -> Option<(String, String, StaticLessColorTransformMode)> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    match arguments.as_slice() {
        [color, amount] => Some((
            color.trim().to_string(),
            amount.trim().to_string(),
            StaticLessColorTransformMode::Absolute,
        )),
        [color, amount, method] if method.trim().eq_ignore_ascii_case("relative") => Some((
            color.trim().to_string(),
            amount.trim().to_string(),
            StaticLessColorTransformMode::Relative,
        )),
        _ => None,
    }
}

fn parse_static_less_spin_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "spin")?;
    let [color, amount] = arguments.as_slice() else {
        return None;
    };
    let color = parse_static_less_color_argument(color.trim())?;
    let mut channels = static_less_hsl_channels(color);
    channels.hue =
        (channels.hue + parse_static_less_angle_degrees(amount.trim())?).rem_euclid(360.0);
    format_static_less_color_from_hsl_channels(color, channels)
}

fn parse_static_less_greyscale_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "greyscale")?;
    let [color] = arguments.as_slice() else {
        return None;
    };
    let color = parse_static_less_color_argument(color.trim())?;
    let mut channels = static_less_hsl_channels(color);
    channels.saturation = 0.0;
    format_static_less_color_from_hsl_channels(color, channels)
}

fn parse_static_less_hsl_amount_transform_value(
    value: &str,
    function_name: &str,
    transform: impl FnOnce(
        StaticLessHslChannels,
        f64,
        StaticLessColorTransformMode,
    ) -> StaticLessHslChannels,
) -> Option<String> {
    let (color, amount, mode) = parse_static_less_color_transform_arguments(value, function_name)?;
    let color = parse_static_less_color_argument(color.as_str())?;
    let amount = parse_static_less_percentage_points(amount.as_str())?;
    format_static_less_color_from_hsl_channels(
        color,
        transform(static_less_hsl_channels(color), amount, mode),
    )
}

fn parse_static_less_alpha_transform_value(
    value: &str,
    function_name: &str,
    transform: impl FnOnce(f64, f64, StaticLessColorTransformMode) -> f64,
) -> Option<String> {
    let (color, amount, mode) = parse_static_less_color_transform_arguments(value, function_name)?;
    let color = parse_static_less_color_argument(color.as_str())?;
    let amount = parse_static_less_alpha_amount(amount.as_str())?;
    let alpha = transform(color.alpha.unwrap_or(1.0), amount, mode).clamp(0.0, 1.0);
    Some(format_static_less_color_with_alpha(color, alpha))
}

fn parse_static_less_color_channel_value(
    value: &str,
    function_name: &str,
    channel: impl FnOnce(StaticSrgbColorWithAlpha) -> f64,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [color] = arguments.as_slice() else {
        return None;
    };
    let color = parse_static_less_color_argument(color.trim())?;
    Some(format_static_less_number(channel(color)))
}

fn parse_static_less_hsl_channel_value(
    value: &str,
    function_name: &str,
    channel: impl FnOnce(StaticLessHslChannels) -> f64,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [color] = arguments.as_slice() else {
        return None;
    };
    let color = parse_static_less_color_argument(color.trim())?;
    Some(format_static_less_channel_number(channel(
        static_less_hsl_channels(color),
    )))
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct StaticLessHslChannels {
    hue: f64,
    saturation: f64,
    lightness: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct StaticLessHsvChannels {
    hue: f64,
    saturation: f64,
    value: f64,
}

fn static_less_hsl_channels(color: StaticSrgbColorWithAlpha) -> StaticLessHslChannels {
    let red = f64::from(color.color.red) / 255.0;
    let green = f64::from(color.color.green) / 255.0;
    let blue = f64::from(color.color.blue) / 255.0;
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let lightness = (max + min) / 2.0;
    let delta = max - min;

    if delta == 0.0 {
        return StaticLessHslChannels {
            hue: 0.0,
            saturation: 0.0,
            lightness: lightness * 100.0,
        };
    }

    let saturation = delta / (1.0 - (2.0 * lightness - 1.0).abs());
    let hue_sector = if max == red {
        ((green - blue) / delta).rem_euclid(6.0)
    } else if max == green {
        (blue - red) / delta + 2.0
    } else {
        (red - green) / delta + 4.0
    };
    StaticLessHslChannels {
        hue: hue_sector * 60.0,
        saturation: saturation * 100.0,
        lightness: lightness * 100.0,
    }
}

fn static_less_hsv_channels(color: StaticSrgbColorWithAlpha) -> StaticLessHsvChannels {
    let red = f64::from(color.color.red) / 255.0;
    let green = f64::from(color.color.green) / 255.0;
    let blue = f64::from(color.color.blue) / 255.0;
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let delta = max - min;
    let saturation = if max == 0.0 { 0.0 } else { delta / max };

    if delta == 0.0 {
        return StaticLessHsvChannels {
            hue: 0.0,
            saturation: saturation * 100.0,
            value: max * 100.0,
        };
    }

    let hue_sector = if max == red {
        ((green - blue) / delta).rem_euclid(6.0)
    } else if max == green {
        (blue - red) / delta + 2.0
    } else {
        (red - green) / delta + 4.0
    };
    StaticLessHsvChannels {
        hue: hue_sector * 60.0,
        saturation: saturation * 100.0,
        value: max * 100.0,
    }
}

fn format_static_less_color_from_hsl_channels(
    original_color: StaticSrgbColorWithAlpha,
    channels: StaticLessHslChannels,
) -> Option<String> {
    let hue = format_static_less_channel_number(channels.hue.rem_euclid(360.0));
    let saturation = format_static_less_channel_number(channels.saturation.clamp(0.0, 100.0));
    let lightness = format_static_less_channel_number(channels.lightness.clamp(0.0, 100.0));
    let color = parse_static_hsl_function_color_with_alpha(&format!(
        "hsl({hue}, {saturation}%, {lightness}%)"
    ))?;
    Some(format_static_less_color_with_alpha(
        color,
        original_color.alpha.unwrap_or(1.0),
    ))
}

fn format_static_less_mixed_color(
    first: StaticSrgbColorWithAlpha,
    second: StaticSrgbColorWithAlpha,
    weight_percentage: f64,
) -> String {
    let first_alpha = first.alpha.unwrap_or(1.0);
    let second_alpha = second.alpha.unwrap_or(1.0);
    let first_stop = (weight_percentage.clamp(0.0, 100.0)) / 100.0;
    let channel_weight = static_less_mix_channel_weight(first_stop, first_alpha, second_alpha);
    let inverse_channel_weight = 1.0 - channel_weight;
    let alpha = first_alpha * first_stop + second_alpha * (1.0 - first_stop);
    let color = StaticSrgbColorWithAlpha {
        color: SrgbColor {
            red: static_less_mix_channel(
                first.color.red,
                second.color.red,
                channel_weight,
                inverse_channel_weight,
            ),
            green: static_less_mix_channel(
                first.color.green,
                second.color.green,
                channel_weight,
                inverse_channel_weight,
            ),
            blue: static_less_mix_channel(
                first.color.blue,
                second.color.blue,
                channel_weight,
                inverse_channel_weight,
            ),
        },
        alpha: None,
    };
    format_static_less_color_with_alpha(color, alpha)
}

fn parse_static_less_hsv_color(
    hue: &str,
    saturation: &str,
    value: &str,
    alpha: &str,
) -> Option<String> {
    let hue = parse_static_less_positive_degrees(hue)?;
    let saturation = parse_static_less_hsv_unit_interval(saturation)?;
    let value = parse_static_less_hsv_unit_interval(value)?;
    let alpha = parse_static_less_alpha_unit_interval(alpha)?;
    let hue = hue.rem_euclid(360.0);
    let sector = ((hue / 60.0).floor() as usize) % 6;
    let fraction = (hue / 60.0) - sector as f64;
    let candidates = [
        value,
        value * (1.0 - saturation),
        value * (1.0 - fraction * saturation),
        value * (1.0 - (1.0 - fraction) * saturation),
    ];
    let permutation = match sector {
        0 => [0, 3, 1],
        1 => [2, 0, 1],
        2 => [1, 0, 3],
        3 => [1, 2, 0],
        4 => [3, 1, 0],
        _ => [0, 1, 2],
    };
    Some(format_static_less_color_with_alpha(
        StaticSrgbColorWithAlpha {
            color: SrgbColor {
                red: static_less_blend_channel(candidates[permutation[0]] * 255.0),
                green: static_less_blend_channel(candidates[permutation[1]] * 255.0),
                blue: static_less_blend_channel(candidates[permutation[2]] * 255.0),
            },
            alpha: None,
        },
        alpha,
    ))
}

fn static_less_mix_channel_weight(first_stop: f64, first_alpha: f64, second_alpha: f64) -> f64 {
    let raw_weight = first_stop * 2.0 - 1.0;
    let alpha_delta = first_alpha - second_alpha;
    let weighted_alpha_delta = raw_weight * alpha_delta;
    let adjusted = if (weighted_alpha_delta + 1.0).abs() < f64::EPSILON {
        raw_weight
    } else {
        (raw_weight + alpha_delta) / (1.0 + weighted_alpha_delta)
    };
    (adjusted + 1.0) / 2.0
}

fn static_less_mix_channel(first: u8, second: u8, first_weight: f64, second_weight: f64) -> u8 {
    (f64::from(first) * first_weight + f64::from(second) * second_weight)
        .round()
        .clamp(0.0, 255.0) as u8
}

fn format_static_less_blended_color(
    first: StaticSrgbColorWithAlpha,
    second: StaticSrgbColorWithAlpha,
    blend_channel: impl Fn(f64, f64) -> f64,
) -> String {
    let backdrop_alpha = first.alpha.unwrap_or(1.0);
    let source_alpha = second.alpha.unwrap_or(1.0);
    let alpha = source_alpha + backdrop_alpha * (1.0 - source_alpha);
    format_static_less_color_with_alpha(
        StaticSrgbColorWithAlpha {
            color: SrgbColor {
                red: static_less_blend_result_channel(
                    first.color.red,
                    second.color.red,
                    backdrop_alpha,
                    source_alpha,
                    alpha,
                    &blend_channel,
                ),
                green: static_less_blend_result_channel(
                    first.color.green,
                    second.color.green,
                    backdrop_alpha,
                    source_alpha,
                    alpha,
                    &blend_channel,
                ),
                blue: static_less_blend_result_channel(
                    first.color.blue,
                    second.color.blue,
                    backdrop_alpha,
                    source_alpha,
                    alpha,
                    &blend_channel,
                ),
            },
            alpha: None,
        },
        alpha,
    )
}

fn static_less_blend_result_channel(
    backdrop: u8,
    source: u8,
    backdrop_alpha: f64,
    source_alpha: f64,
    alpha: f64,
    blend_channel: &impl Fn(f64, f64) -> f64,
) -> u8 {
    let backdrop = f64::from(backdrop) / 255.0;
    let source = f64::from(source) / 255.0;
    let blended = blend_channel(backdrop, source);
    let result = if alpha > 0.0 {
        (source_alpha * source
            + backdrop_alpha * (backdrop - source_alpha * (backdrop + source - blended)))
            / alpha
    } else {
        blended
    };
    static_less_blend_channel(result * 255.0)
}

fn static_less_multiply_value(backdrop: f64, source: f64) -> f64 {
    backdrop * source
}

fn static_less_screen_value(backdrop: f64, source: f64) -> f64 {
    backdrop + source - backdrop * source
}

fn static_less_overlay_value(backdrop: f64, source: f64) -> f64 {
    if backdrop * 2.0 <= 1.0 {
        return static_less_multiply_value(backdrop * 2.0, source);
    }
    static_less_screen_value(backdrop * 2.0 - 1.0, source)
}

fn static_less_softlight_value(backdrop: f64, source: f64) -> f64 {
    let mut distance = 1.0;
    let mut factor = backdrop;
    if source > 0.5 {
        factor = 1.0;
        distance = if backdrop > 0.25 {
            backdrop.sqrt()
        } else {
            ((16.0 * backdrop - 12.0) * backdrop + 4.0) * backdrop
        };
    }
    backdrop - (1.0 - 2.0 * source) * factor * (distance - backdrop)
}

fn static_less_exclusion_value(backdrop: f64, source: f64) -> f64 {
    backdrop + source - 2.0 * backdrop * source
}

fn static_less_average_value(backdrop: f64, source: f64) -> f64 {
    (backdrop + source) / 2.0
}

fn static_less_negation_value(backdrop: f64, source: f64) -> f64 {
    1.0 - (backdrop + source - 1.0).abs()
}

fn static_less_blend_channel(value: f64) -> u8 {
    value.round().clamp(0.0, 255.0) as u8
}

fn parse_static_less_color_argument(value: &str) -> Option<StaticSrgbColorWithAlpha> {
    parse_static_srgb_color_with_alpha(value)
        .or_else(|| parse_static_rgb_function_color_with_alpha(value))
        .or_else(|| parse_static_hsl_function_color_with_alpha(value))
        .or_else(|| parse_static_hwb_function_color_with_alpha(value))
        .or_else(|| {
            parse_color_function_value(value)
                .or_else(|| parse_color_mix_value(value))
                .or_else(|| parse_oklab_oklch_value(value))
                .and_then(|value| parse_static_less_color_argument(value.as_str()))
        })
}

fn static_less_opaque_srgb_color(red: u8, green: u8, blue: u8) -> StaticSrgbColorWithAlpha {
    StaticSrgbColorWithAlpha {
        color: SrgbColor { red, green, blue },
        alpha: None,
    }
}

fn parse_static_less_hsv_channel_value(
    value: &str,
    function_name: &str,
    channel: impl FnOnce(StaticLessHsvChannels) -> f64,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [color] = arguments.as_slice() else {
        return None;
    };
    let color = parse_static_less_color_argument(color.trim())?;
    Some(format_static_less_channel_number(channel(
        static_less_hsv_channels(color),
    )))
}

fn parse_static_less_luma_or_luminance_value(
    value: &str,
    function_name: &str,
    channel: impl FnOnce(SrgbColor) -> f64,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [color] = arguments.as_slice() else {
        return None;
    };
    let color = parse_static_less_color_argument(color.trim())?;
    let alpha = color.alpha.unwrap_or(1.0);
    Some(format!(
        "{}%",
        format_static_less_channel_number(channel(color.color) * alpha * 100.0)
    ))
}

fn static_less_luma(color: SrgbColor) -> f64 {
    0.2126 * static_less_linear_rgb_channel(color.red)
        + 0.7152 * static_less_linear_rgb_channel(color.green)
        + 0.0722 * static_less_linear_rgb_channel(color.blue)
}

fn static_less_luminance(color: SrgbColor) -> f64 {
    (0.2126 * f64::from(color.red)
        + 0.7152 * f64::from(color.green)
        + 0.0722 * f64::from(color.blue))
        / 255.0
}

fn static_less_linear_rgb_channel(channel: u8) -> f64 {
    let channel = f64::from(channel) / 255.0;
    if channel <= 0.03928 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

fn parse_static_less_round_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "round")?;
    let (number, decimal_places) = match arguments.as_slice() {
        [number] => (number.as_str(), 0usize),
        [number, decimal_places] => {
            let decimal_places = parse_static_less_unitless_integer(decimal_places.trim())?;
            (number.as_str(), decimal_places)
        }
        _ => return None,
    };
    let parsed = parse_numeric_value_with_unit(number.trim())?;
    let factor = 10_f64.powi(i32::try_from(decimal_places).ok()?);
    let rounded = (parsed.value * factor).round() / factor;
    Some(format!(
        "{}{}",
        format_static_less_number(rounded),
        parsed.unit
    ))
}

fn parse_static_less_pi_value(value: &str) -> Option<String> {
    value
        .trim()
        .eq_ignore_ascii_case("pi()")
        .then(|| format_static_less_math_number(std::f64::consts::PI))
        .flatten()
}

fn parse_static_less_sin_value(value: &str) -> Option<String> {
    parse_static_less_trig_value(value, "sin", f64::sin)
}

fn parse_static_less_cos_value(value: &str) -> Option<String> {
    parse_static_less_trig_value(value, "cos", f64::cos)
}

fn parse_static_less_tan_value(value: &str) -> Option<String> {
    parse_static_less_trig_value(value, "tan", f64::tan)
}

fn parse_static_less_asin_value(value: &str) -> Option<String> {
    parse_static_less_inverse_trig_value(value, "asin", f64::asin, true)
}

fn parse_static_less_acos_value(value: &str) -> Option<String> {
    parse_static_less_inverse_trig_value(value, "acos", f64::acos, true)
}

fn parse_static_less_atan_value(value: &str) -> Option<String> {
    parse_static_less_inverse_trig_value(value, "atan", f64::atan, false)
}

fn parse_static_less_trig_value(
    value: &str,
    function_name: &str,
    evaluate: fn(f64) -> f64,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [angle] = arguments.as_slice() else {
        return None;
    };
    format_static_less_math_number(evaluate(parse_static_less_angle_radians(angle.trim())?))
}

fn parse_static_less_inverse_trig_value(
    value: &str,
    function_name: &str,
    evaluate: fn(f64) -> f64,
    requires_unit_interval: bool,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [number] = arguments.as_slice() else {
        return None;
    };
    let parsed = parse_numeric_value_with_unit(number.trim())?;
    if !parsed.unit.is_empty() {
        return None;
    }
    if requires_unit_interval && !(-1.0..=1.0).contains(&parsed.value) {
        return None;
    }
    let radians = evaluate(parsed.value);
    if !radians.is_finite() {
        return None;
    }
    Some(format!("{}rad", format_static_less_math_number(radians)?))
}

fn parse_static_less_angle_radians(value: &str) -> Option<f64> {
    let parsed = parse_numeric_value_with_unit(value)?;
    if !parsed.value.is_finite() {
        return None;
    }
    match parsed.unit {
        "" | "rad" => Some(parsed.value),
        "deg" => Some(parsed.value.to_radians()),
        "grad" => Some(parsed.value * std::f64::consts::PI / 200.0),
        "turn" => Some(parsed.value * std::f64::consts::TAU),
        _ => None,
    }
}

fn format_static_less_math_number(value: f64) -> Option<String> {
    value
        .is_finite()
        .then(|| format_static_less_channel_number(if value.abs() < 1e-10 { 0.0 } else { value }))
}

fn format_static_less_number(value: f64) -> String {
    let formatted = format_css_number(value);
    if let Some(suffix) = formatted.strip_prefix('.') {
        return format!("0.{suffix}");
    }
    if let Some(suffix) = formatted.strip_prefix("-.") {
        return format!("-0.{suffix}");
    }
    formatted
}

fn format_static_less_channel_number(value: f64) -> String {
    let formatted = if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        format!("{value:.8}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    };
    if let Some(suffix) = formatted.strip_prefix('.') {
        return format!("0.{suffix}");
    }
    if let Some(suffix) = formatted.strip_prefix("-.") {
        return format!("-0.{suffix}");
    }
    formatted
}

fn static_less_alpha_byte(alpha: f64) -> u8 {
    (alpha.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn format_static_less_color_with_alpha(color: StaticSrgbColorWithAlpha, alpha: f64) -> String {
    if (alpha - 1.0).abs() < f64::EPSILON {
        return format!(
            "#{:02x}{:02x}{:02x}",
            color.color.red, color.color.green, color.color.blue
        );
    }
    format!(
        "rgba({}, {}, {}, {})",
        color.color.red,
        color.color.green,
        color.color.blue,
        format_static_less_channel_number(alpha)
    )
}

fn parse_static_less_isnumber_value(value: &str) -> Option<String> {
    parse_static_less_unary_predicate_value(value, "isnumber", static_less_guard_value_is_number)
}

fn parse_static_less_iscolor_value(value: &str) -> Option<String> {
    parse_static_less_unary_predicate_value(value, "iscolor", static_less_guard_value_is_color)
}

fn parse_static_less_isstring_value(value: &str) -> Option<String> {
    parse_static_less_unary_predicate_value(value, "isstring", static_less_guard_value_is_string)
}

fn parse_static_less_iskeyword_value(value: &str) -> Option<String> {
    parse_static_less_unary_predicate_value(value, "iskeyword", static_less_guard_value_is_keyword)
}

fn parse_static_less_isurl_value(value: &str) -> Option<String> {
    parse_static_less_unary_predicate_value(value, "isurl", static_less_guard_value_is_url)
}

fn parse_static_less_isdefined_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "isdefined")?;
    let [value] = arguments.as_slice() else {
        return None;
    };
    let value = value.trim();
    (!value.starts_with('@') && !value.starts_with('$')).then_some(true.to_string())
}

fn parse_static_less_isdefined_value_with_context(
    value: &str,
    scope_id: usize,
    reference_position: usize,
    scopes: &[StaticStylesheetScope],
    variable_declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    property_declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "isdefined")?;
    let [value] = arguments.as_slice() else {
        return None;
    };
    static_less_isdefined_argument_matches(
        value,
        scope_id,
        reference_position,
        scopes,
        variable_declarations,
        property_declarations,
        detached_ruleset_declarations,
    )
    .map(|defined| defined.to_string())
}

fn static_less_isdefined_argument_matches(
    value: &str,
    scope_id: usize,
    reference_position: usize,
    scopes: &[StaticStylesheetScope],
    variable_declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    property_declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
) -> Option<bool> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if value.starts_with('$') {
        if !value
            .strip_prefix('$')
            .is_some_and(static_stylesheet_property_name_is_safe)
        {
            return None;
        }
        return Some(
            find_static_less_property_declaration_before(
                value,
                scope_id,
                scopes,
                property_declarations,
                reference_position,
            )
            .is_some(),
        );
    }
    if !value.starts_with('@') {
        return Some(true);
    }
    if value.starts_with("@@") || !static_less_variable_name_is_safe(value) {
        return None;
    }
    Some(
        find_static_less_variable_declaration(value, scope_id, scopes, variable_declarations)
            .is_some()
            || find_static_less_detached_ruleset_declaration(
                value,
                scope_id,
                scopes,
                detached_ruleset_declarations,
            )
            .is_some(),
    )
}

fn parse_static_less_isruleset_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "isruleset")?;
    let [value] = arguments.as_slice() else {
        return None;
    };
    let value = value.trim();
    (!value.starts_with('@') && static_stylesheet_literal_value_is_safe(value))
        .then(|| false.to_string())
}

fn parse_static_less_isruleset_value_with_context(
    value: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "isruleset")?;
    let [value] = arguments.as_slice() else {
        return None;
    };
    let value = value.trim();
    if static_less_value_is_detached_ruleset_reference(
        value,
        scope_id,
        scopes,
        detached_ruleset_declarations,
    ) {
        return Some(true.to_string());
    }
    (!value.starts_with('@') && static_stylesheet_literal_value_is_safe(value))
        .then(|| false.to_string())
}

fn parse_static_less_ispixel_value(value: &str) -> Option<String> {
    parse_static_less_unary_predicate_value(value, "ispixel", |value| {
        static_less_guard_value_has_unit(value, "px")
    })
}

fn parse_static_less_ispercentage_value(value: &str) -> Option<String> {
    parse_static_less_unary_predicate_value(value, "ispercentage", |value| {
        static_less_guard_value_has_unit(value, "%")
    })
}

fn parse_static_less_isem_value(value: &str) -> Option<String> {
    parse_static_less_unary_predicate_value(value, "isem", |value| {
        static_less_guard_value_has_unit(value, "em")
    })
}

fn parse_static_less_isunit_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "isunit")?;
    let [value, unit] = arguments.as_slice() else {
        return None;
    };
    let unit = static_less_guard_unit_text(unit.trim())?;
    Some(static_less_guard_value_has_unit(value.trim(), unit).to_string())
}

fn parse_static_less_unary_predicate_value(
    value: &str,
    function_name: &str,
    predicate: impl FnOnce(&str) -> bool,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [value] = arguments.as_slice() else {
        return None;
    };
    Some(predicate(value.trim()).to_string())
}

fn parse_static_less_length_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "length")?;
    let items = static_less_list_items_from_arguments(arguments.as_slice())?;
    Some(items.len().to_string())
}

fn parse_static_less_extract_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "extract")?;
    if arguments.len() < 2 {
        return None;
    }
    let (index, list_arguments) = arguments.split_last()?;
    let index = parse_static_less_unitless_integer(index.trim())?;
    let items = static_less_list_items_from_arguments(list_arguments)?;
    items.get(index.checked_sub(1)?).cloned()
}

fn parse_static_less_range_value(value: &str) -> Option<String> {
    const MAX_STATIC_LESS_RANGE_ITEMS: usize = 1024;

    let arguments = parse_whole_function_value_arguments(value, "range")?;
    let (start, end, step) = match arguments.as_slice() {
        [end] => {
            let end = parse_numeric_value_with_unit(end.trim())?;
            let start = StaticLessRangeEndpoint {
                value: 1.0,
                unit: end.unit,
            };
            let step = StaticLessRangeEndpoint {
                value: 1.0,
                unit: "",
            };
            (start, static_less_range_endpoint_from_numeric(end)?, step)
        }
        [start, end] => {
            let start = static_less_range_endpoint(start.trim())?;
            let end = static_less_range_endpoint(end.trim())?;
            let step = StaticLessRangeEndpoint {
                value: 1.0,
                unit: "",
            };
            (start, end, step)
        }
        [start, end, step] => (
            static_less_range_endpoint(start.trim())?,
            static_less_range_endpoint(end.trim())?,
            static_less_range_endpoint(step.trim())?,
        ),
        _ => return None,
    };

    if step.value <= 0.0 {
        return None;
    }
    if start.value > end.value {
        return Some(String::new());
    }

    let mut items = Vec::new();
    let mut current = start.value;
    while current <= end.value + f64::EPSILON {
        if items.len() >= MAX_STATIC_LESS_RANGE_ITEMS {
            return None;
        }
        items.push(format!(
            "{}{}",
            format_static_less_number(current),
            end.unit
        ));
        current += step.value;
    }
    Some(items.join(" "))
}

#[derive(Debug, Clone, Copy)]
struct StaticLessRangeEndpoint<'a> {
    value: f64,
    unit: &'a str,
}

fn static_less_range_endpoint(value: &str) -> Option<StaticLessRangeEndpoint<'_>> {
    static_less_range_endpoint_from_numeric(parse_numeric_value_with_unit(value)?)
}

fn static_less_range_endpoint_from_numeric(
    parsed: NumericValueV0<'_>,
) -> Option<StaticLessRangeEndpoint<'_>> {
    parsed.value.is_finite().then_some(StaticLessRangeEndpoint {
        value: parsed.value,
        unit: parsed.unit,
    })
}

fn parse_static_less_replace_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "replace")?;
    let (input, pattern, replacement, flags) = match arguments.as_slice() {
        [input, pattern, replacement] => {
            (input.as_str(), pattern.as_str(), replacement.as_str(), None)
        }
        [input, pattern, replacement, flags] => (
            input.as_str(),
            pattern.as_str(),
            replacement.as_str(),
            Some(flags.as_str()),
        ),
        _ => return None,
    };
    let input = static_less_string_argument(input.trim())?;
    let pattern = static_less_string_argument(pattern.trim())?.text;
    let replacement = static_less_string_argument(replacement.trim())?.text;
    if !static_less_replace_pattern_is_literal(pattern.as_str())
        || replacement.contains('$')
        || replacement
            .chars()
            .any(|ch| matches!(ch, '\n' | '\r' | '\u{000c}'))
    {
        return None;
    }
    let flags = flags
        .map(|flags| static_less_replace_flags(flags.trim()))
        .unwrap_or(Some(StaticLessReplaceFlags {
            global: false,
            case_insensitive: false,
        }))?;
    if flags.case_insensitive
        && (!input.text.is_ascii() || !pattern.is_ascii() || !replacement.is_ascii())
    {
        return None;
    }
    if pattern.is_empty() && flags.global {
        return None;
    }

    let output = static_less_replace_literal(
        input.text.as_str(),
        pattern.as_str(),
        replacement.as_str(),
        flags,
    )?;
    input.render(output.as_str())
}

fn parse_static_less_format_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "%")?;
    let [format, replacements @ ..] = arguments.as_slice() else {
        return None;
    };
    let format = static_less_string_argument(format.trim())?;
    let mut replacement_index = 0usize;
    let mut output = String::new();
    let mut chars = format.text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '%' {
            output.push(ch);
            continue;
        }

        let Some(specifier) = chars.next() else {
            output.push('%');
            break;
        };
        if specifier == '%' {
            output.push('%');
            continue;
        }
        if !matches!(specifier, 's' | 'S' | 'd' | 'D' | 'a' | 'A') {
            return None;
        }

        let Some(replacement) = replacements.get(replacement_index) else {
            output.push('%');
            output.push(specifier);
            continue;
        };
        replacement_index += 1;
        let replacement = static_less_format_argument_text(replacement.trim())?;
        if specifier.is_ascii_uppercase() {
            output.push_str(percent_encode_static_less_escape_value(replacement.as_str()).as_str());
        } else {
            output.push_str(replacement.as_str());
        }
    }

    format.render(output.as_str())
}

fn static_less_format_argument_text(value: &str) -> Option<String> {
    if let Some(argument) = static_less_string_argument(value) {
        return Some(argument.text);
    }
    (!value.is_empty()
        && !value
            .chars()
            .any(|ch| matches!(ch, '\n' | '\r' | '\u{000c}')))
    .then(|| value.to_string())
}

#[derive(Debug, Clone)]
struct StaticLessStringArgument {
    text: String,
    quote: Option<char>,
    escaped: bool,
}

impl StaticLessStringArgument {
    fn render(&self, output: &str) -> Option<String> {
        if self.escaped || self.quote.is_none() {
            return static_less_unquoted_string_argument_is_safe(output)
                .then(|| output.to_string());
        }
        let quote = self.quote?;
        if output
            .chars()
            .any(|ch| ch == quote || matches!(ch, '\\' | '\n' | '\r' | '\u{000c}'))
        {
            return None;
        }
        Some(format!("{quote}{output}{quote}"))
    }
}

#[derive(Debug, Clone, Copy)]
struct StaticLessReplaceFlags {
    global: bool,
    case_insensitive: bool,
}

fn static_less_replace_flags(value: &str) -> Option<StaticLessReplaceFlags> {
    let text = static_less_string_argument(value)?.text;
    let mut flags = StaticLessReplaceFlags {
        global: false,
        case_insensitive: false,
    };
    for ch in text.chars() {
        match ch {
            'g' if !flags.global => flags.global = true,
            'i' if !flags.case_insensitive => flags.case_insensitive = true,
            _ => return None,
        }
    }
    Some(flags)
}

fn static_less_replace_literal(
    input: &str,
    pattern: &str,
    replacement: &str,
    flags: StaticLessReplaceFlags,
) -> Option<String> {
    if pattern.is_empty() {
        return Some(format!("{replacement}{input}"));
    }
    if !flags.case_insensitive {
        return if flags.global {
            Some(input.replace(pattern, replacement))
        } else {
            Some(input.replacen(pattern, replacement, 1))
        };
    }

    let mut output = String::new();
    let mut cursor = 0usize;
    let mut replaced = false;
    while cursor <= input.len() {
        let Some(relative) = static_less_ascii_case_insensitive_find(&input[cursor..], pattern)
        else {
            break;
        };
        let start = cursor + relative;
        let end = start + pattern.len();
        output.push_str(&input[cursor..start]);
        output.push_str(replacement);
        cursor = end;
        replaced = true;
        if !flags.global {
            break;
        }
    }
    if !replaced {
        return Some(input.to_string());
    }
    output.push_str(&input[cursor..]);
    Some(output)
}

fn static_less_ascii_case_insensitive_find(input: &str, pattern: &str) -> Option<usize> {
    input
        .as_bytes()
        .windows(pattern.len())
        .position(|window| window.eq_ignore_ascii_case(pattern.as_bytes()))
}

fn static_less_replace_pattern_is_literal(pattern: &str) -> bool {
    pattern.chars().all(|ch| {
        !matches!(
            ch,
            '\\' | '^' | '$' | '.' | '|' | '?' | '*' | '+' | '(' | ')' | '[' | ']' | '{' | '}'
        ) && !matches!(ch, '\n' | '\r' | '\u{000c}')
    })
}

fn static_less_string_argument(value: &str) -> Option<StaticLessStringArgument> {
    if let Some(rest) = value.trim().strip_prefix('~') {
        let (quote, text) = static_less_quoted_string(rest)?;
        return Some(StaticLessStringArgument {
            text,
            quote: Some(quote),
            escaped: true,
        });
    }
    if let Some((quote, text)) = static_less_quoted_string(value) {
        return Some(StaticLessStringArgument {
            text,
            quote: Some(quote),
            escaped: false,
        });
    }
    static_less_unquoted_string_argument_is_safe(value).then(|| StaticLessStringArgument {
        text: value.to_string(),
        quote: None,
        escaped: false,
    })
}

fn static_less_unquoted_string_argument_is_safe(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
}

fn parse_static_less_escape_value(value: &str) -> Option<String> {
    let argument = parse_whole_function_value_inner(value, "e")?.trim();
    static_less_quoted_string_contents(argument).or_else(|| {
        (!argument.is_empty()
            && argument
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')))
        .then(|| argument.to_string())
    })
}

fn parse_static_less_url_escape_value(value: &str) -> Option<String> {
    let argument = parse_whole_function_value_inner(value, "escape")?.trim();
    let text = static_less_quoted_string_contents(argument).unwrap_or_else(|| argument.to_string());
    Some(percent_encode_static_less_escape_value(text.as_str()))
}

fn percent_encode_static_less_escape_value(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut output = String::new();
    for byte in value.bytes() {
        if static_less_escape_byte_is_safe(byte) {
            output.push(char::from(byte));
        } else {
            output.push('%');
            output.push(char::from(HEX[usize::from(byte >> 4)]));
            output.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
    }
    output
}

fn static_less_escape_byte_is_safe(byte: u8) -> bool {
    matches!(
        byte,
        b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'_'
            | b'.'
            | b'!'
            | b'~'
            | b'*'
            | b'\''
            | b'/'
            | b'?'
            | b'&'
            | b'@'
            | b'+'
            | b','
            | b'$'
    )
}

fn static_less_list_items_from_arguments(arguments: &[String]) -> Option<Vec<String>> {
    if arguments.len() == 1 {
        return split_top_level_whitespace_value_components_owned(arguments[0].as_str())
            .filter(|items| !items.is_empty());
    }
    Some(
        arguments
            .iter()
            .map(|argument| argument.trim().to_string())
            .filter(|argument| !argument.is_empty())
            .collect::<Vec<_>>(),
    )
    .filter(|items| !items.is_empty())
}

fn parse_static_less_unitless_integer(value: &str) -> Option<usize> {
    let parsed = parse_numeric_value_with_unit(value)?;
    if !parsed.unit.is_empty() || !parsed.value.is_finite() || parsed.value.fract() != 0.0 {
        return None;
    }
    usize::try_from(parsed.value as i64).ok()
}

fn parse_static_less_alpha_amount(value: &str) -> Option<f64> {
    let parsed = parse_numeric_value_with_unit(value)?;
    if !parsed.value.is_finite() || !matches!(parsed.unit, "" | "%") {
        return None;
    }
    Some((parsed.value / 100.0).clamp(0.0, 1.0))
}

fn parse_static_less_alpha_unit_interval(value: &str) -> Option<f64> {
    let parsed = parse_numeric_value_with_unit(value)?;
    if !parsed.value.is_finite() || !matches!(parsed.unit, "" | "%") {
        return None;
    }
    let value = if parsed.unit == "%" {
        parsed.value / 100.0
    } else {
        parsed.value
    };
    (0.0..=1.0).contains(&value).then_some(value)
}

fn parse_static_less_hsv_unit_interval(value: &str) -> Option<f64> {
    parse_static_less_alpha_unit_interval(value)
}

fn parse_static_less_threshold_number(value: &str) -> Option<f64> {
    let parsed = parse_numeric_value_with_unit(value)?;
    if !parsed.value.is_finite() || !matches!(parsed.unit, "" | "%") {
        return None;
    }
    Some(if parsed.unit == "%" {
        parsed.value / 100.0
    } else {
        parsed.value
    })
}

fn parse_static_less_percentage_points(value: &str) -> Option<f64> {
    let parsed = parse_numeric_value_with_unit(value)?;
    if !parsed.value.is_finite() || !matches!(parsed.unit, "" | "%") {
        return None;
    }
    Some(parsed.value)
}

fn parse_static_less_positive_degrees(value: &str) -> Option<f64> {
    let degrees = parse_static_less_angle_degrees(value)?;
    (degrees >= 0.0).then_some(degrees)
}

fn parse_static_less_angle_degrees(value: &str) -> Option<f64> {
    let parsed = parse_numeric_value_with_unit(value)?;
    if !parsed.value.is_finite() {
        return None;
    }
    match parsed.unit.to_ascii_lowercase().as_str() {
        "" | "deg" => Some(parsed.value),
        "rad" => Some(parsed.value.to_degrees()),
        "grad" => Some(parsed.value * 0.9),
        _ => None,
    }
}

fn parse_static_less_unit_argument(unit: &str) -> Option<&str> {
    if unit == "%" {
        return Some(unit);
    }
    if unit.is_empty()
        || !unit
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return None;
    }
    Some(unit)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticLessConvertibleUnitFamily {
    Length,
    Time,
    Angle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct StaticLessConvertibleUnit {
    family: StaticLessConvertibleUnitFamily,
    unit: &'static str,
    base_factor: f64,
}

fn static_less_convertible_unit(unit: &str) -> Option<StaticLessConvertibleUnit> {
    match unit {
        "px" => Some(static_less_convertible_length_unit("px", 1.0)),
        "in" => Some(static_less_convertible_length_unit("in", 96.0)),
        "cm" => Some(static_less_convertible_length_unit("cm", 96.0 / 2.54)),
        "mm" => Some(static_less_convertible_length_unit("mm", 96.0 / 25.4)),
        "pt" => Some(static_less_convertible_length_unit("pt", 96.0 / 72.0)),
        "pc" => Some(static_less_convertible_length_unit("pc", 16.0)),
        "s" => Some(StaticLessConvertibleUnit {
            family: StaticLessConvertibleUnitFamily::Time,
            unit: "s",
            base_factor: 1.0,
        }),
        "ms" => Some(StaticLessConvertibleUnit {
            family: StaticLessConvertibleUnitFamily::Time,
            unit: "ms",
            base_factor: 0.001,
        }),
        "deg" => Some(static_less_convertible_angle_unit("deg", 1.0)),
        "rad" => Some(static_less_convertible_angle_unit(
            "rad",
            180.0 / std::f64::consts::PI,
        )),
        "grad" => Some(static_less_convertible_angle_unit("grad", 0.9)),
        "turn" => Some(static_less_convertible_angle_unit("turn", 360.0)),
        _ => None,
    }
}

fn static_less_convertible_length_unit(
    unit: &'static str,
    base_factor: f64,
) -> StaticLessConvertibleUnit {
    StaticLessConvertibleUnit {
        family: StaticLessConvertibleUnitFamily::Length,
        unit,
        base_factor,
    }
}

fn static_less_convertible_angle_unit(
    unit: &'static str,
    base_factor: f64,
) -> StaticLessConvertibleUnit {
    StaticLessConvertibleUnit {
        family: StaticLessConvertibleUnitFamily::Angle,
        unit,
        base_factor,
    }
}

fn parse_static_less_convert_unit_argument(unit: &str) -> Option<String> {
    static_less_quoted_string_contents(unit).or_else(|| {
        parse_static_less_unit_argument(unit)
            .map(str::to_string)
            .filter(|unit| unit != "%")
    })
}

fn reduce_static_less_escaped_string_value(value: &str) -> Option<String> {
    let trimmed = value.trim();
    let rest = trimmed.strip_prefix('~')?;
    static_less_quoted_string_contents(rest)
}

fn preserve_static_less_dynamic_escaped_string_value(
    value: &str,
) -> Option<StaticLessResolvedValue> {
    let trimmed = value.trim();
    let rest = trimmed.strip_prefix('~')?;
    let contents = static_less_quoted_string_contents(rest)?;
    contents.contains("@{").then(|| StaticLessResolvedValue {
        text: trimmed.to_string(),
        escaped: true,
    })
}

fn static_less_quoted_string_contents(value: &str) -> Option<String> {
    static_less_quoted_string(value).map(|(_, text)| text)
}

fn static_less_quoted_string(value: &str) -> Option<(char, String)> {
    let rest = value.trim();
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
            return (index + ch.len_utf8() == rest.len()).then_some((quote, output));
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

fn parse_static_less_quoted_hex_color_literal(value: &str) -> Option<String> {
    let text = static_less_quoted_string_contents(value)?;
    let hex = text.strip_prefix('#')?;
    matches!(hex.len(), 3 | 4 | 6 | 8)
        .then_some(hex)?
        .chars()
        .all(|ch| ch.is_ascii_hexdigit())
        .then_some(text)
}

fn static_stylesheet_less_declaration_value_is_removal_safe(value: &str) -> bool {
    if preserve_static_less_dynamic_escaped_string_value(value).is_some() {
        return true;
    }
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
    collect_static_stylesheet_variable_references_with_options(value, variable_kind, false, false)
}

fn collect_static_less_property_variable_references(
    value: &str,
) -> Option<Vec<StaticStylesheetVariableReference>> {
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
        if ch != '$' {
            index += ch.len_utf8();
            continue;
        }

        let name_start = index + ch.len_utf8();
        let name_end = static_stylesheet_variable_name_end(value, name_start);
        if name_end == name_start {
            return None;
        }
        let bare_name = &value[name_start..name_end];
        if !static_stylesheet_property_name_is_safe(bare_name) {
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

fn collect_static_stylesheet_variable_references_with_options(
    value: &str,
    variable_kind: StaticStylesheetVariableKind,
    allow_scss_include_at_keyword: bool,
    allow_less_property_variables: bool,
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
            if allow_less_property_variables && variable_kind == StaticStylesheetVariableKind::Less
            {
                let name_start = index + ch.len_utf8();
                let name_end = static_stylesheet_variable_name_end(value, name_start);
                if name_end == name_start {
                    return None;
                }
                let bare_name = &value[name_start..name_end];
                if !static_stylesheet_property_name_is_safe(bare_name) {
                    return None;
                }
                index = name_end;
                continue;
            }
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
    let Some(previous) = value.get(..start).and_then(|prefix| {
        prefix
            .chars()
            .rev()
            .find(|candidate| !candidate.is_ascii_whitespace())
    }) else {
        return false;
    };
    if !matches!(previous, '(' | ',' | ';') {
        return false;
    }
    if previous == ';' && !static_stylesheet_position_is_inside_parentheses(value, start) {
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

fn static_stylesheet_position_is_inside_parentheses(value: &str, end: usize) -> bool {
    let mut index = 0usize;
    let mut paren_depth = 0usize;
    let mut quote: Option<char> = None;
    while index < end && index < value.len() {
        let Some(ch) = value[index..].chars().next() else {
            break;
        };
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
            ')' => paren_depth = paren_depth.saturating_sub(1),
            _ => {}
        }
        index += ch.len_utf8();
    }
    paren_depth > 0
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
    edits: Vec<StaticStylesheetEvaluationEdit>,
) -> Option<String> {
    let edits = normalize_static_stylesheet_evaluation_edits(source, edits)?;
    Some(apply_normalized_static_stylesheet_evaluation_edits(
        source, &edits,
    ))
}

fn normalize_static_stylesheet_evaluation_edits(
    source: &str,
    mut edits: Vec<StaticStylesheetEvaluationEdit>,
) -> Option<Vec<StaticStylesheetEvaluationEdit>> {
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
    Some(edits)
}

fn apply_normalized_static_stylesheet_evaluation_edits(
    source: &str,
    edits: &[StaticStylesheetEvaluationEdit],
) -> String {
    let mut output = source.to_string();
    for edit in edits.iter().rev() {
        output.replace_range(edit.start..edit.end, edit.replacement.as_str());
    }
    output
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
        assert_eq!(report.fixture_count, 55);
        assert_eq!(report.scss_fixture_count, 10);
        assert_eq!(report.less_fixture_count, 45);
        assert_eq!(report.evaluated_fixture_count, report.fixture_count);
        assert_eq!(report.missing_evaluation_count, 0);
        assert_eq!(report.divergence_count, 0);
        assert!(report.native_replacement_count > 0);
        assert!(report.native_replacement_legacy_reflection_count > 0);
        assert_eq!(
            report.native_replacement_legacy_reflection_count
                + report.native_replacement_legacy_unreflected_count,
            report.native_replacement_count
        );
        assert!(report.native_edit_count > 0);
        assert!(report.native_value_edit_count > 0);
        assert!(report.native_structural_edit_count > 0);
        assert_eq!(
            report.native_value_edit_count + report.native_structural_edit_count,
            report.native_edit_count
        );
        assert_eq!(
            report.native_edit_output_match_count,
            report.evaluated_fixture_count
        );
        assert!(report.native_value_reference_count > 0);
        assert!(report.native_resolved_value_count > 0);
        assert!(report.native_top_value_count > 0);
        assert!(report.all_legacy_declaration_values_preserved);
        assert!(report.all_native_edit_outputs_match_evaluated_css);
        assert!(
            report
                .fixtures
                .iter()
                .any(|fixture| fixture.id == "scss.dynamic-function-return"
                    && fixture.native_top_value_count == 1)
        );
        assert!(
            report
                .fixtures
                .iter()
                .any(|fixture| fixture.id == "scss.recursive-function-return"
                    && fixture.native_top_value_count == 1)
        );
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
        assert_eq!(report.native_replacement_legacy_reflection_count, 1);
        assert_eq!(report.native_replacement_legacy_unreflected_count, 0);
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert_eq!(report.resolved_replacements[0].text, "0px");
        assert_eq!(report.native_edit_count, 2);
        assert_eq!(report.native_value_edit_count, 1);
        assert_eq!(report.native_structural_edit_count, 1);
        assert!(report.native_edit_output_matches_evaluated_css);
        assert!(
            report
                .native_edits
                .iter()
                .any(|edit| edit.edit_kind == "valueReplacement"
                    && edit.replacement == "0px"
                    && edit.abstract_value_kind == Some("exact"))
        );
        assert!(
            report
                .native_edits
                .iter()
                .any(|edit| edit.edit_kind == "structuralRemoval"
                    && edit.replacement.is_empty()
                    && edit.abstract_value.is_none())
        );
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
    fn static_less_evaluation_preserves_dynamic_escaped_string_variable_values_as_raw() {
        let report = derive_static_stylesheet_module_evaluation(
            "@filter: ~\"@{name}\"; .button { filter: @filter; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.resolved_replacements[0].text, "~\"@{name}\"");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "raw");
        assert_eq!(report.value_resolution.raw_count, 1);
        assert_eq!(report.value_resolution.top_count, 0);
        assert!(report.evaluated_css.contains("filter: ~\"@{name}\""));
        assert!(!report.evaluated_css.contains("@filter:"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_reduces_escape_builtin_values_without_reentry() {
        let report = derive_static_stylesheet_module_evaluation(
            "@name: e(\"hello\"); @calc: e(\"calc(1px + 2px)\"); @min: e(\"min(1px, 2px)\"); @sign: e(\"sign(-2px)\"); .button { a: @name; b: @calc; c: @min; d: @sign; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 4);
        assert_eq!(report.value_resolution.raw_count, 4);
        assert_eq!(report.value_resolution.top_count, 0);
        assert!(report.evaluated_css.contains("a: hello"));
        assert!(report.evaluated_css.contains("b: calc(1px + 2px)"));
        assert!(report.evaluated_css.contains("c: min(1px, 2px)"));
        assert!(report.evaluated_css.contains("d: sign(-2px)"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_reduces_url_escape_builtin_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "@query: escape(\"a=1\"); @space: escape(\"hello world\"); @hash: escape(\"#fff\"); @unicode: escape(\"ä\"); @fn: escape(\"min(1px, 2px)\"); .button { a: @query; b: @space; c: @hash; d: @unicode; e: @fn; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 5);
        assert_eq!(report.value_resolution.raw_count, 5);
        assert_eq!(report.value_resolution.top_count, 0);
        assert!(report.evaluated_css.contains("a: a%3D1"));
        assert!(report.evaluated_css.contains("b: hello%20world"));
        assert!(report.evaluated_css.contains("c: %23fff"));
        assert!(report.evaluated_css.contains("d: %C3%A4"));
        assert!(report.evaluated_css.contains("e: min%281px,%202px%29"));
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
    fn static_less_evaluation_preserves_unknown_mixin_accessor_members_as_oracle_report() {
        let report = derive_static_stylesheet_module_evaluation(
            ".tokens(@color) { @result: @color; } .button { color: .tokens(red)[@missing]; }",
            StyleDialect::Less,
        );

        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains(".tokens(red)[@missing]"));
        assert!(report.evaluated_css.contains("@result: @color"));
        assert_eq!(report.replacement_count, 0);
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_preserves_unknown_mixin_accessor_property_members_as_oracle_report() {
        let report = derive_static_stylesheet_module_evaluation(
            ".tokens(@color) { result: @color; } .button { color: .tokens(red)[missing]; }",
            StyleDialect::Less,
        );

        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains(".tokens(red)[missing]"));
        assert!(report.evaluated_css.contains("result: @color"));
        assert_eq!(report.replacement_count, 0);
        assert!(report.oracle.all_legacy_declaration_values_preserved);
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
    fn static_less_evaluation_preserves_false_guarded_namespace_mixin_access_as_oracle_report() {
        let report = derive_static_stylesheet_module_evaluation(
            "#bundle() when (iscolor(1px)) { .tone() { color: red; } } .button { #bundle > .tone(); }",
            StyleDialect::Less,
        );

        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains("#bundle > .tone();"));
        assert!(report.evaluated_css.contains("when (iscolor(1px))"));
        assert!(!report.evaluated_css.contains(".button { color: red"));
        assert_eq!(report.replacement_count, 0);
        assert!(report.oracle.all_legacy_declaration_values_preserved);
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
    fn static_less_evaluation_preserves_false_ruleset_guarded_mixins_as_oracle_report() {
        let report = derive_static_stylesheet_module_evaluation(
            ".apply(@block) when (isruleset(@block)) { @block(); } .button { .apply(red); }",
            StyleDialect::Less,
        );

        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains(".apply(red);"));
        assert!(report.evaluated_css.contains("when (isruleset(@block))"));
        assert_eq!(report.replacement_count, 0);
        assert!(report.oracle.all_legacy_declaration_values_preserved);
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
    fn static_less_evaluation_preserves_unknown_detached_ruleset_accessor_members_as_oracle_report()
    {
        let report = derive_static_stylesheet_module_evaluation(
            "@tokens: { primary: red; }; .button { color: @tokens[missing]; }",
            StyleDialect::Less,
        );

        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains("@tokens[missing]"));
        assert!(report.evaluated_css.contains("@tokens: { primary: red; };"));
        assert_eq!(report.replacement_count, 0);
        assert!(report.oracle.all_legacy_declaration_values_preserved);
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
    fn static_less_evaluation_preserves_unknown_detached_ruleset_mixin_calls_as_oracle_report() {
        let report = derive_static_stylesheet_module_evaluation(
            "@rules: { .unknown(); }; .button { @rules(); }",
            StyleDialect::Less,
        );

        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains("@rules: { .unknown(); };"));
        assert!(report.evaluated_css.contains("@rules();"));
        assert_eq!(report.replacement_count, 0);
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_preserves_unbound_parameterized_namespace_mixin_access_as_oracle_report()
     {
        let report = derive_static_stylesheet_module_evaluation(
            "#bundle(@color) { .tone() { color: @color; } } .button { #bundle > .tone(); }",
            StyleDialect::Less,
        );

        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains("#bundle > .tone();"));
        assert!(report.evaluated_css.contains("#bundle(@color)"));
        assert!(!report.evaluated_css.contains(".button { color:"));
        assert_eq!(report.replacement_count, 0);
        assert!(report.oracle.all_legacy_declaration_values_preserved);
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
    fn static_less_evaluation_expands_named_mixin_arguments() {
        let report = derive_static_stylesheet_module_evaluation(
            ".tone(@color, @gap: 1px) { color: @color; margin: @gap; } .button { .tone(@gap: 2px, @color: blue); }",
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
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_expands_semicolon_named_mixin_arguments() {
        let report = derive_static_stylesheet_module_evaluation(
            ".tone(@color; @gap: 1px) { color: @color; margin: @gap; } .button { .tone(@gap: 2px; @color: blue); }",
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
    fn static_less_evaluation_preserves_unmatched_literal_pattern_mixins_as_oracle_report() {
        let report = derive_static_stylesheet_module_evaluation(
            ".tone(dark, @color) { color: @color; background: black; } .button { .tone(light, red); }",
            StyleDialect::Less,
        );

        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains(".tone(light, red);"));
        assert!(report.evaluated_css.contains(".tone(dark, @color)"));
        assert!(!report.evaluated_css.contains(".button { color: red"));
        assert_eq!(report.replacement_count, 0);
        assert!(report.oracle.all_legacy_declaration_values_preserved);
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
        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.native_replacement_legacy_reflection_count, 0);
        assert_eq!(report.native_replacement_legacy_unreflected_count, 1);
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
    fn static_less_evaluation_preserves_unknown_mixin_call_suffixes_as_oracle_report() {
        let report = derive_static_stylesheet_module_evaluation(
            ".tone(@color) { color: @color; } .button { .tone(red) !default; }",
            StyleDialect::Less,
        );

        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains(".tone(red) !default;"));
        assert!(report.evaluated_css.contains(".tone(@color)"));
        assert!(!report.evaluated_css.contains(".button { color: red"));
        assert_eq!(report.replacement_count, 0);
        assert!(report.oracle.all_legacy_declaration_values_preserved);
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
    fn static_less_evaluation_preserves_recursive_nested_mixin_calls_as_oracle_report() {
        let source = ".again() { .again(); } .button { .again(); }";
        let report = derive_static_stylesheet_module_evaluation(source, StyleDialect::Less);

        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.evaluated_css, source);
        assert_eq!(report.replacement_count, 0);
        assert!(report.oracle.all_legacy_declaration_values_preserved);
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
        assert!(report.evaluated_css.contains("color: #800080"));
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
.present() when (isdefined(@brand)) { color: @brand; }
.with-param(@tone) when (isdefined(@tone)) { border-color: @tone; }
@brand: red;
.button { .space(2px); .ratio(50%); .font("Roboto"); .display(block); .asset(url("./icon.svg")); .unit(1rem); .present(); .with-param(green); }"#,
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
        assert!(report.evaluated_css.contains("color: red"));
        assert!(report.evaluated_css.contains("border-color: green"));
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
    fn static_less_evaluation_expands_property_isdefined_guarded_mixin_calls() {
        let report = derive_static_stylesheet_module_evaluation(
            ".present() when (isdefined($color)) { border-color: $color; } .button { color: red; .present(); }",
            StyleDialect::Less,
        );

        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains(".present()"));
        assert!(report.evaluated_css.contains("color: red"));
        assert!(report.evaluated_css.contains("border-color: red"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_expands_property_predicate_guarded_mixin_calls() {
        let report = derive_static_stylesheet_module_evaluation(
            ".space() when (isnumber($margin)) { padding: $margin; } .tone() when (iscolor($color)) { border-color: $color; } .unit() when (isunit($gap, px)) { inset: $gap; } .button { margin: (1px + 2px); color: red; gap: 4px; .space(); .tone(); .unit(); }",
            StyleDialect::Less,
        );

        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains(".space()"));
        assert!(!report.evaluated_css.contains(".tone()"));
        assert!(!report.evaluated_css.contains(".unit()"));
        assert!(report.evaluated_css.contains("padding: 3px"));
        assert!(report.evaluated_css.contains("border-color: red"));
        assert!(report.evaluated_css.contains("inset: 4px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_expands_property_comparison_guarded_mixin_calls() {
        let report = derive_static_stylesheet_module_evaluation(
            ".space() when ($margin > 1px) { padding: $margin; } .button { margin: 2px; .space(); }",
            StyleDialect::Less,
        );

        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(!report.evaluated_css.contains(".space()"));
        assert!(report.evaluated_css.contains("padding: 2px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_preserves_future_property_guarded_mixins_as_oracle_report() {
        let report = derive_static_stylesheet_module_evaluation(
            ".space() when (isnumber($margin)) { padding: $margin; } .button { .space(); margin: 2px; }",
            StyleDialect::Less,
        );

        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 0);
        assert!(
            report
                .evaluated_css
                .contains(".space() when (isnumber($margin))")
        );
        assert!(
            report
                .evaluated_css
                .contains(".button { .space(); margin: 2px; }")
        );
        assert!(!report.evaluated_css.contains("padding: 2px"));
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
    fn static_less_evaluation_preserves_false_guarded_mixins_as_oracle_report() {
        let report = derive_static_stylesheet_module_evaluation(
            ".tone(@value) when (iscolor(@value)) { color: @value; } .button { .tone(1px); }",
            StyleDialect::Less,
        );

        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains(".tone(1px)"));
        assert!(!report.evaluated_css.contains("color: 1px"));
        assert_eq!(report.replacement_count, 0);
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_preserves_false_comparison_guarded_mixins_as_oracle_report() {
        let report = derive_static_stylesheet_module_evaluation(
            ".space(@gap) when (@gap > 2px) { margin: @gap; } .button { .space(1px); }",
            StyleDialect::Less,
        );

        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains(".space(1px)"));
        assert!(!report.evaluated_css.contains("margin: 1px"));
        assert_eq!(report.replacement_count, 0);
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_preserves_false_unit_guarded_mixins_as_oracle_report() {
        let report = derive_static_stylesheet_module_evaluation(
            ".space(@gap) when (ispixel(@gap)) { margin: @gap; } .button { .space(2em); }",
            StyleDialect::Less,
        );

        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains(".space(2em)"));
        assert!(!report.evaluated_css.contains("margin: 2em"));
        assert_eq!(report.replacement_count, 0);
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_preserves_false_isunit_guarded_mixins_as_oracle_report() {
        let report = derive_static_stylesheet_module_evaluation(
            r#".space(@gap) when (isunit(@gap, "px")) { margin: @gap; } .button { .space(2em); }"#,
            StyleDialect::Less,
        );

        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains(".space(2em)"));
        assert!(!report.evaluated_css.contains("margin: 2em"));
        assert_eq!(report.replacement_count, 0);
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_preserves_false_isdefined_guarded_mixins_as_oracle_report() {
        let report = derive_static_stylesheet_module_evaluation(
            ".missing() when (isdefined(@missing)) { color: blue; } .button { .missing(); }",
            StyleDialect::Less,
        );

        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains(".missing();"));
        assert!(!report.evaluated_css.contains(".button { color: blue"));
        assert_eq!(report.replacement_count, 0);
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_preserves_false_property_isdefined_guarded_mixins_as_oracle_report() {
        let report = derive_static_stylesheet_module_evaluation(
            ".missing() when (isdefined($missing)) { color: blue; } .button { .missing(); }",
            StyleDialect::Less,
        );

        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert!(report.evaluated_css.contains(".missing();"));
        assert!(!report.evaluated_css.contains(".button { color: blue"));
        assert_eq!(report.replacement_count, 0);
        assert!(report.oracle.all_legacy_declaration_values_preserved);
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
    fn static_less_evaluation_reduces_max_builtin_value() {
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
    fn static_less_evaluation_reduces_unit_builtin_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "@gap: unit(5, px); @plain: unit(5px); @unit-name: get-unit(1.5rem); .button { margin: @gap; padding: @plain; --unit: @unit-name; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 3);
        assert_eq!(report.resolved_replacements[0].text, "5px");
        assert_eq!(report.resolved_replacements[1].text, "5");
        assert_eq!(report.resolved_replacements[2].text, "rem");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert_eq!(report.resolved_replacements[1].abstract_value_kind, "exact");
        assert_eq!(report.resolved_replacements[2].abstract_value_kind, "raw");
        assert_eq!(report.value_resolution.resolved_count, 2);
        assert_eq!(report.value_resolution.raw_count, 1);
        assert!(report.evaluated_css.contains("margin: 5px"));
        assert!(report.evaluated_css.contains("padding: 5"));
        assert!(report.evaluated_css.contains("--unit: rem"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_reduces_convert_builtin_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "@cm: convert(1in, cm); @inch: convert(2.54cm, in); @px: convert(96px, in); @ms: convert(1s, ms); @sec: convert(250ms, s); @deg: convert(1rad, deg); @turn: convert(.5turn, deg); @same: convert(1in, s); .button { cm: @cm; inch: @inch; px: @px; ms: @ms; sec: @sec; deg: @deg; turn: @turn; same: @same; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 8);
        assert_eq!(report.value_resolution.resolved_count, 6);
        assert_eq!(report.value_resolution.raw_count, 2);
        assert!(report.evaluated_css.contains("cm: 2.54cm"));
        assert!(report.evaluated_css.contains("inch: 1in"));
        assert!(report.evaluated_css.contains("px: 1in"));
        assert!(report.evaluated_css.contains("ms: 1000ms"));
        assert!(report.evaluated_css.contains("sec: 0.25s"));
        assert!(report.evaluated_css.contains("deg: 57.29577951deg"));
        assert!(report.evaluated_css.contains("turn: 180deg"));
        assert!(report.evaluated_css.contains("same: 1in"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_reduces_trig_builtin_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "@pi: pi(); @sin: sin(30deg); @sinRad: sin(1rad); @sinUnitless: sin(1); @cos: cos(60deg); @tan: tan(45deg); @asin: asin(.5); @acos: acos(.5); @atan: atan(1); .button { pi: @pi; sin: @sin; sin-rad: @sinRad; sin-unitless: @sinUnitless; cos: @cos; tan: @tan; asin: @asin; acos: @acos; atan: @atan; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 9);
        assert_eq!(report.value_resolution.resolved_count, 9);
        assert_eq!(report.value_resolution.raw_count, 0);
        assert!(report.evaluated_css.contains("pi: 3.14159265"));
        assert!(report.evaluated_css.contains("sin: 0.5"));
        assert!(report.evaluated_css.contains("sin-rad: 0.84147098"));
        assert!(report.evaluated_css.contains("sin-unitless: 0.84147098"));
        assert!(report.evaluated_css.contains("cos: 0.5"));
        assert!(report.evaluated_css.contains("tan: 1"));
        assert!(report.evaluated_css.contains("asin: 0.52359878rad"));
        assert!(report.evaluated_css.contains("acos: 1.04719755rad"));
        assert!(report.evaluated_css.contains("atan: 0.78539816rad"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_reduces_percentage_and_rounding_builtin_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "@ratio: percentage(.5); @ceil: ceil(1.2px); @floor: floor(1.8px); .button { width: @ratio; top: @ceil; bottom: @floor; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 3);
        assert_eq!(report.resolved_replacements[0].text, "50%");
        assert_eq!(report.resolved_replacements[1].text, "2px");
        assert_eq!(report.resolved_replacements[2].text, "1px");
        assert!(
            report
                .resolved_replacements
                .iter()
                .all(|replacement| replacement.abstract_value_kind == "exact")
        );
        assert_eq!(report.value_resolution.resolved_count, 3);
        assert_eq!(report.value_resolution.raw_count, 0);
        assert!(report.evaluated_css.contains("width: 50%"));
        assert!(report.evaluated_css.contains("top: 2px"));
        assert!(report.evaluated_css.contains("bottom: 1px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_reduces_extended_numeric_builtin_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "@sqrt: sqrt(4); @pow: pow(2, 3); @mod: mod(11px, 4px); @min: min(1px, 2px, 3px); @max: max(1px, 2px, 3px); @abs: abs(-2.4px); @round1: round(1.6px); @round2: round(1.234px, 2); .button { sqrt: @sqrt; pow: @pow; mod: @mod; min: @min; max: @max; abs: @abs; round1: @round1; round2: @round2; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 8);
        assert_eq!(report.value_resolution.resolved_count, 8);
        assert_eq!(report.value_resolution.raw_count, 0);
        assert!(report.evaluated_css.contains("sqrt: 2"));
        assert!(report.evaluated_css.contains("pow: 8"));
        assert!(report.evaluated_css.contains("mod: 3px"));
        assert!(report.evaluated_css.contains("min: 1px"));
        assert!(report.evaluated_css.contains("max: 3px"));
        assert!(report.evaluated_css.contains("abs: 2.4px"));
        assert!(report.evaluated_css.contains("round1: 2px"));
        assert!(report.evaluated_css.contains("round2: 1.23px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_preserves_unsupported_css_math_functions() {
        let report = derive_static_stylesheet_module_evaluation(
            "@sign: sign(-2px); @clamp: clamp(1px, 3px, 2px); @rem: rem(11px, 4px); @hypot: hypot(3px, 4px); @exp: exp(1); @log: log(8, 2); @calc: calc(1px + 2px); .button { sign: @sign; clamp: @clamp; rem: @rem; hypot: @hypot; exp: @exp; log: @log; calc: @calc; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 7);
        assert_eq!(report.value_resolution.raw_count, 7);
        assert!(report.evaluated_css.contains("sign: sign(-2px)"));
        assert!(report.evaluated_css.contains("clamp: clamp(1px, 3px, 2px)"));
        assert!(report.evaluated_css.contains("rem: rem(11px, 4px)"));
        assert!(report.evaluated_css.contains("hypot: hypot(3px, 4px)"));
        assert!(report.evaluated_css.contains("exp: exp(1)"));
        assert!(report.evaluated_css.contains("log: log(8, 2)"));
        assert!(report.evaluated_css.contains("calc: calc(1px + 2px)"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_reduces_type_predicate_builtin_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "@number: isnumber(2px); @color: iscolor(red); @string: isstring(\"Roboto\"); @keyword: iskeyword(block); @url: isurl(url(\"a.png\")); @defined: isdefined(@color); @missing: isdefined(@absent); @literal: isdefined(red); @future-defined: isdefined(@future); @future: blue; @px: ispixel(2px); @pct: ispercentage(50%); @em: isem(1em); @unit-ok: isunit(1rem, rem); @unit-bad: isunit(1rem, px); .button { --number: @number; --color: @color; --string: @string; --keyword: @keyword; --url: @url; --defined: @defined; --missing: @missing; --literal: @literal; --future-defined: @future-defined; --px: @px; --pct: @pct; --em: @em; --unit-ok: @unit-ok; --unit-bad: @unit-bad; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 14);
        assert_eq!(report.value_resolution.resolved_count, 0);
        assert_eq!(report.value_resolution.raw_count, 14);
        assert!(
            report
                .resolved_replacements
                .iter()
                .all(|replacement| replacement.abstract_value_kind == "raw")
        );
        assert!(report.evaluated_css.contains("--number: true"));
        assert!(report.evaluated_css.contains("--defined: true"));
        assert!(report.evaluated_css.contains("--missing: false"));
        assert!(report.evaluated_css.contains("--literal: true"));
        assert!(report.evaluated_css.contains("--future-defined: true"));
        assert!(report.evaluated_css.contains("--unit-ok: true"));
        assert!(report.evaluated_css.contains("--unit-bad: false"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_reduces_property_isdefined_builtin_values() {
        let report = derive_static_stylesheet_module_evaluation(
            ".button { color: red; @has-color: isdefined($color); @missing-prop: isdefined($missing); has: @has-color; missing: @missing-prop; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 2);
        assert_eq!(report.value_resolution.raw_count, 2);
        assert_eq!(report.value_resolution.top_count, 0);
        assert!(
            report
                .resolved_replacements
                .iter()
                .all(|replacement| replacement.abstract_value_kind == "raw")
        );
        assert!(report.evaluated_css.contains("color: red"));
        assert!(report.evaluated_css.contains("has: true"));
        assert!(report.evaluated_css.contains("missing: false"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_reduces_isruleset_builtin_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "@rules: { color: red; }; @ok: isruleset(@rules); @bad: isruleset(red); .button { ok: @ok; bad: @bad; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 2);
        assert_eq!(report.value_resolution.raw_count, 2);
        assert_eq!(report.value_resolution.top_count, 0);
        assert!(
            report
                .resolved_replacements
                .iter()
                .all(|replacement| replacement.abstract_value_kind == "raw")
        );
        assert!(report.evaluated_css.contains("ok: true"));
        assert!(report.evaluated_css.contains("bad: false"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_reduces_conditional_builtin_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "@gap: 1; @a: if(@gap > 0, red, blue); @b: if(false, red, blue); @c: if(isnumber(2px), yes, no); @d: boolean(@gap > 0); @e: if(default(), red, blue); .button { a: @a; b: @b; c: @c; d: @d; e: @e; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 5);
        assert_eq!(report.value_resolution.resolved_count, 3);
        assert_eq!(report.value_resolution.raw_count, 2);
        assert!(report.evaluated_css.contains("a: red"));
        assert!(report.evaluated_css.contains("b: blue"));
        assert!(report.evaluated_css.contains("c: yes"));
        assert!(report.evaluated_css.contains("d: true"));
        assert!(report.evaluated_css.contains("e: blue"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_reduces_color_channel_builtin_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "@color: #123456; @r: red(@color); @g: green(@color); @b: blue(@color); @a: alpha(rgba(10, 20, 30, .5)); .button { r: @r; g: @g; b: @b; a: @a; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 4);
        assert_eq!(report.value_resolution.resolved_count, 4);
        assert_eq!(report.value_resolution.raw_count, 0);
        assert!(report.evaluated_css.contains("r: 18"));
        assert!(report.evaluated_css.contains("g: 52"));
        assert!(report.evaluated_css.contains("b: 86"));
        assert!(report.evaluated_css.contains("a: 0.5"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_reduces_rgb_color_constructor_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "@rgb: rgb(18, 52, 86); @rgba: rgba(18, 52, 86, .5); @pct: rgba(100%, 0%, 0%, 50%); @slash: rgb(18 52 86 / .5); .button { color: @rgb; background: @rgba; border-color: @pct; outline-color: @slash; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 4);
        assert_eq!(report.value_resolution.resolved_count, 4);
        assert_eq!(report.value_resolution.raw_count, 0);
        assert!(report.evaluated_css.contains("color: #123456"));
        assert!(
            report
                .evaluated_css
                .contains("background: rgba(18, 52, 86, 0.5)")
        );
        assert!(
            report
                .evaluated_css
                .contains("border-color: rgba(255, 0, 0, 0.5)")
        );
        assert!(
            report
                .evaluated_css
                .contains("outline-color: rgba(18, 52, 86, 0.5)")
        );
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_reduces_color_metadata_builtin_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "@color: #123456; @h: hue(@color); @s: saturation(@color); @l: lightness(@color); @legacy: argb(rgba(18, 52, 86, .5)); .button { h: @h; s: @s; l: @l; legacy: @legacy; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 4);
        assert_eq!(report.value_resolution.resolved_count, 4);
        assert_eq!(report.value_resolution.raw_count, 0);
        assert!(report.evaluated_css.contains("h: 210"));
        assert!(report.evaluated_css.contains("s: 65.38461538%"));
        assert!(report.evaluated_css.contains("l: 20.39215686%"));
        assert!(report.evaluated_css.contains("legacy: #80123456"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_reduces_hsv_color_metadata_builtin_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "@hsv: hsv(210, 60%, 40%); @hsvUnitless: hsv(60, .6, .4); @hsva: hsva(210, 60%, 40%, 50%); @color: #123456; @h: hsvhue(@color); @s: hsvsaturation(@color); @v: hsvvalue(@color); @luma: luma(rgba(18, 52, 86, .5)); @lum: luminance(rgba(18, 52, 86, .5)); .button { hsv: @hsv; hsv-unitless: @hsvUnitless; hsva: @hsva; h: @h; s: @s; v: @v; luma: @luma; luminance: @lum; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 8);
        assert_eq!(report.value_resolution.resolved_count, 8);
        assert_eq!(report.value_resolution.raw_count, 0);
        assert!(report.evaluated_css.contains("hsv: #294766"));
        assert!(report.evaluated_css.contains("hsv-unitless: #666629"));
        assert!(
            report
                .evaluated_css
                .contains("hsva: rgba(41, 71, 102, 0.5)")
        );
        assert!(report.evaluated_css.contains("h: 210"));
        assert!(report.evaluated_css.contains("s: 79.06976744%"));
        assert!(report.evaluated_css.contains("v: 33.7254902%"));
        assert!(report.evaluated_css.contains("luma: 1.62823344%"));
        assert!(report.evaluated_css.contains("luminance: 9.26007843%"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_reduces_contrast_and_color_builtin_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "@dark: contrast(#123456); @light: contrast(#eeeeee); @custom: contrast(#123456, #111111, #eeeeee); @threshold: contrast(#888888, #111111, #eeeeee, 60%); @hex: color(\"#123456\"); @short: color(\"#abc\"); @alpha: color(\"#12345680\"); @kw: color(red); .button { dark: @dark; light: @light; custom: @custom; threshold: @threshold; hex: @hex; short: @short; alpha: @alpha; kw: @kw; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 8);
        assert_eq!(report.value_resolution.resolved_count, 8);
        assert_eq!(report.value_resolution.raw_count, 0);
        assert!(report.evaluated_css.contains("dark: #ffffff"));
        assert!(report.evaluated_css.contains("light: #000000"));
        assert!(report.evaluated_css.contains("custom: #eeeeee"));
        assert!(report.evaluated_css.contains("threshold: #eeeeee"));
        assert!(report.evaluated_css.contains("hex: #123456"));
        assert!(report.evaluated_css.contains("short: #abc"));
        assert!(report.evaluated_css.contains("alpha: #12345680"));
        assert!(report.evaluated_css.contains("kw: #ff0000"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_reduces_alpha_transform_builtin_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "@faded: fade(#123456, 50%); @raised: fadein(rgba(18, 52, 86, .5), 10%); @lowered: fadeout(rgba(18, 52, 86, .5), 10%); @raisedRel: fadein(rgba(18, 52, 86, .5), 10%, relative); @loweredRel: fadeout(rgba(18, 52, 86, .5), 10%, relative); @opaque: fadein(red, 10%); .button { a: @faded; b: @raised; c: @lowered; d: @opaque; e: @raisedRel; f: @loweredRel; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 6);
        assert_eq!(report.value_resolution.resolved_count, 6);
        assert_eq!(report.value_resolution.raw_count, 0);
        assert!(report.evaluated_css.contains("a: rgba(18, 52, 86, 0.5)"));
        assert!(report.evaluated_css.contains("b: rgba(18, 52, 86, 0.6)"));
        assert!(report.evaluated_css.contains("c: rgba(18, 52, 86, 0.4)"));
        assert!(report.evaluated_css.contains("d: #ff0000"));
        assert!(report.evaluated_css.contains("e: rgba(18, 52, 86, 0.55)"));
        assert!(report.evaluated_css.contains("f: rgba(18, 52, 86, 0.45)"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_reduces_hsl_color_transform_builtin_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "@light: lighten(#123456, 10%); @dark: darken(#123456, 10%); @sat: saturate(#123456, 10%); @desat: desaturate(#123456, 10%); @lightRel: lighten(#123456, 10%, relative); @darkRel: darken(#123456, 10%, relative); @satRel: saturate(#123456, 10%, relative); @desatRel: desaturate(#123456, 10%, relative); @spin: spin(#123456, 10); @gray: greyscale(#123456); @alpha: lighten(rgba(18, 52, 86, .5), 10%); .button { light: @light; dark: @dark; sat: @sat; desat: @desat; light-rel: @lightRel; dark-rel: @darkRel; sat-rel: @satRel; desat-rel: @desatRel; spin: @spin; gray: @gray; alpha: @alpha; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 11);
        assert_eq!(report.value_resolution.resolved_count, 11);
        assert_eq!(report.value_resolution.raw_count, 0);
        assert!(report.evaluated_css.contains("light: #1b4d80"));
        assert!(report.evaluated_css.contains("dark: #091a2c"));
        assert!(report.evaluated_css.contains("sat: #0d345b"));
        assert!(report.evaluated_css.contains("desat: #173451"));
        assert!(report.evaluated_css.contains("light-rel: #14395f"));
        assert!(report.evaluated_css.contains("dark-rel: #102f4d"));
        assert!(report.evaluated_css.contains("sat-rel: #0f3459"));
        assert!(report.evaluated_css.contains("desat-rel: #153453"));
        assert!(report.evaluated_css.contains("spin: #122956"));
        assert!(report.evaluated_css.contains("gray: #343434"));
        assert!(
            report
                .evaluated_css
                .contains("alpha: rgba(27, 77, 128, 0.5)")
        );
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_reduces_color_mix_builtin_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "@default: mix(red, blue); @weighted: mix(red, blue, 25%); @tinted: tint(#123456, 10%); @shaded: shade(#123456, 10%); @alpha: mix(rgba(255, 0, 0, .5), blue, 50%); @transparent: mix(transparent, red, 50%); .button { default: @default; weighted: @weighted; tinted: @tinted; shaded: @shaded; alpha: @alpha; transparent: @transparent; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 6);
        assert_eq!(report.value_resolution.resolved_count, 6);
        assert_eq!(report.value_resolution.raw_count, 0);
        assert!(report.evaluated_css.contains("default: #800080"));
        assert!(report.evaluated_css.contains("weighted: #4000bf"));
        assert!(report.evaluated_css.contains("tinted: #2a4867"));
        assert!(report.evaluated_css.contains("shaded: #102f4d"));
        assert!(
            report
                .evaluated_css
                .contains("alpha: rgba(64, 0, 191, 0.75)")
        );
        assert!(
            report
                .evaluated_css
                .contains("transparent: rgba(255, 0, 0, 0.5)")
        );
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_reduces_color_blend_builtin_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "@multiply: multiply(red, blue); @screen: screen(red, blue); @overlay: overlay(#123456, #abcdef); @softlight: softlight(#123456, #abcdef); @hardlight: hardlight(#123456, #abcdef); @difference: difference(#123456, #abcdef); @exclusion: exclusion(#123456, #abcdef); @average: average(#123456, #abcdef); @negation: negation(#123456, #abcdef); .button { multiply: @multiply; screen: @screen; overlay: @overlay; softlight: @softlight; hardlight: @hardlight; difference: @difference; exclusion: @exclusion; average: @average; negation: @negation; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 9);
        assert_eq!(report.value_resolution.resolved_count, 9);
        assert_eq!(report.value_resolution.raw_count, 0);
        assert!(report.evaluated_css.contains("multiply: #000000"));
        assert!(report.evaluated_css.contains("screen: #ff00ff"));
        assert!(report.evaluated_css.contains("overlay: #1854a1"));
        assert!(report.evaluated_css.contains("softlight: #205b8c"));
        assert!(report.evaluated_css.contains("hardlight: #63afea"));
        assert!(report.evaluated_css.contains("difference: #999999"));
        assert!(report.evaluated_css.contains("exclusion: #a5ada4"));
        assert!(report.evaluated_css.contains("average: #5f81a3"));
        assert!(report.evaluated_css.contains("negation: #bdfdb9"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_reduces_alpha_color_blend_builtin_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "@multiply: multiply(rgba(255, 102, 0, .5), #0000ff); @screen: screen(rgba(255, 102, 0, .5), rgba(0, 0, 255, .25)); @overlay: overlay(rgba(255, 102, 0, .5), rgba(0, 0, 255, .25)); @softlight: softlight(rgba(255, 102, 0, .5), rgba(0, 0, 255, .25)); @hardlight: hardlight(rgba(255, 102, 0, .5), rgba(0, 0, 255, .25)); @difference: difference(rgba(255, 102, 0, .5), rgba(0, 0, 255, .25)); @exclusion: exclusion(rgba(255, 102, 0, .5), rgba(0, 0, 255, .25)); @average: average(rgba(255, 102, 0, .5), rgba(0, 0, 255, .25)); @negation: negation(rgba(255, 102, 0, .5), rgba(0, 0, 255, .25)); @both: multiply(transparent, transparent); @transparent: multiply(transparent, #0000ff); @sourceTransparent: screen(#ff6600, transparent); @transparentAverage: average(transparent, #ff6600); .button { multiply: @multiply; screen: @screen; overlay: @overlay; softlight: @softlight; hardlight: @hardlight; difference: @difference; exclusion: @exclusion; average: @average; negation: @negation; both: @both; transparent: @transparent; source-transparent: @sourceTransparent; transparent-average: @transparentAverage; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 13);
        assert_eq!(report.value_resolution.resolved_count, 13);
        assert_eq!(report.value_resolution.raw_count, 0);
        assert!(report.evaluated_css.contains("multiply: #000080"));
        assert!(
            report
                .evaluated_css
                .contains("screen: rgba(204, 82, 102, 0.625)")
        );
        assert!(
            report
                .evaluated_css
                .contains("overlay: rgba(204, 61, 51, 0.625)")
        );
        assert!(
            report
                .evaluated_css
                .contains("softlight: rgba(204, 69, 51, 0.625)")
        );
        assert!(
            report
                .evaluated_css
                .contains("hardlight: rgba(153, 61, 102, 0.625)")
        );
        assert!(
            report
                .evaluated_css
                .contains("difference: rgba(204, 82, 102, 0.625)")
        );
        assert!(
            report
                .evaluated_css
                .contains("exclusion: rgba(204, 82, 102, 0.625)")
        );
        assert!(
            report
                .evaluated_css
                .contains("average: rgba(179, 71, 77, 0.625)")
        );
        assert!(
            report
                .evaluated_css
                .contains("negation: rgba(204, 82, 102, 0.625)")
        );
        assert!(report.evaluated_css.contains("both: rgba(0, 0, 0, 0)"));
        assert!(report.evaluated_css.contains("transparent: #0000ff"));
        assert!(report.evaluated_css.contains("source-transparent: #ff6600"));
        assert!(
            report
                .evaluated_css
                .contains("transparent-average: #ff6600")
        );
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_reduces_list_builtin_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "@items: a b c; @comma: a, b, c; @len1: length(@items); @len2: length(@comma); @x1: extract(@items, 2); @x2: extract(@comma, 3); .button { len1: @len1; len2: @len2; x1: @x1; x2: @x2; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 4);
        assert_eq!(report.value_resolution.resolved_count, 2);
        assert_eq!(report.value_resolution.raw_count, 2);
        assert!(report.evaluated_css.contains("len1: 3"));
        assert!(report.evaluated_css.contains("len2: 3"));
        assert!(report.evaluated_css.contains("x1: b"));
        assert!(report.evaluated_css.contains("x2: c"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_reduces_range_builtin_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "@items: range(4); @gaps: range(1px, 5px, 2); @half: range(1, 2, .5); @empty: range(3, 1); .button { a: @items; b: @gaps; c: @half; d: @empty; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 4);
        assert_eq!(report.value_resolution.raw_count, 3);
        assert!(report.evaluated_css.contains("a: 1 2 3 4"));
        assert!(report.evaluated_css.contains("b: 1px 3px 5px"));
        assert!(report.evaluated_css.contains("c: 1 1.5 2"));
        assert!(report.evaluated_css.contains("d: ;"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_reduces_replace_builtin_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "@name: replace(\"hello world\", \"world\", \"less\"); @first: replace(\"hello\", \"l\", \"L\"); @all: replace(\"hello\", \"l\", \"L\", \"g\"); @fold: replace(\"ABCabc\", \"abc\", \"x\", \"gi\"); @bare: replace(hello, l, X); .button { name: @name; first: @first; all: @all; fold: @fold; bare: @bare; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 5);
        assert_eq!(report.value_resolution.raw_count, 5);
        assert!(report.evaluated_css.contains("name: \"hello less\""));
        assert!(report.evaluated_css.contains("first: \"heLlo\""));
        assert!(report.evaluated_css.contains("all: \"heLLo\""));
        assert!(report.evaluated_css.contains("fold: \"xx\""));
        assert!(report.evaluated_css.contains("bare: heXlo"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_reduces_format_builtin_values() {
        let report = derive_static_stylesheet_module_evaluation(
            "@name: %(\"hello %s\", \"less\"); @num: %(\"%dpx\", 12); @encoded: %(\"%S\", \"x y\"); @literal: %(\"%% done\"); @missing: %(\"%s %s\", alpha); @extra: %(\"%s\", beta, ignored); @escaped: %(~\"hello-%s\", less); .button { name: @name; num: @num; encoded: @encoded; literal: @literal; missing: @missing; extra: @extra; escaped: @escaped; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 7);
        assert_eq!(report.value_resolution.raw_count, 7);
        assert!(report.evaluated_css.contains("name: \"hello less\""));
        assert!(report.evaluated_css.contains("num: \"12px\""));
        assert!(report.evaluated_css.contains("encoded: \"x%20y\""));
        assert!(report.evaluated_css.contains("literal: \"% done\""));
        assert!(report.evaluated_css.contains("missing: \"alpha %s\""));
        assert!(report.evaluated_css.contains("extra: \"beta\""));
        assert!(report.evaluated_css.contains("escaped: hello-less"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_keeps_regex_replace_patterns_raw() {
        let report = derive_static_stylesheet_module_evaluation(
            "@rx: replace(\"abc123\", \"[0-9]+\", \"#\"); .button { rx: @rx; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.value_resolution.raw_count, 1);
        assert!(
            report
                .evaluated_css
                .contains("rx: replace(\"abc123\", \"[0-9]+\", \"#\")")
        );
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_keeps_out_of_range_extract_raw() {
        let report = derive_static_stylesheet_module_evaluation(
            "@items: a b c; @bad: extract(@items, 4); .button { bad: @bad; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.value_resolution.raw_count, 1);
        assert!(report.evaluated_css.contains("bad: extract(a b c, 4)"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
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
    fn static_less_evaluation_reduces_property_variable_alias_values() {
        let report = derive_static_stylesheet_module_evaluation(
            ".button { margin: (1px + 2px); @gap: $margin; padding: @gap; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "@gap");
        assert_eq!(report.resolved_replacements[0].text, "3px");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
        assert_eq!(report.value_resolution.values[0].name, "@gap");
        assert_eq!(
            report.value_resolution.values[0].rendered_value.as_deref(),
            Some("3px")
        );
        assert!(report.evaluated_css.contains("padding: 3px"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_less_evaluation_reduces_property_variable_composite_alias_values() {
        let report = derive_static_stylesheet_module_evaluation(
            ".button { color: red; @outline: 1px solid $color; border: @outline; }",
            StyleDialect::Less,
        );
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 1);
        assert_eq!(report.resolved_replacements[0].name, "@outline");
        assert_eq!(report.resolved_replacements[0].text, "1px solid red");
        assert_eq!(report.resolved_replacements[0].abstract_value_kind, "raw");
        assert_eq!(
            report.value_resolution.values[0].rendered_value.as_deref(),
            Some("1px solid red")
        );
        assert!(report.evaluated_css.contains("border: 1px solid red"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
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
        let source = "@function tone($enabled) { @if $enabled { @return red; } @else { @return blue; } } .button { color: tone(var(--enabled)); }";
        let report = derive_static_stylesheet_module_evaluation(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.evaluated_css, source);
        assert_eq!(report.replacement_count, 0);
        assert_eq!(report.value_resolution.reference_count, 1);
        assert_eq!(report.value_resolution.top_count, 1);
        assert_eq!(report.value_resolution.unsupported_dynamic_count, 1);
        assert_eq!(report.value_resolution.values[0].outcome, "top");
        assert_eq!(
            report.value_resolution.values[0].reason,
            "unsupportedDynamic"
        );
        assert!(report.oracle.all_legacy_declaration_values_preserved);

        let resolution = summarize_static_stylesheet_value_resolution(source, StyleDialect::Scss);
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
    fn static_scss_evaluation_preserves_indirect_recursive_function_calls_as_top() {
        let source = "@function a($value) { @return b($value); } @function b($value) { @return a($value); } .button { color: a(red); }";
        let report = derive_static_stylesheet_module_evaluation(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.evaluated_css, source);
        assert_eq!(report.replacement_count, 0);
        assert_eq!(report.value_resolution.reference_count, 1);
        assert_eq!(report.value_resolution.top_count, 1);
        assert_eq!(report.value_resolution.cycle_count, 1);
        assert_eq!(report.value_resolution.values[0].outcome, "top");
        assert_eq!(report.value_resolution.values[0].reason, "cycle");
        assert!(report.oracle.all_legacy_declaration_values_preserved);

        let resolution = summarize_static_stylesheet_value_resolution(source, StyleDialect::Scss);
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
    fn static_scss_evaluation_preserves_recursive_function_calls_as_top() {
        let source =
            "@function loop($value) { @return loop($value); } .button { color: loop(red); }";
        let report = derive_static_stylesheet_module_evaluation(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.evaluated_css, source);
        assert_eq!(report.replacement_count, 0);
        assert_eq!(report.value_resolution.reference_count, 1);
        assert_eq!(report.value_resolution.top_count, 1);
        assert_eq!(report.value_resolution.cycle_count, 1);
        assert!(
            report
                .value_resolution
                .values
                .iter()
                .all(|value| value.outcome == "top" && value.reason == "cycle")
        );
        assert!(report.oracle.all_legacy_declaration_values_preserved);

        let resolution = summarize_static_stylesheet_value_resolution(source, StyleDialect::Scss);
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
    fn static_scss_evaluation_preserves_dynamic_mixin_local_variables_as_oracle_report() {
        let report = derive_static_stylesheet_module_evaluation(
            "@mixin tone { $space: meta.inspect((a: b)); margin: $space; } .button { @include tone; }",
            StyleDialect::Scss,
        );

        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 0);
        assert!(report.evaluated_css.contains("@mixin tone"));
        assert!(report.evaluated_css.contains("meta.inspect((a: b))"));
        assert!(report.evaluated_css.contains("@include tone"));
        assert!(!report.evaluated_css.contains("margin: (a: b)"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
    }

    #[test]
    fn static_scss_evaluation_preserves_recursive_nested_mixin_includes_as_oracle_report() {
        let report = derive_static_stylesheet_module_evaluation(
            "@mixin a { @include b; } @mixin b { @include a; } .button { @include a; }",
            StyleDialect::Scss,
        );

        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 0);
        assert!(report.evaluated_css.contains("@mixin a"));
        assert!(report.evaluated_css.contains("@mixin b"));
        assert!(report.evaluated_css.contains("@include a"));
        assert!(report.evaluated_css.contains("@include b"));
        assert!(report.oracle.all_legacy_declaration_values_preserved);
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
    fn static_scss_evaluation_preserves_forward_composite_as_top_oracle_report() {
        let source = "$border: 1px solid $brand; $brand: red; .button { border: $border; }";
        let report = derive_static_stylesheet_module_evaluation(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.replacement_count, 0);
        assert_eq!(report.evaluated_css, source);
        assert_eq!(report.value_resolution.reference_count, 1);
        assert_eq!(report.value_resolution.top_count, 1);
        assert_eq!(report.value_resolution.unresolved_reference_count, 1);
        assert_eq!(report.value_resolution.values[0].outcome, "top");
        assert_eq!(
            report.value_resolution.values[0].reason,
            "unresolvedReference"
        );
        assert!(report.oracle.all_legacy_declaration_values_preserved);
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
    fn static_value_resolution_emits_exact_alpha_color_mix_values() {
        let report = summarize_static_stylesheet_value_resolution(
            "$tone: color.mix(rgba(red, .5), blue); .button { color: $tone; }",
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
        assert_eq!(report.values[0].abstract_value_kind, "exact");
    }

    #[test]
    fn static_value_resolution_emits_exact_nested_opacity_color_mix_values() {
        let report = summarize_static_stylesheet_value_resolution(
            "$tone: color.mix(transparentize(red, .25), blue); .button { color: $tone; }",
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
        assert_eq!(report.values[0].abstract_value_kind, "exact");
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
