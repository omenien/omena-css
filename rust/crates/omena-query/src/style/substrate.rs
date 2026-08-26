use super::*;
use omena_parser::ParsedStyleFacts;
use omena_syntax::ident::CanonicalCustomPropertyNameV0;
use std::collections::{BTreeMap, BTreeSet};

pub const OMENA_QUERY_FRAGILE_GUARDED_WINNER_THRESHOLD_V0: u32 = 1;

pub fn summarize_omena_query_fast_facts(style_path: &str, style_source: &str) -> FastFactsV0 {
    let dialect = omena_parser_dialect_for_style_path(style_path);
    let facts = collect_omena_query_omena_parser_style_facts_raw(style_source, dialect);
    fast_facts_from_collected_style_facts(style_path, dialect, &facts)
}

fn fast_facts_from_collected_style_facts(
    style_path: &str,
    dialect: OmenaParserStyleDialect,
    facts: &ParsedStyleFacts,
) -> FastFactsV0 {
    let custom_property_count = facts
        .variables
        .iter()
        .filter(|fact| {
            matches!(
                fact.kind,
                ParsedVariableFactKind::CustomPropertyDeclaration
                    | ParsedVariableFactKind::CustomPropertyReference
            )
        })
        .count();

    FastFactsV0 {
        schema_version: "0",
        product: "omena-query.fast-facts",
        tier: "fastFactsV0",
        style_path: style_path.to_string(),
        language: omena_parser_style_dialect_label(dialect),
        selector_count: facts.selectors.len(),
        custom_property_count,
        sass_symbol_count: facts.sass_symbols.len(),
        module_edge_count: facts.sass_module_edges.len(),
        parser_error_count: facts.error_count,
    }
}

pub fn summarize_omena_query_analyzed_graph(
    style_path: &str,
    style_source: &str,
) -> AnalyzedGraphV0 {
    let dialect = omena_parser_dialect_for_style_path(style_path);
    let facts = collect_omena_query_omena_parser_style_facts_raw(style_source, dialect);
    let fast_facts = fast_facts_from_collected_style_facts(style_path, dialect, &facts);
    let edge_count = facts.css_module_composes_edges.len()
        + facts.css_module_value_import_edges.len()
        + facts.css_module_value_definition_edges.len()
        + facts.icss_import_edges.len()
        + facts.icss_export_edges.len()
        + facts.sass_module_edges.len()
        + facts.animations.len();

    AnalyzedGraphV0 {
        schema_version: "0",
        product: "omena-query.analyzed-graph",
        tier: "analyzedGraphV0",
        style_path: style_path.to_string(),
        node_count: facts.selectors.len() + facts.variables.len() + facts.sass_symbols.len(),
        edge_count,
        cycle_count: 0,
        graph_kinds: vec![
            "selectorFacts",
            "customPropertyFacts",
            "cssModulesFacts",
            "sassModuleFacts",
        ],
        fast_facts,
    }
}

pub fn summarize_omena_query_style_edit_distance(
    left_style_path: &str,
    left_style_source: &str,
    right_style_path: &str,
    right_style_source: &str,
) -> StyleEditDistanceSummaryV0 {
    let left_analyzed_graph =
        summarize_omena_query_analyzed_graph(left_style_path, left_style_source);
    let right_analyzed_graph =
        summarize_omena_query_analyzed_graph(right_style_path, right_style_source);
    let left_fast_facts = left_analyzed_graph.fast_facts.clone();
    let right_fast_facts = right_analyzed_graph.fast_facts.clone();

    let selector_delta = absolute_count_delta(
        left_fast_facts.selector_count,
        right_fast_facts.selector_count,
    );
    let custom_property_delta = absolute_count_delta(
        left_fast_facts.custom_property_count,
        right_fast_facts.custom_property_count,
    );
    let sass_symbol_delta = absolute_count_delta(
        left_fast_facts.sass_symbol_count,
        right_fast_facts.sass_symbol_count,
    );
    let module_edge_delta = absolute_count_delta(
        left_fast_facts.module_edge_count,
        right_fast_facts.module_edge_count,
    );
    let parser_error_delta = absolute_count_delta(
        left_fast_facts.parser_error_count,
        right_fast_facts.parser_error_count,
    );
    let graph_node_delta = absolute_count_delta(
        left_analyzed_graph.node_count,
        right_analyzed_graph.node_count,
    );
    let graph_edge_delta = absolute_count_delta(
        left_analyzed_graph.edge_count,
        right_analyzed_graph.edge_count,
    );
    let graph_cycle_delta = absolute_count_delta(
        left_analyzed_graph.cycle_count,
        right_analyzed_graph.cycle_count,
    );
    let total_distance = selector_delta
        + custom_property_delta
        + sass_symbol_delta
        + module_edge_delta
        + parser_error_delta
        + graph_node_delta
        + graph_edge_delta
        + graph_cycle_delta;

    StyleEditDistanceSummaryV0 {
        schema_version: "0",
        product: "omena-query.style-edit-distance",
        tier: "fastFactsAnalyzedGraphEditDistanceV0",
        metric_kind: "absoluteCountDeltaOverFastFactsAndAnalyzedGraph",
        claim_level: "researchStagedMetricSubstrate",
        public_safety_claim_ready: false,
        left_style_path: left_style_path.to_string(),
        right_style_path: right_style_path.to_string(),
        left_fast_facts,
        right_fast_facts,
        left_analyzed_graph,
        right_analyzed_graph,
        selector_delta,
        custom_property_delta,
        sass_symbol_delta,
        module_edge_delta,
        parser_error_delta,
        graph_node_delta,
        graph_edge_delta,
        graph_cycle_delta,
        total_distance,
    }
}

pub fn summarize_omena_query_style_edit_distance_cascade_margin_bridge(
    edit_distance: &StyleEditDistanceSummaryV0,
    cascade_margin: &omena_cascade::CascadeMarginV0,
) -> StyleEditDistanceCascadeMarginBridgeV0 {
    let edit_distance_total = edit_distance.total_distance as u64;
    let cascade_margin_abs_distance = cascade_margin.signed_distance.unsigned_abs();
    let lipschitz_constant = if cascade_margin_abs_distance == 0 {
        Some(0)
    } else if edit_distance_total == 0 {
        None
    } else {
        Some(cascade_margin_abs_distance.div_ceil(edit_distance_total))
    };
    let lipschitz_bound =
        lipschitz_constant.map(|constant| constant.saturating_mul(edit_distance_total));
    let checked = lipschitz_bound
        .map(|bound| cascade_margin_abs_distance <= bound)
        .unwrap_or(false);
    let calibration_stage = "fixtureWitnessOnlyUncalibrated";

    StyleEditDistanceCascadeMarginBridgeV0 {
        schema_version: "0",
        product: "omena-query.style-edit-distance-cascade-margin-bridge",
        bridge_kind: "checkedEmpiricalLipschitzWitness",
        claim_level: "fixtureWitnessOnly",
        theorem_claimed: false,
        public_safety_claim_ready: false,
        metric_product: edit_distance.product,
        metric_kind: edit_distance.metric_kind,
        margin_product: cascade_margin.product,
        margin_kind: cascade_margin.margin_kind,
        dominant_axis: cascade_margin.dominant_axis,
        edit_distance_total: edit_distance.total_distance,
        cascade_margin_signed_distance: cascade_margin.signed_distance,
        cascade_margin_abs_distance,
        lipschitz_constant_name: "K_A",
        lipschitz_constant,
        lipschitz_bound,
        checked,
        calibration_stage,
        incremental_priority_input: IncrementalEditDistancePriorityInputV0 {
            schema_version: "0",
            product: "omena-incremental.edit-distance-priority-input",
            feature_gate: "incremental-edit-distance-priority-v0",
            claim_level: "fixtureWitnessMetricInput",
            theorem_claimed: false,
            node_id: edit_distance.right_style_path.clone(),
            edit_distance_total: edit_distance.total_distance,
            cascade_margin_abs_distance,
            bridge_checked: checked,
            bridge_calibration_stage: calibration_stage,
        },
    }
}

pub fn summarize_omena_query_fragile_guarded_winner_v0(
    robustness: &omena_cascade::GuardedCascadeRobustnessRadiusV0,
    baseline_winner_declaration_id: impl Into<String>,
    fragile_threshold: u32,
) -> Option<OmenaQueryFragileGuardedWinnerDiagnosticV0> {
    let omena_cascade::GuardedCascadeRobustnessRadiusValueV0::Finite(radius) = robustness.radius
    else {
        return None;
    };
    if radius > fragile_threshold {
        return None;
    }

    let witness_name = robustness
        .witness
        .first()
        .map(|perturbation| format!("{:?}", perturbation.kind()))
        .unwrap_or_else(|| "unknownPerturbation".to_string());
    let baseline_winner_declaration_id = baseline_winner_declaration_id.into();
    Some(OmenaQueryFragileGuardedWinnerDiagnosticV0 {
        schema_version: "0",
        product: "omena-query.fragile-guarded-winner-diagnostic",
        diagnostic_kind: "fragileGuardedCascadeWinner",
        claim_level: "declaredAlphabetUncalibrated",
        baseline_winner_declaration_id: baseline_winner_declaration_id.clone(),
        robustness_radius: radius,
        fragile_threshold,
        witness: robustness.witness.clone(),
        calibration_stage: robustness.calibration_stage,
        public_safety_claim_ready: robustness.public_safety_claim_ready,
        false_alarm_boundary: "realisabilityIsCheckedOnlyForSameUnitWidthBoundsInsideTheDeclaredGuardedCascadeFragment",
        message: format!(
            "guarded cascade winner declaration {} is fragile at declared cost {radius} via {witness_name}",
            baseline_winner_declaration_id
        ),
    })
}

pub fn summarize_omena_query_custom_property_annotations(
    style_path: &str,
    style_source: &str,
) -> OmenaQueryCustomPropertyAnnotationSummaryV0 {
    let dialect = omena_parser_dialect_for_style_path(style_path);
    let facts = collect_omena_query_omena_parser_style_facts_raw(style_source, dialect);
    let mut declarations_by_name =
        BTreeMap::<CanonicalCustomPropertyNameV0, (AuthoredPropertyTextV0, usize)>::new();
    let mut references_by_name =
        BTreeMap::<CanonicalCustomPropertyNameV0, (AuthoredPropertyTextV0, usize)>::new();

    for fact in facts.variables {
        let Some(property_key) = fact.property_key else {
            continue;
        };
        match fact.kind {
            ParsedVariableFactKind::CustomPropertyDeclaration => {
                let Some(name) = fact.name.as_custom_property().cloned() else {
                    continue;
                };
                declarations_by_name
                    .entry(property_key)
                    .and_modify(|entry| entry.1 += 1)
                    .or_insert((name, 1));
            }
            ParsedVariableFactKind::CustomPropertyReference => {
                let Some(name) = fact.name.as_custom_property().cloned() else {
                    continue;
                };
                references_by_name
                    .entry(property_key)
                    .and_modify(|entry| entry.1 += 1)
                    .or_insert((name, 1));
            }
            _ => {}
        }
    }

    let names = declarations_by_name
        .keys()
        .chain(references_by_name.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let annotations = names
        .into_iter()
        .map(|property_key| {
            let declaration = declarations_by_name.get(&property_key);
            let reference = references_by_name.get(&property_key);
            let declaration_count = declaration.map(|entry| entry.1).unwrap_or(0);
            let reference_count = reference.map(|entry| entry.1).unwrap_or(0);
            OmenaQueryCustomPropertyAnnotationV0 {
                name: declaration
                    .or(reference)
                    .map(|entry| entry.0.clone())
                    .unwrap_or_else(|| AuthoredPropertyTextV0::new(property_key.as_str())),
                property_key,
                declaration_count,
                reference_count,
                annotation_kind: match (declaration_count > 0, reference_count > 0) {
                    (true, true) => "declarationAndReference",
                    (true, false) => "declaration",
                    (false, true) => "reference",
                    (false, false) => "empty",
                },
                participates_in_fixed_point: declaration_count > 0 && reference_count > 0,
            }
        })
        .collect::<Vec<_>>();

    OmenaQueryCustomPropertyAnnotationSummaryV0 {
        schema_version: "0",
        product: "omena-query.custom-property-annotations",
        style_path: style_path.to_string(),
        annotation_count: annotations.len(),
        annotations,
    }
}

fn absolute_count_delta(left: usize, right: usize) -> usize {
    left.abs_diff(right)
}

#[cfg(test)]
mod fragile_guarded_winner_tests {
    use omena_cascade::{
        CascadeKey, CascadeLevel, GuardedCascadeCandidateV0, GuardedCascadeConditionAtomV0,
        GuardedCascadeFragmentV0, GuardedCascadeSpecificityExactnessV0, LayerOrdinal, Specificity,
        compute_guarded_cascade_robustness_radius_v0, guarded_cascade_perturbation_cost_model_v0,
        normalized_layer_rank,
    };
    use omena_syntax::ident::AuthoredPropertyTextV0;

    use super::summarize_omena_query_fragile_guarded_winner_v0;

    fn candidate(
        declaration_id: u32,
        source_order: u32,
        condition: Option<&str>,
    ) -> GuardedCascadeCandidateV0<CascadeKey> {
        GuardedCascadeCandidateV0::new(
            declaration_id,
            ".a",
            AuthoredPropertyTextV0::new("color"),
            CascadeKey::new(
                CascadeLevel::AuthorNormal,
                normalized_layer_rank(false, LayerOrdinal::new(0)),
                0,
                Specificity::new(0, 1, 0),
                source_order,
            ),
            GuardedCascadeSpecificityExactnessV0::Exact,
            0,
            condition
                .map(|condition| vec![GuardedCascadeConditionAtomV0::media(condition, [0], true)])
                .unwrap_or_default(),
        )
    }

    #[test]
    fn fragile_diagnostic_consumes_computed_radius_without_promoting_calibration()
    -> Result<(), Box<dyn std::error::Error>> {
        let condition = "@media (min-width: 1px)";
        let fragment = GuardedCascadeFragmentV0::admit(
            [condition],
            [candidate(0, 0, None), candidate(1, 1, Some(condition))],
        )?;
        let radius = compute_guarded_cascade_robustness_radius_v0(
            &fragment,
            &[false],
            &guarded_cascade_perturbation_cost_model_v0(),
        )?;
        let diagnostic = summarize_omena_query_fragile_guarded_winner_v0(&radius, "base", 1)
            .ok_or("expected finite radius to produce a fragile-winner diagnostic")?;

        assert_eq!(diagnostic.robustness_radius, 1);
        assert_eq!(diagnostic.baseline_winner_declaration_id, "base");
        assert_eq!(diagnostic.witness, radius.witness);
        assert_eq!(diagnostic.calibration_stage, "schemaOnlyUncalibrated");
        assert!(!diagnostic.public_safety_claim_ready);
        assert!(
            diagnostic
                .false_alarm_boundary
                .contains("SameUnitWidthBounds")
        );
        Ok(())
    }
}
