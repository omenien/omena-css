use std::collections::{BTreeMap, BTreeSet};

use omena_query_checker_orchestrator::{
    CanonicalSelector, OmenaCheckerCascadeDeclarationInputV0, OmenaCheckerCascadeInputV0,
    OmenaCheckerCustomPropertyInputV0,
};

use super::super::{
    ParserByteSpanV0, ParserRangeV0, omena_parser_dialect_for_style_path,
    parser_range_for_byte_span, summarize_static_css_custom_property_fixed_point_from_source,
};
use super::custom_property_registration::collect_query_checker_custom_property_registrations;
use super::source_scanner::{
    canonical_query_checker_selector, collect_query_var_references_in_value,
    find_query_top_level_byte, find_query_top_level_colon, matching_query_block_end,
    normalize_query_condition_prelude, query_at_root_selector_from_prelude,
    query_layer_name_from_prelude, query_prelude_start, query_value_has_important_suffix,
    split_query_selector_list, strip_query_statement_comments, trimmed_query_span,
};

pub(super) fn collect_query_checker_cascade_input(
    style_uri: &str,
    source: &str,
) -> (
    OmenaCheckerCascadeInputV0,
    BTreeMap<String, ParserRangeV0>,
    BTreeMap<String, ParserRangeV0>,
) {
    let declarations = collect_query_checker_cascade_declarations(source);
    let custom_property_registrations =
        collect_query_checker_custom_property_registrations(style_uri, source);
    let declaration_ranges = declarations
        .iter()
        .map(|declaration| {
            (
                declaration.input.declaration_id.clone(),
                parser_range_for_byte_span(source, declaration.byte_span),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let dialect = omena_parser_dialect_for_style_path(style_uri);
    let guaranteed_invalid_custom_properties =
        summarize_static_css_custom_property_fixed_point_from_source(source, dialect)
            .entries
            .into_iter()
            .filter(|entry| entry.guaranteed_invalid)
            .map(|entry| entry.name)
            .collect::<BTreeSet<_>>();
    let mut custom_properties_by_name =
        BTreeMap::<String, (BTreeSet<String>, bool, ParserByteSpanV0)>::new();

    for declaration in &declarations {
        if !declaration.input.property.starts_with("--") {
            continue;
        }
        let entry = custom_properties_by_name
            .entry(declaration.input.property.clone())
            .or_insert_with(|| {
                (
                    BTreeSet::new(),
                    guaranteed_invalid_custom_properties.contains(&declaration.input.property),
                    declaration.byte_span,
                )
            });
        entry.1 |= guaranteed_invalid_custom_properties.contains(&declaration.input.property);
        for dependency in collect_query_var_references_in_value(&declaration.input.value) {
            entry.0.insert(dependency);
        }
    }

    let custom_property_ranges = custom_properties_by_name
        .iter()
        .map(|(name, (_, _, byte_span))| {
            (name.clone(), parser_range_for_byte_span(source, *byte_span))
        })
        .collect::<BTreeMap<_, _>>();
    let custom_properties = custom_properties_by_name
        .into_iter()
        .map(
            |(name, (dependencies, guaranteed_invalid, _))| OmenaCheckerCustomPropertyInputV0 {
                name,
                dependencies: dependencies.into_iter().collect(),
                guaranteed_invalid,
            },
        )
        .collect::<Vec<_>>();

    (
        OmenaCheckerCascadeInputV0 {
            declarations: declarations
                .into_iter()
                .map(|declaration| declaration.input)
                .collect(),
            custom_properties,
            custom_property_registrations,
        },
        declaration_ranges,
        custom_property_ranges,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::style) struct QueryCheckerCascadeDeclaration {
    pub(in crate::style) input: OmenaCheckerCascadeDeclarationInputV0,
    pub(in crate::style) byte_span: ParserByteSpanV0,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QueryCheckerCascadeScope {
    condition_context: Vec<String>,
    layer_name: Option<String>,
    layer_order: Option<i32>,
}

pub(in crate::style) fn collect_query_checker_cascade_declarations(
    source: &str,
) -> Vec<QueryCheckerCascadeDeclaration> {
    #[cfg(test)]
    cascade_declarations_collect_probe::record();
    let mut declarations = Vec::new();
    let mut layer_orders = BTreeMap::new();
    let mut next_layer_order = 0i32;
    collect_query_checker_cascade_blocks(
        source,
        0,
        source.len(),
        None,
        Vec::new(),
        None,
        None,
        &mut layer_orders,
        &mut next_layer_order,
        &mut declarations,
    );
    declarations
}

#[allow(clippy::too_many_arguments)]
fn collect_query_checker_cascade_blocks(
    source: &str,
    start: usize,
    end: usize,
    parent_selector: Option<String>,
    condition_context: Vec<String>,
    layer_name: Option<String>,
    layer_order: Option<i32>,
    layer_orders: &mut BTreeMap<String, i32>,
    next_layer_order: &mut i32,
    declarations: &mut Vec<QueryCheckerCascadeDeclaration>,
) {
    let mut index = start;
    while let Some(open_index) = find_query_top_level_byte(source, index, end, b'{') {
        let Some(close_index) = matching_query_block_end(source, open_index, end) else {
            break;
        };
        let prelude_start = query_prelude_start(source, start, open_index);
        let prelude = source[prelude_start..open_index].trim();
        let body_start = open_index + 1;

        if let Some(layer) = query_layer_name_from_prelude(prelude) {
            let order = *layer_orders.entry(layer.clone()).or_insert_with(|| {
                let order = *next_layer_order;
                *next_layer_order += 1;
                order
            });
            collect_query_checker_cascade_blocks(
                source,
                body_start,
                close_index,
                parent_selector.clone(),
                condition_context.clone(),
                Some(layer),
                Some(order),
                layer_orders,
                next_layer_order,
                declarations,
            );
        } else if let Some(at_root_selector) = query_at_root_selector_from_prelude(prelude) {
            // `@at-root <selector> { ... }` resets the cascade context to the
            // document root and applies `<selector>` as the new context. This
            // keeps nested Sass declarations tied to the selector that will
            // own them after Sass expansion.
            let mut canonical_members = Vec::new();
            for member in split_query_selector_list(&at_root_selector) {
                let canonical_selector = canonical_query_checker_selector(None, &member);
                if !canonical_members.contains(&canonical_selector) {
                    canonical_members.push(canonical_selector);
                }
            }

            for canonical_selector in canonical_members {
                collect_query_checker_direct_declarations(
                    source,
                    body_start,
                    close_index,
                    &canonical_selector,
                    QueryCheckerCascadeScope {
                        condition_context: condition_context.clone(),
                        layer_name: layer_name.clone(),
                        layer_order,
                    },
                    declarations,
                );
                collect_query_checker_cascade_blocks(
                    source,
                    body_start,
                    close_index,
                    Some(canonical_selector),
                    condition_context.clone(),
                    layer_name.clone(),
                    layer_order,
                    layer_orders,
                    next_layer_order,
                    declarations,
                );
            }
        } else if prelude.starts_with('@') {
            let mut nested_condition_context = condition_context.clone();
            nested_condition_context.push(normalize_query_condition_prelude(prelude));
            collect_query_checker_cascade_blocks(
                source,
                body_start,
                close_index,
                parent_selector.clone(),
                nested_condition_context,
                layer_name.clone(),
                layer_order,
                layer_orders,
                next_layer_order,
                declarations,
            );
        } else if !prelude.is_empty() {
            // A selector list records one declaration set per member so each
            // member can tie with a sibling rule on the same selector. Identical
            // canonical members within one prelude are de-duplicated to avoid a
            // spurious self-tie.
            let mut canonical_members = Vec::new();
            for member in split_query_selector_list(prelude) {
                let canonical_selector =
                    canonical_query_checker_selector(parent_selector.as_deref(), &member);
                if !canonical_members.contains(&canonical_selector) {
                    canonical_members.push(canonical_selector);
                }
            }

            for canonical_selector in canonical_members {
                collect_query_checker_direct_declarations(
                    source,
                    body_start,
                    close_index,
                    &canonical_selector,
                    QueryCheckerCascadeScope {
                        condition_context: condition_context.clone(),
                        layer_name: layer_name.clone(),
                        layer_order,
                    },
                    declarations,
                );
                collect_query_checker_cascade_blocks(
                    source,
                    body_start,
                    close_index,
                    Some(canonical_selector),
                    condition_context.clone(),
                    layer_name.clone(),
                    layer_order,
                    layer_orders,
                    next_layer_order,
                    declarations,
                );
            }
        }

        index = close_index + 1;
    }
}

fn collect_query_checker_direct_declarations(
    source: &str,
    body_start: usize,
    body_end: usize,
    selector: &str,
    scope: QueryCheckerCascadeScope,
    declarations: &mut Vec<QueryCheckerCascadeDeclaration>,
) {
    let mut statement_start = body_start;
    let mut index = body_start;
    while index < body_end {
        if let Some(open_index) = find_query_top_level_byte(source, index, body_end, b'{') {
            while let Some(semicolon_index) =
                find_query_top_level_byte(source, index, open_index, b';')
            {
                push_query_checker_declaration(
                    source,
                    statement_start,
                    semicolon_index,
                    selector,
                    &scope,
                    declarations,
                );
                statement_start = semicolon_index + 1;
                index = statement_start;
            }
            let Some(close_index) = matching_query_block_end(source, open_index, body_end) else {
                break;
            };
            statement_start = close_index + 1;
            index = statement_start;
            continue;
        }

        while let Some(semicolon_index) = find_query_top_level_byte(source, index, body_end, b';') {
            push_query_checker_declaration(
                source,
                statement_start,
                semicolon_index,
                selector,
                &scope,
                declarations,
            );
            statement_start = semicolon_index + 1;
            index = statement_start;
        }
        break;
    }

    push_query_checker_declaration(
        source,
        statement_start,
        body_end,
        selector,
        &scope,
        declarations,
    );
}

fn push_query_checker_declaration(
    source: &str,
    start: usize,
    end: usize,
    selector: &str,
    scope: &QueryCheckerCascadeScope,
    declarations: &mut Vec<QueryCheckerCascadeDeclaration>,
) {
    let Some((trimmed_start, trimmed_end)) = trimmed_query_span(source, start, end) else {
        return;
    };
    let raw_statement = &source[trimmed_start..trimmed_end];
    // Strip CSS/Sass comments before the property/value split. A leading
    // comment before the property name otherwise poisons the property string,
    // so the whitespace guard below rejects it and drops the declaration from
    // cascade analysis.
    let statement = strip_query_statement_comments(raw_statement);
    let statement = statement.as_str();
    let Some(colon_offset) = find_query_top_level_colon(statement) else {
        return;
    };
    let property = statement[..colon_offset].trim();
    if property.is_empty()
        || property.starts_with('@')
        // Sass `$`-variable assignments are compile-time bindings that are erased
        // before CSS emission, so they never participate in the cascade.
        || property.starts_with('$')
        || property.contains(char::is_whitespace)
        || property.contains('{')
        || property.contains('}')
    {
        return;
    }
    let mut value = statement[colon_offset + 1..].trim().to_string();
    let important = query_value_has_important_suffix(&value);
    if important {
        value = value
            .trim_end()
            .trim_end_matches(|ch: char| ch.is_ascii_whitespace())
            .trim_end_matches("!important")
            .trim_end()
            .to_string();
    }
    let source_order = declarations.len();
    let declaration_id = format!("decl-{source_order}");
    declarations.push(QueryCheckerCascadeDeclaration {
        input: OmenaCheckerCascadeDeclarationInputV0 {
            declaration_id,
            selector: CanonicalSelector::from_canonical(selector),
            property: property.to_string(),
            value: value.clone(),
            source_order: source_order.min(u32::MAX as usize) as u32,
            condition_context: scope.condition_context.clone(),
            layer_name: scope.layer_name.clone(),
            layer_order: scope.layer_order,
            important,
            var_references: collect_query_var_references_in_value(&value),
        },
        byte_span: ParserByteSpanV0 {
            start: trimmed_start,
            end: trimmed_end,
        },
    });
}

/// Test-only per-thread counter of cascade-declaration collections, so the
/// substrate-backed narrowing paths can assert zero re-collections per candidate.
#[cfg(test)]
pub(crate) mod cascade_declarations_collect_probe {
    use std::cell::Cell;

    thread_local! {
        static COLLECT_CALLS: Cell<usize> = const { Cell::new(0) };
    }

    pub(crate) fn reset() {
        COLLECT_CALLS.with(|calls| calls.set(0));
    }

    pub(crate) fn count() -> usize {
        COLLECT_CALLS.with(Cell::get)
    }

    pub(super) fn record() {
        COLLECT_CALLS.with(|calls| calls.set(calls.get() + 1));
    }
}
