use crate::{
    TransformCssModuleComposesResolutionV0, TransformExecutionContextV0,
    TransformModuleEvaluationNativeEditV0, TransformModuleEvaluationOracleV0,
    TransformModuleEvaluationV0, execute_transform_passes_on_source_with_dialect,
    execute_transform_passes_on_source_with_dialect_and_context,
};
use omena_parser::StyleDialect;
use omena_transform_cst::TransformPassKind;

#[test]
fn execution_runtime_applies_explicit_scss_module_evaluation() {
    let source = r#".button { color: $brand; }"#;
    let evaluated_css = ".button { color: red; }";
    let context = TransformExecutionContextV0 {
        scss_module_evaluation: Some(TransformModuleEvaluationV0 {
            evaluator: "dart-sass-compatible".to_string(),
            evaluated_css: evaluated_css.to_string(),
            native_edit_output: Some(evaluated_css.to_string()),
            native_replacements: Vec::new(),
            native_edits: vec![native_module_evaluation_edit(source, "$brand", "red")],
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
        execution.outcomes.first().map(|outcome| outcome.detail),
        Some(
            "applied explicit SCSS module evaluation native edit output from the evaluator boundary"
        )
    );
    assert_eq!(
        execution.css_module_evaluation,
        Some(TransformModuleEvaluationV0 {
            evaluator: "dart-sass-compatible".to_string(),
            evaluated_css: evaluated_css.to_string(),
            native_edit_output: Some(evaluated_css.to_string()),
            native_replacements: Vec::new(),
            native_edits: vec![native_module_evaluation_edit(source, "$brand", "red")],
            oracle: None,
        })
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["scss-module-evaluate", "print-css"]
    );
}

#[test]
fn execution_runtime_prefers_native_output_over_legacy_evaluated_css() {
    let source = r#".button { color: $brand; }"#;
    let native_css = ".button { color: red; }";
    let legacy_css = ".button { color: legacy; }";
    let context = TransformExecutionContextV0 {
        scss_module_evaluation: Some(TransformModuleEvaluationV0 {
            evaluator: "dart-sass-compatible".to_string(),
            evaluated_css: legacy_css.to_string(),
            native_edit_output: Some(native_css.to_string()),
            native_replacements: Vec::new(),
            native_edits: vec![native_module_evaluation_edit(source, "$brand", "red")],
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
    assert_eq!(execution.output_css, native_css);
    assert!(!execution.output_css.contains("legacy"));
    assert_eq!(
        execution.outcomes.first().map(|outcome| outcome.detail),
        Some(
            "applied explicit SCSS module evaluation native edit output from the evaluator boundary"
        )
    );
}

#[test]
fn execution_runtime_applies_explicit_less_module_evaluation() {
    let source = r#".button { color: @brand; }"#;
    let evaluated_css = ".button { color: red; }";
    let context = TransformExecutionContextV0 {
        less_module_evaluation: Some(TransformModuleEvaluationV0 {
            evaluator: "less-js-compatible".to_string(),
            evaluated_css: evaluated_css.to_string(),
            native_edit_output: Some(evaluated_css.to_string()),
            native_replacements: Vec::new(),
            native_edits: vec![native_module_evaluation_edit(source, "@brand", "red")],
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
        execution.outcomes.first().map(|outcome| outcome.detail),
        Some(
            "applied explicit Less module evaluation native edit output from the evaluator boundary"
        )
    );
    assert_eq!(
        execution.css_module_evaluation,
        Some(TransformModuleEvaluationV0 {
            evaluator: "less-js-compatible".to_string(),
            evaluated_css: evaluated_css.to_string(),
            native_edit_output: Some(evaluated_css.to_string()),
            native_replacements: Vec::new(),
            native_edits: vec![native_module_evaluation_edit(source, "@brand", "red")],
            oracle: None,
        })
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["less-module-evaluate", "print-css"]
    );
}

#[test]
fn execution_runtime_prefers_less_native_output_over_legacy_evaluated_css() {
    let source = r#".button { color: @brand; }"#;
    let native_css = ".button { color: red; }";
    let legacy_css = ".button { color: legacy; }";
    let context = TransformExecutionContextV0 {
        less_module_evaluation: Some(TransformModuleEvaluationV0 {
            evaluator: "less-js-compatible".to_string(),
            evaluated_css: legacy_css.to_string(),
            native_edit_output: Some(native_css.to_string()),
            native_replacements: Vec::new(),
            native_edits: vec![native_module_evaluation_edit(source, "@brand", "red")],
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
    assert_eq!(execution.output_css, native_css);
    assert!(!execution.output_css.contains("legacy"));
    assert_eq!(
        execution.outcomes.first().map(|outcome| outcome.detail),
        Some(
            "applied explicit Less module evaluation native edit output from the evaluator boundary"
        )
    );
}

#[test]
fn execution_runtime_consumes_scss_oracle_output_without_native_edits() {
    let source = r#"@use "./tokens" as tokens; .button { color: tokens.$brand; }"#;
    let evaluated_css = ".base { color: blue; } .button { color: red; }";
    let context = TransformExecutionContextV0 {
        scss_module_evaluation: Some(TransformModuleEvaluationV0 {
            evaluator: "omena-query-static-scss-module-system-evaluator".to_string(),
            evaluated_css: evaluated_css.to_string(),
            native_edit_output: None,
            native_replacements: Vec::new(),
            native_edits: Vec::new(),
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
    assert_eq!(execution.output_css, evaluated_css);
    assert_eq!(
        execution.outcomes.first().map(|outcome| outcome.detail),
        Some("applied SCSS module evaluation oracle output from the evaluator boundary")
    );
}

#[test]
fn execution_runtime_consumes_less_oracle_output_without_native_edits() {
    let source = r#"@import "./tokens.less"; .button { color: @brand; }"#;
    let evaluated_css = ".base { color: blue; } .button { color: red; }";
    let context = TransformExecutionContextV0 {
        less_module_evaluation: Some(TransformModuleEvaluationV0 {
            evaluator: "omena-query-static-less-module-system-evaluator".to_string(),
            evaluated_css: evaluated_css.to_string(),
            native_edit_output: None,
            native_replacements: Vec::new(),
            native_edits: Vec::new(),
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
    assert_eq!(execution.output_css, evaluated_css);
    assert_eq!(
        execution.outcomes.first().map(|outcome| outcome.detail),
        Some("applied Less module evaluation oracle output from the evaluator boundary")
    );
}

#[test]
fn execution_runtime_consumes_oracle_output_when_native_edits_are_stale_but_oracle_matches() {
    let source =
        r#"@brand: red; .base { color: @brand; } @local: blue; .button { color: @local; }"#;
    let evaluated_css = "@brand: red; .base { color: @brand; }  .button { color: blue; }";
    let context = TransformExecutionContextV0 {
        less_module_evaluation: Some(TransformModuleEvaluationV0 {
            evaluator: "omena-query-static-less-variable-evaluator".to_string(),
            evaluated_css: evaluated_css.to_string(),
            native_edit_output: Some(evaluated_css.to_string()),
            native_replacements: Vec::new(),
            native_edits: vec![TransformModuleEvaluationNativeEditV0 {
                start: 67,
                end: 73,
                replacement: "blue".to_string(),
                edit_kind: "valueReplacement".to_string(),
                abstract_value: None,
                abstract_value_kind: None,
            }],
            oracle: Some(TransformModuleEvaluationOracleV0 {
                mode: "oracleOnly".to_string(),
                product_output_source: "legacyEvaluatedCss".to_string(),
                legacy_declaration_value_count: 1,
                abstract_value_count: 1,
                exact_value_count: 1,
                raw_value_count: 0,
                bottom_value_count: 0,
                top_value_count: 0,
                divergence_count: 0,
                all_legacy_declaration_values_preserved: true,
                native_replacement_count: 1,
                native_replacement_legacy_reflection_count: 1,
                native_replacement_legacy_unreflected_count: 0,
                native_value_reference_count: 1,
                native_resolved_value_count: 1,
                native_raw_value_count: 0,
                native_top_value_count: 0,
                native_cycle_count: 0,
                native_fuel_exhausted_count: 0,
                native_unresolved_reference_count: 0,
                native_unsupported_dynamic_count: 0,
            }),
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
    assert_eq!(execution.output_css, evaluated_css);
    assert_eq!(
        execution.outcomes.first().map(|outcome| outcome.detail),
        Some("applied Less module evaluation oracle output from the evaluator boundary")
    );
}

#[test]
fn execution_runtime_preserves_scss_source_when_native_edits_diverge_from_oracle() {
    let source = r#".button { color: $brand; }"#;
    let context = TransformExecutionContextV0 {
        scss_module_evaluation: Some(TransformModuleEvaluationV0 {
            evaluator: "dart-sass-compatible".to_string(),
            evaluated_css: ".button { color: red; }".to_string(),
            native_edit_output: Some(".button { color: red; }".to_string()),
            native_replacements: Vec::new(),
            native_edits: vec![native_module_evaluation_edit(source, "$brand", "blue")],
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

    assert_eq!(execution.mutation_count, 0);
    assert_eq!(execution.output_css, source);
    assert_eq!(
        execution.outcomes.first().map(|outcome| outcome.detail),
        Some(
            "preserved SCSS source because native evaluator edits did not match the oracle boundary"
        )
    );
}

#[test]
fn execution_runtime_preserves_less_source_when_native_edits_diverge_from_oracle() {
    let source = r#".button { color: @brand; }"#;
    let context = TransformExecutionContextV0 {
        less_module_evaluation: Some(TransformModuleEvaluationV0 {
            evaluator: "less-js-compatible".to_string(),
            evaluated_css: ".button { color: red; }".to_string(),
            native_edit_output: Some(".button { color: red; }".to_string()),
            native_replacements: Vec::new(),
            native_edits: vec![native_module_evaluation_edit(source, "@brand", "blue")],
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

    assert_eq!(execution.mutation_count, 0);
    assert_eq!(execution.output_css, source);
    assert_eq!(
        execution.outcomes.first().map(|outcome| outcome.detail),
        Some(
            "preserved Less source because native evaluator edits did not match the oracle boundary"
        )
    );
}

fn native_module_evaluation_edit(
    source: &str,
    needle: &str,
    replacement: &str,
) -> TransformModuleEvaluationNativeEditV0 {
    let Some(start) = source.find(needle) else {
        panic!("test fixture missing native edit needle: {needle}");
    };
    TransformModuleEvaluationNativeEditV0 {
        start,
        end: start + needle.len(),
        replacement: replacement.to_string(),
        edit_kind: "valueReplacement".to_string(),
        abstract_value: None,
        abstract_value_kind: None,
    }
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
