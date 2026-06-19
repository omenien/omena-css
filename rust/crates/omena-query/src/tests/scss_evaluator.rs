use crate::{
    OmenaParserStyleDialect, summarize_omena_query_scss_evaluator_control_flow_from_source,
    summarize_omena_query_static_stylesheet_evaluator_from_source,
    summarize_omena_query_static_stylesheet_evaluator_oracle_corpus,
};

#[test]
fn exposes_static_stylesheet_oracle_corpus_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_oracle_corpus();

    assert_eq!(summary.schema_version, "0");
    assert_eq!(
        summary.product,
        "omena-query.static-stylesheet-evaluator-oracle-corpus"
    );
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert_eq!(summary.product_output_source, "legacyEvaluatedCss");
    assert_eq!(summary.fixture_count, 12);
    assert_eq!(summary.scss_fixture_count, 6);
    assert_eq!(summary.less_fixture_count, 6);
    assert_eq!(summary.evaluated_fixture_count, summary.fixture_count);
    assert_eq!(summary.missing_evaluation_count, 0);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.native_replacement_count > 0);
    assert!(summary.native_value_reference_count > 0);
    assert!(summary.native_resolved_value_count > 0);
    assert_eq!(summary.native_top_value_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(
        summary.corpus.product,
        "omena-scss-eval.static-stylesheet-oracle-corpus"
    );
    assert!(
        summary
            .corpus
            .fixtures
            .iter()
            .any(|fixture| fixture.id == "scss.static-for-return")
    );
    assert!(
        summary
            .corpus
            .fixtures
            .iter()
            .any(|fixture| fixture.id == "less.detached-ruleset")
    );
    assert!(
        summary
            .corpus
            .fixtures
            .iter()
            .any(|fixture| fixture.id == "less.ruleset-guarded-mixin")
    );
}

#[test]
fn exposes_static_stylesheet_evaluator_oracle_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "$gap: 1px; .card { margin: $gap; }",
        OmenaParserStyleDialect::Scss,
    );

    assert_eq!(summary.schema_version, "0");
    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.dialect, "scss");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.supported_dialect);
    assert_eq!(summary.product_output_source, "legacyEvaluatedCss");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert!(summary.evaluation_available);
    assert!(summary.value_resolution_available);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_replacement_count, 1);
    assert_eq!(summary.native_value_reference_count, 1);
    assert_eq!(summary.native_resolved_value_count, 1);
    assert_eq!(summary.native_raw_value_count, 0);
    assert_eq!(summary.native_top_value_count, 0);
    assert!(
        summary
            .evaluation
            .as_ref()
            .is_some_and(|evaluation| evaluation.evaluated_css.contains("margin: 1px"))
    );
}

#[test]
fn exposes_static_scss_legacy_rounding_aliases_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "$ceil: ceil(1.2px); $floor: floor(1.8px); $round: round(1.5px); .card { top: $ceil; bottom: $floor; left: $round; }",
        OmenaParserStyleDialect::Scss,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_value_reference_count, 3);
    assert_eq!(summary.native_resolved_value_count, 3);
    assert!(
        summary
            .evaluation
            .as_ref()
            .is_some_and(|evaluation| evaluation.evaluated_css.contains("top: 2px"))
    );
    assert!(
        summary
            .evaluation
            .as_ref()
            .is_some_and(|evaluation| evaluation.evaluated_css.contains("bottom: 1px"))
    );
    assert!(
        summary
            .evaluation
            .as_ref()
            .is_some_and(|evaluation| evaluation.evaluated_css.contains("left: 2px"))
    );
}

#[test]
fn exposes_static_scss_math_percentage_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "$ratio: math.percentage(.25); .card { width: $ratio; }",
        OmenaParserStyleDialect::Scss,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_value_reference_count, 1);
    assert_eq!(summary.native_resolved_value_count, 1);
    assert_eq!(summary.native_raw_value_count, 0);
    assert!(
        summary
            .evaluation
            .as_ref()
            .is_some_and(|evaluation| evaluation.evaluated_css.contains("width: 25%"))
    );
}

#[test]
fn exposes_static_scss_math_trig_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "$sin: math.sin(30deg); $atan2: math.atan2(1px, 1px); .card { opacity: $sin; rotate: $atan2; }",
        OmenaParserStyleDialect::Scss,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_value_reference_count, 2);
    assert_eq!(summary.native_resolved_value_count, 2);
    assert_eq!(summary.native_raw_value_count, 0);
    assert!(
        summary
            .evaluation
            .as_ref()
            .is_some_and(|evaluation| evaluation.evaluated_css.contains("opacity: 0.5"))
    );
    assert!(
        summary
            .evaluation
            .as_ref()
            .is_some_and(|evaluation| evaluation.evaluated_css.contains("rotate: 45deg"))
    );
}

#[test]
fn exposes_static_scss_math_constants_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "$pi: math.$pi; $e: math.$e; .card { --pi: $pi; --e: $e; }",
        OmenaParserStyleDialect::Scss,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_value_reference_count, 2);
    assert_eq!(summary.native_resolved_value_count, 2);
    assert_eq!(summary.native_raw_value_count, 0);
    assert!(
        summary
            .evaluation
            .as_ref()
            .is_some_and(|evaluation| evaluation.evaluated_css.contains("--pi: 3.1415926536"))
    );
    assert!(
        summary
            .evaluation
            .as_ref()
            .is_some_and(|evaluation| evaluation.evaluated_css.contains("--e: 2.7182818285"))
    );
}

#[test]
fn exposes_static_scss_math_constant_arguments_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "$sin: math.sin(math.$pi); $unitless: if(math.is-unitless(math.$pi), 1px, 2px); .card { opacity: $sin; margin: $unitless; }",
        OmenaParserStyleDialect::Scss,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_value_reference_count, 2);
    assert_eq!(summary.native_resolved_value_count, 2);
    assert_eq!(summary.native_raw_value_count, 0);
    assert!(
        summary
            .evaluation
            .as_ref()
            .is_some_and(|evaluation| evaluation.evaluated_css.contains("opacity: 0"))
    );
    assert!(
        summary
            .evaluation
            .as_ref()
            .is_some_and(|evaluation| evaluation.evaluated_css.contains("margin: 1px"))
    );
}

#[test]
fn exposes_static_scss_legacy_list_metadata_aliases_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "$separator: list-separator((1px, 2px)); $bracketed: if(is-bracketed([1px]), 1px, 2px); .card { content: $separator; margin: $bracketed; }",
        OmenaParserStyleDialect::Scss,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_value_reference_count, 2);
    assert_eq!(summary.native_resolved_value_count, 1);
    assert_eq!(summary.native_raw_value_count, 1);
    assert!(
        summary
            .evaluation
            .as_ref()
            .is_some_and(|evaluation| evaluation.evaluated_css.contains("content: \"comma\""))
    );
    assert!(
        summary
            .evaluation
            .as_ref()
            .is_some_and(|evaluation| evaluation.evaluated_css.contains("margin: 1px"))
    );
}

#[test]
fn exposes_static_scss_slash_lists_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "$stroke: list.slash(1px, solid, red); $separator: list.separator($stroke); .card { font: $stroke; content: $separator; }",
        OmenaParserStyleDialect::Scss,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_value_reference_count, 2);
    assert_eq!(summary.native_resolved_value_count, 0);
    assert_eq!(summary.native_raw_value_count, 2);
    assert!(
        summary
            .evaluation
            .as_ref()
            .is_some_and(|evaluation| evaluation.evaluated_css.contains("font: 1px / solid / red"))
    );
    assert!(
        summary
            .evaluation
            .as_ref()
            .is_some_and(|evaluation| evaluation.evaluated_css.contains("content: \"slash\""))
    );
}

#[test]
fn exposes_static_scss_function_comparison_operands_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "$stroke: list.slash(1px, solid, red); $kind: if(meta.type-of($stroke) == list and list.separator($stroke) == \"slash\" and hue(#808000) == 60deg, 1px, 2px); .card { font: $stroke; margin: $kind; }",
        OmenaParserStyleDialect::Scss,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_value_reference_count, 2);
    assert_eq!(summary.native_resolved_value_count, 1);
    assert_eq!(summary.native_raw_value_count, 1);
    assert!(
        summary
            .evaluation
            .as_ref()
            .is_some_and(|evaluation| evaluation.evaluated_css.contains("font: 1px / solid / red"))
    );
    assert!(
        summary
            .evaluation
            .as_ref()
            .is_some_and(|evaluation| evaluation.evaluated_css.contains("margin: 1px"))
    );
}

#[test]
fn exposes_static_scss_hsl_color_constructors_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "$tone: hsl(180, 100%, 50%); $overlay: hsla(120, 100%, 50%, .5); .card { color: $tone; background: $overlay; }",
        OmenaParserStyleDialect::Scss,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_value_reference_count, 2);
    assert_eq!(summary.native_resolved_value_count, 2);
    assert_eq!(summary.native_raw_value_count, 0);
    assert!(
        summary
            .evaluation
            .as_ref()
            .is_some_and(|evaluation| evaluation.evaluated_css.contains("color: #0ff"))
    );
    assert!(summary.evaluation.as_ref().is_some_and(|evaluation| {
        evaluation
            .evaluated_css
            .contains("background: rgba(0, 255, 0, 0.5)")
    }));
}

#[test]
fn exposes_static_scss_ie_hex_str_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "$legacy: ie-hex-str(rgba(red, .5)); .card { color: $legacy; }",
        OmenaParserStyleDialect::Scss,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_value_reference_count, 1);
    assert_eq!(summary.native_resolved_value_count, 1);
    assert_eq!(summary.native_raw_value_count, 0);
    assert!(
        summary
            .evaluation
            .as_ref()
            .is_some_and(|evaluation| evaluation.evaluated_css.contains("color: #80FF0000"))
    );
}

#[test]
fn exposes_static_scss_inspect_values_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "$tone: meta.inspect(red); $gap: inspect(2px); .card { color: $tone; margin: $gap; }",
        OmenaParserStyleDialect::Scss,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_value_reference_count, 2);
    assert_eq!(summary.native_resolved_value_count, 2);
    assert_eq!(summary.native_raw_value_count, 0);
    assert!(
        summary
            .evaluation
            .as_ref()
            .is_some_and(|evaluation| evaluation.evaluated_css.contains("color: red"))
    );
    assert!(
        summary
            .evaluation
            .as_ref()
            .is_some_and(|evaluation| evaluation.evaluated_css.contains("margin: 2px"))
    );
}

#[test]
fn exposes_nested_static_scss_color_helpers_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "$tone: list.nth(list.append(1px, transparentize(red, .25)), 2); $scaled: list.nth(list.append(1px, color.scale(#808000, $lightness: 50%)), 2); $opacity: list.nth(list.append(1px, color.opacity(rgba(red, .5))), 2); .card { color: $tone; background: $scaled; opacity: $opacity; }",
        OmenaParserStyleDialect::Scss,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_value_reference_count, 3);
    assert_eq!(summary.native_resolved_value_count, 3);
    assert_eq!(summary.native_raw_value_count, 0);
    assert!(summary.evaluation.as_ref().is_some_and(|evaluation| {
        evaluation
            .evaluated_css
            .contains("color: rgba(255, 0, 0, 0.75)")
    }));
    assert!(
        summary
            .evaluation
            .as_ref()
            .is_some_and(|evaluation| { evaluation.evaluated_css.contains("background: #ffff40") })
    );
    assert!(
        summary
            .evaluation
            .as_ref()
            .is_some_and(|evaluation| evaluation.evaluated_css.contains("opacity: 0.5"))
    );
}

#[test]
fn exposes_alpha_aware_scss_color_mix_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "$tone: color.mix(rgba(red, .5), blue); $nested: color.mix(transparentize(red, .25), blue); .card { color: $tone; border-color: $nested; }",
        OmenaParserStyleDialect::Scss,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_value_reference_count, 2);
    assert_eq!(summary.native_resolved_value_count, 2);
    assert_eq!(summary.native_raw_value_count, 0);
    assert!(summary.evaluation.as_ref().is_some_and(|evaluation| {
        evaluation
            .evaluated_css
            .contains("color: rgba(63.75, 0, 191.25, 0.75)")
    }));
    assert!(summary.evaluation.as_ref().is_some_and(|evaluation| {
        evaluation
            .evaluated_css
            .contains("border-color: rgba(95.625, 0, 159.375, 0.875)")
    }));
}

#[test]
fn exposes_less_static_stylesheet_evaluator_oracle_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "@gap: 2px; .card { margin: @gap; }",
        OmenaParserStyleDialect::Less,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.dialect, "less");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.supported_dialect);
    assert_eq!(summary.product_output_source, "legacyEvaluatedCss");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert!(summary.evaluation_available);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_replacement_count, 1);
    assert_eq!(summary.native_resolved_value_count, 1);
    assert!(
        summary
            .evaluation
            .as_ref()
            .is_some_and(|evaluation| evaluation.evaluated_css.contains("margin: 2px"))
    );
}

#[test]
fn exposes_less_unit_builtin_evaluator_oracle_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "@gap: unit(5, px); @plain: unit(5px); @unit-name: get-unit(1.5rem); .button { margin: @gap; padding: @plain; --unit: @unit-name; }",
        OmenaParserStyleDialect::Less,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.dialect, "less");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.supported_dialect);
    assert_eq!(summary.product_output_source, "legacyEvaluatedCss");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert!(summary.evaluation_available);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_replacement_count, 3);
    assert_eq!(summary.native_resolved_value_count, 2);
    assert_eq!(summary.native_raw_value_count, 1);
    assert_eq!(summary.native_top_value_count, 0);
    assert!(summary.evaluation.as_ref().is_some_and(|evaluation| {
        evaluation.evaluated_css.contains("margin: 5px")
            && evaluation.evaluated_css.contains("padding: 5")
            && evaluation.evaluated_css.contains("--unit: rem")
    }));
}

#[test]
fn exposes_less_percentage_and_rounding_builtins_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "@ratio: percentage(.5); @ceil: ceil(1.2px); @floor: floor(1.8px); .button { width: @ratio; top: @ceil; bottom: @floor; }",
        OmenaParserStyleDialect::Less,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.dialect, "less");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.supported_dialect);
    assert_eq!(summary.product_output_source, "legacyEvaluatedCss");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert!(summary.evaluation_available);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_replacement_count, 3);
    assert_eq!(summary.native_resolved_value_count, 3);
    assert_eq!(summary.native_raw_value_count, 0);
    assert_eq!(summary.native_top_value_count, 0);
    assert!(summary.evaluation.as_ref().is_some_and(|evaluation| {
        evaluation.evaluated_css.contains("width: 50%")
            && evaluation.evaluated_css.contains("top: 2px")
            && evaluation.evaluated_css.contains("bottom: 1px")
    }));
}

#[test]
fn exposes_less_numeric_builtins_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "@sqrt: sqrt(4); @pow: pow(2, 3); @mod: mod(11px, 4px); @min: min(1px, 2px, 3px); @max: max(1px, 2px, 3px); @abs: abs(-2.4px); @round1: round(1.6px); @round2: round(1.234px, 2); .button { sqrt: @sqrt; pow: @pow; mod: @mod; min: @min; max: @max; abs: @abs; round1: @round1; round2: @round2; }",
        OmenaParserStyleDialect::Less,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.dialect, "less");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.supported_dialect);
    assert_eq!(summary.product_output_source, "legacyEvaluatedCss");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert!(summary.evaluation_available);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_replacement_count, 8);
    assert_eq!(summary.native_resolved_value_count, 8);
    assert_eq!(summary.native_raw_value_count, 0);
    assert_eq!(summary.native_top_value_count, 0);
    assert!(summary.evaluation.as_ref().is_some_and(|evaluation| {
        evaluation.evaluated_css.contains("sqrt: 2")
            && evaluation.evaluated_css.contains("pow: 8")
            && evaluation.evaluated_css.contains("mod: 3px")
            && evaluation.evaluated_css.contains("min: 1px")
            && evaluation.evaluated_css.contains("max: 3px")
            && evaluation.evaluated_css.contains("abs: 2.4px")
            && evaluation.evaluated_css.contains("round1: 2px")
            && evaluation.evaluated_css.contains("round2: 1.23px")
    }));
}

#[test]
fn preserves_less_unsupported_css_math_builtins_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "@sign: sign(-2px); @clamp: clamp(1px, 3px, 2px); @rem: rem(11px, 4px); @hypot: hypot(3px, 4px); @exp: exp(1); @log: log(8, 2); @calc: calc(1px + 2px); .button { sign: @sign; clamp: @clamp; rem: @rem; hypot: @hypot; exp: @exp; log: @log; calc: @calc; }",
        OmenaParserStyleDialect::Less,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.dialect, "less");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.supported_dialect);
    assert_eq!(summary.product_output_source, "legacyEvaluatedCss");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert!(summary.evaluation_available);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_replacement_count, 7);
    assert_eq!(summary.native_raw_value_count, 7);
    assert_eq!(summary.native_top_value_count, 0);
    assert!(summary.evaluation.as_ref().is_some_and(|evaluation| {
        evaluation.evaluated_css.contains("sign: sign(-2px)")
            && evaluation
                .evaluated_css
                .contains("clamp: clamp(1px, 3px, 2px)")
            && evaluation.evaluated_css.contains("rem: rem(11px, 4px)")
            && evaluation.evaluated_css.contains("hypot: hypot(3px, 4px)")
            && evaluation.evaluated_css.contains("exp: exp(1)")
            && evaluation.evaluated_css.contains("log: log(8, 2)")
            && evaluation.evaluated_css.contains("calc: calc(1px + 2px)")
    }));
}

#[test]
fn exposes_less_escape_builtin_without_reentry_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "@name: e(\"hello\"); @calc: e(\"calc(1px + 2px)\"); @min: e(\"min(1px, 2px)\"); @sign: e(\"sign(-2px)\"); .button { a: @name; b: @calc; c: @min; d: @sign; }",
        OmenaParserStyleDialect::Less,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.dialect, "less");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.supported_dialect);
    assert_eq!(summary.product_output_source, "legacyEvaluatedCss");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert!(summary.evaluation_available);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_replacement_count, 4);
    assert_eq!(summary.native_resolved_value_count, 0);
    assert_eq!(summary.native_raw_value_count, 4);
    assert_eq!(summary.native_top_value_count, 0);
    assert!(summary.evaluation.as_ref().is_some_and(|evaluation| {
        evaluation.evaluated_css.contains("a: hello")
            && evaluation.evaluated_css.contains("b: calc(1px + 2px)")
            && evaluation.evaluated_css.contains("c: min(1px, 2px)")
            && evaluation.evaluated_css.contains("d: sign(-2px)")
    }));
}

#[test]
fn exposes_less_type_predicate_builtins_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "@number: isnumber(2px); @color: iscolor(red); @string: isstring(\"Roboto\"); @keyword: iskeyword(block); @url: isurl(url(\"a.png\")); @px: ispixel(2px); @pct: ispercentage(50%); @em: isem(1em); @unit-ok: isunit(1rem, rem); @unit-bad: isunit(1rem, px); .button { --number: @number; --color: @color; --string: @string; --keyword: @keyword; --url: @url; --px: @px; --pct: @pct; --em: @em; --unit-ok: @unit-ok; --unit-bad: @unit-bad; }",
        OmenaParserStyleDialect::Less,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.dialect, "less");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.supported_dialect);
    assert_eq!(summary.product_output_source, "legacyEvaluatedCss");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert!(summary.evaluation_available);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_replacement_count, 10);
    assert_eq!(summary.native_resolved_value_count, 0);
    assert_eq!(summary.native_raw_value_count, 10);
    assert_eq!(summary.native_top_value_count, 0);
    assert!(summary.evaluation.as_ref().is_some_and(|evaluation| {
        evaluation.evaluated_css.contains("--number: true")
            && evaluation.evaluated_css.contains("--unit-ok: true")
            && evaluation.evaluated_css.contains("--unit-bad: false")
    }));
}

#[test]
fn exposes_less_conditional_builtins_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "@gap: 1; @a: if(@gap > 0, red, blue); @b: if(false, red, blue); @c: if(isnumber(2px), yes, no); @d: boolean(@gap > 0); @e: if(default(), red, blue); .button { a: @a; b: @b; c: @c; d: @d; e: @e; }",
        OmenaParserStyleDialect::Less,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.dialect, "less");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.supported_dialect);
    assert_eq!(summary.product_output_source, "legacyEvaluatedCss");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert!(summary.evaluation_available);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_replacement_count, 5);
    assert_eq!(summary.native_resolved_value_count, 3);
    assert_eq!(summary.native_raw_value_count, 2);
    assert_eq!(summary.native_top_value_count, 0);
    assert!(summary.evaluation.as_ref().is_some_and(|evaluation| {
        evaluation.evaluated_css.contains("a: red")
            && evaluation.evaluated_css.contains("b: blue")
            && evaluation.evaluated_css.contains("c: yes")
            && evaluation.evaluated_css.contains("d: true")
            && evaluation.evaluated_css.contains("e: blue")
    }));
}

#[test]
fn exposes_less_color_channel_builtins_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "@color: #123456; @r: red(@color); @g: green(@color); @b: blue(@color); @a: alpha(rgba(10, 20, 30, .5)); .button { r: @r; g: @g; b: @b; a: @a; }",
        OmenaParserStyleDialect::Less,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.dialect, "less");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.supported_dialect);
    assert_eq!(summary.product_output_source, "legacyEvaluatedCss");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert!(summary.evaluation_available);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_replacement_count, 4);
    assert_eq!(summary.native_resolved_value_count, 4);
    assert_eq!(summary.native_raw_value_count, 0);
    assert_eq!(summary.native_top_value_count, 0);
    assert!(summary.evaluation.as_ref().is_some_and(|evaluation| {
        evaluation.evaluated_css.contains("r: 18")
            && evaluation.evaluated_css.contains("g: 52")
            && evaluation.evaluated_css.contains("b: 86")
            && evaluation.evaluated_css.contains("a: 0.5")
    }));
}

#[test]
fn exposes_less_color_metadata_builtins_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "@color: #123456; @h: hue(@color); @s: saturation(@color); @l: lightness(@color); @legacy: argb(rgba(18, 52, 86, .5)); .button { h: @h; s: @s; l: @l; legacy: @legacy; }",
        OmenaParserStyleDialect::Less,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.dialect, "less");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.supported_dialect);
    assert_eq!(summary.product_output_source, "legacyEvaluatedCss");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert!(summary.evaluation_available);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_replacement_count, 4);
    assert_eq!(summary.native_resolved_value_count, 4);
    assert_eq!(summary.native_raw_value_count, 0);
    assert_eq!(summary.native_top_value_count, 0);
    assert!(summary.evaluation.as_ref().is_some_and(|evaluation| {
        evaluation.evaluated_css.contains("h: 210")
            && evaluation.evaluated_css.contains("s: 65.384615%")
            && evaluation.evaluated_css.contains("l: 20.392157%")
            && evaluation.evaluated_css.contains("legacy: #80123456")
    }));
}

#[test]
fn exposes_less_alpha_transform_builtins_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "@faded: fade(#123456, 50%); @raised: fadein(rgba(18, 52, 86, .5), 10%); @lowered: fadeout(rgba(18, 52, 86, .5), 10%); @opaque: fadein(red, 10%); .button { a: @faded; b: @raised; c: @lowered; d: @opaque; }",
        OmenaParserStyleDialect::Less,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.dialect, "less");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.supported_dialect);
    assert_eq!(summary.product_output_source, "legacyEvaluatedCss");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert!(summary.evaluation_available);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_replacement_count, 4);
    assert_eq!(summary.native_resolved_value_count, 4);
    assert_eq!(summary.native_raw_value_count, 0);
    assert_eq!(summary.native_top_value_count, 0);
    assert!(summary.evaluation.as_ref().is_some_and(|evaluation| {
        evaluation
            .evaluated_css
            .contains("a: rgba(18, 52, 86, 0.5)")
            && evaluation
                .evaluated_css
                .contains("b: rgba(18, 52, 86, 0.6)")
            && evaluation
                .evaluated_css
                .contains("c: rgba(18, 52, 86, 0.4)")
            && evaluation.evaluated_css.contains("d: #ff0000")
    }));
}

#[test]
fn exposes_less_list_builtins_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "@items: a b c; @comma: a, b, c; @len1: length(@items); @len2: length(@comma); @x1: extract(@items, 2); @x2: extract(@comma, 3); .button { len1: @len1; len2: @len2; x1: @x1; x2: @x2; }",
        OmenaParserStyleDialect::Less,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.dialect, "less");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.supported_dialect);
    assert_eq!(summary.product_output_source, "legacyEvaluatedCss");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert!(summary.evaluation_available);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_replacement_count, 4);
    assert_eq!(summary.native_resolved_value_count, 2);
    assert_eq!(summary.native_raw_value_count, 2);
    assert_eq!(summary.native_top_value_count, 0);
    assert!(summary.evaluation.as_ref().is_some_and(|evaluation| {
        evaluation.evaluated_css.contains("len1: 3")
            && evaluation.evaluated_css.contains("len2: 3")
            && evaluation.evaluated_css.contains("x1: b")
            && evaluation.evaluated_css.contains("x2: c")
    }));
}

#[test]
fn exposes_less_static_mixin_evaluator_oracle_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "@brand: red; .tone(@color, @gap: 1px) { color: @color; margin: @gap; padding: @brand; } .button { .tone(blue, 2px); }",
        OmenaParserStyleDialect::Less,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.dialect, "less");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.supported_dialect);
    assert_eq!(summary.product_output_source, "legacyEvaluatedCss");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert!(summary.evaluation_available);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert!(summary.evaluation.as_ref().is_some_and(|evaluation| {
        !evaluation.evaluated_css.contains(".tone(@color")
            && !evaluation.evaluated_css.contains(".tone(blue")
            && evaluation.evaluated_css.contains("color: blue")
            && evaluation.evaluated_css.contains("margin: 2px")
            && evaluation.evaluated_css.contains("padding: red")
    }));
}

#[test]
fn exposes_less_ruleset_guarded_mixin_oracle_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        ".apply(@block) when (isruleset(@block)) { @block(); } @rules: { color: red; margin: 1px; }; .button { .apply(@rules); }",
        OmenaParserStyleDialect::Less,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.dialect, "less");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert!(summary.evaluation_available);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert!(summary.evaluation.as_ref().is_some_and(|evaluation| {
        !evaluation.evaluated_css.contains(".apply(@block")
            && !evaluation.evaluated_css.contains("@rules:")
            && !evaluation.evaluated_css.contains(".apply(@rules")
            && evaluation.evaluated_css.contains("color: red")
            && evaluation.evaluated_css.contains("margin: 1px")
    }));
}

#[test]
fn exposes_static_scss_while_argument_returns_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "@function pick($target) { $i: 0; @while $i < 3 { @if $i == $target { @return $i + 1; } $i: $i + 1; } @return 0; } .button { z-index: pick(2); }",
        OmenaParserStyleDialect::Scss,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_replacement_count, 1);
    assert_eq!(summary.native_resolved_value_count, 1);
    assert_eq!(summary.native_raw_value_count, 0);
    assert_eq!(summary.native_top_value_count, 0);
    assert!(summary.evaluation.as_ref().is_some_and(|evaluation| {
        evaluation.evaluated_css.contains(".button { z-index: 3; }")
            && evaluation.resolved_replacements.iter().any(|replacement| {
                replacement.name == "function:pick"
                    && replacement.text == "3"
                    && replacement.abstract_value_kind == "exact"
            })
    }));
}

#[test]
fn exposes_static_scss_for_loop_returns_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "@function pick($target) { @for $i from 1 through 3 { @if $i == $target { @return $i; } } @return 0; } .button { z-index: pick(2); }",
        OmenaParserStyleDialect::Scss,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_replacement_count, 1);
    assert_eq!(summary.native_resolved_value_count, 1);
    assert_eq!(summary.native_raw_value_count, 0);
    assert_eq!(summary.native_top_value_count, 0);
    assert!(summary.evaluation.as_ref().is_some_and(|evaluation| {
        evaluation.evaluated_css.contains(".button { z-index: 2; }")
            && evaluation.resolved_replacements.iter().any(|replacement| {
                replacement.name == "function:pick"
                    && replacement.text == "2"
                    && replacement.abstract_value_kind == "exact"
            })
    }));
}

#[test]
fn exposes_static_scss_each_loop_returns_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "@function tone($target) { @each $name, $color in (primary: red, secondary: blue) { @if $name == $target { @return $color; } } @return black; } .button { color: tone(secondary); }",
        OmenaParserStyleDialect::Scss,
    );

    assert_eq!(summary.product, "omena-query.static-stylesheet-evaluator");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.legacy_output_consumed_until_cutover);
    assert_eq!(summary.divergence_count, 0);
    assert!(summary.all_legacy_declaration_values_preserved);
    assert_eq!(summary.native_replacement_count, 1);
    assert_eq!(summary.native_resolved_value_count, 1);
    assert_eq!(summary.native_raw_value_count, 0);
    assert_eq!(summary.native_top_value_count, 0);
    assert!(summary.evaluation.as_ref().is_some_and(|evaluation| {
        evaluation
            .evaluated_css
            .contains(".button { color: blue; }")
            && evaluation.resolved_replacements.iter().any(|replacement| {
                replacement.name == "function:tone"
                    && replacement.text == "blue"
                    && replacement.abstract_value_kind == "exact"
            })
    }));
}

#[test]
fn exposes_scss_evaluator_control_flow_oracle_through_query_boundary() {
    let source = r#"
$enabled: true;
@if $enabled {
  .on { color: green; }
}
@function space($input) {
  @return $input + 1px;
}
.card { margin: space(2px); }
"#;

    let summary = summarize_omena_query_scss_evaluator_control_flow_from_source(
        source,
        OmenaParserStyleDialect::Scss,
    );

    assert_eq!(summary.schema_version, "0");
    assert_eq!(summary.product, "omena-query.scss-evaluator-control-flow");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.dialect, "scss");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.supported_dialect);
    assert!(!summary.flat_css_cfg_built);
    assert!(!summary.merged_cross_file_graph);
    assert!(summary.control_flow_ir.is_some());
    assert!(summary.value_analysis.is_some());
    assert!(summary.call_return_ir.is_some());
    assert!(summary.control_flow_branch_block_count >= 1);
    assert!(summary.call_return_node_count >= 3);
    assert!(summary.call_resolved_return_value_count >= 1);
    assert!(summary.exact_call_resolved_return_value_count >= 1);
    assert!(summary.value_analysis_converged);
    assert!(
        summary
            .ready_surfaces
            .contains(&"scssEvaluatorControlFlowValueAnalysis")
    );

    if let Some(control_flow) = summary.control_flow_ir.as_ref() {
        assert_eq!(control_flow.node_key_type, "StableNodeKeyV0");
        assert!(!control_flow.flat_css_cfg_built);
        assert!(!control_flow.merged_cross_file_graph);
    }

    if let Some(value_analysis) = summary.value_analysis.as_ref() {
        assert_eq!(value_analysis.value_type, "AbstractCssValueV0");
        assert!(!value_analysis.flat_css_cfg_built);
        assert!(!value_analysis.merged_cross_file_graph);
    }

    if let Some(call_return) = summary.call_return_ir.as_ref() {
        assert_eq!(call_return.node_key_type, "StableNodeKeyV0");
        assert!(!call_return.flat_css_cfg_built);
        assert!(!call_return.merged_cross_file_graph);
    }
}

#[test]
fn exposes_static_while_bindings_through_query_boundary() -> Result<(), serde_json::Error> {
    let source = "$i: 0; @while $i < 3 { $i: $i + 1; .n { order: $i; } }";

    let summary = summarize_omena_query_scss_evaluator_control_flow_from_source(
        source,
        OmenaParserStyleDialect::Scss,
    );

    assert_eq!(summary.schema_version, "0");
    assert_eq!(summary.product, "omena-query.scss-evaluator-control-flow");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.supported_dialect);
    assert_eq!(summary.control_flow_loop_block_count, 1);
    assert_eq!(summary.control_flow_back_edge_count, 1);
    assert!(summary.value_analysis_converged);

    assert!(summary.value_analysis.is_some());
    let Some(value_analysis) = summary.value_analysis.as_ref() else {
        return Ok(());
    };
    assert_eq!(value_analysis.blocks[0].loop_carried_bindings, vec!["$i"]);
    assert_eq!(
        serde_json::to_value(&value_analysis.blocks[0].loop_carried_binding_values[0].value)?,
        serde_json::json!({
            "kind": "finiteSet",
            "values": ["0", "1", "2"],
        })
    );
    Ok(())
}

#[test]
fn exposes_static_while_bound_variable_bindings_through_query_boundary()
-> Result<(), serde_json::Error> {
    let source = "$end: 3; $i: 0; @while $i < $end { $i: $i + 1; .n { order: $i; } }";

    let summary = summarize_omena_query_scss_evaluator_control_flow_from_source(
        source,
        OmenaParserStyleDialect::Scss,
    );

    assert_eq!(summary.schema_version, "0");
    assert_eq!(summary.product, "omena-query.scss-evaluator-control-flow");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.supported_dialect);
    assert_eq!(summary.control_flow_loop_block_count, 1);
    assert_eq!(summary.control_flow_back_edge_count, 1);
    assert!(summary.value_analysis_converged);

    assert!(summary.value_analysis.is_some());
    let Some(value_analysis) = summary.value_analysis.as_ref() else {
        return Ok(());
    };
    assert_eq!(value_analysis.blocks[0].loop_carried_bindings, vec!["$i"]);
    assert_eq!(
        serde_json::to_value(&value_analysis.blocks[0].loop_carried_binding_values[0].value)?,
        serde_json::json!({
            "kind": "finiteSet",
            "values": ["0", "1", "2"],
        })
    );
    Ok(())
}

#[test]
fn exposes_static_while_call_return_values_through_query_boundary() -> Result<(), serde_json::Error>
{
    let source = "@function collect() { $i: 0; @while $i < 3 { @return $i; $i: $i + 1; } } .a { width: collect(); }";

    let summary = summarize_omena_query_scss_evaluator_control_flow_from_source(
        source,
        OmenaParserStyleDialect::Scss,
    );

    assert_eq!(summary.schema_version, "0");
    assert_eq!(summary.product, "omena-query.scss-evaluator-control-flow");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.supported_dialect);
    assert_eq!(summary.control_flow_loop_block_count, 1);
    assert_eq!(summary.control_flow_back_edge_count, 1);
    assert_eq!(summary.call_resolved_return_value_count, 1);
    assert_eq!(summary.exact_call_resolved_return_value_count, 0);

    assert!(summary.call_return_ir.is_some());
    let Some(call_return) = summary.call_return_ir.as_ref() else {
        return Ok(());
    };
    let function_call = call_return
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("collect"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return Ok(());
    };
    assert_eq!(
        function_call.call_resolved_return_value_kind,
        Some("finiteSet")
    );
    assert_eq!(
        serde_json::to_value(function_call.call_resolved_return_value.as_ref())?,
        serde_json::json!({
            "kind": "finiteSet",
            "values": ["0", "1", "2"],
        })
    );
    Ok(())
}

#[test]
fn exposes_static_while_conditional_call_return_values_through_query_boundary()
-> Result<(), serde_json::Error> {
    let source = "@function collect() { $i: 0; @while $i < 3 { @if $i == 2 { @return $i; } $i: $i + 1; } @return 0; } .a { width: collect(); }";

    let summary = summarize_omena_query_scss_evaluator_control_flow_from_source(
        source,
        OmenaParserStyleDialect::Scss,
    );

    assert_eq!(summary.schema_version, "0");
    assert_eq!(summary.product, "omena-query.scss-evaluator-control-flow");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.supported_dialect);
    assert_eq!(summary.control_flow_loop_block_count, 1);
    assert_eq!(summary.control_flow_back_edge_count, 1);
    assert_eq!(summary.call_resolved_return_value_count, 1);
    assert_eq!(summary.exact_call_resolved_return_value_count, 1);

    assert!(summary.call_return_ir.is_some());
    let Some(call_return) = summary.call_return_ir.as_ref() else {
        return Ok(());
    };
    let function_call = call_return
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("collect"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return Ok(());
    };
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        serde_json::to_value(function_call.call_resolved_return_value.as_ref())?,
        serde_json::json!({
            "kind": "exact",
            "value": "2",
        })
    );
    Ok(())
}

#[test]
fn exposes_static_for_conditional_call_return_values_through_query_boundary()
-> Result<(), serde_json::Error> {
    let source = "@function collect($target) { @for $i from 1 through 3 { @if $i == $target { @return $i; } } @return 0; } .a { width: collect(2); }";

    let summary = summarize_omena_query_scss_evaluator_control_flow_from_source(
        source,
        OmenaParserStyleDialect::Scss,
    );

    assert_eq!(summary.schema_version, "0");
    assert_eq!(summary.product, "omena-query.scss-evaluator-control-flow");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.supported_dialect);
    assert_eq!(summary.control_flow_loop_block_count, 1);
    assert_eq!(summary.control_flow_back_edge_count, 1);
    assert_eq!(summary.call_resolved_return_value_count, 1);
    assert_eq!(summary.exact_call_resolved_return_value_count, 1);

    assert!(summary.call_return_ir.is_some());
    let Some(call_return) = summary.call_return_ir.as_ref() else {
        return Ok(());
    };
    let function_call = call_return
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("collect"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return Ok(());
    };
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        serde_json::to_value(function_call.call_resolved_return_value.as_ref())?,
        serde_json::json!({
            "kind": "exact",
            "value": "2",
        })
    );
    Ok(())
}

#[test]
fn exposes_static_each_conditional_call_return_values_through_query_boundary()
-> Result<(), serde_json::Error> {
    let source = "@function tone($target) { @each $name, $tone in (primary: red, secondary: blue) { @if $name == $target { @return $tone; } } @return black; } .a { color: tone(secondary); }";

    let summary = summarize_omena_query_scss_evaluator_control_flow_from_source(
        source,
        OmenaParserStyleDialect::Scss,
    );

    assert_eq!(summary.schema_version, "0");
    assert_eq!(summary.product, "omena-query.scss-evaluator-control-flow");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.supported_dialect);
    assert_eq!(summary.control_flow_loop_block_count, 1);
    assert_eq!(summary.control_flow_back_edge_count, 1);
    assert_eq!(summary.call_resolved_return_value_count, 1);
    assert_eq!(summary.exact_call_resolved_return_value_count, 1);

    assert!(summary.call_return_ir.is_some());
    let Some(call_return) = summary.call_return_ir.as_ref() else {
        return Ok(());
    };
    let function_call = call_return
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("tone"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return Ok(());
    };
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        serde_json::to_value(function_call.call_resolved_return_value.as_ref())?,
        serde_json::json!({
            "kind": "exact",
            "value": "#00f",
        })
    );
    Ok(())
}

#[test]
fn keeps_dynamic_loop_call_return_values_top_through_query_boundary()
-> Result<(), serde_json::Error> {
    let source = "@function collect($count) { @for $i from 1 through $count { @return $i; } } .a { width: collect(var(--count)); }";

    let summary = summarize_omena_query_scss_evaluator_control_flow_from_source(
        source,
        OmenaParserStyleDialect::Scss,
    );

    assert_eq!(summary.schema_version, "0");
    assert_eq!(summary.product, "omena-query.scss-evaluator-control-flow");
    assert_eq!(summary.mode, "oracleOnly");
    assert_eq!(summary.value_type, "AbstractCssValueV0");
    assert!(summary.supported_dialect);
    assert_eq!(summary.control_flow_loop_block_count, 1);
    assert_eq!(summary.control_flow_back_edge_count, 1);
    assert_eq!(summary.call_resolved_return_value_count, 1);
    assert_eq!(summary.exact_call_resolved_return_value_count, 0);

    assert!(summary.call_return_ir.is_some());
    let Some(call_return) = summary.call_return_ir.as_ref() else {
        return Ok(());
    };
    assert_eq!(call_return.top_call_resolved_return_value_count, 1);
    let function_call = call_return
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("collect"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return Ok(());
    };
    assert_eq!(function_call.call_resolved_return_value_kind, Some("top"));
    assert_eq!(
        serde_json::to_value(function_call.call_resolved_return_value.as_ref())?,
        serde_json::json!({
            "kind": "top",
        })
    );
    Ok(())
}

#[test]
fn keeps_plain_css_out_of_scss_evaluator_control_flow_oracle() {
    let summary = summarize_omena_query_scss_evaluator_control_flow_from_source(
        ".card { color: red; }",
        OmenaParserStyleDialect::Css,
    );

    assert_eq!(summary.product, "omena-query.scss-evaluator-control-flow");
    assert_eq!(summary.dialect, "css");
    assert!(!summary.supported_dialect);
    assert_eq!(summary.control_flow_block_count, 0);
    assert_eq!(summary.call_return_node_count, 0);
    assert!(!summary.value_analysis_converged);
    assert!(summary.control_flow_ir.is_none());
    assert!(summary.value_analysis.is_none());
    assert!(summary.call_return_ir.is_none());
}
