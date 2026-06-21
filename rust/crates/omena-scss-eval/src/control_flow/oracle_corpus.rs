use omena_parser::StyleDialect;
use serde::Serialize;

use super::{
    OmenaScssEvalControlFlowWideningWitnessV0, OmenaScssEvalTypedValueLatticeWitnessV0,
    SCSS_CALL_RETURN_RECURSION_LIMIT, analyze_scss_control_flow_values, dialect_label,
    summarize_scss_call_return_ir, summarize_scss_control_flow_ir,
    summarize_scss_control_flow_prune_reachability, summarize_scss_control_flow_widening_witness,
    summarize_typed_value_lattice_witness,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalControlFlowOracleCorpusReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub mode: &'static str,
    pub value_type: &'static str,
    pub node_key_type: &'static str,
    pub recursion_cap: usize,
    pub fixture_count: usize,
    pub scss_fixture_count: usize,
    pub sass_fixture_count: usize,
    pub supported_fixture_count: usize,
    pub rejected_flat_css_fixture_count: usize,
    pub control_flow_fixture_count: usize,
    pub branch_fixture_count: usize,
    pub loop_fixture_count: usize,
    pub back_edge_fixture_count: usize,
    pub call_return_fixture_count: usize,
    pub resolved_call_return_fixture_count: usize,
    pub top_call_return_fixture_count: usize,
    pub recursive_call_fixture_count: usize,
    pub converged_value_analysis_fixture_count: usize,
    pub widened_to_top_fixture_count: usize,
    pub widening_witness_widened_to_top_count: usize,
    pub widening_witness_converged: bool,
    pub prune_reachability_fixture_count: usize,
    pub prune_reachability_changed_fixture_count: usize,
    pub prune_reachability_flat_css_cfg_built_count: usize,
    pub flat_css_cfg_built_count: usize,
    pub merged_cross_file_graph_count: usize,
    pub all_supported_fixtures_converged: bool,
    pub no_flat_css_cfg_built: bool,
    pub no_merged_cross_file_graph: bool,
    pub widening_witness: OmenaScssEvalControlFlowWideningWitnessV0,
    pub typed_value_lattice_witness: OmenaScssEvalTypedValueLatticeWitnessV0,
    pub fixtures: Vec<OmenaScssEvalControlFlowOracleCorpusFixtureReportV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalControlFlowOracleCorpusFixtureReportV0 {
    pub id: &'static str,
    pub dialect: &'static str,
    pub supported_dialect: bool,
    pub control_flow_available: bool,
    pub value_analysis_available: bool,
    pub call_return_available: bool,
    pub control_flow_block_count: usize,
    pub branch_block_count: usize,
    pub loop_block_count: usize,
    pub back_edge_count: usize,
    pub call_return_node_count: usize,
    pub call_return_edge_count: usize,
    pub call_resolved_return_value_count: usize,
    pub exact_call_resolved_return_value_count: usize,
    pub top_call_resolved_return_value_count: usize,
    pub recursive_edge_count: usize,
    pub capped_recursive_call_count: usize,
    pub value_analysis_converged: bool,
    pub value_analysis_iteration_count: usize,
    pub widened_to_top_count: usize,
    pub prune_reachability_available: bool,
    pub prune_reachability_converged: bool,
    pub prune_reachability_flat_css_cfg_built: bool,
    pub prune_reachability_have_terminals_changed: bool,
    pub flat_css_cfg_built: bool,
    pub merged_cross_file_graph: bool,
}

struct ScssControlFlowOracleCorpusFixtureV0 {
    id: &'static str,
    dialect: StyleDialect,
    source: &'static str,
}

pub fn summarize_scss_control_flow_oracle_corpus() -> OmenaScssEvalControlFlowOracleCorpusReportV0 {
    let fixtures = scss_control_flow_oracle_corpus_fixtures()
        .iter()
        .map(scss_control_flow_oracle_corpus_fixture_report)
        .collect::<Vec<_>>();
    let widening_witness = summarize_scss_control_flow_widening_witness();
    let fixture_count = fixtures.len();
    let scss_fixture_count = fixtures
        .iter()
        .filter(|fixture| fixture.dialect == "scss")
        .count();
    let sass_fixture_count = fixtures
        .iter()
        .filter(|fixture| fixture.dialect == "sass")
        .count();
    let supported_fixture_count = fixtures
        .iter()
        .filter(|fixture| fixture.supported_dialect)
        .count();
    let rejected_flat_css_fixture_count = fixtures
        .iter()
        .filter(|fixture| {
            !fixture.supported_dialect
                && !fixture.control_flow_available
                && !fixture.value_analysis_available
                && !fixture.call_return_available
        })
        .count();
    let control_flow_fixture_count = fixtures
        .iter()
        .filter(|fixture| fixture.control_flow_available)
        .count();
    let branch_fixture_count = fixtures
        .iter()
        .filter(|fixture| fixture.branch_block_count > 0)
        .count();
    let loop_fixture_count = fixtures
        .iter()
        .filter(|fixture| fixture.loop_block_count > 0)
        .count();
    let back_edge_fixture_count = fixtures
        .iter()
        .filter(|fixture| fixture.back_edge_count > 0)
        .count();
    let call_return_fixture_count = fixtures
        .iter()
        .filter(|fixture| fixture.call_return_node_count > 0)
        .count();
    let resolved_call_return_fixture_count = fixtures
        .iter()
        .filter(|fixture| fixture.call_resolved_return_value_count > 0)
        .count();
    let top_call_return_fixture_count = fixtures
        .iter()
        .filter(|fixture| fixture.top_call_resolved_return_value_count > 0)
        .count();
    let recursive_call_fixture_count = fixtures
        .iter()
        .filter(|fixture| fixture.recursive_edge_count > 0)
        .count();
    let converged_value_analysis_fixture_count = fixtures
        .iter()
        .filter(|fixture| fixture.value_analysis_converged)
        .count();
    let widened_to_top_fixture_count = fixtures
        .iter()
        .filter(|fixture| fixture.widened_to_top_count > 0)
        .count();
    let prune_reachability_fixture_count = fixtures
        .iter()
        .filter(|fixture| fixture.prune_reachability_available)
        .count();
    let prune_reachability_changed_fixture_count = fixtures
        .iter()
        .filter(|fixture| fixture.prune_reachability_have_terminals_changed)
        .count();
    let prune_reachability_flat_css_cfg_built_count = fixtures
        .iter()
        .filter(|fixture| fixture.prune_reachability_flat_css_cfg_built)
        .count();
    let flat_css_cfg_built_count = fixtures
        .iter()
        .filter(|fixture| fixture.flat_css_cfg_built)
        .count();
    let merged_cross_file_graph_count = fixtures
        .iter()
        .filter(|fixture| fixture.merged_cross_file_graph)
        .count();
    let all_supported_fixtures_converged = fixtures
        .iter()
        .filter(|fixture| fixture.supported_dialect)
        .all(|fixture| fixture.value_analysis_available && fixture.value_analysis_converged);
    let typed_value_lattice_witness = summarize_typed_value_lattice_witness();

    OmenaScssEvalControlFlowOracleCorpusReportV0 {
        schema_version: "0",
        product: "omena-scss-eval.control-flow-oracle-corpus",
        mode: "oracleOnly",
        value_type: "AbstractCssValueV0",
        node_key_type: "StableNodeKeyV0",
        recursion_cap: SCSS_CALL_RETURN_RECURSION_LIMIT,
        fixture_count,
        scss_fixture_count,
        sass_fixture_count,
        supported_fixture_count,
        rejected_flat_css_fixture_count,
        control_flow_fixture_count,
        branch_fixture_count,
        loop_fixture_count,
        back_edge_fixture_count,
        call_return_fixture_count,
        resolved_call_return_fixture_count,
        top_call_return_fixture_count,
        recursive_call_fixture_count,
        converged_value_analysis_fixture_count,
        widened_to_top_fixture_count,
        widening_witness_widened_to_top_count: widening_witness.widened_to_top_count,
        widening_witness_converged: widening_witness.converged,
        prune_reachability_fixture_count,
        prune_reachability_changed_fixture_count,
        prune_reachability_flat_css_cfg_built_count,
        flat_css_cfg_built_count,
        merged_cross_file_graph_count,
        all_supported_fixtures_converged,
        no_flat_css_cfg_built: flat_css_cfg_built_count == 0,
        no_merged_cross_file_graph: merged_cross_file_graph_count == 0,
        widening_witness,
        typed_value_lattice_witness,
        fixtures,
    }
}

fn scss_control_flow_oracle_corpus_fixture_report(
    fixture: &ScssControlFlowOracleCorpusFixtureV0,
) -> OmenaScssEvalControlFlowOracleCorpusFixtureReportV0 {
    let supported_dialect = matches!(fixture.dialect, StyleDialect::Scss | StyleDialect::Sass);
    let control_flow_ir = summarize_scss_control_flow_ir(fixture.source, fixture.dialect);
    let value_analysis = analyze_scss_control_flow_values(fixture.source, fixture.dialect);
    let prune_reachability =
        summarize_scss_control_flow_prune_reachability(fixture.source, fixture.dialect);
    let call_return_ir = summarize_scss_call_return_ir(fixture.source, fixture.dialect);
    let flat_css_cfg_built = control_flow_ir
        .as_ref()
        .is_some_and(|summary| summary.flat_css_cfg_built)
        || value_analysis
            .as_ref()
            .is_some_and(|summary| summary.flat_css_cfg_built)
        || call_return_ir
            .as_ref()
            .is_some_and(|summary| summary.flat_css_cfg_built);
    let merged_cross_file_graph = control_flow_ir
        .as_ref()
        .is_some_and(|summary| summary.merged_cross_file_graph)
        || value_analysis
            .as_ref()
            .is_some_and(|summary| summary.merged_cross_file_graph)
        || call_return_ir
            .as_ref()
            .is_some_and(|summary| summary.merged_cross_file_graph);

    OmenaScssEvalControlFlowOracleCorpusFixtureReportV0 {
        id: fixture.id,
        dialect: dialect_label(fixture.dialect),
        supported_dialect,
        control_flow_available: control_flow_ir.is_some(),
        value_analysis_available: value_analysis.is_some(),
        call_return_available: call_return_ir.is_some(),
        control_flow_block_count: control_flow_ir
            .as_ref()
            .map_or(0, |summary| summary.block_count),
        branch_block_count: control_flow_ir
            .as_ref()
            .map_or(0, |summary| summary.branch_block_count),
        loop_block_count: control_flow_ir
            .as_ref()
            .map_or(0, |summary| summary.loop_block_count),
        back_edge_count: control_flow_ir
            .as_ref()
            .map_or(0, |summary| summary.back_edge_count),
        call_return_node_count: call_return_ir
            .as_ref()
            .map_or(0, |summary| summary.node_count),
        call_return_edge_count: call_return_ir
            .as_ref()
            .map_or(0, |summary| summary.edge_count),
        call_resolved_return_value_count: call_return_ir
            .as_ref()
            .map_or(0, |summary| summary.call_resolved_return_value_count),
        exact_call_resolved_return_value_count: call_return_ir
            .as_ref()
            .map_or(0, |summary| summary.exact_call_resolved_return_value_count),
        top_call_resolved_return_value_count: call_return_ir
            .as_ref()
            .map_or(0, |summary| summary.top_call_resolved_return_value_count),
        recursive_edge_count: call_return_ir
            .as_ref()
            .map_or(0, |summary| summary.recursive_edge_count),
        capped_recursive_call_count: call_return_ir
            .as_ref()
            .map_or(0, |summary| summary.capped_recursive_call_count),
        value_analysis_converged: value_analysis
            .as_ref()
            .is_some_and(|summary| summary.converged),
        value_analysis_iteration_count: value_analysis
            .as_ref()
            .map_or(0, |summary| summary.iteration_count),
        widened_to_top_count: value_analysis
            .as_ref()
            .map_or(0, |summary| summary.widened_to_top_count),
        prune_reachability_available: prune_reachability.is_some(),
        prune_reachability_converged: prune_reachability
            .as_ref()
            .is_some_and(|summary| summary.converged),
        prune_reachability_flat_css_cfg_built: prune_reachability
            .as_ref()
            .is_some_and(|summary| summary.flat_css_cfg_built),
        prune_reachability_have_terminals_changed: prune_reachability
            .as_ref()
            .is_some_and(|summary| summary.have_terminals_changed),
        flat_css_cfg_built,
        merged_cross_file_graph,
    }
}

fn scss_control_flow_oracle_corpus_fixtures() -> &'static [ScssControlFlowOracleCorpusFixtureV0] {
    &[
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.branch-if-else",
            dialect: StyleDialect::Scss,
            source: "$enabled: true; @if $enabled { .on { color: green; } } @else { .off { color: gray; } }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.static-while-loop",
            dialect: StyleDialect::Scss,
            source: "$i: 0; @while $i < 3 { $i: $i + 1; .n { order: $i; } }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.static-while-finite-set-bound",
            dialect: StyleDialect::Scss,
            source: "@each $limit in 2, 4 { $i: 0; @while $i < $limit { $i: $i + 1; .n { order: $i; } } }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.static-for-return",
            dialect: StyleDialect::Scss,
            source: "@function pick($target) { @for $i from 1 through 3 { @if $i == $target { @return $i; } } @return 0; } .button { z-index: pick(2); }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.descending-static-for-return",
            dialect: StyleDialect::Scss,
            source: "@function pick($target) { @for $i from 3 through 1 { @if $i == $target { @return $i; } } @return 0; } .button { z-index: pick(2); }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.static-for-expression-bounds",
            dialect: StyleDialect::Scss,
            source: "@function pick($target) { @for $i from 1 + 1 through 1 + 2 { @if $i == $target { @return $i; } } @return 0; } .button { z-index: pick(1); }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.static-for-exclusive-bound",
            dialect: StyleDialect::Scss,
            source: "@function pick($target) { @for $i from 1 to 3 { @if $i == $target { @return $i; } } @return 0; } .button { z-index: pick(3); }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.static-for-finite-set-bound",
            dialect: StyleDialect::Scss,
            source: "@each $end in 2, 4 { @for $i from 1 through $end { .n { order: $i; } } }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.static-while-expression-step",
            dialect: StyleDialect::Scss,
            source: "@function pick($target) { $step: 1 + 1; $i: 0; @while $i < 6 { @if $i == $target { @return $i; } $i: $i + $step; } @return 0; } .button { z-index: pick(4); }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.static-default-function-arguments",
            dialect: StyleDialect::Scss,
            source: "@function offset($value: 1px, $extra: 2px) { @return $value + $extra; } .button { margin: offset(); }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.static-default-argument-prior-parameter",
            dialect: StyleDialect::Scss,
            source: "@function offset($value, $extra: $value + 1px) { @return $extra; } .button { margin: offset(2px); }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.static-named-default-argument-prior-parameter",
            dialect: StyleDialect::Scss,
            source: "@function offset($value, $extra: $value + 1px) { @return $extra; } .button { margin: offset($value: 2px); }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.static-named-function-arguments",
            dialect: StyleDialect::Scss,
            source: "@function pair($left, $right) { @return $left + $right; } .button { margin: pair($right: 2px, $left: 1px); }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.static-named-argument-default-tail",
            dialect: StyleDialect::Scss,
            source: "@function pair($left, $right: 2px) { @return $left + $right; } .button { margin: pair($left: 1px); }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.static-hyphen-underscore-function-reference",
            dialect: StyleDialect::Scss,
            source: "@function gap($base_value) { @return $base-value + 1px; } .button { margin: gap(2px); }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.static-hyphen-underscore-named-argument",
            dialect: StyleDialect::Scss,
            source: "@function gap($base_value) { @return $base-value + 1px; } .button { margin: gap($base-value: 2px); }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.static-each-return",
            dialect: StyleDialect::Scss,
            source: "@function tone($target) { @each $name, $tone in (primary: red, secondary: blue) { @if $name == $target { @return $tone; } } @return black; } .button { color: tone(secondary); }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.static-each-function-source-return",
            dialect: StyleDialect::Scss,
            source: "@function pick($target) { @each $item in list.append(1px 2px, 3px) { @if $item == $target { @return $item; } } @return 0px; } .button { margin: pick(3px); }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.static-each-tuple-function-source-return",
            dialect: StyleDialect::Scss,
            source: "@function width-for($target) { @each $width, $style in list.zip(1px 2px, solid dashed) { @if $style == $target { @return $width; } } @return 0px; } .button { margin: width-for(dashed); }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.nested-static-loop-return",
            dialect: StyleDialect::Scss,
            source: "@function collect($target) { @for $i from 1 through 2 { @for $j from 1 through 2 { @if $i == $target { @return $i + $j; } } } @return 0; } .button { z-index: collect(2); }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.dynamic-loop-top",
            dialect: StyleDialect::Scss,
            source: "@function collect($count) { @for $i from 1 through $count { @return $i; } } .button { z-index: collect(var(--count)); }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.recursive-mixin-cap",
            dialect: StyleDialect::Scss,
            source: "@mixin again { @include again; } .button { @include again; }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.mixin-branch-control-flow",
            dialect: StyleDialect::Scss,
            source: "@mixin tone($enabled) { @if $enabled { color: red; } @else { color: blue; } } .button { @include tone(false); }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.mixin-loop-control-flow",
            dialect: StyleDialect::Scss,
            source: "@mixin items() { @for $i from 1 through 3 { order: $i; } } .button { @include items(); }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.static-named-mixin-arguments",
            dialect: StyleDialect::Scss,
            source: "@mixin tone($color, $gap: 1px) { color: $color; margin: $gap; } .button { @include tone($gap: 2px, $color: blue); }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.static-named-mixin-default-tail",
            dialect: StyleDialect::Scss,
            source: "@mixin tone($color, $gap: 2px) { color: $color; margin: $gap; } .button { @include tone($color: blue); }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.static-mixin-default-argument-prior-parameter",
            dialect: StyleDialect::Scss,
            source: "@mixin tone($color, $border: $color) { color: $color; border-color: $border; } .button { @include tone(blue); }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.static-named-mixin-default-argument-prior-parameter",
            dialect: StyleDialect::Scss,
            source: "@mixin tone($color, $border: $color) { color: $color; border-color: $border; } .button { @include tone($color: blue); }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.static-mixin-content-block",
            dialect: StyleDialect::Scss,
            source: "@mixin tone($color) { @content; color: $color; } .button { @include tone(red) { background: white; } }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.static-mixin-content-arguments",
            dialect: StyleDialect::Scss,
            source: "@mixin apply($color) { @content($color); } .button { @include apply(red) using ($tone) { color: $tone; } }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.static-mixin-content-expression-arguments",
            dialect: StyleDialect::Scss,
            source: "@mixin apply($color, $gap) { @content($color, $gap + 1px); } .button { @include apply(red, 1px) using ($tone, $space) { color: $tone; margin: $space; } }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.static-mixin-content-nested-include",
            dialect: StyleDialect::Scss,
            source: "@mixin spacing($gap) { margin: $gap; } @mixin apply($gap) { @content($gap); color: blue; } .button { @include apply(2px) using ($space) { @include spacing($space); background: white; } }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.static-nested-mixin-include",
            dialect: StyleDialect::Scss,
            source: "@mixin spacing($gap) { margin: $gap; } @mixin tone($gap, $color: red) { @include spacing($gap); color: $color; } .button { @include tone(2px, blue); }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.static-hyphen-underscore-mixin-include",
            dialect: StyleDialect::Scss,
            source: "@mixin tone_color($color) { color: $color; } .button { @include tone-color(green); }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.branch-if-else",
            dialect: StyleDialect::Sass,
            source: "$enabled: true\n@if $enabled\n  .on\n    color: green\n@else\n  .off\n    color: gray",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.static-while-loop",
            dialect: StyleDialect::Sass,
            source: "$i: 0\n@while $i < 3\n  $i: $i + 1\n  .n\n    order: $i",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.static-while-finite-set-bound",
            dialect: StyleDialect::Sass,
            source: "@each $limit in 2, 4\n  $i: 0\n  @while $i < $limit\n    $i: $i + 1\n    .n\n      order: $i",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.static-for-return",
            dialect: StyleDialect::Sass,
            source: "@function pick($target)\n  @for $i from 1 through 3\n    @if $i == $target\n      @return $i\n  @return 0\n.button\n  z-index: pick(2)",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.descending-static-for-return",
            dialect: StyleDialect::Sass,
            source: "@function pick($target)\n  @for $i from 3 through 1\n    @if $i == $target\n      @return $i\n  @return 0\n.button\n  z-index: pick(2)",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.static-for-expression-bounds",
            dialect: StyleDialect::Sass,
            source: "@function pick($target)\n  @for $i from 1 + 1 through 1 + 2\n    @if $i == $target\n      @return $i\n  @return 0\n.button\n  z-index: pick(1)",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.static-for-exclusive-bound",
            dialect: StyleDialect::Sass,
            source: "@function pick($target)\n  @for $i from 1 to 3\n    @if $i == $target\n      @return $i\n  @return 0\n.button\n  z-index: pick(3)",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.static-for-finite-set-bound",
            dialect: StyleDialect::Sass,
            source: "@each $end in 2, 4\n  @for $i from 1 through $end\n    .n\n      order: $i",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.static-while-expression-step",
            dialect: StyleDialect::Sass,
            source: "@function pick($target)\n  $step: 1 + 1\n  $i: 0\n  @while $i < 6\n    @if $i == $target\n      @return $i\n    $i: $i + $step\n  @return 0\n.button\n  z-index: pick(4)",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.static-default-function-arguments",
            dialect: StyleDialect::Sass,
            source: "@function offset($value: 1px, $extra: 2px)\n  @return $value + $extra\n.button\n  margin: offset()",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.static-default-argument-prior-parameter",
            dialect: StyleDialect::Sass,
            source: "@function offset($value, $extra: $value + 1px)\n  @return $extra\n.button\n  margin: offset(2px)",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.static-named-default-argument-prior-parameter",
            dialect: StyleDialect::Sass,
            source: "@function offset($value, $extra: $value + 1px)\n  @return $extra\n.button\n  margin: offset($value: 2px)",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.static-named-function-arguments",
            dialect: StyleDialect::Sass,
            source: "@function pair($left, $right)\n  @return $left + $right\n.button\n  margin: pair($right: 2px, $left: 1px)",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.static-named-argument-default-tail",
            dialect: StyleDialect::Sass,
            source: "@function pair($left, $right: 2px)\n  @return $left + $right\n.button\n  margin: pair($left: 1px)",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.static-hyphen-underscore-function-reference",
            dialect: StyleDialect::Sass,
            source: "@function gap($base_value)\n  @return $base-value + 1px\n.button\n  margin: gap(2px)",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.static-hyphen-underscore-named-argument",
            dialect: StyleDialect::Sass,
            source: "@function gap($base_value)\n  @return $base-value + 1px\n.button\n  margin: gap($base-value: 2px)",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.static-each-return",
            dialect: StyleDialect::Sass,
            source: "@function tone($target)\n  @each $name, $tone in (primary: red, secondary: blue)\n    @if $name == $target\n      @return $tone\n  @return black\n.button\n  color: tone(secondary)",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.static-each-function-source-return",
            dialect: StyleDialect::Sass,
            source: "@function pick($target)\n  @each $item in append(1px 2px, 3px)\n    @if $item == $target\n      @return $item\n  @return 0px\n.button\n  margin: pick(3px)",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.static-each-tuple-function-source-return",
            dialect: StyleDialect::Sass,
            source: "@function width-for($target)\n  @each $width, $style in zip(1px 2px, solid dashed)\n    @if $style == $target\n      @return $width\n  @return 0px\n.button\n  margin: width-for(dashed)",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.nested-static-loop-return",
            dialect: StyleDialect::Sass,
            source: "@function collect($target)\n  @for $i from 1 through 2\n    @for $j from 1 through 2\n      @if $i == $target\n        @return $i + $j\n  @return 0\n.button\n  z-index: collect(2)",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.dynamic-loop-top",
            dialect: StyleDialect::Sass,
            source: "@function collect($count)\n  @for $i from 1 through $count\n    @return $i\n.button\n  z-index: collect(var(--count))",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.recursive-mixin-cap",
            dialect: StyleDialect::Sass,
            source: "@mixin again\n  @include again\n.button\n  @include again",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.mixin-branch-control-flow",
            dialect: StyleDialect::Sass,
            source: "@mixin tone($enabled)\n  @if $enabled\n    color: red\n  @else\n    color: blue\n.button\n  @include tone(false)",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.mixin-loop-control-flow",
            dialect: StyleDialect::Sass,
            source: "@mixin items()\n  @for $i from 1 through 3\n    order: $i\n.button\n  @include items()",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.static-named-mixin-arguments",
            dialect: StyleDialect::Sass,
            source: "@mixin tone($color, $gap: 1px)\n  color: $color\n  margin: $gap\n.button\n  @include tone($gap: 2px, $color: blue)",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.static-named-mixin-default-tail",
            dialect: StyleDialect::Sass,
            source: "@mixin tone($color, $gap: 2px)\n  color: $color\n  margin: $gap\n.button\n  @include tone($color: blue)",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.static-mixin-default-argument-prior-parameter",
            dialect: StyleDialect::Sass,
            source: "@mixin tone($color, $border: $color)\n  color: $color\n  border-color: $border\n.button\n  @include tone(blue)",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.static-named-mixin-default-argument-prior-parameter",
            dialect: StyleDialect::Sass,
            source: "@mixin tone($color, $border: $color)\n  color: $color\n  border-color: $border\n.button\n  @include tone($color: blue)",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.static-mixin-content-block",
            dialect: StyleDialect::Sass,
            source: "@mixin tone($color)\n  @content\n  color: $color\n.button\n  @include tone(red)\n    background: white",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.static-mixin-content-arguments",
            dialect: StyleDialect::Sass,
            source: "@mixin apply($color)\n  @content($color)\n.button\n  @include apply(red) using ($tone)\n    color: $tone",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.static-mixin-content-expression-arguments",
            dialect: StyleDialect::Sass,
            source: "@mixin apply($color, $gap)\n  @content($color, $gap + 1px)\n.button\n  @include apply(red, 1px) using ($tone, $space)\n    color: $tone\n    margin: $space",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.static-mixin-content-nested-include",
            dialect: StyleDialect::Sass,
            source: "@mixin spacing($gap)\n  margin: $gap\n@mixin apply($gap)\n  @content($gap)\n  color: blue\n.button\n  @include apply(2px) using ($space)\n    @include spacing($space)\n    background: white",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.static-nested-mixin-include",
            dialect: StyleDialect::Sass,
            source: "@mixin spacing($gap)\n  margin: $gap\n@mixin tone($gap, $color: red)\n  @include spacing($gap)\n  color: $color\n.button\n  @include tone(2px, blue)",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "sass.static-hyphen-underscore-mixin-include",
            dialect: StyleDialect::Sass,
            source: "@mixin tone_color($color)\n  color: $color\n.button\n  @include tone-color(green)",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "css.flat-rejected",
            dialect: StyleDialect::Css,
            source: ".button { color: red; }",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_flow_oracle_corpus_reports_u3_surfaces() {
        let report = summarize_scss_control_flow_oracle_corpus();

        assert_eq!(report.schema_version, "0");
        assert_eq!(report.product, "omena-scss-eval.control-flow-oracle-corpus");
        assert_eq!(report.mode, "oracleOnly");
        assert_eq!(report.value_type, "AbstractCssValueV0");
        assert_eq!(report.node_key_type, "StableNodeKeyV0");
        assert_eq!(
            report.typed_value_lattice_witness.payload_type,
            "AbstractCssTypedValueV0"
        );
        assert_eq!(report.typed_value_lattice_witness.typed_payload_count, 5);
        assert_eq!(report.typed_value_lattice_witness.raw_value_count, 1);
        assert_eq!(report.recursion_cap, SCSS_CALL_RETURN_RECURSION_LIMIT);
        assert_eq!(report.fixture_count, 69);
        assert_eq!(report.scss_fixture_count, 34);
        assert_eq!(report.sass_fixture_count, 34);
        assert_eq!(report.supported_fixture_count, 68);
        assert_eq!(report.rejected_flat_css_fixture_count, 1);
        assert!(report.branch_fixture_count >= 5);
        assert!(report.loop_fixture_count >= 6);
        assert!(report.back_edge_fixture_count >= 6);
        assert!(report.call_return_fixture_count >= 5);
        assert!(report.resolved_call_return_fixture_count >= 4);
        assert!(report.top_call_return_fixture_count >= 1);
        assert!(report.recursive_call_fixture_count >= 1);
        assert!(!report.widening_witness_converged);
        assert_eq!(
            report.widening_witness.iteration_count,
            report.widening_witness.max_iterations
        );
        assert_eq!(
            report.widening_witness_widened_to_top_count,
            report.widening_witness.node_count
        );
        assert_eq!(
            report.widening_witness.output_top_count,
            report.widening_witness.node_count
        );
        assert_eq!(
            report.converged_value_analysis_fixture_count,
            report.supported_fixture_count
        );
        assert_eq!(
            report.prune_reachability_fixture_count,
            report.supported_fixture_count
        );
        assert_eq!(
            report.prune_reachability_flat_css_cfg_built_count,
            report.supported_fixture_count
        );
        assert!(report.prune_reachability_changed_fixture_count > 0);
        assert_eq!(
            report.flat_css_cfg_built_count,
            report.supported_fixture_count
        );
        assert_eq!(report.merged_cross_file_graph_count, 0);
        assert!(report.all_supported_fixtures_converged);
        assert!(!report.no_flat_css_cfg_built);
        assert!(report.no_merged_cross_file_graph);

        assert!(report.fixtures.iter().any(|fixture| {
            fixture.id == "sass.static-while-loop"
                && fixture.control_flow_available
                && fixture.value_analysis_converged
                && fixture.loop_block_count >= 1
                && fixture.back_edge_count >= 1
        }));

        assert!(report.fixtures.iter().any(|fixture| {
            fixture.id == "sass.static-for-return"
                && fixture.control_flow_available
                && fixture.call_return_available
                && fixture.value_analysis_converged
                && fixture.loop_block_count >= 1
                && fixture.back_edge_count >= 1
                && fixture.call_resolved_return_value_count >= 1
        }));

        assert!(report.fixtures.iter().any(|fixture| {
            fixture.id == "scss.descending-static-for-return"
                && fixture.call_resolved_return_value_count == 1
                && fixture.value_analysis_converged
        }));
        assert!(report.fixtures.iter().any(|fixture| {
            fixture.id == "sass.descending-static-for-return"
                && fixture.dialect == "sass"
                && fixture.call_resolved_return_value_count == 1
                && fixture.value_analysis_converged
        }));
        assert!(report.fixtures.iter().any(|fixture| {
            fixture.id == "scss.static-for-expression-bounds"
                && fixture.call_resolved_return_value_count == 1
                && fixture.value_analysis_converged
        }));
        assert!(report.fixtures.iter().any(|fixture| {
            fixture.id == "scss.static-for-finite-set-bound"
                && fixture.loop_block_count == 2
                && fixture.back_edge_count == 2
                && fixture.value_analysis_converged
        }));
        assert!(report.fixtures.iter().any(|fixture| {
            fixture.id == "sass.static-for-finite-set-bound"
                && fixture.dialect == "sass"
                && fixture.loop_block_count == 2
                && fixture.back_edge_count == 2
                && fixture.value_analysis_converged
        }));
        assert!(report.fixtures.iter().any(|fixture| {
            fixture.id == "sass.static-for-expression-bounds"
                && fixture.dialect == "sass"
                && fixture.call_resolved_return_value_count == 1
                && fixture.value_analysis_converged
        }));
        assert!(report.fixtures.iter().any(|fixture| {
            fixture.id == "scss.static-while-expression-step"
                && fixture.call_resolved_return_value_count == 1
                && fixture.value_analysis_converged
        }));
        assert!(report.fixtures.iter().any(|fixture| {
            fixture.id == "scss.static-while-finite-set-bound"
                && fixture.loop_block_count == 2
                && fixture.back_edge_count == 2
                && fixture.value_analysis_converged
        }));
        assert!(report.fixtures.iter().any(|fixture| {
            fixture.id == "sass.static-while-finite-set-bound"
                && fixture.dialect == "sass"
                && fixture.loop_block_count == 2
                && fixture.back_edge_count == 2
                && fixture.value_analysis_converged
        }));
        assert!(report.fixtures.iter().any(|fixture| {
            fixture.id == "sass.static-while-expression-step"
                && fixture.dialect == "sass"
                && fixture.call_resolved_return_value_count == 1
                && fixture.value_analysis_converged
        }));
        for id in [
            "scss.static-default-function-arguments",
            "scss.static-default-argument-prior-parameter",
            "scss.static-named-default-argument-prior-parameter",
            "sass.static-default-function-arguments",
            "sass.static-default-argument-prior-parameter",
            "sass.static-named-default-argument-prior-parameter",
            "scss.static-named-function-arguments",
            "scss.static-named-argument-default-tail",
            "sass.static-named-function-arguments",
            "sass.static-named-argument-default-tail",
            "scss.static-hyphen-underscore-function-reference",
            "scss.static-hyphen-underscore-named-argument",
            "sass.static-hyphen-underscore-function-reference",
            "sass.static-hyphen-underscore-named-argument",
        ] {
            assert!(
                report.fixtures.iter().any(|fixture| {
                    fixture.id == id
                        && fixture.call_resolved_return_value_count == 1
                        && fixture.exact_call_resolved_return_value_count == 1
                        && fixture.value_analysis_converged
                }),
                "missing control-flow default-argument oracle fixture {id}"
            );
        }
        assert!(report.fixtures.iter().any(|fixture| {
            fixture.id == "sass.static-each-return"
                && fixture.dialect == "sass"
                && fixture.call_resolved_return_value_count == 1
                && fixture.value_analysis_converged
        }));
        assert!(report.fixtures.iter().any(|fixture| {
            fixture.id == "sass.static-each-tuple-function-source-return"
                && fixture.dialect == "sass"
                && fixture.call_resolved_return_value_count == 1
                && fixture.value_analysis_converged
        }));
        assert!(report.fixtures.iter().any(|fixture| {
            fixture.id == "sass.nested-static-loop-return"
                && fixture.dialect == "sass"
                && fixture.call_resolved_return_value_count == 1
                && fixture.value_analysis_converged
        }));
        assert!(report.fixtures.iter().any(|fixture| {
            fixture.id == "sass.dynamic-loop-top"
                && fixture.dialect == "sass"
                && fixture.top_call_resolved_return_value_count == 1
                && fixture.value_analysis_converged
        }));
        assert!(report.fixtures.iter().any(|fixture| {
            fixture.id == "sass.recursive-mixin-cap"
                && fixture.dialect == "sass"
                && fixture.capped_recursive_call_count == 1
                && fixture.value_analysis_converged
        }));
        assert!(report.fixtures.iter().any(|fixture| {
            fixture.id == "scss.mixin-branch-control-flow"
                && fixture.control_flow_available
                && fixture.call_return_available
                && fixture.branch_block_count == 2
                && fixture.call_return_edge_count == 1
                && fixture.value_analysis_converged
        }));
        assert!(report.fixtures.iter().any(|fixture| {
            fixture.id == "sass.mixin-branch-control-flow"
                && fixture.dialect == "sass"
                && fixture.control_flow_available
                && fixture.call_return_available
                && fixture.branch_block_count == 2
                && fixture.call_return_edge_count == 1
                && fixture.value_analysis_converged
        }));
        assert!(report.fixtures.iter().any(|fixture| {
            fixture.id == "scss.mixin-loop-control-flow"
                && fixture.control_flow_available
                && fixture.call_return_available
                && fixture.loop_block_count == 1
                && fixture.back_edge_count == 1
                && fixture.call_return_edge_count == 1
                && fixture.value_analysis_converged
        }));
        assert!(report.fixtures.iter().any(|fixture| {
            fixture.id == "sass.mixin-loop-control-flow"
                && fixture.dialect == "sass"
                && fixture.control_flow_available
                && fixture.call_return_available
                && fixture.loop_block_count == 1
                && fixture.back_edge_count == 1
                && fixture.call_return_edge_count == 1
                && fixture.value_analysis_converged
        }));
        for id in [
            "scss.static-named-mixin-arguments",
            "scss.static-named-mixin-default-tail",
            "scss.static-mixin-default-argument-prior-parameter",
            "scss.static-named-mixin-default-argument-prior-parameter",
            "scss.static-mixin-content-block",
            "scss.static-mixin-content-arguments",
            "scss.static-mixin-content-expression-arguments",
            "scss.static-hyphen-underscore-mixin-include",
            "sass.static-named-mixin-arguments",
            "sass.static-named-mixin-default-tail",
            "sass.static-mixin-default-argument-prior-parameter",
            "sass.static-named-mixin-default-argument-prior-parameter",
            "sass.static-mixin-content-block",
            "sass.static-mixin-content-arguments",
            "sass.static-mixin-content-expression-arguments",
            "sass.static-hyphen-underscore-mixin-include",
        ] {
            assert!(
                report.fixtures.iter().any(|fixture| {
                    fixture.id == id
                        && fixture.call_return_available
                        && fixture.call_return_edge_count == 1
                        && fixture.value_analysis_converged
                }),
                "missing control-flow mixin call edge oracle fixture {id}"
            );
        }
        assert!(report.fixtures.iter().any(|fixture| {
            fixture.id == "scss.static-mixin-content-nested-include"
                && fixture.call_return_available
                && fixture.call_return_edge_count == 2
                && fixture.value_analysis_converged
        }));
        assert!(report.fixtures.iter().any(|fixture| {
            fixture.id == "sass.static-mixin-content-nested-include"
                && fixture.call_return_available
                && fixture.call_return_edge_count == 2
                && fixture.value_analysis_converged
        }));
        for id in [
            "scss.static-nested-mixin-include",
            "sass.static-nested-mixin-include",
        ] {
            assert!(
                report.fixtures.iter().any(|fixture| {
                    fixture.id == id
                        && fixture.call_return_available
                        && fixture.call_return_edge_count == 2
                        && fixture.value_analysis_converged
                }),
                "missing nested mixin call edge oracle fixture {id}"
            );
        }
    }
}
