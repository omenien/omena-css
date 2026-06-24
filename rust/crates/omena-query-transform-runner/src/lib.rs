//! Transform-runner boundary for the `omena-query` facade.
//!
//! `omena-query` remains the consumer-facing analysis facade, but it should not
//! directly depend on every transform-family crate. This boundary groups the
//! transform planner, executor, target planner, printer, and egg witness surface
//! behind one dependency while preserving the existing query API.

use serde::Serialize;

pub use omena_bundler::{
    TransformBundleAssetUrlRewriteSummaryV0, TransformBundleEdgeKind,
    TransformBundleSourceSummaryV0, rewrite_omena_transform_bundle_asset_urls_in_source,
    summarize_omena_transform_bundle_from_source,
};
pub use omena_transform_cst::{
    NATIVE_CSS_STATIC_EVAL_DIALECT_RESTRICTION_V0, NATIVE_CSS_STATIC_EVAL_OPT_IN_POLICY_V0,
    NATIVE_CSS_STATIC_EVAL_SPEC_SNAPSHOT_V0, TransformPassKind, all_transform_pass_kinds,
};
pub use omena_transform_egg::{
    EggRewriteSourceWitnessV0, TransformEggPlanV0, execute_egg_rewrite_witnesses_for_css_source,
    plan_egg_rewrite_passes_for_source,
};
pub use omena_transform_passes::{
    CustomPropertyLeastFixedPointSummaryV0, TransformClassNameRewriteV0,
    TransformCssModuleComposesResolutionV0, TransformCssModuleValueResolutionV0,
    TransformDesignTokenRouteV0, TransformExecutionContextV0, TransformExecutionSummaryV0,
    TransformImportInlineV0, TransformLessInlineLiteralPlaceholderV0,
    TransformModuleEvaluationNativeEditV0, TransformModuleEvaluationNativeReplacementV0,
    TransformModuleEvaluationOracleV0, TransformModuleEvaluationV0, TransformPassPlanV0,
    execute_transform_passes_on_source_with_dialect_and_context, expand_css_nested_selector,
    inline_css_imports, inline_css_imports_for_static_module_evaluation,
    parse_static_css_cascade_value, plan_transform_passes, reduce_static_numeric_expression,
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
    TransformPrintArtifactV0, TransformPrintMode, TransformPrintOptionsV0,
    TransformSourceMapCompositionV0, TransformSourceMapPointV0, TransformSourceMapSegmentV0,
    TransformSourceMapV3V0, compose_transform_source_map_v3_with_upstream_map,
    default_print_options, print_transform_execution_artifact_with_dialect_and_source,
    serialize_transform_source_map_v3, serialize_transform_source_map_v3_with_source_contents,
    transform_source_map_point,
};
pub use omena_transform_target::{
    TargetFeatureSupportV0, TargetTransformOptionsV0, TransformTargetPlanV0,
    TransformTargetQueryPlanV0, conservative_target_options, modern_feature_support,
    plan_target_transforms, plan_target_transforms_from_query,
};

pub const OMENA_QUERY_TRANSFORM_RUNNER_COLLAPSED_CRATES_V0: [&str; 6] = [
    "omena-bundler",
    "omena-transform-cst",
    "omena-transform-egg",
    "omena-transform-passes",
    "omena-transform-print",
    "omena-transform-target",
];

pub fn materialize_transform_module_evaluation_native_edits(
    input_css: &str,
    native_edits: &[TransformModuleEvaluationNativeEditV0],
) -> Option<String> {
    if native_edits.is_empty() {
        return None;
    }

    let mut edits = native_edits.to_vec();
    edits.sort_by_key(|edit| edit.start);

    let mut previous_end = 0usize;
    for edit in &edits {
        if edit.start < previous_end
            || edit.start > edit.end
            || edit.end > input_css.len()
            || !input_css.is_char_boundary(edit.start)
            || !input_css.is_char_boundary(edit.end)
        {
            return None;
        }
        previous_end = edit.end;
    }

    let mut output = input_css.to_string();
    for edit in edits.iter().rev() {
        output.replace_range(edit.start..edit.end, edit.replacement.as_str());
    }
    Some(output)
}

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
        "transformSourceMapV3SerializerBoundary",
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

        assert_eq!(summary.collapsed_transform_crate_count, 5);
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

    #[test]
    fn transform_runner_materializes_sorted_native_edits() {
        let input_css = ".button { color: red; margin: 1px; }";
        let color_start = input_css.find("red").unwrap_or(input_css.len());
        let margin_start = input_css.find("1px").unwrap_or(input_css.len());

        let output = materialize_transform_module_evaluation_native_edits(
            input_css,
            &[
                TransformModuleEvaluationNativeEditV0 {
                    start: margin_start,
                    end: margin_start + "1px".len(),
                    replacement: "2px".to_string(),
                    edit_kind: "value".to_string(),
                    abstract_value: None,
                    abstract_value_kind: None,
                },
                TransformModuleEvaluationNativeEditV0 {
                    start: color_start,
                    end: color_start + "red".len(),
                    replacement: "blue".to_string(),
                    edit_kind: "value".to_string(),
                    abstract_value: None,
                    abstract_value_kind: None,
                },
            ],
        );

        assert_eq!(
            output.as_deref(),
            Some(".button { color: blue; margin: 2px; }")
        );
    }

    #[test]
    fn transform_runner_rejects_overlapping_native_edits() {
        let output = materialize_transform_module_evaluation_native_edits(
            "abcdef",
            &[
                TransformModuleEvaluationNativeEditV0 {
                    start: 1,
                    end: 4,
                    replacement: "x".to_string(),
                    edit_kind: "value".to_string(),
                    abstract_value: None,
                    abstract_value_kind: None,
                },
                TransformModuleEvaluationNativeEditV0 {
                    start: 3,
                    end: 5,
                    replacement: "y".to_string(),
                    edit_kind: "value".to_string(),
                    abstract_value: None,
                    abstract_value_kind: None,
                },
            ],
        );

        assert_eq!(output, None);
    }
}
