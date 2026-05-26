use super::{
    TransformExecutionContextV0, execute_transform_passes_on_source,
    execute_transform_passes_on_source_with_dialect_and_context,
};
use omena_parser::StyleDialect;
use omena_transform_cst::TransformPassKind;

#[test]
fn execution_runtime_unwraps_simple_single_depth_nesting() {
    let source = r#".card { color: red; & .title { color: blue; } &:hover { color: green; } } .comma, .skip { & .x { color: red; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::NestingUnwrap,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#".card { color: red; } .card .title { color: blue; } .card:hover { color: green; } .comma .x, .skip .x { color: red; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["nesting-unwrap", "print-css"]
    );
}

#[test]
fn execution_runtime_unwraps_selector_list_nesting_without_splitting_function_commas() {
    let source = r#".card:is(.active, .selected), .panel { &:hover, &--open { color: red; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::NestingUnwrap,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#".card:is(.active, .selected):hover, .card:is(.active, .selected)--open, .panel:hover, .panel--open { color: red; }"#
    );
}

#[test]
fn execution_runtime_unwraps_nested_rule_descendants() {
    let source = r#".card { color: red; & .title { font-weight: bold; &:hover { color: blue; } .icon, &__icon { color: green; } } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::NestingUnwrap,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#".card { color: red; } .card .title { font-weight: bold; } .card .title:hover { color: blue; } .card .title .icon, .card .title__icon { color: green; }"#
    );
}

#[test]
fn execution_runtime_unwraps_explicit_nest_at_rules() {
    let source = r#".card { color: red; @nest .theme & { color: blue; & .title { color: green; } } @nest &:is(:hover, :focus) { color: purple; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::NestingUnwrap,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#".card { color: red; } .theme .card { color: blue; } .theme .card .title { color: green; } .card:is(:hover, :focus) { color: purple; }"#
    );
}

#[test]
fn execution_runtime_bubbles_nested_conditional_group_rules() {
    let source = r#".card { color: red; @media (min-width: 40rem) { color: blue; &:hover { color: green; } } @supports (display: grid) { & .title { display: grid; } } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::NestingUnwrap,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#".card { color: red; } @media (min-width: 40rem) { .card { color: blue; } .card:hover { color: green; } } @supports (display: grid) { .card .title { display: grid; } }"#
    );
}

#[test]
fn execution_runtime_unwraps_style_nesting_inside_conditional_groups() {
    let source = r#"@media (min-width: 40rem) { .card { color: red; & .title { color: blue; } } } @supports (display: grid) { .grid, .panel { &__item { display: grid; } } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::NestingUnwrap,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#"@media (min-width: 40rem) { .card { color: red; } .card .title { color: blue; } } @supports (display: grid) { .grid__item, .panel__item { display: grid; } }"#
    );
}

#[test]
fn execution_runtime_bubbles_starting_style_nesting() {
    let source =
        r#".card { color: red; @starting-style { opacity: 0; & .title { opacity: .5; } } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::NestingUnwrap,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#".card { color: red; } @starting-style { .card { opacity: 0; } .card .title { opacity: .5; } }"#
    );
}

#[test]
fn execution_runtime_flattens_only_root_scope_proof_candidates() {
    let source =
        r#"@scope (:root) { .card { color: red; } } @scope (.theme) { .title { color: blue; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[TransformPassKind::ScopeFlatten, TransformPassKind::PrintCss],
    );

    assert_eq!(execution.mutation_count, 0);
    assert_eq!(execution.output_css, source);

    let accepted = execute_transform_passes_on_source(
        r#"@scope (:root) { .card { color: red; } }"#,
        &[TransformPassKind::ScopeFlatten, TransformPassKind::PrintCss],
    );
    assert_eq!(accepted.mutation_count, 1);
    assert_eq!(accepted.output_css, r#".card { color: red; }"#);
    assert_eq!(
        accepted.executed_pass_ids,
        vec!["scope-flatten", "print-css"]
    );
    assert_eq!(
        accepted.cascade_proof_obligations.checked_pass_ids,
        vec!["scope-flatten"]
    );
    assert_eq!(accepted.cascade_proof_obligations.obligation_count, 1);
    assert_eq!(accepted.cascade_proof_obligations.accepted_count, 1);
    assert_eq!(
        accepted.cascade_proof_obligations.obligations[0].proof_product,
        "omena-cascade.scope-flatten-proof"
    );
    assert!(
        accepted.cascade_proof_obligations.obligations[0]
            .canonical_smt_input
            .as_ref()
            .is_some_and(|input| input.l1_primitive == "prove_scope_flatten_candidate")
    );
    assert!(
        execution
            .cascade_proof_obligations
            .obligations
            .iter()
            .any(|obligation| {
                obligation.proof_product == "omena-cascade.scope-flatten-proof"
                    && !obligation.accepted
                    && obligation.blocked_reason.as_deref()
                        == Some("peer scopes may change scope-proximity cascade ordering")
            })
    );
}

#[test]
fn execution_runtime_flattens_layers_only_with_closed_bundle_context() {
    let source = r#"@layer theme { .card { color: red; } }"#;
    let planned = execute_transform_passes_on_source(
        source,
        &[TransformPassKind::LayerFlatten, TransformPassKind::PrintCss],
    );
    assert_eq!(planned.output_css, source);
    assert_eq!(planned.planned_only_pass_ids, vec!["layer-flatten"]);

    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[TransformPassKind::LayerFlatten, TransformPassKind::PrintCss],
        &context,
    );
    assert_eq!(execution.mutation_count, 1);
    assert_eq!(execution.output_css, r#".card { color: red; }"#);
    assert_eq!(
        execution.executed_pass_ids,
        vec!["layer-flatten", "print-css"]
    );
    assert_eq!(planned.cascade_proof_obligations.obligation_count, 1);
    assert_eq!(planned.cascade_proof_obligations.blocked_count, 1);
    assert_eq!(
        planned.cascade_proof_obligations.obligations[0]
            .blocked_reason
            .as_deref(),
        Some("requires an explicit closed-style-world bundle witness before mutation")
    );
    assert!(
        planned.cascade_proof_obligations.obligations[0]
            .canonical_smt_input
            .as_ref()
            .is_some_and(|input| input.l1_primitive == "prove_layer_flatten_candidate")
    );
    assert_eq!(execution.cascade_proof_obligations.obligation_count, 1);
    assert_eq!(execution.cascade_proof_obligations.accepted_count, 1);
    assert_eq!(
        execution.cascade_proof_obligations.obligations[0].proof_product,
        "omena-cascade.layer-flatten-proof"
    );
    assert!(
        execution.cascade_proof_obligations.obligations[0]
            .canonical_smt_input
            .as_ref()
            .is_some_and(|input| input.l1_primitive == "prove_layer_flatten_candidate")
    );
}
