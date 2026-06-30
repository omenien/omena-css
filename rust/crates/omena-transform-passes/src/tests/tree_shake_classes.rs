use crate::{
    TransformCssModuleComposesResolutionV0, TransformExecutionContextV0,
    execute_transform_passes_on_source_with_dialect_and_context,
    execute_transform_passes_on_source_with_dialect_context_and_closed_world_bundle,
};
use omena_parser::{
    ClosedWorldBundleV0, ClosedWorldLinkedModuleV0, ConfigurationHashV0, ModuleIdV0,
    ModuleInstanceKeyV0, StyleDialect,
};
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
fn tree_shake_bundle_driven_matches_bool_driven_byte_identical() -> Result<(), String> {
    let source = r#".used { color: red; } .dead { color: blue; } .used .child { color: purple; }"#;
    let legacy_context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["used".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let open_context = TransformExecutionContextV0::default();
    let instance = ModuleInstanceKeyV0::new(
        ModuleIdV0::new("tree-shake-bundle.css"),
        ConfigurationHashV0::none(),
    );
    let bundle = ClosedWorldBundleV0::try_from_linked_modules(
        vec![instance.clone()],
        vec![ClosedWorldLinkedModuleV0::new(instance).with_class_name("used")],
    )
    .map_err(|err| format!("closed-world bundle should be constructible: {err:?}"))?;
    let passes = [
        TransformPassKind::TreeShakeClass,
        TransformPassKind::PrintCss,
    ];

    let legacy = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &passes,
        &legacy_context,
    );
    let bundle_driven =
        execute_transform_passes_on_source_with_dialect_context_and_closed_world_bundle(
            source,
            StyleDialect::Css,
            &passes,
            &open_context,
            &bundle,
        );

    assert_eq!(bundle_driven.output_css, legacy.output_css);
    assert_eq!(bundle_driven.mutation_count, legacy.mutation_count);
    assert_eq!(bundle_driven.semantic_removals, legacy.semantic_removals);
    Ok(())
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
fn execution_runtime_keeps_comment_prefixed_first_rule_during_class_tree_shaking() {
    let source = r#"/* generated header */
.headerDead { color: red; }
.used { color: blue; }
.plainDead { color: black; }"#;
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

    assert!(execution.output_css.contains(".headerDead { color: red; }"));
    assert!(execution.output_css.contains(".used { color: blue; }"));
    assert!(!execution.output_css.contains(".plainDead"));
    assert_eq!(execution.semantic_removals.len(), 1);
    assert!(
        execution
            .semantic_removals
            .iter()
            .any(|removal| removal.pass_id == "tree-shake-class" && removal.name == "plainDead")
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
