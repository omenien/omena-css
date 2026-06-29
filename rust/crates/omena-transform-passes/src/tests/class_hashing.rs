use crate::{
    TransformClassNameRewriteV0, TransformExecutionContextV0,
    execute_transform_passes_on_source_with_dialect_and_context,
};
use omena_parser::StyleDialect;
use omena_transform_cst::TransformPassKind;

#[test]
fn execution_runtime_rewrites_css_module_class_names_with_identity_map() {
    let source = r#".button { composes: base utility global(reset); color: red; } .base, .utility { color: blue; } .button:hover { color: green; } .button :global(.external) { color: purple; } :global(.root) .button { color: orange; } :global(.standalone) { color: teal; } :global { .global-block { color: silver; } } :local(.button) { color: navy; } :local { .button { color: maroon; } } @media (min-width: 1px) { .button { color: black; } }"#;
    let context = TransformExecutionContextV0 {
        class_name_rewrites: vec![
            TransformClassNameRewriteV0 {
                original_name: "button".to_string(),
                rewritten_name: "_button_abc123".to_string(),
            },
            TransformClassNameRewriteV0 {
                original_name: "base".to_string(),
                rewritten_name: "_base_def456".to_string(),
            },
            TransformClassNameRewriteV0 {
                original_name: "utility".to_string(),
                rewritten_name: "_utility_ghi789".to_string(),
            },
            TransformClassNameRewriteV0 {
                original_name: "external".to_string(),
                rewritten_name: "_external_global".to_string(),
            },
            TransformClassNameRewriteV0 {
                original_name: "root".to_string(),
                rewritten_name: "_root_global".to_string(),
            },
            TransformClassNameRewriteV0 {
                original_name: "global-block".to_string(),
                rewritten_name: "_global_block_should_not_apply".to_string(),
            },
            TransformClassNameRewriteV0 {
                original_name: "reset".to_string(),
                rewritten_name: "_reset_should_not_apply".to_string(),
            },
        ],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::HashCssModuleClassNames,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 14);
    assert_eq!(
        execution.output_css,
        r#"._button_abc123{ composes: _base_def456 _utility_ghi789 global(reset); color: red; } ._base_def456, ._utility_ghi789{ color: blue; } ._button_abc123:hover{ color: green; } ._button_abc123 .external{ color: purple; } .root ._button_abc123{ color: orange; } .standalone{ color: teal; }  .global-block { color: silver; }  ._button_abc123{ color: navy; }  ._button_abc123{ color: maroon; }  @media (min-width: 1px) { ._button_abc123{ color: black; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["css-modules-class-hashing", "print-css"]
    );
    assert_eq!(
        execution
            .structural_ir_transaction_telemetry
            .source_range_rewrite_fallback_count,
        0
    );
    assert!(
        execution
            .structural_ir_transaction_telemetry
            .transaction_commit_count
            > 0
    );
}

#[test]
fn execution_runtime_preserves_global_composes_during_css_module_class_hashing() {
    let source = r#".button { composes: global(reset); color: red; }"#;
    let context = TransformExecutionContextV0 {
        class_name_rewrites: vec![
            TransformClassNameRewriteV0 {
                original_name: "button".to_string(),
                rewritten_name: "_button_x".to_string(),
            },
            TransformClassNameRewriteV0 {
                original_name: "reset".to_string(),
                rewritten_name: "_reset_should_not_apply".to_string(),
            },
        ],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::HashCssModuleClassNames,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#"._button_x{ composes: global(reset); color: red; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["css-modules-class-hashing", "print-css"]
    );
    let hash_node = execution
        .provenance_derivation_forest
        .nodes
        .iter()
        .find(|node| node.pass_id == "css-modules-class-hashing");
    assert!(
        hash_node.is_some(),
        "missing css-modules-class-hashing provenance node"
    );
    if let Some(hash_node) = hash_node {
        assert!(
            hash_node.mutation_spans.iter().any(|span| {
                span.node_key.as_ref().map(|key| key.as_str()) == Some("class-selector:button#0")
            }),
            "{:?}",
            hash_node.mutation_spans
        );
    }
}

#[test]
fn execution_runtime_does_not_hash_less_mixin_definitions() {
    let source = r#".space() when (isnumber($margin)) { padding: $margin; } .button { .space(); margin: 2px; }"#;
    let context = TransformExecutionContextV0 {
        class_name_rewrites: vec![
            TransformClassNameRewriteV0 {
                original_name: "space".to_string(),
                rewritten_name: "_space_should_not_apply".to_string(),
            },
            TransformClassNameRewriteV0 {
                original_name: "button".to_string(),
                rewritten_name: "_button_x".to_string(),
            },
        ],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Less,
        &[
            TransformPassKind::HashCssModuleClassNames,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(
        execution.output_css,
        r#".space() when (isnumber($margin)) { padding: $margin; } ._button_x{ .space(); margin: 2px; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["css-modules-class-hashing", "print-css"]
    );
}

#[test]
fn execution_runtime_rewrites_css_module_scope_prelude_class_names() {
    let source = r#"@scope (.card) to (:global(.footer)) { .title { color: red; } }"#;
    let context = TransformExecutionContextV0 {
        class_name_rewrites: vec![
            TransformClassNameRewriteV0 {
                original_name: "card".to_string(),
                rewritten_name: "_card_x".to_string(),
            },
            TransformClassNameRewriteV0 {
                original_name: "footer".to_string(),
                rewritten_name: "_footer_should_not_apply".to_string(),
            },
            TransformClassNameRewriteV0 {
                original_name: "title".to_string(),
                rewritten_name: "_title_z".to_string(),
            },
        ],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::HashCssModuleClassNames,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#"@scope (._card_x) to (.footer) { ._title_z{ color: red; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["css-modules-class-hashing", "print-css"]
    );
}

#[test]
fn execution_runtime_rewrites_css_module_supports_selector_class_names() {
    let source = r#"@supports selector(.card:has(:global(.footer), .title)) { .item { color: red; } } @supports (background: paint(.card)) { .paint { color: blue; } }"#;
    let context = TransformExecutionContextV0 {
        class_name_rewrites: vec![
            TransformClassNameRewriteV0 {
                original_name: "card".to_string(),
                rewritten_name: "_card_x".to_string(),
            },
            TransformClassNameRewriteV0 {
                original_name: "footer".to_string(),
                rewritten_name: "_footer_should_not_apply".to_string(),
            },
            TransformClassNameRewriteV0 {
                original_name: "title".to_string(),
                rewritten_name: "_title_z".to_string(),
            },
            TransformClassNameRewriteV0 {
                original_name: "item".to_string(),
                rewritten_name: "_item_q".to_string(),
            },
            TransformClassNameRewriteV0 {
                original_name: "paint".to_string(),
                rewritten_name: "_paint_p".to_string(),
            },
        ],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::HashCssModuleClassNames,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 3);
    assert_eq!(
        execution.output_css,
        r#"@supports selector(._card_x:has(.footer, ._title_z)) { ._item_q{ color: red; } } @supports (background: paint(.card)) { ._paint_p{ color: blue; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["css-modules-class-hashing", "print-css"]
    );
}

#[test]
fn execution_runtime_hashes_escaped_css_module_class_selectors() {
    let source = r#".foo\:bar { color: red; } :local(.foo\:bar) { color: blue; } :global(.foo\:bar) .foo\:bar { color: green; }"#;
    let context = TransformExecutionContextV0 {
        class_name_rewrites: vec![TransformClassNameRewriteV0 {
            original_name: "foo:bar".to_string(),
            rewritten_name: "_foo_bar_0".to_string(),
        }],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::HashCssModuleClassNames,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 3);
    assert_eq!(
        execution.output_css,
        r#"._foo_bar_0{ color: red; } ._foo_bar_0{ color: blue; } .foo\:bar ._foo_bar_0{ color: green; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["css-modules-class-hashing", "print-css"]
    );
}

#[test]
fn execution_runtime_hashes_nested_css_module_selectors_after_unwrap() {
    let source = r#".item { color: red; &--primary { color: blue; } & .body { color: green; } }"#;
    let context = TransformExecutionContextV0 {
        class_name_rewrites: vec![
            TransformClassNameRewriteV0 {
                original_name: "item".to_string(),
                rewritten_name: "_item_0".to_string(),
            },
            TransformClassNameRewriteV0 {
                original_name: "item--primary".to_string(),
                rewritten_name: "_item--primary_1".to_string(),
            },
            TransformClassNameRewriteV0 {
                original_name: "body".to_string(),
                rewritten_name: "_body_2".to_string(),
            },
        ],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Scss,
        &[
            TransformPassKind::HashCssModuleClassNames,
            TransformPassKind::NestingUnwrap,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(
        execution.ordered_pass_ids,
        vec!["nesting-unwrap", "css-modules-class-hashing", "print-css"]
    );
    assert!(execution.output_css.contains("._item_0{ color: red; }"));
    assert!(
        execution
            .output_css
            .contains("._item--primary_1{ color: blue; }")
    );
    assert!(
        execution
            .output_css
            .contains("._item_0 ._body_2{ color: green; }")
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["nesting-unwrap", "css-modules-class-hashing", "print-css"]
    );
}
