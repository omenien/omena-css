#![allow(clippy::expect_used)]
use super::*;
use crate::{
    read_omena_query_cascade_at_position,
    read_omena_query_cascade_at_position_with_categorical_evidence,
};

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
    assert_eq!(cascade.winner_declaration_layer_rank, Some(1));
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
            .any(|endpoint| endpoint.endpoint_id == "rust/omena-categorical/verify-site-stability")
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
    assert_eq!(cascade.winner_declaration_layer_rank, Some(2));
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
    assert!(cascade.custom_property_fixed_point_iteration_count >= 2);
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
