use crate::{
    TransformDesignTokenRouteV0, TransformExecutionContextV0, TransformImportInlineV0,
    execute_transform_passes_on_source_with_dialect_and_context,
};
use omena_parser::StyleDialect;
use omena_transform_cst::TransformPassKind;

#[test]
fn execution_runtime_routes_design_tokens_from_bridge_context() {
    let source = r#"@property --registered { syntax: "<color>"; inherits: false; initial-value: var(--pkg-brand); } @keyframes pulse { to { color: var(--pkg-border); } } .button { color: var(--pkg-brand); background: var(--pkg-brand, blue); border: 1px solid var(--pkg-border); outline-color: var(--pkg-brand) !important; box-shadow: 0 0 1px var(--unsafe); --local: var(--pkg-brand); --important-local: var(--pkg-border) !important; } @media screen { .button { outline-color: var(--pkg-brand); } }"#;
    let context = TransformExecutionContextV0 {
        design_token_routes: vec![
            TransformDesignTokenRouteV0 {
                token_name: "--pkg-brand".to_string(),
                routed_value: "var(--theme-brand)".to_string(),
            },
            TransformDesignTokenRouteV0 {
                token_name: "--pkg-border".to_string(),
                routed_value: "#123456".to_string(),
            },
            TransformDesignTokenRouteV0 {
                token_name: "--unsafe".to_string(),
                routed_value: "red; color: blue".to_string(),
            },
        ],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::DesignTokenRouting,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 9);
    assert_eq!(
        execution.output_css,
        r#"@property --registered { syntax: "<color>"; inherits: false; initial-value: var(--theme-brand); } @keyframes pulse { to { color: #123456; } } .button { color: var(--theme-brand); background: var(--theme-brand, blue); border: 1px solid #123456; outline-color: var(--theme-brand)!important; box-shadow: 0 0 1px var(--unsafe); --local: var(--theme-brand); --important-local: #123456!important; } @media screen { .button { outline-color: var(--theme-brand); } }"#
    );
    assert_eq!(execution.design_token_routes, context.design_token_routes);
    assert_eq!(
        execution.executed_pass_ids,
        vec!["design-token-routing", "print-css"]
    );
}

#[test]
fn execution_runtime_routes_design_tokens_in_supported_at_rule_preludes() {
    let source = r#"@custom-media --wide (min-width: var(--pkg-breakpoint)); @container card style(--theme: var(--pkg-theme)) { .button { color: var(--pkg-brand); } } @supports (color: var(--pkg-brand)) { .button { border-color: currentColor; } } @media (min-width: var(--pkg-breakpoint)) { .button { color: red; } }"#;
    let context = TransformExecutionContextV0 {
        design_token_routes: vec![
            TransformDesignTokenRouteV0 {
                token_name: "--pkg-theme".to_string(),
                routed_value: "var(--theme-mode)".to_string(),
            },
            TransformDesignTokenRouteV0 {
                token_name: "--pkg-brand".to_string(),
                routed_value: "#123456".to_string(),
            },
            TransformDesignTokenRouteV0 {
                token_name: "--pkg-breakpoint".to_string(),
                routed_value: "40rem".to_string(),
            },
        ],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::DesignTokenRouting,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 5);
    assert_eq!(
        execution.output_css,
        r#"@custom-media --wide (min-width: 40rem); @container card style(--theme: var(--theme-mode)) { .button { color: #123456; } } @supports (color: #123456) { .button { border-color: currentColor; } } @media (min-width: 40rem) { .button { color: red; } }"#
    );
}

#[test]
fn execution_runtime_routes_design_tokens_inside_custom_property_aliases() {
    let source = r#":root { --pkg-brand: var(--pkg-brand, black); --alias: var(--pkg-brand); --fallback-alias: var(--pkg-brand, var(--pkg-border)); --bridge: var(--pkg-border); } .button { color: var(--alias); }"#;
    let context = TransformExecutionContextV0 {
        design_token_routes: vec![
            TransformDesignTokenRouteV0 {
                token_name: "--pkg-brand".to_string(),
                routed_value: "var(--theme-brand)".to_string(),
            },
            TransformDesignTokenRouteV0 {
                token_name: "--pkg-border".to_string(),
                routed_value: "#123456".to_string(),
            },
        ],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::DesignTokenRouting,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 3);
    assert_eq!(
        execution.output_css,
        r#":root { --pkg-brand: var(--pkg-brand, black); --alias: var(--theme-brand); --fallback-alias: var(--theme-brand, #123456); --bridge: #123456; } .button { color: var(--alias); }"#
    );
}

#[test]
fn execution_runtime_routes_transitive_design_token_aliases_without_fallbacks() {
    let source =
        r#".button { color: var(--alias); border-color: var(--tone); box-shadow: var(--shadow); }"#;
    let context = TransformExecutionContextV0 {
        design_token_routes: vec![
            TransformDesignTokenRouteV0 {
                token_name: "--alias".to_string(),
                routed_value: "var(--brand)".to_string(),
            },
            TransformDesignTokenRouteV0 {
                token_name: "--brand".to_string(),
                routed_value: "red".to_string(),
            },
            TransformDesignTokenRouteV0 {
                token_name: "--tone".to_string(),
                routed_value: "color-mix(in srgb, var(--brand), white)".to_string(),
            },
            TransformDesignTokenRouteV0 {
                token_name: "--shadow".to_string(),
                routed_value: "0 0 var(--gap)".to_string(),
            },
            TransformDesignTokenRouteV0 {
                token_name: "--gap".to_string(),
                routed_value: "2rem".to_string(),
            },
        ],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::DesignTokenRouting,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 3);
    assert_eq!(
        execution.output_css,
        r#".button { color: red; border-color: color-mix(in srgb, red, white); box-shadow: 0 0 2rem; }"#
    );
}

#[test]
fn execution_runtime_routes_design_tokens_inserted_by_import_inline() {
    let source = r#"@import "./tokens.css"; .button { color: var(--alias); }"#;
    let context = TransformExecutionContextV0 {
        import_inlines: vec![TransformImportInlineV0 {
            import_source: "./tokens.css".to_string(),
            replacement_css: ":root { --alias: var(--brand); --brand: red; }".to_string(),
        }],
        design_token_routes: vec![
            TransformDesignTokenRouteV0 {
                token_name: "--alias".to_string(),
                routed_value: "var(--brand)".to_string(),
            },
            TransformDesignTokenRouteV0 {
                token_name: "--brand".to_string(),
                routed_value: "red".to_string(),
            },
        ],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::ImportInline,
            TransformPassKind::DesignTokenRouting,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(
        execution.output_css,
        ":root { --alias: red; --brand: red; } .button { color: red; }"
    );
    assert_eq!(execution.design_token_routes, context.design_token_routes);
}

#[test]
fn execution_runtime_preserves_design_token_fallback_aliases() {
    let source = r#".button { color: var(--alias, blue); }"#;
    let context = TransformExecutionContextV0 {
        design_token_routes: vec![
            TransformDesignTokenRouteV0 {
                token_name: "--alias".to_string(),
                routed_value: "var(--brand)".to_string(),
            },
            TransformDesignTokenRouteV0 {
                token_name: "--brand".to_string(),
                routed_value: "red".to_string(),
            },
        ],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::DesignTokenRouting,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#".button { color: var(--brand, blue); }"#
    );
}

#[test]
fn execution_runtime_routes_design_token_fallback_expressions_after_alias_routes() {
    let source = r#".button { color: var(--pkg-brand, var(--pkg-border)); background: var(--unknown, var(--pkg-border)); outline-color: var(--pkg-brand, color-mix(in srgb, var(--pkg-border), black)); }"#;
    let context = TransformExecutionContextV0 {
        design_token_routes: vec![
            TransformDesignTokenRouteV0 {
                token_name: "--pkg-brand".to_string(),
                routed_value: "var(--theme-brand)".to_string(),
            },
            TransformDesignTokenRouteV0 {
                token_name: "--pkg-border".to_string(),
                routed_value: "#123456".to_string(),
            },
        ],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::DesignTokenRouting,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 3);
    assert_eq!(
        execution.output_css,
        r#".button { color: var(--theme-brand, #123456); background: var(--unknown, #123456); outline-color: var(--theme-brand, color-mix(in srgb, #123456, black)); }"#
    );
}

#[test]
fn execution_runtime_routes_design_token_multi_segment_fallbacks() {
    let source = r#".button { background: var(--pkg-brand, linear-gradient(red, var(--pkg-border)), var(--pkg-border)); }"#;
    let context = TransformExecutionContextV0 {
        design_token_routes: vec![
            TransformDesignTokenRouteV0 {
                token_name: "--pkg-brand".to_string(),
                routed_value: "var(--theme-brand)".to_string(),
            },
            TransformDesignTokenRouteV0 {
                token_name: "--pkg-border".to_string(),
                routed_value: "#123456".to_string(),
            },
        ],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::DesignTokenRouting,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#".button { background: var(--theme-brand, linear-gradient(red, #123456), #123456); }"#
    );
}

#[test]
fn execution_runtime_recovers_design_token_routing_after_malformed_var() {
    let source =
        r#".button { color: var(--pkg-brand); box-shadow: 0 0 var(--pkg-border) var(--broken; }"#;
    let context = TransformExecutionContextV0 {
        design_token_routes: vec![
            TransformDesignTokenRouteV0 {
                token_name: "--pkg-brand".to_string(),
                routed_value: "var(--theme-brand)".to_string(),
            },
            TransformDesignTokenRouteV0 {
                token_name: "--pkg-border".to_string(),
                routed_value: "#123456".to_string(),
            },
        ],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::DesignTokenRouting,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#".button { color: var(--theme-brand); box-shadow: 0 0 #123456 var(--broken; }"#
    );
}
