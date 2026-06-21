use std::collections::{BTreeMap, BTreeSet};

use omena_parser::StyleDialect as OmenaParserStyleDialect;
use omena_query_transform_runner::{
    TransformModuleEvaluationNativeEditV0, TransformModuleEvaluationNativeReplacementV0,
    TransformModuleEvaluationOracleV0, TransformModuleEvaluationV0,
};
use omena_scss_eval::{
    OmenaScssEvalStaticStylesheetEvaluationV0,
    derive_static_scss_stylesheet_module_configurable_variable_names as derive_omena_scss_eval_static_scss_stylesheet_module_configurable_variable_names,
    derive_static_scss_stylesheet_module_variable_exports as derive_omena_scss_eval_static_scss_stylesheet_module_variable_exports,
    derive_static_stylesheet_module_evaluation as derive_omena_scss_eval_static_stylesheet_module_evaluation,
};

pub(super) use omena_scss_eval::{
    canonical_static_scss_variable_name, static_scss_variable_names_equal,
};

pub(super) fn derive_static_stylesheet_module_evaluation(
    style_source: &str,
    dialect: OmenaParserStyleDialect,
) -> Option<TransformModuleEvaluationV0> {
    let evaluation =
        derive_omena_scss_eval_static_stylesheet_module_evaluation(style_source, dialect)?;
    // NOTE: `all_legacy_declaration_values_preserved` is a value-WELL-FORMEDNESS self-check on the
    // native-edit output (every native-emitted declaration value canonically round-trips), NOT a
    // differential against an external SCSS/Less compiler. See `summarize_omena_scss_eval_oracle`.
    debug_assert!(
        evaluation.oracle.all_legacy_declaration_values_preserved,
        "native-emitted value failed canonical round-trip self-check"
    );
    let oracle = transform_module_evaluation_oracle(&evaluation);
    Some(TransformModuleEvaluationV0 {
        evaluator: evaluation.evaluator.to_string(),
        product_output_source: Some(evaluation.product_output_source.to_string()),
        evaluated_css: evaluation.evaluated_css,
        native_edit_output: Some(evaluation.native_edit_output),
        native_replacements: evaluation
            .resolved_replacements
            .into_iter()
            .map(transform_module_evaluation_native_replacement)
            .collect(),
        native_edits: evaluation
            .native_edits
            .into_iter()
            .map(transform_module_evaluation_native_edit)
            .collect(),
        oracle: Some(oracle),
    })
}

fn transform_module_evaluation_native_replacement(
    replacement: omena_scss_eval::OmenaScssEvalResolvedReplacementV0,
) -> TransformModuleEvaluationNativeReplacementV0 {
    TransformModuleEvaluationNativeReplacementV0 {
        name: replacement.name,
        start: replacement.start,
        end: replacement.end,
        text: replacement.text,
        rendered_value: replacement.rendered_value,
        abstract_value: replacement.abstract_value,
        abstract_value_kind: replacement.abstract_value_kind.to_string(),
    }
}

fn transform_module_evaluation_native_edit(
    edit: omena_scss_eval::OmenaScssEvalStaticStylesheetNativeEditV0,
) -> TransformModuleEvaluationNativeEditV0 {
    TransformModuleEvaluationNativeEditV0 {
        start: edit.start,
        end: edit.end,
        replacement: edit.replacement,
        edit_kind: edit.edit_kind.to_string(),
        abstract_value: edit.abstract_value,
        abstract_value_kind: edit.abstract_value_kind.map(str::to_string),
    }
}

fn transform_module_evaluation_oracle(
    evaluation: &OmenaScssEvalStaticStylesheetEvaluationV0,
) -> TransformModuleEvaluationOracleV0 {
    TransformModuleEvaluationOracleV0 {
        mode: evaluation.oracle.mode.to_string(),
        product_output_source: evaluation.oracle.product_output_source.to_string(),
        legacy_declaration_value_count: evaluation.oracle.legacy_declaration_value_count,
        abstract_value_count: evaluation.oracle.values.len(),
        exact_value_count: evaluation.oracle.exact_value_count,
        raw_value_count: evaluation.oracle.raw_value_count,
        bottom_value_count: evaluation.oracle.bottom_value_count,
        top_value_count: evaluation.oracle.top_value_count,
        divergence_count: evaluation.oracle.divergence_count,
        all_legacy_declaration_values_preserved: evaluation
            .oracle
            .all_legacy_declaration_values_preserved,
        native_replacement_count: evaluation.replacement_count,
        native_replacement_legacy_reflection_count: evaluation
            .native_replacement_legacy_reflection_count,
        native_replacement_legacy_unreflected_count: evaluation
            .native_replacement_legacy_unreflected_count,
        native_value_reference_count: evaluation.value_resolution.reference_count,
        native_resolved_value_count: evaluation.value_resolution.resolved_count,
        native_raw_value_count: evaluation.value_resolution.raw_count,
        native_top_value_count: evaluation.value_resolution.top_count,
        native_cycle_count: evaluation.value_resolution.cycle_count,
        native_fuel_exhausted_count: evaluation.value_resolution.fuel_exhausted_count,
        native_unresolved_reference_count: evaluation.value_resolution.unresolved_reference_count,
        native_unsupported_dynamic_count: evaluation.value_resolution.unsupported_dynamic_count,
    }
}

pub(super) fn derive_static_scss_stylesheet_module_variable_exports(
    style_source: &str,
) -> BTreeMap<String, String> {
    derive_omena_scss_eval_static_scss_stylesheet_module_variable_exports(style_source)
}

pub(super) fn derive_static_scss_stylesheet_module_configurable_variable_names(
    style_source: &str,
) -> BTreeSet<String> {
    derive_omena_scss_eval_static_scss_stylesheet_module_configurable_variable_names(style_source)
}
