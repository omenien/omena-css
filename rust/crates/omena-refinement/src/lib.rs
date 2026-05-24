//! Refinement type system contracts for cascade analysis.
//!
//! The crate keeps legacy abstract property values wire-compatible by adding a
//! strict-superset wrapper and delegating cascade checks to the byte-stable
//! `omena-cascade` proof primitives.

use std::marker::PhantomData;

use omena_abstract_value::AbstractPropertyValueV0;
use omena_cascade::{
    CascadeDeclaration, CascadeRefinementContextV0,
    refine_declaration_in_context as refine_cascade_declaration_in_context,
};
use omena_refinement_trait::{
    PropertyIndexV0, REFINEMENT_FEATURE_GATE_V0, REFINEMENT_LAYER_MARKER_V0,
    REFINEMENT_SCHEMA_VERSION_V0, RefinementPredicateV0, RefinementWitnessV0,
};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AbstractValueShapeV0 {
    Bottom,
    Exact,
    FiniteSet,
    CustomPropertyReference,
    Top,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TopPredicateV0;

impl RefinementPredicateV0 for TopPredicateV0 {
    const PREDICATE_ID: &'static str = "top";
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AnyPropertyIndexV0;

impl PropertyIndexV0 for AnyPropertyIndexV0 {
    const PROPERTY_NAME: &'static str = "*";
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", bound = "")]
pub struct RefinedAbstractPropertyValueV0<P: PropertyIndexV0, R: RefinementPredicateV0> {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub property_name: &'static str,
    pub predicate_id: &'static str,
    pub value_shape: AbstractValueShapeV0,
    pub legacy_value: AbstractPropertyValueV0,
    pub strict_superset_of_legacy_v0: bool,
    #[serde(skip)]
    marker: PhantomData<(P, R)>,
}

pub fn project_legacy_to_refined_v0<P, R>(
    legacy_value: AbstractPropertyValueV0,
) -> RefinedAbstractPropertyValueV0<P, R>
where
    P: PropertyIndexV0,
    R: RefinementPredicateV0,
{
    RefinedAbstractPropertyValueV0 {
        schema_version: REFINEMENT_SCHEMA_VERSION_V0,
        product: "omena-refinement.refined-abstract-property-value",
        layer_marker: REFINEMENT_LAYER_MARKER_V0,
        feature_gate: REFINEMENT_FEATURE_GATE_V0,
        property_name: P::PROPERTY_NAME,
        predicate_id: R::PREDICATE_ID,
        value_shape: abstract_property_value_shape_v0(&legacy_value),
        legacy_value,
        strict_superset_of_legacy_v0: true,
        marker: PhantomData,
    }
}

pub fn project_refined_to_legacy_v0<P, R>(
    refined: &RefinedAbstractPropertyValueV0<P, R>,
) -> AbstractPropertyValueV0
where
    P: PropertyIndexV0,
    R: RefinementPredicateV0,
{
    refined.legacy_value.clone()
}

pub fn refine_declaration_in_context(
    declaration: &CascadeDeclaration,
    context: &CascadeRefinementContextV0,
) -> RefinementWitnessV0 {
    refine_cascade_declaration_in_context(declaration, context)
}

pub fn abstract_property_value_shape_v0(value: &AbstractPropertyValueV0) -> AbstractValueShapeV0 {
    match value {
        AbstractPropertyValueV0::Bottom { .. } => AbstractValueShapeV0::Bottom,
        AbstractPropertyValueV0::Exact { .. } => AbstractValueShapeV0::Exact,
        AbstractPropertyValueV0::FiniteSet { .. } => AbstractValueShapeV0::FiniteSet,
        AbstractPropertyValueV0::CustomPropertyReference { .. } => {
            AbstractValueShapeV0::CustomPropertyReference
        }
        AbstractPropertyValueV0::Top { .. } => AbstractValueShapeV0::Top,
    }
}

#[cfg(feature = "refinement-smt")]
pub fn refinement_smt_backend_available_v0() -> bool {
    let _ = omena_smt::cascade_theory_signature_v0();
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refined_value_round_trips_to_legacy_without_mutating_v0() {
        let legacy = AbstractPropertyValueV0::Top {
            property_name: "color".to_string(),
        };
        let refined =
            project_legacy_to_refined_v0::<AnyPropertyIndexV0, TopPredicateV0>(legacy.clone());
        assert_eq!(refined.schema_version, "0");
        assert_eq!(refined.layer_marker, "refinement-cascade");
        assert_eq!(project_refined_to_legacy_v0(&refined), legacy);
    }
}
