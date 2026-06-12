//! Abstract class-value and property-value domains for Omena product analysis.
//!
//! claim_level: product-wired class-value, selector-projection, provenance, and
//! reduced-product substrate; cascade-family remains research-staged and this is
//! not a completed abstract-interpretation theorem.

mod algebra;
mod cascade_family;
mod domain;
mod facts;
mod flow;
mod property_value;
mod provenance;
mod reduced_product;
mod registered_property;
mod selector_projection;
mod semiring;
mod types;

pub use algebra::*;
pub use cascade_family::*;
pub use domain::*;
pub use facts::*;
pub use flow::*;
pub use property_value::*;
pub use provenance::*;
pub use reduced_product::{
    concatenate_reduced_class_value_products, intersect_reduced_class_value_products,
    iterate_reduced_class_value_product_constraints, join_reduced_class_value_products,
    reduce_class_value_product, reduced_class_value_product_is_subset,
    reduced_class_value_product_matches_string, summarize_belief_propagation_iteration_v0,
    summarize_reduced_class_value_product,
    summarize_reduced_product_belief_propagation_domain_graph_v0, summarize_reduced_product_domain,
};
pub use registered_property::*;
pub use selector_projection::*;
pub use semiring::*;
pub use types::*;

#[cfg(test)]
mod tests;

pub const ABSTRACT_VALUE_CLAIM_LEVEL_V0: &str =
    "productWiredClassValueSelectorProjectionProvenanceReducedProductSubstrate";
pub const ABSTRACT_VALUE_CASCADE_FAMILY_CLAIM_LEVEL_V0: &str =
    "researchStagedCascadeFamilySubstrate";
