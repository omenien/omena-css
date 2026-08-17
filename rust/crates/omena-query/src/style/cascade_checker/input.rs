use std::collections::{BTreeMap, BTreeSet};

use omena_cascade::CascadeStandardValueVerdictV0;
use omena_query_checker_orchestrator::{
    CanonicalSelector, OmenaCheckerCascadeDeclarationInputV0, OmenaCheckerCascadeInputV0,
    OmenaCheckerCustomPropertyInputV0, standard_property_value_verdicts_v0,
};
use omena_semantic::{
    LayerBindingResolutionV0, StyleLayerIndexV0, layer_ordinal_for_byte_span,
    summarize_style_layer_order_from_source,
};
use omena_syntax::StyleDialect;

use super::super::{
    ParserByteSpanV0, ParserRangeV0, omena_parser_dialect_for_style_path,
    parser_range_for_byte_span, summarize_static_css_custom_property_fixed_point_from_source,
};
use super::custom_property_registration::collect_query_checker_custom_property_registrations;
use super::declaration_facts::collect_parsed_declaration_fact_collection;
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
) -> QueryCheckerCascadeInputCollection {
    let dialect = omena_parser_dialect_for_style_path(style_uri);
    let collection = collect_query_checker_cascade_declaration_collection(source, dialect);
    let declarations = collection.declarations;
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
    let checker_declarations = declarations
        .into_iter()
        .map(|declaration| declaration.input)
        .collect::<Vec<_>>();
    let standard_property_value_verdicts =
        standard_property_value_verdicts_v0(checker_declarations.as_slice());

    QueryCheckerCascadeInputCollection {
        checker_input: OmenaCheckerCascadeInputV0 {
            declarations: checker_declarations,
            custom_properties,
            custom_property_registrations,
        },
        declaration_ranges,
        custom_property_ranges,
        topology_incomplete_unresolved_count: collection.topology_incomplete_unresolved_count,
        standard_property_value_verdicts,
    }
}

pub(super) struct QueryCheckerCascadeInputCollection {
    pub(super) checker_input: OmenaCheckerCascadeInputV0,
    pub(super) declaration_ranges: BTreeMap<String, ParserRangeV0>,
    pub(super) custom_property_ranges: BTreeMap<String, ParserRangeV0>,
    pub(super) topology_incomplete_unresolved_count: Option<usize>,
    pub(super) standard_property_value_verdicts: BTreeMap<String, CascadeStandardValueVerdictV0>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::style) struct QueryCheckerCascadeDeclaration {
    pub(in crate::style) input: OmenaCheckerCascadeDeclarationInputV0,
    pub(in crate::style) byte_span: ParserByteSpanV0,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QueryCheckerCascadeScope {
    condition_context: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QueryCheckerCascadeDeclarationCollection {
    declarations: Vec<QueryCheckerCascadeDeclaration>,
    topology_incomplete_unresolved_count: Option<usize>,
}

pub(in crate::style) fn collect_query_checker_cascade_declarations(
    source: &str,
) -> Vec<QueryCheckerCascadeDeclaration> {
    collect_query_checker_cascade_declarations_with_dialect(source, StyleDialect::Css)
}

pub(in crate::style) fn collect_query_checker_cascade_declarations_with_dialect(
    source: &str,
    dialect: StyleDialect,
) -> Vec<QueryCheckerCascadeDeclaration> {
    collect_query_checker_cascade_declaration_collection(source, dialect).declarations
}

fn collect_query_checker_cascade_declaration_collection(
    source: &str,
    dialect: StyleDialect,
) -> QueryCheckerCascadeDeclarationCollection {
    collect_query_checker_cascade_declaration_collection_from_scanner(source, dialect)
}

#[allow(dead_code)]
pub(in crate::style) fn collect_query_checker_cascade_declarations_from_facts(
    style_uri: &str,
    source: &str,
    dialect: StyleDialect,
) -> Vec<QueryCheckerCascadeDeclaration> {
    collect_query_checker_cascade_declaration_collection_from_facts(style_uri, source, dialect)
        .declarations
}

#[allow(dead_code)]
fn collect_query_checker_cascade_declaration_collection_from_facts(
    style_uri: &str,
    source: &str,
    dialect: StyleDialect,
) -> QueryCheckerCascadeDeclarationCollection {
    let collection = collect_parsed_declaration_fact_collection(style_uri, source, dialect);
    QueryCheckerCascadeDeclarationCollection {
        declarations: collection
            .facts
            .into_iter()
            .map(|fact| QueryCheckerCascadeDeclaration {
                input: fact.checker_input(),
                byte_span: fact.byte_span,
            })
            .collect(),
        topology_incomplete_unresolved_count: collection.topology_incomplete_unresolved_count,
    }
}

fn collect_query_checker_cascade_declaration_collection_from_scanner(
    source: &str,
    dialect: StyleDialect,
) -> QueryCheckerCascadeDeclarationCollection {
    #[cfg(test)]
    cascade_declarations_collect_probe::record();
    let layer_index = summarize_style_layer_order_from_source(source, dialect);
    let mut declarations = Vec::new();
    collect_query_checker_cascade_blocks(
        source,
        0,
        source.len(),
        None,
        Vec::new(),
        &mut declarations,
    );
    for declaration in &mut declarations {
        apply_query_checker_layer_binding(&layer_index, declaration);
    }
    QueryCheckerCascadeDeclarationCollection {
        declarations,
        topology_incomplete_unresolved_count: (!layer_index.topology_complete)
            .then_some(layer_index.unresolved_topology_count),
    }
}

fn collect_query_checker_cascade_blocks(
    source: &str,
    start: usize,
    end: usize,
    parent_selector: Option<String>,
    condition_context: Vec<String>,
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

        if query_layer_name_from_prelude(prelude).is_some() {
            collect_query_checker_cascade_blocks(
                source,
                body_start,
                close_index,
                parent_selector.clone(),
                condition_context.clone(),
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
                    },
                    declarations,
                );
                collect_query_checker_cascade_blocks(
                    source,
                    body_start,
                    close_index,
                    Some(canonical_selector),
                    condition_context.clone(),
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
                    },
                    declarations,
                );
                collect_query_checker_cascade_blocks(
                    source,
                    body_start,
                    close_index,
                    Some(canonical_selector),
                    condition_context.clone(),
                    declarations,
                );
            }
        }

        index = close_index + 1;
    }
}

fn apply_query_checker_layer_binding(
    layer_index: &StyleLayerIndexV0,
    declaration: &mut QueryCheckerCascadeDeclaration,
) {
    let span = declaration.byte_span;
    let resolution = layer_ordinal_for_byte_span(layer_index, span.start, span.end);
    let binding = layer_index
        .block_bindings
        .iter()
        .filter(|binding| {
            binding.byte_span.start <= span.start && span.end <= binding.byte_span.end
        })
        .max_by_key(|binding| binding.nesting_depth);

    match resolution {
        LayerBindingResolutionV0::Resolved(ordinal) => {
            declaration.input.layer_name = binding.map(|binding| binding.canonical_name.clone());
            declaration.input.layer_order = ordinal.map(omena_cascade::LayerOrdinal::get);
        }
        LayerBindingResolutionV0::TopologyIncomplete { .. } => {
            declaration.input.layer_name = None;
            declaration.input.layer_order = None;
        }
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
            layer_name: None,
            layer_order: None,
            origin: omena_cascade::CascadeOriginV0::Author,
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

#[cfg(test)]
mod layer_binding_tests {
    use super::*;
    use omena_cascade::{
        CascadeComputedValueInputV0, CascadeDeclaration, CascadeKey, CascadeLevel,
        CascadeStandardValueVerdictV0, CascadeValue, ComputedCascadeValueStatusV0,
        CustomPropertyEnv, OpenWorldTieEvidence, Specificity, SpecificityExactnessV0,
        compute_cascade_computed_value, normalized_layer_rank,
    };
    use omena_query_checker_orchestrator::{
        OmenaCheckerCascadeInputV0,
        run_omena_query_checker_cascade_gate_with_standard_property_value_verdicts_v0,
    };

    #[test]
    fn one_standard_grammar_verdict_drives_checker_and_computed_value() {
        let declaration_id = "invalid-color";
        let invalid_value = "definitely-not-a-color";
        let non_definite_value = "!!! not-a-color 42px };drop";
        assert_eq!(
            omena_query_checker_orchestrator::standard_property_value_verdict_v0(
                "color",
                non_definite_value,
            ),
            CascadeStandardValueVerdictV0::Unknown
        );
        let checker_input = OmenaCheckerCascadeInputV0 {
            declarations: vec![OmenaCheckerCascadeDeclarationInputV0 {
                declaration_id: declaration_id.to_string(),
                selector: CanonicalSelector::from_canonical(".target"),
                property: "color".to_string(),
                value: invalid_value.to_string(),
                source_order: 0,
                condition_context: Vec::new(),
                layer_name: None,
                layer_order: None,
                origin: omena_cascade::CascadeOriginV0::Author,
                important: false,
                var_references: Vec::new(),
            }],
            custom_properties: Vec::new(),
            custom_property_registrations: Vec::new(),
        };
        let verdicts = standard_property_value_verdicts_v0(checker_input.declarations.as_slice());
        let checker_gate =
            run_omena_query_checker_cascade_gate_with_standard_property_value_verdicts_v0(
                checker_input,
                &verdicts,
            );
        let computed = compute_cascade_computed_value(CascadeComputedValueInputV0 {
            property: "color".to_string(),
            declarations: vec![CascadeDeclaration {
                id: declaration_id.to_string(),
                property: "color".to_string(),
                value: CascadeValue::Literal(invalid_value.to_string()),
                key: CascadeKey::new(
                    CascadeLevel::AuthorNormal,
                    normalized_layer_rank(false, None),
                    0,
                    Specificity::ZERO,
                    0,
                ),
                open_world_tie_evidence: OpenWorldTieEvidence::NONE,
                specificity_exactness: SpecificityExactnessV0::Exact,
            }],
            custom_property_env: CustomPropertyEnv::new(),
            parent_computed_value: None,
            registered_custom_property: None,
            standard_property_value_verdicts: verdicts.clone(),
        });

        assert_eq!(
            verdicts.get(declaration_id),
            Some(&CascadeStandardValueVerdictV0::Unmatched)
        );
        assert!(
            checker_gate
                .evaluations
                .iter()
                .any(|evaluation| evaluation.rule_code_name == "invalid-property-value")
        );
        assert_eq!(
            computed.status,
            ComputedCascadeValueStatusV0::InvalidAtComputedValueTime
        );
        assert!(computed.invalid_at_computed_value_time);
    }

    #[test]
    fn cascade_input_collection_carries_standard_grammar_verdicts() {
        let collection = collect_query_checker_cascade_input(
            "file:///tmp/invalid.css",
            ".target { color: definitely-not-a-color; }",
        );
        let declaration_id = collection.checker_input.declarations[0]
            .declaration_id
            .as_str();

        assert_eq!(
            collection
                .standard_property_value_verdicts
                .get(declaration_id),
            Some(&CascadeStandardValueVerdictV0::Unmatched)
        );
    }

    #[test]
    fn statement_order_and_nested_paths_share_the_semantic_layer_index() {
        let statement_order = collect_query_checker_cascade_declarations_with_dialect(
            r#"
@layer overrides, base;
@layer base { .target { color: red; } }
@layer overrides { .target { color: blue; } }
"#,
            StyleDialect::Css,
        );
        let statement_bindings = statement_order
            .iter()
            .map(|declaration| {
                (
                    declaration.input.value.as_str(),
                    declaration.input.layer_name.as_deref(),
                    declaration.input.layer_order,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            statement_bindings,
            vec![
                ("red", Some("base"), Some(1)),
                ("blue", Some("overrides"), Some(0)),
            ]
        );

        let nested = collect_query_checker_cascade_declarations_with_dialect(
            r#"
@layer b { .target { color: blue; } }
@layer a { @layer b { .target { color: purple; } } }
"#,
            StyleDialect::Css,
        );
        let nested_bindings = nested
            .iter()
            .map(|declaration| {
                (
                    declaration.input.value.as_str(),
                    declaration.input.layer_name.as_deref(),
                    declaration.input.layer_order,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            nested_bindings,
            vec![
                ("blue", Some("b"), Some(0)),
                ("purple", Some("a.b"), Some(1)),
            ]
        );
    }

    #[test]
    fn string_embedded_layer_tokens_do_not_enter_checker_bindings() {
        let declarations = collect_query_checker_cascade_declarations_with_dialect(
            r#"
.noise::before {
  content: "@layer fakeString; @layer fakeStringBlock { .fakeString {";
}
@layer reset;
@layer components {
  .card { content: "{"; color: red; }
}
"#,
            StyleDialect::Scss,
        );
        let noise = declarations
            .iter()
            .filter(|declaration| declaration.input.selector.as_str() == ".noise::before")
            .collect::<Vec<_>>();

        assert_eq!(noise.len(), 1);
        assert!(
            noise
                .iter()
                .all(|declaration| declaration.input.layer_name.is_none())
        );
        assert!(
            noise
                .iter()
                .all(|declaration| declaration.input.layer_order.is_none())
        );
        assert!(
            declarations
                .iter()
                .all(|declaration| declaration.input.layer_name.as_deref() != Some("fakeString"))
        );
    }

    #[test]
    fn mixed_case_anonymous_layer_is_not_a_condition_context() {
        let declarations = collect_query_checker_cascade_declarations_with_dialect(
            "@LaYeR { .target { color: red; } }",
            StyleDialect::Css,
        );

        assert_eq!(declarations.len(), 1);
        assert!(declarations[0].input.condition_context.is_empty());
    }
}
