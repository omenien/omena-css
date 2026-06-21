use std::collections::{BTreeMap, BTreeSet};

use omena_parser::{ParsedVariableFact, ParsedVariableFactKind, StyleDialect, lex};
use omena_syntax::SyntaxKind;

use super::{
    canonical_static_less_mixin_name,
    declarations::{
        collect_static_less_property_declarations, collect_static_less_variable_declarations,
        static_less_mixin_declaration_ranges_from_declarations,
    },
    edits::apply_static_stylesheet_evaluation_edits,
    less_detached_ruleset_edits::{
        collect_static_less_detached_ruleset_accessor_evaluation_edits,
        collect_static_less_detached_ruleset_evaluation_edits,
    },
    less_detached_rulesets::{
        collect_static_less_detached_ruleset_accessors, collect_static_less_detached_ruleset_calls,
        collect_static_less_detached_ruleset_declarations,
        static_less_detached_ruleset_ranges_from_accessors,
        static_less_detached_ruleset_ranges_from_calls,
        static_less_detached_ruleset_ranges_from_declarations,
    },
    less_interpolation::collect_static_less_interpolation_edits,
    less_literal_edits::collect_static_less_literal_value_edits,
    less_mixin_edits::{
        collect_static_less_mixin_accessor_evaluation_edits,
        collect_static_less_mixin_evaluation_edits,
    },
    less_mixin_values::static_less_value_is_detached_ruleset_reference,
    less_mixins::{
        collect_static_less_mixin_accessors, collect_static_less_mixin_calls,
        collect_static_less_mixin_declarations, static_less_mixin_accessor_ranges_from_accessors,
        static_less_mixin_ranges_from_calls,
    },
    less_variables::{
        resolve_static_less_property_value_in_scope, resolve_static_less_variable_value_in_scope,
    },
    model::{
        OmenaScssEvalResolvedReplacementV0, OmenaScssEvalStaticStylesheetEvaluationV0,
        StaticLessDetachedRulesetDeclaration, StaticStylesheetEvaluationEdit,
        StaticStylesheetPropertyDeclaration, StaticStylesheetScope,
        StaticStylesheetVariableDeclaration, StaticStylesheetVariableKind,
    },
    reports::{build_static_stylesheet_evaluation_report, resolved_replacement_value},
    scopes::{
        collect_static_stylesheet_scopes, static_stylesheet_position_is_inside_scoped_declaration,
        static_stylesheet_scope_for_position,
    },
    tokens::{
        parser_text_size_to_usize, static_stylesheet_position_is_inside_ranges,
        static_stylesheet_token_end, static_stylesheet_token_start,
    },
    variable_references::static_stylesheet_variable_reference_is_named_argument_label,
};

pub(super) fn derive_static_less_stylesheet_module_evaluation(
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
    let interpolation_edits = collect_static_less_interpolation_edits(
        style_source,
        tokens,
        &scopes,
        &declarations,
        &property_declarations,
        detached_rulesets.as_slice(),
        &mixin_declaration_ranges,
        &detached_ruleset_ranges,
    )?;

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
    let indirect_variable_edits = collect_static_less_indirect_variable_reference_edits(
        style_source,
        &scopes,
        &declarations,
        &property_declarations,
        detached_rulesets.as_slice(),
        &mixin_declaration_ranges,
        &detached_ruleset_ranges,
        &detached_ruleset_call_ranges,
        &detached_ruleset_accessor_ranges,
        &mixin_accessor_ranges,
    )?;
    let indirect_variable_ranges = indirect_variable_edits.ranges.clone();
    resolved_replacements.extend(indirect_variable_edits.resolved_replacements);
    edits.extend(indirect_variable_edits.edits);
    for fact in variable_facts {
        if fact.kind != ParsedVariableFactKind::LessReference {
            continue;
        }
        let fact_reference_start = parser_text_size_to_usize(fact.range.start().into());
        let fact_reference_end = parser_text_size_to_usize(fact.range.end().into());
        if static_stylesheet_position_is_inside_ranges(
            fact_reference_start,
            &indirect_variable_ranges,
        ) {
            continue;
        }
        if static_stylesheet_variable_reference_is_named_argument_label(
            style_source,
            fact_reference_start,
            fact_reference_end,
        ) {
            continue;
        }
        if fact.name == "@"
            && static_less_indirect_variable_reference_at(style_source, fact_reference_start)
                .is_some()
        {
            continue;
        }
        let (reference_name, reference_start, reference_end) =
            if static_less_variable_reference_is_indirect_inner(style_source, fact_reference_start)
            {
                let indirect_start = fact_reference_start.checked_sub(1)?;
                let (indirect_name, indirect_end) =
                    static_less_indirect_variable_reference_at(style_source, indirect_start)?;
                (indirect_name, indirect_start, indirect_end)
            } else if let Some((indirect_name, indirect_end)) =
                static_less_indirect_variable_reference_at(style_source, fact_reference_start)
            {
                (indirect_name, fact_reference_start, indirect_end)
            } else {
                (fact.name.clone(), fact_reference_start, fact_reference_end)
            };
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
                reference_name.as_str(),
                reference_scope_id,
                &scopes,
                detached_rulesets.as_slice(),
            )
        {
            continue;
        }
        let mut stack = BTreeSet::new();
        let replacement = resolve_static_less_variable_value_in_scope(
            reference_name.as_str(),
            reference_scope_id,
            &scopes,
            &declarations,
            &property_declarations,
            detached_rulesets.as_slice(),
            &mut stack,
        )?;
        let replacement = replacement.text;
        resolved_replacements.push(resolved_replacement_value(
            reference_name.as_str(),
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
    edits.extend(interpolation_edits);
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
    let mut union_used_mixin_names = BTreeSet::new();
    let mut union_preserved_mixin_names = BTreeSet::new();
    union_used_mixin_names.extend(detached_ruleset_evaluation_edits.used_mixin_names);
    union_preserved_mixin_names.extend(detached_ruleset_evaluation_edits.preserved_mixin_names);
    edits.extend(detached_ruleset_evaluation_edits.edits);
    edits.extend(detached_ruleset_accessor_evaluation_edits.edits);
    let accessor_evaluation_edits = collect_static_less_mixin_accessor_evaluation_edits(
        style_source,
        tokens,
        &mixin_declarations,
        &mixin_declaration_ranges,
        &detached_rulesets,
        &scopes,
        &declarations,
        &property_declarations,
        &detached_ruleset_ranges,
    )?;
    preserved_less_evaluation_count += accessor_evaluation_edits.preserved_raw_accessor_count;
    union_used_mixin_names.extend(accessor_evaluation_edits.used_mixin_names);
    union_preserved_mixin_names.extend(accessor_evaluation_edits.preserved_mixin_names);
    edits.extend(accessor_evaluation_edits.edits);
    if let Some(mixin_evaluation_edits) = collect_static_less_mixin_evaluation_edits(
        style_source,
        tokens,
        &mixin_declarations,
        &mixin_declaration_ranges,
        &detached_rulesets,
        &scopes,
        &declarations,
        &property_declarations,
        &detached_ruleset_ranges,
    ) {
        preserved_less_evaluation_count +=
            mixin_evaluation_edits.preserved_non_rendering_call_count;
        union_used_mixin_names.extend(mixin_evaluation_edits.used_mixin_names);
        union_preserved_mixin_names.extend(mixin_evaluation_edits.preserved_mixin_names);
        edits.extend(mixin_evaluation_edits.edits);
    }
    // Single mixin-declaration deletion authority (decouple-to-orchestrator): delete a top-level
    // mixin declaration only when its canonical name is statically CONSUMED by some pass AND no
    // preserved (non-resolving) reference in any pass still needs it. This closes the name-keyed
    // overload bug where a rendered call to one overload deleted a sibling overload that a
    // preserved call still referenced (dropping live CSS). Correctness-monotone: any preserved
    // reference to a name keeps all same-name declarations.
    for declaration in mixin_declarations.iter().filter(|declaration| {
        let name = canonical_static_less_mixin_name(declaration.name.as_str());
        union_used_mixin_names.contains(&name) && !union_preserved_mixin_names.contains(&name)
    }) {
        edits.push(StaticStylesheetEvaluationEdit {
            start: declaration.span_start,
            end: declaration.span_end,
            replacement: String::new(),
        });
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

struct StaticLessIndirectVariableEdits {
    edits: Vec<StaticStylesheetEvaluationEdit>,
    resolved_replacements: Vec<OmenaScssEvalResolvedReplacementV0>,
    ranges: Vec<(usize, usize)>,
}

#[allow(clippy::too_many_arguments)]
fn collect_static_less_indirect_variable_reference_edits(
    source: &str,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    property_declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
    mixin_declaration_ranges: &[(usize, usize)],
    detached_ruleset_ranges: &[(usize, usize)],
    detached_ruleset_call_ranges: &[(usize, usize)],
    detached_ruleset_accessor_ranges: &[(usize, usize)],
    mixin_accessor_ranges: &[(usize, usize)],
) -> Option<StaticLessIndirectVariableEdits> {
    let mut edits = Vec::new();
    let mut resolved_replacements = Vec::new();
    let mut ranges = Vec::new();
    let mut index = 0usize;
    let mut quote: Option<char> = None;

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
        if source.get(index..index + 2) == Some("/*") {
            let end = source.get(index + 2..)?.find("*/")?;
            index += end + 4;
            continue;
        }
        if source.get(index..index + 2) == Some("//") {
            index = source
                .get(index + 2..)?
                .find('\n')
                .map(|offset| index + 2 + offset)
                .unwrap_or(source.len());
            continue;
        }

        let Some((reference_name, reference_end)) =
            static_less_indirect_variable_reference_at(source, index)
        else {
            index += ch.len_utf8();
            continue;
        };
        if static_stylesheet_position_is_inside_scoped_declaration(declarations, index)
            || static_stylesheet_position_is_inside_ranges(index, mixin_declaration_ranges)
            || static_stylesheet_position_is_inside_ranges(index, detached_ruleset_ranges)
            || static_stylesheet_position_is_inside_ranges(index, detached_ruleset_call_ranges)
            || static_stylesheet_position_is_inside_ranges(index, detached_ruleset_accessor_ranges)
            || static_stylesheet_position_is_inside_ranges(index, mixin_accessor_ranges)
        {
            index = reference_end;
            continue;
        }
        let reference_scope_id = static_stylesheet_scope_for_position(scopes, index)?;
        let mut stack = BTreeSet::new();
        let replacement = resolve_static_less_variable_value_in_scope(
            reference_name.as_str(),
            reference_scope_id,
            scopes,
            declarations,
            property_declarations,
            detached_ruleset_declarations,
            &mut stack,
        )?;
        let replacement = replacement.text;
        resolved_replacements.push(resolved_replacement_value(
            reference_name.as_str(),
            index,
            reference_end,
            replacement.as_str(),
        ));
        edits.push(StaticStylesheetEvaluationEdit {
            start: index,
            end: reference_end,
            replacement,
        });
        ranges.push((index, reference_end));
        index = reference_end;
    }

    Some(StaticLessIndirectVariableEdits {
        edits,
        resolved_replacements,
        ranges,
    })
}

fn static_less_indirect_variable_reference_at(
    source: &str,
    reference_start: usize,
) -> Option<(String, usize)> {
    let rest = source.get(reference_start..)?;
    if !rest.starts_with("@@") {
        return None;
    }
    let name_start = reference_start + "@@".len();
    let mut name_end = name_start;
    while name_end < source.len() {
        let ch = source[name_end..].chars().next()?;
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-') {
            name_end += ch.len_utf8();
            continue;
        }
        break;
    }
    (name_end > name_start).then(|| (source[reference_start..name_end].to_string(), name_end))
}

fn static_less_variable_reference_is_indirect_inner(source: &str, reference_start: usize) -> bool {
    reference_start > 0 && source.get(reference_start - 1..reference_start) == Some("@")
}
