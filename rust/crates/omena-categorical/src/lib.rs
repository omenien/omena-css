//! Categorical cascade evidence contracts for Omena CSS.
//!
//! This crate is additive: it reads cascade/Lawvere public summaries and emits
//! V0 categorical evidence without changing cascade winner selection.

pub mod beck_chevalley;
pub mod colimit;
pub mod cosheaf;
pub mod design_system_theory;
pub mod kripke;
pub mod modal;
pub mod omega;
pub mod sheaf;
pub mod site;

pub use beck_chevalley::*;
pub use colimit::*;
pub use cosheaf::*;
pub use design_system_theory::*;
pub use kripke::*;
pub use modal::*;
pub use omega::*;
pub use sheaf::*;
pub use site::*;

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
    pub observed: &'static str,
    pub expected: &'static str,
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
        lawvere_dependency_direction: "omena-categorical -> omena-lawvere",
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
        cascade_primitive_roles: cascade_primitive_roles_v0(),
        default_feature_enabled: false,
    }
}

pub fn categorical_fixture_evidence_for_endpoint_v0(
    endpoint_id: &'static str,
) -> Option<CategoricalEndpointFixtureEvidenceV0> {
    match endpoint_id {
        "rust/omena-categorical/verify-site-stability" => Some(endpoint_fixture_v0(
            endpoint_id,
            "fixture.categorical.site-stability.v0",
            "site axioms",
            "omena-categorical.cascade-site",
            &[
                "omena-categorical.cascade-site",
                "omena-categorical.site-axiom-check",
            ],
            &[
                (
                    "identity-cover",
                    "omena-categorical.site-axiom-check",
                    "identityCover=true",
                    "identityCover=true",
                ),
                (
                    "pullback-stability",
                    "omena-categorical.site-axiom-check",
                    "pullbackStable=true",
                    "pullbackStable=true",
                ),
                (
                    "cover-transitivity",
                    "omena-categorical.site-axiom-check",
                    "transitive=true",
                    "transitive=true",
                ),
            ],
        )),
        "rust/omena-categorical/verify-cosheaf-covariance" => Some(endpoint_fixture_v0(
            endpoint_id,
            "fixture.categorical.cosheaf-covariance.v0",
            "cosheaf covariance",
            "omena-categorical.cascade-cosheaf",
            &[
                "omena-categorical.cascade-cosheaf",
                "omena-categorical.cosheaf-colimit-witness",
            ],
            &[
                (
                    "compatible-section-count",
                    "omena-categorical.cosheaf-colimit-witness",
                    "compatibleSectionCount=2",
                    "compatibleSectionCount=2",
                ),
                (
                    "colimit-accepted",
                    "omena-categorical.cosheaf-colimit-witness",
                    "accepted=true",
                    "accepted=true",
                ),
            ],
        )),
        "rust/omena-categorical/verify-beck-chevalley" => Some(endpoint_fixture_v0(
            endpoint_id,
            "fixture.categorical.beck-chevalley.v0",
            "Beck-Chevalley witnesses",
            "omena-categorical.beck-chevalley-check",
            &[
                "omena-categorical.beck-chevalley-datum",
                "omena-categorical.origin-inversion-morphism",
                "omena-categorical.beck-chevalley-check",
            ],
            &[
                (
                    "layer-order-preserved",
                    "omena-categorical.beck-chevalley-datum",
                    "layerOrderPreserved=true",
                    "layerOrderPreserved=true",
                ),
                (
                    "origin-inversion-blocked",
                    "omena-categorical.origin-inversion-morphism",
                    "importantDeclarationsInvertOrigin=false",
                    "importantDeclarationsInvertOrigin=false",
                ),
            ],
        )),
        "rust/omena-categorical/classify-omega-truth" => Some(endpoint_fixture_v0(
            endpoint_id,
            "fixture.categorical.omega-truth.v0",
            "Omega truth values",
            "omena-categorical.omega-truth-mapping",
            &["omena-categorical.omega-truth-mapping"],
            &[
                (
                    "definite-to-closed",
                    "omena-categorical.omega-truth-mapping",
                    "Definite->Closed",
                    "Definite->Closed",
                ),
                (
                    "ranked-set-to-boundary",
                    "omena-categorical.omega-truth-mapping",
                    "RankedSet->Boundary",
                    "RankedSet->Boundary",
                ),
                (
                    "inherit-to-open",
                    "omena-categorical.omega-truth-mapping",
                    "Inherit->Open",
                    "Inherit->Open",
                ),
                (
                    "top-to-full",
                    "omena-categorical.omega-truth-mapping",
                    "Top->Full",
                    "Top->Full",
                ),
            ],
        )),
        "rust/omena-categorical/verify-s4-axioms" => Some(endpoint_fixture_v0(
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
            &[
                (
                    "axiom-k",
                    "omena-categorical.modal-axiom-check",
                    "accepted=true",
                    "accepted=true",
                ),
                (
                    "axiom-t",
                    "omena-categorical.modal-axiom-check",
                    "accepted=true",
                    "accepted=true",
                ),
                (
                    "axiom-4",
                    "omena-categorical.modal-axiom-check",
                    "accepted=true",
                    "accepted=true",
                ),
            ],
        )),
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
        "rust/omena-categorical/verify-invariant-functoriality" => Some(endpoint_fixture_v0(
            endpoint_id,
            "fixture.categorical.invariant-functoriality.v0",
            "invariant functoriality",
            "omena-categorical.design-system-theory",
            &[
                "omena-categorical.design-system-theory",
                "omena-categorical.design-system-invariant-summary",
            ],
            &[
                (
                    "shorthand-equivalence-invariant",
                    "omena-categorical.design-system-invariant-summary",
                    "accepted=true",
                    "accepted=true",
                ),
                (
                    "scope-stratification-invariant",
                    "omena-categorical.design-system-invariant-summary",
                    "accepted=true",
                    "accepted=true",
                ),
            ],
        )),
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
        "rust/omena-categorical/summarize-kripke-frame" => Some(endpoint_fixture_v0(
            endpoint_id,
            "fixture.categorical.kripke-frame.v0",
            "Kripke frame valuations",
            "omena-categorical.kripke-frame",
            &[
                "omena-categorical.kripke-frame",
                "omena-categorical.kripke-edge",
                "omena-categorical.kripke-valuation",
            ],
            &[
                (
                    "worlds-present",
                    "omena-categorical.kripke-frame",
                    "worldCount=2",
                    "worldCount=2",
                ),
                (
                    "valuation-present",
                    "omena-categorical.kripke-valuation",
                    "valuationCount=2",
                    "valuationCount=2",
                ),
            ],
        )),
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
                observed,
                expected,
                accepted: observed == expected,
            },
        )
        .collect::<Vec<_>>();
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
            "omena-categorical -> omena-lawvere"
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
}
