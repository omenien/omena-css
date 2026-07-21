use std::collections::{BTreeMap, VecDeque};

use crate::EngineInputV2;
use omena_bridge::{OmenaBridgeParserRangeV0, StyleSemanticGraphSummaryV0};
use omena_cascade::{
    CascadeComputedValueInputV0, CascadeDeclaration, CascadeKey, CascadeLevel, CascadeValue,
    ComputedCascadeValueStatusV0, CustomPropertyEnv, CustomPropertyLeastFixedPointEntryV0,
    LayerRank, ModuleRank, Specificity, compute_cascade_computed_value,
    summarize_custom_property_least_fixed_point,
};
use omena_parser::{ParserByteSpanV0, ParserPositionV0, ParserRangeV0};
use omena_query_transform_runner::parse_static_css_cascade_value;
use omena_semantic::DesignTokenRankedReferenceV0;

use crate::{
    AbstractPropertyValueV0, CascadeContextV0, CascadeDimensionalRefinementBridgeV0,
    CascadeValueFamilyMemberV0, OmenaQueryAnalysisPrecisionV0, OmenaQueryAnalysisResultV0,
    OmenaQueryCascadeAtPositionV0, OmenaQueryEvaluationRuntimeSummaryV0,
    RefinementPropertyPredicateV0, derive_cascade_restriction_maps_v0,
    summarize_cascade_dimensional_refinement_bridge_v0, summarize_cascade_value_family_v0,
};

use super::{
    byte_offset_for_parser_position, is_css_identifier_continue, parser_range_for_byte_span,
    summarize_omena_query_style_semantic_graph_from_source,
};

pub fn read_omena_query_cascade_at_position(
    style_path: &str,
    style_source: &str,
    input: &EngineInputV2,
    position: ParserPositionV0,
) -> Option<OmenaQueryCascadeAtPositionV0> {
    let graph =
        summarize_omena_query_style_semantic_graph_from_source(style_path, style_source, input)?;
    Some(read_omena_query_cascade_at_position_from_graph(
        style_path,
        style_source,
        &graph,
        position,
    ))
}

pub fn read_omena_query_cascade_at_position_analysis_result(
    style_path: &str,
    style_source: &str,
    input: &EngineInputV2,
    position: ParserPositionV0,
    runtime_summary: &OmenaQueryEvaluationRuntimeSummaryV0,
) -> Option<OmenaQueryAnalysisResultV0<OmenaQueryCascadeAtPositionV0>> {
    let value = read_omena_query_cascade_at_position(style_path, style_source, input, position)?;
    Some(cascade_at_position_analysis_result(
        value,
        runtime_summary.expression_domain_revision,
    ))
}

fn cascade_at_position_analysis_result(
    value: OmenaQueryCascadeAtPositionV0,
    revision: u64,
) -> OmenaQueryAnalysisResultV0<OmenaQueryCascadeAtPositionV0> {
    OmenaQueryAnalysisResultV0::new(
        value,
        OmenaQueryAnalysisPrecisionV0 {
            product: "omena-query.analysis-precision".to_string(),
            value_domain: "cascadeAtPosition".to_string(),
            flow_sensitivity: "positionScopedCascade".to_string(),
            context_sensitivity: "styleSemanticGraph".to_string(),
            revision_axis: "OmenaQueryEvaluationRuntimeSummaryV0.expressionDomainRevision"
                .to_string(),
        },
        vec![
            "omena-query.read-cascade-at-position".to_string(),
            "omena-cascade.winner-resolution".to_string(),
        ],
        revision,
    )
}

pub fn read_omena_query_cascade_at_position_with_categorical_evidence(
    style_path: &str,
    style_source: &str,
    input: &EngineInputV2,
    position: ParserPositionV0,
    include_categorical_evidence: bool,
) -> Option<OmenaQueryCascadeAtPositionV0> {
    let mut result =
        read_omena_query_cascade_at_position(style_path, style_source, input, position)?;
    if include_categorical_evidence {
        let exercised_primitive_role_pairs =
            super::cascade_checker::query_exercised_cascade_primitive_role_pairs_from_source(
                style_source,
            );
        result.categorical_evidence = Some(
            omena_query_checker_orchestrator::checker_categorical_cascade_evidence_for_exercised_primitives_v0(
                "omena-query.read-cascade-at-position",
                &exercised_primitive_role_pairs,
            ),
        );
    }
    Some(result)
}

pub fn read_omena_query_cascade_at_position_from_graph(
    style_path: &str,
    style_source: &str,
    graph: &StyleSemanticGraphSummaryV0,
    position: ParserPositionV0,
) -> OmenaQueryCascadeAtPositionV0 {
    let positioned_references = positioned_custom_property_reference_facts(
        style_source,
        graph
            .parser_facts
            .custom_properties
            .ref_facts
            .iter()
            .map(|fact| CustomPropertyReferenceFactView {
                name: fact.name.as_str(),
                source_order: fact.source_order,
            }),
    );
    let reference = positioned_references
        .iter()
        .find(|(_, range)| parser_range_contains_position(range, position));

    let Some((reference, reference_range)) = reference else {
        let custom_property_env = collect_same_file_custom_property_env_from_graph(graph);
        let fixed_point = summarize_custom_property_least_fixed_point(&custom_property_env);
        return OmenaQueryCascadeAtPositionV0 {
            schema_version: "0",
            product: "omena-query.read-cascade-at-position",
            style_path: style_path.to_string(),
            query_position: position,
            status: "noCustomPropertyReference",
            cascade_engine: "omena-cascade",
            reference_name: None,
            reference_range: None,
            winner_declaration_source_order: None,
            winner_declaration_file_path: None,
            winner_declaration_range: None,
            winner_context_kind: None,
            winner_declaration_layer_rank: None,
            winner_declaration_layer_name: None,
            candidate_declaration_count: 0,
            shadowed_declaration_source_orders: Vec::new(),
            referenced_declaration_property: None,
            referenced_declaration_value: None,
            referenced_declaration_computed_value_status: None,
            referenced_declaration_computed_value: None,
            referenced_declaration_invalid_at_computed_value_time: false,
            referenced_declaration_computed_value_derivation_steps: Vec::new(),
            custom_property_fixed_point_iteration_count: fixed_point.iteration_count,
            custom_property_fixed_point_guaranteed_invalid_count: fixed_point
                .guaranteed_invalid_count,
            reference_custom_property_fixed_point_status: None,
            reference_custom_property_fixed_point_value: None,
            refinement_evidence: None,
            categorical_evidence: None,
        };
    };

    let ranking = graph
        .design_token_semantics
        .cascade_ranking_signal
        .ranked_references
        .iter()
        .find(|ranking| {
            ranking.reference_name == reference.name
                && ranking.reference_source_order == reference.source_order
        });
    let custom_property_env = collect_same_file_custom_property_env_from_graph_for_reference_winner(
        graph,
        reference.name,
        ranking.map(|ranking| ranking.winner_declaration_source_order),
    );
    let fixed_point = summarize_custom_property_least_fixed_point(&custom_property_env);
    let computed = compute_referenced_declaration_cascade_value_seed(
        style_path,
        style_source,
        *reference_range,
        &custom_property_env,
    );
    let fixed_point_entry = fixed_point
        .entries
        .iter()
        .find(|entry| entry.name == reference.name);
    let fixed_point_value =
        fixed_point_entry.and_then(|entry| render_query_cascade_value(&entry.resolved));
    let refinement_evidence = fixed_point_value
        .as_deref()
        .map(|value| summarize_query_cascade_refinement_evidence(reference.name, value, ranking));

    OmenaQueryCascadeAtPositionV0 {
        schema_version: "0",
        product: "omena-query.read-cascade-at-position",
        style_path: style_path.to_string(),
        query_position: position,
        status: if ranking.is_some() {
            "resolved"
        } else {
            "unresolved"
        },
        cascade_engine: "omena-cascade",
        reference_name: Some(reference.name.to_string()),
        reference_range: Some(*reference_range),
        winner_declaration_source_order: ranking
            .map(|ranking| ranking.winner_declaration_source_order),
        winner_declaration_file_path: ranking
            .and_then(|ranking| ranking.winner_declaration_file_path.clone()),
        winner_declaration_range: ranking
            .and_then(|ranking| ranking.winner_declaration_range)
            .map(parser_range_from_semantic_range),
        winner_context_kind: ranking.map(|ranking| ranking.winner_context_kind),
        winner_declaration_layer_rank: ranking.map(|ranking| ranking.winner_declaration_layer_rank),
        winner_declaration_layer_name: ranking
            .and_then(|ranking| ranking.winner_declaration_layer_name.clone()),
        candidate_declaration_count: ranking
            .map(|ranking| ranking.candidate_declaration_count)
            .unwrap_or(0),
        shadowed_declaration_source_orders: ranking
            .map(|ranking| ranking.shadowed_declaration_source_orders.clone())
            .unwrap_or_default(),
        referenced_declaration_property: computed
            .as_ref()
            .map(|computed| computed.property.clone()),
        referenced_declaration_value: computed.as_ref().map(|computed| computed.value.clone()),
        referenced_declaration_computed_value_status: computed
            .as_ref()
            .map(|computed| computed.status),
        referenced_declaration_computed_value: computed
            .as_ref()
            .and_then(|computed| computed.computed_value.clone()),
        referenced_declaration_invalid_at_computed_value_time: computed
            .as_ref()
            .is_some_and(|computed| computed.invalid_at_computed_value_time),
        referenced_declaration_computed_value_derivation_steps: computed
            .map(|computed| computed.derivation_steps)
            .unwrap_or_default(),
        custom_property_fixed_point_iteration_count: fixed_point.iteration_count,
        custom_property_fixed_point_guaranteed_invalid_count: fixed_point.guaranteed_invalid_count,
        reference_custom_property_fixed_point_status: fixed_point_entry
            .map(query_custom_property_fixed_point_entry_status),
        reference_custom_property_fixed_point_value: fixed_point_value,
        refinement_evidence,
        categorical_evidence: None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReferencedDeclarationComputedValueSeed {
    property: String,
    value: String,
    status: &'static str,
    computed_value: Option<String>,
    invalid_at_computed_value_time: bool,
    derivation_steps: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StyleDeclarationAtOffset {
    property: String,
    value: String,
    source_order: usize,
}

fn compute_referenced_declaration_cascade_value_seed(
    style_path: &str,
    style_source: &str,
    reference_range: ParserRangeV0,
    custom_property_env: &CustomPropertyEnv,
) -> Option<ReferencedDeclarationComputedValueSeed> {
    let reference_offset = byte_offset_for_parser_position(style_source, reference_range.start)?;
    let declaration = style_declaration_at_byte_offset(style_source, reference_offset)?;
    let cascade_value = parse_static_css_cascade_value(&declaration.value)?;
    let result = compute_cascade_computed_value(CascadeComputedValueInputV0 {
        property: declaration.property.clone(),
        declarations: vec![CascadeDeclaration {
            id: format!(
                "{style_path}:{}:{}",
                declaration.property, declaration.source_order
            ),
            property: declaration.property.clone(),
            value: cascade_value,
            key: CascadeKey::new(
                CascadeLevel::AuthorNormal,
                LayerRank(0),
                0,
                Specificity::ZERO,
                ModuleRank::ZERO,
                declaration.source_order.min(u32::MAX as usize) as u32,
            ),
            specificity_exactness: omena_cascade::SpecificityExactnessV0::Exact,
        }],
        custom_property_env: custom_property_env.clone(),
        parent_computed_value: None,
        registered_custom_property: None,
    });

    Some(ReferencedDeclarationComputedValueSeed {
        property: declaration.property,
        value: declaration.value,
        status: query_computed_cascade_value_status(&result.status),
        computed_value: render_query_cascade_value(&result.value),
        invalid_at_computed_value_time: result.invalid_at_computed_value_time,
        derivation_steps: result.derivation_steps,
    })
}

fn query_custom_property_fixed_point_entry_status(
    entry: &CustomPropertyLeastFixedPointEntryV0,
) -> &'static str {
    if entry.guaranteed_invalid {
        "guaranteedInvalid"
    } else if entry.changed {
        "resolvedByLeastFixedPoint"
    } else {
        "fixedPointStable"
    }
}

fn summarize_query_cascade_refinement_evidence(
    reference_name: &str,
    fixed_point_value: &str,
    ranking: Option<&DesignTokenRankedReferenceV0>,
) -> CascadeDimensionalRefinementBridgeV0 {
    let layers = ranking
        .and_then(|ranking| ranking.winner_declaration_layer_name.clone())
        .into_iter()
        .collect::<Vec<_>>();
    let context = CascadeContextV0 {
        id: format!("custom-property-fixed-point:{reference_name}"),
        parent_id: None,
        selectors: Vec::new(),
        conditions: Vec::new(),
        layers,
    };
    let members = vec![CascadeValueFamilyMemberV0 {
        context,
        value: AbstractPropertyValueV0::Exact {
            property_name: reference_name.to_string(),
            value: fixed_point_value.to_string(),
            pseudo_state: None,
        },
    }];
    let restrictions = derive_cascade_restriction_maps_v0(members.as_slice());
    let family = summarize_cascade_value_family_v0(reference_name, members, restrictions);
    let predicate = RefinementPropertyPredicateV0::Not {
        predicate: Box::new(RefinementPropertyPredicateV0::ExactValue {
            property_name: reference_name.to_string(),
            value: "guaranteed-invalid".to_string(),
        }),
    };
    summarize_cascade_dimensional_refinement_bridge_v0(&family, &[predicate])
}

fn collect_same_file_custom_property_env_from_graph(
    graph: &StyleSemanticGraphSummaryV0,
) -> CustomPropertyEnv {
    let mut latest_values = BTreeMap::<String, (usize, CascadeValue)>::new();
    for declaration in &graph.parser_facts.custom_properties.decl_facts {
        let Some(value) = parse_static_css_cascade_value(&declaration.value) else {
            continue;
        };
        let entry = latest_values
            .entry(declaration.name.clone())
            .or_insert((declaration.source_order, value.clone()));
        if declaration.source_order >= entry.0 {
            *entry = (declaration.source_order, value);
        }
    }
    latest_values
        .into_iter()
        .map(|(name, (_, value))| (name, value))
        .collect()
}

fn collect_same_file_custom_property_env_from_graph_for_reference_winner(
    graph: &StyleSemanticGraphSummaryV0,
    reference_name: &str,
    winner_declaration_source_order: Option<usize>,
) -> CustomPropertyEnv {
    let mut env = collect_same_file_custom_property_env_from_graph(graph);
    let Some(winner_declaration_source_order) = winner_declaration_source_order else {
        return env;
    };
    let Some(winner) = graph
        .parser_facts
        .custom_properties
        .decl_facts
        .iter()
        .find(|declaration| {
            declaration.name == reference_name
                && declaration.source_order == winner_declaration_source_order
        })
    else {
        return env;
    };
    let Some(value) = parse_static_css_cascade_value(&winner.value) else {
        return env;
    };
    env.insert(reference_name.to_string(), value);
    env
}

fn style_declaration_at_byte_offset(
    source: &str,
    offset: usize,
) -> Option<StyleDeclarationAtOffset> {
    let start = source
        .get(..offset)?
        .rfind(['{', ';'])
        .map(|index| index + 1)
        .unwrap_or(0);
    let end = source
        .get(offset..)?
        .find([';', '}'])
        .map(|index| offset + index)
        .unwrap_or(source.len());
    if start >= end {
        return None;
    }

    let statement = source.get(start..end)?.trim();
    let colon = statement.find(':')?;
    let property = statement.get(..colon)?.trim();
    let value = statement.get(colon + 1..)?.trim();
    if property.is_empty() || value.is_empty() {
        return None;
    }

    Some(StyleDeclarationAtOffset {
        property: property.to_string(),
        value: value.to_string(),
        source_order: source.get(..start).unwrap_or_default().matches(';').count(),
    })
}

fn query_computed_cascade_value_status(status: &ComputedCascadeValueStatusV0) -> &'static str {
    match status {
        ComputedCascadeValueStatusV0::Resolved => "resolved",
        ComputedCascadeValueStatusV0::Inherited => "inherited",
        ComputedCascadeValueStatusV0::Initial => "initial",
        ComputedCascadeValueStatusV0::InvalidAtComputedValueTime => "invalidAtComputedValueTime",
    }
}

fn render_query_cascade_value(value: &CascadeValue) -> Option<String> {
    match value {
        CascadeValue::Literal(value) => Some(value.clone()),
        CascadeValue::Composite(parts) => {
            let mut output = String::new();
            for part in parts {
                output.push_str(&render_query_cascade_value(part)?);
            }
            Some(output)
        }
        CascadeValue::Initial => Some("initial".to_string()),
        CascadeValue::Inherit => Some("inherit".to_string()),
        CascadeValue::GuaranteedInvalid => Some("guaranteed-invalid".to_string()),
        CascadeValue::Unset => Some("unset".to_string()),
        CascadeValue::Var { .. } => None,
    }
}

fn custom_property_ref_byte_spans(source: &str, name: &str) -> Vec<ParserByteSpanV0> {
    let mut spans = Vec::new();
    let mut search_offset = 0usize;

    while let Some(relative_match) = source[search_offset..].find(name) {
        let name_start = search_offset + relative_match;
        let name_end = name_start + name.len();
        if source[..name_start].trim_end().ends_with("var(")
            && is_selector_name_boundary(source, name_end)
        {
            spans.push(ParserByteSpanV0 {
                start: name_start,
                end: name_end,
            });
        }
        search_offset += relative_match + name.len();
    }

    spans
}

#[derive(Debug, Clone, Copy)]
struct CustomPropertyReferenceFactView<'a> {
    name: &'a str,
    source_order: usize,
}

fn positioned_custom_property_reference_facts<'a>(
    source: &str,
    ref_facts: impl IntoIterator<Item = CustomPropertyReferenceFactView<'a>>,
) -> Vec<(CustomPropertyReferenceFactView<'a>, ParserRangeV0)> {
    let ref_facts = ref_facts.into_iter().collect::<Vec<_>>();
    let mut ranges_by_name = BTreeMap::<&str, VecDeque<ParserRangeV0>>::new();
    for name in ref_facts
        .iter()
        .map(|fact| fact.name)
        .collect::<std::collections::BTreeSet<_>>()
    {
        ranges_by_name.insert(
            name,
            custom_property_ref_byte_spans(source, name)
                .into_iter()
                .map(|span| parser_range_for_byte_span(source, span))
                .collect(),
        );
    }

    let mut ordered_ref_facts = ref_facts;
    ordered_ref_facts.sort_by_key(|fact| fact.source_order);
    ordered_ref_facts
        .into_iter()
        .filter_map(|fact| {
            ranges_by_name
                .get_mut(fact.name)
                .and_then(VecDeque::pop_front)
                .map(|range| (fact, range))
        })
        .collect()
}

fn is_selector_name_boundary(source: &str, byte_offset: usize) -> bool {
    source[byte_offset..]
        .chars()
        .next()
        .is_none_or(|ch| !is_css_identifier_continue(ch))
}

fn parser_range_from_semantic_range(range: OmenaBridgeParserRangeV0) -> ParserRangeV0 {
    ParserRangeV0 {
        start: ParserPositionV0 {
            line: range.start.line,
            character: range.start.character,
        },
        end: ParserPositionV0 {
            line: range.end.line,
            character: range.end.character,
        },
    }
}

fn parser_range_contains_position(range: &ParserRangeV0, position: ParserPositionV0) -> bool {
    parser_position_is_after_or_equal(position, range.start)
        && parser_position_is_before(position, range.end)
}

fn parser_position_is_after_or_equal(position: ParserPositionV0, start: ParserPositionV0) -> bool {
    position.line > start.line
        || (position.line == start.line && position.character >= start.character)
}

fn parser_position_is_before(position: ParserPositionV0, end: ParserPositionV0) -> bool {
    position.line < end.line || (position.line == end.line && position.character < end.character)
}
