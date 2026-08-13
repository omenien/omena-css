use serde::Serialize;
use std::collections::BTreeSet;

use crate::{
    CATEGORICAL_FEATURE_GATE_V0, CATEGORICAL_LAYER_MARKER_V0, CATEGORICAL_SCHEMA_VERSION_V0,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeSectionAggregationPlanV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    /// Pre-1.0 wire-compatible field. Owner: `omena-categorical` maintainers;
    /// remove after downstream migration and zero audited non-compat uses.
    #[deprecated(
        since = "0.4.0",
        note = "use aggregation_id(); removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
    )]
    pub site_id: String,
    pub axes: Vec<CascadeSectionAxisV0>,
    pub cover_families: Vec<CoverFamilyV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverFamilyV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub cover_id: String,
    pub object_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeSectionAggregationCheckV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    #[deprecated(
        since = "0.4.0",
        note = "use aggregation_id(); removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
    )]
    pub site_id: String,
    pub identity_cover: bool,
    pub pullback_stable: bool,
    pub transitive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CascadeSectionAggregationTruthValueV0 {
    Open,
    Boundary,
    Closed,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CascadeSectionAxisV0 {
    Origin,
    Importance,
    Layer,
    Specificity,
    Scope,
    SourceOrder,
    Supports,
}

#[allow(deprecated)]
pub fn cascade_section_aggregation_plan_v0(
    aggregation_id: impl Into<String>,
) -> CascadeSectionAggregationPlanV0 {
    let aggregation_id = aggregation_id.into();
    let axes = cascade_section_axes_v0();
    let mut cover_families = axes
        .iter()
        .map(|axis| cascade_axis_identity_cover_v0(*axis))
        .collect::<Vec<_>>();
    cover_families.extend([
        cover_family_v0(
            "cascade-priority-cover",
            axes.iter().map(|axis| cascade_section_axis_id_v0(*axis)),
        ),
        cover_family_v0(
            "conditional-context-cover",
            [CascadeSectionAxisV0::Scope, CascadeSectionAxisV0::Supports]
                .into_iter()
                .map(cascade_section_axis_id_v0),
        ),
        cover_family_v0(
            "cascade-order-cover",
            [
                CascadeSectionAxisV0::Layer,
                CascadeSectionAxisV0::Specificity,
                CascadeSectionAxisV0::SourceOrder,
            ]
            .into_iter()
            .map(cascade_section_axis_id_v0),
        ),
    ]);

    build_cascade_section_aggregation_plan_with_legacy_field_v0(
        aggregation_id,
        axes,
        cover_families,
    )
}

#[deprecated(
    since = "0.4.0",
    note = "constructs a retained serialized field; owned by omena-categorical maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
fn build_cascade_section_aggregation_plan_with_legacy_field_v0(
    aggregation_id: String,
    axes: Vec<CascadeSectionAxisV0>,
    cover_families: Vec<CoverFamilyV0>,
) -> CascadeSectionAggregationPlanV0 {
    CascadeSectionAggregationPlanV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.cascade-section-aggregation-plan",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        site_id: aggregation_id,
        axes,
        cover_families,
    }
}

#[allow(deprecated)]
pub fn check_cascade_section_aggregation_v0(
    plan: &CascadeSectionAggregationPlanV0,
) -> CascadeSectionAggregationCheckV0 {
    let axis_object_ids = plan
        .axes
        .iter()
        .map(|axis| cascade_section_axis_id_v0(*axis).to_string())
        .collect::<BTreeSet<_>>();
    let singleton_cover_ids = plan
        .cover_families
        .iter()
        .filter(|cover| {
            cover.object_ids.len() == 1 && axis_object_ids.contains(&cover.object_ids[0])
        })
        .map(|cover| cover.object_ids[0].clone())
        .collect::<BTreeSet<_>>();
    let covered_object_ids = plan
        .cover_families
        .iter()
        .flat_map(|cover| cover.object_ids.iter().cloned())
        .collect::<BTreeSet<_>>();
    let every_cover_object_is_known = covered_object_ids
        .iter()
        .all(|object_id| axis_object_ids.contains(object_id));
    let every_cover_refines_to_singletons = plan.cover_families.iter().all(|cover| {
        !cover.object_ids.is_empty()
            && cover
                .object_ids
                .iter()
                .all(|object_id| singleton_cover_ids.contains(object_id))
    });
    let identity_cover =
        !axis_object_ids.is_empty() && axis_object_ids.is_subset(&singleton_cover_ids);
    let pullback_stable =
        identity_cover && every_cover_object_is_known && every_cover_refines_to_singletons;
    let transitive = pullback_stable && axis_object_ids.is_subset(&covered_object_ids);

    build_cascade_section_aggregation_check_with_legacy_field_v0(
        plan.aggregation_id().to_string(),
        identity_cover,
        pullback_stable,
        transitive,
    )
}

#[deprecated(
    since = "0.4.0",
    note = "constructs a retained serialized field; owned by omena-categorical maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
fn build_cascade_section_aggregation_check_with_legacy_field_v0(
    aggregation_id: String,
    identity_cover: bool,
    pullback_stable: bool,
    transitive: bool,
) -> CascadeSectionAggregationCheckV0 {
    CascadeSectionAggregationCheckV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.cascade-section-aggregation-check",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        site_id: aggregation_id,
        identity_cover,
        pullback_stable,
        transitive,
    }
}

pub fn cover_family_v0(
    cover_id: impl Into<String>,
    object_ids: impl IntoIterator<Item = impl Into<String>>,
) -> CoverFamilyV0 {
    CoverFamilyV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.cover-family",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        cover_id: cover_id.into(),
        object_ids: object_ids.into_iter().map(Into::into).collect(),
    }
}

pub fn cascade_section_axis_id_v0(axis: CascadeSectionAxisV0) -> &'static str {
    match axis {
        CascadeSectionAxisV0::Origin => "axis:origin",
        CascadeSectionAxisV0::Importance => "axis:importance",
        CascadeSectionAxisV0::Layer => "axis:layer",
        CascadeSectionAxisV0::Specificity => "axis:specificity",
        CascadeSectionAxisV0::Scope => "axis:scope",
        CascadeSectionAxisV0::SourceOrder => "axis:source-order",
        CascadeSectionAxisV0::Supports => "axis:supports",
    }
}

fn cascade_section_axes_v0() -> Vec<CascadeSectionAxisV0> {
    vec![
        CascadeSectionAxisV0::Origin,
        CascadeSectionAxisV0::Importance,
        CascadeSectionAxisV0::Layer,
        CascadeSectionAxisV0::Specificity,
        CascadeSectionAxisV0::Scope,
        CascadeSectionAxisV0::SourceOrder,
        CascadeSectionAxisV0::Supports,
    ]
}

impl CascadeSectionAggregationPlanV0 {
    #[allow(deprecated)]
    pub fn aggregation_id(&self) -> &str {
        cascade_section_aggregation_plan_id_from_legacy_field_v0(self)
    }
}

impl CascadeSectionAggregationCheckV0 {
    #[allow(deprecated)]
    pub fn aggregation_id(&self) -> &str {
        cascade_section_aggregation_check_id_from_legacy_field_v0(self)
    }
}

#[deprecated(
    since = "0.4.0",
    note = "compatibility field adapter owned by omena-categorical maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
fn cascade_section_aggregation_plan_id_from_legacy_field_v0(
    plan: &CascadeSectionAggregationPlanV0,
) -> &str {
    plan.site_id.as_str()
}

#[deprecated(
    since = "0.4.0",
    note = "compatibility field adapter owned by omena-categorical maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
fn cascade_section_aggregation_check_id_from_legacy_field_v0(
    check: &CascadeSectionAggregationCheckV0,
) -> &str {
    check.site_id.as_str()
}

fn cascade_axis_identity_cover_v0(axis: CascadeSectionAxisV0) -> CoverFamilyV0 {
    cover_family_v0(
        format!(
            "identity-{}",
            cascade_section_axis_id_v0(axis).replace(':', "-")
        ),
        [cascade_section_axis_id_v0(axis)],
    )
}
