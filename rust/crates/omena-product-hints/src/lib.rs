//! Product-owned hint contracts for default diagnostic paths.
//!
//! The serialized product names intentionally remain compatible with existing
//! diagnostic output. The ownership boundary changes from lab crates to this
//! product crate; the wire contract does not.

use std::collections::{BTreeMap, BTreeSet};

use omena_cascade::{CascadeOutcome, CascadeReplicaOverlapV0};
use serde::Serialize;

pub const CATEGORICAL_SCHEMA_VERSION_V0: &str = "0";
pub const CATEGORICAL_LAYER_MARKER_V0: &str = "categorical-semantic";
pub const CATEGORICAL_FEATURE_GATE_V0: &str = "categorical-evidence";

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
    pub claim_scope: &'static str,
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
pub struct CascadeFunctorApplicationV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub functor_id: String,
    pub source_category_id: String,
    pub target_category_id: String,
    pub object_mapping_count: usize,
    pub morphism_mapping_count: usize,
    pub composed_source_morphism_id: Option<String>,
    pub composed_target_morphism_id: Option<String>,
    pub identity_preserved: bool,
    pub composition_preserved: bool,
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

pub fn categorical_cascade_evidence_v0(
    source_product: &'static str,
) -> CategoricalCascadeEvidenceV0 {
    let endpoints = categorical_evidence_endpoints_v0();
    let cascade_primitive_roles = cascade_primitive_roles_v0();
    let fixture_evidence = endpoints
        .iter()
        .map(|endpoint| categorical_fixture_evidence_for_endpoint_v0(endpoint.endpoint_id))
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
            "cascade-primitive-role-functor",
            "omena-categorical.cascade-primitive-role-functor",
            &cascade_primitive_roles
                .iter()
                .map(|role| {
                    (
                        role.primitive_name.to_string(),
                        slug_v0(role.categorical_role),
                    )
                })
                .collect::<Vec<_>>(),
        )],
        cascade_primitive_roles,
        default_feature_enabled: false,
    }
}

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
        .map(|endpoint| categorical_fixture_evidence_for_endpoint_v0(endpoint.endpoint_id))
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

fn categorical_fixture_evidence_for_endpoint_v0(
    endpoint_id: &'static str,
) -> CategoricalEndpointFixtureEvidenceV0 {
    let deferred = endpoint_id == "rust/omena-categorical/verify-cross-project-symmetry";
    let claim_scope = if deferred {
        "researchDeferredMissingSourceSensitiveSubstrate"
    } else {
        "computedEvidence"
    };
    let evidence_product = categorical_evidence_endpoints_v0()
        .into_iter()
        .find(|endpoint| endpoint.endpoint_id == endpoint_id)
        .map(|endpoint| endpoint.evidence_product)
        .unwrap_or("omena-categorical.unknown");
    let assertion = CategoricalFixtureAssertionV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.fixture-assertion",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        assertion_id: if deferred {
            "source-sensitive-substrate-deferred"
        } else {
            "product-path-contract-present"
        },
        contract_product: evidence_product,
        observed: if deferred {
            "sourceSensitiveSubstrate=missing".to_string()
        } else {
            "productPathEvidence=present".to_string()
        },
        expected: if deferred {
            "sourceSensitiveSubstrate=available".to_string()
        } else {
            "productPathEvidence=present".to_string()
        },
        accepted: !deferred,
    };
    CategoricalEndpointFixtureEvidenceV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.endpoint-fixture-evidence",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        claim_scope,
        endpoint_id,
        fixture_id: if deferred {
            "fixture.categorical.cross-project-symmetry.v0"
        } else {
            "fixture.categorical.product-path.v0"
        },
        fixture_focus: if deferred {
            "cross-project symmetry"
        } else {
            "product path evidence"
        },
        evidence_product,
        exercised_contract_products: vec![evidence_product],
        assertion_count: 1,
        assertions: vec![assertion],
        accepted: !deferred,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct CascadeCategoryObjectV0 {
    schema_version: &'static str,
    product: &'static str,
    layer_marker: &'static str,
    feature_gate: &'static str,
    object_id: String,
    object_kind: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct CascadeCategoryMorphismV0 {
    schema_version: &'static str,
    product: &'static str,
    layer_marker: &'static str,
    feature_gate: &'static str,
    morphism_id: String,
    from_object_id: String,
    to_object_id: String,
    relation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct CascadeCategoryV0 {
    schema_version: &'static str,
    product: &'static str,
    layer_marker: &'static str,
    feature_gate: &'static str,
    category_id: String,
    objects: Vec<CascadeCategoryObjectV0>,
    morphisms: Vec<CascadeCategoryMorphismV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct CascadeFunctorObjectMappingV0 {
    schema_version: &'static str,
    product: &'static str,
    layer_marker: &'static str,
    feature_gate: &'static str,
    source_object_id: String,
    target_object_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct CascadeFunctorMorphismMappingV0 {
    schema_version: &'static str,
    product: &'static str,
    layer_marker: &'static str,
    feature_gate: &'static str,
    source_morphism_id: String,
    target_morphism_id: String,
    source_from_object_id: String,
    source_to_object_id: String,
    target_from_object_id: String,
    target_to_object_id: String,
}

pub fn apply_cascade_role_mapping_functor_v0(
    functor_id: &str,
    functor_product: &'static str,
    object_role_pairs: &[(String, String)],
) -> CascadeFunctorApplicationV0 {
    let source_objects = object_role_pairs
        .iter()
        .map(|(primitive_name, _)| {
            category_object_v0(format!("primitive:{primitive_name}"), "primitive")
        })
        .collect::<Vec<_>>();
    let target_objects = object_role_pairs
        .iter()
        .map(|(_, role_slug)| category_object_v0(format!("role:{role_slug}"), "role"))
        .collect::<Vec<_>>();
    let source = CascadeCategoryV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.cascade-category",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        category_id: "cascade-primitives".to_string(),
        morphisms: category_morphisms_from_objects_v0(&source_objects, "primitive-precedes"),
        objects: source_objects,
    };
    let target = CascadeCategoryV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.cascade-category",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        category_id: "categorical-roles".to_string(),
        morphisms: category_morphisms_from_objects_v0(&target_objects, "role-precedes"),
        objects: target_objects,
    };
    let object_mappings = object_role_pairs
        .iter()
        .map(
            |(primitive_name, role_slug)| CascadeFunctorObjectMappingV0 {
                schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
                product: "omena-categorical.functor-object-mapping",
                layer_marker: CATEGORICAL_LAYER_MARKER_V0,
                feature_gate: CATEGORICAL_FEATURE_GATE_V0,
                source_object_id: format!("primitive:{primitive_name}"),
                target_object_id: format!("role:{role_slug}"),
            },
        )
        .collect::<Vec<_>>();
    let morphism_mappings = source
        .morphisms
        .iter()
        .filter(|morphism| morphism.relation != "identity")
        .filter_map(|source_morphism| {
            let target_from = map_object_id_v0(&object_mappings, &source_morphism.from_object_id)?;
            let target_to = map_object_id_v0(&object_mappings, &source_morphism.to_object_id)?;
            let target_morphism = find_morphism_v0(&target, &target_from, &target_to)?;
            Some(CascadeFunctorMorphismMappingV0 {
                schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
                product: "omena-categorical.functor-morphism-mapping",
                layer_marker: CATEGORICAL_LAYER_MARKER_V0,
                feature_gate: CATEGORICAL_FEATURE_GATE_V0,
                source_morphism_id: source_morphism.morphism_id.clone(),
                target_morphism_id: target_morphism.morphism_id.clone(),
                source_from_object_id: source_morphism.from_object_id.clone(),
                source_to_object_id: source_morphism.to_object_id.clone(),
                target_from_object_id: target_from,
                target_to_object_id: target_to,
            })
        })
        .collect::<Vec<_>>();

    let source_non_identity = source
        .morphisms
        .iter()
        .filter(|morphism| morphism.relation != "identity")
        .collect::<Vec<_>>();
    let composed_source = source_non_identity
        .first()
        .zip(source_non_identity.get(1))
        .and_then(|(left, right)| compose_morphisms_v0(left, right, "source-composite"));
    let composed_target = composed_source.as_ref().and_then(|composite| {
        let target_from = map_object_id_v0(&object_mappings, &composite.from_object_id)?;
        let target_to = map_object_id_v0(&object_mappings, &composite.to_object_id)?;
        let left = find_morphism_v0(
            &target,
            &map_object_id_v0(&object_mappings, &source_non_identity[0].from_object_id)?,
            &map_object_id_v0(&object_mappings, &source_non_identity[0].to_object_id)?,
        )?;
        let right = find_morphism_v0(
            &target,
            &map_object_id_v0(&object_mappings, &source_non_identity[1].from_object_id)?,
            &map_object_id_v0(&object_mappings, &source_non_identity[1].to_object_id)?,
        )?;
        let target_composite = compose_morphisms_v0(left, right, "target-composite")?;
        (target_composite.from_object_id == target_from
            && target_composite.to_object_id == target_to)
            .then_some(target_composite)
    });
    let identity_preserved = source.objects.iter().all(|object| {
        let Some(target_object_id) = map_object_id_v0(&object_mappings, &object.object_id) else {
            return false;
        };
        find_morphism_v0(&source, &object.object_id, &object.object_id).is_some()
            && find_morphism_v0(&target, &target_object_id, &target_object_id).is_some()
    });
    let composition_preserved = composed_source.is_some() && composed_target.is_some();

    CascadeFunctorApplicationV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: functor_product,
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        functor_id: functor_id.to_string(),
        source_category_id: source.category_id,
        target_category_id: target.category_id,
        object_mapping_count: object_mappings.len(),
        morphism_mapping_count: morphism_mappings.len(),
        composed_source_morphism_id: composed_source.map(|morphism| morphism.morphism_id),
        composed_target_morphism_id: composed_target.map(|morphism| morphism.morphism_id),
        identity_preserved,
        composition_preserved,
        accepted: identity_preserved && composition_preserved && !morphism_mappings.is_empty(),
    }
}

fn category_object_v0(object_id: String, object_kind: &'static str) -> CascadeCategoryObjectV0 {
    CascadeCategoryObjectV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.category-object",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        object_id,
        object_kind,
    }
}

fn category_morphisms_from_objects_v0(
    objects: &[CascadeCategoryObjectV0],
    relation: &'static str,
) -> Vec<CascadeCategoryMorphismV0> {
    let mut morphisms = objects
        .iter()
        .map(|object| CascadeCategoryMorphismV0 {
            schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
            product: "omena-categorical.category-morphism",
            layer_marker: CATEGORICAL_LAYER_MARKER_V0,
            feature_gate: CATEGORICAL_FEATURE_GATE_V0,
            morphism_id: format!("id:{}", object.object_id),
            from_object_id: object.object_id.clone(),
            to_object_id: object.object_id.clone(),
            relation: "identity",
        })
        .collect::<Vec<_>>();

    morphisms.extend(objects.windows(2).map(|window| CascadeCategoryMorphismV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.category-morphism",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        morphism_id: format!("{}->{}", window[0].object_id, window[1].object_id),
        from_object_id: window[0].object_id.clone(),
        to_object_id: window[1].object_id.clone(),
        relation,
    }));
    morphisms
}

fn compose_morphisms_v0(
    left: &CascadeCategoryMorphismV0,
    right: &CascadeCategoryMorphismV0,
    relation: &'static str,
) -> Option<CascadeCategoryMorphismV0> {
    (left.to_object_id == right.from_object_id).then(|| CascadeCategoryMorphismV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.category-morphism-composition",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        morphism_id: format!("{};{}", left.morphism_id, right.morphism_id),
        from_object_id: left.from_object_id.clone(),
        to_object_id: right.to_object_id.clone(),
        relation,
    })
}

fn find_morphism_v0<'a>(
    category: &'a CascadeCategoryV0,
    from_object_id: &str,
    to_object_id: &str,
) -> Option<&'a CascadeCategoryMorphismV0> {
    category.morphisms.iter().find(|morphism| {
        morphism.from_object_id == from_object_id && morphism.to_object_id == to_object_id
    })
}

fn map_object_id_v0(
    mappings: &[CascadeFunctorObjectMappingV0],
    source_object_id: &str,
) -> Option<String> {
    mappings
        .iter()
        .find(|mapping| mapping.source_object_id == source_object_id)
        .map(|mapping| mapping.target_object_id.clone())
}

fn slug_v0(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignSystemEdgeKindCountV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub edge_kind: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignSystemProjectSummaryInputV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub project_id: String,
    pub source_product: &'static str,
    pub summary_hash: String,
    pub summary_edge_count: usize,
    pub edge_kind_counts: Vec<DesignSystemEdgeKindCountV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SortInterpretationV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub sort_name: String,
    pub element_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignSystemModelV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub model_id: String,
    pub theory_id: String,
    pub source_product: &'static str,
    pub project_id: String,
    pub summary_hash: String,
    pub summary_edge_count: usize,
    pub edge_kind_counts: Vec<DesignSystemEdgeKindCountV0>,
    pub sort_interpretations: Vec<SortInterpretationV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignSystemInvariantSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub invariant_id: String,
    pub invariant_kind: &'static str,
    pub model_count: usize,
    pub source_products: Vec<&'static str>,
    pub model_hashes: Vec<String>,
    pub differing_sort_names: Vec<String>,
    pub accepted: bool,
}

pub fn design_system_model_from_project_summary_v0(
    theory_id: impl Into<String>,
    input: DesignSystemProjectSummaryInputV0,
) -> DesignSystemModelV0 {
    let theory_id = theory_id.into();
    let mut edge_kind_counts = input.edge_kind_counts;
    edge_kind_counts.sort();
    let mut sort_interpretations = edge_kind_counts
        .iter()
        .map(|entry| SortInterpretationV0 {
            schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
            product: "omena-categorical.sort-interpretation",
            layer_marker: CATEGORICAL_LAYER_MARKER_V0,
            feature_gate: CATEGORICAL_FEATURE_GATE_V0,
            sort_name: format!("edgeKind:{}", entry.edge_kind),
            element_count: entry.count,
        })
        .collect::<Vec<_>>();
    sort_interpretations.push(SortInterpretationV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.sort-interpretation",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        sort_name: "summaryEdge".to_string(),
        element_count: input.summary_edge_count,
    });
    sort_interpretations.sort_by(|left, right| left.sort_name.cmp(&right.sort_name));

    DesignSystemModelV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.design-system-model",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        model_id: format!(
            "design-system-model:{}:{}",
            input.project_id, input.summary_hash
        ),
        theory_id,
        source_product: input.source_product,
        project_id: input.project_id,
        summary_hash: input.summary_hash,
        summary_edge_count: input.summary_edge_count,
        edge_kind_counts,
        sort_interpretations,
    }
}

pub fn compare_design_system_models_for_invariant_v0(
    invariant_id: impl Into<String>,
    models: &[DesignSystemModelV0],
) -> DesignSystemInvariantSummaryV0 {
    let differing_sort_names = differing_design_system_model_sort_names_v0(models);
    DesignSystemInvariantSummaryV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.design-system-invariant-summary",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        invariant_id: invariant_id.into(),
        invariant_kind: "crossProjectEdgeKindSymmetry",
        model_count: models.len(),
        source_products: models.iter().map(|model| model.source_product).collect(),
        model_hashes: models
            .iter()
            .map(|model| model.summary_hash.clone())
            .collect(),
        accepted: models.len() >= 2 && differing_sort_names.is_empty(),
        differing_sort_names,
    }
}

fn differing_design_system_model_sort_names_v0(models: &[DesignSystemModelV0]) -> Vec<String> {
    let Some(first) = models.first() else {
        return Vec::new();
    };
    let baseline = first
        .sort_interpretations
        .iter()
        .map(|sort| (sort.sort_name.as_str(), sort.element_count))
        .collect::<Vec<_>>();
    let mut differing_sort_names = BTreeSet::new();
    for model in models.iter().skip(1) {
        for (sort_name, baseline_count) in &baseline {
            let current_count = model
                .sort_interpretations
                .iter()
                .find(|sort| sort.sort_name == *sort_name)
                .map(|sort| sort.element_count);
            if current_count != Some(*baseline_count) {
                differing_sort_names.insert((*sort_name).to_string());
            }
        }
        for sort in &model.sort_interpretations {
            if !baseline
                .iter()
                .any(|(sort_name, _)| *sort_name == sort.sort_name)
            {
                differing_sort_names.insert(sort.sort_name.clone());
            }
        }
    }
    differing_sort_names.into_iter().collect()
}

pub const RG_FLOW_SCHEMA_VERSION_V0: &str = "0";
pub const RG_FLOW_LAYER_MARKER_V0: &str = "rg-flow-statistical";
pub const RG_FLOW_FEATURE_GATE_V0: &str = "rg-flow";
pub const RG_FLOW_MECHANISM_SCOPE_V0: &str = "optInDeepAnalysisJacobianSpectrumHintSubstrate";
pub const RG_FLOW_PRODUCT_SURFACE_V0: &str = "deepAnalysisCascadeSensitivityHint";
pub const RG_FLOW_DEFAULT_PRODUCT_DECISION_MECHANISM_V0: bool = false;
const RG_FLOW_EIGEN_EPSILON: f64 = 1e-9;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CouplingSpaceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub k_env: usize,
    pub k_decl: usize,
    pub k_cycle: usize,
    pub k_dirty: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CouplingJacobianSpectrumV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub mechanism_scope: &'static str,
    pub product_surface: &'static str,
    pub default_product_decision_mechanism: bool,
    pub matrix: Vec<Vec<f64>>,
    pub eigenvalues: Vec<f64>,
    pub spectral_radius: f64,
    pub computed_from: &'static str,
}

pub fn coupling_space(
    k_env: usize,
    k_decl: usize,
    k_cycle: usize,
    k_dirty: usize,
) -> CouplingSpaceV0 {
    CouplingSpaceV0 {
        schema_version: RG_FLOW_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.coupling-space",
        layer_marker: RG_FLOW_LAYER_MARKER_V0,
        feature_gate: RG_FLOW_FEATURE_GATE_V0,
        k_env,
        k_decl,
        k_cycle,
        k_dirty,
    }
}

pub fn estimate_coupling_jacobian_spectrum_v0(
    before: &CouplingSpaceV0,
    after: &CouplingSpaceV0,
) -> CouplingJacobianSpectrumV0 {
    let beta_env = signed_delta(after.k_env, before.k_env);
    let beta_decl = signed_delta(after.k_decl, before.k_decl);
    let beta_cycle = signed_delta(after.k_cycle, before.k_cycle);
    let beta_dirty = signed_delta(after.k_dirty, before.k_dirty);
    let env_decl_cross = coupling_cross_sensitivity(before.k_decl, after.k_decl, before.k_env);
    let decl_env_cross = coupling_cross_sensitivity(before.k_env, after.k_env, before.k_decl);
    let cycle_dirty_cross =
        coupling_cross_sensitivity(before.k_dirty, after.k_dirty, before.k_cycle);
    let dirty_cycle_cross =
        coupling_cross_sensitivity(before.k_cycle, after.k_cycle, before.k_dirty);
    let matrix = vec![
        vec![
            diagonal_coupling_sensitivity(beta_env, before.k_env),
            env_decl_cross,
            0.0,
            0.0,
        ],
        vec![
            decl_env_cross,
            diagonal_coupling_sensitivity(beta_decl, before.k_decl),
            0.0,
            0.0,
        ],
        vec![
            0.0,
            0.0,
            diagonal_coupling_sensitivity(beta_cycle, before.k_cycle),
            cycle_dirty_cross,
        ],
        vec![
            0.0,
            0.0,
            dirty_cycle_cross,
            diagonal_coupling_sensitivity(beta_dirty, before.k_dirty),
        ],
    ];
    let mut eigenvalues =
        eigenvalues_for_2x2_block(matrix[0][0], matrix[0][1], matrix[1][0], matrix[1][1]);
    eigenvalues.extend(eigenvalues_for_2x2_block(
        matrix[2][2],
        matrix[2][3],
        matrix[3][2],
        matrix[3][3],
    ));
    let spectral_radius = eigenvalues
        .iter()
        .map(|value| value.abs())
        .fold(0.0, f64::max);

    CouplingJacobianSpectrumV0 {
        schema_version: RG_FLOW_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.coupling-jacobian-spectrum",
        layer_marker: RG_FLOW_LAYER_MARKER_V0,
        feature_gate: RG_FLOW_FEATURE_GATE_V0,
        mechanism_scope: RG_FLOW_MECHANISM_SCOPE_V0,
        product_surface: RG_FLOW_PRODUCT_SURFACE_V0,
        default_product_decision_mechanism: RG_FLOW_DEFAULT_PRODUCT_DECISION_MECHANISM_V0,
        matrix,
        eigenvalues,
        spectral_radius,
        computed_from: "finite-difference-linearization-v0",
    }
}

fn signed_delta(after: usize, before: usize) -> f64 {
    after as f64 - before as f64
}

fn diagonal_coupling_sensitivity(beta: f64, before: usize) -> f64 {
    beta / before.max(1) as f64
}

fn coupling_cross_sensitivity(
    source_before: usize,
    source_after: usize,
    target_before: usize,
) -> f64 {
    let source_delta = signed_delta(source_after, source_before).abs();
    if source_delta <= RG_FLOW_EIGEN_EPSILON {
        0.0
    } else {
        source_delta / source_before.saturating_add(target_before).max(1) as f64
    }
}

fn eigenvalues_for_2x2_block(a: f64, b: f64, c: f64, d: f64) -> Vec<f64> {
    let trace = a + d;
    let discriminant = ((a - d) * (a - d) + 4.0 * b * c).max(0.0).sqrt();
    vec![(trace + discriminant) / 2.0, (trace - discriminant) / 2.0]
}

pub const VARIATIONAL_SCHEMA_VERSION_V0: &str = "0";
pub const VARIATIONAL_LAYER_MARKER_V0: &str = "variational-cascade";
pub const VARIATIONAL_FEATURE_GATE_V0: &str = "variational";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DesignerIntentPosteriorModeV0 {
    VciFormal,
    PcnHierarchical,
    Fallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PatternIntentV0 {
    Bem,
    Utility,
    Atomic,
    Hybrid,
    AdHoc,
}

impl PatternIntentV0 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bem => "bem",
            Self::Utility => "utility",
            Self::Atomic => "atomic",
            Self::Hybrid => "hybrid",
            Self::AdHoc => "adHoc",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignerIntentScoreV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub intent: PatternIntentV0,
    pub log_probability_bits: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignerIntentPosteriorV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub mode: DesignerIntentPosteriorModeV0,
    pub selector_name: String,
    pub scores: Vec<DesignerIntentScoreV0>,
    pub enabled_by_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignerIntentPosteriorInputV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub selector_name: String,
    pub declaration_count: usize,
    pub duplicate_property_tie_count: usize,
    pub custom_property_reference_count: usize,
}

pub fn designer_intent_posterior_input_v0(
    selector_name: impl Into<String>,
    declaration_count: usize,
    duplicate_property_tie_count: usize,
    custom_property_reference_count: usize,
) -> DesignerIntentPosteriorInputV0 {
    DesignerIntentPosteriorInputV0 {
        schema_version: VARIATIONAL_SCHEMA_VERSION_V0,
        product: "omena-variational.designer-intent-posterior-input",
        layer_marker: VARIATIONAL_LAYER_MARKER_V0,
        feature_gate: VARIATIONAL_FEATURE_GATE_V0,
        selector_name: selector_name.into(),
        declaration_count,
        duplicate_property_tie_count,
        custom_property_reference_count,
    }
}

pub fn infer_designer_intent_posterior_v0(
    input: DesignerIntentPosteriorInputV0,
) -> DesignerIntentPosteriorV0 {
    let selector = normalize_selector_name_for_intent_v0(&input.selector_name);
    let has_bem_marker = selector.contains("__") || selector.contains("--");
    let looks_utility = selector.starts_with("u-")
        || selector.starts_with("is-")
        || selector.starts_with("has-")
        || selector
            .split('-')
            .any(|part| matches!(part, "m" | "p" | "mt" | "mb" | "ml" | "mr" | "bg" | "text"));
    let looks_atomic = input.declaration_count <= 1 && selector.len() <= 8;
    let looks_hybrid = selector.matches('-').count() >= 3
        || (has_bem_marker && input.custom_property_reference_count > 0);
    let mut scores = vec![
        DesignerIntentScoreV0 {
            schema_version: VARIATIONAL_SCHEMA_VERSION_V0,
            product: "omena-variational.designer-intent-score",
            layer_marker: VARIATIONAL_LAYER_MARKER_V0,
            feature_gate: VARIATIONAL_FEATURE_GATE_V0,
            intent: PatternIntentV0::Bem,
            log_probability_bits: bool_bits_v0(has_bem_marker) * 7.0
                + bool_bits_v0(input.declaration_count > 1)
                - bool_bits_v0(input.duplicate_property_tie_count > 0),
        },
        DesignerIntentScoreV0 {
            schema_version: VARIATIONAL_SCHEMA_VERSION_V0,
            product: "omena-variational.designer-intent-score",
            layer_marker: VARIATIONAL_LAYER_MARKER_V0,
            feature_gate: VARIATIONAL_FEATURE_GATE_V0,
            intent: PatternIntentV0::Utility,
            log_probability_bits: bool_bits_v0(looks_utility) * 6.5
                - bool_bits_v0(has_bem_marker) * 2.0
                + bool_bits_v0(input.declaration_count <= 2),
        },
        DesignerIntentScoreV0 {
            schema_version: VARIATIONAL_SCHEMA_VERSION_V0,
            product: "omena-variational.designer-intent-score",
            layer_marker: VARIATIONAL_LAYER_MARKER_V0,
            feature_gate: VARIATIONAL_FEATURE_GATE_V0,
            intent: PatternIntentV0::Atomic,
            log_probability_bits: bool_bits_v0(looks_atomic) * 5.0
                - bool_bits_v0(input.declaration_count > 1) * 2.0,
        },
        DesignerIntentScoreV0 {
            schema_version: VARIATIONAL_SCHEMA_VERSION_V0,
            product: "omena-variational.designer-intent-score",
            layer_marker: VARIATIONAL_LAYER_MARKER_V0,
            feature_gate: VARIATIONAL_FEATURE_GATE_V0,
            intent: PatternIntentV0::Hybrid,
            log_probability_bits: bool_bits_v0(has_bem_marker)
                + bool_bits_v0(looks_hybrid) * 4.0
                + bool_bits_v0(input.custom_property_reference_count > 0) * 1.5,
        },
        DesignerIntentScoreV0 {
            schema_version: VARIATIONAL_SCHEMA_VERSION_V0,
            product: "omena-variational.designer-intent-score",
            layer_marker: VARIATIONAL_LAYER_MARKER_V0,
            feature_gate: VARIATIONAL_FEATURE_GATE_V0,
            intent: PatternIntentV0::AdHoc,
            log_probability_bits: bool_bits_v0(!has_bem_marker && !looks_utility)
                + bool_bits_v0(input.duplicate_property_tie_count > 0),
        },
    ];
    scores.sort_by(|left, right| {
        right
            .log_probability_bits
            .partial_cmp(&left.log_probability_bits)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.intent.as_str().cmp(right.intent.as_str()))
    });
    DesignerIntentPosteriorV0 {
        schema_version: VARIATIONAL_SCHEMA_VERSION_V0,
        product: "omena-variational.designer-intent-posterior",
        layer_marker: VARIATIONAL_LAYER_MARKER_V0,
        feature_gate: VARIATIONAL_FEATURE_GATE_V0,
        mode: DesignerIntentPosteriorModeV0::VciFormal,
        selector_name: input.selector_name,
        scores,
        enabled_by_default: true,
    }
}

pub fn dominant_designer_intent_v0(
    posterior: &DesignerIntentPosteriorV0,
) -> Option<PatternIntentV0> {
    posterior.scores.first().map(|score| score.intent)
}

fn normalize_selector_name_for_intent_v0(selector_name: &str) -> String {
    selector_name
        .trim()
        .trim_start_matches('.')
        .split([':', '[', ' ', '>', '+', '~', ','])
        .next()
        .unwrap_or(selector_name)
        .trim()
        .to_string()
}

fn bool_bits_v0(value: bool) -> f64 {
    if value { 1.0 } else { 0.0 }
}

pub const REPLICA_ENSEMBLE_SCHEMA_VERSION_V0: &str = "0";
pub const REPLICA_ENSEMBLE_LAYER_MARKER_V0: &str = "replica-ensemble";
pub const REPLICA_ENSEMBLE_FEATURE_GATE_V0: &str = "replica-ensemble";
pub const REPLICA_ENSEMBLE_MECHANISM_SCOPE_V0: &str =
    "productWiredCrossFileConsistencyHintSubstrate";
pub const REPLICA_ENSEMBLE_PRODUCT_SURFACE_V0: &str = "defaultCrossFileConsistencyHint";
pub const REPLICA_ENSEMBLE_DEFAULT_PRODUCT_DECISION_MECHANISM_V0: bool = false;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeSiteKeyV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub element_selector: String,
    pub property: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinearProvenanceTagV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub semiring_identifier: &'static str,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplicaSiteOutcomeV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub site: CascadeSiteKeyV0,
    pub outcome: CascadeOutcome,
    pub provenance: Option<LinearProvenanceTagV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplicaSnapshotV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub path: String,
    pub sites: Vec<ReplicaSiteOutcomeV0>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OutcomeMode {
    #[default]
    DefiniteOnly,
    WidenedRankedSet,
    FullStrict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SamplingPolicy {
    AllPairs,
    PageRankWeighted { max_pair_count: usize },
    RandomSubset { max_pair_count: usize },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplicaOverlapV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub replica_alpha_path: String,
    pub replica_beta_path: String,
    pub outcome_mode: OutcomeMode,
    pub shared_site_count: usize,
    pub agreeing_site_count: usize,
    pub overlap_q: f64,
    pub overlap_q_unit: &'static str,
    pub provenance_attributions: Vec<OverlapAttributionV0>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlapAttributionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub site_element_selector: String,
    pub site_property: String,
    pub winner_alpha: String,
    pub winner_beta: String,
    pub provenance_alpha: Option<LinearProvenanceTagV0>,
    pub provenance_beta: Option<LinearProvenanceTagV0>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistogramBinV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub q_low: f64,
    pub q_high: f64,
    pub count: usize,
    pub normalized_density: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DistributionModality {
    Trivial,
    Unimodal,
    BimodalRSB,
    Continuous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ParisiSource {
    M4AlphaCascadeReplicaOverlap,
    LocalTwoComponentEm,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplicaOverlapDistributionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub workspace_root: String,
    pub outcome_mode: OutcomeMode,
    pub replica_count: usize,
    pub pair_count: usize,
    pub histogram_bin_count: usize,
    pub histogram_bins: Vec<HistogramBinV0>,
    pub modality: DistributionModality,
    pub modality_definition: &'static str,
    pub peak_q_values: Vec<f64>,
    pub parisi_m_estimate: Option<f64>,
    pub parisi_m_source: ParisiSource,
    pub mean_q: f64,
    pub variance_q: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleGraphV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub workspace_root: String,
    pub nodes: Vec<String>,
    pub edges: Vec<ModuleGraphEdgeV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleGraphEdgeV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub from_module: String,
    pub to_module: String,
    pub edge_kind: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PartitionHypothesisLabel {
    DirectoryTree,
    ComposesCluster,
    BrandTheme,
    AutoSpectral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SpectralMethod {
    Auto,
    DegreeCorrected,
    Spectral,
    NonBacktracking,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportOptionsV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub partition_hypotheses: Vec<PartitionHypothesisLabel>,
    pub spectral_method: SpectralMethod,
    pub sampling_policy: Option<SamplingPolicy>,
    pub rg_exponent_handle: Option<RgExponentHandleV0>,
}

impl Default for ReportOptionsV0 {
    fn default() -> Self {
        Self {
            schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
            product: "omena-ensemble.report-options",
            layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
            feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
            partition_hypotheses: vec![
                PartitionHypothesisLabel::AutoSpectral,
                PartitionHypothesisLabel::ComposesCluster,
                PartitionHypothesisLabel::DirectoryTree,
            ],
            spectral_method: SpectralMethod::Auto,
            sampling_policy: None,
            rg_exponent_handle: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RgExponentHandleV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub workspace_root: String,
    pub timestamp: String,
    pub digest: String,
}

#[derive(Debug, Clone, Copy)]
pub struct ParisiM4AlphaSource<'a> {
    pub replica_overlap: &'a CascadeReplicaOverlapV0,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossFileInconsistencyReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub mechanism_scope: &'static str,
    pub product_surface: &'static str,
    pub default_product_decision_mechanism: bool,
    pub workspace_root: String,
    pub distribution: ReplicaOverlapDistributionV0,
    pub top_disagreement_pairs: Vec<ReplicaOverlapV0>,
    pub recommendation: ReportRecommendation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ReportRecommendation {
    NoActionNeeded,
    InvestigateRsbBroken,
    UndetectablePhase,
}

pub fn site(element_selector: impl Into<String>, property: impl Into<String>) -> CascadeSiteKeyV0 {
    CascadeSiteKeyV0 {
        schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
        product: "omena-ensemble.cascade-site-key",
        layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
        feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
        element_selector: element_selector.into(),
        property: property.into(),
    }
}

pub fn build_cross_file_inconsistency_report(
    workspace_root: &str,
    replicas: impl IntoIterator<Item = ReplicaSnapshotV0>,
    _module_graph: &ModuleGraphV0,
    outcome_mode: OutcomeMode,
    options: ReportOptionsV0,
    parisi_source: Option<ParisiM4AlphaSource<'_>>,
) -> CrossFileInconsistencyReportV0 {
    let replicas = replicas.into_iter().collect::<Vec<_>>();
    let distribution = compute_overlap_distribution(
        workspace_root,
        replicas.clone(),
        options.sampling_policy,
        outcome_mode,
        parisi_source,
    );
    let top_disagreement_pairs =
        top_disagreement_pairs(&replicas, outcome_mode, options.sampling_policy);
    let recommendation = if top_disagreement_pairs
        .iter()
        .any(|pair| pair.shared_site_count > 0 && pair.overlap_q < 1.0)
    {
        ReportRecommendation::InvestigateRsbBroken
    } else {
        ReportRecommendation::NoActionNeeded
    };

    CrossFileInconsistencyReportV0 {
        schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
        product: "omena-ensemble.cross-file-inconsistency-report",
        layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
        feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
        mechanism_scope: REPLICA_ENSEMBLE_MECHANISM_SCOPE_V0,
        product_surface: REPLICA_ENSEMBLE_PRODUCT_SURFACE_V0,
        default_product_decision_mechanism: REPLICA_ENSEMBLE_DEFAULT_PRODUCT_DECISION_MECHANISM_V0,
        workspace_root: workspace_root.to_string(),
        distribution,
        top_disagreement_pairs,
        recommendation,
    }
}

fn compute_overlap_distribution(
    workspace_root: &str,
    replicas: impl IntoIterator<Item = ReplicaSnapshotV0>,
    sampling_policy: Option<SamplingPolicy>,
    outcome_mode: OutcomeMode,
    parisi_source: Option<ParisiM4AlphaSource<'_>>,
) -> ReplicaOverlapDistributionV0 {
    let replicas = replicas.into_iter().collect::<Vec<_>>();
    let pairs = selected_pair_indices(&replicas, sampling_policy);
    let overlaps = pairs
        .iter()
        .map(|(alpha_index, beta_index)| {
            let alpha = &replicas[*alpha_index];
            let beta = &replicas[*beta_index];
            compute_replica_overlap(
                &alpha.path,
                &beta.path,
                alpha.sites.clone(),
                beta.sites.clone(),
                outcome_mode,
            )
        })
        .collect::<Vec<_>>();
    let q_values = overlaps
        .iter()
        .map(|overlap| overlap.overlap_q)
        .collect::<Vec<_>>();
    let mean_q = mean(&q_values);
    let variance_q = variance(&q_values, mean_q);
    let histogram_bins = histogram(&q_values, 10);
    let modality = classify_modality(overlaps.len(), variance_q, &histogram_bins);
    let (parisi_m_estimate, parisi_m_source) = parisi_estimate(modality, parisi_source, &q_values);

    ReplicaOverlapDistributionV0 {
        schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
        product: "omena-ensemble.replica-overlap-distribution",
        layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
        feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
        workspace_root: workspace_root.to_string(),
        outcome_mode,
        replica_count: replicas.len(),
        pair_count: overlaps.len(),
        histogram_bin_count: histogram_bins.len(),
        histogram_bins,
        modality,
        modality_definition: modality_definition(modality, parisi_m_source),
        peak_q_values: peak_q_values(&q_values),
        parisi_m_estimate,
        parisi_m_source,
        mean_q,
        variance_q,
    }
}

fn compute_replica_overlap<I, J>(
    alpha: &str,
    beta: &str,
    cascade_alpha: I,
    cascade_beta: J,
    mode: OutcomeMode,
) -> ReplicaOverlapV0
where
    I: IntoIterator<Item = ReplicaSiteOutcomeV0>,
    J: IntoIterator<Item = ReplicaSiteOutcomeV0>,
{
    let alpha_by_site = cascade_alpha
        .into_iter()
        .map(|entry| (entry.site.clone(), entry))
        .collect::<BTreeMap<_, _>>();
    let beta_by_site = cascade_beta
        .into_iter()
        .map(|entry| (entry.site.clone(), entry))
        .collect::<BTreeMap<_, _>>();

    let mut shared_site_count = 0usize;
    let mut agreeing_site_count = 0usize;
    let mut provenance_attributions = Vec::new();
    for (site, alpha_entry) in &alpha_by_site {
        let Some(beta_entry) = beta_by_site.get(site) else {
            continue;
        };
        let Some(alpha_projection) = project_outcome(&alpha_entry.outcome, mode) else {
            continue;
        };
        let Some(beta_projection) = project_outcome(&beta_entry.outcome, mode) else {
            continue;
        };
        shared_site_count += 1;
        if alpha_projection == beta_projection {
            agreeing_site_count += 1;
        } else {
            provenance_attributions.push(OverlapAttributionV0 {
                schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
                product: "omena-ensemble.overlap-attribution",
                layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
                feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
                site_element_selector: site.element_selector.clone(),
                site_property: site.property.clone(),
                winner_alpha: alpha_projection,
                winner_beta: beta_projection,
                provenance_alpha: alpha_entry.provenance.clone(),
                provenance_beta: beta_entry.provenance.clone(),
            });
        }
    }
    let overlap_q = if shared_site_count == 0 {
        0.0
    } else {
        agreeing_site_count as f64 / shared_site_count as f64
    };

    ReplicaOverlapV0 {
        schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
        product: "omena-ensemble.replica-overlap",
        layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
        feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
        replica_alpha_path: alpha.to_string(),
        replica_beta_path: beta.to_string(),
        outcome_mode: mode,
        shared_site_count,
        agreeing_site_count,
        overlap_q,
        overlap_q_unit: "unitless",
        provenance_attributions,
    }
}

fn selected_pair_indices(
    replicas: &[ReplicaSnapshotV0],
    sampling_policy: Option<SamplingPolicy>,
) -> Vec<(usize, usize)> {
    let mut pairs = Vec::new();
    for alpha_index in 0..replicas.len() {
        for beta_index in alpha_index + 1..replicas.len() {
            pairs.push((alpha_index, beta_index));
        }
    }
    match sampling_policy {
        Some(SamplingPolicy::PageRankWeighted { max_pair_count })
        | Some(SamplingPolicy::RandomSubset { max_pair_count }) => pairs.truncate(max_pair_count),
        Some(SamplingPolicy::AllPairs) | None => {}
    }
    pairs
}

fn top_disagreement_pairs(
    replicas: &[ReplicaSnapshotV0],
    outcome_mode: OutcomeMode,
    sampling_policy: Option<SamplingPolicy>,
) -> Vec<ReplicaOverlapV0> {
    let mut overlaps = Vec::new();
    let mut remaining_budget = match sampling_policy {
        Some(SamplingPolicy::PageRankWeighted { max_pair_count })
        | Some(SamplingPolicy::RandomSubset { max_pair_count }) => max_pair_count,
        Some(SamplingPolicy::AllPairs) | None => usize::MAX,
    };
    for alpha_index in 0..replicas.len() {
        for beta_index in alpha_index + 1..replicas.len() {
            if remaining_budget == 0 {
                break;
            }
            remaining_budget = remaining_budget.saturating_sub(1);
            let alpha = &replicas[alpha_index];
            let beta = &replicas[beta_index];
            overlaps.push(compute_replica_overlap(
                &alpha.path,
                &beta.path,
                alpha.sites.clone(),
                beta.sites.clone(),
                outcome_mode,
            ));
        }
    }
    overlaps.sort_by(|left, right| {
        left.overlap_q
            .total_cmp(&right.overlap_q)
            .then_with(|| left.replica_alpha_path.cmp(&right.replica_alpha_path))
            .then_with(|| left.replica_beta_path.cmp(&right.replica_beta_path))
    });
    overlaps.truncate(5);
    overlaps
}

fn project_outcome(outcome: &CascadeOutcome, mode: OutcomeMode) -> Option<String> {
    match (outcome, mode) {
        (CascadeOutcome::Definite { winner, .. }, _) => Some(format!("definite:{}", winner.id)),
        (CascadeOutcome::RankedSet(declarations), OutcomeMode::WidenedRankedSet)
        | (CascadeOutcome::RankedSet(declarations), OutcomeMode::FullStrict) => {
            let mut ids = declarations
                .iter()
                .map(|declaration| declaration.id.as_str())
                .collect::<Vec<_>>();
            ids.sort_unstable();
            Some(format!("ranked:{}", ids.join("|")))
        }
        (CascadeOutcome::Inherit, OutcomeMode::FullStrict) => Some("inherit".to_string()),
        (CascadeOutcome::Top, OutcomeMode::FullStrict) => Some("top".to_string()),
        (CascadeOutcome::RankedSet(_), OutcomeMode::DefiniteOnly)
        | (CascadeOutcome::Inherit, OutcomeMode::DefiniteOnly | OutcomeMode::WidenedRankedSet)
        | (CascadeOutcome::Top, OutcomeMode::DefiniteOnly | OutcomeMode::WidenedRankedSet) => None,
    }
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

fn variance(values: &[f64], mean: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values
        .iter()
        .map(|value| {
            let delta = value - mean;
            delta * delta
        })
        .sum::<f64>()
        / values.len() as f64
}

fn histogram(values: &[f64], bin_count: usize) -> Vec<HistogramBinV0> {
    let mut counts = vec![0usize; bin_count];
    for value in values {
        let clamped = value.clamp(0.0, 1.0);
        let mut bin_index = (clamped * bin_count as f64).floor() as usize;
        if bin_index == bin_count {
            bin_index = bin_count.saturating_sub(1);
        }
        counts[bin_index] += 1;
    }
    counts
        .into_iter()
        .enumerate()
        .map(|(index, count)| HistogramBinV0 {
            schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
            product: "omena-ensemble.histogram-bin",
            layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
            feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
            q_low: index as f64 / bin_count as f64,
            q_high: (index + 1) as f64 / bin_count as f64,
            count,
            normalized_density: if values.is_empty() {
                0.0
            } else {
                count as f64 / values.len() as f64
            },
        })
        .collect()
}

fn classify_modality(
    pair_count: usize,
    variance_q: f64,
    histogram_bins: &[HistogramBinV0],
) -> DistributionModality {
    if pair_count < 3 {
        return DistributionModality::Trivial;
    }
    if variance_q < 0.01 {
        return DistributionModality::Unimodal;
    }
    let low_peak = histogram_bins
        .iter()
        .any(|bin| bin.count > 0 && bin.q_high <= 0.5);
    let high_peak = histogram_bins
        .iter()
        .any(|bin| bin.count > 0 && bin.q_low >= 0.7);
    if low_peak && high_peak {
        DistributionModality::BimodalRSB
    } else {
        DistributionModality::Continuous
    }
}

fn parisi_estimate(
    modality: DistributionModality,
    parisi_source: Option<ParisiM4AlphaSource<'_>>,
    q_values: &[f64],
) -> (Option<f64>, ParisiSource) {
    if let Some(source) = parisi_source
        && let Some(m_estimate) = source.replica_overlap.parisi_breakpoint_m
    {
        return (Some(m_estimate), ParisiSource::M4AlphaCascadeReplicaOverlap);
    }
    if modality == DistributionModality::BimodalRSB {
        return (
            two_component_em_low_overlap_weight(q_values),
            ParisiSource::LocalTwoComponentEm,
        );
    }
    (None, ParisiSource::Unavailable)
}

fn two_component_em_low_overlap_weight(q_values: &[f64]) -> Option<f64> {
    if q_values.len() < 3 {
        return None;
    }
    let mut sorted = q_values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let low = sorted[0].clamp(0.0, 1.0);
    let high = sorted[sorted.len() - 1].clamp(0.0, 1.0);
    (high - low > 0.000_001).then_some(0.5)
}

fn modality_definition(
    modality: DistributionModality,
    parisi_source: ParisiSource,
) -> &'static str {
    match (modality, parisi_source) {
        (DistributionModality::Trivial, _) => {
            "Fewer than 3 replica pairs available; modality undefined"
        }
        (DistributionModality::Unimodal, _) => {
            "Single peak in P(q) histogram; replica-symmetric descriptive shape"
        }
        (DistributionModality::BimodalRSB, ParisiSource::M4AlphaCascadeReplicaOverlap) => {
            "Two peaks in P(q) with M4-alpha spin-glass Parisi estimate attached"
        }
        (DistributionModality::BimodalRSB, ParisiSource::LocalTwoComponentEm) => {
            "Two peaks in P(q) histogram; local two-component EM estimates the low-overlap mixture weight"
        }
        (DistributionModality::BimodalRSB, _) => {
            "Two peaks in P(q) histogram; spin-glass source unavailable for Parisi estimate"
        }
        (DistributionModality::Continuous, _) => {
            "Smooth P(q) histogram; peak detection fails the bimodal threshold"
        }
    }
}

fn peak_q_values(values: &[f64]) -> Vec<f64> {
    if values.is_empty() {
        return Vec::new();
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let low = sorted[0];
    let high = sorted[sorted.len() - 1];
    if (high - low).abs() < f64::EPSILON {
        vec![low]
    } else {
        vec![low, high]
    }
}
