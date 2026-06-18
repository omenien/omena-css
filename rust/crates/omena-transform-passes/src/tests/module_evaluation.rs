use crate::{
    TransformCssModuleComposesResolutionV0, TransformExecutionContextV0,
    TransformModuleEvaluationV0, execute_transform_passes_on_source_with_dialect,
    execute_transform_passes_on_source_with_dialect_and_context,
};
use omena_parser::StyleDialect;
use omena_transform_cst::TransformPassKind;

#[test]
fn execution_runtime_applies_explicit_scss_module_evaluation() {
    let source = r#"$brand: red; .button { color: $brand; }"#;
    let context = TransformExecutionContextV0 {
        scss_module_evaluation: Some(TransformModuleEvaluationV0 {
            evaluator: "dart-sass-compatible".to_string(),
            evaluated_css: ".button { color: red; }".to_string(),
            oracle: None,
        }),
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Scss,
        &[
            TransformPassKind::ScssModuleEvaluate,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(execution.output_css, ".button { color: red; }");
    assert_eq!(
        execution.css_module_evaluation,
        Some(TransformModuleEvaluationV0 {
            evaluator: "dart-sass-compatible".to_string(),
            evaluated_css: ".button { color: red; }".to_string(),
            oracle: None,
        })
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["scss-module-evaluate", "print-css"]
    );
}

#[test]
fn execution_runtime_applies_explicit_less_module_evaluation() {
    let source = r#"@brand: red; .button { color: @brand; }"#;
    let context = TransformExecutionContextV0 {
        less_module_evaluation: Some(TransformModuleEvaluationV0 {
            evaluator: "less-js-compatible".to_string(),
            evaluated_css: ".button { color: red; }".to_string(),
            oracle: None,
        }),
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Less,
        &[
            TransformPassKind::LessModuleEvaluate,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(execution.output_css, ".button { color: red; }");
    assert_eq!(
        execution.css_module_evaluation,
        Some(TransformModuleEvaluationV0 {
            evaluator: "less-js-compatible".to_string(),
            evaluated_css: ".button { color: red; }".to_string(),
            oracle: None,
        })
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["less-module-evaluate", "print-css"]
    );
}

#[test]
fn execution_runtime_resolves_css_module_composes_with_export_set() {
    let source = r#".button { composes: base from "./base.module.css"; color: red; } .button:hover { color: blue; } .card, .panel { composes: shared; color: green; } :local(.card) { composes: shared; color: yellow; } :local(.card, .panel) { composes: shared; color: purple; } :local { .button { composes: base; color: navy; } } :global { .button { composes: base; color: pink; } } @media (min-width: 1px) { .button { composes: base; color: black; } }"#;
    let context = TransformExecutionContextV0 {
        css_module_composes_resolutions: vec![
            TransformCssModuleComposesResolutionV0 {
                local_class_name: "button".to_string(),
                exported_class_names: vec!["button".to_string(), "base".to_string()],
            },
            TransformCssModuleComposesResolutionV0 {
                local_class_name: "card".to_string(),
                exported_class_names: vec!["card".to_string(), "shared".to_string()],
            },
            TransformCssModuleComposesResolutionV0 {
                local_class_name: "panel".to_string(),
                exported_class_names: vec!["panel".to_string(), "shared".to_string()],
            },
        ],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::ResolveCssModulesComposes,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 6);
    assert_eq!(
        execution.output_css,
        r#".button {  color: red; } .button:hover { color: blue; } .card, .panel {  color: green; } :local(.card) {  color: yellow; } :local(.card, .panel) {  color: purple; } :local { .button {  color: navy; } } :global { .button { composes: base; color: pink; } } @media (min-width: 1px) { .button {  color: black; } }"#
    );
    assert_eq!(
        execution.css_module_composes_exports,
        vec![
            TransformCssModuleComposesResolutionV0 {
                local_class_name: "button".to_string(),
                exported_class_names: vec!["button".to_string(), "base".to_string()],
            },
            TransformCssModuleComposesResolutionV0 {
                local_class_name: "card".to_string(),
                exported_class_names: vec!["card".to_string(), "shared".to_string()],
            },
            TransformCssModuleComposesResolutionV0 {
                local_class_name: "panel".to_string(),
                exported_class_names: vec!["panel".to_string(), "shared".to_string()],
            },
        ]
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["composes-resolution", "print-css"]
    );
}

#[test]
fn execution_runtime_resolves_local_css_module_composes_without_explicit_export_set() {
    let source = r#".button { composes: base global(reset); color: red; } .base { composes: utility; color: blue; } .utility { color: green; }"#;
    let execution = execute_transform_passes_on_source_with_dialect(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::ResolveCssModulesComposes,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#".button {  color: red; } .base {  color: blue; } .utility { color: green; }"#
    );
    assert_eq!(
        execution.css_module_composes_exports,
        vec![
            TransformCssModuleComposesResolutionV0 {
                local_class_name: "base".to_string(),
                exported_class_names: vec!["base".to_string(), "utility".to_string()],
            },
            TransformCssModuleComposesResolutionV0 {
                local_class_name: "button".to_string(),
                exported_class_names: vec![
                    "button".to_string(),
                    "base".to_string(),
                    "reset".to_string(),
                    "utility".to_string(),
                ],
            },
        ]
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["composes-resolution", "print-css"]
    );
    assert!(execution.planned_only_pass_ids.is_empty());
}
