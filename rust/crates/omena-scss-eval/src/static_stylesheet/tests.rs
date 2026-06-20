use super::*;
use crate::value_eval::static_scss_bang_usage_is_comparison_only;
use std::fmt::Write as _;

#[test]
fn static_stylesheet_oracle_corpus_reports_native_product_output_with_legacy_oracle() {
    let report = summarize_static_stylesheet_oracle_corpus();

    assert_eq!(
        report.product,
        "omena-scss-eval.static-stylesheet-oracle-corpus"
    );
    assert_eq!(report.mode, "oracleOnly");
    assert_eq!(report.value_type, "AbstractCssValueV0");
    assert_eq!(report.product_output_source, "nativeEditOutput");
    assert_eq!(
        report.legacy_output_retained_as_oracle_count,
        report.evaluated_fixture_count
    );
    assert_eq!(report.legacy_output_consumed_until_cutover_count, 0);
    assert!(report.all_legacy_outputs_retained_as_oracle);
    assert_eq!(report.fixture_count, 135);
    assert_eq!(report.scss_fixture_count, 46);
    assert_eq!(report.sass_fixture_count, 40);
    assert_eq!(report.less_fixture_count, 49);
    assert_eq!(report.evaluated_fixture_count, report.fixture_count);
    assert_eq!(report.missing_evaluation_count, 0);
    assert_eq!(report.divergence_count, 0);
    assert!(report.native_replacement_count > 0);
    assert!(report.native_replacement_legacy_reflection_count > 0);
    assert_eq!(
        report.native_replacement_legacy_reflection_count
            + report.native_replacement_legacy_unreflected_count,
        report.native_replacement_count
    );
    assert!(report.native_edit_count > 0);
    assert!(report.native_value_edit_count > 0);
    assert!(report.native_structural_edit_count > 0);
    assert_eq!(
        report.native_value_edit_count + report.native_structural_edit_count,
        report.native_edit_count
    );
    assert_eq!(
        report.native_edit_output_match_count,
        report.evaluated_fixture_count
    );
    assert!(report.native_value_reference_count > 0);
    assert!(report.native_resolved_value_count > 0);
    assert!(report.native_raw_value_count > 0);
    assert!(report.native_top_value_count > 0);
    assert!(report.native_cycle_value_count > 0);
    assert!(report.native_fuel_exhausted_value_count > 0);
    assert!(report.native_unresolved_reference_value_count > 0);
    assert!(report.native_unsupported_dynamic_value_count > 0);
    assert!(report.all_legacy_declaration_values_preserved);
    assert!(report.all_native_edit_outputs_match_evaluated_css);
    assert!(report.native_product_output_corpus_ready);
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "sass.static-map-list-builtins"
            && fixture.dialect == "sass"
            && fixture.evaluation_available
            && fixture.native_edit_output_matches_evaluated_css
            && fixture.divergence_count == 0
    }));
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "scss.static-map-list-builtins"
            && fixture.dialect == "scss"
            && fixture.evaluation_available
            && fixture.native_edit_output_matches_evaluated_css
            && fixture.divergence_count == 0
    }));
    for id in [
        "sass.static-default-function-arguments",
        "sass.static-default-argument-prior-parameter",
        "sass.static-named-default-argument-prior-parameter",
        "scss.static-default-function-arguments",
        "scss.static-default-argument-prior-parameter",
        "scss.static-named-default-argument-prior-parameter",
        "sass.static-named-function-arguments",
        "sass.static-named-argument-default-tail",
        "scss.static-named-function-arguments",
        "scss.static-named-argument-default-tail",
        "sass.static-hyphen-underscore-function-reference",
        "sass.static-hyphen-underscore-named-argument",
        "scss.static-hyphen-underscore-function-reference",
        "scss.static-hyphen-underscore-named-argument",
        "sass.static-named-mixin-arguments",
        "sass.static-named-mixin-default-tail",
        "sass.static-mixin-default-argument-prior-parameter",
        "sass.static-named-mixin-default-argument-prior-parameter",
        "sass.static-mixin-content-block",
        "sass.static-mixin-content-arguments",
        "sass.static-mixin-content-expression-arguments",
        "sass.static-mixin-content-nested-include",
        "sass.static-nested-mixin-include",
        "sass.static-hyphen-underscore-mixin-include",
        "scss.static-named-mixin-arguments",
        "scss.static-named-mixin-default-tail",
        "scss.static-mixin-default-argument-prior-parameter",
        "scss.static-named-mixin-default-argument-prior-parameter",
        "scss.static-mixin-content-block",
        "scss.static-mixin-content-arguments",
        "scss.static-mixin-content-expression-arguments",
        "scss.static-mixin-content-nested-include",
        "scss.static-nested-mixin-include",
        "scss.static-hyphen-underscore-mixin-include",
    ] {
        assert!(
            report.fixtures.iter().any(|fixture| {
                fixture.id == id
                    && fixture.evaluation_available
                    && fixture.native_edit_output_matches_evaluated_css
                    && fixture.divergence_count == 0
            }),
            "missing evaluated default-argument oracle fixture {id}"
        );
    }
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "less.extended-numeric-builtins"
            && fixture.dialect == "less"
            && fixture.evaluation_available
            && fixture.native_edit_output_matches_evaluated_css
            && fixture.divergence_count == 0
    }));
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "less.percentage-rounding-builtins"
            && fixture.dialect == "less"
            && fixture.evaluation_available
            && fixture.native_edit_output_matches_evaluated_css
            && fixture.divergence_count == 0
    }));
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "sass.variable-basic"
            && fixture.dialect == "sass"
            && fixture.evaluation_available
            && fixture.native_edit_output_matches_evaluated_css
            && fixture.divergence_count == 0
    }));
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "sass.static-function-return"
            && fixture.dialect == "sass"
            && fixture.evaluation_available
            && fixture.native_replacement_count == 1
            && fixture.native_edit_output_matches_evaluated_css
            && fixture.divergence_count == 0
    }));
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "sass.static-mixin-include"
            && fixture.dialect == "sass"
            && fixture.evaluation_available
            && fixture.native_structural_edit_count == 2
            && fixture.native_edit_output_matches_evaluated_css
            && fixture.divergence_count == 0
    }));
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "sass.static-mixin-if"
            && fixture.dialect == "sass"
            && fixture.evaluation_available
            && fixture.native_structural_edit_count == 2
            && fixture.native_edit_output_matches_evaluated_css
            && fixture.divergence_count == 0
    }));
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "sass.static-mixin-for"
            && fixture.dialect == "sass"
            && fixture.evaluation_available
            && fixture.native_structural_edit_count == 2
            && fixture.native_edit_output_matches_evaluated_css
            && fixture.divergence_count == 0
    }));
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "sass.static-mixin-each"
            && fixture.dialect == "sass"
            && fixture.evaluation_available
            && fixture.native_structural_edit_count == 2
            && fixture.native_edit_output_matches_evaluated_css
            && fixture.divergence_count == 0
    }));
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "sass.static-mixin-while"
            && fixture.dialect == "sass"
            && fixture.evaluation_available
            && fixture.native_structural_edit_count == 2
            && fixture.native_edit_output_matches_evaluated_css
            && fixture.divergence_count == 0
    }));
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "scss.static-mixin-if"
            && fixture.dialect == "scss"
            && fixture.evaluation_available
            && fixture.native_structural_edit_count == 2
            && fixture.native_edit_output_matches_evaluated_css
            && fixture.divergence_count == 0
    }));
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "scss.static-mixin-for"
            && fixture.dialect == "scss"
            && fixture.evaluation_available
            && fixture.native_structural_edit_count == 2
            && fixture.native_edit_output_matches_evaluated_css
            && fixture.divergence_count == 0
    }));
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "scss.static-mixin-each"
            && fixture.dialect == "scss"
            && fixture.evaluation_available
            && fixture.native_structural_edit_count == 2
            && fixture.native_edit_output_matches_evaluated_css
            && fixture.divergence_count == 0
    }));
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "scss.static-mixin-while"
            && fixture.dialect == "scss"
            && fixture.evaluation_available
            && fixture.native_structural_edit_count == 2
            && fixture.native_edit_output_matches_evaluated_css
            && fixture.divergence_count == 0
    }));
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "scss.static-top-level-if"
            && fixture.dialect == "scss"
            && fixture.evaluation_available
            && fixture.native_structural_edit_count == 2
            && fixture.native_edit_output_matches_evaluated_css
            && fixture.divergence_count == 0
    }));
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "scss.dynamic-top-level-if"
            && fixture.dialect == "scss"
            && fixture.evaluation_available
            && fixture.native_edit_count == 0
            && fixture.native_raw_value_count > 0
            && fixture.native_edit_output_matches_evaluated_css
            && fixture.divergence_count == 0
    }));
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "scss.static-top-level-if-variable"
            && fixture.dialect == "scss"
            && fixture.evaluation_available
            && fixture.native_replacement_count == 1
            && fixture.native_replacement_legacy_reflection_count == 1
            && fixture.native_structural_edit_count == 3
            && fixture.native_edit_output_matches_evaluated_css
            && fixture.divergence_count == 0
    }));
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "scss.static-top-level-if-function"
            && fixture.dialect == "scss"
            && fixture.evaluation_available
            && fixture.native_replacement_count == 1
            && fixture.native_replacement_legacy_reflection_count == 1
            && fixture.native_structural_edit_count == 3
            && fixture.native_edit_output_matches_evaluated_css
            && fixture.divergence_count == 0
    }));
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "scss.static-top-level-for"
            && fixture.dialect == "scss"
            && fixture.evaluation_available
            && fixture.native_replacement_count == 3
            && fixture.native_replacement_legacy_reflection_count == 3
            && fixture.native_structural_edit_count == 1
            && fixture.native_edit_output_matches_evaluated_css
            && fixture.divergence_count == 0
    }));
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "scss.static-top-level-each"
            && fixture.dialect == "scss"
            && fixture.evaluation_available
            && fixture.native_replacement_count == 2
            && fixture.native_replacement_legacy_reflection_count == 2
            && fixture.native_structural_edit_count == 1
            && fixture.native_edit_output_matches_evaluated_css
            && fixture.divergence_count == 0
    }));
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "scss.static-top-level-while"
            && fixture.dialect == "scss"
            && fixture.evaluation_available
            && fixture.native_replacement_count == 3
            && fixture.native_replacement_legacy_reflection_count == 3
            && fixture.native_structural_edit_count == 2
            && fixture.native_edit_output_matches_evaluated_css
            && fixture.divergence_count == 0
    }));
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "sass.static-top-level-for"
            && fixture.dialect == "sass"
            && fixture.evaluation_available
            && fixture.native_replacement_count == 3
            && fixture.native_replacement_legacy_reflection_count == 3
            && fixture.native_structural_edit_count == 1
            && fixture.native_edit_output_matches_evaluated_css
            && fixture.divergence_count == 0
    }));
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "sass.static-top-level-each"
            && fixture.dialect == "sass"
            && fixture.evaluation_available
            && fixture.native_replacement_count == 2
            && fixture.native_replacement_legacy_reflection_count == 2
            && fixture.native_structural_edit_count == 1
            && fixture.native_edit_output_matches_evaluated_css
            && fixture.divergence_count == 0
    }));
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "sass.static-top-level-while"
            && fixture.dialect == "sass"
            && fixture.evaluation_available
            && fixture.native_replacement_count == 3
            && fixture.native_replacement_legacy_reflection_count == 3
            && fixture.native_structural_edit_count == 2
            && fixture.native_edit_output_matches_evaluated_css
            && fixture.divergence_count == 0
    }));
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "sass.static-top-level-if"
            && fixture.dialect == "sass"
            && fixture.evaluation_available
            && fixture.native_replacement_count == 0
            && fixture.native_replacement_legacy_reflection_count == 0
            && fixture.native_structural_edit_count == 2
            && fixture.native_edit_output_matches_evaluated_css
            && fixture.divergence_count == 0
    }));
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "sass.static-top-level-if-variable"
            && fixture.dialect == "sass"
            && fixture.evaluation_available
            && fixture.native_replacement_count == 1
            && fixture.native_replacement_legacy_reflection_count == 1
            && fixture.native_structural_edit_count == 3
            && fixture.native_edit_output_matches_evaluated_css
            && fixture.divergence_count == 0
    }));
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "sass.static-top-level-if-function"
            && fixture.dialect == "sass"
            && fixture.evaluation_available
            && fixture.native_replacement_count == 1
            && fixture.native_replacement_legacy_reflection_count == 1
            && fixture.native_structural_edit_count == 3
            && fixture.native_edit_output_matches_evaluated_css
            && fixture.divergence_count == 0
    }));
    for id in [
        "sass.static-if-return",
        "sass.static-for-return",
        "sass.static-while-return",
        "sass.static-while-expression-step",
        "sass.static-each-return",
        "sass.static-each-tuple-function-source-return",
    ] {
        assert!(
            report.fixtures.iter().any(|fixture| {
                fixture.id == id
                    && fixture.dialect == "sass"
                    && fixture.evaluation_available
                    && fixture.native_replacement_count == 1
                    && fixture.native_edit_output_matches_evaluated_css
                    && fixture.divergence_count == 0
            }),
            "missing evaluated Sass control-flow oracle fixture {id}"
        );
    }
    assert!(report.fixtures.iter().any(|fixture| fixture.id
        == "scss.indirect-recursive-function-return"
        && fixture.native_top_value_count == 1
        && fixture.native_cycle_value_count == 1));
    assert!(
        report
            .fixtures
            .iter()
            .any(|fixture| fixture.id == "scss.dynamic-function-return"
                && fixture.native_top_value_count == 1
                && fixture.native_unsupported_dynamic_value_count == 1)
    );
    assert!(report.fixtures.iter().any(|fixture| {
        fixture.id == "scss.unresolved-forward-composite"
            && fixture.native_top_value_count == 1
            && fixture.native_unresolved_reference_value_count == 1
    }));
    assert!(
        report
            .fixtures
            .iter()
            .any(|fixture| fixture.id == "scss.recursive-function-return"
                && fixture.native_top_value_count == 1
                && fixture.native_cycle_value_count == 1)
    );
    assert!(
        report
            .fixtures
            .iter()
            .any(|fixture| fixture.id == "less.fuel-exhausted-variable-chain"
                && fixture.native_top_value_count == 1
                && fixture.native_fuel_exhausted_value_count == 1)
    );
    assert!(
        report
            .fixtures
            .iter()
            .all(|fixture| fixture.legacy_output_retained_as_oracle
                && !fixture.legacy_output_consumed_until_cutover)
    );
}

#[test]
fn static_scss_evaluation_emits_abstract_replacement_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "$gap: 0px; .button { margin: $gap; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(
        report.evaluator,
        "omena-query-static-scss-variable-evaluator"
    );
    assert_eq!(report.product_output_source, "nativeEditOutput");
    assert!(report.legacy_output_retained_as_oracle);
    assert!(!report.legacy_output_consumed_until_cutover);
    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.native_replacement_legacy_reflection_count, 1);
    assert_eq!(report.native_replacement_legacy_unreflected_count, 0);
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert_eq!(report.resolved_replacements[0].text, "0px");
    assert_eq!(report.native_edit_count, 2);
    assert_eq!(report.native_value_edit_count, 1);
    assert_eq!(report.native_structural_edit_count, 1);
    assert_eq!(report.native_edit_output, report.evaluated_css);
    assert!(report.native_edit_output_matches_evaluated_css);
    assert!(
        report
            .native_edits
            .iter()
            .any(|edit| edit.edit_kind == "valueReplacement"
                && edit.replacement == "0px"
                && edit.abstract_value_kind == Some("exact"))
    );
    assert!(
        report
            .native_edits
            .iter()
            .any(|edit| edit.edit_kind == "structuralRemoval"
                && edit.replacement.is_empty()
                && edit.abstract_value.is_none())
    );
    assert_eq!(report.value_resolution.resolved_count, 1);
    assert!(report.evaluated_css.contains("margin: 0px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_uses_value_lattice_numeric_reduction() {
    let report = derive_static_stylesheet_module_evaluation(
        "@gap: (1px + 2px); .button { margin: @gap; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.resolved_replacements[0].text, "3px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert_eq!(
        report.value_resolution.values[0].rendered_value.as_deref(),
        Some("3px")
    );
    assert!(report.evaluated_css.contains("margin: 3px"));
}

#[test]
fn static_less_evaluation_reduces_escaped_string_variable_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "@filter: ~\"alpha(opacity=50)\"; .button { filter: @filter; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.resolved_replacements[0].text, "alpha(opacity=50)");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "raw");
    assert_eq!(
        report.value_resolution.values[0].rendered_value.as_deref(),
        Some("alpha(opacity=50)")
    );
    assert!(report.evaluated_css.contains("filter: alpha(opacity=50)"));
    assert!(!report.evaluated_css.contains("~\"alpha"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_preserves_dynamic_escaped_string_variable_values_as_raw() {
    let report = derive_static_stylesheet_module_evaluation(
        "@filter: ~\"@{name}\"; .button { filter: @filter; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.resolved_replacements[0].text, "~\"@{name}\"");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "raw");
    assert_eq!(report.value_resolution.raw_count, 1);
    assert_eq!(report.value_resolution.top_count, 0);
    assert!(report.evaluated_css.contains("filter: ~\"@{name}\""));
    assert!(!report.evaluated_css.contains("@filter:"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_escape_builtin_values_without_reentry() {
    let report = derive_static_stylesheet_module_evaluation(
        "@name: e(\"hello\"); @calc: e(\"calc(1px + 2px)\"); @min: e(\"min(1px, 2px)\"); @sign: e(\"sign(-2px)\"); .button { a: @name; b: @calc; c: @min; d: @sign; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 4);
    assert_eq!(report.value_resolution.raw_count, 4);
    assert_eq!(report.value_resolution.top_count, 0);
    assert!(report.evaluated_css.contains("a: hello"));
    assert!(report.evaluated_css.contains("b: calc(1px + 2px)"));
    assert!(report.evaluated_css.contains("c: min(1px, 2px)"));
    assert!(report.evaluated_css.contains("d: sign(-2px)"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_url_escape_builtin_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "@query: escape(\"a=1\"); @space: escape(\"hello world\"); @hash: escape(\"#fff\"); @unicode: escape(\"ä\"); @fn: escape(\"min(1px, 2px)\"); .button { a: @query; b: @space; c: @hash; d: @unicode; e: @fn; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 5);
    assert_eq!(report.value_resolution.raw_count, 5);
    assert_eq!(report.value_resolution.top_count, 0);
    assert!(report.evaluated_css.contains("a: a%3D1"));
    assert!(report.evaluated_css.contains("b: hello%20world"));
    assert!(report.evaluated_css.contains("c: %23fff"));
    assert!(report.evaluated_css.contains("d: %C3%A4"));
    assert!(report.evaluated_css.contains("e: min%281px,%202px%29"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_static_mixin_calls() {
    let report = derive_static_stylesheet_module_evaluation(
        "@brand: red; .tone(@color, @gap: 1px) { color: @color; margin: @gap; padding: @brand; } .button { .tone(blue, 2px); }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains(".tone(@color"));
    assert!(!report.evaluated_css.contains(".tone(blue"));
    assert!(report.evaluated_css.contains("color: blue"));
    assert!(report.evaluated_css.contains("margin: 2px"));
    assert!(report.evaluated_css.contains("padding: red"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_hash_mixin_calls() {
    let report = derive_static_stylesheet_module_evaluation(
        "#tone(@color, @gap: 1px) { color: @color; margin: @gap; } .button { #tone(red, 2px); }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("#tone(@color"));
    assert!(!report.evaluated_css.contains("#tone(red"));
    assert!(report.evaluated_css.contains("color: red"));
    assert!(report.evaluated_css.contains("margin: 2px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_mixin_declaration_accessors() {
    let report = derive_static_stylesheet_module_evaluation(
        ".tokens(@color, @gap: 1px) { @result: @color; width: @gap; } .button { color: .tokens(red)[@result]; margin: .tokens(red, 2px)[width]; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains(".tokens(@color"));
    assert!(!report.evaluated_css.contains(".tokens(red)[@result]"));
    assert!(!report.evaluated_css.contains(".tokens(red, 2px)[width]"));
    assert!(report.evaluated_css.contains("color: red"));
    assert!(report.evaluated_css.contains("margin: 2px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_preserves_unknown_mixin_accessor_members_as_oracle_report() {
    let report = derive_static_stylesheet_module_evaluation(
        ".tokens(@color) { @result: @color; } .button { color: .tokens(red)[@missing]; }",
        StyleDialect::Less,
    );

    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(report.evaluated_css.contains(".tokens(red)[@missing]"));
    assert!(report.evaluated_css.contains("@result: @color"));
    assert_eq!(report.replacement_count, 0);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_preserves_unknown_mixin_accessor_property_members_as_oracle_report() {
    let report = derive_static_stylesheet_module_evaluation(
        ".tokens(@color) { result: @color; } .button { color: .tokens(red)[missing]; }",
        StyleDialect::Less,
    );

    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(report.evaluated_css.contains(".tokens(red)[missing]"));
    assert!(report.evaluated_css.contains("result: @color"));
    assert_eq!(report.replacement_count, 0);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_namespace_mixin_access() {
    let report = derive_static_stylesheet_module_evaluation(
        "#bundle() { .rounded(@radius) { border-radius: @radius; } } .button { #bundle > .rounded(2px); }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("#bundle()"));
    assert!(!report.evaluated_css.contains("#bundle > .rounded"));
    assert!(report.evaluated_css.contains("border-radius: 2px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_parameterized_namespace_mixin_access() {
    let report = derive_static_stylesheet_module_evaluation(
        "#bundle(@color) { .tone() { color: @color; } } .button { #bundle(red) > .tone(); }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("#bundle(@color"));
    assert!(!report.evaluated_css.contains("#bundle(red) > .tone"));
    assert!(report.evaluated_css.contains("color: red"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_guarded_namespace_mixin_access() {
    let report = derive_static_stylesheet_module_evaluation(
        "#bundle() when (iscolor(red)) { .tone() { color: red; } } .button { #bundle > .tone(); }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("#bundle()"));
    assert!(!report.evaluated_css.contains("#bundle > .tone"));
    assert!(report.evaluated_css.contains("color: red"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_removes_false_guarded_namespace_mixin_access() {
    let report = derive_static_stylesheet_module_evaluation(
        "#bundle() when (iscolor(1px)) { .tone() { color: red; } } .button { #bundle > .tone(); }",
        StyleDialect::Less,
    );

    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("#bundle > .tone();"));
    assert!(!report.evaluated_css.contains("when (iscolor(1px))"));
    assert!(!report.evaluated_css.contains(".button { color: red"));
    assert_eq!(report.replacement_count, 0);
    assert!(report.native_structural_edit_count > 0);
    assert!(report.native_edit_output_matches_evaluated_css);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_detached_ruleset_calls() {
    let report = derive_static_stylesheet_module_evaluation(
        "@brand: red; @rules: { color: @brand; margin: 1px; }; .button { @rules(); }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@rules:"));
    assert!(!report.evaluated_css.contains("@rules();"));
    assert!(report.evaluated_css.contains("color: red"));
    assert!(report.evaluated_css.contains("margin: 1px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_detached_ruleset_body_property_variables() {
    let report = derive_static_stylesheet_module_evaluation(
        "@gap: 3px; @rules: { margin: @gap; padding: $margin; gap: $padding; }; .button { @rules(); }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@rules:"));
    assert!(!report.evaluated_css.contains("@rules();"));
    assert!(!report.evaluated_css.contains("$margin"));
    assert!(!report.evaluated_css.contains("$padding"));
    assert!(report.evaluated_css.contains("margin: 3px"));
    assert!(report.evaluated_css.contains("padding: 3px"));
    assert!(report.evaluated_css.contains("gap: 3px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_less_evaluation_expands_ruleset_guarded_mixin_arguments() {
    let report = derive_static_stylesheet_module_evaluation(
        ".apply(@block) when (isruleset(@block)) { @block(); } @rules: { color: red; margin: 1px; }; .button { .apply(@rules); }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains(".apply(@block"));
    assert!(!report.evaluated_css.contains("@rules:"));
    assert!(!report.evaluated_css.contains(".apply(@rules"));
    assert!(!report.evaluated_css.contains("@block();"));
    assert!(report.evaluated_css.contains("color: red"));
    assert!(report.evaluated_css.contains("margin: 1px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_removes_false_ruleset_guarded_mixins() {
    let report = derive_static_stylesheet_module_evaluation(
        ".apply(@block) when (isruleset(@block)) { @block(); } .button { .apply(red); }",
        StyleDialect::Less,
    );

    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains(".apply(red);"));
    assert!(!report.evaluated_css.contains("when (isruleset(@block))"));
    assert!(!report.evaluated_css.contains("@block();"));
    assert_eq!(report.replacement_count, 0);
    assert!(report.native_structural_edit_count > 0);
    assert!(report.native_edit_output_matches_evaluated_css);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_detached_ruleset_accessors() {
    let report = derive_static_stylesheet_module_evaluation(
        "@brand: red; @tokens: { primary: @brand; @gap: 2px; }; .button { color: @tokens[primary]; margin: @tokens[@gap]; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@tokens:"));
    assert!(!report.evaluated_css.contains("@tokens[primary]"));
    assert!(!report.evaluated_css.contains("@tokens[@gap]"));
    assert!(report.evaluated_css.contains("color: red"));
    assert!(report.evaluated_css.contains("margin: 2px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_resolves_detached_ruleset_accessor_properties_from_call_scope() {
    let report = derive_static_stylesheet_module_evaluation(
        "@tokens: { @gap: 2px; padding: @gap; gap: $padding; }; .button { padding: 4px; inset: @tokens[gap]; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@tokens:"));
    assert!(!report.evaluated_css.contains("@tokens[gap]"));
    assert!(report.evaluated_css.contains("padding: 4px"));
    assert!(report.evaluated_css.contains("inset: 4px"));
    assert!(!report.evaluated_css.contains("inset: 2px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_less_evaluation_preserves_detached_ruleset_accessor_missing_property_scope_as_raw() {
    let source = "@tokens: { margin: 2px; padding: $margin; }; .button { gap: @tokens[padding]; }";
    let report = derive_static_stylesheet_module_evaluation(source, StyleDialect::Less);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.evaluated_css, source);
    assert!(report.evaluated_css.contains("@tokens[padding]"));
    assert!(report.evaluated_css.contains("$margin"));
    assert_eq!(report.replacement_count, 0);
    assert_eq!(report.native_edit_count, 0);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_less_evaluation_preserves_unknown_detached_ruleset_accessor_members_as_oracle_report() {
    let report = derive_static_stylesheet_module_evaluation(
        "@tokens: { primary: red; }; .button { color: @tokens[missing]; }",
        StyleDialect::Less,
    );

    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(report.evaluated_css.contains("@tokens[missing]"));
    assert!(report.evaluated_css.contains("@tokens: { primary: red; };"));
    assert_eq!(report.replacement_count, 0);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_scoped_detached_ruleset_calls() {
    let report = derive_static_stylesheet_module_evaluation(
        "@rules: { color: red; }; .scope { @rules: { color: blue; }; .button { @rules(); } }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@rules:"));
    assert!(report.evaluated_css.contains("color: blue"));
    assert!(!report.evaluated_css.contains("color: red"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_detached_rulesets_with_mixin_calls() {
    let report = derive_static_stylesheet_module_evaluation(
        ".rounded() { border-radius: 2px; } @rules: { .rounded(); }; .button { @rules(); }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains(".rounded()"));
    assert!(!report.evaluated_css.contains("@rules:"));
    assert!(!report.evaluated_css.contains("@rules();"));
    assert!(report.evaluated_css.contains("border-radius: 2px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_preserves_unknown_detached_ruleset_mixin_calls_as_oracle_report() {
    let report = derive_static_stylesheet_module_evaluation(
        "@rules: { .unknown(); }; .button { @rules(); }",
        StyleDialect::Less,
    );

    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(report.evaluated_css.contains("@rules: { .unknown(); };"));
    assert!(report.evaluated_css.contains("@rules();"));
    assert_eq!(report.replacement_count, 0);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_preserves_unbound_parameterized_namespace_mixin_access_as_oracle_report()
{
    let report = derive_static_stylesheet_module_evaluation(
        "#bundle(@color) { .tone() { color: @color; } } .button { #bundle > .tone(); }",
        StyleDialect::Less,
    );

    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(report.evaluated_css.contains("#bundle > .tone();"));
    assert!(report.evaluated_css.contains("#bundle(@color)"));
    assert!(!report.evaluated_css.contains(".button { color:"));
    assert_eq!(report.replacement_count, 0);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_escaped_string_mixin_arguments() {
    let report = derive_static_stylesheet_module_evaluation(
        ".legacy(@value) { filter: @value; } .button { .legacy(~\"alpha(opacity=50)\"); }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains(".legacy(@value"));
    assert!(!report.evaluated_css.contains(".legacy(~\"alpha"));
    assert!(report.evaluated_css.contains("filter: alpha(opacity=50)"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_semicolon_separated_mixin_calls() {
    let report = derive_static_stylesheet_module_evaluation(
        ".shadow(@value; @color: red) { box-shadow: @value; color: @color; } .button { .shadow(1px, 2px, 3px; blue); }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains(".shadow(@value"));
    assert!(!report.evaluated_css.contains(".shadow(1px"));
    assert!(report.evaluated_css.contains("box-shadow: 1px, 2px, 3px"));
    assert!(report.evaluated_css.contains("color: blue"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_variadic_mixin_arguments() {
    let report = derive_static_stylesheet_module_evaluation(
        ".shadow(@color; @rest...) { color: @color; box-shadow: @rest; trace: @arguments; } .button { .shadow(red; 1px, 2px, 3px); }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains(".shadow(@color"));
    assert!(!report.evaluated_css.contains(".shadow(red"));
    assert!(report.evaluated_css.contains("color: red"));
    assert!(report.evaluated_css.contains("box-shadow: 1px, 2px, 3px"));
    assert!(report.evaluated_css.contains("trace: red, 1px, 2px, 3px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_named_mixin_arguments() {
    let report = derive_static_stylesheet_module_evaluation(
        ".tone(@color, @gap: 1px) { color: @color; margin: @gap; } .button { .tone(@gap: 2px, @color: blue); }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains(".tone(@color"));
    assert!(!report.evaluated_css.contains(".tone(@gap"));
    assert!(report.evaluated_css.contains("color: blue"));
    assert!(report.evaluated_css.contains("margin: 2px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_semicolon_named_mixin_arguments() {
    let report = derive_static_stylesheet_module_evaluation(
        ".tone(@color; @gap: 1px) { color: @color; margin: @gap; } .button { .tone(@gap: 2px; @color: blue); }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains(".tone(@color"));
    assert!(!report.evaluated_css.contains(".tone(@gap"));
    assert!(report.evaluated_css.contains("color: blue"));
    assert!(report.evaluated_css.contains("margin: 2px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_literal_pattern_mixins() {
    let report = derive_static_stylesheet_module_evaluation(
        ".tone(dark, @color) { color: @color; background: black; } .tone(light, @color) { color: @color; background: white; } .button { .tone(dark, red); }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains(".tone(dark"));
    assert!(!report.evaluated_css.contains(".tone(light"));
    assert!(report.evaluated_css.contains("color: red"));
    assert!(report.evaluated_css.contains("background: black"));
    assert!(!report.evaluated_css.contains("background: white"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_preserves_unmatched_literal_pattern_mixins_as_oracle_report() {
    let report = derive_static_stylesheet_module_evaluation(
        ".tone(dark, @color) { color: @color; background: black; } .button { .tone(light, red); }",
        StyleDialect::Less,
    );

    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(report.evaluated_css.contains(".tone(light, red);"));
    assert!(report.evaluated_css.contains(".tone(dark, @color)"));
    assert!(!report.evaluated_css.contains(".button { color: red"));
    assert_eq!(report.replacement_count, 0);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_does_not_expand_variadic_tokens_in_calls() {
    let report = derive_static_stylesheet_module_evaluation(
        "@gap: 1px; .space(@value) { margin: @value; } .button { .space(@gap...); }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(report.evaluated_css.contains(".space(1px...)"));
    assert!(!report.evaluated_css.contains("margin: 1px"));
    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.native_replacement_legacy_reflection_count, 0);
    assert_eq!(report.native_replacement_legacy_unreflected_count, 1);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_important_mixin_calls() {
    let report = derive_static_stylesheet_module_evaluation(
        ".tone(@color, @gap: 1px) { color: @color; margin: @gap; } .button { .tone(red, 2px) !important; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains(".tone(@color"));
    assert!(!report.evaluated_css.contains(".tone(red"));
    assert!(report.evaluated_css.contains("color: red !important"));
    assert!(report.evaluated_css.contains("margin: 2px !important"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_preserves_unknown_mixin_call_suffixes_as_oracle_report() {
    let report = derive_static_stylesheet_module_evaluation(
        ".tone(@color) { color: @color; } .button { .tone(red) !default; }",
        StyleDialect::Less,
    );

    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(report.evaluated_css.contains(".tone(red) !default;"));
    assert!(report.evaluated_css.contains(".tone(@color)"));
    assert!(!report.evaluated_css.contains(".button { color: red"));
    assert_eq!(report.replacement_count, 0);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_named_and_default_mixin_arguments() {
    let report = derive_static_stylesheet_module_evaluation(
        ".tone(@color: red, @gap: 1px, @double: 4px) { color: @color; margin: @gap; padding: @double; } .button { .tone(@gap: 2px, @color: blue); }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains(".tone(@color"));
    assert!(!report.evaluated_css.contains(".tone(@gap"));
    assert!(report.evaluated_css.contains("color: blue"));
    assert!(report.evaluated_css.contains("margin: 2px"));
    assert!(report.evaluated_css.contains("padding: 4px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_mixin_local_variables() {
    let report = derive_static_stylesheet_module_evaluation(
        ".tone(@gap) { @space: (@gap * 2); margin: @space; } .button { .tone(2px); }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@space"));
    assert!(!report.evaluated_css.contains(".tone(@gap"));
    assert!(!report.evaluated_css.contains(".tone(2px"));
    assert!(report.evaluated_css.contains("margin: 4px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_mixin_body_property_variables() {
    let report = derive_static_stylesheet_module_evaluation(
        ".space(@gap) { margin: @gap; padding: $margin; gap: $padding; } .button { .space(3px); }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains(".space(@gap"));
    assert!(!report.evaluated_css.contains(".space(3px"));
    assert!(!report.evaluated_css.contains("$margin"));
    assert!(!report.evaluated_css.contains("$padding"));
    assert!(report.evaluated_css.contains("margin: 3px"));
    assert!(report.evaluated_css.contains("padding: 3px"));
    assert!(report.evaluated_css.contains("gap: 3px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_less_evaluation_expands_nested_static_mixin_calls() {
    let report = derive_static_stylesheet_module_evaluation(
        ".spacing(@gap) { margin: @gap; } .tone(@gap, @color: red) { .spacing(@gap); color: @color; } .button { .tone(2px, blue); }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains(".spacing(@gap"));
    assert!(!report.evaluated_css.contains(".tone(@gap"));
    assert!(!report.evaluated_css.contains(".spacing(2px"));
    assert!(!report.evaluated_css.contains(".tone(2px"));
    assert!(report.evaluated_css.contains("margin: 2px"));
    assert!(report.evaluated_css.contains("color: blue"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_preserves_recursive_nested_mixin_calls_as_oracle_report() {
    let source = ".again() { .again(); } .button { .again(); }";
    let report = derive_static_stylesheet_module_evaluation(source, StyleDialect::Less);

    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.evaluated_css, source);
    assert_eq!(report.replacement_count, 0);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_static_guarded_mixin_calls() {
    let report = derive_static_stylesheet_module_evaluation(
        ".tone(@color) when (iscolor(@color)) { color: @color; } .button { .tone(red); }",
        StyleDialect::Less,
    );

    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains(".tone(@color"));
    assert!(!report.evaluated_css.contains(".tone(red"));
    assert!(report.evaluated_css.contains("color: red"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_treats_oklab_values_as_static_colors() {
    let report = derive_static_stylesheet_module_evaluation(
        ".tone(@color) when (iscolor(@color)) { color: @color; } .button { .tone(oklab(1 0 0)); }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains(".tone(@color"));
    assert!(!report.evaluated_css.contains(".tone(oklab"));
    assert!(report.evaluated_css.contains("color: oklab(1 0 0)"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_treats_rgb_values_as_static_colors() {
    let report = derive_static_stylesheet_module_evaluation(
        ".tone(@color) when (iscolor(@color)) { color: @color; } .button { .tone(rgb(127.5, 0, 127.5)); }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains(".tone(@color"));
    assert!(!report.evaluated_css.contains(".tone(rgb"));
    assert!(report.evaluated_css.contains("color: #800080"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_numeric_guarded_mixin_calls() {
    let report = derive_static_stylesheet_module_evaluation(
        ".space(@gap) when (isnumber(@gap)) { margin: @gap; } .button { .space(2px); }",
        StyleDialect::Less,
    );

    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains(".space(@gap"));
    assert!(!report.evaluated_css.contains(".space(2px"));
    assert!(report.evaluated_css.contains("margin: 2px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_type_guarded_mixin_calls() {
    let report = derive_static_stylesheet_module_evaluation(
        r#".space(@gap) when (ispixel(@gap)) { margin: @gap; }
.ratio(@value) when (ispercentage(@value)) { width: @value; }
.font(@family) when (isstring(@family)) { font-family: @family; }
.display(@value) when (iskeyword(@value)) { display: @value; }
.asset(@value) when (isurl(@value)) { background-image: @value; }
.unit(@gap) when (isunit(@gap, "rem")) { padding: @gap; }
.present() when (isdefined(@brand)) { color: @brand; }
.with-param(@tone) when (isdefined(@tone)) { border-color: @tone; }
@brand: red;
.button { .space(2px); .ratio(50%); .font("Roboto"); .display(block); .asset(url("./icon.svg")); .unit(1rem); .present(); .with-param(green); }"#,
        StyleDialect::Less,
    );

    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(report.evaluated_css.contains("margin: 2px"));
    assert!(report.evaluated_css.contains("width: 50%"));
    assert!(report.evaluated_css.contains(r#"font-family: "Roboto""#));
    assert!(report.evaluated_css.contains("display: block"));
    assert!(report.evaluated_css.contains("padding: 1rem"));
    assert!(report.evaluated_css.contains("color: red"));
    assert!(report.evaluated_css.contains("border-color: green"));
    assert!(
        report
            .evaluated_css
            .contains(r#"background-image: url("./icon.svg")"#)
    );
    assert!(!report.evaluated_css.contains(".space(2px"));
    assert!(!report.evaluated_css.contains(".asset(url"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_property_isdefined_guarded_mixin_calls() {
    let report = derive_static_stylesheet_module_evaluation(
        ".present() when (isdefined($color)) { border-color: $color; } .button { color: red; .present(); }",
        StyleDialect::Less,
    );

    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains(".present()"));
    assert!(report.evaluated_css.contains("color: red"));
    assert!(report.evaluated_css.contains("border-color: red"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_property_predicate_guarded_mixin_calls() {
    let report = derive_static_stylesheet_module_evaluation(
        ".space() when (isnumber($margin)) { padding: $margin; } .tone() when (iscolor($color)) { border-color: $color; } .unit() when (isunit($gap, px)) { inset: $gap; } .button { margin: (1px + 2px); color: red; gap: 4px; .space(); .tone(); .unit(); }",
        StyleDialect::Less,
    );

    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains(".space()"));
    assert!(!report.evaluated_css.contains(".tone()"));
    assert!(!report.evaluated_css.contains(".unit()"));
    assert!(report.evaluated_css.contains("padding: 3px"));
    assert!(report.evaluated_css.contains("border-color: red"));
    assert!(report.evaluated_css.contains("inset: 4px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_property_comparison_guarded_mixin_calls() {
    let report = derive_static_stylesheet_module_evaluation(
        ".space() when ($margin > 1px) { padding: $margin; } .button { margin: 2px; .space(); }",
        StyleDialect::Less,
    );

    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains(".space()"));
    assert!(report.evaluated_css.contains("padding: 2px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_future_property_guarded_mixins() {
    let report = derive_static_stylesheet_module_evaluation(
        ".space() when (isnumber($margin)) { padding: $margin; } .button { .space(); margin: 2px; }",
        StyleDialect::Less,
    );

    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(report.native_structural_edit_count > 0);
    assert!(report.native_edit_output_matches_evaluated_css);
    assert!(!report.evaluated_css.contains(".space()"));
    assert!(!report.evaluated_css.contains(".space();"));
    assert!(report.evaluated_css.contains("padding: 2px"));
    assert!(report.evaluated_css.contains("margin: 2px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_comparison_guarded_mixin_calls() {
    let report = derive_static_stylesheet_module_evaluation(
        r#".space(@gap) when (@gap > 1px) { margin: @gap; }
.tone(@color) when (@color = red) { color: @color; }
.combo(@gap, @color) when (@gap >= 2px) and (iscolor(@color)) { padding: @gap; border-color: @color; }
.inverse(@gap) when not (@gap < 2px) { inset: @gap; }
.fallback(@name) when (@name = primary), (@name = secondary) { content: @name; }
.button { .space(2px); .tone(red); .combo(2px, blue); .inverse(2px); .fallback(secondary); }"#,
        StyleDialect::Less,
    );

    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(report.evaluated_css.contains("margin: 2px"));
    assert!(report.evaluated_css.contains("color: red"));
    assert!(report.evaluated_css.contains("padding: 2px"));
    assert!(report.evaluated_css.contains("border-color: blue"));
    assert!(report.evaluated_css.contains("inset: 2px"));
    assert!(report.evaluated_css.contains("content: secondary"));
    assert!(!report.evaluated_css.contains(".space(2px"));
    assert!(!report.evaluated_css.contains(".fallback(secondary"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_multiple_matching_guarded_mixins() {
    let report = derive_static_stylesheet_module_evaluation(
        r#".tone(@color) when (@color = blue) { outline-color: blue; }
.tone(@color) when (@color = red) { color: @color; }
.tone(@color) when (iscolor(@color)) { border-color: @color; }
.button { .tone(red); }"#,
        StyleDialect::Less,
    );

    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("outline-color: blue"));
    assert!(report.evaluated_css.contains("color: red"));
    assert!(report.evaluated_css.contains("border-color: red"));
    assert!(!report.evaluated_css.contains(".tone(@color"));
    assert!(!report.evaluated_css.contains(".tone(red"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_expands_default_guarded_mixins() {
    let red_report = derive_static_stylesheet_module_evaluation(
        r#".tone(@color) when (@color = red) { color: @color; }
.tone(@color) when (default()) and (iscolor(@color)) { color: gray; }
.button { .tone(red); }"#,
        StyleDialect::Less,
    );
    assert!(red_report.is_some());
    let Some(red_report) = red_report else {
        return;
    };

    assert!(red_report.evaluated_css.contains("color: red"));
    assert!(!red_report.evaluated_css.contains("color: gray"));
    assert!(!red_report.evaluated_css.contains(".tone(@color"));
    assert!(!red_report.evaluated_css.contains(".tone(red"));
    assert!(red_report.oracle.all_legacy_declaration_values_preserved);

    let blue_report = derive_static_stylesheet_module_evaluation(
        r#".tone(@color) when (@color = red) { color: @color; }
.tone(@color) when (default()) and (iscolor(@color)) { color: gray; }
.button { .tone(blue); }"#,
        StyleDialect::Less,
    );
    assert!(blue_report.is_some());
    let Some(blue_report) = blue_report else {
        return;
    };

    assert!(blue_report.evaluated_css.contains("color: gray"));
    assert!(!blue_report.evaluated_css.contains("color: blue"));
    assert!(!blue_report.evaluated_css.contains(".tone(@color"));
    assert!(!blue_report.evaluated_css.contains(".tone(blue"));
    assert!(blue_report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_removes_false_guarded_mixins() {
    let report = derive_static_stylesheet_module_evaluation(
        ".tone(@value) when (iscolor(@value)) { color: @value; } .button { .tone(1px); }",
        StyleDialect::Less,
    );

    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains(".tone(1px)"));
    assert!(!report.evaluated_css.contains(".tone(@value)"));
    assert!(!report.evaluated_css.contains("color: 1px"));
    assert_eq!(report.replacement_count, 0);
    assert!(report.native_structural_edit_count > 0);
    assert!(report.native_edit_output_matches_evaluated_css);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_removes_false_comparison_guarded_mixins() {
    let report = derive_static_stylesheet_module_evaluation(
        ".space(@gap) when (@gap > 2px) { margin: @gap; } .button { .space(1px); }",
        StyleDialect::Less,
    );

    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains(".space(1px)"));
    assert!(!report.evaluated_css.contains(".space(@gap)"));
    assert!(!report.evaluated_css.contains("margin: 1px"));
    assert_eq!(report.replacement_count, 0);
    assert!(report.native_structural_edit_count > 0);
    assert!(report.native_edit_output_matches_evaluated_css);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_removes_false_unit_guarded_mixins() {
    let report = derive_static_stylesheet_module_evaluation(
        ".space(@gap) when (ispixel(@gap)) { margin: @gap; } .button { .space(2em); }",
        StyleDialect::Less,
    );

    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains(".space(2em)"));
    assert!(!report.evaluated_css.contains(".space(@gap)"));
    assert!(!report.evaluated_css.contains("margin: 2em"));
    assert_eq!(report.replacement_count, 0);
    assert!(report.native_structural_edit_count > 0);
    assert!(report.native_edit_output_matches_evaluated_css);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_removes_false_isunit_guarded_mixins() {
    let report = derive_static_stylesheet_module_evaluation(
        r#".space(@gap) when (isunit(@gap, "px")) { margin: @gap; } .button { .space(2em); }"#,
        StyleDialect::Less,
    );

    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains(".space(2em)"));
    assert!(!report.evaluated_css.contains(".space(@gap)"));
    assert!(!report.evaluated_css.contains("margin: 2em"));
    assert_eq!(report.replacement_count, 0);
    assert!(report.native_structural_edit_count > 0);
    assert!(report.native_edit_output_matches_evaluated_css);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_removes_false_type_predicate_guarded_mixins() {
    let report = derive_static_stylesheet_module_evaluation(
        r#".number(@value) when (isnumber(@value)) { margin: @value; }
.ratio(@value) when (ispercentage(@value)) { width: @value; }
.font(@value) when (isstring(@value)) { font-family: @value; }
.display(@value) when (iskeyword(@value)) { display: @value; }
.asset(@value) when (isurl(@value)) { background-image: @value; }
.em(@value) when (isem(@value)) { letter-spacing: @value; }
.button { .number(red); .ratio(2px); .font(block); .display("Roboto"); .asset(red); .em(1rem); }"#,
        StyleDialect::Less,
    );

    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

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
            !report.evaluated_css.contains(snippet),
            "false type-predicate guard output retained `{snippet}` in {}",
            report.evaluated_css
        );
    }
    assert_eq!(report.replacement_count, 0);
    assert!(report.native_structural_edit_count > 0);
    assert!(report.native_edit_output_matches_evaluated_css);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_removes_false_isdefined_guarded_mixins() {
    let report = derive_static_stylesheet_module_evaluation(
        ".missing() when (isdefined(@missing)) { color: blue; } .button { .missing(); }",
        StyleDialect::Less,
    );

    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains(".missing();"));
    assert!(!report.evaluated_css.contains(".missing() when"));
    assert!(!report.evaluated_css.contains(".button { color: blue"));
    assert_eq!(report.replacement_count, 0);
    assert!(report.native_structural_edit_count > 0);
    assert!(report.native_edit_output_matches_evaluated_css);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_removes_false_property_isdefined_guarded_mixins() {
    let report = derive_static_stylesheet_module_evaluation(
        ".missing() when (isdefined($missing)) { color: blue; } .button { .missing(); }",
        StyleDialect::Less,
    );

    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains(".missing();"));
    assert!(!report.evaluated_css.contains(".missing() when"));
    assert!(!report.evaluated_css.contains(".button { color: blue"));
    assert_eq!(report.replacement_count, 0);
    assert!(report.native_structural_edit_count > 0);
    assert!(report.native_edit_output_matches_evaluated_css);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_uses_value_lattice_numeric_reduction() {
    let report = derive_static_stylesheet_module_evaluation(
        "$gap: (1px + 2px); .button { margin: $gap; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.resolved_replacements[0].text, "3px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert_eq!(
        report.value_resolution.values[0].rendered_value.as_deref(),
        Some("3px")
    );
    assert!(report.evaluated_css.contains("margin: 3px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_bare_numeric_expressions() {
    let report = derive_static_stylesheet_module_evaluation(
        "$gap: 1px + 2px; .button { margin: $gap; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.resolved_replacements[0].text, "3px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert_eq!(
        report.value_resolution.values[0].rendered_value.as_deref(),
        Some("3px")
    );
    assert!(report.evaluated_css.contains("margin: 3px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_calc_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "$gap: calc(1px + 2px); .button { margin: $gap; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.resolved_replacements[0].text, "3px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert_eq!(
        report.value_resolution.values[0].rendered_value.as_deref(),
        Some("3px")
    );
    assert!(report.evaluated_css.contains("margin: 3px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_numeric_builtin_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "$gap: min(10px, 4px); $offset: clamp(1px, 3px, 2px); .button { margin: $gap; padding: $offset; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    let replacements = report
        .resolved_replacements
        .iter()
        .map(|replacement| replacement.text.as_str())
        .collect::<Vec<_>>();
    assert!(replacements.contains(&"4px"));
    assert!(replacements.contains(&"2px"));
    assert!(
        report
            .resolved_replacements
            .iter()
            .all(|replacement| replacement.abstract_value_kind == "exact")
    );
    assert!(report.evaluated_css.contains("margin: 4px"));
    assert!(report.evaluated_css.contains("padding: 2px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_if_function_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "$gap: if(false, 1px, 2px); .button { margin: $gap; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.resolved_replacements[0].text, "2px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 2px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_nth_function_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "$gap: nth(1px 2px 3px, 2); $pad: list.nth((4px, 5px, 6px), -1); .button { margin: $gap; padding: $pad; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    let replacements = report
        .resolved_replacements
        .iter()
        .map(|replacement| replacement.text.as_str())
        .collect::<Vec<_>>();
    assert!(replacements.contains(&"2px"));
    assert!(replacements.contains(&"6px"));
    assert!(report.evaluated_css.contains("margin: 2px"));
    assert!(report.evaluated_css.contains("padding: 6px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_map_get_function_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "$gap: map-get((default: 2px, dense: 1px), default); $tone: map.get((primary: red, secondary: blue), secondary); .button { margin: $gap; color: $tone; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    let replacements = report
        .resolved_replacements
        .iter()
        .map(|replacement| replacement.text.as_str())
        .collect::<Vec<_>>();
    assert!(replacements.contains(&"2px"));
    assert!(replacements.contains(&"blue"));
    assert!(report.evaluated_css.contains("margin: 2px"));
    assert!(report.evaluated_css.contains("color: blue"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_nested_static_map_get_and_has_key_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "$weight: map.get((font: (weights: (regular: 400, medium: 500))), font, weights, medium); $tone: map-get((theme: (primary: red)), theme, primary); $has: if(map.has-key((theme: (primary: red)), theme, primary), 1px, 2px); $missing: if(map-has-key((theme: (primary: red)), theme, missing), 3px, 4px); .button { font-weight: $weight; color: $tone; margin: $has; padding: $missing; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    let replacements = report
        .resolved_replacements
        .iter()
        .map(|replacement| replacement.text.as_str())
        .collect::<Vec<_>>();
    assert!(replacements.contains(&"500"));
    assert!(replacements.contains(&"red"));
    assert!(replacements.contains(&"1px"));
    assert!(replacements.contains(&"4px"));
    assert!(report.evaluated_css.contains("font-weight: 500"));
    assert!(report.evaluated_css.contains("color: red"));
    assert!(report.evaluated_css.contains("margin: 1px"));
    assert!(report.evaluated_css.contains("padding: 4px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_collection_size_and_search_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "$count: list.length((1px, 2px, 3px)); $position: index(red blue green, green); .button { z-index: $count; order: $position; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    let replacements = report
        .resolved_replacements
        .iter()
        .map(|replacement| replacement.text.as_str())
        .collect::<Vec<_>>();
    assert!(replacements.contains(&"3"));
    assert!(report.evaluated_css.contains("z-index: 3"));
    assert!(report.evaluated_css.contains("order: 3"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_list_metadata_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "$separator: list.separator((1px, 2px)); $legacy-separator: list-separator(1px 2px); $space: if(list-separator(1px 2px) == \"space\", 1px, 2px); $bracketed: if(list.is-bracketed([1px 2px]), 3px, 4px); $legacy-bracketed: if(is-bracketed([1px 2px]), 5px, 6px); .button { content: $separator; quotes: $legacy-separator; margin: $space; padding: $bracketed; inset: $legacy-bracketed; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    let replacements = report
        .resolved_replacements
        .iter()
        .map(|replacement| replacement.text.as_str())
        .collect::<Vec<_>>();
    assert!(replacements.contains(&"\"comma\""));
    assert!(replacements.contains(&"\"space\""));
    assert!(replacements.contains(&"1px"));
    assert!(replacements.contains(&"3px"));
    assert!(replacements.contains(&"5px"));
    assert!(report.evaluated_css.contains("content: \"comma\""));
    assert!(report.evaluated_css.contains("quotes: \"space\""));
    assert!(report.evaluated_css.contains("margin: 1px"));
    assert!(report.evaluated_css.contains("padding: 3px"));
    assert!(report.evaluated_css.contains("inset: 5px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_string_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "$family: string.quote(Demo); $style: unquote(\"serif\"); $length: string.length(\"Helvetica Neue\"); $position: str-index(\"Helvetica Neue\", \"Neue\"); $slice: string.slice(\"Helvetica Neue\", 1, -6); $inserted: string.insert(\"Roboto Bold\", \" Mono\", 7); $upper: to-upper-case(sans-serif); $lower: string.to-lower-case(\"BOLD\"); .button { font-family: $family, $style; z-index: $length; order: $position; content: $slice; src: $inserted; text-transform: $upper; font-style: $lower; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    let rendered_values = report
        .resolved_replacements
        .iter()
        .filter_map(|replacement| replacement.rendered_value.as_deref())
        .collect::<Vec<_>>();
    assert!(rendered_values.contains(&"\"Demo\""));
    assert!(rendered_values.contains(&"serif"));
    assert!(rendered_values.contains(&"14"));
    assert!(rendered_values.contains(&"11"));
    assert!(rendered_values.contains(&"\"Helvetica\""));
    assert!(rendered_values.contains(&"\"Roboto Mono Bold\""));
    assert!(rendered_values.contains(&"SANS-SERIF"));
    assert!(rendered_values.contains(&"\"bold\""));
    assert!(
        report
            .evaluated_css
            .contains("font-family: \"Demo\", serif")
    );
    assert!(report.evaluated_css.contains("z-index: 14"));
    assert!(report.evaluated_css.contains("order: 11"));
    assert!(report.evaluated_css.contains("content: \"Helvetica\""));
    assert!(report.evaluated_css.contains("src: \"Roboto Mono Bold\""));
    assert!(report.evaluated_css.contains("text-transform: SANS-SERIF"));
    assert!(report.evaluated_css.contains("font-style: \"bold\""));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_map_has_key_conditions() {
    let report = derive_static_stylesheet_module_evaluation(
        "$gap: if(map.has-key((default: 2px, dense: 1px), dense), 1px, 2px); $pad: if(map-has-key((default: 2px), missing), 3px, 4px); .button { margin: $gap; padding: $pad; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    let replacements = report
        .resolved_replacements
        .iter()
        .map(|replacement| replacement.text.as_str())
        .collect::<Vec<_>>();
    assert!(replacements.contains(&"1px"));
    assert!(replacements.contains(&"4px"));
    assert!(report.evaluated_css.contains("margin: 1px"));
    assert!(report.evaluated_css.contains("padding: 4px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_map_key_and_value_lists() {
    let report = derive_static_stylesheet_module_evaluation(
        "$key-count: list.length(map.keys((default: 1px, dense: 2px))); $first-value: list.nth(map.values((default: 1px, dense: 2px)), 1); $legacy-key-count: length(map-keys((primary: red, secondary: blue))); $legacy-value: nth(map-values((primary: red, secondary: blue)), 2); .button { z-index: $key-count; margin: $first-value; order: $legacy-key-count; color: $legacy-value; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    let replacements = report
        .resolved_replacements
        .iter()
        .map(|replacement| replacement.text.as_str())
        .collect::<Vec<_>>();
    assert!(replacements.contains(&"2"));
    assert!(replacements.contains(&"1px"));
    assert!(replacements.contains(&"blue"));
    assert!(report.evaluated_css.contains("z-index: 2"));
    assert!(report.evaluated_css.contains("margin: 1px"));
    assert!(report.evaluated_css.contains("order: 2"));
    assert!(report.evaluated_css.contains("color: blue"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_map_merge_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "$gap: map.get(map.merge((default: 1px, dense: 2px), (dense: 3px, compact: 4px)), dense); $count: list.length(map.keys(map-merge((default: 1px), (compact: 4px)))); .button { margin: $gap; z-index: $count; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    let replacements = report
        .resolved_replacements
        .iter()
        .map(|replacement| replacement.text.as_str())
        .collect::<Vec<_>>();
    assert!(replacements.contains(&"3px"));
    assert!(replacements.contains(&"2"));
    assert!(report.evaluated_css.contains("margin: 3px"));
    assert!(report.evaluated_css.contains("z-index: 2"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_nested_static_map_merge_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "$gap: map.get(map.merge((theme: (spacing: (sm: 4px))), theme, spacing, (md: 8px)), theme, spacing, md); $count: list.length(map.keys(map.merge((), theme, colors, (primary: red, secondary: blue)))); .button { margin: $gap; z-index: $count; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    let replacements = report
        .resolved_replacements
        .iter()
        .map(|replacement| replacement.text.as_str())
        .collect::<Vec<_>>();
    assert!(replacements.contains(&"8px"));
    assert!(replacements.contains(&"1"));
    assert!(report.evaluated_css.contains("margin: 8px"));
    assert!(report.evaluated_css.contains("z-index: 1"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_map_deep_merge_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "$merged: map.deep-merge((theme: (spacing: (sm: 4px), tone: blue)), (theme: (spacing: (md: 8px), tone: red))); $gap: map.get($merged, theme, spacing, md); $old: map.get($merged, theme, spacing, sm); $tone: map.get($merged, theme, tone); $count: list.length(map.keys(map.get($merged, theme, spacing))); .button { margin: $gap; padding: $old; color: $tone; z-index: $count; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    let replacements = report
        .resolved_replacements
        .iter()
        .map(|replacement| replacement.text.as_str())
        .collect::<Vec<_>>();
    assert!(replacements.contains(&"8px"));
    assert!(replacements.contains(&"4px"));
    assert!(replacements.contains(&"red"));
    assert!(replacements.contains(&"2"));
    assert!(report.evaluated_css.contains("margin: 8px"));
    assert!(report.evaluated_css.contains("padding: 4px"));
    assert!(report.evaluated_css.contains("color: red"));
    assert!(report.evaluated_css.contains("z-index: 2"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_map_remove_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "$gap: map.get(map.remove((default: 1px, dense: 2px, compact: 4px), dense, missing), compact); $count: list.length(map.keys(map-remove((default: 1px, dense: 2px), default, dense))); .button { margin: $gap; z-index: $count; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    let replacements = report
        .resolved_replacements
        .iter()
        .map(|replacement| replacement.text.as_str())
        .collect::<Vec<_>>();
    assert!(replacements.contains(&"4px"));
    assert!(replacements.contains(&"0"));
    assert!(report.evaluated_css.contains("margin: 4px"));
    assert!(report.evaluated_css.contains("z-index: 0"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_nested_static_map_deep_remove_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "$gap: map.get(map.deep-remove((theme: (spacing: (sm: 4px, md: 8px))), theme, spacing, sm), theme, spacing, md); $count: list.length(map.keys(map.deep-remove((theme: (colors: (primary: red, secondary: blue))), theme, colors, primary))); $tone: map.get(map.deep-remove((theme: blue), theme, colors, primary), theme); .button { margin: $gap; z-index: $count; color: $tone; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    let replacements = report
        .resolved_replacements
        .iter()
        .map(|replacement| replacement.text.as_str())
        .collect::<Vec<_>>();
    assert!(replacements.contains(&"8px"));
    assert!(replacements.contains(&"1"));
    assert!(replacements.contains(&"blue"));
    assert!(report.evaluated_css.contains("margin: 8px"));
    assert!(report.evaluated_css.contains("z-index: 1"));
    assert!(report.evaluated_css.contains("color: blue"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_map_set_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "$weight: map.get(map.set((regular: 400, medium: 500), regular, 300), regular); $count: list.length(map.keys(map.set((), compact, 4px))); .button { font-weight: $weight; z-index: $count; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    let replacements = report
        .resolved_replacements
        .iter()
        .map(|replacement| replacement.text.as_str())
        .collect::<Vec<_>>();
    assert!(replacements.contains(&"300"));
    assert!(replacements.contains(&"1"));
    assert!(report.evaluated_css.contains("font-weight: 300"));
    assert!(report.evaluated_css.contains("z-index: 1"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_nested_static_map_set_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "$tone: map.get(map.set((theme: blue), theme, colors, primary, red), theme, colors, primary); $gap: map.get(map.set((theme: (spacing: (sm: 4px))), theme, spacing, md, 8px), theme, spacing, md); .button { color: $tone; margin: $gap; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    let replacements = report
        .resolved_replacements
        .iter()
        .map(|replacement| replacement.text.as_str())
        .collect::<Vec<_>>();
    assert!(replacements.contains(&"red"));
    assert!(replacements.contains(&"8px"));
    assert!(report.evaluated_css.contains("color: red"));
    assert!(report.evaluated_css.contains("margin: 8px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_math_numeric_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "$gap: math.div(6px, 3); $ratio: percentage(.25); $math-ratio: math.percentage(.5); $pad: if(math.is-unitless(2), 1px, 2px); $border: if(unitless(2px), 3px, 4px); $unit: math.unit(2px); $unitless-name: unit(2); $compatible: if(math.compatible(1px, 2px), 5px, 6px); $global-compatible: if(comparable(1, 1px), 7px, 8px); .button { margin: $gap; width: $ratio; max-width: $math-ratio; padding: $pad; border-width: $border; content: $unit; quotes: $unitless-name; outline-width: $compatible; min-width: $global-compatible; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    let replacements = report
        .resolved_replacements
        .iter()
        .map(|replacement| replacement.text.as_str())
        .collect::<Vec<_>>();
    assert!(replacements.contains(&"2px"));
    assert!(replacements.contains(&"25%"));
    assert!(replacements.contains(&"50%"));
    assert!(replacements.contains(&"1px"));
    assert!(replacements.contains(&"4px"));
    assert!(replacements.contains(&"\"px\""));
    assert!(replacements.contains(&"\"\""));
    assert!(replacements.contains(&"5px"));
    assert!(replacements.contains(&"8px"));
    assert!(report.evaluated_css.contains("margin: 2px"));
    assert!(report.evaluated_css.contains("width: 25%"));
    assert!(report.evaluated_css.contains("max-width: 50%"));
    assert!(report.evaluated_css.contains("padding: 1px"));
    assert!(report.evaluated_css.contains("border-width: 4px"));
    assert!(report.evaluated_css.contains("content: \"px\""));
    assert!(report.evaluated_css.contains("quotes: \"\""));
    assert!(report.evaluated_css.contains("outline-width: 5px"));
    assert!(report.evaluated_css.contains("min-width: 8px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_namespaced_math_aliases() {
    let report = derive_static_stylesheet_module_evaluation(
        "$gap: math.max(1px, 3px); $pad: math.min(4px, 2px); $offset: math.abs(-2px); $width: math.clamp(1px, 5px, 3px); .button { margin: $gap; padding: $pad; inset: $offset; width: $width; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    let replacements = report
        .resolved_replacements
        .iter()
        .map(|replacement| replacement.text.as_str())
        .collect::<Vec<_>>();
    assert!(replacements.contains(&"3px"));
    assert!(replacements.contains(&"2px"));
    assert!(report.evaluated_css.contains("margin: 3px"));
    assert!(report.evaluated_css.contains("padding: 2px"));
    assert!(report.evaluated_css.contains("inset: 2px"));
    assert!(report.evaluated_css.contains("width: 3px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_legacy_rounding_aliases() {
    let report = derive_static_stylesheet_module_evaluation(
        "$ceil: ceil(1.2px); $floor: floor(1.8px); $round: round(1.5px); .button { top: $ceil; bottom: $floor; left: $round; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    let replacements = report
        .resolved_replacements
        .iter()
        .map(|replacement| replacement.text.as_str())
        .collect::<Vec<_>>();
    assert!(replacements.contains(&"2px"));
    assert!(replacements.contains(&"1px"));
    assert!(report.evaluated_css.contains("top: 2px"));
    assert!(report.evaluated_css.contains("bottom: 1px"));
    assert!(report.evaluated_css.contains("left: 2px"));
    assert_eq!(report.value_resolution.reference_count, 3);
    assert_eq!(report.value_resolution.resolved_count, 3);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_extended_namespaced_math_aliases() {
    let report = derive_static_stylesheet_module_evaluation(
        "$sign: math.sign(-2px); $ceil: math.ceil(1.2px); $floor: math.floor(1.8px); $round: math.round(1.5px); $mod: math.mod(7px, 3px); $rem: math.rem(8px, 3px); $hypot: math.hypot(3px, 4px); $sqrt: math.sqrt(9); $pow: math.pow(2, 3); $exp: math.exp(0); $log: math.log(8, 2); .button { z-index: $sign; margin: $mod; padding: $rem; width: $hypot; opacity: $sqrt; order: $pow; flex-grow: $exp; flex-shrink: $log; top: $ceil; bottom: $floor; left: $round; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    let replacements = report
        .resolved_replacements
        .iter()
        .map(|replacement| replacement.text.as_str())
        .collect::<Vec<_>>();
    assert!(replacements.contains(&"-1"));
    assert!(replacements.contains(&"1px"));
    assert!(replacements.contains(&"2px"));
    assert!(replacements.contains(&"5px"));
    assert!(replacements.contains(&"3"));
    assert!(replacements.contains(&"8"));
    assert!(report.evaluated_css.contains("z-index: -1"));
    assert!(report.evaluated_css.contains("margin: 1px"));
    assert!(report.evaluated_css.contains("padding: 2px"));
    assert!(report.evaluated_css.contains("width: 5px"));
    assert!(report.evaluated_css.contains("opacity: 3"));
    assert!(report.evaluated_css.contains("order: 8"));
    assert!(report.evaluated_css.contains("flex-grow: 1"));
    assert!(report.evaluated_css.contains("flex-shrink: 3"));
    assert!(report.evaluated_css.contains("top: 2px"));
    assert!(report.evaluated_css.contains("bottom: 1px"));
    assert!(report.evaluated_css.contains("left: 2px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_namespaced_math_trig_aliases() {
    let report = derive_static_stylesheet_module_evaluation(
        "$sin: math.sin(30deg); $cos: math.cos(60deg); $tan: math.tan(45deg); $asin: math.asin(.5); $acos: math.acos(.5); $atan: math.atan(1); $atan2: math.atan2(1px, 1px); .button { opacity: $sin; flex-grow: $cos; flex-shrink: $tan; rotate: $asin; offset-rotate: $acos; --atan: $atan; --atan2: $atan2; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    let replacements = report
        .resolved_replacements
        .iter()
        .map(|replacement| replacement.text.as_str())
        .collect::<Vec<_>>();
    assert!(replacements.contains(&"0.5"));
    assert!(replacements.contains(&"1"));
    assert!(replacements.contains(&"30deg"));
    assert!(replacements.contains(&"60deg"));
    assert!(replacements.contains(&"45deg"));
    assert!(report.evaluated_css.contains("opacity: 0.5"));
    assert!(report.evaluated_css.contains("flex-grow: 0.5"));
    assert!(report.evaluated_css.contains("flex-shrink: 1"));
    assert!(report.evaluated_css.contains("rotate: 30deg"));
    assert!(report.evaluated_css.contains("offset-rotate: 60deg"));
    assert!(report.evaluated_css.contains("--atan: 45deg"));
    assert!(report.evaluated_css.contains("--atan2: 45deg"));
    assert_eq!(report.value_resolution.reference_count, 7);
    assert_eq!(report.value_resolution.resolved_count, 7);
    assert_eq!(report.value_resolution.raw_count, 0);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_namespaced_math_constants() {
    let report = derive_static_stylesheet_module_evaluation(
        "$pi: math.$pi; $e: math.$e; $epsilon: math.$epsilon; $max-safe: math.$max-safe-integer; $min-safe: math.$min-safe-integer; .button { --pi: $pi; --e: $e; --epsilon: $epsilon; z-index: $max-safe; order: $min-safe; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    let replacements = report
        .resolved_replacements
        .iter()
        .map(|replacement| replacement.text.as_str())
        .collect::<Vec<_>>();
    assert!(replacements.contains(&"3.1415926536"));
    assert!(replacements.contains(&"2.7182818285"));
    assert!(replacements.contains(&"0"));
    assert!(replacements.contains(&"9007199254740991"));
    assert!(replacements.contains(&"-9007199254740991"));
    assert!(report.evaluated_css.contains("--pi: 3.1415926536"));
    assert!(report.evaluated_css.contains("--e: 2.7182818285"));
    assert!(report.evaluated_css.contains("--epsilon: 0"));
    assert!(report.evaluated_css.contains("z-index: 9007199254740991"));
    assert!(report.evaluated_css.contains("order: -9007199254740991"));
    assert_eq!(report.value_resolution.reference_count, 5);
    assert_eq!(report.value_resolution.resolved_count, 5);
    assert_eq!(report.value_resolution.raw_count, 0);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_does_not_treat_math_constants_as_variable_dependencies() {
    let report = summarize_static_stylesheet_value_resolution(
        "$pi: math.$pi; .button { --pi: $pi; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.reference_count, 1);
    assert_eq!(report.resolved_count, 1);
    assert_eq!(report.values[0].source_text, "$pi");
    assert_eq!(report.values[0].rendered_value.as_deref(), Some("3.141593"));
}

#[test]
fn static_scss_evaluation_reduces_math_constant_function_arguments() {
    let report = derive_static_stylesheet_module_evaluation(
        "$unitless: if(math.is-unitless(math.$pi), 1px, 2px); $unit-ok: if(math.unit(math.$pi) == \"\", 5px, 6px); $compatible: if(math.compatible(math.$pi, 1), 3px, 4px); $sin: math.sin(math.$pi); .button { padding: $unitless; border-width: $unit-ok; margin: $compatible; opacity: $sin; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(report.evaluated_css.contains("padding: 1px"));
    assert!(report.evaluated_css.contains("border-width: 5px"));
    assert!(report.evaluated_css.contains("margin: 3px"));
    assert!(report.evaluated_css.contains("opacity: 0"));
    assert_eq!(report.value_resolution.reference_count, 4);
    assert_eq!(report.value_resolution.resolved_count, 4);
    assert_eq!(report.value_resolution.raw_count, 0);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_keeps_unsupported_namespaced_math_trig_raw() {
    let report = derive_static_stylesheet_module_evaluation(
        "$bad-angle: math.sin(1px); $bad-inverse: math.asin(2); .button { width: $bad-angle; height: $bad-inverse; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    assert_eq!(report.replacement_count, 2);
    let replacements = report
        .resolved_replacements
        .iter()
        .map(|replacement| replacement.text.as_str())
        .collect::<Vec<_>>();
    assert!(replacements.contains(&"math.sin(1px)"));
    assert!(replacements.contains(&"math.asin(2)"));
    assert_eq!(report.value_resolution.raw_count, 2);
    assert!(report.evaluated_css.contains("width: math.sin(1px)"));
    assert!(report.evaluated_css.contains("height: math.asin(2)"));
}

#[test]
fn static_scss_evaluation_keeps_unsupported_namespaced_math_raw() {
    let report = derive_static_stylesheet_module_evaluation(
        "$gap: math.random(); .button { margin: $gap; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].text, "math.random()");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "raw");
    assert!(report.evaluated_css.contains("margin: math.random()"));

    let resolution = summarize_static_stylesheet_value_resolution(
        "$gap: math.random(); .button { margin: $gap; }",
        StyleDialect::Scss,
    );
    assert!(resolution.is_some());
    let Some(resolution) = resolution else {
        return;
    };

    assert_eq!(resolution.reference_count, 1);
    assert_eq!(resolution.raw_count, 1);
    assert_eq!(resolution.values[0].source_text, "$gap");
    assert_eq!(
        resolution.values[0].rendered_value.as_deref(),
        Some("math.random()")
    );
    assert_eq!(resolution.values[0].outcome, "raw");
    assert_eq!(resolution.values[0].reason, "unsupportedDynamic");
}

#[test]
fn static_scss_evaluation_reduces_nested_static_list_conditions_in_order() {
    let report = derive_static_stylesheet_module_evaluation(
        "$count: list.length(if(false, 1px 2px, 3px 4px 5px)); .button { z-index: $count; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(
        report
            .resolved_replacements
            .iter()
            .any(|replacement| { replacement.name == "$count" && replacement.text == "3" })
    );
    assert!(report.evaluated_css.contains("z-index: 3"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_if_not_conditions() {
    let report = derive_static_stylesheet_module_evaluation(
        "$gap: if(not true, 1px, 2px); .button { margin: $gap; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.resolved_replacements[0].text, "2px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 2px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_if_boolean_conditions() {
    let and_report = derive_static_stylesheet_module_evaluation(
        "$gap: if(false and true, 1px, 2px); .button { margin: $gap; }",
        StyleDialect::Scss,
    );
    assert!(and_report.is_some());
    let Some(and_report) = and_report else {
        return;
    };
    assert_eq!(and_report.resolved_replacements[0].text, "2px");
    assert!(
        and_report
            .evaluated_css
            .contains(".button { margin: 2px; }")
    );
    assert!(and_report.oracle.all_legacy_declaration_values_preserved);

    let or_report = derive_static_stylesheet_module_evaluation(
        "$gap: if(false or true, 1px, 2px); .button { margin: $gap; }",
        StyleDialect::Scss,
    );
    assert!(or_report.is_some());
    let Some(or_report) = or_report else {
        return;
    };
    assert_eq!(or_report.resolved_replacements[0].text, "1px");
    assert!(or_report.evaluated_css.contains(".button { margin: 1px; }"));
    assert!(or_report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_if_equality_conditions() {
    let report = derive_static_stylesheet_module_evaluation(
        "$gap: if(1px == 2px, 1px, 2px); .button { margin: $gap; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.resolved_replacements[0].text, "2px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 2px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_if_inequality_conditions() {
    let report = derive_static_stylesheet_module_evaluation(
        "$gap: if(1px != 2px, 1px, 2px); .button { margin: $gap; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.resolved_replacements[0].text, "1px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 1px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_if_numeric_ordering_conditions() {
    let report = derive_static_stylesheet_module_evaluation(
        "$gap: if(3px > 2px, 1px, 2px); .button { margin: $gap; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.resolved_replacements[0].text, "1px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 1px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_if_zero_numeric_ordering_conditions() {
    let report = derive_static_stylesheet_module_evaluation(
        "$gap: if(0px >= 0, 1px, 2px); .button { margin: $gap; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.resolved_replacements[0].text, "1px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 1px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_stylesheet_bang_safety_only_allows_comparisons() {
    assert!(static_scss_bang_usage_is_comparison_only(
        "if(1px != 2px, 1px, 2px)"
    ));
    assert!(!static_scss_bang_usage_is_comparison_only("1px !important"));
}

#[test]
fn static_scss_evaluation_reduces_parenthesized_if_conditions() {
    let report = derive_static_stylesheet_module_evaluation(
        "$gap: if((false or true), 1px, 2px); .button { margin: $gap; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.resolved_replacements[0].text, "1px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 1px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_max_builtin_value() {
    let report = derive_static_stylesheet_module_evaluation(
        "@gap: max(1px, 2px); .button { margin: @gap; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.resolved_replacements[0].text, "2px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert_eq!(
        report.value_resolution.values[0].rendered_value.as_deref(),
        Some("2px")
    );
    assert!(report.evaluated_css.contains("margin: 2px"));
}

#[test]
fn static_less_evaluation_reduces_unit_builtin_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "@gap: unit(5, px); @plain: unit(5px); @unit-name: get-unit(1.5rem); .button { margin: @gap; padding: @plain; --unit: @unit-name; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 3);
    assert_eq!(report.resolved_replacements[0].text, "5px");
    assert_eq!(report.resolved_replacements[1].text, "5");
    assert_eq!(report.resolved_replacements[2].text, "rem");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert_eq!(report.resolved_replacements[1].abstract_value_kind, "exact");
    assert_eq!(report.resolved_replacements[2].abstract_value_kind, "raw");
    assert_eq!(report.value_resolution.resolved_count, 2);
    assert_eq!(report.value_resolution.raw_count, 1);
    assert!(report.evaluated_css.contains("margin: 5px"));
    assert!(report.evaluated_css.contains("padding: 5"));
    assert!(report.evaluated_css.contains("--unit: rem"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_convert_builtin_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "@cm: convert(1in, cm); @inch: convert(2.54cm, in); @px: convert(96px, in); @ms: convert(1s, ms); @sec: convert(250ms, s); @deg: convert(1rad, deg); @turn: convert(.5turn, deg); @same: convert(1in, s); .button { cm: @cm; inch: @inch; px: @px; ms: @ms; sec: @sec; deg: @deg; turn: @turn; same: @same; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 8);
    assert_eq!(report.value_resolution.resolved_count, 6);
    assert_eq!(report.value_resolution.raw_count, 2);
    assert!(report.evaluated_css.contains("cm: 2.54cm"));
    assert!(report.evaluated_css.contains("inch: 1in"));
    assert!(report.evaluated_css.contains("px: 1in"));
    assert!(report.evaluated_css.contains("ms: 1000ms"));
    assert!(report.evaluated_css.contains("sec: 0.25s"));
    assert!(report.evaluated_css.contains("deg: 57.29577951deg"));
    assert!(report.evaluated_css.contains("turn: 180deg"));
    assert!(report.evaluated_css.contains("same: 1in"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_trig_builtin_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "@pi: pi(); @sin: sin(30deg); @sinRad: sin(1rad); @sinUnitless: sin(1); @cos: cos(60deg); @tan: tan(45deg); @asin: asin(.5); @acos: acos(.5); @atan: atan(1); .button { pi: @pi; sin: @sin; sin-rad: @sinRad; sin-unitless: @sinUnitless; cos: @cos; tan: @tan; asin: @asin; acos: @acos; atan: @atan; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 9);
    assert_eq!(report.value_resolution.resolved_count, 9);
    assert_eq!(report.value_resolution.raw_count, 0);
    assert!(report.evaluated_css.contains("pi: 3.14159265"));
    assert!(report.evaluated_css.contains("sin: 0.5"));
    assert!(report.evaluated_css.contains("sin-rad: 0.84147098"));
    assert!(report.evaluated_css.contains("sin-unitless: 0.84147098"));
    assert!(report.evaluated_css.contains("cos: 0.5"));
    assert!(report.evaluated_css.contains("tan: 1"));
    assert!(report.evaluated_css.contains("asin: 0.52359878rad"));
    assert!(report.evaluated_css.contains("acos: 1.04719755rad"));
    assert!(report.evaluated_css.contains("atan: 0.78539816rad"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_percentage_and_rounding_builtin_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "@ratio: percentage(.5); @ceil: ceil(1.2px); @floor: floor(1.8px); .button { width: @ratio; top: @ceil; bottom: @floor; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 3);
    assert_eq!(report.resolved_replacements[0].text, "50%");
    assert_eq!(report.resolved_replacements[1].text, "2px");
    assert_eq!(report.resolved_replacements[2].text, "1px");
    assert!(
        report
            .resolved_replacements
            .iter()
            .all(|replacement| replacement.abstract_value_kind == "exact")
    );
    assert_eq!(report.value_resolution.resolved_count, 3);
    assert_eq!(report.value_resolution.raw_count, 0);
    assert!(report.evaluated_css.contains("width: 50%"));
    assert!(report.evaluated_css.contains("top: 2px"));
    assert!(report.evaluated_css.contains("bottom: 1px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_extended_numeric_builtin_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "@sqrt: sqrt(4); @pow: pow(2, 3); @mod: mod(11px, 4px); @min: min(1px, 2px, 3px); @max: max(1px, 2px, 3px); @abs: abs(-2.4px); @round1: round(1.6px); @round2: round(1.234px, 2); .button { sqrt: @sqrt; pow: @pow; mod: @mod; min: @min; max: @max; abs: @abs; round1: @round1; round2: @round2; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 8);
    assert_eq!(report.value_resolution.resolved_count, 8);
    assert_eq!(report.value_resolution.raw_count, 0);
    assert!(report.evaluated_css.contains("sqrt: 2"));
    assert!(report.evaluated_css.contains("pow: 8"));
    assert!(report.evaluated_css.contains("mod: 3px"));
    assert!(report.evaluated_css.contains("min: 1px"));
    assert!(report.evaluated_css.contains("max: 3px"));
    assert!(report.evaluated_css.contains("abs: 2.4px"));
    assert!(report.evaluated_css.contains("round1: 2px"));
    assert!(report.evaluated_css.contains("round2: 1.23px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_preserves_unsupported_css_math_functions() {
    let report = derive_static_stylesheet_module_evaluation(
        "@sign: sign(-2px); @clamp: clamp(1px, 3px, 2px); @rem: rem(11px, 4px); @hypot: hypot(3px, 4px); @exp: exp(1); @log: log(8, 2); @calc: calc(1px + 2px); .button { sign: @sign; clamp: @clamp; rem: @rem; hypot: @hypot; exp: @exp; log: @log; calc: @calc; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 7);
    assert_eq!(report.value_resolution.raw_count, 7);
    assert!(report.evaluated_css.contains("sign: sign(-2px)"));
    assert!(report.evaluated_css.contains("clamp: clamp(1px, 3px, 2px)"));
    assert!(report.evaluated_css.contains("rem: rem(11px, 4px)"));
    assert!(report.evaluated_css.contains("hypot: hypot(3px, 4px)"));
    assert!(report.evaluated_css.contains("exp: exp(1)"));
    assert!(report.evaluated_css.contains("log: log(8, 2)"));
    assert!(report.evaluated_css.contains("calc: calc(1px + 2px)"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_type_predicate_builtin_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "@number: isnumber(2px); @color: iscolor(red); @string: isstring(\"Roboto\"); @keyword: iskeyword(block); @url: isurl(url(\"a.png\")); @defined: isdefined(@color); @missing: isdefined(@absent); @literal: isdefined(red); @future-defined: isdefined(@future); @future: blue; @px: ispixel(2px); @pct: ispercentage(50%); @em: isem(1em); @unit-ok: isunit(1rem, rem); @unit-bad: isunit(1rem, px); .button { --number: @number; --color: @color; --string: @string; --keyword: @keyword; --url: @url; --defined: @defined; --missing: @missing; --literal: @literal; --future-defined: @future-defined; --px: @px; --pct: @pct; --em: @em; --unit-ok: @unit-ok; --unit-bad: @unit-bad; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 14);
    assert_eq!(report.value_resolution.resolved_count, 0);
    assert_eq!(report.value_resolution.raw_count, 14);
    assert!(
        report
            .resolved_replacements
            .iter()
            .all(|replacement| replacement.abstract_value_kind == "raw")
    );
    assert!(report.evaluated_css.contains("--number: true"));
    assert!(report.evaluated_css.contains("--defined: true"));
    assert!(report.evaluated_css.contains("--missing: false"));
    assert!(report.evaluated_css.contains("--literal: true"));
    assert!(report.evaluated_css.contains("--future-defined: true"));
    assert!(report.evaluated_css.contains("--unit-ok: true"));
    assert!(report.evaluated_css.contains("--unit-bad: false"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_property_isdefined_builtin_values() {
    let report = derive_static_stylesheet_module_evaluation(
        ".button { color: red; @has-color: isdefined($color); @missing-prop: isdefined($missing); has: @has-color; missing: @missing-prop; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 2);
    assert_eq!(report.value_resolution.raw_count, 2);
    assert_eq!(report.value_resolution.top_count, 0);
    assert!(
        report
            .resolved_replacements
            .iter()
            .all(|replacement| replacement.abstract_value_kind == "raw")
    );
    assert!(report.evaluated_css.contains("color: red"));
    assert!(report.evaluated_css.contains("has: true"));
    assert!(report.evaluated_css.contains("missing: false"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_isruleset_builtin_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "@rules: { color: red; }; @ok: isruleset(@rules); @bad: isruleset(red); .button { ok: @ok; bad: @bad; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 2);
    assert_eq!(report.value_resolution.raw_count, 2);
    assert_eq!(report.value_resolution.top_count, 0);
    assert!(
        report
            .resolved_replacements
            .iter()
            .all(|replacement| replacement.abstract_value_kind == "raw")
    );
    assert!(report.evaluated_css.contains("ok: true"));
    assert!(report.evaluated_css.contains("bad: false"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_conditional_builtin_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "@gap: 1; @a: if(@gap > 0, red, blue); @b: if(false, red, blue); @c: if(isnumber(2px), yes, no); @d: boolean(@gap > 0); @e: if(default(), red, blue); .button { a: @a; b: @b; c: @c; d: @d; e: @e; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 5);
    assert_eq!(report.value_resolution.resolved_count, 3);
    assert_eq!(report.value_resolution.raw_count, 2);
    assert!(report.evaluated_css.contains("a: red"));
    assert!(report.evaluated_css.contains("b: blue"));
    assert!(report.evaluated_css.contains("c: yes"));
    assert!(report.evaluated_css.contains("d: true"));
    assert!(report.evaluated_css.contains("e: blue"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_color_channel_builtin_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "@color: #123456; @r: red(@color); @g: green(@color); @b: blue(@color); @a: alpha(rgba(10, 20, 30, .5)); .button { r: @r; g: @g; b: @b; a: @a; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 4);
    assert_eq!(report.value_resolution.resolved_count, 4);
    assert_eq!(report.value_resolution.raw_count, 0);
    assert!(report.evaluated_css.contains("r: 18"));
    assert!(report.evaluated_css.contains("g: 52"));
    assert!(report.evaluated_css.contains("b: 86"));
    assert!(report.evaluated_css.contains("a: 0.5"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_rgb_color_constructor_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "@rgb: rgb(18, 52, 86); @rgba: rgba(18, 52, 86, .5); @pct: rgba(100%, 0%, 0%, 50%); @slash: rgb(18 52 86 / .5); .button { color: @rgb; background: @rgba; border-color: @pct; outline-color: @slash; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 4);
    assert_eq!(report.value_resolution.resolved_count, 4);
    assert_eq!(report.value_resolution.raw_count, 0);
    assert!(report.evaluated_css.contains("color: #123456"));
    assert!(
        report
            .evaluated_css
            .contains("background: rgba(18, 52, 86, 0.5)")
    );
    assert!(
        report
            .evaluated_css
            .contains("border-color: rgba(255, 0, 0, 0.5)")
    );
    assert!(
        report
            .evaluated_css
            .contains("outline-color: rgba(18, 52, 86, 0.5)")
    );
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_color_metadata_builtin_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "@color: #123456; @h: hue(@color); @s: saturation(@color); @l: lightness(@color); @legacy: argb(rgba(18, 52, 86, .5)); .button { h: @h; s: @s; l: @l; legacy: @legacy; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 4);
    assert_eq!(report.value_resolution.resolved_count, 4);
    assert_eq!(report.value_resolution.raw_count, 0);
    assert!(report.evaluated_css.contains("h: 210"));
    assert!(report.evaluated_css.contains("s: 65.38461538%"));
    assert!(report.evaluated_css.contains("l: 20.39215686%"));
    assert!(report.evaluated_css.contains("legacy: #80123456"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_hsv_color_metadata_builtin_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "@hsv: hsv(210, 60%, 40%); @hsvUnitless: hsv(60, .6, .4); @hsva: hsva(210, 60%, 40%, 50%); @color: #123456; @h: hsvhue(@color); @s: hsvsaturation(@color); @v: hsvvalue(@color); @luma: luma(rgba(18, 52, 86, .5)); @lum: luminance(rgba(18, 52, 86, .5)); .button { hsv: @hsv; hsv-unitless: @hsvUnitless; hsva: @hsva; h: @h; s: @s; v: @v; luma: @luma; luminance: @lum; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 8);
    assert_eq!(report.value_resolution.resolved_count, 8);
    assert_eq!(report.value_resolution.raw_count, 0);
    assert!(report.evaluated_css.contains("hsv: #294766"));
    assert!(report.evaluated_css.contains("hsv-unitless: #666629"));
    assert!(
        report
            .evaluated_css
            .contains("hsva: rgba(41, 71, 102, 0.5)")
    );
    assert!(report.evaluated_css.contains("h: 210"));
    assert!(report.evaluated_css.contains("s: 79.06976744%"));
    assert!(report.evaluated_css.contains("v: 33.7254902%"));
    assert!(report.evaluated_css.contains("luma: 1.62823344%"));
    assert!(report.evaluated_css.contains("luminance: 9.26007843%"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_contrast_and_color_builtin_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "@dark: contrast(#123456); @light: contrast(#eeeeee); @custom: contrast(#123456, #111111, #eeeeee); @threshold: contrast(#888888, #111111, #eeeeee, 60%); @hex: color(\"#123456\"); @short: color(\"#abc\"); @alpha: color(\"#12345680\"); @kw: color(red); .button { dark: @dark; light: @light; custom: @custom; threshold: @threshold; hex: @hex; short: @short; alpha: @alpha; kw: @kw; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 8);
    assert_eq!(report.value_resolution.resolved_count, 8);
    assert_eq!(report.value_resolution.raw_count, 0);
    assert!(report.evaluated_css.contains("dark: #ffffff"));
    assert!(report.evaluated_css.contains("light: #000000"));
    assert!(report.evaluated_css.contains("custom: #eeeeee"));
    assert!(report.evaluated_css.contains("threshold: #eeeeee"));
    assert!(report.evaluated_css.contains("hex: #123456"));
    assert!(report.evaluated_css.contains("short: #abc"));
    assert!(report.evaluated_css.contains("alpha: #12345680"));
    assert!(report.evaluated_css.contains("kw: #ff0000"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_alpha_transform_builtin_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "@faded: fade(#123456, 50%); @raised: fadein(rgba(18, 52, 86, .5), 10%); @lowered: fadeout(rgba(18, 52, 86, .5), 10%); @raisedRel: fadein(rgba(18, 52, 86, .5), 10%, relative); @loweredRel: fadeout(rgba(18, 52, 86, .5), 10%, relative); @opaque: fadein(red, 10%); .button { a: @faded; b: @raised; c: @lowered; d: @opaque; e: @raisedRel; f: @loweredRel; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 6);
    assert_eq!(report.value_resolution.resolved_count, 6);
    assert_eq!(report.value_resolution.raw_count, 0);
    assert!(report.evaluated_css.contains("a: rgba(18, 52, 86, 0.5)"));
    assert!(report.evaluated_css.contains("b: rgba(18, 52, 86, 0.6)"));
    assert!(report.evaluated_css.contains("c: rgba(18, 52, 86, 0.4)"));
    assert!(report.evaluated_css.contains("d: #ff0000"));
    assert!(report.evaluated_css.contains("e: rgba(18, 52, 86, 0.55)"));
    assert!(report.evaluated_css.contains("f: rgba(18, 52, 86, 0.45)"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_hsl_color_transform_builtin_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "@light: lighten(#123456, 10%); @dark: darken(#123456, 10%); @sat: saturate(#123456, 10%); @desat: desaturate(#123456, 10%); @lightRel: lighten(#123456, 10%, relative); @darkRel: darken(#123456, 10%, relative); @satRel: saturate(#123456, 10%, relative); @desatRel: desaturate(#123456, 10%, relative); @spin: spin(#123456, 10); @gray: greyscale(#123456); @alpha: lighten(rgba(18, 52, 86, .5), 10%); .button { light: @light; dark: @dark; sat: @sat; desat: @desat; light-rel: @lightRel; dark-rel: @darkRel; sat-rel: @satRel; desat-rel: @desatRel; spin: @spin; gray: @gray; alpha: @alpha; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 11);
    assert_eq!(report.value_resolution.resolved_count, 11);
    assert_eq!(report.value_resolution.raw_count, 0);
    assert!(report.evaluated_css.contains("light: #1b4d80"));
    assert!(report.evaluated_css.contains("dark: #091a2c"));
    assert!(report.evaluated_css.contains("sat: #0d345b"));
    assert!(report.evaluated_css.contains("desat: #173451"));
    assert!(report.evaluated_css.contains("light-rel: #14395f"));
    assert!(report.evaluated_css.contains("dark-rel: #102f4d"));
    assert!(report.evaluated_css.contains("sat-rel: #0f3459"));
    assert!(report.evaluated_css.contains("desat-rel: #153453"));
    assert!(report.evaluated_css.contains("spin: #122956"));
    assert!(report.evaluated_css.contains("gray: #343434"));
    assert!(
        report
            .evaluated_css
            .contains("alpha: rgba(27, 77, 128, 0.5)")
    );
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_color_mix_builtin_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "@default: mix(red, blue); @weighted: mix(red, blue, 25%); @tinted: tint(#123456, 10%); @shaded: shade(#123456, 10%); @alpha: mix(rgba(255, 0, 0, .5), blue, 50%); @transparent: mix(transparent, red, 50%); .button { default: @default; weighted: @weighted; tinted: @tinted; shaded: @shaded; alpha: @alpha; transparent: @transparent; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 6);
    assert_eq!(report.value_resolution.resolved_count, 6);
    assert_eq!(report.value_resolution.raw_count, 0);
    assert!(report.evaluated_css.contains("default: #800080"));
    assert!(report.evaluated_css.contains("weighted: #4000bf"));
    assert!(report.evaluated_css.contains("tinted: #2a4867"));
    assert!(report.evaluated_css.contains("shaded: #102f4d"));
    assert!(
        report
            .evaluated_css
            .contains("alpha: rgba(64, 0, 191, 0.75)")
    );
    assert!(
        report
            .evaluated_css
            .contains("transparent: rgba(255, 0, 0, 0.5)")
    );
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_color_blend_builtin_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "@multiply: multiply(red, blue); @screen: screen(red, blue); @overlay: overlay(#123456, #abcdef); @softlight: softlight(#123456, #abcdef); @hardlight: hardlight(#123456, #abcdef); @difference: difference(#123456, #abcdef); @exclusion: exclusion(#123456, #abcdef); @average: average(#123456, #abcdef); @negation: negation(#123456, #abcdef); .button { multiply: @multiply; screen: @screen; overlay: @overlay; softlight: @softlight; hardlight: @hardlight; difference: @difference; exclusion: @exclusion; average: @average; negation: @negation; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 9);
    assert_eq!(report.value_resolution.resolved_count, 9);
    assert_eq!(report.value_resolution.raw_count, 0);
    assert!(report.evaluated_css.contains("multiply: #000000"));
    assert!(report.evaluated_css.contains("screen: #ff00ff"));
    assert!(report.evaluated_css.contains("overlay: #1854a1"));
    assert!(report.evaluated_css.contains("softlight: #205b8c"));
    assert!(report.evaluated_css.contains("hardlight: #63afea"));
    assert!(report.evaluated_css.contains("difference: #999999"));
    assert!(report.evaluated_css.contains("exclusion: #a5ada4"));
    assert!(report.evaluated_css.contains("average: #5f81a3"));
    assert!(report.evaluated_css.contains("negation: #bdfdb9"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_alpha_color_blend_builtin_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "@multiply: multiply(rgba(255, 102, 0, .5), #0000ff); @screen: screen(rgba(255, 102, 0, .5), rgba(0, 0, 255, .25)); @overlay: overlay(rgba(255, 102, 0, .5), rgba(0, 0, 255, .25)); @softlight: softlight(rgba(255, 102, 0, .5), rgba(0, 0, 255, .25)); @hardlight: hardlight(rgba(255, 102, 0, .5), rgba(0, 0, 255, .25)); @difference: difference(rgba(255, 102, 0, .5), rgba(0, 0, 255, .25)); @exclusion: exclusion(rgba(255, 102, 0, .5), rgba(0, 0, 255, .25)); @average: average(rgba(255, 102, 0, .5), rgba(0, 0, 255, .25)); @negation: negation(rgba(255, 102, 0, .5), rgba(0, 0, 255, .25)); @both: multiply(transparent, transparent); @transparent: multiply(transparent, #0000ff); @sourceTransparent: screen(#ff6600, transparent); @transparentAverage: average(transparent, #ff6600); .button { multiply: @multiply; screen: @screen; overlay: @overlay; softlight: @softlight; hardlight: @hardlight; difference: @difference; exclusion: @exclusion; average: @average; negation: @negation; both: @both; transparent: @transparent; source-transparent: @sourceTransparent; transparent-average: @transparentAverage; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 13);
    assert_eq!(report.value_resolution.resolved_count, 13);
    assert_eq!(report.value_resolution.raw_count, 0);
    assert!(report.evaluated_css.contains("multiply: #000080"));
    assert!(
        report
            .evaluated_css
            .contains("screen: rgba(204, 82, 102, 0.625)")
    );
    assert!(
        report
            .evaluated_css
            .contains("overlay: rgba(204, 61, 51, 0.625)")
    );
    assert!(
        report
            .evaluated_css
            .contains("softlight: rgba(204, 69, 51, 0.625)")
    );
    assert!(
        report
            .evaluated_css
            .contains("hardlight: rgba(153, 61, 102, 0.625)")
    );
    assert!(
        report
            .evaluated_css
            .contains("difference: rgba(204, 82, 102, 0.625)")
    );
    assert!(
        report
            .evaluated_css
            .contains("exclusion: rgba(204, 82, 102, 0.625)")
    );
    assert!(
        report
            .evaluated_css
            .contains("average: rgba(179, 71, 77, 0.625)")
    );
    assert!(
        report
            .evaluated_css
            .contains("negation: rgba(204, 82, 102, 0.625)")
    );
    assert!(report.evaluated_css.contains("both: rgba(0, 0, 0, 0)"));
    assert!(report.evaluated_css.contains("transparent: #0000ff"));
    assert!(report.evaluated_css.contains("source-transparent: #ff6600"));
    assert!(
        report
            .evaluated_css
            .contains("transparent-average: #ff6600")
    );
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_list_builtin_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "@items: a b c; @comma: a, b, c; @len1: length(@items); @len2: length(@comma); @x1: extract(@items, 2); @x2: extract(@comma, 3); .button { len1: @len1; len2: @len2; x1: @x1; x2: @x2; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 4);
    assert_eq!(report.value_resolution.resolved_count, 2);
    assert_eq!(report.value_resolution.raw_count, 2);
    assert!(report.evaluated_css.contains("len1: 3"));
    assert!(report.evaluated_css.contains("len2: 3"));
    assert!(report.evaluated_css.contains("x1: b"));
    assert!(report.evaluated_css.contains("x2: c"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_range_builtin_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "@items: range(4); @gaps: range(1px, 5px, 2); @half: range(1, 2, .5); @empty: range(3, 1); .button { a: @items; b: @gaps; c: @half; d: @empty; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 4);
    assert_eq!(report.value_resolution.raw_count, 3);
    assert!(report.evaluated_css.contains("a: 1 2 3 4"));
    assert!(report.evaluated_css.contains("b: 1px 3px 5px"));
    assert!(report.evaluated_css.contains("c: 1 1.5 2"));
    assert!(report.evaluated_css.contains("d: ;"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_replace_builtin_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "@name: replace(\"hello world\", \"world\", \"less\"); @first: replace(\"hello\", \"l\", \"L\"); @all: replace(\"hello\", \"l\", \"L\", \"g\"); @fold: replace(\"ABCabc\", \"abc\", \"x\", \"gi\"); @bare: replace(hello, l, X); .button { name: @name; first: @first; all: @all; fold: @fold; bare: @bare; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 5);
    assert_eq!(report.value_resolution.raw_count, 5);
    assert!(report.evaluated_css.contains("name: \"hello less\""));
    assert!(report.evaluated_css.contains("first: \"heLlo\""));
    assert!(report.evaluated_css.contains("all: \"heLLo\""));
    assert!(report.evaluated_css.contains("fold: \"xx\""));
    assert!(report.evaluated_css.contains("bare: heXlo"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_format_builtin_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "@name: %(\"hello %s\", \"less\"); @num: %(\"%dpx\", 12); @encoded: %(\"%S\", \"x y\"); @literal: %(\"%% done\"); @missing: %(\"%s %s\", alpha); @extra: %(\"%s\", beta, ignored); @escaped: %(~\"hello-%s\", less); .button { name: @name; num: @num; encoded: @encoded; literal: @literal; missing: @missing; extra: @extra; escaped: @escaped; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 7);
    assert_eq!(report.value_resolution.raw_count, 7);
    assert!(report.evaluated_css.contains("name: \"hello less\""));
    assert!(report.evaluated_css.contains("num: \"12px\""));
    assert!(report.evaluated_css.contains("encoded: \"x%20y\""));
    assert!(report.evaluated_css.contains("literal: \"% done\""));
    assert!(report.evaluated_css.contains("missing: \"alpha %s\""));
    assert!(report.evaluated_css.contains("extra: \"beta\""));
    assert!(report.evaluated_css.contains("escaped: hello-less"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_keeps_regex_replace_patterns_raw() {
    let report = derive_static_stylesheet_module_evaluation(
        "@rx: replace(\"abc123\", \"[0-9]+\", \"#\"); .button { rx: @rx; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.value_resolution.raw_count, 1);
    assert!(
        report
            .evaluated_css
            .contains("rx: replace(\"abc123\", \"[0-9]+\", \"#\")")
    );
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_keeps_out_of_range_extract_raw() {
    let report = derive_static_stylesheet_module_evaluation(
        "@items: a b c; @bad: extract(@items, 4); .button { bad: @bad; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.value_resolution.raw_count, 1);
    assert!(report.evaluated_css.contains("bad: extract(a b c, 4)"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_property_variable_numeric_values() {
    let report = derive_static_stylesheet_module_evaluation(
        ".button { margin: (1px + 2px); padding: $margin; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "$margin");
    assert_eq!(report.resolved_replacements[0].text, "3px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert_eq!(
        report.value_resolution.values[0].rendered_value.as_deref(),
        Some("3px")
    );
    assert!(report.evaluated_css.contains("padding: 3px"));
}

#[test]
fn static_less_evaluation_reduces_property_variable_alias_values() {
    let report = derive_static_stylesheet_module_evaluation(
        ".button { margin: (1px + 2px); @gap: $margin; padding: @gap; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "@gap");
    assert_eq!(report.resolved_replacements[0].text, "3px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert_eq!(report.value_resolution.values[0].name, "@gap");
    assert_eq!(
        report.value_resolution.values[0].rendered_value.as_deref(),
        Some("3px")
    );
    assert!(report.evaluated_css.contains("padding: 3px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_property_variable_composite_alias_values() {
    let report = derive_static_stylesheet_module_evaluation(
        ".button { color: red; @outline: 1px solid $color; border: @outline; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "@outline");
    assert_eq!(report.resolved_replacements[0].text, "1px solid red");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "raw");
    assert_eq!(
        report.value_resolution.values[0].rendered_value.as_deref(),
        Some("1px solid red")
    );
    assert!(report.evaluated_css.contains("border: 1px solid red"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_less_evaluation_reduces_property_variable_escaped_string_values() {
    let report = derive_static_stylesheet_module_evaluation(
        ".button { filter: ~\"alpha(opacity=50)\"; background: $filter; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "$filter");
    assert_eq!(report.resolved_replacements[0].text, "alpha(opacity=50)");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "raw");
    assert_eq!(
        report.value_resolution.values[0].rendered_value.as_deref(),
        Some("alpha(opacity=50)")
    );
    assert!(
        report
            .evaluated_css
            .contains("background: alpha(opacity=50)")
    );
    assert!(!report.evaluated_css.contains("~\"alpha"));
}

#[test]
fn static_value_resolution_keeps_irreducible_numeric_functions_raw() {
    let report = summarize_static_stylesheet_value_resolution(
        "$gap: min(1px, 2rem); .button { margin: $gap; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.reference_count, 1);
    assert_eq!(report.raw_count, 1);
    assert_eq!(report.unsupported_dynamic_count, 1);
    assert_eq!(report.values[0].outcome, "raw");
    assert_eq!(report.values[0].reason, "unsupportedDynamic");
    assert_eq!(
        report.values[0].rendered_value.as_deref(),
        Some("min(1px, 2rem)")
    );
}

#[test]
fn static_scss_evaluation_resolves_same_file_function_calls() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function gap($value) { @return $value; } .button { margin: gap(0px); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:gap");
    assert_eq!(report.resolved_replacements[0].text, "0px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(!report.evaluated_css.contains("@function"));
    assert!(report.evaluated_css.contains(".button { margin: 0px; }"));
    assert_eq!(report.value_resolution.reference_count, 1);
    assert_eq!(report.value_resolution.resolved_count, 1);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_function_numeric_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function double($value) { @return ($value + $value); } .button { margin: double(2px); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:double");
    assert_eq!(report.resolved_replacements[0].text, "4px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 4px; }"));
    assert_eq!(
        report.value_resolution.values[0].rendered_value.as_deref(),
        Some("4px")
    );
}

#[test]
fn static_scss_evaluation_resolves_named_function_arguments() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function pair($left, $right) { @return $left + $right; } .button { margin: pair($right: 2px, $left: 1px); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:pair");
    assert_eq!(report.resolved_replacements[0].text, "3px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 3px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_rejects_positional_arguments_after_named_arguments() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function pair($left, $right) { @return $left + $right; } .button { margin: pair($left: 1px, 2px); }",
        StyleDialect::Scss,
    );
    assert!(report.is_none());

    let resolution = summarize_static_stylesheet_value_resolution(
        "@function pair($left, $right) { @return $left + $right; } .button { margin: pair($left: 1px, 2px); }",
        StyleDialect::Scss,
    );
    assert!(resolution.is_some());
    let Some(resolution) = resolution else {
        return;
    };

    assert_eq!(resolution.raw_count, 1);
    assert_eq!(resolution.unsupported_dynamic_count, 1);
}

#[test]
fn static_scss_evaluation_resolves_function_default_arguments() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function offset($value: 1px, $extra: 2px) { @return $value + $extra; } .button { margin: offset(); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:offset");
    assert_eq!(report.resolved_replacements[0].text, "3px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(!report.evaluated_css.contains("@function"));
    assert!(report.evaluated_css.contains(".button { margin: 3px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_resolves_named_arguments_with_default_tail() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function pair($left, $right: 2px) { @return $left + $right; } .button { margin: pair($left: 1px); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:pair");
    assert_eq!(report.resolved_replacements[0].text, "3px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 3px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_resolves_default_arguments_from_prior_parameters() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function offset($value, $extra: $value + 1px) { @return $extra; } .button { margin: offset(2px); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:offset");
    assert_eq!(report.resolved_replacements[0].text, "3px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 3px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_resolves_named_default_arguments_from_prior_parameters() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function offset($value, $extra: $value + 1px) { @return $extra; } .button { margin: offset($value: 2px); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:offset");
    assert_eq!(report.resolved_replacements[0].text, "3px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 3px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_resolves_mixin_default_arguments_from_prior_parameters() {
    let report = derive_static_stylesheet_module_evaluation(
        "@mixin tone($color, $border: $color) { color: $color; border-color: $border; } .button { @include tone(blue); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@mixin"));
    assert!(report.evaluated_css.contains("color: blue;"));
    assert!(report.evaluated_css.contains("border-color: blue;"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_scss_evaluation_resolves_named_mixin_default_arguments_from_prior_parameters() {
    let report = derive_static_stylesheet_module_evaluation(
        "@mixin tone($color, $border: $color) { color: $color; border-color: $border; } .button { @include tone($color: blue); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@mixin"));
    assert!(report.evaluated_css.contains("color: blue;"));
    assert!(report.evaluated_css.contains("border-color: blue;"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_scss_evaluation_expands_mixin_content_blocks() {
    let report = derive_static_stylesheet_module_evaluation(
        "@mixin tone($color) { @content; color: $color; } .button { @include tone(red) { background: white; } }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@mixin"));
    assert!(!report.evaluated_css.contains("@include"));
    assert!(!report.evaluated_css.contains("@content"));
    assert!(report.evaluated_css.contains("background: white;"));
    assert!(report.evaluated_css.contains("color: red;"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_scss_evaluation_expands_mixin_content_arguments() {
    let report = derive_static_stylesheet_module_evaluation(
        "@mixin apply($color) { @content($color); } .button { @include apply(red) using ($tone) { color: $tone; } }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@mixin"));
    assert!(!report.evaluated_css.contains("@include"));
    assert!(!report.evaluated_css.contains("@content"));
    assert!(report.evaluated_css.contains("color: red;"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_sass_evaluation_expands_mixin_content_arguments() {
    let report = derive_static_stylesheet_module_evaluation(
        "@mixin apply($color)\n  @content($color)\n.button\n  @include apply(red) using ($tone)\n    color: $tone\n",
        StyleDialect::Sass,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@mixin"));
    assert!(!report.evaluated_css.contains("@include"));
    assert!(!report.evaluated_css.contains("@content"));
    assert!(report.evaluated_css.contains("color: red"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_scss_evaluation_expands_mixin_content_expression_arguments() {
    let report = derive_static_stylesheet_module_evaluation(
        "@mixin apply($color, $gap) { @content($color, $gap + 1px); } .button { @include apply(red, 1px) using ($tone, $space) { color: $tone; margin: $space; } }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@mixin"));
    assert!(!report.evaluated_css.contains("@include"));
    assert!(!report.evaluated_css.contains("@content"));
    assert!(report.evaluated_css.contains("color: red;"));
    assert!(report.evaluated_css.contains("margin: 2px;"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_sass_evaluation_expands_mixin_content_expression_arguments() {
    let report = derive_static_stylesheet_module_evaluation(
        "@mixin apply($color, $gap)\n  @content($color, $gap + 1px)\n.button\n  @include apply(red, 1px) using ($tone, $space)\n    color: $tone\n    margin: $space\n",
        StyleDialect::Sass,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@mixin"));
    assert!(!report.evaluated_css.contains("@include"));
    assert!(!report.evaluated_css.contains("@content"));
    assert!(report.evaluated_css.contains("color: red"));
    assert!(report.evaluated_css.contains("margin: 2px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_scss_evaluation_expands_mixin_content_nested_includes() {
    let report = derive_static_stylesheet_module_evaluation(
        "@mixin spacing($gap) { margin: $gap; } @mixin apply($gap) { @content($gap); color: blue; } .button { @include apply(2px) using ($space) { @include spacing($space); background: white; } }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@mixin"));
    assert!(!report.evaluated_css.contains("@include"));
    assert!(!report.evaluated_css.contains("@content"));
    assert!(report.evaluated_css.contains("margin: 2px"));
    assert!(report.evaluated_css.contains("background: white"));
    assert!(report.evaluated_css.contains("color: blue"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_scss_evaluation_preserves_content_argument_arity_mismatch_as_raw() {
    let source = "@mixin apply($color) { @content($color); } .button { @include apply(red) { color: red; } }";
    let report = derive_static_stylesheet_module_evaluation(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.evaluated_css, source);
    assert!(report.evaluated_css.contains("@include apply"));
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_sass_evaluation_expands_mixin_content_blocks() {
    let report = derive_static_stylesheet_module_evaluation(
        "@mixin tone($color)\n  @content\n  color: $color\n.button\n  @include tone(red)\n    background: white\n",
        StyleDialect::Sass,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@mixin"));
    assert!(!report.evaluated_css.contains("@include"));
    assert!(!report.evaluated_css.contains("@content"));
    assert!(report.evaluated_css.contains("background: white"));
    assert!(report.evaluated_css.contains("color: red"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_scss_evaluation_preserves_nested_mixin_content_blocks_as_raw() {
    let source = "@mixin wrap { @content; } .button { @include wrap { .inner { color: red; } } }";
    let report = derive_static_stylesheet_module_evaluation(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.evaluated_css, source);
    assert!(report.evaluated_css.contains("@include wrap"));
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_sass_evaluation_preserves_nested_mixin_content_blocks_as_raw() {
    let source =
        "@mixin wrap\n  @content\n.button\n  @include wrap\n    .inner\n      color: red\n";
    let report = derive_static_stylesheet_module_evaluation(source, StyleDialect::Sass);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.evaluated_css, source);
    assert!(report.evaluated_css.contains("@include wrap"));
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_sass_evaluation_expands_mixin_content_nested_includes() {
    let report = derive_static_stylesheet_module_evaluation(
        "@mixin spacing($gap)\n  margin: $gap\n@mixin apply($gap)\n  @content($gap)\n  color: blue\n.button\n  @include apply(2px) using ($space)\n    @include spacing($space)\n    background: white\n",
        StyleDialect::Sass,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@mixin"));
    assert!(!report.evaluated_css.contains("@include"));
    assert!(!report.evaluated_css.contains("@content"));
    assert!(report.evaluated_css.contains("margin: 2px"));
    assert!(report.evaluated_css.contains("background: white"));
    assert!(report.evaluated_css.contains("color: blue"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_scss_evaluation_reduces_static_if_function_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function choose($condition) { @return if($condition, 1px, 2px) + 1px; } .button { margin: choose(true); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:choose");
    assert_eq!(report.resolved_replacements[0].text, "2px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 2px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_function_if_not_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function choose($condition) { @return if(not $condition, 1px, 2px); } .button { margin: choose(true); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:choose");
    assert_eq!(report.resolved_replacements[0].text, "2px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 2px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_function_boolean_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function choose($condition) { @return if($condition and false, 1px, 2px); } .button { margin: choose(true); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:choose");
    assert_eq!(report.resolved_replacements[0].text, "2px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 2px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_function_equality_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function choose($value) { @return if($value == 2px, 1px, 2px); } .button { margin: choose(2px); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:choose");
    assert_eq!(report.resolved_replacements[0].text, "1px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 1px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_function_inequality_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function choose($value) { @return if($value != 2px, 1px, 2px); } .button { margin: choose(3px); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:choose");
    assert_eq!(report.resolved_replacements[0].text, "1px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 1px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_function_numeric_ordering_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function choose($value) { @return if($value <= 2px, 1px, 2px); } .button { margin: choose(2px); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:choose");
    assert_eq!(report.resolved_replacements[0].text, "1px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 1px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_ignores_inactive_if_branch_callables() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function choose() { @return if(false, min(1px, 2px), 3px) + 1px; } .button { margin: choose(); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:choose");
    assert_eq!(report.resolved_replacements[0].text, "4px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 4px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_function_bare_numeric_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function double($value) { @return $value * 2; } .button { margin: double(2px); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:double");
    assert_eq!(report.resolved_replacements[0].text, "4px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 4px; }"));
    assert_eq!(
        report.value_resolution.values[0].rendered_value.as_deref(),
        Some("4px")
    );
}

#[test]
fn static_scss_evaluation_resolves_function_local_variables() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function offset($base) { $next: $base + 1px; @return $next + 1px; } .button { margin: offset(2px); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:offset");
    assert_eq!(report.resolved_replacements[0].text, "4px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 4px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_resolves_function_local_variable_chains() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function scale($base) { $next: $base + 1px; $double: $next * 2; @return $double; } .button { margin: scale(2px); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:scale");
    assert_eq!(report.resolved_replacements[0].text, "6px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 6px; }"));
    assert_eq!(
        report.value_resolution.values[0].rendered_value.as_deref(),
        Some("6px")
    );
}

#[test]
fn static_scss_evaluation_resolves_local_variables_after_prior_branch() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function pick($enabled) { @if $enabled { @return 3px; } $after: 1px + 1px; @return $after; } .button { margin: pick(false); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:pick");
    assert_eq!(report.resolved_replacements[0].text, "2px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 2px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_resolves_branch_local_variables() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function pick($enabled) { @if $enabled { $inside: 1px + 1px; @return $inside; } @return 1px; } .button { margin: pick(true); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:pick");
    assert_eq!(report.resolved_replacements[0].text, "2px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 2px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_does_not_leak_sibling_branch_local_variables() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function pick($enabled) { @if $enabled { @return $other; } @else { $other: 1px; @return $other; } } .button { margin: pick(true); }",
        StyleDialect::Scss,
    );
    assert!(report.is_none());
}

#[test]
fn static_scss_evaluation_skips_future_local_variable_replacements() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function pick($enabled) { @if $enabled { @return $after; } $after: 1px; @return $after; } .button { margin: pick(true); }",
        StyleDialect::Scss,
    );
    assert!(report.is_none());
}

#[test]
fn static_scss_evaluation_ignores_future_unsafe_local_variables() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function pick($enabled) { @if $enabled { @return 2px; } $after: 1px !global; @return $after; } .button { margin: pick(true); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:pick");
    assert_eq!(report.resolved_replacements[0].text, "2px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 2px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_resolves_composed_same_file_function_calls() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function inc($value) { @return $value + 1px; } @function gap($value) { @return inc($value) + 1px; } .button { margin: gap(2px); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:gap");
    assert_eq!(report.resolved_replacements[0].text, "4px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(!report.evaluated_css.contains("@function"));
    assert!(report.evaluated_css.contains(".button { margin: 4px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_resolves_local_values_with_same_file_function_calls() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function inc($value) { @return $value + 1px; } @function gap($value) { $next: inc($value); @return $next + 1px; } .button { margin: gap(2px); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:gap");
    assert_eq!(report.resolved_replacements[0].text, "4px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(!report.evaluated_css.contains("@function"));
    assert!(report.evaluated_css.contains(".button { margin: 4px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_resolves_static_if_function_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function tone($enabled) { @if $enabled { @return red; } @return blue; } .button { color: tone(true); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:tone");
    assert_eq!(report.resolved_replacements[0].text, "red");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(!report.evaluated_css.contains("@function"));
    assert!(report.evaluated_css.contains(".button { color: red; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_resolves_static_else_function_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function tone($enabled) { @if $enabled { @return red; } @else { @return blue; } } .button { color: tone(false); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:tone");
    assert_eq!(report.resolved_replacements[0].text, "blue");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { color: blue; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_resolves_static_else_if_function_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function tone($first, $second) { @if $first { @return red; } @else if $second { @return green; } @else { @return blue; } } .button { color: tone(false, true); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:tone");
    assert_eq!(report.resolved_replacements[0].text, "green");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { color: green; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_resolves_static_for_loop_function_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function pick($target) { @for $i from 1 through 3 { @if $i == $target { @return $i + 1; } } @return 0; } .button { z-index: pick(2); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:pick");
    assert_eq!(report.resolved_replacements[0].text, "3");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { z-index: 3; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_resolves_descending_static_for_loop_function_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function pick($target) { @for $i from 3 through 1 { @if $i == $target { @return $i + 1; } } @return 0; } .button { z-index: pick(2); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:pick");
    assert_eq!(report.resolved_replacements[0].text, "3");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { z-index: 3; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_respects_descending_to_loop_exclusive_end() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function pick() { @for $i from 3 to 1 { @if $i == 1 { @return 9; } } @return 2; } .button { z-index: pick(); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:pick");
    assert_eq!(report.resolved_replacements[0].text, "2");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { z-index: 2; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_resolves_static_for_loop_expression_bounds() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function pick($target) { @for $i from 1 + 1 through 1 + 2 { @if $i == $target { @return $i; } } @return 0; } .button { z-index: pick(1); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:pick");
    assert_eq!(report.resolved_replacements[0].text, "0");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { z-index: 0; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_resolves_nested_static_for_loop_function_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function collect($target) { @for $i from 1 through 2 { @for $j from 1 through 2 { @if $i == $target { @return $i + $j; } } } @return 0; } .button { z-index: collect(2); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:collect");
    assert_eq!(report.resolved_replacements[0].text, "3");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { z-index: 3; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_continues_after_inactive_static_for_loop_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function pick($target) { @for $i from 1 through 3 { @if $i == $target { @return $i + 1px; } } @return 0px; } .button { margin: pick(4); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:pick");
    assert_eq!(report.resolved_replacements[0].text, "0px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 0px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_resolves_static_each_single_loop_function_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function first-tone() { @each $tone in red, blue { @return $tone; } } .button { color: first-tone(); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:first-tone");
    assert_eq!(report.resolved_replacements[0].text, "red");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { color: red; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_resolves_static_each_function_source_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function pick($target) { @each $item in list.append(1px 2px, 3px) { @if $item == $target { @return $item; } } @return 0px; } .button { margin: pick(3px); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:pick");
    assert_eq!(report.resolved_replacements[0].text, "3px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 3px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_resolves_static_each_tuple_function_source_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function width-for($target) { @each $width, $style in list.zip(1px 2px, solid dashed) { @if $style == $target { @return $width; } } @return 0px; } .button { margin: width-for(dashed); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:width-for");
    assert_eq!(report.resolved_replacements[0].text, "2px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 2px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_resolves_static_each_map_loop_function_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function tone($target) { @each $name, $tone in (primary: red, secondary: blue) { @if $name == $target { @return $tone; } } @return black; } .button { color: tone(secondary); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:tone");
    assert_eq!(report.resolved_replacements[0].text, "blue");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { color: blue; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_resolves_static_each_tuple_loop_function_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function icon-size($target) { $pairs: (save, 16px), (cancel, 24px); @each $icon, $size in $pairs { @if $icon == $target { @return $size; } } @return 0px; } .button { width: icon-size(cancel); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:icon-size");
    assert_eq!(report.resolved_replacements[0].text, "24px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { width: 24px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_resolves_static_while_loop_function_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function pick() { $i: 0; @while $i < 3 { @if $i == 2 { @return $i + 1; } $i: $i + 1; } @return 0; } .button { z-index: pick(); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:pick");
    assert_eq!(report.resolved_replacements[0].text, "3");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { z-index: 3; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_uses_arguments_in_static_while_loop_function_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function pick($target) { $i: 0; @while $i < 3 { @if $i == $target { @return $i + 1; } $i: $i + 1; } @return 0; } .button { z-index: pick(2); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:pick");
    assert_eq!(report.resolved_replacements[0].text, "3");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { z-index: 3; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_continues_after_inactive_static_while_loop_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function pick() { $i: 0; @while $i < 2 { @if $i == 5 { @return $i; } $i: $i + 1; } @return 9; } .button { z-index: pick(); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:pick");
    assert_eq!(report.resolved_replacements[0].text, "9");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { z-index: 9; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_resolves_static_while_cumulative_step_function_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function pick() { $i: 0; @while $i < 7 { @if $i == 3 { @return $i + 1; } $i: $i + 1; $i: $i + 2; } @return 9; } .button { z-index: pick(); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:pick");
    assert_eq!(report.resolved_replacements[0].text, "4");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { z-index: 4; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_resolves_static_while_inequality_operator_function_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function pick() { $i: 0; @while $i != 3 { @if $i == 2 { @return $i + 1; } $i: $i + 1; } @return 9; } .button { z-index: pick(); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:pick");
    assert_eq!(report.resolved_replacements[0].text, "3");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { z-index: 3; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_keeps_dynamic_if_function_returns_top() {
    let source = "@function tone($enabled) { @if $enabled { @return red; } @else { @return blue; } } .button { color: tone(var(--enabled)); }";
    let report = derive_static_stylesheet_module_evaluation(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.evaluated_css, source);
    assert_eq!(report.replacement_count, 0);
    assert_eq!(report.value_resolution.reference_count, 1);
    assert_eq!(report.value_resolution.top_count, 1);
    assert_eq!(report.value_resolution.unsupported_dynamic_count, 1);
    assert_eq!(report.value_resolution.values[0].outcome, "top");
    assert_eq!(
        report.value_resolution.values[0].reason,
        "unsupportedDynamic"
    );
    assert!(report.oracle.all_legacy_declaration_values_preserved);

    let resolution = summarize_static_stylesheet_value_resolution(source, StyleDialect::Scss);
    assert!(resolution.is_some());
    let Some(resolution) = resolution else {
        return;
    };

    assert_eq!(resolution.reference_count, 1);
    assert_eq!(resolution.top_count, 1);
    assert_eq!(resolution.unsupported_dynamic_count, 1);
    assert_eq!(resolution.values[0].outcome, "top");
    assert_eq!(resolution.values[0].reason, "unsupportedDynamic");
}

#[test]
fn static_scss_evaluation_preserves_indirect_recursive_function_calls_as_top() {
    let source = "@function a($value) { @return b($value); } @function b($value) { @return a($value); } .button { color: a(red); }";
    let report = derive_static_stylesheet_module_evaluation(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.evaluated_css, source);
    assert_eq!(report.replacement_count, 0);
    assert_eq!(report.value_resolution.reference_count, 1);
    assert_eq!(report.value_resolution.top_count, 1);
    assert_eq!(report.value_resolution.cycle_count, 1);
    assert_eq!(report.value_resolution.values[0].outcome, "top");
    assert_eq!(report.value_resolution.values[0].reason, "cycle");
    assert!(report.oracle.all_legacy_declaration_values_preserved);

    let resolution = summarize_static_stylesheet_value_resolution(source, StyleDialect::Scss);
    assert!(resolution.is_some());
    let Some(resolution) = resolution else {
        return;
    };

    assert_eq!(resolution.reference_count, 1);
    assert_eq!(resolution.top_count, 1);
    assert_eq!(resolution.cycle_count, 1);
    assert_eq!(resolution.values[0].outcome, "top");
    assert_eq!(resolution.values[0].reason, "cycle");
}

#[test]
fn static_scss_evaluation_preserves_recursive_function_calls_as_top() {
    let source = "@function loop($value) { @return loop($value); } .button { color: loop(red); }";
    let report = derive_static_stylesheet_module_evaluation(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.evaluated_css, source);
    assert_eq!(report.replacement_count, 0);
    assert_eq!(report.value_resolution.reference_count, 1);
    assert_eq!(report.value_resolution.top_count, 1);
    assert_eq!(report.value_resolution.cycle_count, 1);
    assert!(
        report
            .value_resolution
            .values
            .iter()
            .all(|value| value.outcome == "top" && value.reason == "cycle")
    );
    assert!(report.oracle.all_legacy_declaration_values_preserved);

    let resolution = summarize_static_stylesheet_value_resolution(source, StyleDialect::Scss);
    assert!(resolution.is_some());
    let Some(resolution) = resolution else {
        return;
    };

    assert_eq!(resolution.reference_count, 1);
    assert_eq!(resolution.top_count, 1);
    assert_eq!(resolution.cycle_count, 1);
    assert!(
        resolution
            .values
            .iter()
            .all(|value| value.outcome == "top" && value.reason == "cycle")
    );
}

#[test]
fn static_scss_evaluation_reduces_static_list_constructor_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "$items: list.append(1px 2px, 3px); $item-count: list.length($items); $third-item: list.nth($items, 3); $joined: list.join((red, blue), (green, yellow), $separator: comma); $joined-third: list.nth($joined, 3); $set: list.set-nth(4px 5px 6px, -1, 8px); $set-tail: list.nth($set, -1); $zipped: list.zip(1px 2px, solid dashed); $second-pair: list.nth($zipped, 2); .button { z-index: $item-count; margin: $third-item; color: $joined-third; padding: $set-tail; border: $second-pair; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(report.evaluated_css.contains("z-index: 3"));
    assert!(report.evaluated_css.contains("margin: 3px"));
    assert!(report.evaluated_css.contains("color: green"));
    assert!(report.evaluated_css.contains("padding: 8px"));
    assert!(report.evaluated_css.contains("border: 2px dashed"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_slash_list_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "$stroke: list.slash(1px, solid, red); $separator: list.separator($stroke); $middle: list.nth($stroke, 2); .button { font: $stroke; content: $separator; outline-style: $middle; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(report.evaluated_css.contains("font: 1px / solid / red"));
    assert!(report.evaluated_css.contains("content: \"slash\""));
    assert!(report.evaluated_css.contains("outline-style: solid"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_function_comparison_operands() {
    let report = derive_static_stylesheet_module_evaluation(
        "$stroke: list.slash(1px, solid, red); $kind: if(meta.type-of($stroke) == list and list.separator($stroke) == \"slash\" and hue(#808000) == 60deg, 1px, 2px); .button { margin: $kind; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(report.evaluated_css.contains("margin: 1px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_type_metadata_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "$gap: 2px; $tone: red; $transparent-tone: rgba($tone, .5); $mixed-tone: color.mix(red, blue); $red-channel: color.channel($mixed-tone, \"red\", $space: rgb); $legacy-red-channel: red($tone); $relative-tone: oklab(1 0 0); $items: 1px 2px; $config: (dense: true); $kind: if(meta.type-of($gap) == number and type-of($tone) == color and meta.type-of($transparent-tone) == color and meta.type-of($mixed-tone) == color and meta.type-of($red-channel) == number and meta.type-of($legacy-red-channel) == number and meta.type-of($relative-tone) == color and meta.type-of($items) == list and type-of($config) == map and feature-exists(\"at-error\") and meta.feature-exists(custom-property) and not meta.feature-exists(\"unknown\"), 1px, 2px); .button { margin: $kind; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(report.evaluated_css.contains("margin: 1px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_inspect_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "$tone: meta.inspect(red); $gap: inspect(2px); .button { color: $tone; margin: $gap; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(report.evaluated_css.contains("color: red"));
    assert!(report.evaluated_css.contains("margin: 2px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_calculation_metadata_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "$name: meta.calc-name(clamp(1px, 2px, 3px)); $args: meta.calc-args(clamp(1px, 2px, 3px)); $kind: meta.type-of(calc(100% - 1px)); $gap: if($name == \"clamp\" and $kind == calculation and list.length($args) == 3 and list.nth($args, 2) == 2px, 1px, 2px); .button { margin: $gap; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(report.evaluated_css.contains("margin: 1px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_function_metadata_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function present() { @return 1px; } @function gate() { @return if(meta.function-exists(\"present\") and function-exists(\"scale-color\") and function-exists(\"hue\") and not function-exists(\"not-defined-here\"), present(), 2px); } .button { margin: gate(); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(report.evaluated_css.contains("margin: 1px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_preserves_function_exists_declaration_order() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function gate() { @return if(function-exists(\"later\"), 2px, 1px); } @function later() { @return 2px; } .button { margin: gate(); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(report.evaluated_css.contains("margin: 1px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_variable_metadata_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "$global-gap: 1px; $kind: if(variable-exists(\"global-gap\") and meta.global-variable-exists(\"global-gap\") and not global-variable-exists(\"missing\"), 1px, 2px); @function gate($local-gap) { $inner-gap: 2px; @return if(meta.variable-exists(\"local-gap\") and variable-exists(\"inner-gap\") and global-variable-exists(\"global-gap\") and not global-variable-exists(\"inner-gap\"), $global-gap, 4px); } .button { margin: $kind; padding: gate(3px); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(report.evaluated_css.contains("margin: 1px"));
    assert!(report.evaluated_css.contains("padding: 1px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_mixin_metadata_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "@mixin present { color: red; } @function gate() { @return if(meta.mixin-exists(\"present\") and not mixin-exists(\"not-defined-here\"), 1px, 2px); } .button { margin: gate(); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(report.evaluated_css.contains("margin: 1px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_preserves_mixin_exists_declaration_order() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function gate() { @return if(mixin-exists(\"later\"), 2px, 1px); } @mixin later { color: red; } .button { margin: gate(); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(report.evaluated_css.contains("margin: 1px"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_expands_static_mixin_includes() {
    let report = derive_static_stylesheet_module_evaluation(
        "$brand: red; @mixin tone($color, $gap: 1px) { color: $color; margin: $gap; padding: $brand; } .button { @include tone(blue, 2px); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@mixin"));
    assert!(!report.evaluated_css.contains("@include"));
    assert!(report.evaluated_css.contains("color: blue"));
    assert!(report.evaluated_css.contains("margin: 2px"));
    assert!(report.evaluated_css.contains("padding: red"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_expands_static_mixin_if_blocks() {
    let report = derive_static_stylesheet_module_evaluation(
        "@mixin tone($enabled) { @if $enabled { color: red; } @else { color: blue; } } .button { @include tone(false); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@mixin"));
    assert!(!report.evaluated_css.contains("@include"));
    assert!(!report.evaluated_css.contains("@if"));
    assert!(report.evaluated_css.contains("color: blue"));
    assert!(!report.evaluated_css.contains("color: red"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_sass_evaluation_expands_static_mixin_if_branches() {
    let report = derive_static_stylesheet_module_evaluation(
        "@mixin tone($enabled)\n  @if $enabled\n    color: red\n  @else\n    color: blue\n.button\n  @include tone(false)",
        StyleDialect::Sass,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@mixin"));
    assert!(!report.evaluated_css.contains("@include"));
    assert!(!report.evaluated_css.contains("@if"));
    assert!(!report.evaluated_css.contains("@else"));
    assert!(report.evaluated_css.contains("color: blue"));
    assert!(!report.evaluated_css.contains("color: red"));
    assert_eq!(report.replacement_count, 0);
    assert_eq!(report.native_replacement_legacy_reflection_count, 0);
    assert_eq!(report.native_structural_edit_count, 2);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_scss_evaluation_expands_static_mixin_for_loops() {
    let report = derive_static_stylesheet_module_evaluation(
        "@mixin items($count) { @for $i from 1 through $count { order: $i; } } .button { @include items(3); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@mixin"));
    assert!(!report.evaluated_css.contains("@include"));
    assert!(!report.evaluated_css.contains("@for"));
    assert!(!report.evaluated_css.contains("$i"));
    assert!(report.evaluated_css.contains("order: 1"));
    assert!(report.evaluated_css.contains("order: 2"));
    assert!(report.evaluated_css.contains("order: 3"));
    assert_eq!(report.replacement_count, 0);
    assert_eq!(report.native_structural_edit_count, 2);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_sass_evaluation_expands_static_mixin_for_loops() {
    let report = derive_static_stylesheet_module_evaluation(
        "@mixin items($count)\n  @for $i from 1 through $count\n    order: $i\n.button\n  @include items(3)",
        StyleDialect::Sass,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@mixin"));
    assert!(!report.evaluated_css.contains("@include"));
    assert!(!report.evaluated_css.contains("@for"));
    assert!(!report.evaluated_css.contains("$i"));
    assert!(report.evaluated_css.contains("order: 1"));
    assert!(report.evaluated_css.contains("order: 2"));
    assert!(report.evaluated_css.contains("order: 3"));
    assert_eq!(report.replacement_count, 0);
    assert_eq!(report.native_structural_edit_count, 2);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_sass_evaluation_expands_static_mixin_each_loops() {
    let report = derive_static_stylesheet_module_evaluation(
        "@mixin tones()\n  @each $tone in red, blue\n    color: $tone\n.button\n  @include tones()",
        StyleDialect::Sass,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@mixin"));
    assert!(!report.evaluated_css.contains("@include"));
    assert!(!report.evaluated_css.contains("@each"));
    assert!(!report.evaluated_css.contains("$tone"));
    assert!(report.evaluated_css.contains("color: red"));
    assert!(report.evaluated_css.contains("color: blue"));
    assert_eq!(report.replacement_count, 0);
    assert_eq!(report.native_structural_edit_count, 2);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_sass_evaluation_expands_static_mixin_while_loops() {
    let report = derive_static_stylesheet_module_evaluation(
        "@mixin items()\n  $i: 0\n  @while $i < 3\n    $i: $i + 1\n    order: $i\n.button\n  @include items()",
        StyleDialect::Sass,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@mixin"));
    assert!(!report.evaluated_css.contains("@include"));
    assert!(!report.evaluated_css.contains("@while"));
    assert!(!report.evaluated_css.contains("$i"));
    assert!(report.evaluated_css.contains("order: 1"));
    assert!(report.evaluated_css.contains("order: 2"));
    assert!(report.evaluated_css.contains("order: 3"));
    assert_eq!(report.replacement_count, 0);
    assert_eq!(report.native_structural_edit_count, 2);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_scss_evaluation_expands_static_top_level_if_blocks() {
    let report = derive_static_stylesheet_module_evaluation(
        "$enabled: false; @if $enabled { .on { color: green; } } @else { .off { color: red; } }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@if"));
    assert!(!report.evaluated_css.contains("@else"));
    assert!(!report.evaluated_css.contains("$enabled"));
    assert!(report.evaluated_css.contains(".off { color: red; }"));
    assert!(!report.evaluated_css.contains(".on { color: green; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_scss_evaluation_preserves_dynamic_top_level_if_blocks() {
    let source = "$enabled: var(--enabled); @if $enabled { .on { color: green; } } @else { .off { color: red; } }";
    let report = derive_static_stylesheet_module_evaluation(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.evaluated_css, source);
    assert_eq!(report.native_edit_count, 0);
    assert_eq!(report.native_value_edit_count, 0);
    assert_eq!(report.native_structural_edit_count, 0);
    assert!(report.value_resolution.raw_count > 0);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_scss_evaluation_resolves_static_values_inside_top_level_if_blocks() {
    let report = derive_static_stylesheet_module_evaluation(
        "$enabled: true; $brand: red; @if $enabled { .on { color: $brand; } } @else { .off { color: blue; } }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@if"));
    assert!(!report.evaluated_css.contains("@else"));
    assert!(!report.evaluated_css.contains("$brand"));
    assert!(report.evaluated_css.contains(".on { color: red; }"));
    assert!(!report.evaluated_css.contains(".off { color: blue; }"));
    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "$brand");
    assert_eq!(report.resolved_replacements[0].text, "red");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert_eq!(report.native_replacement_legacy_reflection_count, 1);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_scss_evaluation_resolves_static_functions_inside_top_level_if_blocks() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function tone($color) { @return $color; } $enabled: true; @if $enabled { .on { color: tone(red); } } @else { .off { color: blue; } }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@function"));
    assert!(!report.evaluated_css.contains("@if"));
    assert!(!report.evaluated_css.contains("tone(red)"));
    assert!(report.evaluated_css.contains(".on { color: red; }"));
    assert!(!report.evaluated_css.contains(".off { color: blue; }"));
    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].name, "function:tone");
    assert_eq!(report.resolved_replacements[0].text, "red");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert_eq!(report.native_replacement_legacy_reflection_count, 1);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_scss_evaluation_expands_static_top_level_for_loops() {
    let report = derive_static_stylesheet_module_evaluation(
        "@for $i from 1 through 3 { .n { order: $i; } }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@for"));
    assert!(!report.evaluated_css.contains("$i"));
    assert!(report.evaluated_css.contains("order: 1"));
    assert!(report.evaluated_css.contains("order: 2"));
    assert!(report.evaluated_css.contains("order: 3"));
    assert_eq!(report.replacement_count, 3);
    assert_eq!(report.resolved_replacements[0].name, "$i");
    assert_eq!(report.resolved_replacements[0].text, "1");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert_eq!(report.native_replacement_legacy_reflection_count, 3);
    assert_eq!(report.native_structural_edit_count, 1);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_scss_evaluation_expands_static_top_level_each_loops() {
    let report = derive_static_stylesheet_module_evaluation(
        "@each $tone in red, blue { .n { color: $tone; } }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@each"));
    assert!(!report.evaluated_css.contains("$tone"));
    assert!(report.evaluated_css.contains("color: red"));
    assert!(report.evaluated_css.contains("color: blue"));
    assert_eq!(report.replacement_count, 2);
    assert_eq!(report.resolved_replacements[0].name, "$tone");
    assert_eq!(report.resolved_replacements[0].text, "red");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert_eq!(report.native_replacement_legacy_reflection_count, 2);
    assert_eq!(report.native_structural_edit_count, 1);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_scss_evaluation_expands_static_top_level_while_loops() {
    let report = derive_static_stylesheet_module_evaluation(
        "$i: 0; @while $i < 3 { $i: $i + 1; .n { order: $i; } }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@while"));
    assert!(!report.evaluated_css.contains("$i"));
    assert!(report.evaluated_css.contains("order: 1"));
    assert!(report.evaluated_css.contains("order: 2"));
    assert!(report.evaluated_css.contains("order: 3"));
    assert_eq!(report.replacement_count, 3);
    assert_eq!(report.resolved_replacements[0].name, "$i");
    assert_eq!(report.resolved_replacements[0].text, "1");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert_eq!(report.native_replacement_legacy_reflection_count, 3);
    assert_eq!(report.native_structural_edit_count, 2);
    assert!(report.oracle.all_legacy_declaration_values_preserved);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_sass_evaluation_expands_static_top_level_for_loops() {
    let report = derive_static_stylesheet_module_evaluation(
        "@for $i from 1 through 3\n  .n\n    order: $i",
        StyleDialect::Sass,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@for"));
    assert!(!report.evaluated_css.contains("$i"));
    assert!(report.evaluated_css.contains("order: 1"));
    assert!(report.evaluated_css.contains("order: 2"));
    assert!(report.evaluated_css.contains("order: 3"));
    assert_eq!(report.replacement_count, 3);
    assert_eq!(report.native_replacement_legacy_reflection_count, 3);
    assert_eq!(report.native_structural_edit_count, 1);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_sass_evaluation_expands_static_top_level_each_loops() {
    let report = derive_static_stylesheet_module_evaluation(
        "@each $tone in red, blue\n  .n\n    color: $tone",
        StyleDialect::Sass,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@each"));
    assert!(!report.evaluated_css.contains("$tone"));
    assert!(report.evaluated_css.contains("color: red"));
    assert!(report.evaluated_css.contains("color: blue"));
    assert_eq!(report.replacement_count, 2);
    assert_eq!(report.native_replacement_legacy_reflection_count, 2);
    assert_eq!(report.native_structural_edit_count, 1);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_sass_evaluation_expands_static_top_level_while_loops() {
    let report = derive_static_stylesheet_module_evaluation(
        "$i: 0\n@while $i < 3\n  $i: $i + 1\n  .n\n    order: $i",
        StyleDialect::Sass,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@while"));
    assert!(!report.evaluated_css.contains("$i"));
    assert!(report.evaluated_css.contains("order: 1"));
    assert!(report.evaluated_css.contains("order: 2"));
    assert!(report.evaluated_css.contains("order: 3"));
    assert_eq!(report.replacement_count, 3);
    assert_eq!(report.native_replacement_legacy_reflection_count, 3);
    assert_eq!(report.native_structural_edit_count, 2);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_sass_evaluation_expands_static_top_level_if_branches() {
    let report = derive_static_stylesheet_module_evaluation(
        "$enabled: true\n@if $enabled\n  .on\n    color: green\n@else\n  .off\n    color: gray",
        StyleDialect::Sass,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@if"));
    assert!(!report.evaluated_css.contains("@else"));
    assert!(!report.evaluated_css.contains("$enabled"));
    assert!(report.evaluated_css.contains("color: green"));
    assert!(!report.evaluated_css.contains("color: gray"));
    assert_eq!(report.replacement_count, 0);
    assert_eq!(report.native_replacement_legacy_reflection_count, 0);
    assert_eq!(report.native_structural_edit_count, 2);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_sass_evaluation_expands_static_top_level_if_branch_variables() {
    let report = derive_static_stylesheet_module_evaluation(
        "$enabled: true\n$brand: green\n@if $enabled\n  .on\n    color: $brand\n@else\n  .off\n    color: gray",
        StyleDialect::Sass,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@if"));
    assert!(!report.evaluated_css.contains("@else"));
    assert!(!report.evaluated_css.contains("$brand"));
    assert!(report.evaluated_css.contains("color: green"));
    assert!(!report.evaluated_css.contains("color: gray"));
    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.native_replacement_legacy_reflection_count, 1);
    assert_eq!(report.native_structural_edit_count, 3);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_sass_evaluation_expands_static_top_level_if_branch_functions() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function tone($color)\n  @return $color\n$enabled: true\n@if $enabled\n  .on\n    color: tone(green)\n@else\n  .off\n    color: gray",
        StyleDialect::Sass,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@function"));
    assert!(!report.evaluated_css.contains("@if"));
    assert!(!report.evaluated_css.contains("@else"));
    assert!(!report.evaluated_css.contains("tone(green)"));
    assert!(report.evaluated_css.contains("color: green"));
    assert!(!report.evaluated_css.contains("color: gray"));
    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.native_replacement_legacy_reflection_count, 1);
    assert_eq!(report.native_structural_edit_count, 3);
    assert!(report.native_edit_output_matches_evaluated_css);
}

#[test]
fn static_scss_evaluation_expands_mixin_includes_with_static_function_values() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function double($value) { @return $value * 2; } @mixin tone($gap) { margin: double($gap); color: red; } .button { @include tone(2px); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@function"));
    assert!(!report.evaluated_css.contains("@mixin"));
    assert!(!report.evaluated_css.contains("@include"));
    assert!(report.evaluated_css.contains("margin: 4px"));
    assert!(report.evaluated_css.contains("color: red"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_expands_nested_static_mixin_includes() {
    let report = derive_static_stylesheet_module_evaluation(
        "@mixin spacing($gap) { margin: $gap; } @mixin tone($gap, $color: red) { @include spacing($gap); color: $color; } .button { @include tone(2px, blue); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@mixin"));
    assert!(!report.evaluated_css.contains("@include"));
    assert!(report.evaluated_css.contains("margin: 2px"));
    assert!(report.evaluated_css.contains("color: blue"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_sass_evaluation_expands_nested_static_mixin_includes() {
    let report = derive_static_stylesheet_module_evaluation(
        "@mixin spacing($gap)\n  margin: $gap\n@mixin tone($gap, $color: red)\n  @include spacing($gap)\n  color: $color\n.button\n  @include tone(2px, blue)\n",
        StyleDialect::Sass,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@mixin"));
    assert!(!report.evaluated_css.contains("@include"));
    assert!(report.evaluated_css.contains("margin: 2px"));
    assert!(report.evaluated_css.contains("color: blue"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_expands_mixin_local_variables() {
    let report = derive_static_stylesheet_module_evaluation(
        "@mixin tone($gap) { $space: $gap * 2; $color: if($space == 4px, blue, red); margin: $space; color: $color; } .button { @include tone(2px); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("$space"));
    assert!(!report.evaluated_css.contains("$color"));
    assert!(!report.evaluated_css.contains("@mixin"));
    assert!(!report.evaluated_css.contains("@include"));
    assert!(report.evaluated_css.contains("margin: 4px"));
    assert!(report.evaluated_css.contains("color: blue"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_preserves_dynamic_mixin_local_variables_as_oracle_report() {
    let report = derive_static_stylesheet_module_evaluation(
        "@mixin tone { $space: meta.inspect((a: b)); margin: $space; } .button { @include tone; }",
        StyleDialect::Scss,
    );

    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 0);
    assert!(report.evaluated_css.contains("@mixin tone"));
    assert!(report.evaluated_css.contains("meta.inspect((a: b))"));
    assert!(report.evaluated_css.contains("@include tone"));
    assert!(!report.evaluated_css.contains("margin: (a: b)"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_preserves_recursive_nested_mixin_includes_as_oracle_report() {
    let report = derive_static_stylesheet_module_evaluation(
        "@mixin a { @include b; } @mixin b { @include a; } .button { @include a; }",
        StyleDialect::Scss,
    );

    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 0);
    assert!(report.evaluated_css.contains("@mixin a"));
    assert!(report.evaluated_css.contains("@mixin b"));
    assert!(report.evaluated_css.contains("@include a"));
    assert!(report.evaluated_css.contains("@include b"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_expands_hyphen_underscore_mixin_includes() {
    let report = derive_static_stylesheet_module_evaluation(
        "@mixin tone_color($color) { color: $color; } .button { @include tone-color(green); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(!report.evaluated_css.contains("@mixin"));
    assert!(!report.evaluated_css.contains("@include"));
    assert!(report.evaluated_css.contains("color: green"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_function_list_constructor_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function tail($list) { @return list.nth(list.append($list, 3px), 3); } .button { margin: tail(1px 2px); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.resolved_replacements[0].name, "function:tail");
    assert_eq!(report.resolved_replacements[0].text, "3px");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { margin: 3px; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_sass_color_mix_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function tone() { @return color.mix(red, blue); } .button { color: tone(); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.resolved_replacements[0].name, "function:tone");
    assert_eq!(report.resolved_replacements[0].text, "rgb(127.5, 0, 127.5)");
    assert_eq!(
        report.resolved_replacements[0].rendered_value.as_deref(),
        Some("purple")
    );
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(
        report
            .evaluated_css
            .contains(".button { color: rgb(127.5, 0, 127.5); }")
    );
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_color_channel_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function tone-channel() { @return color.channel(color.mix(red, blue), \"red\", rgb); } .button { z-index: tone-channel(); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(
        report.resolved_replacements[0].name,
        "function:tone-channel"
    );
    assert_eq!(report.resolved_replacements[0].text, "127.5");
    assert_eq!(
        report.resolved_replacements[0].rendered_value.as_deref(),
        Some("127.5")
    );
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { z-index: 127.5; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_hsl_color_channel_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function tone-channel() { @return hue(#808000); } .button { --hue: tone-channel(); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(
        report.resolved_replacements[0].name,
        "function:tone-channel"
    );
    assert_eq!(report.resolved_replacements[0].text, "60deg");
    assert_eq!(
        report.resolved_replacements[0].rendered_value.as_deref(),
        Some("60deg")
    );
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { --hue: 60deg; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_static_hsl_color_transform_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function tone() { @return adjust-hue(red, 120deg); } .button { color: tone(); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.resolved_replacements[0].name, "function:tone");
    assert_eq!(report.resolved_replacements[0].text, "#0f0");
    assert_eq!(
        report.resolved_replacements[0].rendered_value.as_deref(),
        Some("#0f0")
    );
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { color: #0f0; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_legacy_global_color_function_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function tone-channel() { @return red(mix(red, blue)); } .button { z-index: tone-channel(); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(
        report.resolved_replacements[0].name,
        "function:tone-channel"
    );
    assert_eq!(report.resolved_replacements[0].text, "127.5");
    assert_eq!(
        report.resolved_replacements[0].rendered_value.as_deref(),
        Some("127.5")
    );
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains(".button { z-index: 127.5; }"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_sass_rgb_color_constructor_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function tone() { @return rgba(red, .5); } .button { color: tone(); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.resolved_replacements[0].name, "function:tone");
    assert_eq!(report.resolved_replacements[0].text, "rgba(255, 0, 0, 0.5)");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(
        report
            .evaluated_css
            .contains(".button { color: rgba(255, 0, 0, 0.5); }")
    );
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_sass_hsl_color_constructor_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function tone() { @return hsl(180, 100%, 50%); } @function overlay() { @return hsla(120, 100%, 50%, .5); } .button { color: tone(); background: overlay(); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.resolved_replacements[0].name, "function:tone");
    assert_eq!(report.resolved_replacements[0].text, "#0ff");
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert_eq!(report.resolved_replacements[1].name, "function:overlay");
    assert_eq!(report.resolved_replacements[1].text, "rgba(0, 255, 0, 0.5)");
    assert_eq!(report.resolved_replacements[1].abstract_value_kind, "exact");
    assert!(
        report
            .evaluated_css
            .contains(".button { color: #0ff; background: rgba(0, 255, 0, 0.5); }")
    );
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reduces_sass_opacity_color_returns() {
    let report = derive_static_stylesheet_module_evaluation(
        "@function tone() { @return transparentize(red, .25); } .button { color: tone(); }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.resolved_replacements[0].name, "function:tone");
    assert_eq!(
        report.resolved_replacements[0].text,
        "rgba(255, 0, 0, 0.75)"
    );
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(
        report
            .evaluated_css
            .contains(".button { color: rgba(255, 0, 0, 0.75); }")
    );
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_value_resolution_reports_unresolved_references_as_top() {
    let report = summarize_static_stylesheet_value_resolution(
        ".button { color: $missing; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.reference_count, 1);
    assert_eq!(report.top_count, 1);
    assert_eq!(report.unresolved_reference_count, 1);
    assert_eq!(report.values[0].outcome, "top");
    assert_eq!(report.values[0].reason, "unresolvedReference");
    assert_eq!(report.values[0].rendered_value, None);
}

#[test]
fn static_scss_evaluation_preserves_forward_composite_as_top_oracle_report() {
    let source = "$border: 1px solid $brand; $brand: red; .button { border: $border; }";
    let report = derive_static_stylesheet_module_evaluation(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 0);
    assert_eq!(report.evaluated_css, source);
    assert_eq!(report.value_resolution.reference_count, 1);
    assert_eq!(report.value_resolution.top_count, 1);
    assert_eq!(report.value_resolution.unresolved_reference_count, 1);
    assert_eq!(report.value_resolution.values[0].outcome, "top");
    assert_eq!(
        report.value_resolution.values[0].reason,
        "unresolvedReference"
    );
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_value_resolution_reports_cycles_as_top() {
    let report = summarize_static_stylesheet_value_resolution(
        "@a: @b; @b: @a; .button { color: @a; }",
        StyleDialect::Less,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.reference_count, 1);
    assert_eq!(report.top_count, 1);
    assert_eq!(report.cycle_count, 1);
    assert_eq!(report.values[0].outcome, "top");
    assert_eq!(report.values[0].reason, "cycle");
}

#[test]
fn static_value_resolution_emits_exact_alpha_color_mix_values() {
    let report = summarize_static_stylesheet_value_resolution(
        "$tone: color.mix(rgba(red, .5), blue); .button { color: $tone; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.reference_count, 1);
    assert_eq!(report.resolved_count, 1);
    assert_eq!(report.raw_count, 0);
    assert_eq!(report.unsupported_dynamic_count, 0);
    assert_eq!(report.values[0].outcome, "resolved");
    assert_eq!(report.values[0].reason, "resolved");
    assert_eq!(report.values[0].abstract_value_kind, "exact");
}

#[test]
fn static_value_resolution_emits_exact_nested_opacity_color_mix_values() {
    let report = summarize_static_stylesheet_value_resolution(
        "$tone: color.mix(transparentize(red, .25), blue); .button { color: $tone; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.reference_count, 1);
    assert_eq!(report.resolved_count, 1);
    assert_eq!(report.raw_count, 0);
    assert_eq!(report.unsupported_dynamic_count, 0);
    assert_eq!(report.values[0].outcome, "resolved");
    assert_eq!(report.values[0].reason, "resolved");
    assert_eq!(report.values[0].abstract_value_kind, "exact");
}

#[test]
fn static_value_resolution_keeps_percent_opacity_amounts_raw() {
    let report = summarize_static_stylesheet_value_resolution(
        "$tone: transparentize(red, 25%); .button { color: $tone; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.reference_count, 1);
    assert_eq!(report.raw_count, 1);
    assert_eq!(report.unsupported_dynamic_count, 1);
    assert_eq!(report.values[0].outcome, "raw");
    assert_eq!(report.values[0].reason, "unsupportedDynamic");
    assert_eq!(
        report.values[0].rendered_value.as_deref(),
        Some("transparentize(red, 25%)")
    );
}

#[test]
fn static_value_resolution_emits_exact_static_sass_color_mix_values() {
    let report = summarize_static_stylesheet_value_resolution(
        "$tone: color.mix(red, blue); $weighted: color.mix(rgb(255 0 0), blue, $weight: 25%); .button { color: $tone; border-color: $weighted; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.reference_count, 2);
    assert_eq!(report.resolved_count, 2);
    assert_eq!(report.raw_count, 0);
    assert!(
        report
            .values
            .iter()
            .all(|value| value.abstract_value_kind == "exact")
    );
    let rendered_values = report
        .values
        .iter()
        .filter_map(|value| value.rendered_value.as_deref())
        .collect::<Vec<_>>();
    assert!(rendered_values.contains(&"purple"));
    assert!(rendered_values.contains(&"#4000bf"));
}

#[test]
fn static_value_resolution_emits_exact_static_color_channel_values() {
    let report = summarize_static_stylesheet_value_resolution(
        "$red: color.channel(color.mix(red, blue), \"red\", $space: rgb); $alpha: color.alpha(rgba(255, 0, 0, .5)); $opacity: color.opacity(rgba(red, .5)); $hue: color.channel(#808000, \"hue\", $space: hsl); $saturation: saturation(#808000); $lightness: color.lightness(#808000); .button { z-index: $red; opacity: $alpha; flex-grow: $opacity; width: $hue; height: $saturation; margin: $lightness; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.reference_count, 6);
    assert_eq!(report.resolved_count, 6);
    assert_eq!(report.raw_count, 0);
    let rendered_values = report
        .values
        .iter()
        .filter_map(|value| value.rendered_value.as_deref())
        .collect::<Vec<_>>();
    assert!(rendered_values.contains(&"127.5"));
    assert!(rendered_values.contains(&"0.5"));
    assert!(rendered_values.contains(&"60deg"));
    assert!(rendered_values.contains(&"100%"));
    assert!(rendered_values.contains(&"25.098039%"));
}

#[test]
fn static_value_resolution_emits_exact_static_hsl_color_transform_values() {
    let report = summarize_static_stylesheet_value_resolution(
        "$adjusted: adjust-hue($color: red, $degrees: 120deg); $complement: color.complement(red); $light: lighten(#808000, 10%); $dark: darken(#808000, 10%); $sat: saturate(#808000, 10%); $desat: desaturate(#808000, 10%); $gray: grayscale(red); $invert: color.invert(red, $weight: 25%); $scaled: color.scale(#808000, $lightness: 50%); $changed: color.change(#808000, $lightness: 50%); .button { color: $adjusted; background: $complement; border-color: $light; outline-color: $dark; caret-color: $sat; text-decoration-color: $desat; column-rule-color: $gray; accent-color: $invert; fill: $scaled; stroke: $changed; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.reference_count, 10);
    assert_eq!(report.resolved_count, 10);
    assert_eq!(report.raw_count, 0);
    assert!(
        report
            .values
            .iter()
            .all(|value| value.abstract_value_kind == "exact")
    );
    let rendered_values = report
        .values
        .iter()
        .filter_map(|value| value.rendered_value.as_deref())
        .collect::<Vec<_>>();
    assert_eq!(
        rendered_values
            .iter()
            .filter(|value| **value == "#0ff")
            .count(),
        1
    );
    assert!(rendered_values.contains(&"#0f0"));
    assert!(rendered_values.contains(&"#b3b300"));
    assert!(rendered_values.contains(&"#4d4d00"));
    assert!(rendered_values.contains(&"olive"));
    assert!(rendered_values.contains(&"#7a7a06"));
    assert!(rendered_values.contains(&"gray"));
    assert!(rendered_values.contains(&"#bf4040"));
    assert!(rendered_values.contains(&"#ffff40"));
    assert!(rendered_values.contains(&"#ff0"));
}

#[test]
fn static_value_resolution_emits_exact_legacy_global_color_values() {
    let report = summarize_static_stylesheet_value_resolution(
        "$red: red(mix(red, blue)); $green: green(rgb(127.5, 10, 20)); $blue: blue(blue); $alpha: alpha(rgba(255, 0, 0, .5)); .button { z-index: $red; --g: $green; --b: $blue; opacity: $alpha; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.reference_count, 4);
    assert_eq!(report.resolved_count, 4);
    assert_eq!(report.raw_count, 0);
    let rendered_values = report
        .values
        .iter()
        .filter_map(|value| value.rendered_value.as_deref())
        .collect::<Vec<_>>();
    assert!(rendered_values.contains(&"127.5"));
    assert!(rendered_values.contains(&"10"));
    assert!(rendered_values.contains(&"255"));
    assert!(rendered_values.contains(&"0.5"));
}

#[test]
fn static_value_resolution_emits_exact_sass_rgb_color_constructor_values() {
    let report = summarize_static_stylesheet_value_resolution(
        "$transparent: rgba(red, .5); $opaque: rgb(red, 1); .button { color: $transparent; border-color: $opaque; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.reference_count, 2);
    assert_eq!(report.resolved_count, 2);
    assert_eq!(report.raw_count, 0);
    let rendered_values = report
        .values
        .iter()
        .filter_map(|value| value.rendered_value.as_deref())
        .collect::<Vec<_>>();
    assert!(rendered_values.contains(&"#ff000080"));
    assert!(rendered_values.contains(&"red"));
}

#[test]
fn static_value_resolution_emits_exact_sass_hsl_color_constructor_values() {
    let report = summarize_static_stylesheet_value_resolution(
        "$tone: hsl(180, 100%, 50%); $overlay: hsla(120, 100%, 50%, .5); .button { color: $tone; background: $overlay; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.reference_count, 2);
    assert_eq!(report.resolved_count, 2);
    assert_eq!(report.raw_count, 0);
    let rendered_values = report
        .values
        .iter()
        .filter_map(|value| value.rendered_value.as_deref())
        .collect::<Vec<_>>();
    assert!(rendered_values.contains(&"#0ff"));
    assert!(rendered_values.contains(&"#00ff0080"));
}

#[test]
fn static_value_resolution_emits_exact_sass_opacity_color_values() {
    let report = summarize_static_stylesheet_value_resolution(
        "$transparent: transparentize(red, .25); $faded: fade-in(rgba(red, .5), .25); $opaque: opacify(rgba(red, .5), .25); $adjusted: color.adjust(red, $alpha: -.25); $changed: color.change(red, $alpha: .5); $scaled: color.scale(rgba(red, .5), $alpha: -50%); .button { color: $transparent; background: $faded; border-color: $opaque; outline-color: $adjusted; caret-color: $changed; text-decoration-color: $scaled; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.reference_count, 6);
    assert_eq!(report.resolved_count, 6);
    assert_eq!(report.raw_count, 0);
    assert!(
        report
            .values
            .iter()
            .all(|value| value.abstract_value_kind == "exact")
    );
    let rendered_values = report
        .values
        .iter()
        .filter_map(|value| value.rendered_value.as_deref())
        .collect::<Vec<_>>();
    assert_eq!(
        rendered_values
            .iter()
            .filter(|value| **value == "#ff0000bf")
            .count(),
        4
    );
    assert!(rendered_values.contains(&"#ff000080"));
    assert!(rendered_values.contains(&"#ff000040"));
}

#[test]
fn static_value_resolution_emits_exact_nested_sass_color_helper_values() {
    let report = summarize_static_stylesheet_value_resolution(
        "$tone: list.nth(list.append(1px, transparentize(red, .25)), 2); $scaled: list.nth(list.append(1px, color.scale(#808000, $lightness: 50%)), 2); $opacity: list.nth(list.append(1px, color.opacity(rgba(red, .5))), 2); .button { color: $tone; background: $scaled; opacity: $opacity; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.reference_count, 3);
    assert_eq!(report.resolved_count, 3);
    assert_eq!(report.raw_count, 0);
    assert!(
        report
            .values
            .iter()
            .all(|value| value.abstract_value_kind == "exact")
    );
    let rendered_values = report
        .values
        .iter()
        .filter_map(|value| value.rendered_value.as_deref())
        .collect::<Vec<_>>();
    assert!(rendered_values.contains(&"#ff0000bf"));
    assert!(rendered_values.contains(&"#ffff40"));
    assert!(rendered_values.contains(&"0.5"));
}

#[test]
fn static_scss_evaluation_preserves_css_rgba_constructor_text() {
    let report = derive_static_stylesheet_module_evaluation(
        "$transparent: rgba(255, 0, 0, .5); .button { color: $transparent; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert!(
        report
            .evaluated_css
            .contains(".button { color: rgba(255, 0, 0, .5); }")
    );
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_value_resolution_keeps_css_filter_alpha_raw() {
    let report = summarize_static_stylesheet_value_resolution(
        "$filter: alpha(opacity=50); .button { filter: $filter; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.reference_count, 1);
    assert_eq!(report.raw_count, 1);
    assert_eq!(report.unsupported_dynamic_count, 1);
    assert_eq!(report.values[0].outcome, "raw");
    assert_eq!(report.values[0].reason, "unsupportedDynamic");
    assert_eq!(
        report.values[0].rendered_value.as_deref(),
        Some("alpha(opacity=50)")
    );
}

#[test]
fn static_value_resolution_keeps_unspecified_hsl_color_channels_raw() {
    let report = summarize_static_stylesheet_value_resolution(
        "$hue: color.channel(red, \"hue\"); .button { z-index: $hue; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.reference_count, 1);
    assert_eq!(report.raw_count, 1);
    assert_eq!(report.unsupported_dynamic_count, 1);
    assert_eq!(report.values[0].outcome, "raw");
    assert_eq!(report.values[0].reason, "unsupportedDynamic");
    assert_eq!(
        report.values[0].rendered_value.as_deref(),
        Some("color.channel(red, \"hue\")")
    );
}

#[test]
fn static_value_resolution_emits_exact_ie_hex_str_values() {
    let report = summarize_static_stylesheet_value_resolution(
        "$legacy: ie-hex-str(rgba(red, .5)); .button { color: $legacy; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.reference_count, 1);
    assert_eq!(report.resolved_count, 1);
    assert_eq!(report.raw_count, 0);
    assert_eq!(report.unsupported_dynamic_count, 0);
    assert_eq!(report.values[0].outcome, "resolved");
    assert_eq!(report.values[0].reason, "resolved");
    assert_eq!(
        report.values[0].rendered_value.as_deref(),
        Some("#80ff0000")
    );
}

#[test]
fn static_value_resolution_emits_exact_static_inspect_values() {
    let report = summarize_static_stylesheet_value_resolution(
        "$tone: meta.inspect(red); $gap: inspect(2px); .button { color: $tone; margin: $gap; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.reference_count, 2);
    assert_eq!(report.resolved_count, 2);
    assert_eq!(report.raw_count, 0);
    assert_eq!(report.unsupported_dynamic_count, 0);
    let rendered_values = report
        .values
        .iter()
        .filter_map(|value| value.rendered_value.as_deref())
        .collect::<Vec<_>>();
    assert!(rendered_values.contains(&"red"));
    assert!(rendered_values.contains(&"2px"));
}

#[test]
fn static_value_resolution_emits_exact_static_color_values() {
    let report = summarize_static_stylesheet_value_resolution(
        "$tone: color-mix(in srgb, red 50%, blue 50%); .button { color: $tone; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.reference_count, 1);
    assert_eq!(report.resolved_count, 1);
    assert_eq!(report.raw_count, 0);
    assert_eq!(report.values[0].outcome, "resolved");
    assert_eq!(report.values[0].abstract_value_kind, "exact");
    assert_eq!(report.values[0].rendered_value.as_deref(), Some("purple"));
}

#[test]
fn static_scss_evaluation_reports_exact_color_replacements_without_cutover() {
    let report = derive_static_stylesheet_module_evaluation(
        "$tone: color-mix(in srgb, red 50%, blue 50%); .button { color: $tone; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(
        report.resolved_replacements[0].text,
        "color-mix(in srgb, red 50%, blue 50%)"
    );
    assert_eq!(
        report.resolved_replacements[0].rendered_value.as_deref(),
        Some("purple")
    );
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(
        report
            .evaluated_css
            .contains("color-mix(in srgb, red 50%, blue 50%)")
    );
    assert!(!report.evaluated_css.contains("color: purple"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_scss_evaluation_reports_exact_sass_color_mix_replacements() {
    let report = derive_static_stylesheet_module_evaluation(
        "$tone: color.mix(red, blue); .button { color: $tone; }",
        StyleDialect::Scss,
    );
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.replacement_count, 1);
    assert_eq!(report.resolved_replacements[0].text, "rgb(127.5, 0, 127.5)");
    assert_eq!(
        report.resolved_replacements[0].rendered_value.as_deref(),
        Some("purple")
    );
    assert_eq!(report.resolved_replacements[0].abstract_value_kind, "exact");
    assert!(report.evaluated_css.contains("color: rgb(127.5, 0, 127.5)"));
    assert!(report.oracle.all_legacy_declaration_values_preserved);
}

#[test]
fn static_value_resolution_reports_fuel_exhaustion_as_top() {
    let mut source = String::new();
    for index in 0..130 {
        let _ = write!(source, "@v{index}: @v{}; ", index + 1);
    }
    source.push_str("@v130: 1px; .button { width: @v0; }");

    let report = summarize_static_stylesheet_value_resolution(&source, StyleDialect::Less);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.reference_count, 1);
    assert_eq!(report.top_count, 1);
    assert_eq!(report.fuel_exhausted_count, 1);
    assert_eq!(report.values[0].outcome, "top");
    assert_eq!(report.values[0].reason, "fuelExhausted");
}
