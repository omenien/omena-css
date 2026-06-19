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
    assert_eq!(summary.fixture_count, 21);
    assert_eq!(summary.scss_fixture_count, 6);
    assert_eq!(summary.less_fixture_count, 15);
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
fn exposes_less_convert_builtin_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "@cm: convert(1in, cm); @inch: convert(2.54cm, in); @px: convert(96px, in); @ms: convert(1s, ms); @sec: convert(250ms, s); @deg: convert(1rad, deg); @turn: convert(.5turn, deg); @same: convert(1in, s); .button { cm: @cm; inch: @inch; px: @px; ms: @ms; sec: @sec; deg: @deg; turn: @turn; same: @same; }",
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
    assert_eq!(summary.native_resolved_value_count, 6);
    assert_eq!(summary.native_raw_value_count, 2);
    assert_eq!(summary.native_top_value_count, 0);
    assert!(summary.evaluation.as_ref().is_some_and(|evaluation| {
        evaluation.evaluated_css.contains("cm: 2.54cm")
            && evaluation.evaluated_css.contains("inch: 1in")
            && evaluation.evaluated_css.contains("px: 1in")
            && evaluation.evaluated_css.contains("ms: 1000ms")
            && evaluation.evaluated_css.contains("sec: 0.25s")
            && evaluation.evaluated_css.contains("deg: 57.29577951deg")
            && evaluation.evaluated_css.contains("turn: 180deg")
            && evaluation.evaluated_css.contains("same: 1in")
    }));
}

#[test]
fn exposes_less_trig_builtins_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "@pi: pi(); @sin: sin(30deg); @sinRad: sin(1rad); @sinUnitless: sin(1); @cos: cos(60deg); @tan: tan(45deg); @asin: asin(.5); @acos: acos(.5); @atan: atan(1); .button { pi: @pi; sin: @sin; sin-rad: @sinRad; sin-unitless: @sinUnitless; cos: @cos; tan: @tan; asin: @asin; acos: @acos; atan: @atan; }",
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
    assert_eq!(summary.native_replacement_count, 9);
    assert_eq!(summary.native_resolved_value_count, 9);
    assert_eq!(summary.native_raw_value_count, 0);
    assert_eq!(summary.native_top_value_count, 0);
    assert!(summary.evaluation.as_ref().is_some_and(|evaluation| {
        evaluation.evaluated_css.contains("pi: 3.14159265")
            && evaluation.evaluated_css.contains("sin: 0.5")
            && evaluation.evaluated_css.contains("sin-rad: 0.84147098")
            && evaluation
                .evaluated_css
                .contains("sin-unitless: 0.84147098")
            && evaluation.evaluated_css.contains("cos: 0.5")
            && evaluation.evaluated_css.contains("tan: 1")
            && evaluation.evaluated_css.contains("asin: 0.52359878rad")
            && evaluation.evaluated_css.contains("acos: 1.04719755rad")
            && evaluation.evaluated_css.contains("atan: 0.78539816rad")
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
fn exposes_less_url_escape_builtin_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "@query: escape(\"a=1\"); @space: escape(\"hello world\"); @hash: escape(\"#fff\"); @unicode: escape(\"ä\"); @fn: escape(\"min(1px, 2px)\"); .button { a: @query; b: @space; c: @hash; d: @unicode; e: @fn; }",
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
    assert_eq!(summary.native_resolved_value_count, 0);
    assert_eq!(summary.native_raw_value_count, 5);
    assert_eq!(summary.native_top_value_count, 0);
    assert!(summary.evaluation.as_ref().is_some_and(|evaluation| {
        evaluation.evaluated_css.contains("a: a%3D1")
            && evaluation.evaluated_css.contains("b: hello%20world")
            && evaluation.evaluated_css.contains("c: %23fff")
            && evaluation.evaluated_css.contains("d: %C3%A4")
            && evaluation.evaluated_css.contains("e: min%281px,%202px%29")
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
            && evaluation.evaluated_css.contains("s: 65.38461538%")
            && evaluation.evaluated_css.contains("l: 20.39215686%")
            && evaluation.evaluated_css.contains("legacy: #80123456")
    }));
}

#[test]
fn exposes_less_hsv_color_metadata_builtins_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "@hsv: hsv(210, 60%, 40%); @hsvUnitless: hsv(60, .6, .4); @hsva: hsva(210, 60%, 40%, 50%); @color: #123456; @h: hsvhue(@color); @s: hsvsaturation(@color); @v: hsvvalue(@color); @luma: luma(rgba(18, 52, 86, .5)); @lum: luminance(rgba(18, 52, 86, .5)); .button { hsv: @hsv; hsv-unitless: @hsvUnitless; hsva: @hsva; h: @h; s: @s; v: @v; luma: @luma; luminance: @lum; }",
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
        evaluation.evaluated_css.contains("hsv: #294766")
            && evaluation.evaluated_css.contains("hsv-unitless: #666629")
            && evaluation
                .evaluated_css
                .contains("hsva: rgba(41, 71, 102, 0.5)")
            && evaluation.evaluated_css.contains("h: 210")
            && evaluation.evaluated_css.contains("s: 79.06976744%")
            && evaluation.evaluated_css.contains("v: 33.7254902%")
            && evaluation.evaluated_css.contains("luma: 1.62823344%")
            && evaluation.evaluated_css.contains("luminance: 9.26007843%")
    }));
}

#[test]
fn exposes_less_contrast_and_color_builtins_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "@dark: contrast(#123456); @light: contrast(#eeeeee); @custom: contrast(#123456, #111111, #eeeeee); @threshold: contrast(#888888, #111111, #eeeeee, 60%); @hex: color(\"#123456\"); @short: color(\"#abc\"); @alpha: color(\"#12345680\"); @kw: color(red); .button { dark: @dark; light: @light; custom: @custom; threshold: @threshold; hex: @hex; short: @short; alpha: @alpha; kw: @kw; }",
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
        evaluation.evaluated_css.contains("dark: #ffffff")
            && evaluation.evaluated_css.contains("light: #000000")
            && evaluation.evaluated_css.contains("custom: #eeeeee")
            && evaluation.evaluated_css.contains("threshold: #eeeeee")
            && evaluation.evaluated_css.contains("hex: #123456")
            && evaluation.evaluated_css.contains("short: #abc")
            && evaluation.evaluated_css.contains("alpha: #12345680")
            && evaluation.evaluated_css.contains("kw: #ff0000")
    }));
}

#[test]
fn exposes_less_alpha_transform_builtins_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "@faded: fade(#123456, 50%); @raised: fadein(rgba(18, 52, 86, .5), 10%); @lowered: fadeout(rgba(18, 52, 86, .5), 10%); @raisedRel: fadein(rgba(18, 52, 86, .5), 10%, relative); @loweredRel: fadeout(rgba(18, 52, 86, .5), 10%, relative); @opaque: fadein(red, 10%); .button { a: @faded; b: @raised; c: @lowered; d: @opaque; e: @raisedRel; f: @loweredRel; }",
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
    assert_eq!(summary.native_replacement_count, 6);
    assert_eq!(summary.native_resolved_value_count, 6);
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
            && evaluation
                .evaluated_css
                .contains("e: rgba(18, 52, 86, 0.55)")
            && evaluation
                .evaluated_css
                .contains("f: rgba(18, 52, 86, 0.45)")
    }));
}

#[test]
fn exposes_less_hsl_color_transform_builtins_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "@light: lighten(#123456, 10%); @dark: darken(#123456, 10%); @sat: saturate(#123456, 10%); @desat: desaturate(#123456, 10%); @lightRel: lighten(#123456, 10%, relative); @darkRel: darken(#123456, 10%, relative); @satRel: saturate(#123456, 10%, relative); @desatRel: desaturate(#123456, 10%, relative); @spin: spin(#123456, 10); @gray: greyscale(#123456); @alpha: lighten(rgba(18, 52, 86, .5), 10%); .button { light: @light; dark: @dark; sat: @sat; desat: @desat; light-rel: @lightRel; dark-rel: @darkRel; sat-rel: @satRel; desat-rel: @desatRel; spin: @spin; gray: @gray; alpha: @alpha; }",
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
    assert_eq!(summary.native_replacement_count, 11);
    assert_eq!(summary.native_resolved_value_count, 11);
    assert_eq!(summary.native_raw_value_count, 0);
    assert_eq!(summary.native_top_value_count, 0);
    assert!(summary.evaluation.as_ref().is_some_and(|evaluation| {
        evaluation.evaluated_css.contains("light: #1b4d80")
            && evaluation.evaluated_css.contains("dark: #091a2c")
            && evaluation.evaluated_css.contains("sat: #0d345b")
            && evaluation.evaluated_css.contains("desat: #173451")
            && evaluation.evaluated_css.contains("light-rel: #14395f")
            && evaluation.evaluated_css.contains("dark-rel: #102f4d")
            && evaluation.evaluated_css.contains("sat-rel: #0f3459")
            && evaluation.evaluated_css.contains("desat-rel: #153453")
            && evaluation.evaluated_css.contains("spin: #122956")
            && evaluation.evaluated_css.contains("gray: #343434")
            && evaluation
                .evaluated_css
                .contains("alpha: rgba(27, 77, 128, 0.5)")
    }));
}

#[test]
fn exposes_less_color_mix_builtins_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "@default: mix(red, blue); @weighted: mix(red, blue, 25%); @tinted: tint(#123456, 10%); @shaded: shade(#123456, 10%); @alpha: mix(rgba(255, 0, 0, .5), blue, 50%); @transparent: mix(transparent, red, 50%); .button { default: @default; weighted: @weighted; tinted: @tinted; shaded: @shaded; alpha: @alpha; transparent: @transparent; }",
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
    assert_eq!(summary.native_replacement_count, 6);
    assert_eq!(summary.native_resolved_value_count, 6);
    assert_eq!(summary.native_raw_value_count, 0);
    assert_eq!(summary.native_top_value_count, 0);
    assert!(summary.evaluation.as_ref().is_some_and(|evaluation| {
        evaluation.evaluated_css.contains("default: #800080")
            && evaluation.evaluated_css.contains("weighted: #4000bf")
            && evaluation.evaluated_css.contains("tinted: #2a4867")
            && evaluation.evaluated_css.contains("shaded: #102f4d")
            && evaluation
                .evaluated_css
                .contains("alpha: rgba(64, 0, 191, 0.75)")
            && evaluation
                .evaluated_css
                .contains("transparent: rgba(255, 0, 0, 0.5)")
    }));
}

#[test]
fn exposes_less_color_blend_builtins_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "@multiply: multiply(red, blue); @screen: screen(red, blue); @overlay: overlay(#123456, #abcdef); @softlight: softlight(#123456, #abcdef); @hardlight: hardlight(#123456, #abcdef); @difference: difference(#123456, #abcdef); @exclusion: exclusion(#123456, #abcdef); @average: average(#123456, #abcdef); @negation: negation(#123456, #abcdef); .button { multiply: @multiply; screen: @screen; overlay: @overlay; softlight: @softlight; hardlight: @hardlight; difference: @difference; exclusion: @exclusion; average: @average; negation: @negation; }",
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
    assert_eq!(summary.native_replacement_count, 9);
    assert_eq!(summary.native_resolved_value_count, 9);
    assert_eq!(summary.native_raw_value_count, 0);
    assert_eq!(summary.native_top_value_count, 0);
    assert!(summary.evaluation.as_ref().is_some_and(|evaluation| {
        evaluation.evaluated_css.contains("multiply: #000000")
            && evaluation.evaluated_css.contains("screen: #ff00ff")
            && evaluation.evaluated_css.contains("overlay: #1854a1")
            && evaluation.evaluated_css.contains("softlight: #205b8c")
            && evaluation.evaluated_css.contains("hardlight: #63afea")
            && evaluation.evaluated_css.contains("difference: #999999")
            && evaluation.evaluated_css.contains("exclusion: #a5ada4")
            && evaluation.evaluated_css.contains("average: #5f81a3")
            && evaluation.evaluated_css.contains("negation: #bdfdb9")
    }));
}

#[test]
fn exposes_less_alpha_color_blend_builtins_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "@multiply: multiply(rgba(255, 102, 0, .5), #0000ff); @screen: screen(rgba(255, 102, 0, .5), rgba(0, 0, 255, .25)); @overlay: overlay(rgba(255, 102, 0, .5), rgba(0, 0, 255, .25)); @softlight: softlight(rgba(255, 102, 0, .5), rgba(0, 0, 255, .25)); @hardlight: hardlight(rgba(255, 102, 0, .5), rgba(0, 0, 255, .25)); @difference: difference(rgba(255, 102, 0, .5), rgba(0, 0, 255, .25)); @exclusion: exclusion(rgba(255, 102, 0, .5), rgba(0, 0, 255, .25)); @average: average(rgba(255, 102, 0, .5), rgba(0, 0, 255, .25)); @negation: negation(rgba(255, 102, 0, .5), rgba(0, 0, 255, .25)); @both: multiply(transparent, transparent); @transparent: multiply(transparent, #0000ff); @sourceTransparent: screen(#ff6600, transparent); @transparentAverage: average(transparent, #ff6600); .button { multiply: @multiply; screen: @screen; overlay: @overlay; softlight: @softlight; hardlight: @hardlight; difference: @difference; exclusion: @exclusion; average: @average; negation: @negation; both: @both; transparent: @transparent; source-transparent: @sourceTransparent; transparent-average: @transparentAverage; }",
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
    assert_eq!(summary.native_replacement_count, 13);
    assert_eq!(summary.native_resolved_value_count, 13);
    assert_eq!(summary.native_raw_value_count, 0);
    assert_eq!(summary.native_top_value_count, 0);
    assert!(summary.evaluation.as_ref().is_some_and(|evaluation| {
        evaluation.evaluated_css.contains("multiply: #000080")
            && evaluation
                .evaluated_css
                .contains("screen: rgba(204, 82, 102, 0.625)")
            && evaluation
                .evaluated_css
                .contains("overlay: rgba(204, 61, 51, 0.625)")
            && evaluation
                .evaluated_css
                .contains("softlight: rgba(204, 69, 51, 0.625)")
            && evaluation
                .evaluated_css
                .contains("hardlight: rgba(153, 61, 102, 0.625)")
            && evaluation
                .evaluated_css
                .contains("difference: rgba(204, 82, 102, 0.625)")
            && evaluation
                .evaluated_css
                .contains("exclusion: rgba(204, 82, 102, 0.625)")
            && evaluation
                .evaluated_css
                .contains("average: rgba(179, 71, 77, 0.625)")
            && evaluation
                .evaluated_css
                .contains("negation: rgba(204, 82, 102, 0.625)")
            && evaluation.evaluated_css.contains("both: rgba(0, 0, 0, 0)")
            && evaluation.evaluated_css.contains("transparent: #0000ff")
            && evaluation
                .evaluated_css
                .contains("source-transparent: #ff6600")
            && evaluation
                .evaluated_css
                .contains("transparent-average: #ff6600")
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
fn exposes_less_range_builtin_through_query_boundary() {
    let summary = summarize_omena_query_static_stylesheet_evaluator_from_source(
        "@items: range(4); @gaps: range(1px, 5px, 2); @half: range(1, 2, .5); @empty: range(3, 1); .button { a: @items; b: @gaps; c: @half; d: @empty; }",
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
    assert_eq!(summary.native_raw_value_count, 3);
    assert_eq!(summary.native_top_value_count, 0);
    assert!(summary.evaluation.as_ref().is_some_and(|evaluation| {
        evaluation.evaluated_css.contains("a: 1 2 3 4")
            && evaluation.evaluated_css.contains("b: 1px 3px 5px")
            && evaluation.evaluated_css.contains("c: 1 1.5 2")
            && evaluation.evaluated_css.contains("d: ;")
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
