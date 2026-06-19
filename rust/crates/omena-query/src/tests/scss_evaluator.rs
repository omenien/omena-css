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
