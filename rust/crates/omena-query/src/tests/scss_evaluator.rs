use crate::{
    OmenaParserStyleDialect, summarize_omena_query_scss_evaluator_control_flow_from_source,
    summarize_omena_query_static_stylesheet_evaluator_from_source,
};

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
        "$ceil: ceil(1.2px); $floor: floor(1.8px); .card { top: $ceil; bottom: $floor; }",
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
