use super::{
    TransformCssModuleValueResolutionV0, TransformExecutionContextV0,
    execute_transform_passes_on_source,
    execute_transform_passes_on_source_with_closed_world_context,
    execute_transform_passes_on_source_with_dialect_and_context,
};
use omena_parser::StyleDialect;
use omena_transform_cst::TransformPassKind;

#[test]
fn execution_runtime_resolves_unique_static_root_custom_properties() {
    let source = r#":root { --brand: red; --gap: 2rem; --shadow: 0 0 var(--gap); --alias: var(--brand); --dynamic: var(--alias); --fallback: var(--missing, blue); --dup: red; --dup: blue; --cycle-a: var(--cycle-b); --cycle-b: var(--cycle-a); } @property --registered { syntax: "<color>"; inherits: false; initial-value: var(--brand); } @keyframes pulse { to { color: var(--brand); } } .card { color: var(--brand); margin: var(--gap); border-color: var(--missing, blue); background: var(--dup); outline-color: var(--dynamic); text-decoration-color: var(--fallback); caret-color: var(--cycle-a, green); box-shadow: var(--shadow); filter: drop-shadow(var(--missing, blue) 0 0); } @media screen { .card { color: var(--dynamic); } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::StaticVarSubstitution,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 11);
    assert_eq!(
        execution.output_css,
        r#":root { --brand: red; --gap: 2rem; --shadow: 0 0 var(--gap); --alias: var(--brand); --dynamic: var(--alias); --fallback: var(--missing, blue); --dup: red; --dup: blue; --cycle-a: var(--cycle-b); --cycle-b: var(--cycle-a); } @property --registered { syntax: "<color>"; inherits: false; initial-value: red; } @keyframes pulse { to { color: red; } } .card { color: red; margin: 2rem; border-color: blue; background: var(--dup); outline-color: red; text-decoration-color: blue; caret-color: green; box-shadow: 0 0 2rem; filter: drop-shadow(blue 0 0); } @media screen { .card { color: red; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["custom-property-static-resolve", "print-css"]
    );
}

#[test]
fn execution_runtime_resolves_static_var_multi_segment_fallbacks() {
    let source = r#":root { --brand: red; --accent: blue; } .card { background: var(--missing, linear-gradient(var(--brand), white), var(--accent)); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::StaticVarSubstitution,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#":root { --brand: red; --accent: blue; } .card { background: linear-gradient(red, white), blue; }"#
    );
}

#[test]
fn execution_runtime_preserves_css_wide_custom_property_keywords() {
    let source = r#":root { --brand: initial; --gap: unset; --ok: red; } .card { color: var(--brand, blue); margin: var(--gap, 1rem); border-color: var(--ok); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::StaticVarSubstitution,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#":root { --brand: initial; --gap: unset; --ok: red; } .card { color: var(--brand, blue); margin: var(--gap, 1rem); border-color: red; }"#
    );
}

#[test]
fn execution_runtime_resolves_static_custom_properties_in_at_rule_preludes() {
    let source = r#":root { --wide: 40rem; --mode: dark; --color: red; --scope-root: .card; } @custom-media --wide (min-width: var(--wide)); @container card style(--mode: var(--mode)) { .card { color: var(--color); } } @supports (color: var(--color)) { .card { border-color: currentColor; } } @media (min-width: var(--wide)) { .card { color: var(--color); } } @scope (var(--scope-root)) { .card { color: var(--color); } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::StaticVarSubstitution,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 8);
    assert_eq!(
        execution.output_css,
        r#":root { --wide: 40rem; --mode: dark; --color: red; --scope-root: .card; } @custom-media --wide (min-width: 40rem); @container card style(--mode: dark) { .card { color: red; } } @supports (color: red) { .card { border-color: currentColor; } } @media (min-width: 40rem) { .card { color: red; } } @scope (.card) { .card { color: red; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["custom-property-static-resolve", "print-css"]
    );
}

#[test]
fn execution_runtime_recovers_static_custom_property_substitution_after_malformed_var() {
    let source = r#":root { --brand: red; --gap: 2rem; } .card { border: 1px solid var(--brand) var(--broken; box-shadow: 0 0 var(--gap) var(--also-broken; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::StaticVarSubstitution,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#":root { --brand: red; --gap: 2rem; } .card { border: 1px solid red var(--broken; box-shadow: 0 0 2rem var(--also-broken; }"#
    );
}

#[test]
fn execution_runtime_recovers_static_custom_property_env_after_malformed_var() {
    let source = r#":root { --gap: 2rem; --shadow: 0 0 var(--gap) var(--broken; } .card { box-shadow: var(--shadow); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::StaticVarSubstitution,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#":root { --gap: 2rem; --shadow: 0 0 var(--gap) var(--broken; } .card { box-shadow: 0 0 2rem var(--broken; }"#
    );
}

#[test]
fn execution_runtime_keeps_shadowed_custom_properties_unresolved() {
    let source = r#":root { --brand: red; --gap: 2rem; --tone: red; --tone: blue !important; } .card { --brand: blue; color: var(--brand); margin: var(--gap); border-color: var(--tone); } .other { color: var(--brand); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::StaticVarSubstitution,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#":root { --brand: red; --gap: 2rem; --tone: red; --tone: blue !important; } .card { --brand: blue; color: var(--brand); margin: 2rem; border-color: var(--tone); } .other { color: var(--brand); }"#
    );
}

#[test]
fn execution_runtime_resolves_unique_property_initial_values() {
    let source = r#"@property --brand { syntax: "<color>"; inherits: false; initial-value: red; } @property --shadowed { syntax: "<color>"; inherits: false; initial-value: green; } @property --dynamic { syntax: "<color>"; inherits: false; initial-value: teal; } @property --dup { syntax: "<color>"; inherits: false; initial-value: blue; } @property --dup { syntax: "<color>"; inherits: false; initial-value: purple; } :root { --dynamic: env(theme-color); } .card { --shadowed: orange; color: var(--brand); background: var(--shadowed); border-color: var(--dup); outline-color: var(--dynamic); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::StaticVarSubstitution,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#"@property --brand { syntax: "<color>"; inherits: false; initial-value: red; } @property --shadowed { syntax: "<color>"; inherits: false; initial-value: green; } @property --dynamic { syntax: "<color>"; inherits: false; initial-value: teal; } @property --dup { syntax: "<color>"; inherits: false; initial-value: blue; } @property --dup { syntax: "<color>"; inherits: false; initial-value: purple; } :root { --dynamic: env(theme-color); } .card { --shadowed: orange; color: red; background: var(--shadowed); border-color: var(--dup); outline-color: var(--dynamic); }"#
    );
}

#[test]
fn execution_runtime_resolves_static_local_css_modules_values() {
    let source = r#"@value primary: #fff; @value spacing: 8px; @value alias: primary; @value shadow: 0 0 4px primary; @value bp: 40rem; @value wide: 80rem; @value width: 1px; @value modulePath: "./tokens.module.css"; @value dup: red; @value dup: blue; .btn { color: primary; margin: spacing spacing; background: alias; box-shadow: shadow; border-color: dup; } @media screen and (min-width: bp) and (width >= wide) and (bp <= width <= wide) { .btn { color: primary; } } @container card (inline-size >= wide) { .btn { margin: spacing; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ValueResolution,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 18);
    assert_eq!(
        execution.output_css,
        r#"       @value modulePath: "./tokens.module.css"; @value dup: red; @value dup: blue; .btn { color: #fff; margin: 8px 8px; background: #fff; box-shadow: 0 0 4px #fff; border-color: dup; } @media screen and (min-width: 40rem) and (width >= 80rem) and (40rem <= width <= 80rem) { .btn { color: #fff; } } @container card (inline-size >= 80rem) { .btn { margin: 8px; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["value-resolution", "print-css"]
    );
}

#[test]
fn execution_runtime_resolves_identifier_values_before_static_branch_evaluation() {
    let source = r#"@value mode: grid; @value bp: 0px; :root { --display: grid; --zero: 0px; } @supports (display: mode) { .value { color: red; } } @supports (display: var(--display)) { .var { color: blue; } } @media (max-width: bp) { .value-media { color: red; } } @media (max-width: var(--zero)) { .var-media { color: blue; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::MediaStaticEval,
            TransformPassKind::SupportsStaticEval,
            TransformPassKind::StaticVarSubstitution,
            TransformPassKind::ValueResolution,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(
        execution.executed_pass_ids,
        vec![
            "value-resolution",
            "custom-property-static-resolve",
            "supports-static-eval",
            "media-static-eval",
            "print-css"
        ]
    );
    assert_eq!(execution.mutation_count, 10);
    assert_eq!(
        execution.output_css,
        "  :root { --display: grid; --zero: 0px; } .value { color: red; } .var { color: blue; }  "
    );
}

#[test]
fn execution_runtime_resolves_scope_prelude_css_modules_values() {
    let source = r#"@value scopeRoot: .card; @value scopeLimit: #app; @value rootScope: :root; @value tone: red; @value dead: blue; @scope (scopeRoot) to (scopeLimit) { .card { color: tone; } } @scope (rootScope) { .card { border-color: tone; } }"#;
    let context = TransformExecutionContextV0 {
        reachable_class_names: vec!["card".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_closed_world_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::ValueResolution,
            TransformPassKind::TreeShakeValue,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 10);
    assert_eq!(
        execution.output_css,
        "     @scope (.card) to (#app) { .card { color: red; } } @scope (:root) { .card { border-color: red; } }"
    );
    assert!(execution.semantic_removals.is_empty());
    assert_eq!(
        execution.executed_pass_ids,
        vec!["value-resolution", "tree-shake-value", "print-css"]
    );
}

#[test]
fn execution_runtime_resolves_imported_static_css_modules_values_from_context() {
    let source = r#"@value primary as brand, gap, tone from "./tokens.module.css"; @custom-media --gap (min-width: gap); .btn { color: brand; margin: gap; border-color: tone; } @media (min-width: gap) { .btn { color: brand; } } @supports (width: gap) { .btn { color: brand; } }"#;
    let context = TransformExecutionContextV0 {
        css_module_value_resolutions: vec![
            TransformCssModuleValueResolutionV0 {
                local_name: "brand".to_string(),
                resolved_value: "#fff".to_string(),
            },
            TransformCssModuleValueResolutionV0 {
                local_name: "gap".to_string(),
                resolved_value: "8px".to_string(),
            },
        ],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::ValueResolution,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 8);
    assert_eq!(
        execution.output_css,
        r#"@value tone from "./tokens.module.css"; @custom-media --gap (min-width: 8px); .btn { color: #fff; margin: 8px; border-color: tone; } @media (min-width: 8px) { .btn { color: #fff; } } @supports (width: 8px) { .btn { color: #fff; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["value-resolution", "print-css"]
    );
}
