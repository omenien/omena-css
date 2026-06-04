use std::collections::{BTreeMap, BTreeSet};

use super::cascade_checker::{
    QueryCheckerCascadeDeclaration, collect_query_checker_cascade_declarations,
    query_smt_box_shorthand_longhand_quartets,
};
use super::*;

pub fn summarize_omena_query_style_insights(
    style_uri: &str,
    source: &str,
) -> OmenaQueryStyleInsightsV0 {
    let mut insights = collect_shorthand_combinable_insights(style_uri, source);
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
        ready_surfaces: vec!["styleInsightSurface", "shorthandCombinableInsights"],
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
        .map(|insight| OmenaQueryCodeActionV0 {
            title: insight.title,
            kind: "quickfix",
            edits: vec![insight.primary_edit],
            source: "omenaQueryStyleInsightCodeActions",
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

fn collect_shorthand_combinable_insights(
    style_uri: &str,
    source: &str,
) -> Vec<OmenaQueryInsightV0> {
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
        for (shorthand, expected_longhands) in query_smt_box_shorthand_longhand_quartets() {
            let Some(quartet) = shorthand_quartet_for_selector(
                selector_declarations.as_slice(),
                expected_longhands,
            ) else {
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
                source: "omenaQueryStyleInsights",
                provenance: vec![
                    "omena-query.style-insights",
                    "omena-query.cascade-checker-declarations",
                    "omena-query.shorthand-combinable",
                ],
                primary_edit: OmenaQueryWorkspaceTextEditV0 {
                    uri: style_uri.to_string(),
                    range,
                    new_text: format!("{shorthand}: {combined_value}"),
                },
                shorthand_combinable: Some(OmenaQueryShorthandCombinableV0 {
                    shorthand_property: shorthand.to_string(),
                    longhand_properties,
                    values,
                    combined_value,
                    declaration_count: quartet.len(),
                }),
            });
        }
    }
    insights
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
