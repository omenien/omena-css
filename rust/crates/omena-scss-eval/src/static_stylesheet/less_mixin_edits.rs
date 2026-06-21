use std::collections::{BTreeMap, BTreeSet};

use omena_parser::LexedToken;

use super::{
    StaticLessDetachedRulesetDeclaration, StaticLessMixinAccessorCallRenderOutcome,
    StaticLessMixinAccessorEvaluationEdits, StaticLessMixinCallRenderOutcome,
    StaticLessMixinDeclaration, StaticLessMixinEvaluationEdits, StaticLessMixinRenderContext,
    StaticStylesheetEvaluationEdit, StaticStylesheetPropertyDeclaration, StaticStylesheetScope,
    StaticStylesheetVariableDeclaration, canonical_static_less_mixin_name,
    less_mixin_render::{render_static_less_mixin_accessor, render_static_less_mixin_call},
    less_mixins::{
        collect_static_less_mixin_accessors, collect_static_less_mixin_calls,
        collect_static_less_unsupported_mixin_call_suffix_ranges,
    },
    static_stylesheet_position_is_inside_ranges, static_stylesheet_scope_for_position,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn collect_static_less_mixin_evaluation_edits(
    source: &str,
    tokens: &[LexedToken],
    declarations: &[StaticLessMixinDeclaration],
    declaration_ranges: &[(usize, usize)],
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
            used_mixin_names: BTreeSet::new(),
            preserved_mixin_names: BTreeSet::new(),
        });
    }

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
    let mut preserved_mixin_names = BTreeSet::new();
    // NOTE: unsupported-suffix ranges are a COUNT metric only — they can overlap a call that
    // actually renders via the namespace path, so they are NOT a reliable preserved-name source.
    // Genuine preservation is captured authoritatively by the PreservedNoOutput render outcome
    // below (and a truly-unrenderable suffix call surfaces there). See StaticLessMixinEvaluationEdits.
    preserved_non_rendering_call_count += unsupported_suffix_ranges
        .iter()
        .filter(|(start, _)| {
            !static_stylesheet_position_is_inside_ranges(*start, declaration_ranges)
                && !static_stylesheet_position_is_inside_ranges(*start, excluded_call_ranges)
        })
        .count();
    for call in calls.iter().filter(|call| {
        !static_stylesheet_position_is_inside_ranges(call.start, declaration_ranges)
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
            StaticLessMixinCallRenderOutcome::KnownNoOutput {
                used_declaration_names: rendered_used_declaration_names,
            } => {
                used_declaration_names.extend(rendered_used_declaration_names);
                edits.push(StaticStylesheetEvaluationEdit {
                    start: call.start,
                    end: call.end,
                    replacement: String::new(),
                });
            }
            StaticLessMixinCallRenderOutcome::PreservedNoOutput => {
                preserved_non_rendering_call_count += 1;
                // A preserved (non-resolving) call still needs its matching declaration. Key on
                // the SAME name the deletion is keyed on: the namespace name for a namespace
                // call (declarations are deleted under the namespace), the inner name otherwise.
                preserved_mixin_names.insert(match call.namespace.as_ref() {
                    Some(namespace) => canonical_static_less_mixin_name(namespace.as_str()),
                    None => canonical_static_less_mixin_name(call.name.as_str()),
                });
            }
        }
    }

    // Declaration deletion is performed once, by the orchestrator (less_evaluation.rs), over the
    // UNION of used/preserved names from all passes — a same-name sibling overload kept alive by a
    // preserved call in any pass must not be deleted here. See StaticLessMixinEvaluationEdits.
    Some(StaticLessMixinEvaluationEdits {
        edits,
        preserved_non_rendering_call_count,
        used_mixin_names: used_declaration_names,
        preserved_mixin_names,
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn collect_static_less_mixin_accessor_evaluation_edits(
    source: &str,
    tokens: &[LexedToken],
    declarations: &[StaticLessMixinDeclaration],
    declaration_ranges: &[(usize, usize)],
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
            used_mixin_names: BTreeSet::new(),
            preserved_mixin_names: BTreeSet::new(),
        });
    }

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
    let mut preserved_mixin_names = BTreeSet::new();
    for accessor in accessors.iter().filter(|accessor| {
        !static_stylesheet_position_is_inside_ranges(accessor.start, declaration_ranges)
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
                preserved_mixin_names
                    .insert(canonical_static_less_mixin_name(accessor.name.as_str()));
            }
        }
    }

    // Declaration deletion is performed once, by the orchestrator (less_evaluation.rs); see
    // StaticLessMixinEvaluationEdits.
    Some(StaticLessMixinAccessorEvaluationEdits {
        edits,
        preserved_raw_accessor_count,
        used_mixin_names: used_declaration_names,
        preserved_mixin_names,
    })
}
