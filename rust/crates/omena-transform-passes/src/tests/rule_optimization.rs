use super::execute_transform_passes_on_source;
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
