use serde::Serialize;
use std::collections::BTreeSet;

use crate::{
    CATEGORICAL_FEATURE_GATE_V0, CATEGORICAL_LAYER_MARKER_V0, CATEGORICAL_SCHEMA_VERSION_V0,
};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SiteTruthValueV0 {
    Open,
    Boundary,
    Closed,
    Full,
}

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

pub fn cascade_site_v0(site_id: impl Into<String>) -> CascadeSiteV0 {
    let site_id = site_id.into();
    let axes = cascade_site_axes_v0();
    let mut cover_families = axes
        .iter()
        .map(|axis| cascade_axis_identity_cover_v0(*axis))
        .collect::<Vec<_>>();
    cover_families.extend([
        cover_family_v0(
            "cascade-priority-cover",
            axes.iter().map(|axis| site_axis_object_id_v0(*axis)),
        ),
        cover_family_v0(
            "conditional-context-cover",
            [SiteAxisV0::Scope, SiteAxisV0::Supports]
                .into_iter()
                .map(site_axis_object_id_v0),
        ),
        cover_family_v0(
            "cascade-order-cover",
            [
                SiteAxisV0::Layer,
                SiteAxisV0::Specificity,
                SiteAxisV0::SourceOrder,
            ]
            .into_iter()
            .map(site_axis_object_id_v0),
        ),
    ]);

    CascadeSiteV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.cascade-site",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        site_id,
        axes,
        cover_families,
    }
}

pub fn check_cascade_site_axioms_v0(site: &CascadeSiteV0) -> SiteAxiomCheckV0 {
    let axis_object_ids = site
        .axes
        .iter()
        .map(|axis| site_axis_object_id_v0(*axis).to_string())
        .collect::<BTreeSet<_>>();
    let singleton_cover_ids = site
        .cover_families
        .iter()
        .filter_map(|cover| {
            (cover.object_ids.len() == 1 && axis_object_ids.contains(&cover.object_ids[0]))
                .then(|| cover.object_ids[0].clone())
        })
        .collect::<BTreeSet<_>>();
    let covered_object_ids = site
        .cover_families
        .iter()
        .flat_map(|cover| cover.object_ids.iter().cloned())
        .collect::<BTreeSet<_>>();
    let every_cover_object_is_known = covered_object_ids
        .iter()
        .all(|object_id| axis_object_ids.contains(object_id));
    let every_cover_refines_to_singletons = site.cover_families.iter().all(|cover| {
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

    SiteAxiomCheckV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.site-axiom-check",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        site_id: site.site_id.clone(),
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

fn cascade_site_axes_v0() -> Vec<SiteAxisV0> {
    vec![
        SiteAxisV0::Origin,
        SiteAxisV0::Importance,
        SiteAxisV0::Layer,
        SiteAxisV0::Specificity,
        SiteAxisV0::Scope,
        SiteAxisV0::SourceOrder,
        SiteAxisV0::Supports,
    ]
}

fn cascade_axis_identity_cover_v0(axis: SiteAxisV0) -> CoverFamilyV0 {
    cover_family_v0(
        format!(
            "identity-{}",
            site_axis_object_id_v0(axis).replace(':', "-")
        ),
        [site_axis_object_id_v0(axis)],
    )
}
