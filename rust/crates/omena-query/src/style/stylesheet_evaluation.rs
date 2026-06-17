use std::collections::{BTreeMap, BTreeSet};

use omena_parser::StyleDialect as OmenaParserStyleDialect;
use omena_query_transform_runner::TransformModuleEvaluationV0;
use omena_scss_eval::{
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
    debug_assert!(
        evaluation.oracle.all_legacy_declaration_values_preserved,
        "native SCSS/Less value oracle diverged from legacy evaluated_css"
    );
    Some(TransformModuleEvaluationV0 {
        evaluator: evaluation.evaluator.to_string(),
        evaluated_css: evaluation.evaluated_css,
    })
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
