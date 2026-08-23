#![allow(clippy::expect_used)]
use super::*;
use crate::{
    read_omena_query_cascade_at_position, read_omena_query_cascade_at_position_analysis_result,
    read_omena_query_cascade_at_position_with_categorical_evidence,
    summarize_omena_query_evaluation_runtime,
};

#[deprecated(
    since = "0.4.0",
    note = "legacy categorical endpoint fixture owned by omena-query maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
fn compatibility_categorical_endpoint_id_v0() -> &'static str {
    "rust/omena-categorical/verify-site-stability"
}

#[test]
fn read_cascade_at_position_is_query_owned() {
    let source = ":root { --surface: white; }\n:root { --surface: black; }\n.button { color: var(--surface); }\n";
    let cascade = read_omena_query_cascade_at_position(
        "Component.module.css",
        source,
        &sample_input(),
        ParserPositionV0 {
            line: 2,
            character: 24,
        },
    );
    assert!(cascade.is_some());
    let Some(cascade) = cascade else {
        return;
    };

    assert_eq!(cascade.product, "omena-query.read-cascade-at-position");
    assert_eq!(cascade.status, "resolved");
    assert_eq!(cascade.cascade_engine, "omena-cascade");
    assert_eq!(cascade.reference_name.as_deref(), Some("--surface"));
    assert_eq!(cascade.winner_declaration_source_order, Some(1));
    // Unlayered ranks are opaque ordering tokens, not layer-count magnitudes.
    assert_eq!(cascade.winner_declaration_layer_rank, Some(i32::MAX));
    assert_eq!(cascade.candidate_declaration_count, 2);
    assert_eq!(cascade.shadowed_declaration_source_orders, vec![0]);
    assert_eq!(
        cascade.referenced_declaration_property.as_deref(),
        Some("color")
    );
    assert_eq!(
        cascade.referenced_declaration_value.as_deref(),
        Some("var(--surface)")
    );
    assert_eq!(
        cascade.referenced_declaration_computed_value_status,
        Some("resolved")
    );
    assert_eq!(
        cascade.referenced_declaration_computed_value.as_deref(),
        Some("black")
    );
    assert!(!cascade.referenced_declaration_invalid_at_computed_value_time);
    assert_eq!(cascade.custom_property_fixed_point_iteration_count, 1);
    assert_eq!(
        cascade.custom_property_fixed_point_guaranteed_invalid_count,
        0
    );
    assert_eq!(
        cascade.reference_custom_property_fixed_point_status,
        Some("fixedPointStable")
    );
    assert_eq!(
        cascade
            .reference_custom_property_fixed_point_value
            .as_deref(),
        Some("black")
    );
    assert!(
        cascade
            .referenced_declaration_computed_value_derivation_steps
            .contains(&"computedValueResolved")
    );
    let refinement = cascade
        .refinement_evidence
        .as_ref()
        .expect("custom property fixed point refinement evidence");
    assert_eq!(
        refinement.product,
        "omena-refinement.cascade-dimensional-refinement-bridge"
    );
    assert_eq!(
        refinement.claim_level,
        "m6DimensionalRefinementBridgeSubstrate"
    );
    assert_eq!(refinement.property_name, "--surface");
    assert_eq!(refinement.predicate_count, 1);
    assert_eq!(refinement.satisfied_all_context_count, 1);
    assert_eq!(refinement.unsatisfiable_context_count, 0);
    assert!(refinement.product_path_evidence_ready);
    assert!(cascade.categorical_evidence.is_none());

    let no_reference = read_omena_query_cascade_at_position(
        "Component.module.css",
        source,
        &sample_input(),
        ParserPositionV0 {
            line: 0,
            character: 1,
        },
    );
    assert!(no_reference.is_some());
    assert_eq!(
        no_reference.map(|cascade| cascade.status),
        Some("noCustomPropertyReference")
    );
}

#[test]
fn read_cascade_at_position_analysis_result_carries_revision_aligned_precision() {
    let first_source = ":root { --surface: white; }\n:root { --surface: black; }\n.button { color: var(--surface); }\n";
    let second_source = ":root { --surface: white; }\n:root { --surface: blue; }\n.button { color: var(--surface); }\n";
    let input = sample_input();
    let mut edited_input = sample_input();
    edited_input.type_facts[0].facts.suffix = Some("-primary".to_string());
    let mut runtime = OmenaQueryExpressionDomainFlowRuntimeV0::default();
    let first_runtime = summarize_omena_query_evaluation_runtime(&input, &mut runtime);
    let second_runtime = summarize_omena_query_evaluation_runtime(&edited_input, &mut runtime);
    let first = read_omena_query_cascade_at_position_analysis_result(
        "Component.module.css",
        first_source,
        &input,
        ParserPositionV0 {
            line: 2,
            character: 24,
        },
        &first_runtime,
    )
    .expect("cascade position analysis result");
    let second = read_omena_query_cascade_at_position_analysis_result(
        "Component.module.css",
        second_source,
        &edited_input,
        ParserPositionV0 {
            line: 2,
            character: 24,
        },
        &second_runtime,
    )
    .expect("cascade position analysis result after source edit");

    assert_eq!(first.product, "omena-query.analysis-result");
    assert_eq!(first.value.product, "omena-query.read-cascade-at-position");
    assert_eq!(first.precision.product, "omena-query.analysis-precision");
    assert_eq!(first.precision.value_domain, "cascadeAtPosition");
    assert_eq!(first.precision.flow_sensitivity, "positionScopedCascade");
    assert_eq!(
        first.precision.revision_axis,
        "OmenaQueryEvaluationRuntimeSummaryV0.expressionDomainRevision"
    );
    assert!(
        first
            .provenance
            .iter()
            .any(|entry| entry == "omena-query.read-cascade-at-position")
    );
    assert_eq!(first_runtime.expression_domain_revision, 1);
    assert_eq!(second_runtime.expression_domain_revision, 2);
    assert!(second_runtime.expression_domain_dirty_graph_count > 0);
    assert_eq!(first.revision, first_runtime.expression_domain_revision);
    assert_eq!(second.revision, second_runtime.expression_domain_revision);
    assert!(second.revision > first.revision);
    assert_eq!(
        first.value.referenced_declaration_computed_value.as_deref(),
        Some("black")
    );
    assert_eq!(
        second
            .value
            .referenced_declaration_computed_value
            .as_deref(),
        Some("blue")
    );
    let serialized = serde_json::to_value(&first).expect("analysis result serializes");
    assert_eq!(
        serialized["revision"],
        first_runtime.expression_domain_revision
    );
    assert_eq!(
        serialized["precision"]["revisionAxis"],
        "OmenaQueryEvaluationRuntimeSummaryV0.expressionDomainRevision"
    );
    assert_eq!(serialized["value"]["status"], "resolved");
}

#[test]
#[allow(deprecated)]
fn read_cascade_at_position_can_attach_categorical_evidence_when_requested() {
    let source = ":root { --surface: white; }\n.button { color: var(--surface); }\n";
    let cascade = read_omena_query_cascade_at_position_with_categorical_evidence(
        "Component.module.css",
        source,
        &sample_input(),
        ParserPositionV0 {
            line: 1,
            character: 24,
        },
        true,
    );
    assert!(cascade.is_some());
    let Some(cascade) = cascade else {
        return;
    };
    assert!(cascade.categorical_evidence.is_some());
    let Some(evidence) = cascade.categorical_evidence else {
        return;
    };
    assert_eq!(evidence.schema_version, "0");
    assert_eq!(evidence.layer_marker, "categorical-semantic");
    assert_eq!(evidence.endpoint_count, 10);
    assert_eq!(evidence.fixture_evidence.len(), 10);
    assert!(
        evidence
            .fixture_evidence
            .iter()
            .filter(|fixture| fixture.claim_scope == "computedEvidence")
            .all(|fixture| fixture.accepted)
    );
    assert!(evidence.fixture_evidence.iter().any(|fixture| {
        fixture.claim_scope == "researchDeferredMissingSourceSensitiveSubstrate"
            && !fixture.accepted
    }));
    assert!(
        evidence
            .endpoints
            .iter()
            .any(|endpoint| endpoint.endpoint_id == compatibility_categorical_endpoint_id_v0())
    );

    // The attached functor application is the real verdict over this cascade's
    // custom-property ranking. The ranking is acyclic (--surface is a literal),
    // so the cascade-ranking primitive keeps its single canonical role, the
    // baseline catalog is functorial, and the verdict is accepted. If the field
    // carried a constant verdict the cyclic sibling test below could not differ.
    let Some(functor) = evidence.functor_applications.first() else {
        return;
    };
    assert!(functor.accepted);
    assert!(functor.composition_preserved);
}

#[test]
fn read_cascade_at_position_categorical_evidence_rejects_cyclic_ranking() {
    // A cyclic custom-property ranking (--a -> --b -> --a) cannot converge, so
    // the cascade-ranking primitive is forced to play a conflicting second
    // categorical role. The functor object mapping is many-valued and the real
    // verdict rejects the mapping. The verdict therefore changes with the source.
    let source = r#":root {
  --a: var(--b);
  --b: var(--a);
}
.card { color: var(--a); }
"#;
    let cascade = read_omena_query_cascade_at_position_with_categorical_evidence(
        "Component.module.css",
        source,
        &sample_input(),
        ParserPositionV0 {
            line: 4,
            character: 22,
        },
        true,
    );
    let Some(cascade) = cascade else {
        return;
    };
    assert!(cascade.categorical_evidence.is_some());
    let Some(evidence) = cascade.categorical_evidence else {
        return;
    };
    let Some(functor) = evidence.functor_applications.first() else {
        return;
    };
    assert!(!functor.accepted);
}

#[test]
fn read_cascade_at_position_uses_exact_conditional_context() {
    let source = r#":root { --surface: base; }
@media (min-width: 40rem) {
  :root { --surface: wide; }
  .button { color: var(--surface); }
}
@media (max-width: 20rem) {
  :root { --surface: narrow; }
}
"#;
    let cascade = read_omena_query_cascade_at_position(
        "Component.module.css",
        source,
        &sample_input(),
        ParserPositionV0 {
            line: 3,
            character: 25,
        },
    );
    assert!(cascade.is_some());
    let Some(cascade) = cascade else {
        return;
    };

    assert_eq!(cascade.status, "resolved");
    assert_eq!(cascade.reference_name.as_deref(), Some("--surface"));
    assert_eq!(cascade.winner_declaration_source_order, Some(1));
    assert_eq!(cascade.candidate_declaration_count, 2);
    assert_eq!(cascade.shadowed_declaration_source_orders, vec![0]);
    assert_eq!(
        cascade
            .reference_custom_property_fixed_point_value
            .as_deref(),
        Some("wide")
    );
}

#[test]
fn read_cascade_at_position_uses_layer_ranked_lfp_winner() {
    let source = r#".button { --surface: unlayered; }
@layer components {
  .button {
    --surface: layered;
    color: var(--surface);
  }
}
"#;
    let cascade = read_omena_query_cascade_at_position(
        "Component.module.css",
        source,
        &sample_input(),
        ParserPositionV0 {
            line: 4,
            character: 15,
        },
    );
    assert!(cascade.is_some());
    let Some(cascade) = cascade else {
        return;
    };

    assert_eq!(cascade.status, "resolved");
    assert_eq!(cascade.reference_name.as_deref(), Some("--surface"));
    assert_eq!(cascade.winner_declaration_source_order, Some(0));
    // Unlayered ranks are opaque ordering tokens, not layer-count magnitudes.
    assert_eq!(cascade.winner_declaration_layer_rank, Some(i32::MAX));
    assert_eq!(
        cascade
            .reference_custom_property_fixed_point_value
            .as_deref(),
        Some("unlayered")
    );
}

#[test]
fn read_cascade_at_position_reports_iacvt_seed() {
    let source = ":root { --a: var(--b); --b: var(--a); }\n.button { color: var(--a); }\n";
    let cascade = read_omena_query_cascade_at_position(
        "Component.module.css",
        source,
        &sample_input(),
        ParserPositionV0 {
            line: 1,
            character: 22,
        },
    );
    assert!(cascade.is_some());
    let Some(cascade) = cascade else {
        return;
    };

    assert_eq!(cascade.status, "resolved");
    assert_eq!(cascade.reference_name.as_deref(), Some("--a"));
    assert_eq!(
        cascade.referenced_declaration_computed_value_status,
        Some("invalidAtComputedValueTime")
    );
    assert_eq!(
        cascade.referenced_declaration_computed_value.as_deref(),
        Some("canvastext")
    );
    assert!(cascade.referenced_declaration_invalid_at_computed_value_time);
    assert_eq!(cascade.custom_property_fixed_point_iteration_count, 1);
    assert_eq!(
        cascade.custom_property_fixed_point_guaranteed_invalid_count,
        2
    );
    assert_eq!(
        cascade.reference_custom_property_fixed_point_status,
        Some("guaranteedInvalid")
    );
    assert_eq!(
        cascade
            .reference_custom_property_fixed_point_value
            .as_deref(),
        Some("guaranteed-invalid")
    );
    assert!(
        cascade
            .referenced_declaration_computed_value_derivation_steps
            .contains(&"invalidAtComputedValueTimeFallsBackAsUnset")
    );
    let refinement = cascade
        .refinement_evidence
        .as_ref()
        .expect("cyclic custom property refinement evidence");
    assert_eq!(
        refinement.claim_level,
        "m6DimensionalRefinementBridgeSubstrate"
    );
    assert_eq!(refinement.property_name, "--a");
    assert_eq!(refinement.satisfied_all_context_count, 0);
    assert_eq!(refinement.unsatisfiable_context_count, 1);
    assert_eq!(
        format!("{:?}", refinement.evaluations[0].combined_verdict),
        "Unsatisfiable"
    );
}

#[test]
fn read_cascade_at_position_keeps_incomplete_color_mismatch_indeterminate() {
    let source = ":root { --tone: 12px; }\n.target { color: var(--tone); }\n";
    let cascade = read_omena_query_cascade_at_position(
        "Component.module.css",
        source,
        &sample_input(),
        ParserPositionV0 {
            line: 1,
            character: 24,
        },
    )
    .expect("color var reference");

    assert_eq!(
        cascade.referenced_declaration_computed_value_status,
        Some("indeterminate")
    );
    assert!(!cascade.referenced_declaration_invalid_at_computed_value_time);
}

#[test]
fn read_cascade_at_position_resolves_paint_values_through_the_pinned_matcher() {
    let declaration = ".target { fill: var(--h3); stroke: var(--h6); }";
    let source = format!(":root {{ --h3: #f0f; --h6: #ff00ff; }}\n{declaration}\n");
    for (name, expected) in [("--h3", "#f0f"), ("--h6", "#ff00ff")] {
        let character = declaration.find(name).expect("paint reference offset");
        let cascade = read_omena_query_cascade_at_position(
            "Component.module.css",
            source.as_str(),
            &sample_input(),
            ParserPositionV0 { line: 1, character },
        )
        .expect("paint var reference");
        assert_eq!(
            cascade.referenced_declaration_computed_value_status,
            Some("resolved")
        );
        assert_eq!(
            cascade.referenced_declaration_computed_value.as_deref(),
            Some(expected)
        );
        assert!(!cascade.referenced_declaration_invalid_at_computed_value_time);
    }
}

#[test]
fn product_grid_gap_remains_resolved_after_custom_property_substitution() {
    let source = include_str!(
        "../../../omena-benchmarks/fixtures/bundler/css-modules-product-grid.module.css"
    );
    let cascade = read_omena_query_cascade_at_position(
        "css-modules-product-grid.module.css",
        source,
        &sample_input(),
        ParserPositionV0 {
            line: 14,
            character: 13,
        },
    )
    .expect("product-grid gap var reference");

    assert_eq!(
        cascade.referenced_declaration_computed_value_status,
        Some("resolved")
    );
    assert_eq!(
        cascade.referenced_declaration_computed_value.as_deref(),
        Some("clamp(0.75rem, 1vw, 1.25rem)")
    );
}

#[test]
fn tracked_thirty_six_var_sites_have_no_undeclared_status_delta() {
    let cases = [
        ("color", "red"),
        ("color", "#ff00aa"),
        ("fill", "#ff00aa"),
        ("stroke", "rgb(1 2 3)"),
        ("width", "calc(10px + 2px)"),
        ("width", "min(10px, 20px)"),
        ("width", "max(10%, 20%)"),
        ("width", "clamp(1px, 2px, 3px)"),
        ("height", "calc(50% + 2px)"),
        ("margin", "calc(1rem + 2px)"),
        ("padding", "min(1rem, 2rem)"),
        ("row-gap", "clamp(1px, 2px, 3px)"),
        ("column-gap", "max(1%, 2%)"),
        ("gap", "clamp(1px, 2px, 3px)"),
        ("opacity", "calc(0.5 + 0.1)"),
        ("line-height", "min(1.2, 1.5)"),
        ("animation-duration", "calc(1s + 200ms)"),
        ("transition-duration", "max(1s, 2s)"),
        ("rotate", "calc(10deg + 5deg)"),
        ("grid-template-columns", "minmax(101px, 1fr)"),
        ("grid-template-columns", "repeat(3, 1fr)"),
        ("grid-template-columns", "repeat(2, minmax(0, 1fr))"),
        ("grid-template-columns", "1fr 2fr"),
        ("border-top", "1px solid red"),
        ("margin", "0 auto"),
        ("padding", "1px 2px"),
        ("display", "grid"),
        ("position", "absolute"),
        ("inset", "0"),
        ("top", "1px"),
        ("z-index", "2"),
        ("font-weight", "700"),
        ("font-size", "16px"),
        ("background-color", "rebeccapurple"),
        ("border-radius", "4px"),
        ("flex-grow", "1"),
    ];
    assert_eq!(cases.len(), 36);

    let status_deltas = cases
        .iter()
        .enumerate()
        .filter_map(|(index, (property, value))| {
            let declaration = format!(".target {{ {property}: var(--tracked); }}");
            let reference_character = declaration
                .find("--tracked")
                .expect("tracked reference offset");
            let source = format!(":root {{ --tracked: {value}; }}\n{declaration}\n");
            let cascade = read_omena_query_cascade_at_position(
                "Tracked.module.css",
                source.as_str(),
                &sample_input(),
                ParserPositionV0 {
                    line: 1,
                    character: reference_character,
                },
            )?;
            (cascade.referenced_declaration_computed_value_status != Some("resolved")).then_some((
                index,
                *property,
                *value,
                cascade.referenced_declaration_computed_value_status,
            ))
        })
        .collect::<Vec<_>>();

    assert!(
        status_deltas.is_empty(),
        "the declared status-delta allowlist is empty: {status_deltas:?}"
    );
}

#[test]
fn read_cascade_at_position_reports_unknown_metadata_as_indeterminate() {
    let source = ".target { future-property: var(--missing, unset); }\n";
    let cascade = read_omena_query_cascade_at_position(
        "Component.module.css",
        source,
        &sample_input(),
        ParserPositionV0 {
            line: 0,
            character: 34,
        },
    );
    assert!(cascade.is_some());
    let Some(cascade) = cascade else {
        return;
    };

    assert_eq!(
        cascade.referenced_declaration_computed_value_status,
        Some("indeterminate")
    );
    assert!(cascade.referenced_declaration_computed_value.is_none());
    assert!(!cascade.referenced_declaration_invalid_at_computed_value_time);
    assert!(cascade.referenced_declaration_computed_value_indeterminate);
    assert_eq!(
        cascade.referenced_declaration_computed_value_indeterminate_reason,
        Some("propertyInitialValueMetadataUnavailable")
    );
}
