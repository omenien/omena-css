//! Transform-runner boundary for the `omena-query` facade.
//!
//! `omena-query` remains the consumer-facing analysis facade, but it should not
//! directly depend on every transform-family crate. This boundary groups the
//! transform planner, executor, target planner, printer, and egg witness surface
//! behind one dependency while preserving the existing query API.

use serde::Serialize;

pub use omena_transform_bundle::{
    TransformBundleSourceSummaryV0, summarize_omena_transform_bundle_from_source,
};
pub use omena_transform_cst::{TransformPassKind, all_transform_pass_kinds};
pub use omena_transform_egg::{
    EggRewriteSourceWitnessV0, TransformEggPlanV0, execute_egg_rewrite_witnesses_for_css_source,
    plan_egg_rewrite_passes_for_source,
};
pub use omena_transform_passes::{
    CustomPropertyLeastFixedPointSummaryV0, TransformClassNameRewriteV0,
    TransformCssModuleComposesResolutionV0, TransformCssModuleValueResolutionV0,
    TransformDesignTokenRouteV0, TransformExecutionContextV0, TransformExecutionSummaryV0,
    TransformImportInlineV0, TransformLessInlineLiteralPlaceholderV0, TransformModuleEvaluationV0,
    TransformPassPlanV0, execute_transform_passes_on_source_with_dialect_and_context,
    expand_css_nested_selector, inline_css_imports,
    inline_css_imports_for_static_module_evaluation, parse_static_css_cascade_value,
    plan_transform_passes, reduce_static_numeric_expression,
    resolve_static_css_modules_local_value_resolutions_from_source,
    restore_less_inline_literal_placeholders,
    summarize_static_css_custom_property_fixed_point_from_source,
};
#[cfg(feature = "lawvere-trace")]
pub use omena_transform_passes::{
    LawvereDifferentialCommutativityWitnessV0, LawvereModelTraceV0, ReorderabilityCertificateV0,
    TransformPassParallelPlanV0, evaluate_lawvere_reorderability_with_differential_corpus,
    execute_transform_passes_on_source_with_lawvere_trace_and_dialect,
    plan_transform_passes_parallel_lawvere_layers,
};
pub use omena_transform_print::{
    TransformPrintArtifactV0, TransformPrintMode, TransformPrintOptionsV0, default_print_options,
    print_transform_execution_artifact_with_dialect_and_source,
};
pub use omena_transform_target::{
    TargetFeatureSupportV0, TargetTransformOptionsV0, TransformTargetPlanV0,
    TransformTargetQueryPlanV0, conservative_target_options, modern_feature_support,
    plan_target_transforms, plan_target_transforms_from_query,
};

pub const OMENA_QUERY_TRANSFORM_RUNNER_COLLAPSED_CRATES_V0: [&str; 6] = [
    "omena-transform-bundle",
    "omena-transform-cst",
    "omena-transform-egg",
    "omena-transform-passes",
    "omena-transform-print",
    "omena-transform-target",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryTransformRunnerBoundaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub boundary_kind: &'static str,
    pub collapsed_transform_crate_count: usize,
    pub collapsed_transform_crates: Vec<&'static str>,
    pub direct_query_dependency_replacement: &'static str,
    pub ready_surfaces: Vec<&'static str>,
}

pub fn summarize_omena_query_transform_runner_boundary_v0() -> OmenaQueryTransformRunnerBoundaryV0 {
    let ready_surfaces = vec![
        "transformPlannerBoundary",
        "transformExecutorBoundary",
        "transformPrinterBoundary",
        "transformTargetPlannerBoundary",
        "transformEggWitnessBoundary",
    ];
    #[cfg(feature = "lawvere-trace")]
    let mut ready_surfaces = ready_surfaces;
    #[cfg(feature = "lawvere-trace")]
    ready_surfaces.push("transformLawvereTraceBoundary");

    OmenaQueryTransformRunnerBoundaryV0 {
        schema_version: "0",
        product: "omena-query-transform-runner.boundary",
        boundary_kind: "query-transform-runner-split",
        collapsed_transform_crate_count: OMENA_QUERY_TRANSFORM_RUNNER_COLLAPSED_CRATES_V0.len(),
        collapsed_transform_crates: OMENA_QUERY_TRANSFORM_RUNNER_COLLAPSED_CRATES_V0.to_vec(),
        direct_query_dependency_replacement: "omena-query-transform-runner",
        ready_surfaces,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform_runner_boundary_collapses_transform_family_for_query() {
        let summary = summarize_omena_query_transform_runner_boundary_v0();

        assert_eq!(summary.collapsed_transform_crate_count, 6);
        assert_eq!(
            summary.collapsed_transform_crates,
            OMENA_QUERY_TRANSFORM_RUNNER_COLLAPSED_CRATES_V0
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"transformExecutorBoundary")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"transformTargetPlannerBoundary")
        );
    }
}
