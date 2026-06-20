use crate::{
    OmenaQueryStyleSourceInputV0, execute_omena_query_consumer_build_style_source,
    execute_omena_query_consumer_build_style_sources,
};

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
            .outcomes
            .first()
            .map(|outcome| outcome.detail),
        Some(
            "applied explicit SCSS module evaluation native edit output from the evaluator boundary"
        )
    );
    assert_eq!(
        summary
            .execution
            .css_module_evaluation
            .as_ref()
            .map(|evaluation| evaluation.evaluator.as_str()),
        Some("omena-query-static-scss-variable-evaluator")
    );
    assert_eq!(
        summary
            .execution
            .css_module_evaluation
            .as_ref()
            .map(|evaluation| evaluation.native_replacements.len()),
        Some(1)
    );
    assert_eq!(
        summary
            .execution
            .css_module_evaluation
            .as_ref()
            .map(|evaluation| evaluation.native_edits.len()),
        Some(2)
    );
    assert!(
        summary
            .execution
            .css_module_evaluation
            .as_ref()
            .and_then(|evaluation| evaluation.native_edit_output.as_deref())
            .is_some_and(|output| output.contains(".button { color: red; }"))
    );
    assert!(
        summary
            .execution
            .css_module_evaluation
            .as_ref()
            .is_some_and(
                |evaluation| evaluation
                    .native_replacements
                    .iter()
                    .any(|replacement| replacement.name == "$brand"
                        && replacement.text == "red"
                        && replacement.rendered_value.as_deref() == Some("red")
                        && replacement.abstract_value_kind == "exact")
            )
    );
    assert_eq!(
        summary
            .execution
            .css_module_evaluation
            .as_ref()
            .and_then(|evaluation| evaluation.oracle.as_ref())
            .map(|oracle| (
                oracle.native_replacement_count,
                oracle.native_replacement_legacy_reflection_count,
                oracle.native_replacement_legacy_unreflected_count,
            )),
        Some((1, 1, 0))
    );
}

#[test]
fn consumer_build_derives_static_scss_evaluator_context_for_mixin_includes() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.scss",
        "@mixin tone($color, $gap: 1px) { color: $color; margin: $gap; } .button { @include tone(red, 2px); }",
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
    assert!(summary.execution.output_css.contains("margin: 2px"));
    assert!(!summary.execution.output_css.contains("@mixin"));
    assert!(!summary.execution.output_css.contains("@include"));
}

#[test]
fn consumer_build_derives_static_scss_evaluator_context_for_mixin_function_values() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.scss",
        "@function double($value) { @return $value * 2; } @mixin tone($gap) { margin: double($gap); color: red; } .button { @include tone(2px); }",
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
    assert!(summary.execution.output_css.contains("margin: 4px"));
    assert!(summary.execution.output_css.contains("color: red"));
    assert!(!summary.execution.output_css.contains("@function"));
    assert!(!summary.execution.output_css.contains("@mixin"));
    assert!(!summary.execution.output_css.contains("@include"));
}

#[test]
fn consumer_build_derives_static_scss_evaluator_context_for_nested_mixin_includes() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.scss",
        "@mixin spacing($gap) { margin: $gap; } @mixin tone($gap, $color: red) { @include spacing($gap); color: $color; } .button { @include tone(2px, blue); }",
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
    assert!(summary.execution.output_css.contains("margin: 2px"));
    assert!(summary.execution.output_css.contains("color: blue"));
    assert!(!summary.execution.output_css.contains("@mixin"));
    assert!(!summary.execution.output_css.contains("@include"));
}

#[test]
fn consumer_build_derives_static_scss_evaluator_context_for_mixin_local_variables() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.scss",
        "@mixin tone($gap) { $space: $gap * 2; $color: if($space == 4px, blue, red); margin: $space; color: $color; } .button { @include tone(2px); }",
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
    assert!(summary.execution.output_css.contains("margin: 4px"));
    assert!(summary.execution.output_css.contains("color: blue"));
    assert!(!summary.execution.output_css.contains("$space"));
    assert!(!summary.execution.output_css.contains("$color"));
    assert!(!summary.execution.output_css.contains("@mixin"));
    assert!(!summary.execution.output_css.contains("@include"));
}

#[test]
fn consumer_build_executes_dynamic_mixin_local_variables_as_preserved_oracle_output() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.scss",
        "@mixin tone { $space: meta.inspect((a: b)); margin: $space; } .button { @include tone; }",
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
    assert!(summary.execution.output_css.contains("@mixin tone"));
    assert!(summary.execution.output_css.contains("$space"));
    assert!(
        summary
            .execution
            .output_css
            .contains("meta.inspect((a: b))")
    );
    assert!(summary.execution.output_css.contains("@include tone"));
    assert!(!summary.execution.output_css.contains("margin: (a: b)"));
}

#[test]
fn consumer_build_executes_recursive_nested_mixin_includes_as_preserved_oracle_output() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.scss",
        "@mixin a { @include b; } @mixin b { @include a; } .button { @include a; }",
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
    assert!(summary.execution.output_css.contains("@mixin a"));
    assert!(summary.execution.output_css.contains("@mixin b"));
    assert!(summary.execution.output_css.contains("@include a"));
    assert!(summary.execution.output_css.contains("@include b"));
}

#[test]
fn consumer_build_derives_static_scss_evaluator_context_for_not_conditions() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.scss",
        "$gap: if(not true, 1px, 2px); .button { margin: $gap; }",
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
    assert!(summary.execution.output_css.contains("margin: 2px"));
    assert!(!summary.execution.output_css.contains("$gap"));
}

#[test]
fn consumer_build_derives_static_scss_evaluator_context_for_boolean_conditions() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.scss",
        "$gap: if(false or true, 1px, 2px); .button { margin: $gap; }",
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
    assert!(summary.execution.output_css.contains("margin: 1px"));
    assert!(!summary.execution.output_css.contains("$gap"));
}

#[test]
fn consumer_build_derives_static_scss_evaluator_context_for_equality_conditions() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.scss",
        "$gap: if(1px == 2px, 1px, 2px); .button { margin: $gap; }",
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
    assert!(summary.execution.output_css.contains("margin: 2px"));
    assert!(!summary.execution.output_css.contains("$gap"));
}

#[test]
fn consumer_build_derives_static_scss_evaluator_context_for_inequality_conditions() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.scss",
        "$gap: if(1px != 2px, 1px, 2px); .button { margin: $gap; }",
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
    assert!(summary.execution.output_css.contains("margin: 1px"));
    assert!(!summary.execution.output_css.contains("$gap"));
}

#[test]
fn consumer_build_derives_static_scss_evaluator_context_for_numeric_ordering_conditions() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.scss",
        "$gap: if(3px > 2px, 1px, 2px); .button { margin: $gap; }",
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
    assert!(summary.execution.output_css.contains("margin: 1px"));
    assert!(!summary.execution.output_css.contains("$gap"));
}

#[test]
fn consumer_build_derives_static_scss_evaluator_context_for_zero_numeric_ordering_conditions() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.scss",
        "$gap: if(0px >= 0, 1px, 2px); .button { margin: $gap; }",
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
    assert!(summary.execution.output_css.contains("margin: 1px"));
    assert!(!summary.execution.output_css.contains("$gap"));
}

#[test]
fn consumer_build_derives_static_scss_evaluator_context_for_parenthesized_conditions() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.scss",
        "$gap: if((false or true), 1px, 2px); .button { margin: $gap; }",
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
    assert!(summary.execution.output_css.contains("margin: 1px"));
    assert!(!summary.execution.output_css.contains("$gap"));
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
            .outcomes
            .first()
            .map(|outcome| outcome.detail),
        Some(
            "applied explicit Less module evaluation native edit output from the evaluator boundary"
        )
    );
    assert_eq!(
        summary
            .execution
            .css_module_evaluation
            .as_ref()
            .map(|evaluation| evaluation.evaluator.as_str()),
        Some("omena-query-static-less-variable-evaluator")
    );
    assert_eq!(
        summary
            .execution
            .css_module_evaluation
            .as_ref()
            .and_then(|evaluation| evaluation.oracle.as_ref())
            .map(|oracle| (
                oracle.mode.as_str(),
                oracle.product_output_source.as_str(),
                oracle.divergence_count,
                oracle.all_legacy_declaration_values_preserved,
                oracle.native_replacement_count,
                oracle.native_replacement_legacy_reflection_count,
                oracle.native_replacement_legacy_unreflected_count,
                oracle.native_value_reference_count,
                oracle.native_resolved_value_count,
                oracle.native_raw_value_count,
                oracle.native_top_value_count,
            )),
        Some((
            "oracleOnly",
            "legacyEvaluatedCss",
            0,
            true,
            1,
            1,
            0,
            1,
            1,
            0,
            0
        ))
    );
}

#[test]
fn consumer_build_uses_native_less_numeric_and_rounding_evaluator_output() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "@sqrt: sqrt(4); @pow: pow(2, 3); @mod: mod(11px, 4px); @round: round(1.234px, 2); @ratio: percentage(.5); @ceil: ceil(1.2px); @floor: floor(1.8px); .button { sqrt: @sqrt; pow: @pow; mod: @mod; round: @round; width: @ratio; top: @ceil; bottom: @floor; }",
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
    assert!(summary.execution.output_css.contains("sqrt: 2"));
    assert!(summary.execution.output_css.contains("pow: 8"));
    assert!(summary.execution.output_css.contains("mod: 3px"));
    assert!(summary.execution.output_css.contains("round: 1.23px"));
    assert!(summary.execution.output_css.contains("width: 50%"));
    assert!(summary.execution.output_css.contains("top: 2px"));
    assert!(summary.execution.output_css.contains("bottom: 1px"));
    assert!(!summary.execution.output_css.contains("@sqrt"));
    assert!(!summary.execution.output_css.contains("@ratio"));
    assert_eq!(
        summary
            .execution
            .outcomes
            .iter()
            .find(|outcome| outcome.pass_id == "less-module-evaluate")
            .map(|outcome| outcome.detail),
        Some(
            "applied explicit Less module evaluation native edit output from the evaluator boundary"
        )
    );
    assert!(
        summary
            .execution
            .css_module_evaluation
            .as_ref()
            .and_then(|evaluation| evaluation.native_edit_output.as_deref())
            .is_some_and(|output| output.contains("round: 1.23px") && output.contains("width: 50%"))
    );
    assert_eq!(
        summary
            .execution
            .css_module_evaluation
            .as_ref()
            .and_then(|evaluation| evaluation.oracle.as_ref())
            .map(|oracle| (
                oracle.product_output_source.as_str(),
                oracle.divergence_count,
                oracle.all_legacy_declaration_values_preserved,
                oracle.native_value_reference_count,
                oracle.native_resolved_value_count,
                oracle.native_raw_value_count,
                oracle.native_top_value_count,
            )),
        Some(("legacyEvaluatedCss", 0, true, 7, 7, 0, 0))
    );
}

#[test]
fn consumer_build_uses_native_less_evaluation_after_import_inlining() -> Result<(), String> {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/tokens.less".to_string(),
            style_source: "@brand: red; .base { color: @brand; }".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/Button.module.less".to_string(),
            style_source: r#"@import "./tokens.less"; .button { color: @brand; }"#.to_string(),
        },
    ];

    let summary = execute_omena_query_consumer_build_style_sources(
        "/tmp/Button.module.less",
        sources.as_slice(),
        &[
            "import-inline".to_string(),
            "less-module-evaluate".to_string(),
            "print-css".to_string(),
        ],
        &[],
    )?;

    assert!(
        summary
            .execution
            .executed_pass_ids
            .contains(&"import-inline"),
        "{:?}",
        summary.execution.executed_pass_ids
    );
    assert!(
        summary
            .execution
            .executed_pass_ids
            .contains(&"less-module-evaluate"),
        "{:?}",
        summary.execution.executed_pass_ids
    );
    assert!(
        summary
            .execution
            .output_css
            .contains(".base { color: red; }"),
        "{}",
        summary.execution.output_css
    );
    assert!(
        summary
            .execution
            .output_css
            .contains(".button { color: red; }"),
        "{}",
        summary.execution.output_css
    );
    assert_eq!(
        summary
            .execution
            .outcomes
            .iter()
            .find(|outcome| outcome.pass_id == "less-module-evaluate")
            .map(|outcome| outcome.detail),
        Some(
            "applied explicit Less module evaluation native edit output from the evaluator boundary"
        )
    );
    assert!(
        summary
            .execution
            .css_module_evaluation
            .as_ref()
            .and_then(|evaluation| evaluation.native_edit_output.as_deref())
            .is_some_and(|output| output.contains(".button { color: red; }"))
    );
    Ok(())
}

#[test]
fn consumer_build_uses_native_scss_module_output_after_use_inlining() -> Result<(), String> {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/tokens.scss".to_string(),
            style_source: "$brand: red; .token { color: $brand; }".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/Button.module.scss".to_string(),
            style_source: r#"@use "./tokens"; .button { color: tokens.$brand; }"#.to_string(),
        },
    ];

    let summary = execute_omena_query_consumer_build_style_sources(
        "/tmp/Button.module.scss",
        sources.as_slice(),
        &["scss-module-evaluate".to_string(), "print-css".to_string()],
        &[],
    )?;

    assert!(
        summary
            .execution
            .executed_pass_ids
            .contains(&"scss-module-evaluate"),
        "{:?}",
        summary.execution.executed_pass_ids
    );
    assert!(
        summary
            .execution
            .output_css
            .contains(".token { color: red; }"),
        "{}",
        summary.execution.output_css
    );
    assert!(
        summary
            .execution
            .output_css
            .contains(".button { color: red; }"),
        "{}",
        summary.execution.output_css
    );
    assert!(!summary.execution.output_css.contains(r#"@use "./tokens""#));
    assert!(!summary.execution.output_css.contains("tokens.$brand"));
    assert!(!summary.execution.output_css.contains("$brand:"));
    Ok(())
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
fn consumer_build_expands_future_property_guarded_less_mixins() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        ".space() when (isnumber($margin)) { padding: $margin; } .button { .space(); margin: 2px; }",
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
    assert!(!summary.execution.output_css.contains(".space() when"));
    assert!(!summary.execution.output_css.contains(".space();"));
    assert!(!summary.execution.output_css.contains("_space"));
    assert!(summary.execution.output_css.contains("padding: 2px"));
    assert!(summary.execution.output_css.contains("margin: 2px"));
}

#[test]
fn consumer_build_expands_less_mixin_body_property_variables() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        ".space(@gap) { margin: @gap; padding: $margin; gap: $padding; } .button { .space(3px); }",
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
    assert!(!summary.execution.output_css.contains(".space(@gap"));
    assert!(!summary.execution.output_css.contains(".space(3px"));
    assert!(!summary.execution.output_css.contains("$margin"));
    assert!(!summary.execution.output_css.contains("$padding"));
    assert!(summary.execution.output_css.contains("margin: 3px"));
    assert!(summary.execution.output_css.contains("padding: 3px"));
    assert!(summary.execution.output_css.contains("gap: 3px"));
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_numeric_property_variables() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        ".card { margin: (1px + 2px); padding: $margin; }",
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
    assert!(summary.execution.output_css.contains("padding: 3px"));
    assert!(!summary.execution.output_css.contains("$margin"));
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
fn consumer_build_derives_static_less_evaluator_context_for_escaped_strings() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "@filter: ~\"alpha(opacity=50)\"; .card { filter: @filter; }",
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
    assert!(
        summary
            .execution
            .output_css
            .contains("filter: alpha(opacity=50)")
    );
    assert!(!summary.execution.output_css.contains("~\"alpha"));
    assert!(!summary.execution.output_css.contains("@filter:"));
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_property_name_interpolation() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "@prop: color; .card { @{prop}: red; }",
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
    assert!(!summary.execution.output_css.contains("@{prop}"));
    assert!(!summary.execution.output_css.contains("@prop:"));
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_simple_selector_interpolation() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "@name: card; .@{name} { color: red; }",
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
    assert!(!summary.execution.output_css.contains("@name: card"));
    assert!(!summary.execution.output_css.contains(".@{name}"));
    assert!(summary.execution.output_css.contains("color: red"));
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_chained_class_selector_interpolation() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "@prefix: card; @suffix: primary; .@{prefix}-@{suffix} { color: red; }",
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
    assert!(!summary.execution.output_css.contains("@prefix: card"));
    assert!(!summary.execution.output_css.contains("@suffix: primary"));
    assert!(
        !summary
            .execution
            .output_css
            .contains(".@{prefix}-@{suffix}")
    );
    assert!(summary.execution.output_css.contains("color: red"));
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_chained_id_selector_interpolation() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "@prefix: card; @suffix: primary; #@{prefix}-@{suffix} { color: red; }",
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
    assert!(!summary.execution.output_css.contains("@prefix: card"));
    assert!(!summary.execution.output_css.contains("@suffix: primary"));
    assert!(
        !summary
            .execution
            .output_css
            .contains("#@{prefix}-@{suffix}")
    );
    assert!(summary.execution.output_css.contains("#card-primary"));
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_chained_type_selector_interpolation() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "@prefix: button; @suffix: primary; @{prefix}-@{suffix} { color: red; }",
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
    assert!(!summary.execution.output_css.contains("@prefix: button"));
    assert!(!summary.execution.output_css.contains("@suffix: primary"));
    assert!(!summary.execution.output_css.contains("@{prefix}-@{suffix}"));
    assert!(summary.execution.output_css.contains("button-primary"));
}

#[test]
fn consumer_build_keeps_less_value_interpolation_planned_only() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "@color: red; .button { color: @{color}; }",
        &[
            "less-module-evaluate".to_string(),
            "css-modules-class-hashing".to_string(),
            "print-css".to_string(),
        ],
    );

    assert!(
        !summary
            .execution
            .executed_pass_ids
            .contains(&"less-module-evaluate")
    );
    assert!(
        summary
            .execution
            .planned_only_pass_ids
            .contains(&"less-module-evaluate")
    );
    assert!(summary.execution.output_css.contains("@color: red"));
    assert!(summary.execution.output_css.contains("@{color}"));
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_quoted_value_interpolation() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        r#"@path: "../img"; @theme: light; .button { color: "@{theme}"; background: url("@{path}/icon.png"); }"#,
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
    assert!(!summary.execution.output_css.contains("@{theme}"));
    assert!(!summary.execution.output_css.contains("@{path}"));
    assert!(!summary.execution.output_css.contains("@theme: light"));
    assert!(!summary.execution.output_css.contains(r#"@path: "../img""#));
    assert!(summary.execution.output_css.contains(r#"color: "light""#));
    assert!(
        summary
            .execution
            .output_css
            .contains(r#"background: url("../img/icon.png")"#)
    );
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_escaped_quoted_value_interpolation() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        r#"@theme: light; .button { color: ~"@{theme}"; }"#,
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
    assert!(!summary.execution.output_css.contains("@{theme}"));
    assert!(!summary.execution.output_css.contains("@theme: light"));
    assert!(!summary.execution.output_css.contains("~\"light\""));
    assert!(summary.execution.output_css.contains("color: light"));
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_variable_indirection() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "@name: color; @color: red; .button { color: @@name; }",
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
    assert!(!summary.execution.output_css.contains("@@name"));
    assert!(!summary.execution.output_css.contains("@name: color"));
    assert!(!summary.execution.output_css.contains("@color: red"));
    assert!(summary.execution.output_css.contains("color: red"));
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_variable_indirection_aliases() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "@name: color; @color: red; @alias: @@name; .button { color: @alias; }",
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
    assert!(!summary.execution.output_css.contains("@@name"));
    assert!(!summary.execution.output_css.contains("@alias"));
    assert!(!summary.execution.output_css.contains("@name: color"));
    assert!(!summary.execution.output_css.contains("@color: red"));
    assert!(summary.execution.output_css.contains("color: red"));
}

#[test]
fn consumer_build_executes_dynamic_less_escaped_strings_as_preserved_raw_output() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "@filter: ~\"@{name}\"; .card { filter: @filter; }",
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
    assert!(
        summary
            .execution
            .output_css
            .contains("filter: ~\"@{name}\"")
    );
    assert!(!summary.execution.output_css.contains("@filter:"));
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_mixin_calls() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "@brand: red; .tone(@color, @gap: 1px) { color: @color; margin: @gap; padding: @brand; } .button { .tone(blue, 2px); }",
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
    assert!(summary.execution.output_css.contains("color: blue"));
    assert!(summary.execution.output_css.contains("margin: 2px"));
    assert!(summary.execution.output_css.contains("padding: red"));
    assert!(!summary.execution.output_css.contains(".tone(@color"));
    assert!(!summary.execution.output_css.contains(".tone(blue"));
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_hash_mixin_calls() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "#tone(@color, @gap: 1px) { color: @color; margin: @gap; } .button { #tone(red, 2px); }",
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
    assert!(summary.execution.output_css.contains("margin: 2px"));
    assert!(!summary.execution.output_css.contains("#tone(@color"));
    assert!(!summary.execution.output_css.contains("#tone(red"));
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_mixin_declaration_accessors() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        ".tokens(@color, @gap: 1px) { @result: @color; width: @gap; } .button { color: .tokens(red)[@result]; margin: .tokens(red, 2px)[width]; }",
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
    assert!(summary.execution.output_css.contains("margin: 2px"));
    assert!(!summary.execution.output_css.contains(".tokens(@color"));
    assert!(
        !summary
            .execution
            .output_css
            .contains(".tokens(red)[@result]")
    );
    assert!(
        !summary
            .execution
            .output_css
            .contains(".tokens(red, 2px)[width]")
    );
}

#[test]
fn consumer_build_executes_unknown_less_mixin_accessor_members_as_preserved_oracle_output() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        ".tokens(@color) { @result: @color; } .button { color: .tokens(red)[@missing]; }",
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
    assert!(
        summary
            .execution
            .output_css
            .contains(".tokens(red)[@missing]")
    );
}

#[test]
fn consumer_build_executes_unknown_less_mixin_accessor_property_members_as_preserved_oracle_output()
{
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        ".tokens(@color) { result: @color; } .button { color: .tokens(red)[missing]; }",
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
    assert!(
        summary
            .execution
            .output_css
            .contains(".tokens(red)[missing]")
    );
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_namespace_mixin_access() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "#bundle() { .rounded(@radius) { border-radius: @radius; } } .button { #bundle > .rounded(2px); }",
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
    assert!(summary.execution.output_css.contains("border-radius: 2px"));
    assert!(!summary.execution.output_css.contains("#bundle()"));
    assert!(!summary.execution.output_css.contains("#bundle > .rounded"));
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_parameterized_namespace_mixin_access() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "#bundle(@color) { .tone() { color: @color; } } .button { #bundle(red) > .tone(); }",
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
    assert!(!summary.execution.output_css.contains("#bundle(@color"));
    assert!(
        !summary
            .execution
            .output_css
            .contains("#bundle(red) > .tone")
    );
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_guarded_namespace_mixin_access() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "#bundle() when (iscolor(red)) { .tone() { color: red; } } .button { #bundle > .tone(); }",
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
    assert!(!summary.execution.output_css.contains("#bundle()"));
    assert!(!summary.execution.output_css.contains("#bundle > .tone"));
}

#[test]
fn consumer_build_removes_false_guarded_less_namespace_mixin_access() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "#bundle() when (iscolor(1px)) { .tone() { color: red; } } .button { #bundle > .tone(); }",
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
    assert!(!summary.execution.output_css.contains("#bundle > .tone()"));
    assert!(!summary.execution.output_css.contains("when (iscolor(1px))"));
    assert!(!summary.execution.output_css.contains(".button { color:"));
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_detached_rulesets() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "@brand: red; @rules: { color: @brand; margin: 1px; }; .button { @rules(); }",
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
    assert!(summary.execution.output_css.contains("margin: 1px"));
    assert!(!summary.execution.output_css.contains("@rules:"));
    assert!(!summary.execution.output_css.contains("@rules();"));
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_detached_ruleset_accessors() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "@brand: red; @tokens: { primary: @brand; @gap: 2px; }; .button { color: @tokens[primary]; margin: @tokens[@gap]; }",
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
    assert!(summary.execution.output_css.contains("margin: 2px"));
    assert!(!summary.execution.output_css.contains("@tokens:"));
    assert!(!summary.execution.output_css.contains("@tokens[primary]"));
    assert!(!summary.execution.output_css.contains("@tokens[@gap]"));
}

#[test]
fn consumer_build_resolves_less_detached_ruleset_accessor_properties_from_call_scope() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "@tokens: { @gap: 2px; padding: @gap; gap: $padding; }; .button { padding: 4px; inset: @tokens[gap]; }",
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
    assert!(summary.execution.output_css.contains("padding: 4px"));
    assert!(summary.execution.output_css.contains("inset: 4px"));
    assert!(!summary.execution.output_css.contains("inset: 2px"));
    assert!(!summary.execution.output_css.contains("@tokens:"));
    assert!(!summary.execution.output_css.contains("@tokens[gap]"));
}

#[test]
fn consumer_build_preserves_less_detached_ruleset_accessor_missing_property_scope() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "@tokens: { margin: 2px; padding: $margin; }; .button { gap: @tokens[padding]; }",
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
    assert!(
        summary
            .execution
            .output_css
            .contains("@tokens: { margin: 2px; padding: $margin; };")
    );
    assert!(summary.execution.output_css.contains("@tokens[padding]"));
    assert!(!summary.execution.output_css.contains("gap: 2px"));
}

#[test]
fn consumer_build_executes_unknown_less_detached_ruleset_accessor_members_as_preserved_oracle_output()
 {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "@tokens: { primary: red; }; .button { color: @tokens[missing]; }",
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
    assert!(
        summary
            .execution
            .output_css
            .contains("@tokens: { primary: red; };")
    );
    assert!(summary.execution.output_css.contains("@tokens[missing]"));
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_detached_ruleset_mixin_calls() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        ".rounded() { border-radius: 2px; } @rules: { .rounded(); }; .button { @rules(); }",
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
    assert!(summary.execution.output_css.contains("border-radius: 2px"));
    assert!(!summary.execution.output_css.contains(".rounded()"));
    assert!(!summary.execution.output_css.contains("@rules:"));
    assert!(!summary.execution.output_css.contains("@rules();"));
}

#[test]
fn consumer_build_executes_unknown_detached_ruleset_mixin_calls_as_preserved_oracle_output() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "@rules: { .unknown(); }; .button { @rules(); }",
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
    assert!(
        summary
            .execution
            .output_css
            .contains("@rules: { .unknown(); };")
    );
    assert!(summary.execution.output_css.contains("@rules();"));
}

#[test]
fn consumer_build_executes_unbound_parameterized_less_namespace_mixin_access_as_preserved_oracle_output()
 {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "#bundle(@color) { .tone() { color: @color; } } .button { #bundle > .tone(); }",
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
    assert!(summary.execution.output_css.contains("#bundle > .tone()"));
    assert!(!summary.execution.output_css.contains(".button { color:"));
}

#[test]
fn consumer_build_preserves_less_namespace_local_detached_rulesets_as_raw() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "#bundle() { @tokens: { color: red; }; .tone() { color: @tokens[color]; } } .button { #bundle > .tone(); }",
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
    assert!(
        summary
            .execution
            .output_css
            .contains("@tokens: { color: red; };")
    );
    assert!(summary.execution.output_css.contains("@tokens[color]"));
    assert!(
        !summary
            .execution
            .output_css
            .contains(".button { color: red")
    );
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_escaped_string_mixin_arguments() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        ".legacy(@value) { filter: @value; } .card { .legacy(~\"alpha(opacity=50)\"); }",
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
        summary
            .execution
            .output_css
            .contains("filter: alpha(opacity=50)")
    );
    assert!(!summary.execution.output_css.contains(".legacy(@value"));
    assert!(!summary.execution.output_css.contains(".legacy(~\"alpha"));
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_semicolon_mixin_calls() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        ".shadow(@value; @color: red) { box-shadow: @value; color: @color; } .button { .shadow(1px, 2px, 3px; blue); }",
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
        summary
            .execution
            .output_css
            .contains("box-shadow: 1px, 2px, 3px")
    );
    assert!(summary.execution.output_css.contains("color: blue"));
    assert!(!summary.execution.output_css.contains(".shadow(@value"));
    assert!(!summary.execution.output_css.contains(".shadow(1px"));
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_variadic_mixin_arguments() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        ".shadow(@color; @rest...) { color: @color; box-shadow: @rest; trace: @arguments; } .button { .shadow(red; 1px, 2px, 3px); }",
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
    assert!(
        summary
            .execution
            .output_css
            .contains("box-shadow: 1px, 2px, 3px")
    );
    assert!(
        summary
            .execution
            .output_css
            .contains("trace: red, 1px, 2px, 3px")
    );
    assert!(!summary.execution.output_css.contains(".shadow(@color"));
    assert!(!summary.execution.output_css.contains(".shadow(red"));
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_literal_pattern_mixins() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        ".tone(dark, @color) { color: @color; background: black; } .tone(light, @color) { color: @color; background: white; } .button { .tone(dark, red); }",
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
    assert!(summary.execution.output_css.contains("background: black"));
    assert!(!summary.execution.output_css.contains("background: white"));
    assert!(!summary.execution.output_css.contains(".tone(dark"));
    assert!(!summary.execution.output_css.contains(".tone(light"));
}

#[test]
fn consumer_build_executes_unmatched_literal_pattern_mixins_as_preserved_oracle_output() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        ".tone(dark, @color) { color: @color; background: black; } .button { .tone(light, red); }",
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
    assert!(summary.execution.output_css.contains(".tone(light, red)"));
    assert!(
        !summary
            .execution
            .output_css
            .contains(".button { color: red")
    );
}

#[test]
fn consumer_build_does_not_expand_variadic_tokens_in_less_calls() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "@gap: 1px; .space(@value) { margin: @value; } .button { .space(@gap...); }",
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
    assert!(summary.execution.output_css.contains(".space(1px...)"));
    assert!(!summary.execution.output_css.contains("margin: 1px"));
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_important_mixin_calls() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        ".tone(@color, @gap: 1px) { color: @color; margin: @gap; } .button { .tone(red, 2px) !important; }",
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
        summary
            .execution
            .output_css
            .contains("color: red !important")
    );
    assert!(
        summary
            .execution
            .output_css
            .contains("margin: 2px !important")
    );
    assert!(!summary.execution.output_css.contains(".tone(@color"));
    assert!(!summary.execution.output_css.contains(".tone(red"));
}

#[test]
fn consumer_build_executes_unknown_less_mixin_call_suffixes_as_preserved_oracle_output() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        ".tone(@color) { color: @color; } .button { .tone(red) !default; }",
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
    assert!(summary.execution.output_css.contains(".tone(red) !default"));
    assert!(
        !summary
            .execution
            .output_css
            .contains(".button { color: red")
    );
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_named_and_default_mixin_arguments() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        ".tone(@color: red, @gap: 1px, @double: 4px) { color: @color; margin: @gap; padding: @double; } .button { .tone(@gap: 2px, @color: blue); }",
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
    assert!(summary.execution.output_css.contains("margin: 2px"));
    assert!(summary.execution.output_css.contains("padding: 4px"));
    assert!(!summary.execution.output_css.contains(".tone(@color"));
    assert!(!summary.execution.output_css.contains(".tone(@gap"));
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_nested_mixin_calls() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        ".spacing(@gap) { margin: @gap; } .tone(@gap, @color: red) { .spacing(@gap); color: @color; } .button { .tone(2px, blue); }",
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
    assert!(summary.execution.output_css.contains("margin: 2px"));
    assert!(summary.execution.output_css.contains("color: blue"));
    assert!(!summary.execution.output_css.contains(".spacing(@gap"));
    assert!(!summary.execution.output_css.contains(".tone(@gap"));
    assert!(!summary.execution.output_css.contains(".spacing(2px"));
    assert!(!summary.execution.output_css.contains(".tone(2px"));
}

#[test]
fn consumer_build_preserves_recursive_less_nested_mixin_calls_through_oracle_evaluator() {
    let source = ".again() { .again(); } .button { .again(); }";
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        source,
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
    assert!(summary.execution.output_css.contains(".again()"));
    assert!(summary.execution.output_css.contains(".again();"));
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_static_guarded_mixin_calls() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        ".tone(@color) when (iscolor(@color)) { color: @color; } .button { .tone(red); }",
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
    assert!(!summary.execution.output_css.contains(".tone(@color"));
    assert!(!summary.execution.output_css.contains(".tone(red)"));
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_ruleset_guarded_mixin_arguments() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        ".apply(@block) when (isruleset(@block)) { @block(); } @rules: { color: red; margin: 1px; }; .button { .apply(@rules); }",
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
    assert!(summary.execution.output_css.contains("margin: 1px"));
    assert!(!summary.execution.output_css.contains(".apply(@block"));
    assert!(!summary.execution.output_css.contains(".apply(@rules"));
    assert!(!summary.execution.output_css.contains("@rules:"));
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_numeric_guarded_mixin_calls() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        ".space(@gap) when (isnumber(@gap)) { margin: @gap; } .button { .space(2px); }",
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
    assert!(summary.execution.output_css.contains("margin: 2px"));
    assert!(!summary.execution.output_css.contains(".space(@gap"));
    assert!(!summary.execution.output_css.contains(".space(2px)"));
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_type_guarded_mixin_calls() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        r#".space(@gap) when (ispixel(@gap)) { margin: @gap; }
.ratio(@value) when (ispercentage(@value)) { width: @value; }
.font(@family) when (isstring(@family)) { font-family: @family; }
.display(@value) when (iskeyword(@value)) { display: @value; }
.asset(@value) when (isurl(@value)) { background-image: @value; }
.unit(@gap) when (isunit(@gap, "rem")) { padding: @gap; }
.button { .space(2px); .ratio(50%); .font("Roboto"); .display(block); .asset(url("./icon.svg")); .unit(1rem); }"#,
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
    assert!(summary.execution.output_css.contains("margin: 2px"));
    assert!(summary.execution.output_css.contains("width: 50%"));
    assert!(
        summary
            .execution
            .output_css
            .contains(r#"font-family: "Roboto""#)
    );
    assert!(summary.execution.output_css.contains("display: block"));
    assert!(summary.execution.output_css.contains("padding: 1rem"));
    assert!(
        summary
            .execution
            .output_css
            .contains(r#"background-image: url("./icon.svg")"#)
    );
    assert!(!summary.execution.output_css.contains(".space(2px)"));
    assert!(!summary.execution.output_css.contains(".asset(url"));
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_comparison_guarded_mixin_calls() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        r#".space(@gap) when (@gap > 1px) { margin: @gap; }
.tone(@color) when (@color = red) { color: @color; }
.combo(@gap, @color) when (@gap >= 2px) and (iscolor(@color)) { padding: @gap; border-color: @color; }
.inverse(@gap) when not (@gap < 2px) { inset: @gap; }
.fallback(@name) when (@name = primary), (@name = secondary) { content: @name; }
.button { .space(2px); .tone(red); .combo(2px, blue); .inverse(2px); .fallback(secondary); }"#,
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
    assert!(summary.execution.output_css.contains("margin: 2px"));
    assert!(summary.execution.output_css.contains("color: red"));
    assert!(summary.execution.output_css.contains("padding: 2px"));
    assert!(summary.execution.output_css.contains("border-color: blue"));
    assert!(summary.execution.output_css.contains("inset: 2px"));
    assert!(summary.execution.output_css.contains("content: secondary"));
    assert!(!summary.execution.output_css.contains(".space(2px)"));
    assert!(
        !summary
            .execution
            .output_css
            .contains(".fallback(secondary)")
    );
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_multiple_matching_guarded_mixins() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        r#".tone(@color) when (@color = blue) { outline-color: blue; }
.tone(@color) when (@color = red) { color: @color; }
.tone(@color) when (iscolor(@color)) { border-color: @color; }
.button { .tone(red); }"#,
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
    assert!(!summary.execution.output_css.contains("outline-color: blue"));
    assert!(summary.execution.output_css.contains("color: red"));
    assert!(summary.execution.output_css.contains("border-color: red"));
    assert!(!summary.execution.output_css.contains(".tone(@color"));
    assert!(!summary.execution.output_css.contains(".tone(red)"));
}

#[test]
fn consumer_build_derives_static_less_evaluator_context_for_default_guarded_mixins() {
    let red_summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        r#".tone(@color) when (@color = red) { color: @color; }
.tone(@color) when (default()) and (iscolor(@color)) { color: gray; }
.button { .tone(red); }"#,
        &[
            "less-module-evaluate".to_string(),
            "css-modules-class-hashing".to_string(),
            "print-css".to_string(),
        ],
    );

    assert!(
        red_summary
            .execution
            .executed_pass_ids
            .contains(&"less-module-evaluate")
    );
    assert!(red_summary.execution.output_css.contains("color: red"));
    assert!(!red_summary.execution.output_css.contains("color: gray"));
    assert!(!red_summary.execution.output_css.contains(".tone(@color"));
    assert!(!red_summary.execution.output_css.contains(".tone(red)"));

    let blue_summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        r#".tone(@color) when (@color = red) { color: @color; }
.tone(@color) when (default()) and (iscolor(@color)) { color: gray; }
.button { .tone(blue); }"#,
        &[
            "less-module-evaluate".to_string(),
            "css-modules-class-hashing".to_string(),
            "print-css".to_string(),
        ],
    );

    assert!(
        blue_summary
            .execution
            .executed_pass_ids
            .contains(&"less-module-evaluate")
    );
    assert!(blue_summary.execution.output_css.contains("color: gray"));
    assert!(!blue_summary.execution.output_css.contains("color: blue"));
    assert!(!blue_summary.execution.output_css.contains(".tone(@color"));
    assert!(!blue_summary.execution.output_css.contains(".tone(blue)"));
}

#[test]
fn consumer_build_removes_false_guarded_less_mixins() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        ".tone(@value) when (iscolor(@value)) { color: @value; } .button { .tone(1px); }",
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
    assert!(!summary.execution.output_css.contains(".tone(1px)"));
    assert!(!summary.execution.output_css.contains(".tone(@value)"));
    assert!(!summary.execution.output_css.contains("color: 1px"));
}

#[test]
fn consumer_build_removes_false_comparison_guarded_less_mixins() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        ".space(@gap) when (@gap > 2px) { margin: @gap; } .button { .space(1px); }",
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
    assert!(!summary.execution.output_css.contains(".space(1px)"));
    assert!(!summary.execution.output_css.contains(".space(@gap)"));
    assert!(!summary.execution.output_css.contains("margin: 1px"));
}

#[test]
fn consumer_build_removes_false_unit_guarded_less_mixins() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        ".space(@gap) when (ispixel(@gap)) { margin: @gap; } .button { .space(2em); }",
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
    assert!(!summary.execution.output_css.contains(".space(2em)"));
    assert!(!summary.execution.output_css.contains(".space(@gap)"));
    assert!(!summary.execution.output_css.contains("margin: 2em"));
}

#[test]
fn consumer_build_removes_false_isunit_guarded_less_mixins() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        r#".space(@gap) when (isunit(@gap, "px")) { margin: @gap; } .button { .space(2em); }"#,
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
    assert!(!summary.execution.output_css.contains(".space(2em)"));
    assert!(!summary.execution.output_css.contains(".space(@gap)"));
    assert!(!summary.execution.output_css.contains("margin: 2em"));
}

#[test]
fn consumer_build_removes_false_type_predicate_guarded_less_mixins() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        r#".number(@value) when (isnumber(@value)) { margin: @value; }
.ratio(@value) when (ispercentage(@value)) { width: @value; }
.font(@value) when (isstring(@value)) { font-family: @value; }
.display(@value) when (iskeyword(@value)) { display: @value; }
.asset(@value) when (isurl(@value)) { background-image: @value; }
.em(@value) when (isem(@value)) { letter-spacing: @value; }
.button { .number(red); .ratio(2px); .font(block); .display("Roboto"); .asset(red); .em(1rem); }"#,
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
    for snippet in [
        ".number(red)",
        ".ratio(2px)",
        ".font(block)",
        r#".display("Roboto")"#,
        ".asset(red)",
        ".em(1rem)",
        ".number(@value)",
        ".ratio(@value)",
        ".font(@value)",
        ".display(@value)",
        ".asset(@value)",
        ".em(@value)",
        "margin: red",
        "width: 2px",
        "font-family: block",
        r#"display: "Roboto""#,
        "background-image: red",
        "letter-spacing: 1rem",
    ] {
        assert!(
            !summary.execution.output_css.contains(snippet),
            "false type-predicate guard output retained `{snippet}` in {}",
            summary.execution.output_css
        );
    }
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
fn consumer_build_preserves_conflicting_sass_module_configuration_boundary() -> Result<(), String> {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/tokens.scss".to_string(),
            style_source: "$brand: blue !default; .token { color: $brand; }".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/theme-red.scss".to_string(),
            style_source: r#"@forward "./tokens" with ($brand: red);"#.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/theme-blue.scss".to_string(),
            style_source: r#"@forward "./tokens" with ($brand: blue);"#.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/App.module.scss".to_string(),
            style_source: r#"@use "./theme-red" as redTheme;
@use "./theme-blue" as blueTheme;
.button { color: redTheme.$brand; background: blueTheme.$brand; }"#
                .to_string(),
        },
    ];

    let summary = execute_omena_query_consumer_build_style_sources(
        "/tmp/App.module.scss",
        sources.as_slice(),
        &["scss-module-evaluate".to_string(), "print-css".to_string()],
        &[],
    )
    .map_err(|error| format!("multi-source SCSS build should return a summary: {error}"))?;

    assert!(
        summary.execution.output_css.contains("color: red"),
        "{}",
        summary.execution.output_css
    );
    assert!(
        !summary.execution.output_css.contains("color: blue"),
        "{}",
        summary.execution.output_css
    );
    assert!(
        summary
            .execution
            .output_css
            .contains(r#"@use "./theme-blue" as blueTheme"#),
        "{}",
        summary.execution.output_css
    );
    Ok(())
}

#[test]
fn consumer_build_preserves_non_default_sass_module_configuration_boundary() -> Result<(), String> {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/tokens.scss".to_string(),
            style_source: "$brand: blue; .token { color: $brand; }".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/App.module.scss".to_string(),
            style_source:
                r#"@use "./tokens" as tokens with ($brand: red); .button { color: tokens.$brand; }"#
                    .to_string(),
        },
    ];

    let summary = execute_omena_query_consumer_build_style_sources(
        "/tmp/App.module.scss",
        sources.as_slice(),
        &["scss-module-evaluate".to_string(), "print-css".to_string()],
        &[],
    )
    .map_err(|error| format!("multi-source SCSS build should return a summary: {error}"))?;

    assert!(
        summary
            .execution
            .output_css
            .contains(r#"@use "./tokens" as tokens with ($brand: red)"#),
        "{}",
        summary.execution.output_css
    );
    assert!(summary.execution.output_css.contains("tokens.$brand"));
    assert!(
        !summary
            .execution
            .output_css
            .contains(".token { color: blue; }")
    );
    Ok(())
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
fn consumer_build_derives_static_scss_evaluator_context_for_opacity_colors() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.scss",
        "$tone: transparentize(red, .25); $hue: hue(#808000); $light: lighten(#808000, 10%); .button { color: $tone; --hue: $hue; border-color: $light; }",
        &[
            "scss-module-evaluate".to_string(),
            "css-modules-class-hashing".to_string(),
            "print-css".to_string(),
        ],
    );

    assert!(
        summary
            .execution
            .output_css
            .contains("color: rgba(255, 0, 0, 0.75)"),
        "{}",
        summary.execution.output_css
    );
    assert!(
        summary.execution.output_css.contains("--hue: 60deg"),
        "{}",
        summary.execution.output_css
    );
    assert!(
        summary
            .execution
            .output_css
            .contains("border-color: #b3b300"),
        "{}",
        summary.execution.output_css
    );
    assert!(!summary.execution.output_css.contains("$tone:"));
    assert!(!summary.execution.output_css.contains("$hue:"));
    assert!(!summary.execution.output_css.contains("$light:"));
}

#[test]
fn consumer_build_preserves_forward_scss_composite_values_through_oracle_evaluator() {
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
            .executed_pass_ids
            .contains(&"scss-module-evaluate")
    );
    assert!(
        !summary
            .execution
            .planned_only_pass_ids
            .contains(&"scss-module-evaluate")
    );
    assert!(summary.execution.output_css.contains("$border:"));
    assert!(summary.execution.output_css.contains("$brand:"));
}

#[test]
fn consumer_build_preserves_dynamic_scss_function_calls_through_oracle_evaluator() {
    let source = "@function tone($enabled) { @if $enabled { @return red; } @else { @return blue; } } .button { color: tone(var(--enabled)); }";
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.scss",
        source,
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
    assert!(summary.execution.output_css.contains("@function tone"));
    assert!(
        summary
            .execution
            .output_css
            .contains("color: tone(var(--enabled))")
    );
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
