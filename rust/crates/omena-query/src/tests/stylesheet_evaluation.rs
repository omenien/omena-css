use crate::execute_omena_query_consumer_build_style_source;

#[test]
fn consumer_build_derives_static_scss_evaluator_context() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.scss",
        "$brand: red; .button { color: $brand; }",
        &[
            "scss-module-evaluate".to_string(),
            "css-modules-class-hashing".to_string(),
            "print-css".to_string(),
        ],
    );

    assert!(
        summary
            .execution
            .executed_pass_ids
            .contains(&"scss-module-evaluate")
    );
    assert!(
        !summary
            .execution
            .planned_only_pass_ids
            .contains(&"scss-module-evaluate")
    );
    assert!(summary.execution.output_css.contains("color: red"));
    assert!(summary.execution.output_css.contains("._button_0"));
    assert_eq!(
        summary
            .execution
            .css_module_evaluation
            .as_ref()
            .map(|evaluation| evaluation.evaluator.as_str()),
        Some("omena-query-static-scss-variable-evaluator")
    );
}

#[test]
fn consumer_build_derives_static_less_evaluator_context() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "@brand: red; .button { color: @brand; }",
        &[
            "less-module-evaluate".to_string(),
            "css-modules-class-hashing".to_string(),
            "print-css".to_string(),
        ],
    );

    assert!(
        summary
            .execution
            .executed_pass_ids
            .contains(&"less-module-evaluate")
    );
    assert!(
        !summary
            .execution
            .planned_only_pass_ids
            .contains(&"less-module-evaluate")
    );
    assert!(summary.execution.output_css.contains("color: red"));
    assert!(summary.execution.output_css.contains("._button_0"));
    assert_eq!(
        summary
            .execution
            .css_module_evaluation
            .as_ref()
            .map(|evaluation| evaluation.evaluator.as_str()),
        Some("omena-query-static-less-variable-evaluator")
    );
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_forward_references() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        ".button { color: @accent; } @accent: @brand; @brand: red;",
        &[
            "less-module-evaluate".to_string(),
            "css-modules-class-hashing".to_string(),
            "print-css".to_string(),
        ],
    );

    assert!(
        summary
            .execution
            .executed_pass_ids
            .contains(&"less-module-evaluate")
    );
    assert!(
        !summary
            .execution
            .planned_only_pass_ids
            .contains(&"less-module-evaluate")
    );
    assert!(summary.execution.output_css.contains("color: red"));
    assert!(!summary.execution.output_css.contains("@accent:"));
    assert!(!summary.execution.output_css.contains("@brand:"));
    assert_eq!(
        summary
            .execution
            .css_module_evaluation
            .as_ref()
            .map(|evaluation| evaluation.evaluator.as_str()),
        Some("omena-query-static-less-variable-evaluator")
    );
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_with_last_declaration_wins() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "@brand: red; .button { color: @brand; } @brand: blue;",
        &[
            "less-module-evaluate".to_string(),
            "css-modules-class-hashing".to_string(),
            "print-css".to_string(),
        ],
    );

    assert!(
        summary
            .execution
            .executed_pass_ids
            .contains(&"less-module-evaluate")
    );
    assert!(summary.execution.output_css.contains("color: blue"));
    assert!(!summary.execution.output_css.contains("@brand:"));
    assert_eq!(
        summary
            .execution
            .css_module_evaluation
            .as_ref()
            .map(|evaluation| evaluation.evaluator.as_str()),
        Some("omena-query-static-less-variable-evaluator")
    );
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_scoped_variables() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "@gap: 1rem; .card { @gap: 2rem; color: @gap; } .other { color: @gap; }",
        &[
            "less-module-evaluate".to_string(),
            "css-modules-class-hashing".to_string(),
            "print-css".to_string(),
        ],
    );

    assert!(
        summary
            .execution
            .executed_pass_ids
            .contains(&"less-module-evaluate")
    );
    assert!(
        !summary
            .execution
            .planned_only_pass_ids
            .contains(&"less-module-evaluate")
    );
    assert!(summary.execution.output_css.contains("color: 2rem"));
    assert!(summary.execution.output_css.contains("color: 1rem"));
    assert!(!summary.execution.output_css.contains("@gap:"));
    assert_eq!(
        summary
            .execution
            .css_module_evaluation
            .as_ref()
            .map(|evaluation| evaluation.evaluator.as_str()),
        Some("omena-query-static-less-variable-evaluator")
    );
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_lazy_scoped_values() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "@tone: @brand; @brand: blue; .card { @brand: red; color: @tone; } .other { color: @tone; }",
        &[
            "less-module-evaluate".to_string(),
            "css-modules-class-hashing".to_string(),
            "print-css".to_string(),
        ],
    );

    assert!(
        summary
            .execution
            .executed_pass_ids
            .contains(&"less-module-evaluate")
    );
    assert!(summary.execution.output_css.contains("color: red"));
    assert!(summary.execution.output_css.contains("color: blue"));
    assert!(!summary.execution.output_css.contains("@tone:"));
    assert!(!summary.execution.output_css.contains("@brand:"));
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_property_variables() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        ".card { color: red; background: $color; color: blue; } .other { color: green; background: $color; }",
        &[
            "less-module-evaluate".to_string(),
            "css-modules-class-hashing".to_string(),
            "print-css".to_string(),
        ],
    );

    assert!(
        summary
            .execution
            .executed_pass_ids
            .contains(&"less-module-evaluate")
    );
    assert!(
        !summary
            .execution
            .planned_only_pass_ids
            .contains(&"less-module-evaluate")
    );
    assert!(summary.execution.output_css.contains("background: blue"));
    assert!(summary.execution.output_css.contains("background: green"));
    assert!(!summary.execution.output_css.contains("$color"));
    assert_eq!(
        summary
            .execution
            .css_module_evaluation
            .as_ref()
            .map(|evaluation| evaluation.evaluator.as_str()),
        Some("omena-query-static-less-variable-evaluator")
    );
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_parenthesized_arithmetic() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "@width: 100px; @half: (@width / 2); @sum: (@half + 10px); .card { width: @half; margin: @sum; }",
        &[
            "less-module-evaluate".to_string(),
            "css-modules-class-hashing".to_string(),
            "print-css".to_string(),
        ],
    );

    assert!(
        summary
            .execution
            .executed_pass_ids
            .contains(&"less-module-evaluate")
    );
    assert!(summary.execution.output_css.contains("width: 50px"));
    assert!(summary.execution.output_css.contains("margin: 60px"));
    assert!(!summary.execution.output_css.contains("@width:"));
    assert!(!summary.execution.output_css.contains("@half:"));
    assert!(!summary.execution.output_css.contains("@sum:"));
}

#[test]
fn consumer_build_derives_static_scss_evaluator_context_with_default_declarations() {
    let first_default_summary = execute_omena_query_consumer_build_style_source(
        "Button.module.scss",
        "$brand: red !default; $accent: $brand !default; .button { color: $accent; }",
        &[
            "scss-module-evaluate".to_string(),
            "css-modules-class-hashing".to_string(),
            "print-css".to_string(),
        ],
    );
    let existing_value_summary = execute_omena_query_consumer_build_style_source(
        "Button.module.scss",
        "$brand: red; $brand: blue !default; .button { color: $brand; }",
        &[
            "scss-module-evaluate".to_string(),
            "css-modules-class-hashing".to_string(),
            "print-css".to_string(),
        ],
    );
    let later_assignment_summary = execute_omena_query_consumer_build_style_source(
        "Button.module.scss",
        "$brand: red !default; $brand: blue; .button { color: $brand; }",
        &[
            "scss-module-evaluate".to_string(),
            "css-modules-class-hashing".to_string(),
            "print-css".to_string(),
        ],
    );

    assert!(
        first_default_summary
            .execution
            .output_css
            .contains("color: red")
    );
    assert!(
        !first_default_summary
            .execution
            .output_css
            .contains("!default")
    );
    assert!(
        existing_value_summary
            .execution
            .output_css
            .contains("color: red")
    );
    assert!(
        later_assignment_summary
            .execution
            .output_css
            .contains("color: blue")
    );
    assert!(
        !later_assignment_summary
            .execution
            .output_css
            .contains("$brand:")
    );
}

#[test]
fn consumer_build_derives_static_stylesheet_evaluator_context_for_composite_values() {
    let scss_summary = execute_omena_query_consumer_build_style_source(
        "Button.module.scss",
        "$brand: red; $border: 1px solid $brand; .button { border: $border; }",
        &[
            "scss-module-evaluate".to_string(),
            "css-modules-class-hashing".to_string(),
            "print-css".to_string(),
        ],
    );
    let less_summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "@brand: red; @border: 1px solid @brand; .button { border: @border; }",
        &[
            "less-module-evaluate".to_string(),
            "css-modules-class-hashing".to_string(),
            "print-css".to_string(),
        ],
    );

    assert!(
        scss_summary
            .execution
            .output_css
            .contains("border: 1px solid red")
    );
    assert!(
        less_summary
            .execution
            .output_css
            .contains("border: 1px solid red")
    );
    assert!(!scss_summary.execution.output_css.contains("$border:"));
    assert!(!less_summary.execution.output_css.contains("@border:"));
}

#[test]
fn consumer_build_keeps_static_scss_evaluator_planned_for_forward_composite_values() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.scss",
        "$border: 1px solid $brand; $brand: red; .button { border: $border; }",
        &[
            "scss-module-evaluate".to_string(),
            "css-modules-class-hashing".to_string(),
            "print-css".to_string(),
        ],
    );

    assert!(
        summary
            .execution
            .planned_only_pass_ids
            .contains(&"scss-module-evaluate")
    );
    assert!(summary.execution.output_css.contains("$border:"));
    assert!(summary.execution.output_css.contains("$brand:"));
}

#[test]
fn consumer_build_derives_static_scss_evaluator_context_for_reassignments() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.scss",
        "$brand: red; .button { color: $brand; } $brand: blue; .link { color: $brand; }",
        &[
            "scss-module-evaluate".to_string(),
            "css-modules-class-hashing".to_string(),
            "print-css".to_string(),
        ],
    );

    assert!(
        summary
            .execution
            .executed_pass_ids
            .contains(&"scss-module-evaluate")
    );
    assert!(
        !summary
            .execution
            .planned_only_pass_ids
            .contains(&"scss-module-evaluate")
    );
    assert!(summary.execution.output_css.contains("color: red"));
    assert!(summary.execution.output_css.contains("color: blue"));
    assert!(!summary.execution.output_css.contains("$brand:"));
}

#[test]
fn consumer_build_derives_static_scss_evaluator_context_for_local_scope() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.scss",
        "$brand: blue; .card { $brand: red; color: $brand; } .other { color: $brand; }",
        &[
            "scss-module-evaluate".to_string(),
            "css-modules-class-hashing".to_string(),
            "print-css".to_string(),
        ],
    );

    assert!(
        summary
            .execution
            .executed_pass_ids
            .contains(&"scss-module-evaluate")
    );
    assert!(summary.execution.output_css.contains("color: red"));
    assert!(summary.execution.output_css.contains("color: blue"));
    assert!(!summary.execution.output_css.contains("$brand:"));
}

#[test]
fn consumer_build_derives_static_scss_evaluator_context_for_global_assignments() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.scss",
        "$brand: blue; .card { $brand: red !global; color: $brand; } .other { color: $brand; }",
        &[
            "scss-module-evaluate".to_string(),
            "css-modules-class-hashing".to_string(),
            "print-css".to_string(),
        ],
    );

    assert!(
        summary
            .execution
            .executed_pass_ids
            .contains(&"scss-module-evaluate")
    );
    assert!(summary.execution.output_css.contains("color: red"));
    assert!(!summary.execution.output_css.contains("color: blue"));
    assert!(!summary.execution.output_css.contains("$brand:"));
}
