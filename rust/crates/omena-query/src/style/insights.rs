use std::collections::{BTreeMap, BTreeSet};

use super::cascade_checker::{
    QueryCheckerCascadeDeclaration, collect_query_checker_cascade_declarations,
    query_smt_box_shorthand_longhand_quartets,
};
use super::*;
use omena_cascade::{
    SelectorMatchVerdict, parse_simple_selector_signature, selector_co_match_verdict,
};

pub fn summarize_omena_query_style_insights(
    style_uri: &str,
    source: &str,
) -> OmenaQueryStyleInsightsV0 {
    let mut insights = collect_cascade_style_insights(style_uri, source);
    insights.sort_by_key(|insight| {
        (
            insight.range.start.line,
            insight.range.start.character,
            insight.title.clone(),
        )
    });

    OmenaQueryStyleInsightsV0 {
        schema_version: "0",
        product: "omena-query.style-insights",
        style_uri: style_uri.to_string(),
        insight_count: insights.len(),
        insights,
        ready_surfaces: vec![
            "styleInsightSurface",
            "shorthandCombinableInsights",
            "cascadeRelationshipInsights",
            "insightConfidenceScope",
        ],
    }
}

pub fn summarize_omena_query_style_insight_code_actions(
    style_uri: &str,
    source: &str,
    range: ParserRangeV0,
) -> OmenaQueryCodeActionPlanV0 {
    let actions = summarize_omena_query_style_insights(style_uri, source)
        .insights
        .into_iter()
        .filter(|insight| insight_matches_selection(source, insight.range, range))
        .filter_map(|insight| {
            let primary_edit = insight.primary_edit?;
            Some(OmenaQueryCodeActionV0 {
                title: insight.title,
                kind: "quickfix",
                edits: vec![primary_edit],
                source: "omenaQueryStyleInsightCodeActions",
            })
        })
        .collect::<Vec<_>>();

    OmenaQueryCodeActionPlanV0 {
        schema_version: "0",
        product: "omena-query.code-actions",
        file_uri: style_uri.to_string(),
        file_kind: "style",
        action_count: actions.len(),
        actions,
        ready_surfaces: vec![
            "styleInsightCodeActions",
            "styleInsightSurface",
            "productFacingCodeActions",
        ],
    }
}

fn collect_cascade_style_insights(style_uri: &str, source: &str) -> Vec<OmenaQueryInsightV0> {
    let declarations = collect_query_checker_cascade_declarations(source);
    let mut by_selector = BTreeMap::<&str, Vec<&QueryCheckerCascadeDeclaration>>::new();
    for declaration in &declarations {
        by_selector
            .entry(declaration.input.selector.as_str())
            .or_default()
            .push(declaration);
    }

    let mut insights = Vec::new();
    let mut emitted_edits = BTreeSet::new();
    for (selector, selector_declarations) in by_selector {
        let selector_declarations = selector_declarations.as_slice();
        insights.extend(collect_shorthand_combinable_insights(
            style_uri,
            source,
            selector,
            selector_declarations,
            &mut emitted_edits,
        ));
        insights.extend(collect_partial_shorthand_override_insights(
            style_uri,
            source,
            selector,
            selector_declarations,
        ));
        insights.extend(collect_longhand_redundant_insights(
            style_uri,
            source,
            selector,
            selector_declarations,
        ));
    }
    insights.extend(collect_specificity_tie_insights(
        style_uri,
        source,
        declarations.as_slice(),
    ));
    insights
}

fn collect_shorthand_combinable_insights(
    style_uri: &str,
    source: &str,
    selector: &str,
    selector_declarations: &[&QueryCheckerCascadeDeclaration],
    emitted_edits: &mut BTreeSet<(usize, usize, &'static str)>,
) -> Vec<OmenaQueryInsightV0> {
    let mut insights = Vec::new();
    for (shorthand, expected_longhands) in query_smt_box_shorthand_longhand_quartets() {
        let Some(quartet) =
            shorthand_quartet_for_selector(selector_declarations, expected_longhands)
        else {
            continue;
        };
        if !shorthand_quartet_is_combinable(quartet.as_slice()) {
            continue;
        }

        let values = quartet
            .iter()
            .map(|declaration| declaration.input.value.trim().to_string())
            .collect::<Vec<_>>();
        let combined_value = values.join(" ");
        let replacement_span = ParserByteSpanV0 {
            start: quartet[0].byte_span.start,
            end: quartet[quartet.len() - 1].byte_span.end,
        };
        if !emitted_edits.insert((replacement_span.start, replacement_span.end, shorthand)) {
            continue;
        }
        let range = parser_range_for_byte_span(source, replacement_span);
        let longhand_properties = expected_longhands
            .iter()
            .map(|property| (*property).to_string())
            .collect::<Vec<_>>();

        insights.push(OmenaQueryInsightV0 {
                kind: "shorthandCombinable",
                title: format!("Combine {shorthand} longhands into shorthand"),
                message: format!(
                    "The {selector} rule declares a complete adjacent {shorthand} longhand quartet that can be replaced by `{shorthand}: {combined_value}`."
                ),
                range,
                confidence: "high",
                scope: "singleSelector",
                source: "omenaQueryStyleInsights",
                provenance: omena_query_evidence_graph_provenance![
                    "omena-query.style-insights",
                    "omena-query.cascade-checker-declarations",
                    "omena-query.shorthand-combinable",
                ],
                primary_edit: Some(OmenaQueryWorkspaceTextEditV0 {
                    uri: style_uri.to_string(),
                    range,
                    new_text: format!("{shorthand}: {combined_value}"),
                }),
                shorthand_combinable: Some(OmenaQueryShorthandCombinableV0 {
                    shorthand_property: shorthand.to_string(),
                    longhand_properties,
                    values,
                    combined_value,
                    declaration_count: quartet.len(),
                }),
                cascade_insight: Some(OmenaQueryCascadeInsightV0 {
                    relationship: "replaceLonghandQuartetWithShorthand",
                    selector: selector.to_string(),
                    property: shorthand.to_string(),
                    related_selector: None,
                    related_property: None,
                    source_order: quartet[0].input.source_order,
                    related_source_order: Some(quartet[quartet.len() - 1].input.source_order),
                }),
            });
    }
    insights
}

fn collect_partial_shorthand_override_insights(
    _style_uri: &str,
    source: &str,
    selector: &str,
    selector_declarations: &[&QueryCheckerCascadeDeclaration],
) -> Vec<OmenaQueryInsightV0> {
    let mut insights = Vec::new();
    let mut emitted = BTreeSet::new();
    for declaration in selector_declarations {
        for (shorthand, longhands) in query_smt_box_shorthand_longhand_quartets() {
            if declaration.input.property != shorthand {
                continue;
            }
            for override_declaration in selector_declarations.iter().copied().filter(|candidate| {
                candidate.input.source_order > declaration.input.source_order
                    && longhands.contains(&candidate.input.property.as_str())
                    && same_cascade_insight_scope(declaration, candidate)
            }) {
                if !emitted.insert((
                    declaration.input.source_order,
                    override_declaration.input.source_order,
                    override_declaration.input.property.clone(),
                )) {
                    continue;
                }
                let range = parser_range_for_byte_span(source, override_declaration.byte_span);
                insights.push(OmenaQueryInsightV0 {
                    kind: "partialShorthandOverride",
                    title: format!(
                        "{} overrides earlier {shorthand}",
                        override_declaration.input.property
                    ),
                    message: format!(
                        "The {selector} rule declares `{shorthand}` and later overrides only `{}`. This is valid CSS, but the longhand is a partial override of the earlier shorthand.",
                        override_declaration.input.property
                    ),
                    range,
                    confidence: "high",
                    scope: "singleSelector",
                    source: "omenaQueryStyleInsights",
                    provenance: omena_query_evidence_graph_provenance![
                        "omena-query.style-insights",
                        "omena-query.cascade-checker-declarations",
                        "omena-query.partial-shorthand-override",
                    ],
                    primary_edit: None,
                    shorthand_combinable: None,
                    cascade_insight: Some(OmenaQueryCascadeInsightV0 {
                        relationship: "longhandOverridesEarlierShorthand",
                        selector: selector.to_string(),
                        property: shorthand.to_string(),
                        related_selector: None,
                        related_property: Some(override_declaration.input.property.clone()),
                        source_order: declaration.input.source_order,
                        related_source_order: Some(override_declaration.input.source_order),
                    }),
                });
            }
        }
    }
    insights
}

fn collect_longhand_redundant_insights(
    _style_uri: &str,
    source: &str,
    selector: &str,
    selector_declarations: &[&QueryCheckerCascadeDeclaration],
) -> Vec<OmenaQueryInsightV0> {
    let mut insights = Vec::new();
    for pair in selector_declarations.windows(2) {
        let previous = pair[0];
        let current = pair[1];
        if previous.input.property != current.input.property
            || previous.input.value.trim() != current.input.value.trim()
            || !same_cascade_insight_scope(previous, current)
        {
            continue;
        }
        let range = parser_range_for_byte_span(source, current.byte_span);
        insights.push(OmenaQueryInsightV0 {
            kind: "longhandRedundant",
            title: format!("Remove redundant {}", current.input.property),
            message: format!(
                "The {selector} rule repeats `{}` with the same value immediately after an equivalent declaration.",
                current.input.property
            ),
            range,
            confidence: "high",
            scope: "singleSelector",
            source: "omenaQueryStyleInsights",
            provenance: omena_query_evidence_graph_provenance![
                "omena-query.style-insights",
                "omena-query.cascade-checker-declarations",
                "omena-query.longhand-redundant",
            ],
            primary_edit: None,
            shorthand_combinable: None,
            cascade_insight: Some(OmenaQueryCascadeInsightV0 {
                relationship: "duplicateSameValueDeclaration",
                selector: selector.to_string(),
                property: current.input.property.clone(),
                related_selector: None,
                related_property: Some(previous.input.property.clone()),
                source_order: current.input.source_order,
                related_source_order: Some(previous.input.source_order),
            }),
        });
    }
    insights
}

fn collect_specificity_tie_insights(
    _style_uri: &str,
    source: &str,
    declarations: &[QueryCheckerCascadeDeclaration],
) -> Vec<OmenaQueryInsightV0> {
    let mut insights = Vec::new();
    let mut emitted = BTreeSet::new();
    for (index, left) in declarations.iter().enumerate() {
        let Some(left_signature) = parse_simple_selector_signature(left.input.selector.as_str())
        else {
            continue;
        };
        for right in declarations.iter().skip(index + 1) {
            if left.input.property != right.input.property
                || left.input.selector == right.input.selector
                || !same_cascade_insight_scope(left, right)
            {
                continue;
            }
            let Some(right_signature) =
                parse_simple_selector_signature(right.input.selector.as_str())
            else {
                continue;
            };
            if left_signature.specificity != right_signature.specificity {
                continue;
            }
            let co_match = selector_co_match_verdict(
                left.input.selector.as_str(),
                right.input.selector.as_str(),
            );
            if co_match == SelectorMatchVerdict::No {
                continue;
            }
            let winner = if left.input.source_order > right.input.source_order {
                left
            } else {
                right
            };
            let challenger = if std::ptr::eq(winner, left) {
                right
            } else {
                left
            };
            if !emitted.insert((
                winner.input.source_order,
                challenger.input.source_order,
                winner.input.property.clone(),
            )) {
                continue;
            }
            let range = parser_range_for_byte_span(source, winner.byte_span);
            insights.push(OmenaQueryInsightV0 {
                kind: "specificityTie",
                title: format!("Source order decides {}", winner.input.property),
                message: format!(
                    "`{}` and `{}` have equal specificity for `{}`{}; source order decides the winning declaration.",
                    challenger.input.selector,
                    winner.input.selector,
                    winner.input.property,
                    specificity_tie_co_match_suffix(co_match)
                ),
                range,
                confidence: specificity_tie_confidence(co_match),
                scope: "crossSelectorSameStylesheet",
                source: "omenaQueryStyleInsights",
                provenance: omena_query_evidence_graph_provenance![
                    "omena-query.style-insights",
                    "omena-query.cascade-checker-declarations",
                    "omena-cascade.selector-co-match",
                    "omena-cascade.specificity",
                ],
                primary_edit: None,
                shorthand_combinable: None,
                cascade_insight: Some(OmenaQueryCascadeInsightV0 {
                    relationship: "sourceOrderDecidesEqualSpecificity",
                    selector: winner.input.selector.to_string(),
                    property: winner.input.property.clone(),
                    related_selector: Some(challenger.input.selector.to_string()),
                    related_property: Some(challenger.input.property.clone()),
                    source_order: winner.input.source_order,
                    related_source_order: Some(challenger.input.source_order),
                }),
            });
        }
    }
    insights
}

fn same_cascade_insight_scope(
    left: &QueryCheckerCascadeDeclaration,
    right: &QueryCheckerCascadeDeclaration,
) -> bool {
    left.input.important == right.input.important
        && left.input.condition_context == right.input.condition_context
        && left.input.layer_name == right.input.layer_name
        && left.input.layer_order == right.input.layer_order
}

fn specificity_tie_confidence(verdict: SelectorMatchVerdict) -> &'static str {
    match verdict {
        SelectorMatchVerdict::Yes => "high",
        SelectorMatchVerdict::Maybe => "medium",
        SelectorMatchVerdict::No => "none",
    }
}

fn specificity_tie_co_match_suffix(verdict: SelectorMatchVerdict) -> &'static str {
    match verdict {
        SelectorMatchVerdict::Yes => "",
        SelectorMatchVerdict::Maybe => " on a conservatively modeled selector pair",
        SelectorMatchVerdict::No => "",
    }
}

fn shorthand_quartet_for_selector<'a>(
    declarations: &'a [&'a QueryCheckerCascadeDeclaration],
    expected_longhands: [&'static str; 4],
) -> Option<Vec<&'a QueryCheckerCascadeDeclaration>> {
    let mut quartet = Vec::with_capacity(expected_longhands.len());
    for expected in expected_longhands {
        let declaration = declarations
            .iter()
            .copied()
            .find(|declaration| declaration.input.property == expected)?;
        quartet.push(declaration);
    }
    Some(quartet)
}

fn shorthand_quartet_is_combinable(quartet: &[&QueryCheckerCascadeDeclaration]) -> bool {
    quartet.iter().all(|declaration| {
        !declaration.input.important && !declaration.input.value.trim().is_empty()
    }) && quartet
        .windows(2)
        .all(|pair| pair[1].input.source_order == pair[0].input.source_order + 1)
}

fn insight_matches_selection(
    source: &str,
    insight_range: ParserRangeV0,
    selection: ParserRangeV0,
) -> bool {
    let Some(insight_start) = byte_offset_for_parser_position(source, insight_range.start) else {
        return false;
    };
    let Some(insight_end) = byte_offset_for_parser_position(source, insight_range.end) else {
        return false;
    };
    let Some(selection_start) = byte_offset_for_parser_position(source, selection.start) else {
        return false;
    };
    let Some(selection_end) = byte_offset_for_parser_position(source, selection.end) else {
        return false;
    };

    selection_start <= insight_end && selection_end >= insight_start
}
