//! Cascade-origin inputs and their mapping onto the existing priority ladder.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::OnceLock;

use crate::CascadeLevel;

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize,
)]
#[serde(rename_all = "camelCase")]
pub enum CascadeOriginV0 {
    UserAgent,
    User,
    #[default]
    Author,
    Inline,
}

impl CascadeOriginV0 {
    pub const fn is_author(&self) -> bool {
        matches!(self, Self::Author)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeOriginDriverV0 {
    pub origin: CascadeOriginV0,
    pub important: bool,
    pub level: CascadeLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CascadeWinnerAxisV0 {
    CascadeLevel,
    LayerRank,
    ScopeProximity,
    Specificity,
    SourceOrder,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CascadeDriverCensusV0 {
    schema_version: String,
    product: String,
    levels: Vec<CascadeDriverLevelV0>,
    winner_axes: Vec<CascadeDriverAxisV0>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CascadeDriverLevelV0 {
    level: String,
    status: String,
    driver_inputs: Vec<String>,
    #[serde(default)]
    follow_up: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CascadeDriverAxisV0 {
    axis: CascadeWinnerAxisV0,
    status: String,
}

const CASCADE_DRIVER_CENSUS_JSON: &str = include_str!("../data/cascade-driver-census.json");
static CASCADE_DRIVER_CENSUS: OnceLock<Result<CascadeDriverCensusV0, String>> = OnceLock::new();

fn cascade_driver_census_v0() -> Option<&'static CascadeDriverCensusV0> {
    CASCADE_DRIVER_CENSUS
        .get_or_init(|| {
            serde_json::from_str(CASCADE_DRIVER_CENSUS_JSON).map_err(|error| error.to_string())
        })
        .as_ref()
        .ok()
}

pub fn cascade_driven_levels_v0() -> Vec<CascadeLevel> {
    cascade_driver_census_v0()
        .into_iter()
        .flat_map(|census| census.levels.iter())
        .filter(|entry| entry.status == "driven")
        .filter_map(|entry| cascade_level_from_name_v0(entry.level.as_str()))
        .collect()
}

pub fn cascade_driven_winner_axes_v0() -> Vec<CascadeWinnerAxisV0> {
    cascade_driver_census_v0()
        .into_iter()
        .flat_map(|census| census.winner_axes.iter())
        .filter(|entry| entry.status == "driven")
        .map(|entry| entry.axis)
        .collect()
}

pub fn cascade_driver_census_is_consistent_v0() -> bool {
    let Some(census) = cascade_driver_census_v0() else {
        return false;
    };
    let catalog = cascade_level_catalog_v0();
    let levels = census
        .levels
        .iter()
        .filter_map(|entry| cascade_level_from_name_v0(entry.level.as_str()))
        .collect::<Vec<_>>();
    let driven = census
        .levels
        .iter()
        .filter(|entry| entry.status == "driven")
        .filter_map(|entry| cascade_level_from_name_v0(entry.level.as_str()))
        .collect::<Vec<_>>();
    let expected_driven = cascade_origin_driver_catalog_v0()
        .into_iter()
        .map(|driver| driver.level)
        .collect::<BTreeSet<_>>();
    let all_levels_have_evidence = census.levels.iter().all(|entry| {
        (entry.status == "driven" && !entry.driver_inputs.is_empty() && entry.follow_up.is_none())
            || (entry.status == "deferred"
                && entry.driver_inputs.is_empty()
                && entry
                    .follow_up
                    .as_deref()
                    .is_some_and(|value| !value.is_empty()))
    });
    let expected_axes = [
        CascadeWinnerAxisV0::CascadeLevel,
        CascadeWinnerAxisV0::LayerRank,
        CascadeWinnerAxisV0::ScopeProximity,
        CascadeWinnerAxisV0::Specificity,
        CascadeWinnerAxisV0::SourceOrder,
    ];
    census.schema_version == "0"
        && census.product == "omena-cascade.driver-census"
        && levels == catalog
        && driven.into_iter().collect::<BTreeSet<_>>() == expected_driven
        && all_levels_have_evidence
        && census.winner_axes.len() == expected_axes.len()
        && census
            .winner_axes
            .iter()
            .all(|entry| entry.status == "driven")
        && census
            .winner_axes
            .iter()
            .map(|entry| entry.axis)
            .eq(expected_axes)
}

fn cascade_level_from_name_v0(name: &str) -> Option<CascadeLevel> {
    cascade_level_catalog_v0()
        .into_iter()
        .find(|level| cascade_level_name_v0(*level) == name)
}

pub const fn cascade_level_for_origin(origin: CascadeOriginV0, important: bool) -> CascadeLevel {
    match (origin, important) {
        (CascadeOriginV0::UserAgent, false) => CascadeLevel::UserAgentNormal,
        (CascadeOriginV0::User, false) => CascadeLevel::UserNormal,
        (CascadeOriginV0::Author, false) => CascadeLevel::AuthorNormal,
        (CascadeOriginV0::Inline, false) => CascadeLevel::InlineNormal,
        (CascadeOriginV0::UserAgent, true) => CascadeLevel::UserAgentImportant,
        (CascadeOriginV0::User, true) => CascadeLevel::UserImportant,
        (CascadeOriginV0::Author | CascadeOriginV0::Inline, true) => CascadeLevel::AuthorImportant,
    }
}

pub const fn cascade_level_catalog_v0() -> [CascadeLevel; 9] {
    [
        CascadeLevel::UserAgentNormal,
        CascadeLevel::UserNormal,
        CascadeLevel::AuthorNormal,
        CascadeLevel::InlineNormal,
        CascadeLevel::Animation,
        CascadeLevel::AuthorImportant,
        CascadeLevel::UserImportant,
        CascadeLevel::UserAgentImportant,
        CascadeLevel::Transition,
    ]
}

pub const fn cascade_level_name_v0(level: CascadeLevel) -> &'static str {
    match level {
        CascadeLevel::UserAgentNormal => "userAgentNormal",
        CascadeLevel::UserNormal => "userNormal",
        CascadeLevel::AuthorNormal => "authorNormal",
        CascadeLevel::InlineNormal => "inlineNormal",
        CascadeLevel::Animation => "animation",
        CascadeLevel::AuthorImportant => "authorImportant",
        CascadeLevel::UserImportant => "userImportant",
        CascadeLevel::UserAgentImportant => "userAgentImportant",
        CascadeLevel::Transition => "transition",
    }
}

pub const fn cascade_origin_driver_catalog_v0() -> [CascadeOriginDriverV0; 8] {
    [
        origin_driver(CascadeOriginV0::UserAgent, false),
        origin_driver(CascadeOriginV0::User, false),
        origin_driver(CascadeOriginV0::Author, false),
        origin_driver(CascadeOriginV0::Inline, false),
        origin_driver(CascadeOriginV0::Author, true),
        origin_driver(CascadeOriginV0::Inline, true),
        origin_driver(CascadeOriginV0::User, true),
        origin_driver(CascadeOriginV0::UserAgent, true),
    ]
}

const fn origin_driver(origin: CascadeOriginV0, important: bool) -> CascadeOriginDriverV0 {
    CascadeOriginDriverV0 {
        origin,
        important,
        level: cascade_level_for_origin(origin, important),
    }
}
