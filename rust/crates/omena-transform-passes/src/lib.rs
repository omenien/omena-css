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
pub use domains::vendor_prefix::StaleVendorPrefixRemovalProofCandidateV0;
pub use model::*;
#[cfg(feature = "lawvere-trace")]
pub use omena_lawvere::{
    LawvereDifferentialCommutativityWitnessV0, LawvereModelTraceV0, ReorderabilityCertificateV0,
    TransformPassParallelPlanV0,
};
pub use omena_value_lattice::{
    StaticSrgbColorWithAlpha, can_shorten_hex_pairs, compress_hex_color_token_text,
    compress_number_prefix, compress_numeric_token_text, format_css_number, numeric_prefix_end,
    parse_basic_named_static_color_with_alpha, parse_color_function_value, parse_color_mix_value,
    parse_numeric_value_with_unit, parse_oklab_oklch_value, parse_reducible_abs_value,
    parse_reducible_calc_value, parse_reducible_clamp_value, parse_reducible_exp_value,
    parse_reducible_hypot_value, parse_reducible_log_value, parse_reducible_max_value,
    parse_reducible_min_value, parse_reducible_mod_value, parse_reducible_pow_value,
    parse_reducible_rem_value, parse_reducible_round_value, parse_reducible_sign_value,
    parse_reducible_sqrt_value, parse_static_hsl_function_color_with_alpha,
    parse_static_hwb_function_color_with_alpha, parse_static_rgb_function_color_with_alpha,
    parse_static_srgb_color, parse_static_srgb_color_with_alpha, shorten_hex_pairs,
    shortest_static_srgb_color_with_alpha_text,
};
pub use registry::{
    inline_css_imports, inline_css_imports_for_static_module_evaluation,
    parse_static_css_cascade_value, restore_less_inline_literal_placeholders,
    summarize_static_css_custom_property_fixed_point_from_source,
};
#[cfg(feature = "lawvere-trace")]
pub use runtime::executor::{
    evaluate_lawvere_reorderability_with_differential_corpus,
    execute_transform_passes_on_source_with_lawvere_trace,
    execute_transform_passes_on_source_with_lawvere_trace_and_dialect,
};
pub use runtime::executor::{
    execute_transform_passes_on_source, execute_transform_passes_on_source_with_dialect,
    execute_transform_passes_on_source_with_dialect_and_context,
    execute_transform_passes_on_source_with_dialect_and_context_without_lex_cache_for_measurement,
};
pub use runtime::fuzz::{run_transform_cascade_safe_fuzz_case, run_transform_fuzz_seed_corpus};
pub use runtime::incremental::{
    execute_transform_passes_incremental_with_database, transform_pass_incremental_graph_input,
};
pub use runtime::lex_cache::{
    reset_transform_lex_cache_splice_telemetry, transform_lex_cache_splice_telemetry_snapshot,
};
#[cfg(feature = "lawvere-trace")]
pub use runtime::planner::plan_transform_passes_parallel_lawvere_layers;
pub use runtime::planner::{
    implemented_mutation_pass_ids, plan_transform_passes, summarize_omena_transform_passes_boundary,
};

/// Expand a CSS Nesting selector against its canonical parent selector.
///
/// This is exposed for analysis/query layers that must compare selectors in
/// their resolved form without running the full transform pipeline.
pub fn expand_css_nested_selector(parent_selector: &str, nested_selector: &str) -> Option<String> {
    domains::nesting::expand_nested_selector(parent_selector, nested_selector)
}

/// Return proof candidates for stale vendor-prefix removals.
///
/// Each candidate identifies both the removable prefixed declaration and the
/// exact unprefixed peer that must survive for the rewrite to be considered.
pub fn collect_stale_vendor_prefix_removal_proof_candidates_from_source(
    source: &str,
    dialect: omena_parser::StyleDialect,
) -> Vec<StaleVendorPrefixRemovalProofCandidateV0> {
    domains::vendor_prefix::collect_stale_vendor_prefix_removal_proof_candidates_with_lexer(
        source, dialect,
    )
}

#[cfg(test)]
mod tests;
