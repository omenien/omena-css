//! Transform pass registry and DAG planner for the post-v5 omena-css track.
//!
//! This crate consumes `omena-transform-cst` contracts. It does not duplicate
//! transform metadata; its job is to register safe mutations, cascade-proven
//! combinations, conservative lowerings, and emission boundaries as a
//! DAG-respecting execution plan for downstream transform crates.

pub use omena_cascade::CustomPropertyLeastFixedPointSummaryV0;

mod domains;
mod helpers;
mod model;
mod registry;
mod runtime;

pub use domains::css_modules_values::resolve_static_css_modules_local_value_resolutions_from_source;
pub use domains::number::reduce_static_numeric_expression;
pub use model::*;
pub use registry::{
    parse_static_css_cascade_value, summarize_static_css_custom_property_fixed_point_from_source,
};
pub use runtime::executor::{
    execute_transform_passes_on_source, execute_transform_passes_on_source_with_dialect,
    execute_transform_passes_on_source_with_dialect_and_context,
};
pub use runtime::fuzz::{run_transform_cascade_safe_fuzz_case, run_transform_fuzz_seed_corpus};
pub use runtime::incremental::{
    execute_transform_passes_incremental_with_database, transform_pass_incremental_graph_input,
};
pub use runtime::planner::{
    implemented_mutation_pass_ids, plan_transform_passes, summarize_omena_transform_passes_boundary,
};

#[cfg(test)]
mod tests;
