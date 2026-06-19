use omena_scss_eval::{
    OmenaScssEvalControlFlowOracleCorpusReportV0, OmenaScssEvalStaticStylesheetEvaluationV0,
    OmenaScssEvalStaticStylesheetOracleCorpusReportV0, OmenaScssEvalStaticValueResolutionReportV0,
    analyze_scss_control_flow_values, derive_static_stylesheet_module_evaluation,
    summarize_scss_call_return_ir, summarize_scss_control_flow_ir,
    summarize_scss_control_flow_oracle_corpus, summarize_static_stylesheet_oracle_corpus,
    summarize_static_stylesheet_value_resolution,
};
use serde::Serialize;

use crate::{
    OMENA_QUERY_CURRENT_SCHEMA_VERSION, OmenaParserStyleDialect,
    OmenaQueryScssEvalCallReturnIrSummaryV0, OmenaQueryScssEvalControlFlowIrSummaryV0,
    OmenaQueryScssEvalControlFlowValueAnalysisV0,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryScssEvaluatorControlFlowSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub mode: &'static str,
    pub dialect: &'static str,
    pub value_type: &'static str,
    pub supported_dialect: bool,
    pub flat_css_cfg_built: bool,
    pub merged_cross_file_graph: bool,
    pub control_flow_block_count: usize,
    pub control_flow_branch_block_count: usize,
    pub control_flow_loop_block_count: usize,
    pub control_flow_back_edge_count: usize,
    pub call_return_node_count: usize,
    pub call_return_edge_count: usize,
    pub call_resolved_return_value_count: usize,
    pub exact_call_resolved_return_value_count: usize,
    pub value_analysis_converged: bool,
    pub value_analysis_iteration_count: usize,
    pub value_analysis_widened_to_top_count: usize,
    pub ready_surfaces: Vec<&'static str>,
    pub control_flow_ir: Option<OmenaQueryScssEvalControlFlowIrSummaryV0>,
    pub value_analysis: Option<OmenaQueryScssEvalControlFlowValueAnalysisV0>,
    pub call_return_ir: Option<OmenaQueryScssEvalCallReturnIrSummaryV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryScssEvaluatorControlFlowOracleCorpusSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub mode: &'static str,
    pub value_type: &'static str,
    pub node_key_type: &'static str,
    pub recursion_cap: usize,
    pub fixture_count: usize,
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
    pub corpus: OmenaScssEvalControlFlowOracleCorpusReportV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStaticStylesheetEvaluatorSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub mode: &'static str,
    pub dialect: &'static str,
    pub value_type: &'static str,
    pub supported_dialect: bool,
    pub product_output_source: &'static str,
    pub legacy_output_consumed_until_cutover: bool,
    pub evaluation_available: bool,
    pub value_resolution_available: bool,
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
    pub evaluation: Option<OmenaScssEvalStaticStylesheetEvaluationV0>,
    pub value_resolution: Option<OmenaScssEvalStaticValueResolutionReportV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStaticStylesheetEvaluatorOracleCorpusSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub mode: &'static str,
    pub value_type: &'static str,
    pub product_output_source: &'static str,
    pub fixture_count: usize,
    pub scss_fixture_count: usize,
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
    pub all_legacy_declaration_values_preserved: bool,
    pub all_native_edit_outputs_match_evaluated_css: bool,
    pub corpus: OmenaScssEvalStaticStylesheetOracleCorpusReportV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStaticLifExportsSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub mode: &'static str,
    pub dialect: &'static str,
    pub source_syntax: &'static str,
    pub sif_superset: bool,
    pub less_specific_export_count: usize,
    pub less_variable_count: usize,
    pub less_mixin_count: usize,
    pub less_detached_ruleset_count: usize,
    pub less_variable_names: Vec<String>,
    pub less_mixin_names: Vec<String>,
    pub less_detached_ruleset_names: Vec<String>,
    pub sif_variable_count: usize,
    pub sif_mixin_count: usize,
    pub sif_function_count: usize,
    pub sif_placeholder_count: usize,
    pub sif_forward_count: usize,
    pub exports: omena_sif::OmenaLifExportsV1,
}

pub fn summarize_omena_query_static_stylesheet_evaluator_from_source(
    source: &str,
    dialect: OmenaParserStyleDialect,
) -> OmenaQueryStaticStylesheetEvaluatorSummaryV0 {
    let evaluation = derive_static_stylesheet_module_evaluation(source, dialect);
    let value_resolution = evaluation
        .as_ref()
        .map(|evaluation| evaluation.value_resolution.clone())
        .or_else(|| summarize_static_stylesheet_value_resolution(source, dialect));
    let oracle = evaluation.as_ref().map(|evaluation| &evaluation.oracle);
    let supported_dialect = matches!(
        dialect,
        OmenaParserStyleDialect::Scss
            | OmenaParserStyleDialect::Sass
            | OmenaParserStyleDialect::Less
    );

    OmenaQueryStaticStylesheetEvaluatorSummaryV0 {
        schema_version: OMENA_QUERY_CURRENT_SCHEMA_VERSION,
        product: "omena-query.static-stylesheet-evaluator",
        mode: "oracleOnly",
        dialect: omena_query_boundary_style_dialect_label(dialect),
        value_type: "AbstractCssValueV0",
        supported_dialect,
        product_output_source: oracle.map_or("none", |oracle| oracle.product_output_source),
        legacy_output_consumed_until_cutover: oracle
            .is_some_and(|oracle| oracle.product_output_source == "legacyEvaluatedCss"),
        evaluation_available: evaluation.is_some(),
        value_resolution_available: value_resolution.is_some(),
        native_edit_output: evaluation
            .as_ref()
            .map(|evaluation| evaluation.native_edit_output.clone()),
        divergence_count: oracle.map_or(0, |oracle| oracle.divergence_count),
        all_legacy_declaration_values_preserved: oracle
            .is_some_and(|oracle| oracle.all_legacy_declaration_values_preserved),
        native_replacement_count: evaluation
            .as_ref()
            .map_or(0, |evaluation| evaluation.replacement_count),
        native_replacement_legacy_reflection_count: evaluation.as_ref().map_or(0, |evaluation| {
            evaluation.native_replacement_legacy_reflection_count
        }),
        native_replacement_legacy_unreflected_count: evaluation.as_ref().map_or(0, |evaluation| {
            evaluation.native_replacement_legacy_unreflected_count
        }),
        native_edit_count: evaluation
            .as_ref()
            .map_or(0, |evaluation| evaluation.native_edit_count),
        native_value_edit_count: evaluation
            .as_ref()
            .map_or(0, |evaluation| evaluation.native_value_edit_count),
        native_structural_edit_count: evaluation
            .as_ref()
            .map_or(0, |evaluation| evaluation.native_structural_edit_count),
        native_edit_output_matches_evaluated_css: evaluation
            .as_ref()
            .is_some_and(|evaluation| evaluation.native_edit_output_matches_evaluated_css),
        native_value_reference_count: value_resolution
            .as_ref()
            .map_or(0, |resolution| resolution.reference_count),
        native_resolved_value_count: value_resolution
            .as_ref()
            .map_or(0, |resolution| resolution.resolved_count),
        native_raw_value_count: value_resolution
            .as_ref()
            .map_or(0, |resolution| resolution.raw_count),
        native_top_value_count: value_resolution
            .as_ref()
            .map_or(0, |resolution| resolution.top_count),
        evaluation,
        value_resolution,
    }
}

pub fn summarize_omena_query_static_lif_exports_from_source(
    source: &str,
    dialect: OmenaParserStyleDialect,
) -> OmenaQueryStaticLifExportsSummaryV0 {
    let source_syntax = omena_query_boundary_sif_source_syntax(dialect);
    let exports = omena_sif::generate_static_omena_lif_exports_v1(
        omena_sif::OmenaSifStaticGeneratorInputV1 {
            canonical_url: "memory:omena-query-static-lif",
            source,
            syntax: source_syntax.clone(),
        },
    );
    let less_variable_names = exports
        .less_variables
        .iter()
        .map(|variable| variable.name.clone())
        .collect::<Vec<_>>();
    let less_mixin_names = exports
        .less_mixins
        .iter()
        .map(|mixin| mixin.name.clone())
        .collect::<Vec<_>>();
    let less_detached_ruleset_names = exports
        .less_detached_rulesets
        .iter()
        .map(|ruleset| ruleset.name.clone())
        .collect::<Vec<_>>();

    OmenaQueryStaticLifExportsSummaryV0 {
        schema_version: OMENA_QUERY_CURRENT_SCHEMA_VERSION,
        product: "omena-query.static-lif-exports",
        mode: "staticInterfaceOnly",
        dialect: omena_query_boundary_style_dialect_label(dialect),
        source_syntax: omena_query_boundary_sif_source_syntax_label(source_syntax),
        sif_superset: true,
        less_specific_export_count: exports.less_variables.len()
            + exports.less_mixins.len()
            + exports.less_detached_rulesets.len(),
        less_variable_count: exports.less_variables.len(),
        less_mixin_count: exports.less_mixins.len(),
        less_detached_ruleset_count: exports.less_detached_rulesets.len(),
        less_variable_names,
        less_mixin_names,
        less_detached_ruleset_names,
        sif_variable_count: exports.sif_exports.variables.len(),
        sif_mixin_count: exports.sif_exports.mixins.len(),
        sif_function_count: exports.sif_exports.functions.len(),
        sif_placeholder_count: exports.sif_exports.placeholders.len(),
        sif_forward_count: exports.sif_exports.forwards.len(),
        exports,
    }
}

pub fn summarize_omena_query_static_stylesheet_evaluator_oracle_corpus()
-> OmenaQueryStaticStylesheetEvaluatorOracleCorpusSummaryV0 {
    let corpus = summarize_static_stylesheet_oracle_corpus();
    OmenaQueryStaticStylesheetEvaluatorOracleCorpusSummaryV0 {
        schema_version: OMENA_QUERY_CURRENT_SCHEMA_VERSION,
        product: "omena-query.static-stylesheet-evaluator-oracle-corpus",
        mode: corpus.mode,
        value_type: corpus.value_type,
        product_output_source: corpus.product_output_source,
        fixture_count: corpus.fixture_count,
        scss_fixture_count: corpus.scss_fixture_count,
        less_fixture_count: corpus.less_fixture_count,
        evaluated_fixture_count: corpus.evaluated_fixture_count,
        missing_evaluation_count: corpus.missing_evaluation_count,
        divergence_count: corpus.divergence_count,
        native_replacement_count: corpus.native_replacement_count,
        native_replacement_legacy_reflection_count: corpus
            .native_replacement_legacy_reflection_count,
        native_replacement_legacy_unreflected_count: corpus
            .native_replacement_legacy_unreflected_count,
        native_edit_count: corpus.native_edit_count,
        native_value_edit_count: corpus.native_value_edit_count,
        native_structural_edit_count: corpus.native_structural_edit_count,
        native_edit_output_match_count: corpus.native_edit_output_match_count,
        native_value_reference_count: corpus.native_value_reference_count,
        native_resolved_value_count: corpus.native_resolved_value_count,
        native_raw_value_count: corpus.native_raw_value_count,
        native_top_value_count: corpus.native_top_value_count,
        all_legacy_declaration_values_preserved: corpus.all_legacy_declaration_values_preserved,
        all_native_edit_outputs_match_evaluated_css: corpus
            .all_native_edit_outputs_match_evaluated_css,
        corpus,
    }
}

pub fn summarize_omena_query_scss_evaluator_control_flow_from_source(
    source: &str,
    dialect: OmenaParserStyleDialect,
) -> OmenaQueryScssEvaluatorControlFlowSummaryV0 {
    let control_flow_ir = summarize_scss_control_flow_ir(source, dialect);
    let value_analysis = analyze_scss_control_flow_values(source, dialect);
    let call_return_ir = summarize_scss_call_return_ir(source, dialect);
    let supported_dialect = matches!(
        dialect,
        OmenaParserStyleDialect::Scss | OmenaParserStyleDialect::Sass
    );

    OmenaQueryScssEvaluatorControlFlowSummaryV0 {
        schema_version: OMENA_QUERY_CURRENT_SCHEMA_VERSION,
        product: "omena-query.scss-evaluator-control-flow",
        mode: "oracleOnly",
        dialect: omena_query_boundary_style_dialect_label(dialect),
        value_type: "AbstractCssValueV0",
        supported_dialect,
        flat_css_cfg_built: control_flow_ir
            .as_ref()
            .is_some_and(|summary| summary.flat_css_cfg_built),
        merged_cross_file_graph: control_flow_ir
            .as_ref()
            .is_some_and(|summary| summary.merged_cross_file_graph),
        control_flow_block_count: control_flow_ir
            .as_ref()
            .map_or(0, |summary| summary.block_count),
        control_flow_branch_block_count: control_flow_ir
            .as_ref()
            .map_or(0, |summary| summary.branch_block_count),
        control_flow_loop_block_count: control_flow_ir
            .as_ref()
            .map_or(0, |summary| summary.loop_block_count),
        control_flow_back_edge_count: control_flow_ir
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
        value_analysis_converged: value_analysis
            .as_ref()
            .is_some_and(|summary| summary.converged),
        value_analysis_iteration_count: value_analysis
            .as_ref()
            .map_or(0, |summary| summary.iteration_count),
        value_analysis_widened_to_top_count: value_analysis
            .as_ref()
            .map_or(0, |summary| summary.widened_to_top_count),
        ready_surfaces: vec![
            "scssEvaluatorControlFlowIr",
            "scssEvaluatorControlFlowValueAnalysis",
            "scssEvaluatorCallReturnIr",
        ],
        control_flow_ir,
        value_analysis,
        call_return_ir,
    }
}

pub fn summarize_omena_query_scss_evaluator_control_flow_oracle_corpus()
-> OmenaQueryScssEvaluatorControlFlowOracleCorpusSummaryV0 {
    let corpus = summarize_scss_control_flow_oracle_corpus();
    OmenaQueryScssEvaluatorControlFlowOracleCorpusSummaryV0 {
        schema_version: OMENA_QUERY_CURRENT_SCHEMA_VERSION,
        product: "omena-query.scss-evaluator-control-flow-oracle-corpus",
        mode: corpus.mode,
        value_type: corpus.value_type,
        node_key_type: corpus.node_key_type,
        recursion_cap: corpus.recursion_cap,
        fixture_count: corpus.fixture_count,
        supported_fixture_count: corpus.supported_fixture_count,
        rejected_flat_css_fixture_count: corpus.rejected_flat_css_fixture_count,
        control_flow_fixture_count: corpus.control_flow_fixture_count,
        branch_fixture_count: corpus.branch_fixture_count,
        loop_fixture_count: corpus.loop_fixture_count,
        back_edge_fixture_count: corpus.back_edge_fixture_count,
        call_return_fixture_count: corpus.call_return_fixture_count,
        resolved_call_return_fixture_count: corpus.resolved_call_return_fixture_count,
        top_call_return_fixture_count: corpus.top_call_return_fixture_count,
        recursive_call_fixture_count: corpus.recursive_call_fixture_count,
        converged_value_analysis_fixture_count: corpus.converged_value_analysis_fixture_count,
        widened_to_top_fixture_count: corpus.widened_to_top_fixture_count,
        flat_css_cfg_built_count: corpus.flat_css_cfg_built_count,
        merged_cross_file_graph_count: corpus.merged_cross_file_graph_count,
        all_supported_fixtures_converged: corpus.all_supported_fixtures_converged,
        no_flat_css_cfg_built: corpus.no_flat_css_cfg_built,
        no_merged_cross_file_graph: corpus.no_merged_cross_file_graph,
        corpus,
    }
}

fn omena_query_boundary_style_dialect_label(dialect: OmenaParserStyleDialect) -> &'static str {
    match dialect {
        OmenaParserStyleDialect::Css => "css",
        OmenaParserStyleDialect::Scss => "scss",
        OmenaParserStyleDialect::Sass => "sass",
        OmenaParserStyleDialect::Less => "less",
    }
}

fn omena_query_boundary_sif_source_syntax(
    dialect: OmenaParserStyleDialect,
) -> omena_sif::OmenaSifSourceSyntaxV1 {
    match dialect {
        OmenaParserStyleDialect::Css => omena_sif::OmenaSifSourceSyntaxV1::Css,
        OmenaParserStyleDialect::Scss => omena_sif::OmenaSifSourceSyntaxV1::Scss,
        OmenaParserStyleDialect::Sass => omena_sif::OmenaSifSourceSyntaxV1::Sass,
        OmenaParserStyleDialect::Less => omena_sif::OmenaSifSourceSyntaxV1::Less,
    }
}

fn omena_query_boundary_sif_source_syntax_label(
    syntax: omena_sif::OmenaSifSourceSyntaxV1,
) -> &'static str {
    match syntax {
        omena_sif::OmenaSifSourceSyntaxV1::Css => "css",
        omena_sif::OmenaSifSourceSyntaxV1::Scss => "scss",
        omena_sif::OmenaSifSourceSyntaxV1::Sass => "sass",
        omena_sif::OmenaSifSourceSyntaxV1::Less => "less",
    }
}
