use std::collections::{BTreeMap, BTreeSet};

use omena_abstract_value::{AbstractCssValueV0, abstract_css_value_from_text};
use omena_parser::{LexedToken, StyleDialect, lex};
use omena_syntax::SyntaxKind;

use crate::value_eval::{reduce_static_scss_value, static_scss_literal_truthiness};

mod declarations;
mod edits;
mod less_colors;
mod less_detached_ruleset_edits;
mod less_detached_ruleset_render;
mod less_detached_rulesets;
mod less_evaluation;
mod less_guard;
mod less_literal_edits;
mod less_mixin_arguments;
mod less_mixin_edits;
mod less_mixin_render;
mod less_mixin_values;
mod less_mixins;
mod less_numbers;
mod less_predicates;
mod less_strings;
mod less_values;
mod less_variables;
mod model;
mod names;
mod oracle_corpus;
mod reports;
mod safety;
mod scopes;
mod scss_argument_binding;
mod scss_arguments;
mod scss_callable_dependencies;
mod scss_calls;
mod scss_declarations;
mod scss_evaluation;
mod scss_exports;
mod scss_function_edits;
mod scss_function_locals;
mod scss_loop_control_flow;
mod scss_loop_returns;
mod scss_mixin_body;
mod scss_mixin_control_flow;
mod scss_mixin_edits;
mod scss_return_clauses;
mod scss_variables;
mod scss_visibility;
mod tokens;
mod value_resolution_model;
mod value_resolution_summary;
mod variable_references;

use declarations::collect_static_less_body_property_declarations;
use edits::apply_static_stylesheet_evaluation_edits;
use less_detached_rulesets::find_static_less_detached_ruleset_declaration;
use less_evaluation::derive_static_less_stylesheet_module_evaluation;
use less_values::{
    format_static_less_channel_number, format_static_less_math_number, format_static_less_number,
};
use less_variables::{
    find_static_less_property_declaration, find_static_less_property_declaration_before,
    find_static_less_variable_declaration, resolve_static_less_property_value_in_scope,
    resolve_static_less_property_value_text_with_position,
    resolve_static_less_variable_value_in_scope,
};
pub use model::{
    OmenaScssEvalResolvedReplacementV0, OmenaScssEvalStaticStylesheetEvaluationV0,
    OmenaScssEvalStaticStylesheetNativeEditV0, OmenaScssEvalStaticValueResolutionReportV0,
    OmenaScssEvalStaticValueResolutionV0,
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
    StaticLessMixinRenderOutcome, StaticLessMixinRenderResult, StaticScssFunctionCall,
    StaticScssFunctionDeclaration, StaticScssFunctionEvaluationEdits, StaticScssFunctionLocalScope,
    StaticScssFunctionLocalVariable, StaticScssFunctionResolutionContext,
    StaticScssFunctionReturnClause, StaticScssLoopHeader, StaticScssMixinDeclaration,
    StaticScssMixinEvaluationEdits, StaticScssMixinIncludeCall, StaticScssMixinRenderResult,
    StaticStylesheetEvaluationEdit, StaticStylesheetPropertyDeclaration, StaticStylesheetScope,
    StaticStylesheetScopedVariableDeclaration, StaticStylesheetVariableDeclaration,
    StaticStylesheetVariableKind,
};
use names::{canonical_static_less_mixin_name, canonical_static_scss_function_name};
pub use names::{canonical_static_scss_variable_name, static_scss_variable_names_equal};
pub use oracle_corpus::{
    OmenaScssEvalStaticStylesheetOracleCorpusFixtureReportV0,
    OmenaScssEvalStaticStylesheetOracleCorpusReportV0, summarize_static_stylesheet_oracle_corpus,
};
use reports::{build_static_value_resolution_report, resolved_replacement_value};
use safety::{
    static_less_mixin_argument_value_is_safe, static_less_mixin_body_is_static_declaration_subset,
    static_less_mixin_hash_name_is_safe, static_less_mixin_name_part_is_safe,
    static_less_variable_name_is_safe, static_scss_mixin_body_is_static_declaration_subset,
    static_stylesheet_composite_value_is_safe,
    static_stylesheet_less_declaration_value_is_removal_safe,
    static_stylesheet_literal_value_is_safe, static_stylesheet_property_name_is_safe,
    static_stylesheet_variable_name_is_safe,
};
use scopes::{collect_static_stylesheet_scopes, static_stylesheet_scope_for_position};
use scss_argument_binding::{
    bind_static_scss_function_arguments, bind_static_scss_mixin_arguments,
};
use scss_arguments::{
    collect_static_scss_content_parameters, split_static_scss_function_arguments,
};
use scss_callable_dependencies::{
    extend_static_scss_used_function_dependencies,
    static_scss_function_value_contains_any_callable,
    static_scss_function_value_contains_callable_to,
};
use scss_calls::{
    collect_static_scss_function_calls, collect_static_scss_mixin_include_calls,
    collect_static_scss_mixin_include_calls_without_sass_content_blocks,
    static_scss_function_call_is_inside_declaration_body,
    static_scss_function_call_is_inside_mixin_declaration_body,
    static_scss_mixin_include_is_inside_declaration_body,
    static_scss_mixin_include_is_inside_function_declaration_body,
};
use scss_evaluation::derive_static_scss_stylesheet_module_evaluation;
pub use scss_exports::{
    derive_static_scss_stylesheet_module_configurable_variable_names,
    derive_static_scss_stylesheet_module_variable_exports,
};
use scss_loop_control_flow::render_static_scss_mixin_loop_control_flow_body;
use scss_loop_returns::{StaticScssLoopReturnResolution, resolve_static_scss_loop_return_clause};
use scss_mixin_body::{
    collect_static_scss_mixin_body_declaration_value_ranges,
    collect_static_scss_mixin_body_local_declarations,
};
use scss_mixin_control_flow::render_static_scss_mixin_control_flow_body;
use scss_variables::{
    resolve_static_scss_variable_abstract_value_at_position,
    resolve_static_scss_variable_value_at_position,
};
use scss_visibility::reduce_static_scss_metadata_with_function_context;
use tokens::{
    parser_text_size_to_usize, static_stylesheet_matching_token_index,
    static_stylesheet_next_token_kind_index, static_stylesheet_position_is_inside_ranges,
    static_stylesheet_skip_trivia_tokens, static_stylesheet_token_end,
    static_stylesheet_token_is_trivia, static_stylesheet_token_start,
    static_stylesheet_value_end_token_until,
};
use value_resolution_model::{
    StaticStylesheetAbstractResolution, StaticStylesheetResolutionOutcome,
    StaticStylesheetResolutionReason, raw_static_abstract_value, resolved_static_abstract_value,
    top_static_abstract_value,
};
use value_resolution_summary::{
    summarize_static_less_value_resolution_values, summarize_static_scss_value_resolution_values,
};
use variable_references::{
    collect_static_less_property_variable_references,
    collect_static_stylesheet_variable_references,
    collect_static_stylesheet_variable_references_with_options,
    static_stylesheet_variable_reference_is_named_argument_label,
};

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
        StaticStylesheetVariableKind::Scss => summarize_static_scss_value_resolution_values(
            style_source,
            dialect,
            &facts.variables,
            &scopes,
        )?,
        StaticStylesheetVariableKind::Less => {
            summarize_static_less_value_resolution_values(style_source, &facts.variables, &scopes)?
        }
    };
    Some(build_static_value_resolution_report(
        dialect_label(dialect),
        values,
    ))
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

fn resolve_static_scss_function_call_abstract_value(
    call: &StaticScssFunctionCall,
    dialect: StyleDialect,
    declarations: &[StaticScssFunctionDeclaration],
    mixin_declarations: &[StaticScssMixinDeclaration],
    scopes: &[StaticStylesheetScope],
    variable_declarations: &[StaticStylesheetScopedVariableDeclaration],
    fuel: usize,
) -> StaticStylesheetAbstractResolution {
    let active_functions = BTreeSet::new();
    let context = StaticScssFunctionResolutionContext {
        dialect,
        declarations,
        mixin_declarations,
        scopes,
        variable_declarations,
        active_functions: &active_functions,
    };
    resolve_static_scss_function_call_abstract_value_with_stack(call, context, fuel)
}

fn resolve_static_scss_function_call_abstract_value_with_stack(
    call: &StaticScssFunctionCall,
    context: StaticScssFunctionResolutionContext<'_>,
    fuel: usize,
) -> StaticStylesheetAbstractResolution {
    if fuel == 0 {
        return top_static_abstract_value(StaticStylesheetResolutionReason::FuelExhausted);
    }
    let Some(declaration) = context.declarations.iter().find(|declaration| {
        canonical_static_scss_function_name(declaration.name.as_str())
            == canonical_static_scss_function_name(call.name.as_str())
    }) else {
        return top_static_abstract_value(StaticStylesheetResolutionReason::UnresolvedReference);
    };
    if call.start >= declaration.body_start && call.start < declaration.body_end {
        return top_static_abstract_value(StaticStylesheetResolutionReason::Cycle);
    }
    let canonical_declaration_name = canonical_static_scss_function_name(declaration.name.as_str());
    if context
        .active_functions
        .contains(&canonical_declaration_name)
    {
        return top_static_abstract_value(StaticStylesheetResolutionReason::Cycle);
    }
    let mut next_active_functions = context.active_functions.clone();
    next_active_functions.insert(canonical_declaration_name);
    let next_context = StaticScssFunctionResolutionContext {
        active_functions: &next_active_functions,
        ..context
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
            next_context,
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
        next_context,
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
    let mut argument_values = BTreeMap::new();
    for (parameter, argument) in bind_static_scss_mixin_arguments(declaration, call)? {
        let resolution = resolve_static_scss_function_argument_abstract_value(
            argument.as_str(),
            &argument_values,
            call_position,
            STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
            context,
        );
        if resolution.outcome != StaticStylesheetResolutionOutcome::Resolved
            && resolution
                .rendered_value
                .as_deref()
                .is_none_or(|value| static_scss_literal_truthiness(value).is_none())
        {
            return None;
        }
        let rendered_value = resolution.rendered_value?;
        argument_values.insert(parameter, rendered_value);
    }

    let body = render_static_scss_mixin_control_flow_body(
        body,
        context.dialect,
        &argument_values,
        call_position,
        context,
    )?;
    let loop_argument_values = static_scss_mixin_body_loop_argument_values(
        body.as_str(),
        context.dialect,
        &argument_values,
        call_position,
        context,
    )?;
    let continuation_indent =
        static_scss_mixin_include_continuation_indent(source, context.dialect, call_position)?;
    let body = render_static_scss_mixin_loop_control_flow_body(
        body.as_str(),
        context.dialect,
        &loop_argument_values,
        continuation_indent.as_str(),
        call_position,
        context,
    )?;
    let body = render_static_scss_mixin_content_body(
        body.as_str(),
        context.dialect,
        call.content_body.as_deref(),
        call.content_parameters.as_slice(),
        &argument_values,
        call_position,
        context,
    )?;
    if !static_scss_mixin_body_is_static_declaration_subset(body.as_str()) {
        return None;
    }
    let body = render_static_scss_mixin_body_variables(
        body.as_str(),
        context.dialect,
        call_position,
        &argument_values,
        context,
    )?;
    let nested = render_static_scss_mixin_body_nested_includes(
        body.as_str(),
        context.dialect,
        source,
        tokens,
        call_position,
        context,
        active_mixins,
    )?;
    let body = resolve_static_scss_mixin_body_declaration_values(
        nested.body.as_str(),
        context.dialect,
        call_position,
        context,
    )?;
    let body = static_sass_mixin_body_replacement_for_include(body.as_str(), context.dialect);
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

fn render_static_scss_mixin_content_body(
    body: &str,
    dialect: StyleDialect,
    content_body: Option<&str>,
    content_parameters: &[String],
    argument_values: &BTreeMap<String, String>,
    call_position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<String> {
    if !body.to_ascii_lowercase().contains("@content") {
        return Some(body.to_string());
    }
    let content_body = content_body?;
    if !static_scss_content_block_is_static_declaration_subset(content_body, dialect) {
        return None;
    }
    let lexed = lex(body, dialect);
    let tokens = lexed.tokens();
    let mut edits = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::AtKeyword || !token.text.eq_ignore_ascii_case("@content") {
            continue;
        }
        let (content_arguments, terminator_index, content_call_end) =
            collect_static_scss_content_arguments(
                body,
                tokens,
                index,
                argument_values,
                call_position,
                context,
            )?;
        let content_argument_bindings =
            bind_static_scss_content_arguments(content_parameters, content_arguments.as_slice())?;
        let rendered_content_body =
            render_static_scss_content_body_parameters(content_body, &content_argument_bindings)?;
        let terminator = tokens.get(terminator_index)?;
        let replacement_end = match dialect {
            StyleDialect::Sass => match terminator.kind {
                SyntaxKind::SassOptionalSemicolon | SyntaxKind::SassDedent => {
                    static_stylesheet_token_end(terminator)
                }
                // Sass mixin body slices start after the block indent, so a following
                // same-block declaration can appear as a relative SassIndent.
                SyntaxKind::SassIndent => content_call_end,
                _ => return None,
            },
            _ if terminator.kind == SyntaxKind::Semicolon => {
                static_stylesheet_token_end(terminator)
            }
            _ => return None,
        };
        edits.push(StaticStylesheetEvaluationEdit {
            start: static_stylesheet_token_start(token),
            end: replacement_end,
            replacement: rendered_content_body.trim().to_string(),
        });
    }
    (!edits.is_empty()).then(|| apply_static_stylesheet_evaluation_edits(body, edits))?
}

fn collect_static_scss_content_arguments(
    body: &str,
    tokens: &[LexedToken],
    content_token_index: usize,
    argument_values: &BTreeMap<String, String>,
    call_position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<(Vec<String>, usize, usize)> {
    let after_content_index = static_stylesheet_skip_trivia_tokens(tokens, content_token_index + 1);
    if tokens
        .get(after_content_index)
        .is_none_or(|token| token.kind != SyntaxKind::LeftParen)
    {
        return Some((
            Vec::new(),
            after_content_index,
            static_stylesheet_token_end(&tokens[content_token_index]),
        ));
    }
    let close_index = static_stylesheet_matching_token_index(
        tokens,
        after_content_index,
        SyntaxKind::LeftParen,
        SyntaxKind::RightParen,
    )?;
    let argument_text = body.get(
        static_stylesheet_token_end(&tokens[after_content_index])
            ..static_stylesheet_token_start(&tokens[close_index]),
    )?;
    let arguments = split_static_scss_function_arguments(argument_text)?;
    let mut rendered_arguments = Vec::new();
    for argument in arguments {
        if argument.name.is_some() {
            return None;
        }
        let resolution = resolve_static_scss_function_value_with_bindings(
            argument.value.as_str(),
            argument_values,
            call_position,
            STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
            context,
        );
        if resolution.outcome != StaticStylesheetResolutionOutcome::Resolved {
            return None;
        }
        rendered_arguments.push(resolution.rendered_value?);
    }
    Some((
        rendered_arguments,
        static_stylesheet_skip_trivia_tokens(tokens, close_index + 1),
        static_stylesheet_token_end(&tokens[close_index]),
    ))
}

fn bind_static_scss_content_arguments(
    content_parameters: &[String],
    content_arguments: &[String],
) -> Option<BTreeMap<String, String>> {
    if content_parameters.len() != content_arguments.len() {
        return None;
    }
    Some(
        content_parameters
            .iter()
            .cloned()
            .zip(content_arguments.iter().cloned())
            .collect(),
    )
}

fn render_static_scss_content_body_parameters(
    content_body: &str,
    content_argument_bindings: &BTreeMap<String, String>,
) -> Option<String> {
    if content_argument_bindings.is_empty() {
        return Some(content_body.to_string());
    }
    let references = collect_static_stylesheet_variable_references_with_options(
        content_body,
        StaticStylesheetVariableKind::Scss,
        true,
        false,
    )?;
    let edits = references
        .into_iter()
        .filter_map(|reference| {
            let canonical_name = canonical_static_scss_variable_name(reference.name.as_str());
            content_argument_bindings
                .get(canonical_name.as_str())
                .map(|replacement| StaticStylesheetEvaluationEdit {
                    start: reference.start,
                    end: reference.end,
                    replacement: replacement.clone(),
                })
        })
        .collect::<Vec<_>>();
    apply_static_stylesheet_evaluation_edits(content_body, edits)
}

fn static_scss_content_block_is_static_declaration_subset(
    content_body: &str,
    dialect: StyleDialect,
) -> bool {
    let lower = content_body.to_ascii_lowercase();
    let has_nested_sass_block =
        dialect == StyleDialect::Sass && static_sass_content_block_has_nested_block(content_body);
    !has_nested_sass_block
        && !content_body.chars().any(|ch| matches!(ch, '{' | '}'))
        && !lower.contains("@content")
        && !lower.contains("@mixin")
        && !lower.contains("@function")
        && !lower.contains("@return")
        && !lower.contains("@if")
        && !lower.contains("@for")
        && !lower.contains("@each")
        && !lower.contains("@while")
}

fn static_sass_content_block_has_nested_block(content_body: &str) -> bool {
    let lexed = lex(content_body, StyleDialect::Sass);
    let mut depth = 0usize;
    for token in lexed.tokens() {
        match token.kind {
            SyntaxKind::SassIndent => {
                depth += 1;
                if depth > 1 {
                    return true;
                }
            }
            SyntaxKind::SassDedent => {
                depth = depth.saturating_sub(1);
            }
            _ => {}
        }
    }
    false
}

fn static_sass_mixin_body_replacement_for_include(body: &str, dialect: StyleDialect) -> String {
    if dialect == StyleDialect::Sass {
        return body.trim_start_matches([' ', '\t']).to_string();
    }
    body.to_string()
}

fn static_scss_mixin_body_loop_argument_values(
    body: &str,
    dialect: StyleDialect,
    argument_values: &BTreeMap<String, String>,
    call_position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<BTreeMap<String, String>> {
    let Some(first_loop_start) = static_scss_mixin_body_first_loop_position(body, dialect)? else {
        return Some(argument_values.clone());
    };
    let mut scoped_values = argument_values.clone();
    for local in collect_static_scss_mixin_body_local_declarations(body, dialect)?
        .into_iter()
        .filter(|local| local.declaration.span_start < first_loop_start)
    {
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
    Some(scoped_values)
}

fn static_scss_mixin_body_first_loop_position(
    body: &str,
    dialect: StyleDialect,
) -> Option<Option<usize>> {
    let lexed = lex(body, dialect);
    Some(lexed.tokens().iter().find_map(|token| {
        (token.kind == SyntaxKind::AtKeyword
            && (token.text.eq_ignore_ascii_case("@for")
                || token.text.eq_ignore_ascii_case("@each")
                || token.text.eq_ignore_ascii_case("@while")))
        .then(|| static_stylesheet_token_start(token))
    }))
}

fn static_scss_mixin_include_continuation_indent(
    source: &str,
    dialect: StyleDialect,
    call_position: usize,
) -> Option<String> {
    if dialect != StyleDialect::Sass {
        return Some(String::new());
    }
    let prefix = source.get(..call_position)?;
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    let indent = source.get(line_start..call_position)?;
    indent
        .chars()
        .all(|character| character == ' ' || character == '\t')
        .then(|| indent.to_string())
}

fn render_static_scss_mixin_body_nested_includes(
    body: &str,
    dialect: StyleDialect,
    source: &str,
    tokens: &[LexedToken],
    call_position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
    active_mixins: &mut BTreeSet<String>,
) -> Option<StaticScssMixinRenderResult> {
    let body_lexed = lex(body, dialect);
    let calls = collect_static_scss_mixin_include_calls_without_sass_content_blocks(
        body,
        dialect,
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
    dialect: StyleDialect,
    call_position: usize,
    argument_values: &BTreeMap<String, String>,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<String> {
    let local_declarations = collect_static_scss_mixin_body_local_declarations(body, dialect)?;
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

fn resolve_static_scss_mixin_body_declaration_values(
    body: &str,
    dialect: StyleDialect,
    call_position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<String> {
    let value_ranges = collect_static_scss_mixin_body_declaration_value_ranges(body, dialect)?;
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
            context,
            fuel - 1,
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

fn dialect_label(dialect: StyleDialect) -> &'static str {
    match dialect {
        StyleDialect::Css => "css",
        StyleDialect::Scss => "scss",
        StyleDialect::Sass => "sass",
        StyleDialect::Less => "less",
    }
}

#[cfg(test)]
mod tests;
