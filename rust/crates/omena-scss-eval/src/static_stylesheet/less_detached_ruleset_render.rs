use std::collections::{BTreeMap, BTreeSet};

use omena_parser::{StyleDialect, lex};

use super::{
    StaticLessDetachedRulesetAccessorRenderOutcome, StaticLessDetachedRulesetCallRenderOutcome,
    StaticLessDetachedRulesetDeclaration, StaticLessMixinDeclaration, StaticLessMixinRenderContext,
    StaticLessMixinRenderResult, StaticStylesheetPropertyDeclaration, StaticStylesheetScope,
    StaticStylesheetVariableDeclaration, collect_static_less_body_property_declarations,
    collect_static_stylesheet_scopes, find_static_less_property_declaration,
    less_detached_rulesets::collect_static_less_detached_ruleset_calls,
    less_mixin_render::{
        render_static_less_mixin_body_nested_calls, render_static_less_mixin_body_variables,
    },
    less_mixin_values::{
        resolve_static_less_mixin_body_declaration_values,
        resolve_static_less_mixin_value_with_bindings, static_less_mixin_body_scoped_values,
    },
    less_mixins::collect_static_less_mixin_calls,
    static_less_mixin_body_is_static_declaration_subset, static_less_variable_name_is_safe,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn render_static_less_detached_ruleset_body(
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
pub(super) fn render_static_less_detached_ruleset_accessor(
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
        match resolve_static_less_detached_ruleset_accessor_property_value(
            body,
            body_lexed.tokens(),
            member,
            &scoped_values,
            call_scope_id,
            context,
        )? {
            Some(value) => StaticLessDetachedRulesetAccessorRenderOutcome::Rendered(value),
            None => StaticLessDetachedRulesetAccessorRenderOutcome::PreservedRaw,
        },
    )
}

fn resolve_static_less_detached_ruleset_accessor_property_value(
    body: &str,
    body_tokens: &[omena_parser::LexedToken],
    member: &str,
    scoped_values: &BTreeMap<String, String>,
    call_scope_id: usize,
    context: StaticLessMixinRenderContext<'_>,
) -> Option<Option<String>> {
    let Some(body_scopes) = collect_static_stylesheet_scopes(body) else {
        return Some(None);
    };
    let Some(property_declarations) =
        collect_static_less_body_property_declarations(body, body_tokens, &body_scopes)
    else {
        return Some(None);
    };
    let member_name = format!("${member}");
    let Some(declaration) = find_static_less_property_declaration(
        member_name.as_str(),
        0,
        &body_scopes,
        &property_declarations,
    ) else {
        return Some(None);
    };
    Some(resolve_static_less_mixin_value_with_bindings(
        declaration.value.as_str(),
        scoped_values,
        context.captured_values,
        call_scope_id,
        context.scopes,
        context.variable_declarations,
        context.property_declarations,
        None,
        context.detached_ruleset_declarations,
    ))
}
