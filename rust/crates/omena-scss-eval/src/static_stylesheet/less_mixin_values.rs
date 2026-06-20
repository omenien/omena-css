use std::collections::{BTreeMap, BTreeSet};

use omena_parser::{ParsedVariableFactKind, StyleDialect, lex};

use crate::value_eval::static_scss_bang_usage_is_comparison_only;

use super::{
    StaticLessBodyPropertyValueOutcome, StaticLessDetachedRulesetDeclaration,
    StaticLessMixinBodyLocalDeclaration, StaticLessMixinRenderContext,
    StaticStylesheetEvaluationEdit, StaticStylesheetPropertyDeclaration, StaticStylesheetScope,
    StaticStylesheetVariableDeclaration, StaticStylesheetVariableKind,
    apply_static_stylesheet_evaluation_edits, collect_static_less_body_property_declarations,
    collect_static_less_property_variable_references, collect_static_stylesheet_scopes,
    collect_static_stylesheet_variable_references_with_options,
    declarations::extract_static_stylesheet_variable_declaration,
    find_static_less_detached_ruleset_declaration, find_static_less_property_declaration,
    less_values::reduce_static_less_value, model::StaticScssFunctionArgument,
    parser_text_size_to_usize, resolve_static_less_property_value_text_with_position,
    resolve_static_less_variable_value_in_scope,
    scss_mixin_body::collect_static_scss_mixin_body_declaration_value_ranges,
    static_less_mixin_argument_value_is_safe, static_stylesheet_composite_value_is_safe,
    static_stylesheet_less_declaration_value_is_removal_safe,
    static_stylesheet_literal_value_is_safe, static_stylesheet_property_name_is_safe,
    static_stylesheet_variable_reference_is_named_argument_label,
};

pub(super) fn static_less_mixin_arguments_value(
    arguments: &[StaticScssFunctionArgument],
) -> Option<String> {
    arguments
        .iter()
        .map(|argument| {
            static_less_mixin_argument_value_is_safe(argument.value.as_str())
                .then(|| argument.value.clone())
        })
        .collect::<Option<Vec<_>>>()
        .map(|values| values.join(", "))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn static_less_mixin_body_scoped_values(
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

pub(super) fn static_less_mixin_accessor_property_value(
    body: &str,
    member: &str,
    scoped_values: &BTreeMap<String, String>,
    call_scope_id: usize,
    context: StaticLessMixinRenderContext<'_>,
) -> Option<StaticLessBodyPropertyValueOutcome> {
    static_less_body_property_value(body, member, scoped_values, call_scope_id, context)
}

pub(super) fn static_less_body_property_value(
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

pub(super) fn collect_static_less_mixin_body_local_declarations(
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
            StyleDialect::Less,
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
pub(super) fn resolve_static_less_mixin_value_with_bindings(
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

pub(super) fn static_less_value_is_detached_ruleset_reference(
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

pub(super) fn resolve_static_less_mixin_body_declaration_values(body: &str) -> Option<String> {
    let value_ranges =
        collect_static_scss_mixin_body_declaration_value_ranges(body, StyleDialect::Less)?;
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

pub(super) fn apply_static_less_mixin_call_importance(body: &str) -> Option<String> {
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
