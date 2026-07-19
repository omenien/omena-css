use super::execute_transform_passes_on_source;
use crate::{
    TransformCascadeEnvironmentV0, TransformExecutionContextV0, TransformExecutionPolicyV0,
    TransformStrictPolicyReasonV0,
    execute_transform_passes_on_source_with_dialect_context_and_policy,
};
use omena_parser::StyleDialect;
use omena_transform_cst::TransformPassKind;

#[test]
fn execution_runtime_rewrites_declaration_values_inside_group_rules() {
    let source = r#"@media (min-width: 1px) { .a { width: calc(1px + 1px); margin: 1px 1px 1px 1px; color: blue; } } @supports (display: grid) { .b { color: blue; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::CalcReduction,
            TransformPassKind::ColorCompression,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        r#"@media (min-width: 1px) { .a { width: 2px; margin: 1px; color: #00f; } } @supports (display: grid) { .b { color: #00f; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec![
            "shorthand-combining",
            "calc-reduction",
            "color-compression",
            "print-css"
        ]
    );
}

#[test]
fn execution_runtime_removes_cascade_safe_duplicate_rules() {
    let source = r#".a { color: red; } .b { color: red; } .a { color: blue; } .a { color: red; } @media (min-width: 1px) { .m { color: red; } .x { color: blue; } .m { color: red; } } @media (max-width: 1px) { .m { color: red; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::RuleDeduplication,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#" .b { color: red; } .a { color: blue; } .a { color: red; } @media (min-width: 1px) {  .x { color: blue; } .m { color: red; } } @media (max-width: 1px) { .m { color: red; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["rule-deduplication", "print-css"]
    );
}

#[test]
fn execution_runtime_removes_overridden_same_property_declarations() {
    let source = r#".a { color: red; color: blue; --tone: red; --tone: blue; display: -webkit-box; display: flex; color: green !important; color: black !important; composes: base; composes: utility; } :export { token: red; token: blue; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::RuleDeduplication,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 3);
    assert_eq!(
        execution.output_css,
        r#".a {  color: blue;  --tone: blue; display: -webkit-box; display: flex;  color: black !important; composes: base; composes: utility; } :export { token: red; token: blue; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["rule-deduplication", "print-css"]
    );
}

#[test]
fn execution_runtime_merges_adjacent_same_selector_rules_only() {
    let source = r#".a { color: red; } .a { background: blue; } .a { outline: 0; } .b { color: red; } .a { border: 0; } @media (min-width: 1px) { .m { color: red; } .m { background: blue; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[TransformPassKind::RuleMerging, TransformPassKind::PrintCss],
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#".a { color: red; background: blue; outline: 0; } .b { color: red; } .a { border: 0; } @media (min-width: 1px) { .m { color: red; background: blue; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["rule-merging", "print-css"]
    );
}

#[test]
fn strict_verification_refuses_winner_sensitive_pass_without_cascade_environment() {
    let source = ".a { color: red; } .a { background: blue; }";
    let policy = TransformExecutionPolicyV0::for_profile("strict-verification").unwrap_or_default();
    let execution = execute_transform_passes_on_source_with_dialect_context_and_policy(
        source,
        StyleDialect::Css,
        &[TransformPassKind::RuleMerging],
        &TransformExecutionContextV0::default(),
        &policy,
    );

    assert_eq!(execution.output_css, source);
    assert_eq!(execution.strict_policy.refused_count, 1);
    assert_eq!(execution.strict_policy.rolled_back_count, 0);
    assert_eq!(
        execution.strict_policy.refusal_reasons[0].pass_id,
        "rule-merging"
    );
}

#[test]
fn strict_verification_admits_winner_sensitive_pass_with_cascade_environment() {
    let source = ".a { color: red; } .a { background: blue; }";
    let policy = TransformExecutionPolicyV0::for_profile("strict-verification").unwrap_or_default();
    let context = TransformExecutionContextV0 {
        cascade_environment: Some(TransformCascadeEnvironmentV0::default()),
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_context_and_policy(
        source,
        StyleDialect::Css,
        &[TransformPassKind::RuleMerging],
        &context,
        &policy,
    );

    assert_eq!(execution.strict_policy.refused_count, 0);
    assert_eq!(execution.strict_policy.rolled_back_count, 0);
    assert_eq!(execution.executed_pass_ids, vec!["rule-merging"]);
    assert_ne!(execution.output_css, source);
}

#[test]
fn strict_verification_rolls_back_when_required_observation_is_unavailable() {
    let source = ".a{& .b{color:red}}";
    let policy = TransformExecutionPolicyV0::for_profile("strict-verification").unwrap_or_default();
    let context = TransformExecutionContextV0 {
        cascade_environment: Some(TransformCascadeEnvironmentV0::default()),
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_context_and_policy(
        source,
        StyleDialect::Css,
        &[TransformPassKind::NestingUnwrap],
        &context,
        &policy,
    );

    assert_eq!(execution.output_css, source);
    assert_eq!(execution.strict_policy.refused_count, 0);
    assert_eq!(execution.strict_policy.rolled_back_count, 1);
    assert_eq!(
        execution.strict_policy.rollback_reasons[0].pass_id,
        "nesting-unwrap"
    );
    assert!(matches!(
        execution.strict_policy.rollback_reasons[0]
            .reasons
            .as_slice(),
        [TransformStrictPolicyReasonV0::ObservationUnavailable { .. }]
    ));

    let descriptive =
        execute_transform_passes_on_source(source, &[TransformPassKind::NestingUnwrap]);
    assert_eq!(descriptive.strict_policy.rolled_back_count, 0);
    assert_ne!(descriptive.output_css, source);
}

#[test]
fn descriptive_execution_does_not_enforce_strict_evidence() {
    let source = ".a { color: red; } .a { background: blue; }";
    let execution = execute_transform_passes_on_source(source, &[TransformPassKind::RuleMerging]);

    assert_eq!(execution.strict_policy.refused_count, 0);
    assert_eq!(execution.strict_policy.rolled_back_count, 0);
    assert_eq!(execution.executed_pass_ids, vec!["rule-merging"]);
}

#[test]
fn execution_runtime_preserves_declaration_boundaries_when_merging_semicolonless_rules() {
    let source = r#".b{color:red}.b{background:blue} @media (min-width: 1px) { .m { color: red } .m { background: blue } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[TransformPassKind::RuleMerging, TransformPassKind::PrintCss],
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#".b { color:red; background:blue; } @media (min-width: 1px) { .m { color: red; background: blue; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["rule-merging", "print-css"]
    );
}

#[test]
fn execution_runtime_merges_adjacent_same_conditional_wrappers() {
    let source = r#"@media (prefers-color-scheme: dark) { .card { color: white; } } @media (prefers-color-scheme: dark) { .card .title { color: #ddd; } } @supports (display: grid) { .grid { display: grid; } } @supports (display: flex) { .flex { display: flex; } } @supports (display: flex) { .flex .child { display: flex; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[TransformPassKind::RuleMerging, TransformPassKind::PrintCss],
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#"@media (prefers-color-scheme: dark) { .card { color: white; } .card .title { color: #ddd; } } @supports (display: grid) { .grid { display: grid; } } @supports (display: flex) { .flex { display: flex; } .flex .child { display: flex; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["rule-merging", "print-css"]
    );
}

#[test]
fn execution_runtime_merges_adjacent_same_block_selectors_only() {
    let source = r#".a { color: red; } .b { color: red; } .c { color: red; } .d { color: blue; } .e { color: red; } .x{color:red;}.y{color:red} @media (min-width: 1px) { .m { color: black; } .n { color: black; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::SelectorMerging,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 3);
    assert_eq!(
        execution.output_css,
        r#".a, .b, .c { color: red; } .d { color: blue; } .e, .x, .y { color: red; } @media (min-width: 1px) { .m, .n { color: black; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["selector-merging", "print-css"]
    );
}
