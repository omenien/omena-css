use std::collections::{BTreeMap, BTreeSet};

use super::{
    StaticLessDetachedRulesetAccessor, StaticLessDetachedRulesetAccessorEvaluationEdits,
    StaticLessDetachedRulesetAccessorRenderOutcome, StaticLessDetachedRulesetCall,
    StaticLessDetachedRulesetCallRenderOutcome, StaticLessDetachedRulesetDeclaration,
    StaticLessDetachedRulesetEvaluationEdits, StaticLessMixinDeclaration,
    StaticStylesheetEvaluationEdit, StaticStylesheetPropertyDeclaration, StaticStylesheetScope,
    StaticStylesheetVariableDeclaration,
    less_detached_ruleset_render::{
        render_static_less_detached_ruleset_accessor, render_static_less_detached_ruleset_body,
    },
    less_detached_rulesets::{
        find_static_less_detached_ruleset_declaration,
        static_less_detached_ruleset_ranges_from_declarations,
    },
    static_stylesheet_position_is_inside_ranges, static_stylesheet_scope_for_position,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn collect_static_less_detached_ruleset_evaluation_edits(
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
        !static_stylesheet_position_is_inside_ranges(
            declaration.span_start,
            mixin_declaration_ranges,
        ) && !preserved_declaration_keys.contains(&(declaration.scope_id, declaration.name.clone()))
            && !call_preserved_declaration_keys
                .contains(&(declaration.scope_id, declaration.name.clone()))
    }) {
        edits.push(StaticStylesheetEvaluationEdit {
            start: declaration.span_start,
            end: declaration.span_end,
            replacement: String::new(),
        });
    }
    // Mixin-declaration deletion is performed once by the orchestrator (less_evaluation.rs) over
    // the union of used/preserved names from all passes; forward this pass's used mixin names.
    // DEFERRED (documented, narrower than the fixed bug): a mixin call textually inside a
    // *preserved* detached-ruleset body is not enumerated, so preserved_mixin_names is empty for
    // this pass — this matches pre-fix behavior for that nested case and does not regress it; the
    // reproducible top-level/overload bug is closed because a top-level preserved sibling call
    // still contributes to the orchestrator's preserved set.
    Some(StaticLessDetachedRulesetEvaluationEdits {
        edits,
        preserved_raw_call_count,
        used_mixin_names: used_mixin_declaration_names,
        preserved_mixin_names: BTreeSet::new(),
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn collect_static_less_detached_ruleset_accessor_evaluation_edits(
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
