use serde::Serialize;

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
    CascadeSiteV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.cascade-site",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        site_id: site_id.into(),
        axes: vec![
            SiteAxisV0::Origin,
            SiteAxisV0::Importance,
            SiteAxisV0::Layer,
            SiteAxisV0::Specificity,
            SiteAxisV0::Scope,
            SiteAxisV0::SourceOrder,
            SiteAxisV0::Supports,
        ],
        cover_families: Vec::new(),
    }
}
