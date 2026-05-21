use crate::{
    TransformExecutionContextV0, execute_transform_passes_on_source_with_dialect_and_context,
};
use omena_parser::StyleDialect;
use omena_transform_cst::TransformPassKind;

#[test]
fn execution_runtime_tree_shakes_local_values_with_closed_world_context() {
    let source = r#"@value used: red; @value dead: blue; @value alias: used; @value shadow: 0 0 4px used; @value bp: 40rem; @value deadAlias: dead; @value deadShadow: 0 0 4px dead; @value deadBp: 50rem; @value deadFromRule: orange; @value deadExpr: calc(1rem + 2px); .btn { color: used; background: alias; box-shadow: shadow; } .dead { color: deadFromRule; } @media (min-width: bp) { .btn { color: red; } } @media (min-width: deadBp) { .dead { color: dead; } }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeValue,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 6);
    assert!(execution.output_css.contains("@value used: red;"));
    assert!(execution.output_css.contains("@value alias: used;"));
    assert!(
        execution
            .output_css
            .contains("@value shadow: 0 0 4px used;")
    );
    assert!(execution.output_css.contains("@value bp: 40rem;"));
    assert!(execution.output_css.contains("box-shadow: shadow;"));
    assert!(execution.output_css.contains("@media (min-width: bp)"));
    assert!(!execution.output_css.contains("@value dead:"));
    assert!(!execution.output_css.contains("@value deadAlias:"));
    assert!(!execution.output_css.contains("@value deadShadow:"));
    assert!(!execution.output_css.contains("@value deadBp:"));
    assert!(!execution.output_css.contains("@value deadFromRule:"));
    assert!(!execution.output_css.contains("@value deadExpr:"));
    assert_eq!(
        execution.executed_pass_ids,
        vec!["tree-shake-value", "print-css"]
    );
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| removal.name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "dead",
            "deadAlias",
            "deadShadow",
            "deadBp",
            "deadFromRule",
            "deadExpr"
        ]
    );
}

#[test]
fn execution_runtime_keeps_values_used_by_reachable_keyframes() {
    let source = r#"@value used: red; @value dead: blue; @value ghost: green; @keyframes pulse { to { color: used; } } @keyframes ghost { to { color: ghost; } } .btn { animation: pulse 1s; }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeKeyframes,
            TransformPassKind::TreeShakeValue,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 3);
    assert!(execution.output_css.contains("@value used: red;"));
    assert!(execution.output_css.contains("color: used;"));
    assert!(!execution.output_css.contains("@value dead:"));
    assert!(!execution.output_css.contains("@value ghost:"));
    assert!(!execution.output_css.contains("@keyframes ghost"));
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.pass_id, removal.symbol_kind, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("tree-shake-keyframes", "keyframes", "ghost"),
            ("tree-shake-value", "cssModuleValue", "dead"),
            ("tree-shake-value", "cssModuleValue", "ghost")
        ]
    );
}

#[test]
fn execution_runtime_keeps_values_used_by_dynamically_reachable_keyframes() {
    let source = r#"@value used: red; @value dead: blue; @keyframes pulse { to { color: used; } } @keyframes ghost { to { color: dead; } } .btn { animation: var(--motion-name); }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeKeyframes,
            TransformPassKind::TreeShakeValue,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 0);
    assert!(execution.output_css.contains("@value used: red;"));
    assert!(execution.output_css.contains("@value dead: blue;"));
    assert!(execution.output_css.contains("@keyframes pulse"));
    assert!(execution.output_css.contains("@keyframes ghost"));
    assert!(execution.semantic_removals.is_empty());
}

#[test]
fn execution_runtime_keeps_values_used_by_explicit_reachable_keyframes() {
    let source = r#"@value ghost: green; @value dead: blue; @keyframes ghost { to { color: ghost; } } @keyframes dead { to { color: dead; } } .btn { color: red; }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        reachable_keyframe_names: vec!["ghost".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeKeyframes,
            TransformPassKind::TreeShakeValue,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert!(execution.output_css.contains("@value ghost: green;"));
    assert!(execution.output_css.contains("@keyframes ghost"));
    assert!(!execution.output_css.contains("@value dead:"));
    assert!(!execution.output_css.contains("@keyframes dead"));
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.pass_id, removal.symbol_kind, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("tree-shake-keyframes", "keyframes", "dead"),
            ("tree-shake-value", "cssModuleValue", "dead")
        ]
    );
}

#[test]
fn execution_runtime_tree_shakes_at_rule_prelude_non_value_identifiers() {
    let source = r#"@value screen: 1px; @value width: 1px; @value bp: 40rem; @value wide: 80rem; @value theme: dark; @media screen and (min-width: bp) and (bp <= width <= wide) { .btn { color: red; } } @container card style(--mode: theme) { .btn { color: blue; } }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeValue,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert!(!execution.output_css.contains("@value screen:"));
    assert!(!execution.output_css.contains("@value width:"));
    assert!(execution.output_css.contains("@value bp: 40rem;"));
    assert!(execution.output_css.contains("@value wide: 80rem;"));
    assert!(execution.output_css.contains("@value theme: dark;"));
    assert!(
        execution
            .output_css
            .contains("@media screen and (min-width: bp) and (bp <= width <= wide)")
    );
    assert!(
        execution
            .output_css
            .contains("@container card style(--mode: theme)")
    );
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| removal.name.as_str())
            .collect::<Vec<_>>(),
        vec!["screen", "width"]
    );
}

#[test]
fn execution_runtime_keeps_values_used_by_descriptor_at_rules() {
    let source = r#"@value face: OmenaSans; @value weight: 700; @value dead: blue; @font-face { font-family: face; font-weight: weight; src: url(omena.woff2); } .btn { color: red; }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeValue,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 1);
    assert!(execution.output_css.contains("@value face: OmenaSans;"));
    assert!(execution.output_css.contains("@value weight: 700;"));
    assert!(
        execution
            .output_css
            .contains("@font-face { font-family: face; font-weight: weight;")
    );
    assert!(!execution.output_css.contains("@value dead:"));
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.symbol_kind, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![("cssModuleValue", "dead")]
    );
}

#[test]
fn execution_runtime_tree_shakes_imported_values_with_closed_world_context() {
    let source = r#"@value used, dead, ghost from "./tokens.module.css"; @value local: used; .btn { color: local; } .dead { color: dead; }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeValue,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert!(
        execution
            .output_css
            .contains(r#"@value used from "./tokens.module.css";"#)
    );
    assert!(execution.output_css.contains("@value local: used;"));
    assert!(!execution.output_css.contains("dead, ghost from"));
    assert!(execution.output_css.contains(".btn { color: local; }"));
    assert!(execution.output_css.contains(".dead { color: dead; }"));
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| removal.name.as_str())
            .collect::<Vec<_>>(),
        vec!["dead", "ghost"]
    );
}

#[test]
fn execution_runtime_tree_shakes_icss_exports_with_closed_world_context() {
    let source = r#"@value primary: red; @value shadow: 0 0 primary; @value dead: blue; :export { public-color: shadow; dead-public: dead; } .btn { color: red; }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        reachable_value_names: vec!["public-color".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeValue,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert!(execution.output_css.contains("@value primary: red;"));
    assert!(execution.output_css.contains("@value shadow: 0 0 primary;"));
    assert!(
        execution
            .output_css
            .contains(":export { public-color: shadow;")
    );
    assert!(!execution.output_css.contains("@value dead:"));
    assert!(!execution.output_css.contains("dead-public: dead"));
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.symbol_kind, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("cssModuleValue", "dead"),
            ("cssModuleIcssExport", "dead-public")
        ]
    );
}
