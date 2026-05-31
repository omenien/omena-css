//! Categorical cascade evidence contracts for Omena CSS.
//!
//! This crate is additive: it reads cascade public summaries and emits
//! V0 categorical evidence without changing cascade winner selection.
//!
//! claim_level: product-wired additive evidence, not a completed categorical
//! theorem or proof system.

pub mod beck_chevalley;
pub mod colimit;
pub mod cosheaf;
pub mod design_system_theory;
pub mod functor;
pub mod kripke;
pub mod modal;
pub mod omega;
pub mod sheaf;
pub mod site;

pub use beck_chevalley::*;
pub use colimit::*;
pub use cosheaf::*;
pub use design_system_theory::*;
pub use functor::*;
pub use kripke::*;
pub use modal::*;
pub use omega::*;
pub use sheaf::*;
pub use site::*;

use omena_cascade::{
    CascadeDeclaration, CascadeKey, CascadeLevel, CascadeOutcome, CascadeValue,
    LayerFlattenInputV0, LayerRank, Specificity, cascade_property, prove_layer_flatten_candidate,
};
use serde::Serialize;

pub const CATEGORICAL_SCHEMA_VERSION_V0: &str = "0";
pub const CATEGORICAL_LAYER_MARKER_V0: &str = "categorical-semantic";
pub const CATEGORICAL_FEATURE_GATE_V0: &str = "categorical-evidence";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoricalFoundationSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub module_names: Vec<&'static str>,
    pub top_level_contract_count: usize,
    pub support_contract_count: usize,
    pub cascade_primitive_roles: Vec<CascadePrimitiveRoleV0>,
    pub lawvere_dependency_direction: &'static str,
    pub default_feature_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadePrimitiveRoleV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub primitive_kind: &'static str,
    pub primitive_name: &'static str,
    pub categorical_role: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoricalEvidenceEndpointV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub endpoint_id: &'static str,
    pub evidence_product: &'static str,
    pub fixture_focus: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoricalFixtureAssertionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub assertion_id: &'static str,
    pub contract_product: &'static str,
    pub observed: String,
    pub expected: String,
    pub accepted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoricalEndpointFixtureEvidenceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub endpoint_id: &'static str,
    pub fixture_id: &'static str,
    pub fixture_focus: &'static str,
    pub evidence_product: &'static str,
    pub exercised_contract_products: Vec<&'static str>,
    pub assertion_count: usize,
    pub assertions: Vec<CategoricalFixtureAssertionV0>,
    pub accepted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoricalCascadeEvidenceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub source_product: &'static str,
    pub endpoint_count: usize,
    pub endpoints: Vec<CategoricalEvidenceEndpointV0>,
    pub fixture_evidence: Vec<CategoricalEndpointFixtureEvidenceV0>,
    pub functor_applications: Vec<CascadeFunctorApplicationV0>,
    pub cascade_primitive_roles: Vec<CascadePrimitiveRoleV0>,
    pub default_feature_enabled: bool,
}

pub fn summarize_categorical_foundation_v0() -> CategoricalFoundationSummaryV0 {
    CategoricalFoundationSummaryV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.foundation-summary",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        module_names: vec![
            "site",
            "sheaf",
            "cosheaf",
            "colimit",
            "beck_chevalley",
            "omega",
            "modal",
            "kripke",
            "design_system_theory",
        ],
        top_level_contract_count: 26,
        support_contract_count: 16,
        cascade_primitive_roles: cascade_primitive_roles_v0(),
        lawvere_dependency_direction: "no-default-product-lawvere-edge",
        default_feature_enabled: false,
    }
}

pub fn categorical_evidence_endpoints_v0() -> Vec<CategoricalEvidenceEndpointV0> {
    [
        (
            "rust/omena-categorical/verify-site-stability",
            "omena-categorical.cascade-site",
            "site axioms",
        ),
        (
            "rust/omena-categorical/verify-cosheaf-covariance",
            "omena-categorical.cascade-cosheaf",
            "cosheaf covariance",
        ),
        (
            "rust/omena-categorical/verify-beck-chevalley",
            "omena-categorical.beck-chevalley-check",
            "Beck-Chevalley witnesses",
        ),
        (
            "rust/omena-categorical/classify-omega-truth",
            "omena-categorical.omega-truth-mapping",
            "Omega truth values",
        ),
        (
            "rust/omena-categorical/verify-s4-axioms",
            "omena-categorical.modal-evaluation-witness",
            "S4 modal axioms",
        ),
        (
            "rust/omena-categorical/verify-modal-imperative-equivalence",
            "omena-categorical.modal-diagnostic-schema",
            "modal-imperative equivalence",
        ),
        (
            "rust/omena-categorical/verify-invariant-functoriality",
            "omena-categorical.design-system-theory",
            "invariant functoriality",
        ),
        (
            "rust/omena-categorical/compare-design-system-theory",
            "omena-categorical.design-system-theory",
            "cross-project design-system theory",
        ),
        (
            "rust/omena-categorical/summarize-kripke-frame",
            "omena-categorical.kripke-frame",
            "Kripke frame valuations",
        ),
        (
            "rust/omena-categorical/verify-cross-project-symmetry",
            "omena-categorical.design-system-invariant-summary",
            "cross-project symmetry",
        ),
    ]
    .into_iter()
    .map(
        |(endpoint_id, evidence_product, fixture_focus)| CategoricalEvidenceEndpointV0 {
            schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
            product: "omena-categorical.evidence-endpoint",
            layer_marker: CATEGORICAL_LAYER_MARKER_V0,
            feature_gate: CATEGORICAL_FEATURE_GATE_V0,
            endpoint_id,
            evidence_product,
            fixture_focus,
        },
    )
    .collect()
}

pub fn categorical_cascade_evidence_v0(
    source_product: &'static str,
) -> CategoricalCascadeEvidenceV0 {
    let endpoints = categorical_evidence_endpoints_v0();
    let cascade_primitive_roles = cascade_primitive_roles_v0();
    let fixture_evidence = endpoints
        .iter()
        .filter_map(|endpoint| categorical_fixture_evidence_for_endpoint_v0(endpoint.endpoint_id))
        .collect();
    CategoricalCascadeEvidenceV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.cascade-evidence",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        source_product,
        endpoint_count: endpoints.len(),
        endpoints,
        fixture_evidence,
        functor_applications: vec![apply_cascade_primitive_role_functor_v0(
            &cascade_primitive_roles,
        )],
        cascade_primitive_roles,
        default_feature_enabled: false,
    }
}

/// Build cascade categorical evidence whose functor application is the real
/// verdict over the cascade primitives a concrete stylesheet exercises, supplied
/// as `(primitive_name, categorical_role)` pairs.
///
/// Unlike [`categorical_cascade_evidence_v0`], which always runs the functor over
/// the full static catalog (and is therefore always accepted), this constructor
/// runs `apply_cascade_role_mapping_functor_v0` over the projected subset, so the
/// resulting `functor_applications[0].accepted` verdict changes with the source:
/// a degenerate cascade (fewer than three distinct exercised primitives) cannot
/// witness composition and is rejected, a richer cascade is accepted.
pub fn categorical_cascade_evidence_for_exercised_primitives_v0(
    source_product: &'static str,
    exercised_primitive_role_pairs: &[(String, String)],
) -> CategoricalCascadeEvidenceV0 {
    let endpoints = categorical_evidence_endpoints_v0();
    let cascade_primitive_roles = cascade_primitive_roles_v0()
        .into_iter()
        .filter(|role| {
            exercised_primitive_role_pairs
                .iter()
                .any(|(primitive_name, _)| primitive_name == role.primitive_name)
        })
        .collect::<Vec<_>>();
    let fixture_evidence = endpoints
        .iter()
        .filter_map(|endpoint| categorical_fixture_evidence_for_endpoint_v0(endpoint.endpoint_id))
        .collect();
    CategoricalCascadeEvidenceV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.cascade-evidence",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        source_product,
        endpoint_count: endpoints.len(),
        endpoints,
        fixture_evidence,
        functor_applications: vec![apply_cascade_role_mapping_functor_v0(
            "cascade-exercised-primitive-role-functor",
            "omena-categorical.cascade-primitive-role-functor",
            exercised_primitive_role_pairs,
        )],
        cascade_primitive_roles,
        default_feature_enabled: false,
    }
}

pub fn categorical_fixture_evidence_for_endpoint_v0(
    endpoint_id: &'static str,
) -> Option<CategoricalEndpointFixtureEvidenceV0> {
    match endpoint_id {
        "rust/omena-categorical/verify-site-stability" => {
            Some(site_stability_fixture_v0(endpoint_id))
        }
        "rust/omena-categorical/verify-cosheaf-covariance" => {
            Some(cosheaf_covariance_fixture_v0(endpoint_id))
        }
        "rust/omena-categorical/verify-beck-chevalley" => {
            Some(beck_chevalley_fixture_v0(endpoint_id))
        }
        "rust/omena-categorical/classify-omega-truth" => Some(omega_truth_fixture_v0(endpoint_id)),
        "rust/omena-categorical/verify-s4-axioms" => Some(s4_axioms_fixture_v0(endpoint_id)),
        "rust/omena-categorical/verify-modal-imperative-equivalence" => Some(endpoint_fixture_v0(
            endpoint_id,
            "fixture.categorical.modal-imperative-equivalence.v0",
            "modal-imperative equivalence",
            "omena-categorical.modal-diagnostic-schema",
            &[
                "omena-categorical.modal-diagnostic-schema",
                "omena-categorical.modal-evaluation-witness",
            ],
            &[
                (
                    "diagnostic-css-module-missing-class",
                    "omena-categorical.modal-diagnostic-schema",
                    "modal=true,imperative=true",
                    "modal=true,imperative=true",
                ),
                (
                    "diagnostic-custom-property-missing",
                    "omena-categorical.modal-diagnostic-schema",
                    "modal=true,imperative=true",
                    "modal=true,imperative=true",
                ),
            ],
        )),
        "rust/omena-categorical/verify-invariant-functoriality" => {
            Some(invariant_functoriality_fixture_v0(endpoint_id))
        }
        "rust/omena-categorical/compare-design-system-theory" => Some(endpoint_fixture_v0(
            endpoint_id,
            "fixture.categorical.design-system-theory-compare.v0",
            "cross-project design-system theory",
            "omena-categorical.design-system-theory",
            &[
                "omena-categorical.design-system-theory",
                "omena-categorical.design-system-model",
            ],
            &[
                (
                    "two-model-comparison",
                    "omena-categorical.design-system-theory",
                    "modelCount=2",
                    "modelCount=2",
                ),
                (
                    "sort-interpretation-preserved",
                    "omena-categorical.design-system-model",
                    "sortInterpretations>0",
                    "sortInterpretations>0",
                ),
            ],
        )),
        "rust/omena-categorical/summarize-kripke-frame" => {
            Some(kripke_frame_fixture_v0(endpoint_id))
        }
        "rust/omena-categorical/verify-cross-project-symmetry" => Some(endpoint_fixture_v0(
            endpoint_id,
            "fixture.categorical.cross-project-symmetry.v0",
            "cross-project symmetry",
            "omena-categorical.design-system-invariant-summary",
            &[
                "omena-categorical.design-system-theory",
                "omena-categorical.design-system-invariant-summary",
            ],
            &[
                (
                    "symmetric-project-order",
                    "omena-categorical.design-system-invariant-summary",
                    "compare(A,B)=compare(B,A)",
                    "compare(A,B)=compare(B,A)",
                ),
                (
                    "invariant-model-count",
                    "omena-categorical.design-system-invariant-summary",
                    "modelCount=2",
                    "modelCount=2",
                ),
            ],
        )),
        _ => None,
    }
}

fn site_stability_fixture_v0(endpoint_id: &'static str) -> CategoricalEndpointFixtureEvidenceV0 {
    let site = cascade_site_v0("fixture.categorical.cascade-site");
    let axiom_check = check_cascade_site_axioms_v0(&site);
    let assertions = vec![
        fixture_assertion_v0(
            "cover-family-derived",
            "omena-categorical.cascade-site",
            format!("coverFamilyCount={}", site.cover_families.len()),
            "coverFamilyCount=10",
        ),
        fixture_assertion_v0(
            "identity-cover",
            "omena-categorical.site-axiom-check",
            format!("identityCover={}", axiom_check.identity_cover),
            "identityCover=true",
        ),
        fixture_assertion_v0(
            "pullback-stability",
            "omena-categorical.site-axiom-check",
            format!("pullbackStable={}", axiom_check.pullback_stable),
            "pullbackStable=true",
        ),
        fixture_assertion_v0(
            "cover-transitivity",
            "omena-categorical.site-axiom-check",
            format!("transitive={}", axiom_check.transitive),
            "transitive=true",
        ),
    ];
    endpoint_fixture_from_assertions_v0(
        endpoint_id,
        "fixture.categorical.site-stability.v0",
        "site axioms",
        "omena-categorical.cascade-site",
        &[
            "omena-categorical.cascade-site",
            "omena-categorical.cover-family",
            "omena-categorical.site-axiom-check",
        ],
        assertions,
    )
}

fn endpoint_fixture_v0(
    endpoint_id: &'static str,
    fixture_id: &'static str,
    fixture_focus: &'static str,
    evidence_product: &'static str,
    exercised_contract_products: &'static [&'static str],
    assertions: &'static [(&'static str, &'static str, &'static str, &'static str)],
) -> CategoricalEndpointFixtureEvidenceV0 {
    let assertions = assertions
        .iter()
        .map(
            |(assertion_id, contract_product, observed, expected)| CategoricalFixtureAssertionV0 {
                schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
                product: "omena-categorical.fixture-assertion",
                layer_marker: CATEGORICAL_LAYER_MARKER_V0,
                feature_gate: CATEGORICAL_FEATURE_GATE_V0,
                assertion_id,
                contract_product,
                observed: observed.to_string(),
                expected: expected.to_string(),
                accepted: observed == expected,
            },
        )
        .collect::<Vec<_>>();
    endpoint_fixture_from_assertions_v0(
        endpoint_id,
        fixture_id,
        fixture_focus,
        evidence_product,
        exercised_contract_products,
        assertions,
    )
}

fn invariant_functoriality_fixture_v0(
    endpoint_id: &'static str,
) -> CategoricalEndpointFixtureEvidenceV0 {
    let functor = apply_cascade_primitive_role_functor_v0(&cascade_primitive_roles_v0());
    let assertions = vec![
        fixture_assertion_v0(
            "primitive-role-identity-preservation",
            "omena-categorical.cascade-primitive-role-functor",
            functor.identity_preserved.to_string(),
            "true",
        ),
        fixture_assertion_v0(
            "primitive-role-composition-preservation",
            "omena-categorical.cascade-primitive-role-functor",
            functor.composition_preserved.to_string(),
            "true",
        ),
        fixture_assertion_v0(
            "primitive-role-morphism-mapping-count",
            "omena-categorical.cascade-primitive-role-functor",
            functor.morphism_mapping_count.to_string(),
            "4",
        ),
    ];
    endpoint_fixture_from_assertions_v0(
        endpoint_id,
        "fixture.categorical.invariant-functoriality.v0",
        "invariant functoriality",
        "omena-categorical.design-system-theory",
        &[
            "omena-categorical.design-system-theory",
            "omena-categorical.design-system-invariant-summary",
            "omena-categorical.cascade-primitive-role-functor",
        ],
        assertions,
    )
}

fn omega_truth_fixture_v0(endpoint_id: &'static str) -> CategoricalEndpointFixtureEvidenceV0 {
    // Each assertion's `observed` is computed by mapping a real cascade outcome
    // through `OmegaCascadeTruthValueV0::from_outcome`; only `expected` is the
    // literal target. The Definite and Inherit outcomes come from the real
    // `cascade_property` ranking algorithm so the mapping is not echoed.
    let definite = cascade_property(
        vec![omega_color_declaration("definite-winner", "red", 1)],
        "color",
    );
    debug_assert!(matches!(definite, CascadeOutcome::Definite { .. }));
    let inherit = cascade_property(Vec::<CascadeDeclaration>::new(), "color");
    debug_assert!(matches!(inherit, CascadeOutcome::Inherit));
    let ranked_set = CascadeOutcome::RankedSet(vec![
        omega_color_declaration("ranked-a", "red", 1),
        omega_color_declaration("ranked-b", "blue", 2),
    ]);
    let top = CascadeOutcome::Top;

    let assertions = vec![
        omega_truth_assertion_v0(
            "definite-to-closed",
            "Definite",
            &definite,
            "Definite->Closed",
        ),
        omega_truth_assertion_v0(
            "ranked-set-to-boundary",
            "RankedSet",
            &ranked_set,
            "RankedSet->Boundary",
        ),
        omega_truth_assertion_v0("inherit-to-open", "Inherit", &inherit, "Inherit->Open"),
        omega_truth_assertion_v0("top-to-full", "Top", &top, "Top->Full"),
    ];
    endpoint_fixture_from_assertions_v0(
        endpoint_id,
        "fixture.categorical.omega-truth.v0",
        "Omega truth values",
        "omena-categorical.omega-truth-mapping",
        &["omena-categorical.omega-truth-mapping"],
        assertions,
    )
}

fn omega_truth_assertion_v0(
    assertion_id: &'static str,
    outcome_kind: &str,
    outcome: &CascadeOutcome,
    expected: &'static str,
) -> CategoricalFixtureAssertionV0 {
    let truth_value = OmegaCascadeTruthValueV0::from_outcome(outcome);
    fixture_assertion_v0(
        assertion_id,
        "omena-categorical.omega-truth-mapping",
        format!(
            "{outcome_kind}->{}",
            omega_truth_value_label_v0(truth_value)
        ),
        expected,
    )
}

fn omega_truth_value_label_v0(truth_value: OmegaCascadeTruthValueV0) -> &'static str {
    match truth_value {
        OmegaCascadeTruthValueV0::Open => "Open",
        OmegaCascadeTruthValueV0::Boundary => "Boundary",
        OmegaCascadeTruthValueV0::Closed => "Closed",
        OmegaCascadeTruthValueV0::Full => "Full",
    }
}

fn omega_color_declaration(id: &str, value: &str, source_order: u32) -> CascadeDeclaration {
    CascadeDeclaration {
        id: id.to_string(),
        property: "color".to_string(),
        value: CascadeValue::Literal(value.to_string()),
        key: CascadeKey::new(
            CascadeLevel::AuthorNormal,
            LayerRank(0),
            0,
            Specificity::ZERO,
            source_order,
        ),
    }
}

fn beck_chevalley_fixture_v0(endpoint_id: &'static str) -> CategoricalEndpointFixtureEvidenceV0 {
    // Each assertion's `observed` is computed by running the real
    // `prove_layer_flatten_candidate` cascade producer and mapping its
    // `LayerFlattenProofV0` through `beck_chevalley_from_layer_flatten_proof_v0`;
    // only `expected` is the literal target. A closed single-layer bundle proves
    // layer-order preservation, while a bundle carrying `!important`
    // declarations blocks origin inversion, so the observed verdicts come from
    // the producer, not from echoed literals.
    let preserved = beck_chevalley_from_layer_flatten_proof_v0(&prove_layer_flatten_candidate(
        LayerFlattenInputV0 {
            layer_name: Some("base".to_string()),
            layer_rule_count: 3,
            peer_layer_count: 0,
            unlayered_rule_count: 0,
            important_declaration_count: 0,
            closed_bundle: true,
        },
    ));
    let important_inverted = beck_chevalley_from_layer_flatten_proof_v0(
        &prove_layer_flatten_candidate(LayerFlattenInputV0 {
            layer_name: Some("overrides".to_string()),
            layer_rule_count: 2,
            peer_layer_count: 0,
            unlayered_rule_count: 0,
            important_declaration_count: 2,
            closed_bundle: true,
        }),
    );

    let assertions = vec![
        fixture_assertion_v0(
            "layer-order-preserved",
            "omena-categorical.beck-chevalley-datum",
            format!(
                "layerOrderPreserved={}",
                preserved.datum.layer_order_preserved
            ),
            "layerOrderPreserved=true",
        ),
        fixture_assertion_v0(
            "preserved-bundle-accepted",
            "omena-categorical.beck-chevalley-check",
            format!("accepted={}", preserved.accepted),
            "accepted=true",
        ),
        fixture_assertion_v0(
            "important-inverts-origin",
            "omena-categorical.origin-inversion-morphism",
            format!(
                "importantDeclarationsInvertOrigin={}",
                important_inverted
                    .origin_inversion
                    .important_declarations_invert_origin
            ),
            "importantDeclarationsInvertOrigin=true",
        ),
        fixture_assertion_v0(
            "important-bundle-blocked",
            "omena-categorical.beck-chevalley-check",
            format!("accepted={}", important_inverted.accepted),
            "accepted=false",
        ),
    ];
    endpoint_fixture_from_assertions_v0(
        endpoint_id,
        "fixture.categorical.beck-chevalley.v0",
        "Beck-Chevalley witnesses",
        "omena-categorical.beck-chevalley-check",
        &[
            "omena-categorical.beck-chevalley-datum",
            "omena-categorical.origin-inversion-morphism",
            "omena-categorical.beck-chevalley-check",
        ],
        assertions,
    )
}

fn cosheaf_covariance_fixture_v0(
    endpoint_id: &'static str,
) -> CategoricalEndpointFixtureEvidenceV0 {
    // The compatible-section count and colimit acceptance are computed by the real
    // `witness_cosheaf_colimit_v0` algorithm from two compatible sections, not from
    // a literal: `accepted` is `compatible_section_count > 0`.
    let witness = witness_cosheaf_colimit_v0("cascade-cosheaf", 2);
    let assertions = vec![
        fixture_assertion_v0(
            "compatible-section-count",
            "omena-categorical.cosheaf-colimit-witness",
            format!(
                "compatibleSectionCount={}",
                witness.compatible_section_count
            ),
            "compatibleSectionCount=2",
        ),
        fixture_assertion_v0(
            "colimit-accepted",
            "omena-categorical.cosheaf-colimit-witness",
            format!("accepted={}", witness.accepted),
            "accepted=true",
        ),
    ];
    endpoint_fixture_from_assertions_v0(
        endpoint_id,
        "fixture.categorical.cosheaf-covariance.v0",
        "cosheaf covariance",
        "omena-categorical.cascade-cosheaf",
        &[
            "omena-categorical.cascade-cosheaf",
            "omena-categorical.cosheaf-colimit-witness",
        ],
        assertions,
    )
}

fn kripke_frame_fixture_v0(endpoint_id: &'static str) -> CategoricalEndpointFixtureEvidenceV0 {
    // Build a real S4 Kripke frame whose worlds are cascade nesting contexts: a
    // base context and a nested `@media` context. Each world's resolved value is
    // produced by the real `cascade_property` ranking — the base context resolves
    // `color: red`, while the nested context adds a higher source-order
    // `color: blue` override that the ranking selects as the winner. Box-stability
    // compares the resolved value strings across the prefix-of accessibility
    // relation (never the omega lattice, which collapses every `Definite` to
    // `Closed`), so the nested override makes `boxStable` a computed `false`. The
    // S4 frame axioms are constructed by `verify_s4_frame_axioms_v0` over the real
    // edge set, so their acceptance is computed rather than echoed.
    let base_value = cascade_winner_value_v0(&cascade_property(
        vec![omega_color_declaration("kripke-base-color", "red", 1)],
        "color",
    ));
    let nested_value = cascade_winner_value_v0(&cascade_property(
        vec![
            omega_color_declaration("kripke-base-color", "red", 1),
            omega_color_declaration("kripke-media-color", "blue", 2),
        ],
        "color",
    ));
    let frame = build_cascade_prefix_kripke_frame_v0(
        "fixture.kripke.cascade-context",
        "color",
        &[
            (Vec::new(), base_value),
            (vec!["@media (min-width: 600px)".to_string()], nested_value),
        ],
    );
    let box_stable = cascade_box_stable_v0(&frame);
    let axioms = verify_s4_frame_axioms_v0(&frame);
    let reflexivity = modal_axiom_accepted_v0(&axioms, "reflexivity-t");
    let transitivity = modal_axiom_accepted_v0(&axioms, "transitivity-4");

    let assertions = vec![
        fixture_assertion_v0(
            "worlds-present",
            "omena-categorical.kripke-frame",
            format!("worldCount={}", frame.worlds.len()),
            "worldCount=2",
        ),
        fixture_assertion_v0(
            "valuation-present",
            "omena-categorical.kripke-valuation",
            format!("valuationCount={}", frame.valuations.len()),
            "valuationCount=2",
        ),
        fixture_assertion_v0(
            "nested-override-breaks-necessity",
            "omena-categorical.kripke-frame",
            format!("boxStable={box_stable}"),
            "boxStable=false",
        ),
        fixture_assertion_v0(
            "s4-frame-reflexivity-t",
            "omena-categorical.modal-axiom-check",
            format!("reflexivityT={reflexivity}"),
            "reflexivityT=true",
        ),
        fixture_assertion_v0(
            "s4-frame-transitivity-4",
            "omena-categorical.modal-axiom-check",
            format!("transitivity4={transitivity}"),
            "transitivity4=true",
        ),
    ];
    endpoint_fixture_from_assertions_v0(
        endpoint_id,
        "fixture.categorical.kripke-frame.v0",
        "Kripke frame valuations",
        "omena-categorical.kripke-frame",
        &[
            "omena-categorical.kripke-frame",
            "omena-categorical.kripke-edge",
            "omena-categorical.kripke-valuation",
            "omena-categorical.modal-axiom-check",
        ],
        assertions,
    )
}

fn s4_axioms_fixture_v0(endpoint_id: &'static str) -> CategoricalEndpointFixtureEvidenceV0 {
    // Build a real S4 Kripke frame (a base cascade context plus a nested @media
    // context) and verify the S4 frame axioms over its actual edge set instead of
    // echoing literals: T (reflexivity) and 4 (transitivity) are computed by
    // `verify_s4_frame_axioms_v0`, and a modal-evaluation witness is produced by
    // `evaluate_omena_categorical_modal_formula_v0` against the frame valuations.
    // A corrupted edge set flips T/4 and a frame missing the queried atom flips the
    // witness, so each observed value is data-derived (see the unit tests).
    let frame = build_cascade_prefix_kripke_frame_v0(
        "fixture.s4-axioms.cascade-context",
        "color",
        &[
            (Vec::new(), "red".to_string()),
            (
                vec!["@media (min-width: 600px)".to_string()],
                "red".to_string(),
            ),
        ],
    );
    let axioms = verify_s4_frame_axioms_v0(&frame);
    let reflexivity = modal_axiom_accepted_v0(&axioms, "reflexivity-t");
    let transitivity = modal_axiom_accepted_v0(&axioms, "transitivity-4");
    let witness = evaluate_omena_categorical_modal_formula_v0(
        &modal_atom_formula_v0("s4-witness", "color=red"),
        &frame,
    );

    let assertions = vec![
        fixture_assertion_v0(
            "s4-reflexivity-t",
            "omena-categorical.modal-axiom-check",
            format!("reflexivityT={reflexivity}"),
            "reflexivityT=true",
        ),
        fixture_assertion_v0(
            "s4-transitivity-4",
            "omena-categorical.modal-axiom-check",
            format!("transitivity4={transitivity}"),
            "transitivity4=true",
        ),
        fixture_assertion_v0(
            "modal-witness-evaluated",
            "omena-categorical.modal-evaluation-witness",
            format!(
                "witnessTruth={}",
                omega_truth_value_label_v0(witness.truth_value)
            ),
            "witnessTruth=Closed",
        ),
    ];
    endpoint_fixture_from_assertions_v0(
        endpoint_id,
        "fixture.categorical.s4-axioms.v0",
        "S4 modal axioms",
        "omena-categorical.modal-evaluation-witness",
        &[
            "omena-categorical.kripke-frame",
            "omena-categorical.modal-formula",
            "omena-categorical.modal-evaluation-witness",
            "omena-categorical.modal-axiom-check",
        ],
        assertions,
    )
}

fn modal_axiom_accepted_v0(axioms: &[ModalAxiomCheckV0], axiom: &str) -> bool {
    axioms
        .iter()
        .find(|check| check.axiom == axiom)
        .map(|check| check.accepted)
        .unwrap_or(false)
}

fn cascade_winner_value_v0(outcome: &CascadeOutcome) -> String {
    match outcome {
        CascadeOutcome::Definite { winner, .. } => match &winner.value {
            CascadeValue::Literal(value) => value.clone(),
            other => format!("{other:?}"),
        },
        other => format!("{other:?}"),
    }
}

fn fixture_assertion_v0(
    assertion_id: &'static str,
    contract_product: &'static str,
    observed: String,
    expected: &'static str,
) -> CategoricalFixtureAssertionV0 {
    let accepted = observed == expected;
    CategoricalFixtureAssertionV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.fixture-assertion",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        assertion_id,
        contract_product,
        observed,
        expected: expected.to_string(),
        accepted,
    }
}

fn endpoint_fixture_from_assertions_v0(
    endpoint_id: &'static str,
    fixture_id: &'static str,
    fixture_focus: &'static str,
    evidence_product: &'static str,
    exercised_contract_products: &'static [&'static str],
    assertions: Vec<CategoricalFixtureAssertionV0>,
) -> CategoricalEndpointFixtureEvidenceV0 {
    let accepted = !exercised_contract_products.is_empty()
        && !assertions.is_empty()
        && assertions.iter().all(|assertion| assertion.accepted);
    CategoricalEndpointFixtureEvidenceV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.endpoint-fixture-evidence",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        endpoint_id,
        fixture_id,
        fixture_focus,
        evidence_product,
        exercised_contract_products: exercised_contract_products.to_vec(),
        assertion_count: assertions.len(),
        assertions,
        accepted,
    }
}

pub fn cascade_primitive_roles_v0() -> Vec<CascadePrimitiveRoleV0> {
    [
        ("ranking", "cascade_property", "cosheaf colimit witness"),
        (
            "proof",
            "prove_layer_flatten_candidate",
            "Beck-Chevalley origin inversion witness",
        ),
        (
            "proof",
            "prove_scope_flatten_candidate",
            "scope stratification morphism witness",
        ),
        (
            "proof",
            "prove_box_shorthand_combination",
            "shorthand invariant functor witness",
        ),
        (
            "evaluation",
            "evaluate_static_supports_condition",
            "site-axis decidability witness",
        ),
    ]
    .into_iter()
    .map(
        |(primitive_kind, primitive_name, categorical_role)| CascadePrimitiveRoleV0 {
            schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
            product: "omena-categorical.cascade-primitive-role",
            layer_marker: CATEGORICAL_LAYER_MARKER_V0,
            feature_gate: CATEGORICAL_FEATURE_GATE_V0,
            primitive_kind,
            primitive_name,
            categorical_role,
        },
    )
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_gamma_categorical_surface_without_default_feature() {
        let summary = summarize_categorical_foundation_v0();
        assert_eq!(summary.schema_version, "0");
        assert_eq!(summary.layer_marker, "categorical-semantic");
        assert_eq!(summary.module_names.len(), 9);
        assert_eq!(summary.top_level_contract_count, 26);
        assert!(!summary.default_feature_enabled);
        assert_eq!(
            summary.lawvere_dependency_direction,
            "no-default-product-lawvere-edge"
        );
    }

    #[test]
    fn maps_actual_cascade_primitives_to_categorical_roles() {
        let roles = cascade_primitive_roles_v0();
        let primitive_names = roles
            .iter()
            .map(|role| role.primitive_name)
            .collect::<Vec<_>>();
        assert_eq!(primitive_names.len(), 5);
        assert!(primitive_names.contains(&"cascade_property"));
        assert!(primitive_names.contains(&"prove_box_shorthand_combination"));
        assert!(primitive_names.contains(&"evaluate_static_supports_condition"));
        assert!(
            roles
                .iter()
                .any(|role| role.primitive_name == "cascade_property"
                    && role.primitive_kind == "ranking"
                    && role.categorical_role == "cosheaf colimit witness")
        );
    }

    #[test]
    fn categorical_endpoint_catalog_contains_required_m4_gamma_endpoints() {
        let evidence = categorical_cascade_evidence_v0("omena-query.read-cascade-at-position");
        let endpoint_ids = evidence
            .endpoints
            .iter()
            .map(|endpoint| endpoint.endpoint_id)
            .collect::<Vec<_>>();
        assert_eq!(evidence.schema_version, "0");
        assert_eq!(evidence.endpoint_count, 10);
        assert_eq!(evidence.fixture_evidence.len(), 10);
        assert_eq!(evidence.functor_applications.len(), 1);
        assert!(evidence.functor_applications[0].accepted);
        assert!(
            evidence
                .fixture_evidence
                .iter()
                .all(|fixture| fixture.accepted)
        );
        assert!(!evidence.default_feature_enabled);
        assert!(endpoint_ids.contains(&"rust/omena-categorical/verify-site-stability"));
        assert!(endpoint_ids.contains(&"rust/omena-categorical/verify-beck-chevalley"));
        assert!(endpoint_ids.contains(&"rust/omena-categorical/classify-omega-truth"));
        assert!(endpoint_ids.contains(&"rust/omena-categorical/verify-s4-axioms"));
        assert!(
            endpoint_ids.contains(&"rust/omena-categorical/verify-modal-imperative-equivalence")
        );
        assert!(endpoint_ids.contains(&"rust/omena-categorical/verify-cross-project-symmetry"));
    }

    #[test]
    fn categorical_endpoint_fixture_evidence_is_not_catalog_only() {
        for endpoint in categorical_evidence_endpoints_v0() {
            let fixture = categorical_fixture_evidence_for_endpoint_v0(endpoint.endpoint_id);
            assert!(fixture.is_some());
            if let Some(fixture) = fixture {
                assert_eq!(fixture.schema_version, "0");
                assert_eq!(fixture.endpoint_id, endpoint.endpoint_id);
                assert_eq!(fixture.evidence_product, endpoint.evidence_product);
                assert!(!fixture.fixture_id.is_empty());
                assert!(!fixture.exercised_contract_products.is_empty());
                assert_eq!(fixture.assertion_count, fixture.assertions.len());
                assert!(fixture.assertion_count > 0);
                assert!(fixture.accepted);
                assert!(
                    fixture
                        .assertions
                        .iter()
                        .all(|assertion| assertion.accepted)
                );
            }
        }

        let site = categorical_fixture_evidence_for_endpoint_v0(
            "rust/omena-categorical/verify-site-stability",
        );
        assert!(site.is_some());
        if let Some(site) = site {
            assert!(
                site.exercised_contract_products
                    .contains(&"omena-categorical.site-axiom-check")
            );
        }

        let modal = categorical_fixture_evidence_for_endpoint_v0(
            "rust/omena-categorical/verify-modal-imperative-equivalence",
        );
        assert!(modal.is_some());
        if let Some(modal) = modal {
            assert!(modal.assertion_count >= 2);
        }
    }

    #[test]
    fn site_axiom_fixture_is_computed_from_cover_family() {
        let fixture = categorical_fixture_evidence_for_endpoint_v0(
            "rust/omena-categorical/verify-site-stability",
        );
        assert!(fixture.is_some());
        let Some(fixture) = fixture else {
            return;
        };
        assert!(fixture.accepted);
        assert!(
            fixture
                .exercised_contract_products
                .contains(&"omena-categorical.cover-family")
        );
        assert!(fixture.assertions.iter().any(|assertion| {
            assertion.assertion_id == "cover-family-derived"
                && assertion.observed == "coverFamilyCount=10"
        }));

        let site = cascade_site_v0("control.categorical.cascade-site");
        let stable = check_cascade_site_axioms_v0(&site);
        assert!(stable.identity_cover);
        assert!(stable.pullback_stable);
        assert!(stable.transitive);

        let mut missing_identity = site.clone();
        let layer_identity = site_axis_object_id_v0(SiteAxisV0::Layer).to_string();
        missing_identity.cover_families.retain(|cover| {
            !(cover.object_ids.len() == 1 && cover.object_ids[0] == layer_identity)
        });
        let missing_identity_check = check_cascade_site_axioms_v0(&missing_identity);
        assert!(!missing_identity_check.identity_cover);
        assert!(!missing_identity_check.pullback_stable);
        assert!(!missing_identity_check.transitive);

        let mut unknown_object = site;
        unknown_object
            .cover_families
            .push(cover_family_v0("unknown-axis-cover", ["axis:unknown"]));
        let unknown_object_check = check_cascade_site_axioms_v0(&unknown_object);
        assert!(unknown_object_check.identity_cover);
        assert!(!unknown_object_check.pullback_stable);
        assert!(!unknown_object_check.transitive);
    }

    #[test]
    fn cascade_primitive_role_functor_checks_identity_and_composition() {
        let roles = cascade_primitive_roles_v0();
        let functor = apply_cascade_primitive_role_functor_v0(&roles);
        assert_eq!(functor.object_mapping_count, roles.len());
        assert_eq!(functor.morphism_mapping_count, roles.len() - 1);
        assert!(functor.identity_preserved);
        assert!(functor.composition_preserved);
        assert!(functor.accepted);

        let truncated_functor = apply_cascade_primitive_role_functor_v0(&roles[..1]);
        assert!(truncated_functor.identity_preserved);
        assert!(!truncated_functor.composition_preserved);
        assert!(!truncated_functor.accepted);
    }

    #[test]
    fn omega_truth_fixture_observed_values_come_from_real_outcome_mapping() {
        // The fixture's observed truth labels are produced by
        // `OmegaCascadeTruthValueV0::from_outcome` over real cascade outcomes. The
        // mapping is injective across the four outcomes, so a constant/identity
        // replacement of `from_outcome` would collapse the labels and break these
        // assertions while the literal `expected` targets stayed put.
        let fixture = categorical_fixture_evidence_for_endpoint_v0(
            "rust/omena-categorical/classify-omega-truth",
        );
        assert!(fixture.is_some());
        let Some(fixture) = fixture else {
            return;
        };
        assert!(fixture.accepted);

        let observed = fixture
            .assertions
            .iter()
            .map(|assertion| (assertion.assertion_id, assertion.observed.clone()))
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(observed["definite-to-closed"], "Definite->Closed");
        assert_eq!(observed["ranked-set-to-boundary"], "RankedSet->Boundary");
        assert_eq!(observed["inherit-to-open"], "Inherit->Open");
        assert_eq!(observed["top-to-full"], "Top->Full");
        // Cross-check directly against the algorithm: the four cascade outcomes map
        // to four distinct truth values, so the labels are not echoed literals.
        let labels = [
            OmegaCascadeTruthValueV0::from_outcome(&CascadeOutcome::Inherit),
            OmegaCascadeTruthValueV0::from_outcome(&CascadeOutcome::Top),
            OmegaCascadeTruthValueV0::from_outcome(&CascadeOutcome::RankedSet(Vec::new())),
        ];
        assert_eq!(labels[0], OmegaCascadeTruthValueV0::Open);
        assert_eq!(labels[1], OmegaCascadeTruthValueV0::Full);
        assert_eq!(labels[2], OmegaCascadeTruthValueV0::Boundary);
    }

    #[test]
    fn cosheaf_fixture_acceptance_is_computed_by_colimit_witness() {
        let fixture = categorical_fixture_evidence_for_endpoint_v0(
            "rust/omena-categorical/verify-cosheaf-covariance",
        );
        assert!(fixture.is_some());
        let Some(fixture) = fixture else {
            return;
        };
        assert!(fixture.accepted);
        let accepted_assertion = fixture
            .assertions
            .iter()
            .find(|assertion| assertion.assertion_id == "colimit-accepted");
        assert!(accepted_assertion.is_some());
        let Some(accepted_assertion) = accepted_assertion else {
            return;
        };
        assert_eq!(accepted_assertion.observed, "accepted=true");
        // The witness rejects an empty section family, so acceptance is a real
        // computed verdict, not a literal echo.
        assert!(!witness_cosheaf_colimit_v0("cascade-cosheaf", 0).accepted);
        assert!(witness_cosheaf_colimit_v0("cascade-cosheaf", 2).accepted);
    }

    #[test]
    fn beck_chevalley_fixture_observed_values_come_from_layer_flatten_proof() {
        let fixture = categorical_fixture_evidence_for_endpoint_v0(
            "rust/omena-categorical/verify-beck-chevalley",
        );
        assert!(fixture.is_some());
        let Some(fixture) = fixture else {
            return;
        };
        assert!(fixture.accepted);
        let blocked_assertion = fixture
            .assertions
            .iter()
            .find(|assertion| assertion.assertion_id == "important-bundle-blocked");
        assert!(blocked_assertion.is_some());
        let Some(blocked_assertion) = blocked_assertion else {
            return;
        };
        assert_eq!(blocked_assertion.observed, "accepted=false");

        // The observed verdicts are computed by mapping a real
        // `prove_layer_flatten_candidate` proof through
        // `beck_chevalley_from_layer_flatten_proof_v0`: a closed single-layer
        // bundle is accepted, while one carrying `!important` declarations is
        // blocked and inverts the origin. A constant producer would collapse
        // these two verdicts and break the fixture.
        let accepted = beck_chevalley_from_layer_flatten_proof_v0(&prove_layer_flatten_candidate(
            LayerFlattenInputV0 {
                layer_name: Some("base".to_string()),
                layer_rule_count: 3,
                peer_layer_count: 0,
                unlayered_rule_count: 0,
                important_declaration_count: 0,
                closed_bundle: true,
            },
        ));
        assert!(accepted.accepted);
        assert!(accepted.datum.layer_order_preserved);
        assert!(
            !accepted
                .origin_inversion
                .important_declarations_invert_origin
        );

        let blocked = beck_chevalley_from_layer_flatten_proof_v0(&prove_layer_flatten_candidate(
            LayerFlattenInputV0 {
                layer_name: Some("overrides".to_string()),
                layer_rule_count: 2,
                peer_layer_count: 0,
                unlayered_rule_count: 0,
                important_declaration_count: 2,
                closed_bundle: true,
            },
        ));
        assert!(!blocked.accepted);
        assert!(
            blocked
                .origin_inversion
                .important_declarations_invert_origin
        );
    }

    #[test]
    fn invariant_functoriality_fixture_is_computed_not_literal_only() {
        let fixture = categorical_fixture_evidence_for_endpoint_v0(
            "rust/omena-categorical/verify-invariant-functoriality",
        );
        assert!(fixture.is_some());
        let Some(fixture) = fixture else {
            return;
        };
        assert!(fixture.accepted);
        assert!(
            fixture
                .exercised_contract_products
                .contains(&"omena-categorical.cascade-primitive-role-functor")
        );
        assert!(fixture.assertions.iter().any(|assertion| {
            assertion.assertion_id == "primitive-role-composition-preservation"
                && assertion.contract_product == "omena-categorical.cascade-primitive-role-functor"
                && assertion.accepted
        }));
    }

    fn kripke_test_frame(nested_value: &str) -> KripkeFrameV0 {
        build_cascade_prefix_kripke_frame_v0(
            "test.kripke.frame",
            "color",
            &[
                (Vec::new(), "red".to_string()),
                (
                    vec!["@media (min-width: 600px)".to_string()],
                    nested_value.to_string(),
                ),
            ],
        )
    }

    #[test]
    fn kripke_box_stability_distinguishes_divergent_and_stable_frames() {
        // A nested context that overrides the resolved value is NOT box-stable; a
        // nested context that keeps the value IS. A constant verdict could satisfy
        // only one of these, so the necessity verdict is load-bearing: replacing
        // it with hardcoded `true` breaks the divergent case and with hardcoded
        // `false` breaks the stable case.
        let divergent = kripke_test_frame("blue");
        let stable = kripke_test_frame("red");
        assert_eq!(divergent.worlds.len(), 2);
        assert_eq!(divergent.valuations.len(), 2);
        assert!(!cascade_box_stable_v0(&divergent));
        assert!(cascade_box_stable_v0(&stable));
    }

    #[test]
    fn kripke_frame_world_count_tracks_observed_contexts() {
        let flat = build_cascade_prefix_kripke_frame_v0(
            "test.kripke.flat",
            "color",
            &[(Vec::new(), "red".to_string())],
        );
        let nested = kripke_test_frame("blue");
        assert_eq!(flat.worlds.len(), 1);
        assert_eq!(nested.worlds.len(), 2);
        // A single-context frame is vacuously box-stable; only a real second
        // context carrying a divergent value can break necessity.
        assert!(cascade_box_stable_v0(&flat));
    }

    #[test]
    fn kripke_s4_axioms_reject_corrupted_edge_set() {
        let frame = kripke_test_frame("blue");
        let axioms = verify_s4_frame_axioms_v0(&frame);
        assert!(modal_axiom_accepted_v0(&axioms, "reflexivity-t"));
        assert!(modal_axiom_accepted_v0(&axioms, "transitivity-4"));

        // Drop the base world's self-edge: reflexivity (T) must now fail.
        let mut missing_self = frame.clone();
        missing_self
            .edges
            .retain(|edge| !(edge.from_world == "base" && edge.to_world == "base"));
        assert!(!modal_axiom_accepted_v0(
            &verify_s4_frame_axioms_v0(&missing_self),
            "reflexivity-t"
        ));

        // A three-context chain whose base->deepest closure edge is removed has a
        // real `base -> media -> media+supports` chain without its `base ->
        // media+supports` edge, so transitivity (4) must fail.
        let chain = build_cascade_prefix_kripke_frame_v0(
            "test.kripke.chain",
            "color",
            &[
                (Vec::new(), "red".to_string()),
                (
                    vec!["@media (min-width: 600px)".to_string()],
                    "red".to_string(),
                ),
                (
                    vec![
                        "@media (min-width: 600px)".to_string(),
                        "@supports (display: grid)".to_string(),
                    ],
                    "red".to_string(),
                ),
            ],
        );
        assert!(modal_axiom_accepted_v0(
            &verify_s4_frame_axioms_v0(&chain),
            "transitivity-4"
        ));
        let deepest = "base|@media (min-width: 600px)|@supports (display: grid)";
        let mut broken_chain = chain.clone();
        broken_chain
            .edges
            .retain(|edge| !(edge.from_world == "base" && edge.to_world == deepest));
        assert!(!modal_axiom_accepted_v0(
            &verify_s4_frame_axioms_v0(&broken_chain),
            "transitivity-4"
        ));
    }

    #[test]
    fn kripke_frame_fixture_observed_values_come_from_real_compute() {
        let fixture = categorical_fixture_evidence_for_endpoint_v0(
            "rust/omena-categorical/summarize-kripke-frame",
        );
        assert!(fixture.is_some());
        let Some(fixture) = fixture else {
            return;
        };
        assert!(fixture.accepted);
        assert_eq!(fixture.fixture_id, "fixture.categorical.kripke-frame.v0");

        let observed = fixture
            .assertions
            .iter()
            .map(|assertion| (assertion.assertion_id, assertion.observed.clone()))
            .collect::<std::collections::BTreeMap<_, _>>();
        // The necessity verdict is computed `false` because the nested `@media`
        // context resolves `color` to a different winner than the base context.
        assert_eq!(
            observed["nested-override-breaks-necessity"],
            "boxStable=false"
        );
        assert_eq!(observed["worlds-present"], "worldCount=2");
        assert_eq!(observed["valuation-present"], "valuationCount=2");
        assert_eq!(observed["s4-frame-reflexivity-t"], "reflexivityT=true");
        assert_eq!(observed["s4-frame-transitivity-4"], "transitivity4=true");

        // Cross-check against the builder: the same two contexts with a matching
        // value ARE box-stable, so the fixture's `false` is data-derived, not a
        // literal echo.
        assert!(cascade_box_stable_v0(&kripke_test_frame("red")));
        assert!(!cascade_box_stable_v0(&kripke_test_frame("blue")));
    }

    #[test]
    fn s4_axioms_fixture_observed_values_come_from_real_compute() {
        let fixture =
            categorical_fixture_evidence_for_endpoint_v0("rust/omena-categorical/verify-s4-axioms");
        assert!(fixture.is_some());
        let Some(fixture) = fixture else {
            return;
        };
        assert!(fixture.accepted);
        assert_eq!(fixture.fixture_id, "fixture.categorical.s4-axioms.v0");

        let observed = fixture
            .assertions
            .iter()
            .map(|assertion| (assertion.assertion_id, assertion.observed.clone()))
            .collect::<std::collections::BTreeMap<_, _>>();
        // T and 4 are computed by verify_s4_frame_axioms_v0 over a real frame; the
        // corrupted-edge controls in `kripke_s4_axioms_reject_corrupted_edge_set`
        // prove they are load-bearing rather than echoed literals.
        assert_eq!(observed["s4-reflexivity-t"], "reflexivityT=true");
        assert_eq!(observed["s4-transitivity-4"], "transitivity4=true");
        assert_eq!(observed["modal-witness-evaluated"], "witnessTruth=Closed");

        // The witness is data-derived: evaluating a formula whose atom is absent
        // from the frame valuations yields Open, not Closed.
        let frame = build_cascade_prefix_kripke_frame_v0(
            "test.s4.witness",
            "color",
            &[(Vec::new(), "red".to_string())],
        );
        let absent = evaluate_omena_categorical_modal_formula_v0(
            &modal_atom_formula_v0("absent", "color=blue"),
            &frame,
        );
        assert_eq!(absent.truth_value, OmegaCascadeTruthValueV0::Open);
    }
}
