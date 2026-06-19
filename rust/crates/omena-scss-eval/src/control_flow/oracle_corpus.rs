use omena_parser::StyleDialect;
use serde::Serialize;

use super::{
    SCSS_CALL_RETURN_RECURSION_LIMIT, analyze_scss_control_flow_values, dialect_label,
    summarize_scss_call_return_ir, summarize_scss_control_flow_ir,
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
    pub flat_css_cfg_built_count: usize,
    pub merged_cross_file_graph_count: usize,
    pub all_supported_fixtures_converged: bool,
    pub no_flat_css_cfg_built: bool,
    pub no_merged_cross_file_graph: bool,
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
        flat_css_cfg_built_count,
        merged_cross_file_graph_count,
        all_supported_fixtures_converged,
        no_flat_css_cfg_built: flat_css_cfg_built_count == 0,
        no_merged_cross_file_graph: merged_cross_file_graph_count == 0,
        fixtures,
    }
}

fn scss_control_flow_oracle_corpus_fixture_report(
    fixture: &ScssControlFlowOracleCorpusFixtureV0,
) -> OmenaScssEvalControlFlowOracleCorpusFixtureReportV0 {
    let supported_dialect = matches!(fixture.dialect, StyleDialect::Scss | StyleDialect::Sass);
    let control_flow_ir = summarize_scss_control_flow_ir(fixture.source, fixture.dialect);
    let value_analysis = analyze_scss_control_flow_values(fixture.source, fixture.dialect);
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
            id: "scss.static-for-return",
            dialect: StyleDialect::Scss,
            source: "@function pick($target) { @for $i from 1 through 3 { @if $i == $target { @return $i; } } @return 0; } .button { z-index: pick(2); }",
        },
        ScssControlFlowOracleCorpusFixtureV0 {
            id: "scss.static-each-return",
            dialect: StyleDialect::Scss,
            source: "@function tone($target) { @each $name, $tone in (primary: red, secondary: blue) { @if $name == $target { @return $tone; } } @return black; } .button { color: tone(secondary); }",
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
            id: "sass.branch-if-else",
            dialect: StyleDialect::Sass,
            source: "$enabled: true\n@if $enabled\n  .on\n    color: green\n@else\n  .off\n    color: gray",
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
        assert_eq!(report.recursion_cap, SCSS_CALL_RETURN_RECURSION_LIMIT);
        assert_eq!(report.fixture_count, 9);
        assert_eq!(report.scss_fixture_count, 7);
        assert_eq!(report.sass_fixture_count, 1);
        assert_eq!(report.supported_fixture_count, 8);
        assert_eq!(report.rejected_flat_css_fixture_count, 1);
        assert!(report.branch_fixture_count >= 4);
        assert!(report.loop_fixture_count >= 4);
        assert!(report.back_edge_fixture_count >= 4);
        assert!(report.call_return_fixture_count >= 4);
        assert!(report.resolved_call_return_fixture_count >= 3);
        assert!(report.top_call_return_fixture_count >= 1);
        assert!(report.recursive_call_fixture_count >= 1);
        assert_eq!(
            report.converged_value_analysis_fixture_count,
            report.supported_fixture_count
        );
        assert_eq!(report.flat_css_cfg_built_count, 0);
        assert_eq!(report.merged_cross_file_graph_count, 0);
        assert!(report.all_supported_fixtures_converged);
        assert!(report.no_flat_css_cfg_built);
        assert!(report.no_merged_cross_file_graph);
    }
}
