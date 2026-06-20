use omena_parser::StyleDialect;
use serde::Serialize;

use super::{derive_static_stylesheet_module_evaluation, dialect_label};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalStaticStylesheetOracleCorpusReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub mode: &'static str,
    pub value_type: &'static str,
    pub product_output_source: &'static str,
    pub legacy_output_retained_as_oracle_count: usize,
    pub legacy_output_consumed_until_cutover_count: usize,
    pub all_legacy_outputs_retained_as_oracle: bool,
    pub fixture_count: usize,
    pub scss_fixture_count: usize,
    pub sass_fixture_count: usize,
    pub less_fixture_count: usize,
    pub evaluated_fixture_count: usize,
    pub missing_evaluation_count: usize,
    pub divergence_count: usize,
    pub native_replacement_count: usize,
    pub native_replacement_legacy_reflection_count: usize,
    pub native_replacement_legacy_unreflected_count: usize,
    pub native_edit_count: usize,
    pub native_value_edit_count: usize,
    pub native_structural_edit_count: usize,
    pub native_edit_output_match_count: usize,
    pub native_value_reference_count: usize,
    pub native_resolved_value_count: usize,
    pub native_raw_value_count: usize,
    pub native_top_value_count: usize,
    pub native_cycle_value_count: usize,
    pub native_fuel_exhausted_value_count: usize,
    pub native_unresolved_reference_value_count: usize,
    pub native_unsupported_dynamic_value_count: usize,
    pub all_legacy_declaration_values_preserved: bool,
    pub all_native_edit_outputs_match_evaluated_css: bool,
    pub native_product_output_corpus_ready: bool,
    pub fixtures: Vec<OmenaScssEvalStaticStylesheetOracleCorpusFixtureReportV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalStaticStylesheetOracleCorpusFixtureReportV0 {
    pub id: &'static str,
    pub dialect: &'static str,
    pub evaluator: &'static str,
    pub product_output_source: &'static str,
    pub legacy_output_retained_as_oracle: bool,
    pub legacy_output_consumed_until_cutover: bool,
    pub evaluation_available: bool,
    pub native_edit_output: Option<String>,
    pub divergence_count: usize,
    pub all_legacy_declaration_values_preserved: bool,
    pub native_replacement_count: usize,
    pub native_replacement_legacy_reflection_count: usize,
    pub native_replacement_legacy_unreflected_count: usize,
    pub native_edit_count: usize,
    pub native_value_edit_count: usize,
    pub native_structural_edit_count: usize,
    pub native_edit_output_matches_evaluated_css: bool,
    pub native_value_reference_count: usize,
    pub native_resolved_value_count: usize,
    pub native_raw_value_count: usize,
    pub native_top_value_count: usize,
    pub native_cycle_value_count: usize,
    pub native_fuel_exhausted_value_count: usize,
    pub native_unresolved_reference_value_count: usize,
    pub native_unsupported_dynamic_value_count: usize,
}

struct StaticStylesheetOracleCorpusFixtureV0 {
    id: &'static str,
    dialect: StyleDialect,
    source: &'static str,
}

pub fn summarize_static_stylesheet_oracle_corpus()
-> OmenaScssEvalStaticStylesheetOracleCorpusReportV0 {
    let fixtures = static_stylesheet_oracle_corpus_fixtures()
        .iter()
        .map(static_stylesheet_oracle_corpus_fixture_report)
        .collect::<Vec<_>>();
    let fixture_count = fixtures.len();
    let scss_fixture_count = fixtures
        .iter()
        .filter(|fixture| fixture.dialect == "scss")
        .count();
    let sass_fixture_count = fixtures
        .iter()
        .filter(|fixture| fixture.dialect == "sass")
        .count();
    let less_fixture_count = fixtures
        .iter()
        .filter(|fixture| fixture.dialect == "less")
        .count();
    let evaluated_fixture_count = fixtures
        .iter()
        .filter(|fixture| fixture.evaluation_available)
        .count();
    let missing_evaluation_count = fixture_count.saturating_sub(evaluated_fixture_count);
    let divergence_count = fixtures
        .iter()
        .map(|fixture| fixture.divergence_count)
        .sum();
    let native_replacement_count = fixtures
        .iter()
        .map(|fixture| fixture.native_replacement_count)
        .sum();
    let native_replacement_legacy_reflection_count = fixtures
        .iter()
        .map(|fixture| fixture.native_replacement_legacy_reflection_count)
        .sum();
    let native_replacement_legacy_unreflected_count = fixtures
        .iter()
        .map(|fixture| fixture.native_replacement_legacy_unreflected_count)
        .sum();
    let native_edit_count = fixtures
        .iter()
        .map(|fixture| fixture.native_edit_count)
        .sum();
    let native_value_edit_count = fixtures
        .iter()
        .map(|fixture| fixture.native_value_edit_count)
        .sum();
    let native_structural_edit_count = fixtures
        .iter()
        .map(|fixture| fixture.native_structural_edit_count)
        .sum();
    let native_edit_output_match_count = fixtures
        .iter()
        .filter(|fixture| fixture.native_edit_output_matches_evaluated_css)
        .count();
    let legacy_output_retained_as_oracle_count = fixtures
        .iter()
        .filter(|fixture| fixture.legacy_output_retained_as_oracle)
        .count();
    let legacy_output_consumed_until_cutover_count = fixtures
        .iter()
        .filter(|fixture| fixture.legacy_output_consumed_until_cutover)
        .count();
    let native_value_reference_count = fixtures
        .iter()
        .map(|fixture| fixture.native_value_reference_count)
        .sum();
    let native_resolved_value_count = fixtures
        .iter()
        .map(|fixture| fixture.native_resolved_value_count)
        .sum();
    let native_raw_value_count = fixtures
        .iter()
        .map(|fixture| fixture.native_raw_value_count)
        .sum();
    let native_top_value_count = fixtures
        .iter()
        .map(|fixture| fixture.native_top_value_count)
        .sum();
    let native_cycle_value_count = fixtures
        .iter()
        .map(|fixture| fixture.native_cycle_value_count)
        .sum();
    let native_fuel_exhausted_value_count = fixtures
        .iter()
        .map(|fixture| fixture.native_fuel_exhausted_value_count)
        .sum();
    let native_unresolved_reference_value_count = fixtures
        .iter()
        .map(|fixture| fixture.native_unresolved_reference_value_count)
        .sum();
    let native_unsupported_dynamic_value_count = fixtures
        .iter()
        .map(|fixture| fixture.native_unsupported_dynamic_value_count)
        .sum();
    let all_legacy_declaration_values_preserved = missing_evaluation_count == 0
        && fixtures
            .iter()
            .all(|fixture| fixture.all_legacy_declaration_values_preserved);
    let all_native_edit_outputs_match_evaluated_css = missing_evaluation_count == 0
        && fixtures
            .iter()
            .all(|fixture| fixture.native_edit_output_matches_evaluated_css);
    let all_legacy_outputs_retained_as_oracle = missing_evaluation_count == 0
        && fixtures
            .iter()
            .all(|fixture| fixture.legacy_output_retained_as_oracle);
    let native_product_output_corpus_ready = missing_evaluation_count == 0
        && divergence_count == 0
        && legacy_output_consumed_until_cutover_count == 0
        && all_legacy_outputs_retained_as_oracle
        && all_legacy_declaration_values_preserved
        && all_native_edit_outputs_match_evaluated_css
        && fixtures
            .iter()
            .all(|fixture| fixture.product_output_source == "nativeEditOutput");

    OmenaScssEvalStaticStylesheetOracleCorpusReportV0 {
        schema_version: "0",
        product: "omena-scss-eval.static-stylesheet-oracle-corpus",
        mode: "oracleOnly",
        value_type: "AbstractCssValueV0",
        product_output_source: "nativeEditOutput",
        legacy_output_retained_as_oracle_count,
        legacy_output_consumed_until_cutover_count,
        all_legacy_outputs_retained_as_oracle,
        fixture_count,
        scss_fixture_count,
        sass_fixture_count,
        less_fixture_count,
        evaluated_fixture_count,
        missing_evaluation_count,
        divergence_count,
        native_replacement_count,
        native_replacement_legacy_reflection_count,
        native_replacement_legacy_unreflected_count,
        native_edit_count,
        native_value_edit_count,
        native_structural_edit_count,
        native_edit_output_match_count,
        native_value_reference_count,
        native_resolved_value_count,
        native_raw_value_count,
        native_top_value_count,
        native_cycle_value_count,
        native_fuel_exhausted_value_count,
        native_unresolved_reference_value_count,
        native_unsupported_dynamic_value_count,
        all_legacy_declaration_values_preserved,
        all_native_edit_outputs_match_evaluated_css,
        native_product_output_corpus_ready,
        fixtures,
    }
}

fn static_stylesheet_oracle_corpus_fixture_report(
    fixture: &StaticStylesheetOracleCorpusFixtureV0,
) -> OmenaScssEvalStaticStylesheetOracleCorpusFixtureReportV0 {
    let evaluation = derive_static_stylesheet_module_evaluation(fixture.source, fixture.dialect);
    let Some(evaluation) = evaluation else {
        return OmenaScssEvalStaticStylesheetOracleCorpusFixtureReportV0 {
            id: fixture.id,
            dialect: dialect_label(fixture.dialect),
            evaluator: "none",
            product_output_source: "none",
            legacy_output_retained_as_oracle: false,
            legacy_output_consumed_until_cutover: false,
            evaluation_available: false,
            native_edit_output: None,
            divergence_count: 0,
            all_legacy_declaration_values_preserved: false,
            native_replacement_count: 0,
            native_replacement_legacy_reflection_count: 0,
            native_replacement_legacy_unreflected_count: 0,
            native_edit_count: 0,
            native_value_edit_count: 0,
            native_structural_edit_count: 0,
            native_edit_output_matches_evaluated_css: false,
            native_value_reference_count: 0,
            native_resolved_value_count: 0,
            native_raw_value_count: 0,
            native_top_value_count: 0,
            native_cycle_value_count: 0,
            native_fuel_exhausted_value_count: 0,
            native_unresolved_reference_value_count: 0,
            native_unsupported_dynamic_value_count: 0,
        };
    };

    OmenaScssEvalStaticStylesheetOracleCorpusFixtureReportV0 {
        id: fixture.id,
        dialect: evaluation.dialect,
        evaluator: evaluation.evaluator,
        product_output_source: evaluation.product_output_source,
        legacy_output_retained_as_oracle: evaluation.legacy_output_retained_as_oracle,
        legacy_output_consumed_until_cutover: evaluation.legacy_output_consumed_until_cutover,
        evaluation_available: true,
        native_edit_output: Some(evaluation.native_edit_output.clone()),
        divergence_count: evaluation.oracle.divergence_count,
        all_legacy_declaration_values_preserved: evaluation
            .oracle
            .all_legacy_declaration_values_preserved,
        native_replacement_count: evaluation.replacement_count,
        native_replacement_legacy_reflection_count: evaluation
            .native_replacement_legacy_reflection_count,
        native_replacement_legacy_unreflected_count: evaluation
            .native_replacement_legacy_unreflected_count,
        native_edit_count: evaluation.native_edit_count,
        native_value_edit_count: evaluation.native_value_edit_count,
        native_structural_edit_count: evaluation.native_structural_edit_count,
        native_edit_output_matches_evaluated_css: evaluation
            .native_edit_output_matches_evaluated_css,
        native_value_reference_count: evaluation.value_resolution.reference_count,
        native_resolved_value_count: evaluation.value_resolution.resolved_count,
        native_raw_value_count: evaluation.value_resolution.raw_count,
        native_top_value_count: evaluation.value_resolution.top_count,
        native_cycle_value_count: evaluation.value_resolution.cycle_count,
        native_fuel_exhausted_value_count: evaluation.value_resolution.fuel_exhausted_count,
        native_unresolved_reference_value_count: evaluation
            .value_resolution
            .unresolved_reference_count,
        native_unsupported_dynamic_value_count: evaluation
            .value_resolution
            .unsupported_dynamic_count,
    }
}

fn static_stylesheet_oracle_corpus_fixtures() -> &'static [StaticStylesheetOracleCorpusFixtureV0] {
    &[
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.variable-basic",
            dialect: StyleDialect::Scss,
            source: "$gap: 1px; .card { margin: $gap; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.variable-basic",
            dialect: StyleDialect::Sass,
            source: "$gap: 1px\n.card\n  margin: $gap",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.static-function-return",
            dialect: StyleDialect::Sass,
            source: "@function gap($value)\n  @return $value\n.card\n  margin: gap(1px)",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.static-mixin-include",
            dialect: StyleDialect::Sass,
            source: "@mixin card($gap)\n  margin: $gap\n.card\n  @include card(1px)",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.static-mixin-if",
            dialect: StyleDialect::Sass,
            source: "@mixin tone($enabled)\n  @if $enabled\n    color: red\n  @else\n    color: blue\n.button\n  @include tone(false)",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.static-mixin-for",
            dialect: StyleDialect::Sass,
            source: "@mixin items($count)\n  @for $i from 1 through $count\n    order: $i\n.button\n  @include items(3)",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.static-mixin-each",
            dialect: StyleDialect::Sass,
            source: "@mixin tones()\n  @each $tone in red, blue\n    color: $tone\n.button\n  @include tones()",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.static-mixin-while",
            dialect: StyleDialect::Sass,
            source: "@mixin items()\n  $i: 0\n  @while $i < 3\n    $i: $i + 1\n    order: $i\n.button\n  @include items()",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.static-if-return",
            dialect: StyleDialect::Sass,
            source: "@function pick($enabled)\n  @if $enabled\n    @return 1px\n  @else\n    @return 2px\n.card\n  margin: pick(true)",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.static-for-return",
            dialect: StyleDialect::Sass,
            source: "@function pick($target)\n  @for $i from 1 through 3\n    @if $i == $target\n      @return $i\n  @return 0\n.card\n  z-index: pick(2)",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.descending-static-for-return",
            dialect: StyleDialect::Sass,
            source: "@function pick($target)\n  @for $i from 3 through 1\n    @if $i == $target\n      @return $i\n  @return 0\n.card\n  z-index: pick(2)",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.static-for-expression-bounds",
            dialect: StyleDialect::Sass,
            source: "@function pick($target)\n  @for $i from 1 + 1 through 1 + 2\n    @if $i == $target\n      @return $i\n  @return 0\n.card\n  z-index: pick(1)",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.static-while-return",
            dialect: StyleDialect::Sass,
            source: "@function pick($target)\n  $i: 0\n  @while $i < 3\n    @if $i == $target\n      @return $i + 1\n    $i: $i + 1\n  @return 0\n.card\n  z-index: pick(2)",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.static-while-expression-step",
            dialect: StyleDialect::Sass,
            source: "@function pick($target)\n  $step: 1 + 1\n  $i: 0\n  @while $i < 6\n    @if $i == $target\n      @return $i\n    $i: $i + $step\n  @return 0\n.card\n  z-index: pick(4)",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.static-each-return",
            dialect: StyleDialect::Sass,
            source: "@function tone($target)\n  @each $name, $tone in (primary: red, secondary: blue)\n    @if $name == $target\n      @return $tone\n  @return black\n.card\n  color: tone(secondary)",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.static-each-function-source-return",
            dialect: StyleDialect::Sass,
            source: "@function pick($target)\n  @each $item in append(1px 2px, 3px)\n    @if $item == $target\n      @return $item\n  @return 0px\n.card\n  margin: pick(3px)",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.static-each-tuple-function-source-return",
            dialect: StyleDialect::Sass,
            source: "@function width-for($target)\n  @each $width, $style in zip(1px 2px, solid dashed)\n    @if $style == $target\n      @return $width\n  @return 0px\n.card\n  margin: width-for(dashed)",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.static-map-list-builtins",
            dialect: StyleDialect::Sass,
            source: "$tokens: (primary: red, secondary: blue)\n$count: length(map-keys($tokens))\n$tone: nth(map-values($tokens), 2)\n.card\n  color: $tone\n  z-index: $count",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.static-default-function-arguments",
            dialect: StyleDialect::Sass,
            source: "@function offset($value: 1px, $extra: 2px)\n  @return $value + $extra\n.card\n  margin: offset()",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.static-default-argument-prior-parameter",
            dialect: StyleDialect::Sass,
            source: "@function offset($value, $extra: $value + 1px)\n  @return $extra\n.card\n  margin: offset(2px)",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.static-named-function-arguments",
            dialect: StyleDialect::Sass,
            source: "@function pair($left, $right)\n  @return $left + $right\n.card\n  margin: pair($right: 2px, $left: 1px)",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.static-named-argument-default-tail",
            dialect: StyleDialect::Sass,
            source: "@function pair($left, $right: 2px)\n  @return $left + $right\n.card\n  margin: pair($left: 1px)",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.static-hyphen-underscore-function-reference",
            dialect: StyleDialect::Sass,
            source: "@function gap($base_value)\n  @return $base-value + 1px\n.card\n  margin: gap(2px)",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.static-hyphen-underscore-named-argument",
            dialect: StyleDialect::Sass,
            source: "@function gap($base_value)\n  @return $base-value + 1px\n.card\n  margin: gap($base-value: 2px)",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.static-top-level-for",
            dialect: StyleDialect::Sass,
            source: "@for $i from 1 through 3\n  .n\n    order: $i",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.static-top-level-each",
            dialect: StyleDialect::Sass,
            source: "@each $tone in red, blue\n  .n\n    color: $tone",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.static-top-level-while",
            dialect: StyleDialect::Sass,
            source: "$i: 0\n@while $i < 3\n  $i: $i + 1\n  .n\n    order: $i",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.static-top-level-if",
            dialect: StyleDialect::Sass,
            source: "$enabled: true\n@if $enabled\n  .on\n    color: green\n@else\n  .off\n    color: gray",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.static-top-level-if-variable",
            dialect: StyleDialect::Sass,
            source: "$enabled: true\n$brand: green\n@if $enabled\n  .on\n    color: $brand\n@else\n  .off\n    color: gray",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "sass.static-top-level-if-function",
            dialect: StyleDialect::Sass,
            source: "@function tone($color)\n  @return $color\n$enabled: true\n@if $enabled\n  .on\n    color: tone(green)\n@else\n  .off\n    color: gray",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.color-helpers",
            dialect: StyleDialect::Scss,
            source: "$tone: list.nth(list.append(1px, transparentize(red, .25)), 2); .card { color: $tone; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.static-map-list-builtins",
            dialect: StyleDialect::Scss,
            source: "$merged: map.deep-merge((theme: (spacing: (sm: 4px), tone: blue)), (theme: (spacing: (md: 8px), tone: red))); $gap: map.get($merged, theme, spacing, md); $tone: map.get($merged, theme, tone); $count: list.length(map.keys(map.get($merged, theme, spacing))); .card { margin: $gap; color: $tone; z-index: $count; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.static-for-return",
            dialect: StyleDialect::Scss,
            source: "@function pick($target) { @for $i from 1 through 3 { @if $i == $target { @return $i; } } @return 0; } .button { z-index: pick(2); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.descending-static-for-return",
            dialect: StyleDialect::Scss,
            source: "@function pick($target) { @for $i from 3 through 1 { @if $i == $target { @return $i; } } @return 0; } .button { z-index: pick(2); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.static-for-expression-bounds",
            dialect: StyleDialect::Scss,
            source: "@function pick($target) { @for $i from 1 + 1 through 1 + 2 { @if $i == $target { @return $i; } } @return 0; } .button { z-index: pick(1); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.static-each-return",
            dialect: StyleDialect::Scss,
            source: "@function tone($target) { @each $name, $tone in (primary: red, secondary: blue) { @if $name == $target { @return $tone; } } @return black; } .button { color: tone(secondary); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.static-each-function-source-return",
            dialect: StyleDialect::Scss,
            source: "@function pick($target) { @each $item in list.append(1px 2px, 3px) { @if $item == $target { @return $item; } } @return 0px; } .button { margin: pick(3px); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.static-each-tuple-function-source-return",
            dialect: StyleDialect::Scss,
            source: "@function width-for($target) { @each $width, $style in list.zip(1px 2px, solid dashed) { @if $style == $target { @return $width; } } @return 0px; } .button { margin: width-for(dashed); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.static-while-return",
            dialect: StyleDialect::Scss,
            source: "@function pick($target) { $i: 0; @while $i < 3 { @if $i == $target { @return $i + 1; } $i: $i + 1; } @return 0; } .button { z-index: pick(2); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.static-while-expression-step",
            dialect: StyleDialect::Scss,
            source: "@function pick($target) { $step: 1 + 1; $i: 0; @while $i < 6 { @if $i == $target { @return $i; } $i: $i + $step; } @return 0; } .button { z-index: pick(4); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.static-default-function-arguments",
            dialect: StyleDialect::Scss,
            source: "@function offset($value: 1px, $extra: 2px) { @return $value + $extra; } .card { margin: offset(); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.static-default-argument-prior-parameter",
            dialect: StyleDialect::Scss,
            source: "@function offset($value, $extra: $value + 1px) { @return $extra; } .card { margin: offset(2px); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.static-named-function-arguments",
            dialect: StyleDialect::Scss,
            source: "@function pair($left, $right) { @return $left + $right; } .card { margin: pair($right: 2px, $left: 1px); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.static-named-argument-default-tail",
            dialect: StyleDialect::Scss,
            source: "@function pair($left, $right: 2px) { @return $left + $right; } .card { margin: pair($left: 1px); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.static-hyphen-underscore-function-reference",
            dialect: StyleDialect::Scss,
            source: "@function gap($base_value) { @return $base-value + 1px; } .card { margin: gap(2px); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.static-hyphen-underscore-named-argument",
            dialect: StyleDialect::Scss,
            source: "@function gap($base_value) { @return $base-value + 1px; } .card { margin: gap($base-value: 2px); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.dynamic-function-return",
            dialect: StyleDialect::Scss,
            source: "@function tone($enabled) { @if $enabled { @return red; } @else { @return blue; } } .button { color: tone(var(--enabled)); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.unresolved-forward-composite",
            dialect: StyleDialect::Scss,
            source: "$border: 1px solid $brand; $brand: red; .button { border: $border; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.recursive-function-return",
            dialect: StyleDialect::Scss,
            source: "@function loop($value) { @return loop($value); } .button { color: loop(red); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.indirect-recursive-function-return",
            dialect: StyleDialect::Scss,
            source: "@function a($value) { @return b($value); } @function b($value) { @return a($value); } .button { color: a(red); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.static-mixin-include",
            dialect: StyleDialect::Scss,
            source: "@mixin tone($color, $gap: 1px) { color: $color; margin: $gap; } .button { @include tone(red, 2px); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.static-mixin-if",
            dialect: StyleDialect::Scss,
            source: "@mixin tone($enabled) { @if $enabled { color: red; } @else { color: blue; } } .button { @include tone(false); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.static-mixin-for",
            dialect: StyleDialect::Scss,
            source: "@mixin items($count) { @for $i from 1 through $count { order: $i; } } .button { @include items(3); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.static-mixin-each",
            dialect: StyleDialect::Scss,
            source: "@mixin tones() { @each $tone in red, blue { color: $tone; } } .button { @include tones(); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.static-mixin-while",
            dialect: StyleDialect::Scss,
            source: "@mixin items() { $i: 0; @while $i < 3 { $i: $i + 1; order: $i; } } .button { @include items(); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.static-top-level-if",
            dialect: StyleDialect::Scss,
            source: "$enabled: false; @if $enabled { .on { color: green; } } @else { .off { color: red; } }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.dynamic-top-level-if",
            dialect: StyleDialect::Scss,
            source: "$enabled: var(--enabled); @if $enabled { .on { color: green; } } @else { .off { color: red; } }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.static-top-level-if-variable",
            dialect: StyleDialect::Scss,
            source: "$enabled: true; $brand: red; @if $enabled { .on { color: $brand; } } @else { .off { color: blue; } }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.static-top-level-if-function",
            dialect: StyleDialect::Scss,
            source: "@function tone($color) { @return $color; } $enabled: true; @if $enabled { .on { color: tone(red); } } @else { .off { color: blue; } }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.static-top-level-for",
            dialect: StyleDialect::Scss,
            source: "@for $i from 1 through 3 { .n { order: $i; } }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.static-top-level-each",
            dialect: StyleDialect::Scss,
            source: "@each $tone in red, blue { .n { color: $tone; } }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.static-top-level-while",
            dialect: StyleDialect::Scss,
            source: "$i: 0; @while $i < 3 { $i: $i + 1; .n { order: $i; } }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.dynamic-mixin-local",
            dialect: StyleDialect::Scss,
            source: "@mixin tone { $space: meta.inspect((a: b)); margin: $space; } .button { @include tone; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "scss.recursive-nested-mixin-include",
            dialect: StyleDialect::Scss,
            source: "@mixin a { @include b; } @mixin b { @include a; } .button { @include a; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.variable-basic",
            dialect: StyleDialect::Less,
            source: "@gap: 2px; .card { margin: @gap; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.dynamic-escaped-string",
            dialect: StyleDialect::Less,
            source: "@filter: ~\"@{name}\"; .card { filter: @filter; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.fuel-exhausted-variable-chain",
            dialect: StyleDialect::Less,
            source: concat!(
                "@v0: @v1; @v1: @v2; @v2: @v3; @v3: @v4; @v4: @v5; @v5: @v6; ",
                "@v6: @v7; @v7: @v8; @v8: @v9; @v9: @v10; @v10: @v11; @v11: @v12; ",
                "@v12: @v13; @v13: @v14; @v14: @v15; @v15: @v16; @v16: @v17; @v17: @v18; ",
                "@v18: @v19; @v19: @v20; @v20: @v21; @v21: @v22; @v22: @v23; @v23: @v24; ",
                "@v24: @v25; @v25: @v26; @v26: @v27; @v27: @v28; @v28: @v29; @v29: @v30; ",
                "@v30: @v31; @v31: @v32; @v32: @v33; @v33: @v34; @v34: @v35; @v35: @v36; ",
                "@v36: @v37; @v37: @v38; @v38: @v39; @v39: @v40; @v40: @v41; @v41: @v42; ",
                "@v42: @v43; @v43: @v44; @v44: @v45; @v45: @v46; @v46: @v47; @v47: @v48; ",
                "@v48: @v49; @v49: @v50; @v50: @v51; @v51: @v52; @v52: @v53; @v53: @v54; ",
                "@v54: @v55; @v55: @v56; @v56: @v57; @v57: @v58; @v58: @v59; @v59: @v60; ",
                "@v60: @v61; @v61: @v62; @v62: @v63; @v63: @v64; @v64: @v65; @v65: @v66; ",
                "@v66: @v67; @v67: @v68; @v68: @v69; @v69: @v70; @v70: @v71; @v71: @v72; ",
                "@v72: @v73; @v73: @v74; @v74: @v75; @v75: @v76; @v76: @v77; @v77: @v78; ",
                "@v78: @v79; @v79: @v80; @v80: @v81; @v81: @v82; @v82: @v83; @v83: @v84; ",
                "@v84: @v85; @v85: @v86; @v86: @v87; @v87: @v88; @v88: @v89; @v89: @v90; ",
                "@v90: @v91; @v91: @v92; @v92: @v93; @v93: @v94; @v94: @v95; @v95: @v96; ",
                "@v96: @v97; @v97: @v98; @v98: @v99; @v99: @v100; @v100: @v101; @v101: @v102; ",
                "@v102: @v103; @v103: @v104; @v104: @v105; @v105: @v106; @v106: @v107; @v107: @v108; ",
                "@v108: @v109; @v109: @v110; @v110: @v111; @v111: @v112; @v112: @v113; @v113: @v114; ",
                "@v114: @v115; @v115: @v116; @v116: @v117; @v117: @v118; @v118: @v119; @v119: @v120; ",
                "@v120: @v121; @v121: @v122; @v122: @v123; @v123: @v124; @v124: @v125; @v125: @v126; ",
                "@v126: @v127; @v127: @v128; @v128: @v129; @v129: @v130; @v130: 1px; ",
                ".button { width: @v0; }",
            ),
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.static-mixin",
            dialect: StyleDialect::Less,
            source: "@brand: red; .tone(@color, @gap: 1px) { color: @color; margin: @gap; padding: @brand; } .button { .tone(blue, 2px); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.recursive-nested-mixin-call",
            dialect: StyleDialect::Less,
            source: ".again() { .again(); } .button { .again(); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.named-mixin-arguments",
            dialect: StyleDialect::Less,
            source: ".tone(@color, @gap: 1px) { color: @color; margin: @gap; } .button { .tone(@gap: 2px, @color: blue); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.semicolon-named-mixin-arguments",
            dialect: StyleDialect::Less,
            source: ".tone(@color; @gap: 1px) { color: @color; margin: @gap; } .button { .tone(@gap: 2px; @color: blue); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.variadic-mixin-arguments",
            dialect: StyleDialect::Less,
            source: ".shadow(@color; @rest...) { color: @color; box-shadow: @rest; trace: @arguments; } .button { .shadow(red; 1px, 2px, 3px); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.mixin-accessor",
            dialect: StyleDialect::Less,
            source: ".tokens(@color, @gap: 1px) { @result: @color; width: @gap; } .button { color: .tokens(red)[@result]; margin: .tokens(red, 2px)[width]; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.unknown-mixin-accessor-member",
            dialect: StyleDialect::Less,
            source: ".tokens(@color) { @result: @color; } .button { color: .tokens(red)[@missing]; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.unknown-mixin-accessor-property-member",
            dialect: StyleDialect::Less,
            source: ".tokens(@color) { result: @color; } .button { color: .tokens(red)[missing]; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.namespace-mixin",
            dialect: StyleDialect::Less,
            source: "#bundle() { .rounded(@radius) { border-radius: @radius; } } .button { #bundle > .rounded(2px); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.parameterized-namespace-mixin",
            dialect: StyleDialect::Less,
            source: "#bundle(@color) { .tone() { color: @color; } } .button { #bundle(red) > .tone(); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.unbound-parameterized-namespace-mixin",
            dialect: StyleDialect::Less,
            source: "#bundle(@color) { .tone() { color: @color; } } .button { #bundle > .tone(); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.literal-pattern-mixin",
            dialect: StyleDialect::Less,
            source: ".tone(dark, @color) { color: @color; background: black; } .tone(light, @color) { color: @color; background: white; } .button { .tone(dark, red); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.unmatched-literal-pattern-mixin",
            dialect: StyleDialect::Less,
            source: ".tone(dark, @color) { color: @color; background: black; } .button { .tone(light, red); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.important-mixin",
            dialect: StyleDialect::Less,
            source: ".tone(@color, @gap: 1px) { color: @color; margin: @gap; } .button { .tone(red, 2px) !important; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.unknown-mixin-call-suffix",
            dialect: StyleDialect::Less,
            source: ".tone(@color) { color: @color; } .button { .tone(red) !default; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.default-guarded-mixin",
            dialect: StyleDialect::Less,
            source: ".tone(@color) when (@color = red) { color: @color; } .tone(@color) when (default()) and (iscolor(@color)) { color: gray; } .button { .tone(blue); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.false-guarded-mixin",
            dialect: StyleDialect::Less,
            source: ".tone() when (iscolor(1px)) { color: red; } .button { .tone(); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.false-guarded-namespace-mixin",
            dialect: StyleDialect::Less,
            source: "#bundle() when (iscolor(1px)) { .tone() { color: red; } } .button { #bundle > .tone(); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.guarded-namespace-mixin",
            dialect: StyleDialect::Less,
            source: "#bundle() when (iscolor(red)) { .tone() { color: red; } } .button { #bundle > .tone(); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.detached-ruleset",
            dialect: StyleDialect::Less,
            source: "@brand: red; @rules: { color: @brand; margin: 1px; }; .button { @rules(); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.unknown-detached-ruleset-mixin-call",
            dialect: StyleDialect::Less,
            source: "@rules: { .unknown(); }; .button { @rules(); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.ruleset-guarded-mixin",
            dialect: StyleDialect::Less,
            source: ".apply(@block) when (isruleset(@block)) { @block(); } @rules: { color: red; margin: 1px; }; .button { .apply(@rules); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.detached-ruleset-accessor",
            dialect: StyleDialect::Less,
            source: "@brand: red; @tokens: { primary: @brand; @gap: 2px; }; .button { color: @tokens[primary]; margin: @tokens[@gap]; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.unknown-detached-ruleset-accessor-member",
            dialect: StyleDialect::Less,
            source: "@tokens: { primary: red; }; .button { color: @tokens[missing]; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.hsl-color-transforms",
            dialect: StyleDialect::Less,
            source: "@tone: lighten(#123456, 10%); @shifted: spin(#123456, 10); .button { color: @tone; border-color: @shifted; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.relative-color-transforms",
            dialect: StyleDialect::Less,
            source: "@tone: lighten(#123456, 10%, relative); @sat: saturate(#123456, 10%, relative); @alpha: fadein(rgba(18, 52, 86, .5), 10%, relative); .button { color: @tone; border-color: @sat; background: @alpha; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.convert-units",
            dialect: StyleDialect::Less,
            source: "@cm: convert(1in, cm); @ms: convert(1s, ms); @deg: convert(.5turn, deg); .button { width: @cm; transition-duration: @ms; rotate: @deg; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.trig-functions",
            dialect: StyleDialect::Less,
            source: "@pi: pi(); @sin: sin(30deg); @asin: asin(.5); .button { opacity: @sin; rotate: @asin; --pi: @pi; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.extended-numeric-builtins",
            dialect: StyleDialect::Less,
            source: "@sqrt: sqrt(4); @pow: pow(2, 3); @mod: mod(11px, 4px); @min: min(1px, 2px, 3px); @max: max(1px, 2px, 3px); @round: round(1.234px, 2); .button { sqrt: @sqrt; pow: @pow; mod: @mod; min: @min; max: @max; round: @round; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.percentage-rounding-builtins",
            dialect: StyleDialect::Less,
            source: "@ratio: percentage(.5); @ceil: ceil(1.2px); @floor: floor(1.8px); .button { width: @ratio; top: @ceil; bottom: @floor; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.range-list",
            dialect: StyleDialect::Less,
            source: "@items: range(4); @gaps: range(1px, 5px, 2); .button { z-index: length(@items); margin: extract(@gaps, 2); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.replace-string",
            dialect: StyleDialect::Less,
            source: "@name: replace(\"hello world\", \"world\", \"less\"); @all: replace(\"hello\", \"l\", \"L\", \"g\"); .button { content: @name; alt: @all; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.format-string",
            dialect: StyleDialect::Less,
            source: "@name: %(\"hello %s\", \"less\"); @encoded: %(\"%S\", \"x y\"); @literal: %(\"%% done\"); .button { name: @name; encoded: @encoded; literal: @literal; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.isruleset-predicate",
            dialect: StyleDialect::Less,
            source: "@rules: { color: red; }; @ok: isruleset(@rules); @bad: isruleset(red); .button { ok: @ok; bad: @bad; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.isdefined-predicate",
            dialect: StyleDialect::Less,
            source: "@brand: red; @defined: isdefined(@brand); @missing: isdefined(@absent); @literal: isdefined(red); @future-defined: isdefined(@future); @future: blue; .button { defined: @defined; missing: @missing; literal: @literal; future-defined: @future-defined; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.property-isdefined-predicate",
            dialect: StyleDialect::Less,
            source: ".button { color: red; @has-color: isdefined($color); @missing-prop: isdefined($missing); has: @has-color; missing: @missing-prop; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.property-variable-alias",
            dialect: StyleDialect::Less,
            source: ".button { margin: (1px + 2px); color: red; @gap: $margin; @outline: 1px solid $color; padding: @gap; border: @outline; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.isdefined-guarded-mixin",
            dialect: StyleDialect::Less,
            source: "@brand: red; .present() when (isdefined(@brand)) { color: @brand; } .with-param(@tone) when (isdefined(@tone)) { border-color: @tone; } .button { .present(); .with-param(green); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.property-guarded-mixin",
            dialect: StyleDialect::Less,
            source: ".space() when (isnumber($margin)) { padding: $margin; } .tone() when (iscolor($color)) { border-color: $color; } .button { margin: (1px + 2px); color: red; .space(); .tone(); }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.future-property-guarded-mixin",
            dialect: StyleDialect::Less,
            source: ".space() when (isnumber($margin)) { padding: $margin; } .button { .space(); margin: 2px; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.rgb-color-constructors",
            dialect: StyleDialect::Less,
            source: "@rgb: rgb(18, 52, 86); @rgba: rgba(18, 52, 86, .5); .button { color: @rgb; background: @rgba; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.color-mix",
            dialect: StyleDialect::Less,
            source: "@tone: mix(red, blue, 25%); @surface: tint(#123456, 10%); .button { color: @tone; background: @surface; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.color-blend",
            dialect: StyleDialect::Less,
            source: "@tone: overlay(#123456, #abcdef); @surface: screen(red, blue); @alpha: screen(rgba(255, 102, 0, .5), rgba(0, 0, 255, .25)); .button { color: @tone; background: @surface; border-color: @alpha; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.hsv-color-metadata",
            dialect: StyleDialect::Less,
            source: "@tone: hsv(210, 60%, 40%); @alpha: hsva(210, 60%, 40%, 50%); @v: hsvvalue(#123456); .button { color: @tone; border-color: @alpha; opacity: @v; }",
        },
        StaticStylesheetOracleCorpusFixtureV0 {
            id: "less.contrast-color",
            dialect: StyleDialect::Less,
            source: "@contrast: contrast(#123456); @tone: color(\"#12345680\"); @keyword: color(red); .button { color: @contrast; border-color: @tone; background: @keyword; }",
        },
    ]
}
