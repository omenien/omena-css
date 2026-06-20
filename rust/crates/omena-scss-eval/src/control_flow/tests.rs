use super::*;
use omena_abstract_value::AbstractCssValueV0;

#[test]
fn scss_control_flow_ir_summarizes_branch_and_loop_blocks() {
    let source = "@if $enabled { .on { color: green; } } @else { .off { color: red; } } @for $i from 1 through 3 { .n { order: $i; } } @each $k, $v in $map { .e { color: $v; } } @while $enabled { .w { color: red; } }";
    let report = summarize_scss_control_flow_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.mode, "oracleOnly");
    assert!(!report.flat_css_cfg_built);
    assert!(!report.merged_cross_file_graph);
    assert_eq!(report.node_key_type, "StableNodeKeyV0");
    assert_eq!(report.block_count, 5);
    assert_eq!(report.branch_block_count, 2);
    assert_eq!(report.loop_block_count, 3);
    assert_eq!(report.back_edge_count, 3);
    assert!(report.blocks.iter().any(|block| {
        block
            .node_key
            .as_str()
            .starts_with("scss-control:branchIf@")
    }));
}

#[test]
fn scss_control_flow_ir_counts_else_if_as_conditional_branch() {
    let source = "$enabled: false; $fallback: true; @if $enabled { .on { color: green; } } @else if $fallback { .fallback { color: yellow; } } @else { .off { color: red; } }";
    let report = summarize_scss_control_flow_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 3);
    assert_eq!(report.branch_block_count, 3);
    assert_eq!(report.edge_count, 5);
    assert_eq!(report.blocks[0].kind, "branchIf");
    assert_eq!(report.blocks[0].successor_count, 2);
    assert_eq!(report.blocks[1].kind, "branchElse");
    assert_eq!(report.blocks[1].header_text, "if $fallback");
    assert_eq!(report.blocks[1].successor_count, 2);
    assert_eq!(report.blocks[2].kind, "branchElse");
    assert_eq!(report.blocks[2].successor_count, 1);
}

#[test]
fn control_flow_ir_does_not_build_flat_css_cfg() {
    assert!(summarize_scss_control_flow_ir(".button { color: red; }", StyleDialect::Css).is_none());
}

#[test]
fn control_flow_value_analysis_uses_single_abstract_css_value_domain() {
    let source = "$enabled: 1; @if $enabled { .on { color: green; } } @for $i from 1 through 3 { .n { order: $i; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.mode, "oracleOnly");
    assert_eq!(report.value_type, "AbstractCssValueV0");
    assert!(!report.flat_css_cfg_built);
    assert!(!report.merged_cross_file_graph);
    assert!(report.converged);
    assert_eq!(report.block_count, 2);
    assert_eq!(report.back_edge_count, 1);
    assert_eq!(report.loop_carried_binding_count, 1);
    assert_eq!(report.widened_to_top_count, 0);
    assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[0].transfer_value_kind, Some("exact"));
    assert_eq!(report.blocks[0].transfer_truthiness, Some("truthy"));
    assert_eq!(report.blocks[1].transfer_kind, "loopCarriedBindings");
    assert_eq!(report.blocks[1].transfer_value_kind, Some("finiteSet"));
    assert_eq!(report.blocks[1].loop_carried_bindings, vec!["$i"]);
    assert_eq!(report.blocks[1].loop_carried_binding_values.len(), 1);
    assert_eq!(report.blocks[1].loop_carried_binding_values[0].name, "$i");
    assert_eq!(
        report.blocks[1].loop_carried_binding_values[0].value_kind,
        "finiteSet"
    );
    assert_eq!(report.blocks[1].output_value_kind, "finiteSet");
}

#[test]
fn control_flow_value_analysis_uses_loop_carried_bindings_for_nested_branch_conditions() {
    let source = "@for $i from 1 through 3 { @if $i == 2 { .item { order: $i; } } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 2);
    assert_eq!(report.blocks[0].transfer_kind, "loopCarriedBindings");
    assert_eq!(report.blocks[0].loop_carried_bindings, vec!["$i"]);
    assert_eq!(report.blocks[1].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[1].transfer_value_kind, Some("finiteSet"));
    assert_eq!(
        report.blocks[1].transfer_value,
        Some(AbstractCssValueV0::FiniteSet {
            values: vec!["false".to_string(), "true".to_string()]
        })
    );
    assert_eq!(report.blocks[1].transfer_truthiness, None);
}

#[test]
fn control_flow_value_analysis_does_not_leak_loop_bindings_after_loop_body() {
    let source =
        "@for $i from 1 through 3 { .item { order: $i; } } @if $i == 2 { .leak { order: $i; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 2);
    assert_eq!(report.blocks[0].loop_carried_bindings, vec!["$i"]);
    assert_eq!(report.blocks[1].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[1].transfer_value_kind, Some("top"));
    assert_eq!(report.blocks[1].output_value_kind, "top");
}

#[test]
fn control_flow_value_analysis_keeps_dynamic_each_loop_top() {
    let source = "@each $key, $value in $tokens { .item { color: $value; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.back_edge_count, 1);
    assert_eq!(
        report.blocks[0].loop_carried_bindings,
        vec!["$key", "$value"]
    );
    assert!(
        report.blocks[0]
            .loop_carried_binding_values
            .iter()
            .all(|binding| binding.value_kind == "top")
    );
    assert_eq!(report.blocks[0].output_value_kind, "top");
}

#[test]
fn control_flow_value_analysis_reports_sass_branch_truthiness() {
    let source = "$enabled: false; @if $enabled { .on { color: green; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[0].transfer_value_kind, Some("raw"));
    assert_eq!(report.blocks[0].transfer_truthiness, Some("falsey"));
}

#[test]
fn control_flow_value_analysis_reports_sass_not_branch_truthiness() {
    let source = "$enabled: true; @if not $enabled { .off { color: red; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[0].transfer_truthiness, Some("falsey"));
}

#[test]
fn control_flow_value_analysis_reports_sass_boolean_branch_truthiness() {
    let source = "$enabled: true; @if $enabled and false { .off { color: red; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[0].transfer_truthiness, Some("falsey"));
}

#[test]
fn control_flow_value_analysis_reduces_static_if_variable_bindings() {
    let source = "$enabled: if(true, true, false); @if $enabled { .on { color: green; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[0].transfer_value_kind, Some("raw"));
    assert_eq!(report.blocks[0].transfer_truthiness, Some("truthy"));
}

#[test]
fn control_flow_value_analysis_reduces_numeric_variable_bindings() {
    let source = "$gap: 1px + 2px; @if $gap == 3px { .on { color: green; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[0].transfer_truthiness, Some("truthy"));
}

#[test]
fn control_flow_value_analysis_reports_sass_variable_metadata_branch_truthiness() {
    let source = "$enabled: true; @if variable-exists(\"enabled\") and not variable-exists(\"missing\") { .on { color: green; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[0].transfer_truthiness, Some("truthy"));
}

#[test]
fn control_flow_value_analysis_reports_sass_global_variable_metadata_branch_truthiness() {
    let source = "$theme: dark; @if global-variable-exists(\"theme\") and not meta.global-variable-exists(\"missing\") { .on { color: green; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[0].transfer_truthiness, Some("truthy"));
}

#[test]
fn control_flow_value_analysis_reports_sass_function_metadata_branch_truthiness() {
    let source = "@function present() { @return 1px; } @if function-exists(\"present\") and not function-exists(\"missing\") { .on { color: green; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[0].transfer_truthiness, Some("truthy"));
}

#[test]
fn control_flow_value_analysis_reports_sass_builtin_function_metadata_branch_truthiness() {
    let source = "@if meta.function-exists(\"scale-color\") { .on { color: green; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[0].transfer_truthiness, Some("truthy"));
}

#[test]
fn control_flow_value_analysis_preserves_function_exists_declaration_order() {
    let source = "@if function-exists(\"later\") { .on { color: green; } } @function later() { @return 1px; }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[0].transfer_truthiness, Some("falsey"));
}

#[test]
fn control_flow_value_analysis_reports_sass_mixin_metadata_branch_truthiness() {
    let source = "@mixin present { color: red; } @if mixin-exists(\"present\") and not meta.mixin-exists(\"missing\") { .on { color: green; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[0].transfer_truthiness, Some("truthy"));
}

#[test]
fn control_flow_value_analysis_preserves_mixin_exists_declaration_order() {
    let source =
        "@if mixin-exists(\"later\") { .on { color: green; } } @mixin later { color: red; }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[0].transfer_truthiness, Some("falsey"));
}

#[test]
fn control_flow_value_analysis_keeps_future_global_variable_metadata_top() {
    let source = "@if global-variable-exists(\"theme\") { .on { color: green; } } $theme: dark;";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[0].transfer_value_kind, Some("top"));
    assert_eq!(report.blocks[0].transfer_truthiness, None);
}

#[test]
fn control_flow_value_analysis_does_not_treat_local_binding_as_global_metadata() {
    let source =
        ".scope { $theme: dark; @if global-variable-exists(\"theme\") { .on { color: green; } } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[0].transfer_truthiness, Some("falsey"));
}

#[test]
fn control_flow_value_analysis_reports_sass_equality_branch_truthiness() {
    let source = "$enabled: false; @if $enabled == false { .off { color: red; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[0].transfer_truthiness, Some("truthy"));
}

#[test]
fn control_flow_value_analysis_reports_sass_inequality_branch_truthiness() {
    let source = "$enabled: false; @if $enabled != true { .off { color: red; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[0].transfer_truthiness, Some("truthy"));
}

#[test]
fn control_flow_value_analysis_reports_sass_numeric_ordering_branch_truthiness() {
    let source = "$gap: 3px; @if $gap > 2px { .on { color: green; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[0].transfer_truthiness, Some("truthy"));
}

#[test]
fn control_flow_value_analysis_reports_sass_zero_numeric_ordering_branch_truthiness() {
    let source = "$gap: 0px; @if $gap >= 0 { .on { color: green; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[0].transfer_truthiness, Some("truthy"));
}

#[test]
fn control_flow_value_analysis_reduces_static_if_header_values() {
    let source = "$enabled: if(false, false, true); @if $enabled { .on { color: green; } } @else { .off { color: red; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 2);
    assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[0].transfer_truthiness, Some("truthy"));
    assert_eq!(report.blocks[1].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[1].transfer_truthiness, Some("falsey"));
}

#[test]
fn control_flow_value_analysis_respects_declaration_order_for_branch_headers() {
    let source = "@if $enabled { .on { color: green; } } $enabled: true;";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[0].transfer_value_kind, Some("top"));
    assert_eq!(report.blocks[0].transfer_truthiness, None);
}

#[test]
fn control_flow_value_analysis_does_not_leak_sibling_block_bindings() {
    let source = "@if true { $enabled: true; } @if $enabled { .on { color: green; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 2);
    assert_eq!(report.blocks[1].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[1].transfer_value_kind, Some("top"));
    assert_eq!(report.blocks[1].transfer_truthiness, None);
}

#[test]
fn control_flow_value_analysis_marks_sibling_block_reassignment_top() {
    let source =
        "$enabled: false; @if true { $enabled: true; } @if $enabled { .on { color: green; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 2);
    assert_eq!(report.blocks[1].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[1].transfer_value_kind, Some("top"));
    assert_eq!(report.blocks[1].transfer_truthiness, None);
}

#[test]
fn control_flow_value_analysis_uses_enclosing_scope_bindings() {
    let source = "$enabled: true; .scope { @if $enabled { .on { color: green; } } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[0].transfer_value_kind, Some("raw"));
    assert_eq!(report.blocks[0].transfer_truthiness, Some("truthy"));
}

#[test]
fn control_flow_value_analysis_reports_sass_else_branch_truthiness() {
    let source =
        "$enabled: false; @if $enabled { .on { color: green; } } @else { .off { color: red; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 2);
    assert_eq!(report.blocks[0].kind, "branchIf");
    assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[0].transfer_truthiness, Some("falsey"));
    assert_eq!(report.blocks[1].kind, "branchElse");
    assert_eq!(report.blocks[1].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[1].transfer_truthiness, Some("truthy"));
}

#[test]
fn control_flow_value_analysis_reports_sass_else_if_branch_truthiness() {
    let source = "$enabled: false; @if $enabled { .on { color: green; } } @else if not $enabled { .off { color: red; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 2);
    assert_eq!(report.blocks[1].kind, "branchElse");
    assert_eq!(report.blocks[1].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[1].transfer_truthiness, Some("truthy"));
}

#[test]
fn control_flow_value_analysis_reports_sass_final_else_after_else_if_truthiness() {
    let source = "$enabled: false; $fallback: false; @if $enabled { .on { color: green; } } @else if $fallback { .fallback { color: yellow; } } @else { .off { color: red; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 3);
    assert_eq!(report.blocks[2].kind, "branchElse");
    assert_eq!(report.blocks[2].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[2].transfer_truthiness, Some("truthy"));
}

#[test]
fn control_flow_value_analysis_final_else_observes_full_else_if_chain() {
    let source = "$enabled: true; $fallback: false; @if $enabled { .on { color: green; } } @else if $fallback { .fallback { color: yellow; } } @else { .off { color: red; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 3);
    assert_eq!(report.blocks[2].kind, "branchElse");
    assert_eq!(report.blocks[2].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[2].transfer_truthiness, Some("falsey"));
}

#[test]
fn control_flow_value_analysis_reports_parenthesized_branch_truthiness() {
    let source = "$enabled: false; @if ($enabled) { .off { color: red; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
    assert_eq!(report.blocks[0].transfer_truthiness, Some("falsey"));
}

#[test]
fn control_flow_value_analysis_tracks_static_each_binding_values() {
    let source = "@each $tone in red, blue { .item { color: $tone; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.blocks[0].loop_carried_bindings, vec!["$tone"]);
    assert_eq!(report.blocks[0].loop_carried_binding_values.len(), 1);
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[0].value_kind,
        "finiteSet"
    );
    assert_eq!(report.blocks[0].output_value_kind, "finiteSet");
}

#[test]
fn control_flow_value_analysis_tracks_static_each_function_source_values() {
    let source = "@each $item in list.append(1px 2px, 3px) { .item { margin: $item; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.blocks[0].loop_carried_bindings, vec!["$item"]);
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[0].value,
        AbstractCssValueV0::FiniteSet {
            values: vec!["1px".to_string(), "2px".to_string(), "3px".to_string()]
        }
    );
    assert_eq!(report.blocks[0].output_value_kind, "finiteSet");
}

#[test]
fn control_flow_value_analysis_tracks_static_each_map_pair_values() {
    let source =
        "@each $name, $color in (primary: red, secondary: blue) { .#{$name} { color: $color; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(
        report.blocks[0].loop_carried_bindings,
        vec!["$name", "$color"]
    );
    assert_eq!(report.blocks[0].loop_carried_binding_values.len(), 2);
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[0].name,
        "$name"
    );
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[0].value,
        AbstractCssValueV0::FiniteSet {
            values: vec!["primary".to_string(), "secondary".to_string()]
        }
    );
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[1].name,
        "$color"
    );
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[1].value,
        AbstractCssValueV0::FiniteSet {
            values: vec!["#00f".to_string(), "red".to_string()]
        }
    );
}

#[test]
fn control_flow_value_analysis_tracks_static_each_map_variable_pair_values() {
    let source = "$tones: (primary: red, secondary: blue); @each $name, $color in $tones { .#{$name} { color: $color; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(
        report.blocks[0].loop_carried_bindings,
        vec!["$name", "$color"]
    );
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[0].value,
        AbstractCssValueV0::FiniteSet {
            values: vec!["primary".to_string(), "secondary".to_string()]
        }
    );
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[1].value,
        AbstractCssValueV0::FiniteSet {
            values: vec!["#00f".to_string(), "red".to_string()]
        }
    );
}

#[test]
fn control_flow_value_analysis_tracks_static_each_map_function_source_values() {
    let source = "@each $name, $color in map.merge((primary: red), (secondary: blue)) { .#{$name} { color: $color; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(
        report.blocks[0].loop_carried_bindings,
        vec!["$name", "$color"]
    );
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[0].value,
        AbstractCssValueV0::FiniteSet {
            values: vec!["primary".to_string(), "secondary".to_string()]
        }
    );
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[1].value,
        AbstractCssValueV0::FiniteSet {
            values: vec!["#00f".to_string(), "red".to_string()]
        }
    );
}

#[test]
fn control_flow_value_analysis_tracks_static_each_tuple_pair_values() {
    let source =
        "@each $icon, $size in (save, 16px), (cancel, 24px) { .#{$icon} { width: $size; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(
        report.blocks[0].loop_carried_bindings,
        vec!["$icon", "$size"]
    );
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[0].value,
        AbstractCssValueV0::FiniteSet {
            values: vec!["cancel".to_string(), "save".to_string()]
        }
    );
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[1].value,
        AbstractCssValueV0::FiniteSet {
            values: vec!["16px".to_string(), "24px".to_string()]
        }
    );
    assert_eq!(report.blocks[0].output_value_kind, "finiteSet");
}

#[test]
fn control_flow_value_analysis_tracks_static_each_tuple_function_source_values() {
    let source = "@each $width, $style in list.zip(1px 2px, solid dashed) { .item { border: $width $style; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(
        report.blocks[0].loop_carried_bindings,
        vec!["$width", "$style"]
    );
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[0].value,
        AbstractCssValueV0::FiniteSet {
            values: vec!["1px".to_string(), "2px".to_string()]
        }
    );
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[1].value,
        AbstractCssValueV0::FiniteSet {
            values: vec!["dashed".to_string(), "solid".to_string()]
        }
    );
}

#[test]
fn control_flow_value_analysis_tracks_static_each_tuple_variable_pair_values() {
    let source = "$pairs: (save, 16px), (cancel, 24px); @each $icon, $size in $pairs { .#{$icon} { width: $size; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(
        report.blocks[0].loop_carried_bindings,
        vec!["$icon", "$size"]
    );
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[0].value,
        AbstractCssValueV0::FiniteSet {
            values: vec!["cancel".to_string(), "save".to_string()]
        }
    );
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[1].value,
        AbstractCssValueV0::FiniteSet {
            values: vec!["16px".to_string(), "24px".to_string()]
        }
    );
}

#[test]
fn control_flow_value_analysis_tracks_static_each_space_tuple_pair_values() {
    let source = "@each $icon, $size in save 16px, cancel 24px { .#{$icon} { width: $size; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(
        report.blocks[0].loop_carried_bindings,
        vec!["$icon", "$size"]
    );
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[0].value,
        AbstractCssValueV0::FiniteSet {
            values: vec!["cancel".to_string(), "save".to_string()]
        }
    );
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[1].value,
        AbstractCssValueV0::FiniteSet {
            values: vec!["16px".to_string(), "24px".to_string()]
        }
    );
}

#[test]
fn control_flow_value_analysis_models_for_to_as_end_exclusive() {
    let source = "@for $i from 1 to 3 { .n { order: $i; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.blocks[0].loop_carried_bindings, vec!["$i"]);
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[0].value,
        AbstractCssValueV0::FiniteSet {
            values: vec!["1".to_string(), "2".to_string()]
        }
    );
}

#[test]
fn control_flow_value_analysis_resolves_static_for_loop_bounds() {
    let source = "$start: 1; $end: 3; @for $i from $start through $end { .n { order: $i; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.blocks[0].loop_carried_bindings, vec!["$i"]);
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[0].value,
        AbstractCssValueV0::FiniteSet {
            values: vec!["1".to_string(), "2".to_string(), "3".to_string()]
        }
    );
}

#[test]
fn control_flow_value_analysis_resolves_static_for_loop_expression_bounds() {
    let source = "@for $i from 1 + 1 through 1 + 2 { .n { order: $i; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.blocks[0].loop_carried_bindings, vec!["$i"]);
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[0].value,
        AbstractCssValueV0::FiniteSet {
            values: vec!["2".to_string(), "3".to_string()]
        }
    );
}

#[test]
fn control_flow_value_analysis_respects_declaration_order_for_loop_bounds() {
    let source = "@for $i from $start through $end { .n { order: $i; } } $start: 1; $end: 3;";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.blocks[0].transfer_kind, "loopCarriedBindings");
    assert_eq!(report.blocks[0].transfer_value_kind, Some("top"));
    assert_eq!(report.blocks[0].loop_carried_bindings, vec!["$i"]);
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[0].value_kind,
        "top"
    );
}

#[test]
fn control_flow_value_analysis_resolves_hyphen_underscore_equivalent_loop_bounds() {
    let source = "$start_value: 1; $end_value: 3; @for $i from $start-value through $end-value { .n { order: $i; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.blocks[0].loop_carried_bindings, vec!["$i"]);
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[0].value,
        AbstractCssValueV0::FiniteSet {
            values: vec!["1".to_string(), "2".to_string(), "3".to_string()]
        }
    );
}

#[test]
fn control_flow_value_analysis_tracks_while_condition_loop_bindings() {
    let source = "$i: 0; @while $i < 3 { $i: $i + 1; .n { order: $i; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.back_edge_count, 1);
    assert_eq!(report.loop_carried_binding_count, 1);
    assert_eq!(report.blocks[0].kind, "loop");
    assert_eq!(report.blocks[0].transfer_kind, "loopCondition");
    assert_eq!(report.blocks[0].transfer_truthiness, None);
    assert_eq!(report.blocks[0].loop_carried_bindings, vec!["$i"]);
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[0].value,
        AbstractCssValueV0::FiniteSet {
            values: vec!["0".to_string(), "1".to_string(), "2".to_string()]
        }
    );
}

#[test]
fn control_flow_value_analysis_tracks_reversed_while_condition_loop_bindings() {
    let source = "$i: 3; @while 0 < $i { $i: $i - 1; .n { order: $i; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.back_edge_count, 1);
    assert_eq!(report.loop_carried_binding_count, 1);
    assert_eq!(report.blocks[0].kind, "loop");
    assert_eq!(report.blocks[0].transfer_kind, "loopCondition");
    assert_eq!(report.blocks[0].transfer_truthiness, None);
    assert_eq!(report.blocks[0].loop_carried_bindings, vec!["$i"]);
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[0].value,
        AbstractCssValueV0::FiniteSet {
            values: vec!["1".to_string(), "2".to_string(), "3".to_string()]
        }
    );
}

#[test]
fn control_flow_value_analysis_tracks_while_bound_variable_bindings() {
    let source = "$end: 3; $i: 0; @while $i < $end { $i: $i + 1; .n { order: $i; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.back_edge_count, 1);
    assert_eq!(report.loop_carried_binding_count, 1);
    assert_eq!(report.blocks[0].kind, "loop");
    assert_eq!(report.blocks[0].transfer_kind, "loopCondition");
    assert_eq!(report.blocks[0].loop_carried_bindings, vec!["$i"]);
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[0].value,
        AbstractCssValueV0::FiniteSet {
            values: vec!["0".to_string(), "1".to_string(), "2".to_string()]
        }
    );
}

#[test]
fn control_flow_value_analysis_tracks_static_while_assignment_steps() {
    let source = "$i: 0; @while $i < 6 { $i: $i + 2; .n { order: $i; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.back_edge_count, 1);
    assert_eq!(report.loop_carried_binding_count, 1);
    assert_eq!(report.blocks[0].loop_carried_bindings, vec!["$i"]);
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[0].value,
        AbstractCssValueV0::FiniteSet {
            values: vec!["0".to_string(), "2".to_string(), "4".to_string()]
        }
    );
}

#[test]
fn control_flow_value_analysis_tracks_static_while_expression_steps() {
    let source = "$step: 1 + 1; $i: 0; @while $i < 6 { $i: $i + $step; .n { order: $i; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.back_edge_count, 1);
    assert_eq!(report.loop_carried_binding_count, 1);
    assert_eq!(report.blocks[0].loop_carried_bindings, vec!["$i"]);
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[0].value,
        AbstractCssValueV0::FiniteSet {
            values: vec!["0".to_string(), "2".to_string(), "4".to_string()]
        }
    );
}

#[test]
fn control_flow_value_analysis_tracks_static_while_compound_expression_steps() {
    let source = "$step: 2; $i: 0; @while $i < 7 { $i += $step + 1; .n { order: $i; } }";
    let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.block_count, 1);
    assert_eq!(report.back_edge_count, 1);
    assert_eq!(report.loop_carried_binding_count, 1);
    assert_eq!(report.blocks[0].loop_carried_bindings, vec!["$i"]);
    assert_eq!(
        report.blocks[0].loop_carried_binding_values[0].value,
        AbstractCssValueV0::FiniteSet {
            values: vec!["0".to_string(), "3".to_string(), "6".to_string()]
        }
    );
}

#[test]
fn call_return_ir_summarizes_mixin_and_function_edges() {
    let source = r#"
@mixin tone($color) { color: $color; }
@function double($n) { @return $n * 2; }
.a { @include tone(red); width: double(2px); }
"#;
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.mode, "oracleOnly");
    assert_eq!(report.node_key_type, "StableNodeKeyV0");
    assert_eq!(report.recursion_cap, SCSS_CALL_RETURN_RECURSION_LIMIT);
    assert!(!report.flat_css_cfg_built);
    assert!(!report.merged_cross_file_graph);
    assert_eq!(report.declaration_node_count, 2);
    assert_eq!(report.call_node_count, 2);
    assert_eq!(report.return_node_count, 1);
    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.call_argument_value_count, 2);
    assert_eq!(report.exact_call_argument_value_count, 2);
    assert_eq!(report.raw_call_argument_value_count, 0);
    assert!(
        report
            .edges
            .iter()
            .any(|edge| edge.kind == "mixinCall" && !edge.recursive)
    );
    assert!(
        report
            .edges
            .iter()
            .any(|edge| edge.kind == "functionCall" && !edge.recursive)
    );
    assert!(
        report
            .edges
            .iter()
            .any(|edge| edge.kind == "functionReturn")
    );
    assert_eq!(report.recursive_edge_count, 0);
    assert!(
        report
            .nodes
            .iter()
            .all(|node| node.node_key.as_str().contains('@'))
    );
}

#[test]
fn call_return_ir_reports_function_return_values_in_abstract_domain() {
    let source = "@function gap($value) { @return calc(1px + 2px); } .a { width: gap(2px); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };

    assert_eq!(return_node.return_text.as_deref(), Some("calc(1px + 2px)"));
    let function_call = report.nodes.iter().find(|node| node.kind == "functionCall");
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(function_call.argument_values.len(), 1);
    assert_eq!(function_call.argument_values[0].text, "2px");
    assert_eq!(function_call.argument_values[0].value_kind, "exact");
    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.call_argument_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(report.exact_call_argument_value_count, 1);
    assert_eq!(report.raw_return_value_count, 0);
    assert_eq!(report.top_return_value_count, 0);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "3px".to_string()
        })
    );
}

#[test]
fn call_return_ir_resolves_function_call_returns_with_arguments() {
    let source = "@function double($value) { @return $value * 2; } .a { width: double(2px); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let declaration = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionDeclaration");
    assert!(declaration.is_some());
    let Some(declaration) = declaration else {
        return;
    };
    assert_eq!(declaration.parameter_names, vec!["$value"]);

    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("double"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.exact_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "4px".to_string()
        })
    );
}

#[test]
fn call_return_ir_resolves_function_call_returns_through_static_if() {
    let source =
        "@function tone($enabled) { @return if($enabled, red, blue); } .a { color: tone(false); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("tone"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.exact_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "#00f".to_string()
        })
    );
}

#[test]
fn call_return_ir_resolves_call_bound_local_variable_returns() {
    let source = "@function offset($base) { $next: $base + 1px; @return $next + 1px; } .a { width: offset(2px); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let declaration = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionDeclaration" && node.name.as_deref() == Some("offset"));
    assert!(declaration.is_some());
    let Some(declaration) = declaration else {
        return;
    };
    assert_eq!(declaration.local_binding_values.len(), 1);
    assert_eq!(declaration.local_binding_values[0].name, "$next");
    assert_eq!(
        declaration.local_binding_values[0].value_text,
        "$base + 1px"
    );

    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("offset"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.exact_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "4px".to_string()
        })
    );
}

#[test]
fn call_return_ir_resolves_hyphen_underscore_equivalent_local_bindings() {
    let source = "@function offset($base) { $next_value: $base + 1px; @return $next-value + 1px; } .a { width: offset(2px); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("offset"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "4px".to_string()
        })
    );
}

#[test]
fn call_return_ir_resolves_call_bound_local_variable_chains() {
    let source = "@function scale($base) { $next: $base + 1px; $double: $next * 2; @return $double; } .a { width: scale(2px); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let declaration = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionDeclaration" && node.name.as_deref() == Some("scale"));
    assert!(declaration.is_some());
    let Some(declaration) = declaration else {
        return;
    };
    assert_eq!(declaration.local_binding_values.len(), 2);
    assert_eq!(declaration.local_binding_values[0].name, "$next");
    assert_eq!(declaration.local_binding_values[1].name, "$double");

    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("scale"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.exact_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "6px".to_string()
        })
    );
}

#[test]
fn call_return_ir_resolves_sass_indented_function_returns() {
    let source = "@function pick($target)\n  @for $i from 1 through 3\n    @if $i == $target\n      @return $i\n  @return 0\n.button\n  z-index: pick(2)";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Sass);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let declaration = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionDeclaration" && node.name.as_deref() == Some("pick"));
    assert!(declaration.is_some());
    let Some(declaration) = declaration else {
        return;
    };
    assert!(declaration.body_has_control_flow);
    assert!(declaration.body_has_loop_control_flow);

    let loop_return = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn" && node.return_text.as_deref() == Some("$i"));
    assert!(loop_return.is_some());
    let Some(loop_return) = loop_return else {
        return;
    };
    assert!(loop_return.return_inside_loop_control_flow);
    assert_eq!(
        loop_return.return_loop_header_text.as_deref(),
        Some("$i from 1 through 3")
    );
    assert_eq!(
        loop_return.return_condition_text.as_deref(),
        Some("$i == $target")
    );

    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("pick"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert!(function_call.containing_declaration_node_key.is_none());
    assert_eq!(report.recursive_edge_count, 0);
    assert_eq!(report.capped_recursive_call_count, 0);
    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.exact_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "2".to_string()
        })
    );
}

#[test]
fn call_return_ir_resolves_local_bindings_after_prior_branch() {
    let source = "@function pick($enabled) { @if $enabled { @return 3px; } $after: 1px + 1px; @return $after; } .a { width: pick(false); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let declaration = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionDeclaration" && node.name.as_deref() == Some("pick"));
    assert!(declaration.is_some());
    let Some(declaration) = declaration else {
        return;
    };
    assert_eq!(declaration.local_binding_values.len(), 1);
    assert_eq!(declaration.local_binding_values[0].name, "$after");

    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("pick"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "2px".to_string()
        })
    );
}

#[test]
fn call_return_ir_resolves_branch_local_bindings() {
    let source = "@function pick($enabled) { @if $enabled { $inside: 1px + 1px; @return $inside; } @return 1px; } .a { width: pick(true); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let declaration = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionDeclaration" && node.name.as_deref() == Some("pick"));
    assert!(declaration.is_some());
    let Some(declaration) = declaration else {
        return;
    };
    assert_eq!(declaration.local_binding_values.len(), 1);
    assert_eq!(declaration.local_binding_values[0].name, "$inside");

    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("pick"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "2px".to_string()
        })
    );
}

#[test]
fn call_return_ir_does_not_leak_sibling_branch_local_bindings() {
    let source = "@function pick($enabled) { @if $enabled { @return $other; } @else { $other: 1px; @return $other; } } .a { width: pick(true); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let declaration = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionDeclaration" && node.name.as_deref() == Some("pick"));
    assert!(declaration.is_some());
    let Some(declaration) = declaration else {
        return;
    };
    assert_eq!(declaration.local_binding_values.len(), 1);
    assert_eq!(declaration.local_binding_values[0].name, "$other");

    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("pick"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(function_call.call_resolved_return_value_kind, Some("top"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Top)
    );
}

#[test]
fn call_return_ir_keeps_future_local_bindings_out_of_active_return() {
    let source = "@function pick($enabled) { @if $enabled { @return $after; } $after: 1px; @return $after; } .a { width: pick(true); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let declaration = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionDeclaration" && node.name.as_deref() == Some("pick"));
    assert!(declaration.is_some());
    let Some(declaration) = declaration else {
        return;
    };
    assert_eq!(declaration.local_binding_values.len(), 1);
    assert_eq!(declaration.local_binding_values[0].name, "$after");

    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("pick"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(function_call.call_resolved_return_value_kind, Some("top"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Top)
    );
}

#[test]
fn call_return_ir_resolves_named_function_arguments() {
    let source = "@function pair($left, $right) { @return $left + $right; } .a { width: pair($right: 2px, $left: 1px); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("pair"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(function_call.argument_values.len(), 2);
    assert_eq!(
        function_call.argument_values[0].name.as_deref(),
        Some("$right")
    );
    assert_eq!(function_call.argument_values[0].text, "2px");
    assert_eq!(
        function_call.argument_values[1].name.as_deref(),
        Some("$left")
    );
    assert_eq!(function_call.argument_values[1].text, "1px");
    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.exact_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "3px".to_string()
        })
    );
}

#[test]
fn call_return_ir_resolves_hyphen_underscore_equivalent_parameter_references() {
    let source =
        "@function gap($base_value) { @return $base-value + 1px; } .a { width: gap(2px); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("gap"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "3px".to_string()
        })
    );
}

#[test]
fn call_return_ir_resolves_hyphen_underscore_equivalent_named_arguments() {
    let source = "@function gap($base_value) { @return $base-value + 1px; } .a { width: gap($base-value: 2px); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("gap"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "3px".to_string()
        })
    );
}

#[test]
fn call_return_ir_resolves_default_function_arguments() {
    let source = "@function offset($value: 1px, $extra: 2px) { @return $value + $extra; } .a { width: offset(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let declaration = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionDeclaration" && node.name.as_deref() == Some("offset"));
    assert!(declaration.is_some());
    let Some(declaration) = declaration else {
        return;
    };
    assert_eq!(declaration.parameter_values.len(), 2);
    assert_eq!(
        declaration.parameter_values[0]
            .default_value_text
            .as_deref(),
        Some("1px")
    );
    assert_eq!(
        declaration.parameter_values[1]
            .default_value_text
            .as_deref(),
        Some("2px")
    );

    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("offset"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.exact_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "3px".to_string()
        })
    );
}

#[test]
fn call_return_ir_resolves_default_arguments_from_prior_parameters() {
    let source = "@function offset($value, $extra: $value + 1px) { @return $extra; } .a { width: offset(2px); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("offset"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.exact_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "3px".to_string()
        })
    );
}

#[test]
fn call_return_ir_resolves_composed_same_file_function_calls() {
    let source = "@function inc($value) { @return $value + 1px; } @function gap($value) { @return inc($value) + 1px; } .a { width: gap(2px); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report.nodes.iter().find(|node| {
        node.kind == "functionCall"
            && node.name.as_deref() == Some("gap")
            && node.containing_declaration_node_key.is_none()
    });
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "4px".to_string()
        })
    );
}

#[test]
fn call_return_ir_resolves_hyphen_underscore_equivalent_function_calls() {
    let source = "@function inc_value($value) { @return $value + 1px; } @function gap_value($value) { @return inc-value($value) + 1px; } .a { width: gap-value(2px); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report.nodes.iter().find(|node| {
        node.kind == "functionCall"
            && node.name.as_deref() == Some("gap-value")
            && node.containing_declaration_node_key.is_none()
    });
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "4px".to_string()
        })
    );
}

#[test]
fn call_return_ir_resolves_local_values_with_same_file_function_calls() {
    let source = "@function inc($value) { @return $value + 1px; } @function gap($value) { $next: inc($value); @return $next + 1px; } .a { width: gap(2px); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report.nodes.iter().find(|node| {
        node.kind == "functionCall"
            && node.name.as_deref() == Some("gap")
            && node.containing_declaration_node_key.is_none()
    });
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "4px".to_string()
        })
    );
}

#[test]
fn call_return_ir_keeps_indirect_recursive_function_calls_top() {
    let source = "@function a($value) { @return b($value); } @function b($value) { @return a($value); } .x { width: a(1px); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report.nodes.iter().find(|node| {
        node.kind == "functionCall"
            && node.name.as_deref() == Some("a")
            && node.containing_declaration_node_key.is_none()
    });
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(function_call.call_resolved_return_value_kind, Some("top"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Top)
    );
}

#[test]
fn call_return_ir_caps_hyphen_underscore_recursive_function_calls() {
    let source = "@function again_value($value) { @return again-value($value); } .a { width: again-value(1px); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report.nodes.iter().find(|node| {
        node.kind == "functionCall"
            && node.name.as_deref() == Some("again-value")
            && node.containing_declaration_node_key.is_none()
    });
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(function_call.call_resolved_return_value_kind, Some("top"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Top)
    );
}

#[test]
fn call_return_ir_resolves_same_file_function_call_arguments() {
    let source = "@function inc($value) { @return $value + 1px; } @function gap($value) { @return $value + 1px; } .a { width: gap(inc(2px)); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report.nodes.iter().find(|node| {
        node.kind == "functionCall"
            && node.name.as_deref() == Some("gap")
            && node.containing_declaration_node_key.is_none()
    });
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "4px".to_string()
        })
    );
}

#[test]
fn call_return_ir_resolves_named_same_file_function_call_arguments() {
    let source = "@function inc($value) { @return $value + 1px; } @function pair($left, $right) { @return $left + $right; } .a { width: pair($right: inc(2px), $left: 1px); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report.nodes.iter().find(|node| {
        node.kind == "functionCall"
            && node.name.as_deref() == Some("pair")
            && node.containing_declaration_node_key.is_none()
    });
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "4px".to_string()
        })
    );
}

#[test]
fn call_return_ir_keeps_positional_after_named_arguments_top() {
    let source = "@function pair($left, $right) { @return $left + $right; } .a { width: pair($left: 1px, 2px); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("pair"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.top_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("top"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Top)
    );
}

#[test]
fn call_return_ir_keeps_malformed_named_argument_top() {
    let source = "@function gap($value) { @return $value; } .a { width: gap(value: 1px); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("gap"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.top_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("top"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Top)
    );
}

#[test]
fn call_return_ir_uses_local_variables_in_branch_conditions() {
    let source = "@function tone($enabled) { $flag: $enabled; @if $flag { @return red; } @else { @return blue; } } .a { color: tone(false); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("tone"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.exact_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "#00f".to_string()
        })
    );
}

#[test]
fn call_return_ir_keeps_dynamic_local_variable_branches_top() {
    let source = "@function tone() { $flag: var(--enabled); @if $flag { @return red; } @else { @return blue; } } .a { color: tone(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("tone"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.top_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("top"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Top)
    );
}

#[test]
fn call_return_ir_resolves_call_bound_if_branch_returns() {
    let source = "@function tone($enabled) { @if $enabled { @return red; } @else { @return blue; } } .a { color: tone(true); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("tone"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.exact_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert!(report.nodes.iter().any(|node| {
        node.kind == "functionReturn" && node.return_condition_text.as_deref() == Some("$enabled")
    }));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "red".to_string()
        })
    );
}

#[test]
fn call_return_ir_respects_first_active_return_before_fallback() {
    let source = "@function tone($enabled) { @if $enabled { @return red; } @return blue; } .a { color: tone(true); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("tone"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.exact_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "red".to_string()
        })
    );
}

#[test]
fn call_return_ir_resolves_call_bound_else_branch_returns() {
    let source = "@function tone($enabled) { @if $enabled { @return red; } @else { @return blue; } } .a { color: tone(false); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("tone"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.exact_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert!(report.nodes.iter().any(|node| {
        node.kind == "functionReturn"
            && node.return_text.as_deref() == Some("blue")
            && node.return_condition_text.is_none()
            && node.return_negated_condition_texts == vec!["$enabled".to_string()]
    }));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "#00f".to_string()
        })
    );
}

#[test]
fn call_return_ir_resolves_call_bound_else_if_branch_returns() {
    let source = "@function tone($first, $second) { @if $first { @return red; } @else if $second { @return green; } @else { @return blue; } } .a { color: tone(false, true); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("tone"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.exact_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "green".to_string()
        })
    );
}

#[test]
fn call_return_ir_keeps_dynamic_branch_returns_top() {
    let source = "@function tone($enabled) { @if $enabled { @return red; } @else { @return blue; } } .a { color: tone(var(--enabled)); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("tone"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.top_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("top"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Top)
    );
}

#[test]
fn call_return_ir_resolves_static_for_loop_body_returns() {
    let source = "@function collect($count) { @for $i from 1 through $count { @return $i; } } .a { width: collect(3); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("collect"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.finite_set_call_resolved_return_value_count, 1);
    assert_eq!(
        function_call.call_resolved_return_value_kind,
        Some("finiteSet")
    );
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::FiniteSet {
            values: vec!["1".to_string(), "2".to_string(), "3".to_string()]
        })
    );
}

#[test]
fn call_return_ir_resolves_static_each_loop_body_returns() {
    let source =
        "@function tones() { @each $tone in red, blue { @return $tone; } } .a { color: tones(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("tones"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.finite_set_call_resolved_return_value_count, 1);
    assert_eq!(
        function_call.call_resolved_return_value_kind,
        Some("finiteSet")
    );
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::FiniteSet {
            values: vec!["#00f".to_string(), "red".to_string()]
        })
    );
}

#[test]
fn call_return_ir_resolves_static_each_function_source_returns() {
    let source = "@function pick($target) { @each $item in list.append(1px 2px, 3px) { @if $item == $target { @return $item; } } @return 0px; } .a { margin: pick(3px); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("pick"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.exact_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "3px".to_string()
        })
    );
}

#[test]
fn call_return_ir_resolves_static_each_tuple_function_source_returns() {
    let source = "@function width-for($target) { @each $width, $style in list.zip(1px 2px, solid dashed) { @if $style == $target { @return $width; } } @return 0px; } .a { margin: width-for(dashed); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("width-for"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.exact_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "2px".to_string()
        })
    );
}

#[test]
fn call_return_ir_resolves_static_while_loop_body_returns() {
    let source = "@function collect() { $i: 0; @while $i < 3 { @return $i; $i: $i + 1; } } .a { width: collect(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("collect"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.finite_set_call_resolved_return_value_count, 1);
    assert_eq!(
        function_call.call_resolved_return_value_kind,
        Some("finiteSet")
    );
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::FiniteSet {
            values: vec!["0".to_string(), "1".to_string(), "2".to_string()]
        })
    );
}

#[test]
fn call_return_ir_filters_static_while_conditional_returns() {
    let source = "@function collect() { $i: 0; @while $i < 3 { @if $i == 2 { @return $i; } $i: $i + 1; } @return 0; } .a { width: collect(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("collect"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.exact_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "2".to_string()
        })
    );
}

#[test]
fn call_return_ir_uses_call_arguments_in_static_while_conditional_returns() {
    let source = "@function collect($target) { $i: 0; @while $i < 3 { @if $i == $target { @return $i + 1; } $i: $i + 1; } @return 0; } .a { width: collect(2); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("collect"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.exact_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "3".to_string()
        })
    );
}

#[test]
fn call_return_ir_filters_static_while_step_conditional_returns() {
    let source = "@function collect() { $i: 0; @while $i < 6 { @if $i == 3 { @return $i; } $i: $i + 2; } @return 9; } .a { width: collect(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("collect"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.exact_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "9".to_string()
        })
    );
}

#[test]
fn call_return_ir_resolves_static_while_cumulative_step_returns() {
    let source = "@function collect() { $i: 0; @while $i < 7 { @if $i == 3 { @return $i + 1; } $i: $i + 1; $i: $i + 2; } @return 9; } .a { width: collect(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("collect"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.exact_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "4".to_string()
        })
    );
}

#[test]
fn call_return_ir_resolves_static_while_expression_step_returns() {
    let source = "@function collect() { $step: 1 + 1; $i: 0; @while $i < 6 { @if $i == 4 { @return $i; } $i: $i + $step; } @return 9; } .a { width: collect(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("collect"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.exact_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "4".to_string()
        })
    );
}

#[test]
fn call_return_ir_keeps_conditional_while_assignment_top() {
    let source = "@function collect() { $i: 0; @while $i < 6 { @if true { $i: $i + 2; } @if $i == 3 { @return $i; } } @return 9; } .a { width: collect(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("collect"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.top_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("top"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Top)
    );
}

#[test]
fn call_return_ir_resolves_static_while_inequality_operator_returns() {
    let source = "@function collect() { $i: 0; @while $i != 3 { @if $i == 2 { @return $i + 1; } $i: $i + 1; } @return 9; } .a { width: collect(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("collect"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.exact_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "3".to_string()
        })
    );
}

#[test]
fn call_return_ir_filters_static_for_loop_conditional_returns() {
    let source = "@function collect($target) { @for $i from 1 through 3 { @if $i == $target { @return $i; } } @return 0; } .a { width: collect(2); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("collect"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.exact_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "2".to_string()
        })
    );
}

#[test]
fn call_return_ir_filters_static_for_loop_expression_bound_returns() {
    let source = "@function collect($target) { @for $i from 1 + 1 through 1 + 2 { @if $i == $target { @return $i; } } @return 0; } .a { width: collect(1); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("collect"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.exact_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "0".to_string()
        })
    );
}

#[test]
fn call_return_ir_continues_after_inactive_static_loop_returns() {
    let source = "@function collect($target) { @for $i from 1 through 3 { @if $i == $target { @return $i; } } @return 0; } .a { width: collect(4); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("collect"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.exact_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "0".to_string()
        })
    );
}

#[test]
fn call_return_ir_resolves_nested_static_loop_body_returns() {
    let source = "@function collect($target) { @for $i from 1 through 2 { @for $j from 1 through 2 { @if $i == $target { @return $i + $j; } } } @return 0; } .a { width: collect(2); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("collect"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert!(
        report.nodes.iter().any(|node| {
            node.kind == "functionReturn" && node.return_loop_header_texts.len() == 2
        })
    );
    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.finite_set_call_resolved_return_value_count, 1);
    assert_eq!(
        function_call.call_resolved_return_value_kind,
        Some("finiteSet")
    );
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::FiniteSet {
            values: vec!["3".to_string(), "4".to_string()]
        })
    );
}

#[test]
fn call_return_ir_continues_after_inactive_nested_static_loop_returns() {
    let source = "@function collect($target) { @for $i from 1 through 2 { @for $j from 1 through 2 { @if $i == $target { @return $i + $j; } } } @return 0; } .a { width: collect(3); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("collect"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.exact_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "0".to_string()
        })
    );
}

#[test]
fn call_return_ir_filters_static_each_map_conditional_returns() {
    let source = "@function tone($target) { @each $name, $tone in (primary: red, secondary: blue) { @if $name == $target { @return $tone; } } @return black; } .a { color: tone(secondary); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("tone"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.exact_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "#00f".to_string()
        })
    );
}

#[test]
fn call_return_ir_keeps_dynamic_loop_body_returns_top() {
    let source = "@function collect($count) { @for $i from 1 through $count { @return $i; } } .a { width: collect(var(--count)); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("collect"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.top_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("top"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Top)
    );
}

#[test]
fn call_return_ir_resolves_return_after_static_loop() {
    let source = "@function collect() { @for $i from 1 through 3 { $seen: $i; } @return 2px; } .a { width: collect(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("collect"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.call_resolved_return_value_count, 1);
    assert_eq!(report.exact_call_resolved_return_value_count, 1);
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "2px".to_string()
        })
    );
}

#[test]
fn call_return_ir_caps_recursive_function_call_return_values() {
    let source = "@function again($value) { @return again($value); } .a { width: again(1px); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report.nodes.iter().find(|node| {
        node.kind == "functionCall"
            && node.name.as_deref() == Some("again")
            && node.containing_declaration_node_key.is_none()
    });
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(function_call.call_resolved_return_value_kind, Some("top"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Top)
    );
}

#[test]
fn call_return_ir_reports_static_scss_if_return_values_in_abstract_domain() {
    let source = "@function gap() { @return if(false, 1px, 2px); } .a { width: gap(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };

    assert_eq!(
        return_node.return_text.as_deref(),
        Some("if(false, 1px, 2px)")
    );
    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(report.raw_return_value_count, 0);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "2px".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_static_scss_nth_return_values_in_abstract_domain() {
    let source = "@function gap() { @return list.nth((1px, 2px, 3px), 2); } .a { width: gap(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };

    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "2px".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_static_scss_map_get_return_values_in_abstract_domain() {
    let source = "@function gap() { @return map-get((default: 2px, dense: 1px), dense); } .a { margin: gap(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };

    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "1px".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_nested_static_scss_map_lookup_values() {
    let source = "@function font-weight() { @return if(map.has-key((font: (weights: (regular: 400, medium: 500))), font, weights, medium), map.get((font: (weights: (regular: 400, medium: 500))), font, weights, medium), 0); } .a { font-weight: font-weight(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };

    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "500".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_static_scss_collection_search_values_in_abstract_domain() {
    let source =
        "@function item() { @return list.index(red blue green, green); } .a { order: item(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };

    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "3".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_static_scss_list_metadata_values() {
    let source = "@function metadata() { @return if(list.separator((1px, 2px)) == \"comma\" and list.is-bracketed([1px]), 3px, 4px); } .a { margin: metadata(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };

    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "3px".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_static_scss_type_metadata_values() {
    let source = "@function metadata() { @return if(meta.type-of(1px) == number and type-of(red) == color and meta.type-of(color.mix(red, blue)) == color and meta.type-of(transparentize(red, .25)) == color and meta.type-of(hue(red)) == number and meta.type-of(color.channel(color.mix(red, blue), \"red\", $space: rgb)) == number and meta.type-of(red(red)) == number and meta.type-of(oklch(100% 0 0)) == color and meta.type-of((dense: true)) == map and feature-exists(\"at-error\") and meta.feature-exists(custom-property) and not meta.feature-exists(\"unknown\"), 3px, 4px); } .a { margin: metadata(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };

    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "3px".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_static_scss_calculation_metadata_values() {
    let source = "@function metadata() { @return if(meta.calc-name(clamp(1px, 2px, 3px)) == \"clamp\" and meta.type-of(calc(100% - 1px)) == calculation and list.nth(meta.calc-args(clamp(1px, 2px, 3px)), 2) == 2px and list.length(meta.calc-args(min(4px, 5px))) == 2, 3px, 4px); } .a { margin: metadata(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };

    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "3px".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_static_scss_function_metadata_values() {
    let source = "@function metadata() { @return if(meta.function-exists(\"scale-color\") and function-exists(\"hue\") and not function-exists(\"not-defined-here\"), 3px, 4px); } .a { margin: metadata(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("metadata"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "3px".to_string()
        })
    );
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "3px".to_string()
        })
    );
}

#[test]
fn call_return_ir_preserves_function_exists_declaration_order() {
    let source = "@function gate() { @return if(function-exists(\"later\"), 2px, 1px); } @function later() { @return 2px; } .a { margin: gate(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("gate"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "1px".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_static_scss_variable_metadata_values() {
    let source = "@function metadata($input) { $local: 1px; @return if(meta.variable-exists(\"input\") and variable-exists(\"local\") and not variable-exists(\"missing\"), 3px, 4px); } .a { margin: metadata(2px); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("metadata"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "3px".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_static_scss_global_variable_metadata_values() {
    let source = "$theme: dark; @function metadata() { @return if(global-variable-exists(\"theme\") and not meta.global-variable-exists(\"missing\"), 3px, 4px); } .a { margin: metadata(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("metadata"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "3px".to_string()
        })
    );
}

#[test]
fn call_return_ir_keeps_future_global_variable_metadata_unknown() {
    let source = "@function metadata() { @return if(global-variable-exists(\"theme\"), 3px, 4px); } $theme: dark; .a { margin: metadata(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("metadata"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(function_call.call_resolved_return_value_kind, Some("top"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Top)
    );
}

#[test]
fn call_return_ir_does_not_treat_local_binding_as_global_metadata() {
    let source = "@function metadata() { $theme: dark; @return if(global-variable-exists(\"theme\"), 3px, 4px); } .a { margin: metadata(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("metadata"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "4px".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_static_scss_mixin_metadata_values() {
    let source = "@mixin present { color: red; } @function metadata() { @return if(meta.mixin-exists(\"present\") and not mixin-exists(\"not-defined-here\"), 3px, 4px); } .a { margin: metadata(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("metadata"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "3px".to_string()
        })
    );
    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "3px".to_string()
        })
    );
}

#[test]
fn call_return_ir_preserves_mixin_exists_declaration_order() {
    let source = "@function gate() { @return if(mixin-exists(\"later\"), 2px, 1px); } @mixin later { color: red; } .a { margin: gate(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("gate"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
    assert_eq!(
        function_call.call_resolved_return_value,
        Some(AbstractCssValueV0::Exact {
            value: "1px".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_static_scss_string_metadata_values() {
    let source = "@function index() { @return if(string.index(\"Helvetica Neue\", \"Neue\") == 11, string.length(\"Helvetica Neue\"), 0); } .a { z-index: index(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };

    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "14".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_static_scss_map_has_key_conditions_in_abstract_domain() {
    let source = "@function gap() { @return if(map.has-key((default: 2px, dense: 1px), dense), list.length((1px, 2px)), 0); } .a { margin: gap(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };

    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "2".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_static_scss_map_key_and_value_lists() {
    let source = "@function map-value() { @return list.nth(map.values((default: 1px, dense: 2px)), list.length(map.keys((default: 1px, dense: 2px)))); } .a { margin: map-value(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };

    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "2px".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_static_scss_map_merge_values() {
    let source = "@function gap() { @return map.get(map.merge((default: 1px, dense: 2px), (dense: 3px, compact: 4px)), dense); } .a { margin: gap(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };

    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "3px".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_nested_static_scss_map_merge_values() {
    let source = "@function gap() { @return map.get(map.merge((theme: (spacing: (sm: 4px))), theme, spacing, (md: 8px)), theme, spacing, md); } .a { margin: gap(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };

    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "8px".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_static_scss_map_deep_merge_values() {
    let source = "@function gap() { @return map.get(map.deep-merge((theme: (spacing: (sm: 4px))), (theme: (spacing: (md: 8px)))), theme, spacing, md); } .a { margin: gap(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };

    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "8px".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_static_scss_map_remove_values() {
    let source = "@function count() { @return list.length(map.keys(map.remove((default: 1px, dense: 2px), dense))); } .a { z-index: count(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };

    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "1".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_nested_static_scss_map_deep_remove_values() {
    let source = "@function gap() { @return map.get(map.deep-remove((theme: (spacing: (sm: 4px, md: 8px))), theme, spacing, sm), theme, spacing, md); } .a { margin: gap(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };

    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "8px".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_static_scss_map_set_values() {
    let source = "@function weight() { @return map.get(map.set((regular: 400), bold, 700), bold); } .a { font-weight: weight(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };

    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "700".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_nested_static_scss_map_set_values() {
    let source = "@function tone() { @return map.get(map.set((theme: blue), theme, colors, primary, red), theme, colors, primary); } .a { color: tone(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };

    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "red".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_static_scss_math_return_values_in_abstract_domain() {
    let source = "@function gap() { @return math.div(6px, 3); } .a { margin: gap(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };

    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "2px".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_static_scss_math_alias_returns() {
    let source = "@function gap() { @return math.max(1px, math.abs(-3px)); } .a { margin: gap(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };

    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "3px".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_static_scss_math_constant_returns() {
    let source = "@function pi() { @return math.$pi; } .a { --pi: pi(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };

    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "3.141593".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_static_scss_math_constant_argument_returns() {
    let source = "@function enabled() { @return if(math.is-unitless(math.$pi), 1px, 2px); } .a { margin: enabled(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };

    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "1px".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_static_scss_extended_math_alias_returns() {
    let source = "@function gap() { @return math.hypot(3px, 4px); } .a { margin: gap(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };

    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "5px".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_static_scss_rounding_alias_returns() {
    let source = "@function gap() { @return math.round(1.5px); } .a { margin: gap(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };

    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "2px".to_string()
        })
    );
}

#[test]
fn call_return_ir_reduces_nested_static_list_conditions_in_order() {
    let source = "@function count() { @return list.length(if(false, 1px 2px, 3px 4px 5px)); } .a { z-index: count(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };

    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "3".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_static_scss_unitless_branch_returns() {
    let source = "@function gap() { @return if(unitless(2px), 1px, math.div(6px, 3)); } .a { margin: gap(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };

    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.exact_return_value_count, 1);
    assert_eq!(return_node.return_value_kind, Some("exact"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Exact {
            value: "2px".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_static_scss_unit_compatibility_returns() {
    let source = "@function unit-name() { @return if(math.compatible(1px, 2px) and not comparable(1, 1px), math.unit(4px), \"bad\"); } .a { content: unit-name(); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let return_node = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionReturn");
    assert!(return_node.is_some());
    let Some(return_node) = return_node else {
        return;
    };

    assert_eq!(report.return_value_count, 1);
    assert_eq!(report.raw_return_value_count, 1);
    assert_eq!(return_node.return_value_kind, Some("raw"));
    assert_eq!(
        return_node.return_value,
        Some(AbstractCssValueV0::Raw {
            value: "\"px\"".to_string()
        })
    );
}

#[test]
fn call_return_ir_reports_static_scss_if_argument_values_in_abstract_domain() {
    let source =
        "@function gap($value) { @return $value; } .a { width: gap(if(false, 1px, 2px)); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("gap"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(function_call.argument_values.len(), 1);
    assert_eq!(function_call.argument_values[0].text, "if(false, 1px, 2px)");
    assert_eq!(function_call.argument_values[0].value_kind, "exact");
    assert_eq!(
        function_call.argument_values[0].value,
        AbstractCssValueV0::Exact {
            value: "2px".to_string()
        }
    );
}

#[test]
fn call_return_ir_reports_static_scss_inequality_argument_values_in_abstract_domain() {
    let source =
        "@function gap($value) { @return $value; } .a { width: gap(if(1px != 2px, 1px, 2px)); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };
    let function_call = report
        .nodes
        .iter()
        .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("gap"));
    assert!(function_call.is_some());
    let Some(function_call) = function_call else {
        return;
    };

    assert_eq!(function_call.argument_values.len(), 1);
    assert_eq!(
        function_call.argument_values[0].text,
        "if(1px != 2px, 1px, 2px)"
    );
    assert_eq!(function_call.argument_values[0].value_kind, "exact");
    assert_eq!(
        function_call.argument_values[0].value,
        AbstractCssValueV0::Exact {
            value: "1px".to_string()
        }
    );
}

#[test]
fn call_return_ir_reports_recursion_cap_for_recursive_mixin() {
    let source = "@mixin again { @include again; }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.declaration_node_count, 1);
    assert_eq!(report.call_node_count, 1);
    assert_eq!(report.recursive_edge_count, 1);
    assert_eq!(report.capped_recursive_call_count, 1);
    assert_eq!(
        report.max_stack_depth_observed,
        SCSS_CALL_RETURN_RECURSION_LIMIT
    );
    assert!(report.edges.iter().any(|edge| edge.capped_by_recursion_cap));
}

#[test]
fn call_return_ir_resolves_hyphen_underscore_equivalent_mixin_edges() {
    let source = "@mixin tone_color($color) { color: $color; } .a { @include tone-color(red); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    assert_eq!(report.declaration_node_count, 1);
    assert_eq!(report.call_node_count, 1);
    assert!(report.edges.iter().any(|edge| {
        edge.kind == "mixinCall" && !edge.recursive && !edge.capped_by_recursion_cap
    }));
}

#[test]
fn call_return_ir_reports_mixin_default_arguments_from_prior_parameters() {
    let source = "@mixin tone($color, $border: $color) { color: $color; border-color: $border; } .a { @include tone(blue); }";
    let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
    assert!(report.is_some());
    let Some(report) = report else {
        return;
    };

    let declaration = report
        .nodes
        .iter()
        .find(|node| node.kind == "mixinDeclaration" && node.name.as_deref() == Some("tone"));
    assert!(declaration.is_some());
    let Some(declaration) = declaration else {
        return;
    };
    assert_eq!(declaration.parameter_values.len(), 2);
    assert_eq!(
        declaration.parameter_values[1]
            .default_value_text
            .as_deref(),
        Some("$color")
    );
    assert_eq!(report.declaration_node_count, 1);
    assert_eq!(report.call_node_count, 1);
    assert!(report.edges.iter().any(|edge| {
        edge.kind == "mixinCall" && !edge.recursive && !edge.capped_by_recursion_cap
    }));
}

#[test]
fn call_return_ir_does_not_build_flat_css_cfg() {
    assert!(summarize_scss_call_return_ir(".button { color: red; }", StyleDialect::Css).is_none());
}

#[test]
fn control_flow_value_analysis_does_not_build_flat_css_cfg() {
    assert!(
        analyze_scss_control_flow_values(".button { color: red; }", StyleDialect::Css).is_none()
    );
}
