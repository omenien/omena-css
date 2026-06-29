use crate::{
    TransformCssModuleComposesResolutionV0, TransformExecutionContextV0,
    execute_transform_passes_on_source_with_dialect_and_context,
};
use omena_parser::StyleDialect;
use omena_transform_cst::TransformPassKind;

#[test]
fn execution_runtime_tree_shakes_class_owned_rules_with_closed_world_context() {
    let source = r#".used { color: red; } .dead { color: blue; } .dead:hover { color: green; } button.other-dead { color: black; } .also-dead, .other-dead { color: black; } .used, .dead-mixed { color: cyan; } .used .child { color: purple; } :where(.used) { color: navy; } :where(.dead-pseudo) { color: gold; } :is(.dead-pseudo-alt, .also-dead-pseudo-alt) { color: tan; } :is(.used, .dead-kept-alt) { color: teal; } :global(.external) { color: gray; } :global { .global-block { color: silver; } } .dead :global(.external) { color: pink; } :global(.root) .dead-global { color: lime; } :local(.dead-local) { color: brown; } @media (min-width: 1px) { .media-dead { color: orange; } .used { color: brown; } }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["used".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeClass,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 11);
    assert!(execution.output_css.contains(".used { color: red; }"));
    assert!(execution.output_css.contains(".used { color: cyan; }"));
    assert!(
        execution
            .output_css
            .contains(".used .child { color: purple; }")
    );
    assert!(
        execution
            .output_css
            .contains(":where(.used) { color: navy; }")
    );
    assert!(
        execution
            .output_css
            .contains(":is(.used, .dead-kept-alt) { color: teal; }")
    );
    assert!(
        execution
            .output_css
            .contains("@media (min-width: 1px) {  .used { color: brown; } }")
    );
    assert!(
        execution
            .output_css
            .contains(":global(.external) { color: gray; }")
    );
    assert!(
        execution
            .output_css
            .contains(":global { .global-block { color: silver; } }")
    );
    assert!(!execution.output_css.contains(".dead {"));
    assert!(!execution.output_css.contains(".dead:hover"));
    assert!(!execution.output_css.contains(".dead :global"));
    assert!(!execution.output_css.contains(".dead-global"));
    assert!(!execution.output_css.contains(".dead-local"));
    assert!(!execution.output_css.contains(".dead-pseudo"));
    assert!(!execution.output_css.contains(".dead-pseudo-alt"));
    assert!(!execution.output_css.contains(".also-dead-pseudo-alt"));
    assert!(!execution.output_css.contains("button.other-dead"));
    assert!(!execution.output_css.contains(".also-dead"));
    assert!(!execution.output_css.contains(".other-dead"));
    assert!(!execution.output_css.contains(".dead-mixed"));
    assert!(!execution.output_css.contains(".media-dead"));
    assert_eq!(
        execution.executed_pass_ids,
        vec!["tree-shake-class", "print-css"]
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
    assert_eq!(execution.semantic_removals.len(), 11);
    assert!(execution.semantic_removals.iter().any(|removal| {
        removal.symbol_kind == "class"
            && removal.name == "also-dead,other-dead"
            && removal.pass_id == "tree-shake-class"
            && removal
                .derivation_steps
                .contains(&"symbolNotMarkedReachable")
    }));
}

#[test]
fn execution_runtime_tree_shakes_escaped_class_owned_rules_with_closed_world_context() {
    let source = r#".foo\:bar { color: red; } .dead { color: blue; } .foo\:bar:hover { color: green; } .dead, .foo\:bar { color: cyan; } .hex\3A bar { color: purple; } .hex-dead { color: black; }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["foo:bar".to_string(), "hex:bar".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeClass,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert!(
        execution
            .output_css
            .contains(r#".foo\:bar { color: red; }"#)
    );
    assert!(
        execution
            .output_css
            .contains(r#".foo\:bar:hover { color: green; }"#)
    );
    assert!(
        execution
            .output_css
            .contains(r#".foo\:bar { color: cyan; }"#)
    );
    assert!(
        execution
            .output_css
            .contains(r#".hex\3A bar { color: purple; }"#)
    );
    assert!(!execution.output_css.contains(".dead {"));
    assert!(!execution.output_css.contains(".dead,"));
    assert!(!execution.output_css.contains(".hex-dead"));
    assert_eq!(
        execution.executed_pass_ids,
        vec!["tree-shake-class", "print-css"]
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
    assert_eq!(execution.semantic_removals.len(), 3);
    assert!(
        execution
            .semantic_removals
            .iter()
            .any(|removal| { removal.pass_id == "tree-shake-class" && removal.name == "dead" })
    );
}

#[test]
fn execution_runtime_keeps_composed_classes_reachable_during_tree_shaking() {
    let source = r#".button { composes: base; color: red; } .base { color: blue; } .utility { animation: spin 1s; color: var(--brand); } .dead { color: black; } @keyframes spin { to { opacity: 1; } } @keyframes ghost { to { opacity: 0; } } :root { --brand: red; --dead: blue; }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["button".to_string()],
        css_module_composes_resolutions: vec![TransformCssModuleComposesResolutionV0 {
            local_class_name: "button".to_string(),
            exported_class_names: vec![
                "button".to_string(),
                "base".to_string(),
                "utility".to_string(),
            ],
        }],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeClass,
            TransformPassKind::TreeShakeKeyframes,
            TransformPassKind::TreeShakeCustomProperty,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert!(execution.output_css.contains(".button"));
    assert!(execution.output_css.contains(".base"));
    assert!(execution.output_css.contains(".utility"));
    assert!(execution.output_css.contains("@keyframes spin"));
    assert!(execution.output_css.contains("--brand: red"));
    assert!(!execution.output_css.contains(".dead"));
    assert!(!execution.output_css.contains("@keyframes ghost"));
    assert!(!execution.output_css.contains("--dead: blue"));
    assert!(
        execution
            .semantic_removals
            .iter()
            .any(|removal| removal.pass_id == "tree-shake-class" && removal.name == "dead")
    );
}

#[test]
fn execution_runtime_expands_local_composes_during_class_tree_shaking() {
    let source = r#".button { composes: base utility global(reset); color: red; } .base { color: blue; } .utility { animation: spin 1s; color: var(--brand); } .dead { color: black; } @keyframes spin { to { opacity: 1; } } @keyframes ghost { to { opacity: 0; } } :root { --brand: red; --dead: blue; }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["button".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeClass,
            TransformPassKind::TreeShakeKeyframes,
            TransformPassKind::TreeShakeCustomProperty,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert!(execution.output_css.contains(".button"));
    assert!(execution.output_css.contains(".base"));
    assert!(execution.output_css.contains(".utility"));
    assert!(execution.output_css.contains("@keyframes spin"));
    assert!(execution.output_css.contains("--brand: red"));
    assert!(!execution.output_css.contains(".dead"));
    assert!(!execution.output_css.contains("@keyframes ghost"));
    assert!(!execution.output_css.contains("--dead: blue"));
    assert!(
        execution
            .semantic_removals
            .iter()
            .any(|removal| removal.pass_id == "tree-shake-class" && removal.name == "dead")
    );
}
