use serde::Serialize;

use crate::{
    CascadeSectionAggregationCheckV0, CascadeSectionAggregationPlanV0, CascadeSectionAxisV0,
    cascade_section_aggregation_plan_v0, check_cascade_section_aggregation_v0,
};

pub use crate::{CoverFamilyV0, cover_family_v0};

#[deprecated(
    since = "0.4.0",
    note = "use CascadeSectionAggregationPlanV0; compatibility owner: omena-categorical maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeSiteV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub site_id: String,
    pub axes: Vec<SiteAxisV0>,
    pub cover_families: Vec<CoverFamilyV0>,
}

#[deprecated(
    since = "0.4.0",
    note = "use CascadeSectionAggregationCheckV0; compatibility owner: omena-categorical maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteAxiomCheckV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub site_id: String,
    pub identity_cover: bool,
    pub pullback_stable: bool,
    pub transitive: bool,
}

#[deprecated(
    since = "0.4.0",
    note = "use CascadeSectionAggregationTruthValueV0; compatibility owner: omena-categorical maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SiteTruthValueV0 {
    Open,
    Boundary,
    Closed,
    Full,
}

#[deprecated(
    since = "0.4.0",
    note = "use CascadeSectionAxisV0; compatibility owner: omena-categorical maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SiteAxisV0 {
    Origin,
    Importance,
    Layer,
    Specificity,
    Scope,
    SourceOrder,
    Supports,
}

#[deprecated(
    since = "0.4.0",
    note = "use cascade_section_aggregation_plan_v0; compatibility owner: omena-categorical maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
pub fn cascade_site_v0(site_id: impl Into<String>) -> CascadeSiteV0 {
    compatibility_plan_from_canonical_v0(cascade_section_aggregation_plan_v0(site_id))
}

#[deprecated(
    since = "0.4.0",
    note = "use check_cascade_section_aggregation_v0; compatibility owner: omena-categorical maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
pub fn check_cascade_site_axioms_v0(site: &CascadeSiteV0) -> SiteAxiomCheckV0 {
    let canonical = canonical_plan_from_compatibility_v0(site);
    compatibility_check_from_canonical_v0(check_cascade_section_aggregation_v0(&canonical))
}

#[deprecated(
    since = "0.4.0",
    note = "use cascade_section_axis_id_v0; compatibility owner: omena-categorical maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
pub fn site_axis_object_id_v0(axis: SiteAxisV0) -> &'static str {
    match axis {
        SiteAxisV0::Origin => "axis:origin",
        SiteAxisV0::Importance => "axis:importance",
        SiteAxisV0::Layer => "axis:layer",
        SiteAxisV0::Specificity => "axis:specificity",
        SiteAxisV0::Scope => "axis:scope",
        SiteAxisV0::SourceOrder => "axis:source-order",
        SiteAxisV0::Supports => "axis:supports",
    }
}

#[deprecated(
    since = "0.4.0",
    note = "compatibility owner: omena-categorical maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
fn compatibility_plan_from_canonical_v0(plan: CascadeSectionAggregationPlanV0) -> CascadeSiteV0 {
    CascadeSiteV0 {
        schema_version: plan.schema_version,
        product: "omena-categorical.cascade-site",
        layer_marker: plan.layer_marker,
        feature_gate: plan.feature_gate,
        site_id: plan.site_id,
        axes: plan
            .axes
            .into_iter()
            .map(compatibility_axis_from_canonical_v0)
            .collect(),
        cover_families: plan.cover_families,
    }
}

#[deprecated(
    since = "0.4.0",
    note = "compatibility owner: omena-categorical maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
fn canonical_plan_from_compatibility_v0(site: &CascadeSiteV0) -> CascadeSectionAggregationPlanV0 {
    CascadeSectionAggregationPlanV0 {
        schema_version: site.schema_version,
        product: "omena-categorical.cascade-section-aggregation-plan",
        layer_marker: site.layer_marker,
        feature_gate: site.feature_gate,
        site_id: site.site_id.clone(),
        axes: site
            .axes
            .iter()
            .copied()
            .map(canonical_axis_from_compatibility_v0)
            .collect(),
        cover_families: site.cover_families.clone(),
    }
}

#[deprecated(
    since = "0.4.0",
    note = "compatibility owner: omena-categorical maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
fn compatibility_check_from_canonical_v0(
    check: CascadeSectionAggregationCheckV0,
) -> SiteAxiomCheckV0 {
    SiteAxiomCheckV0 {
        schema_version: check.schema_version,
        product: "omena-categorical.site-axiom-check",
        layer_marker: check.layer_marker,
        feature_gate: check.feature_gate,
        site_id: check.site_id,
        identity_cover: check.identity_cover,
        pullback_stable: check.pullback_stable,
        transitive: check.transitive,
    }
}

#[deprecated(
    since = "0.4.0",
    note = "compatibility owner: omena-categorical maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
fn compatibility_axis_from_canonical_v0(axis: CascadeSectionAxisV0) -> SiteAxisV0 {
    match axis {
        CascadeSectionAxisV0::Origin => SiteAxisV0::Origin,
        CascadeSectionAxisV0::Importance => SiteAxisV0::Importance,
        CascadeSectionAxisV0::Layer => SiteAxisV0::Layer,
        CascadeSectionAxisV0::Specificity => SiteAxisV0::Specificity,
        CascadeSectionAxisV0::Scope => SiteAxisV0::Scope,
        CascadeSectionAxisV0::SourceOrder => SiteAxisV0::SourceOrder,
        CascadeSectionAxisV0::Supports => SiteAxisV0::Supports,
    }
}

#[deprecated(
    since = "0.4.0",
    note = "compatibility owner: omena-categorical maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
fn canonical_axis_from_compatibility_v0(axis: SiteAxisV0) -> CascadeSectionAxisV0 {
    match axis {
        SiteAxisV0::Origin => CascadeSectionAxisV0::Origin,
        SiteAxisV0::Importance => CascadeSectionAxisV0::Importance,
        SiteAxisV0::Layer => CascadeSectionAxisV0::Layer,
        SiteAxisV0::Specificity => CascadeSectionAxisV0::Specificity,
        SiteAxisV0::Scope => CascadeSectionAxisV0::Scope,
        SiteAxisV0::SourceOrder => CascadeSectionAxisV0::SourceOrder,
        SiteAxisV0::Supports => CascadeSectionAxisV0::Supports,
    }
}
