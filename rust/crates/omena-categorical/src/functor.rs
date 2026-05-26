use serde::Serialize;

use crate::{
    CATEGORICAL_FEATURE_GATE_V0, CATEGORICAL_LAYER_MARKER_V0, CATEGORICAL_SCHEMA_VERSION_V0,
    CascadePrimitiveRoleV0,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeCategoryObjectV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub object_id: String,
    pub object_kind: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeCategoryMorphismV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub morphism_id: String,
    pub from_object_id: String,
    pub to_object_id: String,
    pub relation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeCategoryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub category_id: String,
    pub objects: Vec<CascadeCategoryObjectV0>,
    pub morphisms: Vec<CascadeCategoryMorphismV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeFunctorObjectMappingV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub source_object_id: String,
    pub target_object_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeFunctorMorphismMappingV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub source_morphism_id: String,
    pub target_morphism_id: String,
    pub source_from_object_id: String,
    pub source_to_object_id: String,
    pub target_from_object_id: String,
    pub target_to_object_id: String,
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

pub fn cascade_primitive_category_v0(roles: &[CascadePrimitiveRoleV0]) -> CascadeCategoryV0 {
    let objects = roles
        .iter()
        .map(|role| category_object_v0(format!("primitive:{}", role.primitive_name), "primitive"))
        .collect::<Vec<_>>();
    let morphisms = category_morphisms_from_objects_v0(&objects, "primitive-precedes");

    CascadeCategoryV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.cascade-category",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        category_id: "cascade-primitives".to_string(),
        objects,
        morphisms,
    }
}

pub fn categorical_role_category_v0(roles: &[CascadePrimitiveRoleV0]) -> CascadeCategoryV0 {
    let objects = roles
        .iter()
        .map(|role| category_object_v0(format!("role:{}", slug_v0(role.categorical_role)), "role"))
        .collect::<Vec<_>>();
    let morphisms = category_morphisms_from_objects_v0(&objects, "role-precedes");

    CascadeCategoryV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.cascade-category",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        category_id: "categorical-roles".to_string(),
        objects,
        morphisms,
    }
}

pub fn apply_cascade_primitive_role_functor_v0(
    roles: &[CascadePrimitiveRoleV0],
) -> CascadeFunctorApplicationV0 {
    let source = cascade_primitive_category_v0(roles);
    let target = categorical_role_category_v0(roles);
    let object_mappings = roles
        .iter()
        .map(|role| CascadeFunctorObjectMappingV0 {
            schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
            product: "omena-categorical.functor-object-mapping",
            layer_marker: CATEGORICAL_LAYER_MARKER_V0,
            feature_gate: CATEGORICAL_FEATURE_GATE_V0,
            source_object_id: format!("primitive:{}", role.primitive_name),
            target_object_id: format!("role:{}", slug_v0(role.categorical_role)),
        })
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
        product: "omena-categorical.cascade-primitive-role-functor",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        functor_id: "cascade-primitive-role-functor".to_string(),
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
