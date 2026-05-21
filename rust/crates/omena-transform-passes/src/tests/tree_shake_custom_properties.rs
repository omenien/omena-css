use crate::{
    TransformExecutionContextV0, execute_transform_passes_on_source_with_dialect_and_context,
};
use omena_parser::StyleDialect;
use omena_transform_cst::TransformPassKind;

#[test]
fn execution_runtime_tree_shakes_custom_properties_with_closed_world_context() {
    let source = r#":root { --used: VAR(--alias); --alias: red; --dead: VAR(--dead-dep); --dead-dep: blue; --string-only: orange; --dead-from-rule: black; color: VAR(--used); content: "var(--string-only)"; } .btn { color: var(--external); } .dead { color: var(--dead-from-rule); }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        reachable_custom_property_names: vec!["--external".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeCustomProperty,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 4);
    assert!(execution.output_css.contains("--used: VAR(--alias);"));
    assert!(execution.output_css.contains("--alias: red;"));
    assert!(execution.output_css.contains("color: VAR(--used);"));
    assert!(
        execution
            .output_css
            .contains(r#"content: "var(--string-only)";"#)
    );
    assert!(execution.output_css.contains("color: var(--external);"));
    assert!(!execution.output_css.contains("--dead:"));
    assert!(!execution.output_css.contains("--dead-dep:"));
    assert!(!execution.output_css.contains("--string-only:"));
    assert!(!execution.output_css.contains("--dead-from-rule:"));
    assert_eq!(
        execution.executed_pass_ids,
        vec!["tree-shake-custom-property", "print-css"]
    );
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.symbol_kind, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("customProperty", "--dead"),
            ("customProperty", "--dead-dep"),
            ("customProperty", "--string-only"),
            ("customProperty", "--dead-from-rule")
        ]
    );
}

#[test]
fn execution_runtime_tree_shakes_custom_property_icss_exports_with_closed_world_context() {
    let source = r#":root { --brand: red; --dead: blue; } :export { brand: var(--brand); dead: var(--dead); }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_custom_property_names: vec!["brand".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeCustomProperty,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert!(execution.output_css.contains("--brand: red;"));
    assert!(execution.output_css.contains("brand: var(--brand);"));
    assert!(!execution.output_css.contains("--dead: blue;"));
    assert!(!execution.output_css.contains("dead: var(--dead);"));
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.symbol_kind, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("customPropertyIcssExport", "dead"),
            ("customProperty", "--dead")
        ]
    );

    let all_unreachable = execute_transform_passes_on_source_with_dialect_and_context(
        r#":root { --dead: blue; } :export { dead: var(--dead); }"#,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeCustomProperty,
            TransformPassKind::PrintCss,
        ],
        &TransformExecutionContextV0 {
            closed_style_world: true,
            ..TransformExecutionContextV0::default()
        },
    );

    assert_eq!(all_unreachable.mutation_count, 2);
    assert!(!all_unreachable.output_css.contains(":export"));
    assert!(!all_unreachable.output_css.contains("--dead: blue;"));

    let css_name_root = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeCustomProperty,
            TransformPassKind::PrintCss,
        ],
        &TransformExecutionContextV0 {
            closed_style_world: true,
            reachable_custom_property_names: vec!["--brand".to_string()],
            ..TransformExecutionContextV0::default()
        },
    );

    assert_eq!(css_name_root.mutation_count, 2);
    assert!(css_name_root.output_css.contains("--brand: red;"));
    assert!(css_name_root.output_css.contains("brand: var(--brand);"));
    assert!(!css_name_root.output_css.contains("--dead: blue;"));
    assert!(!css_name_root.output_css.contains("dead: var(--dead);"));
}

#[test]
fn execution_runtime_ignores_unreachable_custom_property_dependencies() {
    let source = r#":root { --used: var(--dep); --dep: red; --ghost: blue; } .btn { color: var(--used); } .dead { --used: var(--ghost); color: var(--ghost); }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeCustomProperty,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert!(execution.output_css.contains("--used: var(--dep);"));
    assert!(execution.output_css.contains("--dep: red;"));
    assert!(!execution.output_css.contains("--ghost: blue;"));
    assert!(!execution.output_css.contains("--used: var(--ghost);"));
    assert!(execution.output_css.contains("color: var(--ghost);"));
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.symbol_kind, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![("customProperty", "--ghost"), ("customProperty", "--used")]
    );
}

#[test]
fn execution_runtime_ignores_malformed_var_in_unreachable_custom_property_rules() {
    let source = r#":root { --used: red; --dead: blue; } .btn { color: var(--used); } .dead { color: var(--broken; --other: var(--also-broken; }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeCustomProperty,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert!(execution.output_css.contains("--used: red;"));
    assert!(!execution.output_css.contains("--dead: blue;"));
    assert!(execution.output_css.contains("color: var(--broken;"));
    assert!(!execution.output_css.contains("--other: var(--also-broken;"));
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.symbol_kind, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![("customProperty", "--dead"), ("customProperty", "--other")]
    );
}

#[test]
fn execution_runtime_recovers_custom_property_tree_shaking_after_reachable_malformed_var() {
    let source = r#":root { --used: red; --dead: blue; } .btn { color: var(--used); outline: var(--broken; }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeCustomProperty,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 1);
    assert!(execution.output_css.contains("--used: red;"));
    assert!(execution.output_css.contains("color: var(--used);"));
    assert!(execution.output_css.contains("outline: var(--broken;"));
    assert!(!execution.output_css.contains("--dead: blue;"));
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.symbol_kind, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![("customProperty", "--dead")]
    );
}

#[test]
fn execution_runtime_ignores_dead_keyframe_custom_property_dependencies() {
    let source = r#":root { --used: red; --ghost: blue; } .btn { animation: live 1s; } @keyframes live { to { color: var(--used); } } @keyframes ghost { to { --used: var(--ghost); color: var(--ghost); } }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeCustomProperty,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert!(execution.output_css.contains("--used: red;"));
    assert!(execution.output_css.contains("color: var(--used);"));
    assert!(execution.output_css.contains("@keyframes ghost"));
    assert!(execution.output_css.contains("color: var(--ghost);"));
    assert!(!execution.output_css.contains("--ghost: blue;"));
    assert!(!execution.output_css.contains("--used: var(--ghost);"));
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.symbol_kind, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![("customProperty", "--ghost"), ("customProperty", "--used")]
    );
}

#[test]
fn execution_runtime_tree_shakes_custom_property_registrations_with_closed_world_context() {
    let source = r#"@property --used { syntax: "<color>"; inherits: false; initial-value: red; } @property --dead { syntax: "<color>"; inherits: false; initial-value: blue; } :root { --used: red; --dead: blue; } .btn { color: var(--used); } .dead { color: var(--dead); }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeCustomProperty,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert!(execution.output_css.contains("@property --used"));
    assert!(execution.output_css.contains("--used: red"));
    assert!(execution.output_css.contains("color: var(--used);"));
    assert!(!execution.output_css.contains("@property --dead"));
    assert!(!execution.output_css.contains("--dead: blue"));
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.symbol_kind, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("customPropertyRegistration", "--dead"),
            ("customProperty", "--dead")
        ]
    );
}

#[test]
fn execution_runtime_keeps_registration_initial_value_custom_property_dependencies() {
    let source = r#"@property --used { syntax: "<color>"; inherits: false; initial-value: var(--registered-dep); } @property --dead { syntax: "<color>"; inherits: false; initial-value: var(--dead-dep); } :root { --registered-dep: red; --dead-dep: blue; --ghost: orange; } .btn { color: var(--used); }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeCustomProperty,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 3);
    assert!(execution.output_css.contains("@property --used"));
    assert!(execution.output_css.contains("--registered-dep: red;"));
    assert!(execution.output_css.contains("color: var(--used);"));
    assert!(!execution.output_css.contains("@property --dead"));
    assert!(!execution.output_css.contains("--dead-dep: blue"));
    assert!(!execution.output_css.contains("--ghost: orange"));
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.symbol_kind, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("customPropertyRegistration", "--dead"),
            ("customProperty", "--dead-dep"),
            ("customProperty", "--ghost")
        ]
    );
}

#[test]
fn execution_runtime_keeps_custom_properties_used_by_descriptor_at_rules() {
    let source = r#":root { --font-src: url(omena.woff2); --dead: blue; } @font-face { font-family: "Omena"; src: var(--font-src); } .btn { color: red; }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeCustomProperty,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 1);
    assert!(
        execution
            .output_css
            .contains("--font-src: url(omena.woff2);")
    );
    assert!(
        execution
            .output_css
            .contains(r#"@font-face { font-family: "Omena"; src: var(--font-src); }"#)
    );
    assert!(!execution.output_css.contains("--dead: blue;"));
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.symbol_kind, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![("customProperty", "--dead")]
    );
}

#[test]
fn execution_runtime_keeps_container_style_query_custom_property_roots() {
    let source = r#"@property --theme { syntax: "<custom-ident>"; inherits: true; initial-value: light; } @property --dead { syntax: "<custom-ident>"; inherits: true; initial-value: off; } :root { --theme: dark; --dead: off; } @container card style(--theme: dark) { .btn { color: white; } } @container card style(--dead: off) { .dead { color: black; } }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeCustomProperty,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert!(execution.output_css.contains("@property --theme"));
    assert!(execution.output_css.contains("--theme: dark;"));
    assert!(
        execution
            .output_css
            .contains("@container card style(--theme: dark)")
    );
    assert!(!execution.output_css.contains("@property --dead"));
    assert!(
        !execution
            .output_css
            .contains(":root { --theme: dark; --dead: off;")
    );
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.symbol_kind, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("customPropertyRegistration", "--dead"),
            ("customProperty", "--dead")
        ]
    );
}

#[test]
fn execution_runtime_keeps_reachable_at_rule_prelude_custom_property_roots() {
    let source = r#":root { --gate: grid; --wide: 40rem; --dead: blue; --unreachable-width: 80rem; } @supports (display: var(--gate)) { .btn { color: red; } } @media (min-width: var(--wide)) { .btn { color: blue; } } @supports (color: var(--dead)) { .dead { color: black; } } @media (min-width: var(--unreachable-width)) { .dead { color: black; } }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeCustomProperty,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert!(execution.output_css.contains("--gate: grid;"));
    assert!(execution.output_css.contains("--wide: 40rem;"));
    assert!(
        execution
            .output_css
            .contains("@supports (display: var(--gate))")
    );
    assert!(
        execution
            .output_css
            .contains("@media (min-width: var(--wide))")
    );
    assert!(!execution.output_css.contains("--dead: blue;"));
    assert!(!execution.output_css.contains("--unreachable-width: 80rem;"));
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.symbol_kind, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("customProperty", "--dead"),
            ("customProperty", "--unreachable-width")
        ]
    );
}
