//! Cascade-origin inputs and their mapping onto the existing priority ladder.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::OnceLock;

use crate::{
    CascadeLevel,
    axis_order::{CascadeKeyAxisV0, cascade_key_axis_order_v0},
};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// A named legacy source that can supply evidence for a cascade axis.
pub enum CascadeAxisNamedDriverV0 {
    /// Selector-context rank used by the design-token plane; this is not CSS `@scope`.
    LegacySelectorContextFallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// Why an axis is outside the currently modeled cascade fragment.
pub enum CascadeAxisOutOfFragmentReasonV0 {
    ShadowTreeEncapsulationContextUnmodeled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
#[non_exhaustive]
/// Reach status for one specification cascade axis.
pub enum CascadeAxisReachStatusV0 {
    Modeled,
    NotReachedByProduct {
        #[serde(rename = "namedDriver")]
        named_driver: CascadeAxisNamedDriverV0,
    },
    OutOfFragment {
        reason: CascadeAxisOutOfFragmentReasonV0,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// Reach disclosure derived from level/axis authorities, the producer census,
/// and the declared encapsulation fragment boundary.
pub struct CascadeAxisReachDisclosureV0 {
    pub origin_and_importance: CascadeAxisReachStatusV0,
    pub encapsulation_context: CascadeAxisReachStatusV0,
    pub style_attribute: CascadeAxisReachStatusV0,
    pub layers: CascadeAxisReachStatusV0,
    pub specificity: CascadeAxisReachStatusV0,
    pub scope_proximity: CascadeAxisReachStatusV0,
    pub order_of_appearance: CascadeAxisReachStatusV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CascadeDriverCensusV0 {
    schema_version: String,
    product: String,
    levels: Vec<CascadeDriverLevelV0>,
    winner_axes: Vec<CascadeDriverAxisV0>,
    cascade_key_producers: Vec<CascadeKeyProducerV0>,
    spec_axis_reach: CascadeAxisReachDisclosureV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CascadeDriverLevelV0 {
    level: String,
    status: String,
    driver_inputs: Vec<String>,
    #[serde(default)]
    follow_up: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CascadeDriverAxisV0 {
    axis: CascadeWinnerAxisV0,
    status: CascadeDriverAxisStatusV0,
    #[serde(default)]
    named_driver: Option<CascadeAxisNamedDriverV0>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
enum CascadeDriverAxisStatusV0 {
    Driven,
    AutomaticProductDriver,
}

impl CascadeDriverAxisStatusV0 {
    const fn is_driven(self) -> bool {
        matches!(self, Self::Driven | Self::AutomaticProductDriver)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CascadeKeyProducerV0 {
    path: String,
    symbol: String,
    occurrence: u32,
    disposition: CascadeKeyProducerDispositionV0,
    scope_proximity_source: CascadeScopeProximitySourceV0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
enum CascadeKeyProducerDispositionV0 {
    AutomaticProductDerived,
    CallerSuppliedBoundary,
    Conformance,
    Generated,
    Fixture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
enum CascadeScopeProximitySourceV0 {
    ConstantZero,
    LegacySelectorContextFallback,
    CallerSupplied,
    GeneratedValue,
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
        .filter(|entry| entry.status.is_driven())
        .map(|entry| entry.axis)
        .collect()
}

pub fn cascade_driver_census_is_consistent_v0() -> bool {
    let Some(census) = cascade_driver_census_v0() else {
        return false;
    };
    cascade_driver_census_payload_is_consistent_v0(census)
}

/// Summarizes modeled cascade-axis reach, failing closed if the embedded
/// authority mirrors or producer census are inconsistent.
pub fn summarize_cascade_axis_reach_v0() -> Option<CascadeAxisReachDisclosureV0> {
    summarize_cascade_axis_reach_from_census_v0(cascade_driver_census_v0()?)
}

fn summarize_cascade_axis_reach_from_census_v0(
    census: &CascadeDriverCensusV0,
) -> Option<CascadeAxisReachDisclosureV0> {
    if !cascade_driver_census_payload_is_consistent_v0(census) {
        return None;
    }
    derive_cascade_axis_reach_v0(census)
}

fn cascade_driver_census_payload_is_consistent_v0(census: &CascadeDriverCensusV0) -> bool {
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
    let inline_important_driver_count = census
        .levels
        .iter()
        .flat_map(|entry| entry.driver_inputs.iter())
        .filter(|input| input.as_str() == "inlineStyleImportant")
        .count();
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
    let expected_axes = cascade_winner_axis_catalog_from_authority_v0();
    let producer_ids = census
        .cascade_key_producers
        .iter()
        .map(|producer| {
            (
                producer.path.as_str(),
                producer.symbol.as_str(),
                producer.occurrence,
            )
        })
        .collect::<BTreeSet<_>>();
    let all_producers_have_consistent_evidence = census
        .cascade_key_producers
        .iter()
        .all(cascade_key_producer_has_consistent_evidence_v0);
    let all_axes_have_consistent_evidence =
        census.winner_axes.iter().all(|entry| match entry.axis {
            CascadeWinnerAxisV0::ScopeProximity => {
                entry.status == CascadeDriverAxisStatusV0::AutomaticProductDriver
                    && entry.named_driver
                        == Some(CascadeAxisNamedDriverV0::LegacySelectorContextFallback)
            }
            _ => entry.status == CascadeDriverAxisStatusV0::Driven && entry.named_driver.is_none(),
        });
    let derived_reach = derive_cascade_axis_reach_v0(census);
    census.schema_version == "0"
        && census.product == "omena-cascade.driver-census"
        && levels == catalog
        && driven.into_iter().collect::<BTreeSet<_>>() == expected_driven
        && inline_important_driver_count == 1
        && all_levels_have_evidence
        && census.winner_axes.len() == expected_axes.len()
        && all_axes_have_consistent_evidence
        && census
            .winner_axes
            .iter()
            .map(|entry| entry.axis)
            .eq(expected_axes)
        && producer_ids.len() == census.cascade_key_producers.len()
        && all_producers_have_consistent_evidence
        && derived_reach.as_ref() == Some(&census.spec_axis_reach)
}

fn cascade_key_producer_has_consistent_evidence_v0(producer: &CascadeKeyProducerV0) -> bool {
    if producer.path.is_empty() || producer.symbol.is_empty() || producer.occurrence == 0 {
        return false;
    }
    matches!(
        (producer.disposition, producer.scope_proximity_source),
        (
            CascadeKeyProducerDispositionV0::AutomaticProductDerived,
            CascadeScopeProximitySourceV0::ConstantZero
                | CascadeScopeProximitySourceV0::LegacySelectorContextFallback,
        ) | (
            CascadeKeyProducerDispositionV0::CallerSuppliedBoundary
                | CascadeKeyProducerDispositionV0::Conformance,
            CascadeScopeProximitySourceV0::CallerSupplied,
        ) | (
            CascadeKeyProducerDispositionV0::Generated,
            CascadeScopeProximitySourceV0::GeneratedValue,
        ) | (
            CascadeKeyProducerDispositionV0::Fixture,
            CascadeScopeProximitySourceV0::ConstantZero,
        )
    )
}

fn derive_cascade_axis_reach_v0(
    census: &CascadeDriverCensusV0,
) -> Option<CascadeAxisReachDisclosureV0> {
    let driven_levels = census
        .levels
        .iter()
        .filter(|entry| entry.status == "driven")
        .filter_map(|entry| cascade_level_from_name_v0(entry.level.as_str()))
        .collect::<BTreeSet<_>>();
    let expected_driven_levels = cascade_origin_driver_catalog_v0()
        .into_iter()
        .map(|driver| driver.level)
        .collect::<BTreeSet<_>>();
    if driven_levels != expected_driven_levels
        || !driven_levels.contains(&CascadeLevel::InlineNormal)
        || !driven_levels.contains(&CascadeLevel::InlineImportant)
    {
        return None;
    }

    let axis_is_driven = |axis| {
        census
            .winner_axes
            .iter()
            .any(|entry| entry.axis == axis && entry.status.is_driven())
    };
    if !axis_is_driven(CascadeWinnerAxisV0::CascadeLevel)
        || !axis_is_driven(CascadeWinnerAxisV0::LayerRank)
        || !axis_is_driven(CascadeWinnerAxisV0::Specificity)
        || !axis_is_driven(CascadeWinnerAxisV0::ScopeProximity)
        || !axis_is_driven(CascadeWinnerAxisV0::SourceOrder)
    {
        return None;
    }

    let automatic_sources = census
        .cascade_key_producers
        .iter()
        .filter(|producer| {
            producer.disposition == CascadeKeyProducerDispositionV0::AutomaticProductDerived
        })
        .map(|producer| producer.scope_proximity_source)
        .collect::<Vec<_>>();
    if automatic_sources.is_empty()
        || automatic_sources.iter().any(|source| {
            !matches!(
                source,
                CascadeScopeProximitySourceV0::ConstantZero
                    | CascadeScopeProximitySourceV0::LegacySelectorContextFallback
            )
        })
        || !automatic_sources
            .contains(&CascadeScopeProximitySourceV0::LegacySelectorContextFallback)
    {
        return None;
    }

    // Modeled limbs above are pinned by the level and key-axis authorities.
    // Scope reach is derived from the syntactic producer census. Encapsulation
    // remains an authored fragment boundary rather than a producer-scan claim.
    Some(CascadeAxisReachDisclosureV0 {
        origin_and_importance: CascadeAxisReachStatusV0::Modeled,
        encapsulation_context: CascadeAxisReachStatusV0::OutOfFragment {
            reason: CascadeAxisOutOfFragmentReasonV0::ShadowTreeEncapsulationContextUnmodeled,
        },
        style_attribute: CascadeAxisReachStatusV0::Modeled,
        layers: CascadeAxisReachStatusV0::Modeled,
        specificity: CascadeAxisReachStatusV0::Modeled,
        scope_proximity: CascadeAxisReachStatusV0::NotReachedByProduct {
            named_driver: CascadeAxisNamedDriverV0::LegacySelectorContextFallback,
        },
        order_of_appearance: CascadeAxisReachStatusV0::Modeled,
    })
}

fn cascade_winner_axis_catalog_from_authority_v0() -> Vec<CascadeWinnerAxisV0> {
    let mut axes = Vec::new();
    for axis in cascade_key_axis_order_v0() {
        let winner_axis = match axis {
            CascadeKeyAxisV0::Level => CascadeWinnerAxisV0::CascadeLevel,
            CascadeKeyAxisV0::LayerRank => CascadeWinnerAxisV0::LayerRank,
            CascadeKeyAxisV0::ScopeProximity => CascadeWinnerAxisV0::ScopeProximity,
            CascadeKeyAxisV0::SpecificityIds
            | CascadeKeyAxisV0::SpecificityClasses
            | CascadeKeyAxisV0::SpecificityElements => CascadeWinnerAxisV0::Specificity,
            CascadeKeyAxisV0::SourceOrder => CascadeWinnerAxisV0::SourceOrder,
        };
        if axes.last() != Some(&winner_axis) {
            axes.push(winner_axis);
        }
    }
    axes
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
        (CascadeOriginV0::Author, true) => CascadeLevel::AuthorImportant,
        (CascadeOriginV0::Inline, true) => CascadeLevel::InlineImportant,
    }
}

pub const fn cascade_level_catalog_v0() -> [CascadeLevel; 10] {
    [
        CascadeLevel::UserAgentNormal,
        CascadeLevel::UserNormal,
        CascadeLevel::AuthorNormal,
        CascadeLevel::InlineNormal,
        CascadeLevel::Animation,
        CascadeLevel::AuthorImportant,
        CascadeLevel::InlineImportant,
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
        CascadeLevel::InlineImportant => "inlineImportant",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_driver_census_derives_the_typed_axis_reach_disclosure() {
        assert!(cascade_driver_census_is_consistent_v0());
        assert_eq!(
            summarize_cascade_axis_reach_v0(),
            Some(CascadeAxisReachDisclosureV0 {
                origin_and_importance: CascadeAxisReachStatusV0::Modeled,
                encapsulation_context: CascadeAxisReachStatusV0::OutOfFragment {
                    reason:
                        CascadeAxisOutOfFragmentReasonV0::ShadowTreeEncapsulationContextUnmodeled,
                },
                style_attribute: CascadeAxisReachStatusV0::Modeled,
                layers: CascadeAxisReachStatusV0::Modeled,
                specificity: CascadeAxisReachStatusV0::Modeled,
                scope_proximity: CascadeAxisReachStatusV0::NotReachedByProduct {
                    named_driver: CascadeAxisNamedDriverV0::LegacySelectorContextFallback,
                },
                order_of_appearance: CascadeAxisReachStatusV0::Modeled,
            })
        );
    }

    #[test]
    fn caller_supplied_proximity_surfaces_are_excluded_from_the_automatic_product_driver()
    -> Result<(), &'static str> {
        let census = cascade_driver_census_v0().ok_or("embedded census must parse")?;
        let caller_supplied = census
            .cascade_key_producers
            .iter()
            .filter(|producer| {
                producer.disposition == CascadeKeyProducerDispositionV0::CallerSuppliedBoundary
            })
            .collect::<Vec<_>>();
        assert_eq!(caller_supplied.len(), 3);
        assert!(caller_supplied.iter().all(|producer| {
            producer.scope_proximity_source == CascadeScopeProximitySourceV0::CallerSupplied
        }));
        assert!(caller_supplied.iter().any(|producer| {
            producer.path == "rust/crates/omena-bundler/src/lib.rs"
                && producer.symbol == "LinkedStylesheetRuleV0::cascade_key_with_global_source_order"
        }));
        assert!(caller_supplied.iter().any(|producer| {
            producer.path == "rust/crates/omena-cascade-proof/src/proof_kernel.rs"
                && producer.symbol == "cascade_key_from_certificate_v0"
        }));
        assert!(caller_supplied.iter().any(|producer| {
            producer.path == "rust/crates/omena-transform-passes/src/runtime/winner_equality.rs"
                && producer.symbol == "winner_for_pair"
        }));

        let automatic = census
            .cascade_key_producers
            .iter()
            .filter(|producer| {
                producer.disposition == CascadeKeyProducerDispositionV0::AutomaticProductDerived
            })
            .collect::<Vec<_>>();
        assert!(automatic.iter().all(|producer| {
            matches!(
                producer.scope_proximity_source,
                CascadeScopeProximitySourceV0::ConstantZero
                    | CascadeScopeProximitySourceV0::LegacySelectorContextFallback
            )
        }));
        assert!(automatic.iter().any(|producer| {
            producer.path == "rust/crates/omena-semantic/src/design_tokens.rs"
                && producer.scope_proximity_source
                    == CascadeScopeProximitySourceV0::LegacySelectorContextFallback
        }));
        Ok(())
    }

    #[test]
    fn inconsistent_axis_reach_census_fails_closed() -> Result<(), &'static str> {
        let mut census = cascade_driver_census_v0()
            .ok_or("embedded census must parse")?
            .clone();
        census.spec_axis_reach.scope_proximity = CascadeAxisReachStatusV0::Modeled;

        assert_eq!(summarize_cascade_axis_reach_from_census_v0(&census), None);
        Ok(())
    }
}
