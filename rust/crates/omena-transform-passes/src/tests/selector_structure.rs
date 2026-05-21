use super::execute_transform_passes_on_source;
use omena_transform_cst::TransformPassKind;

#[test]
fn execution_runtime_compresses_specificity_safe_is_where_selectors() {
    let source = r#".a:is(.ready) { color: red; } .b:where(.x, .x) { color: blue; } .c:where(.y) { color: green; } .d:is(:is(.u, .v), .u) { color: orange; } .g:is(.p, .q):hover { color: lime; } .upper:IS(.one, .two) { color: pink; } .e, .e, .f { color: purple; } .w:where(:where(.one, .two), .one) { color: teal; } @media (min-width: 1px) { .m, .m, .n { color: black; } } @supports (display: grid) { .s, .s { display: grid; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::SelectorIsWhereCompression,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 10);
    assert_eq!(
        execution.output_css,
        r#".a.ready { color: red; } .b:where(.x) { color: blue; } .c:where(.y) { color: green; } .d.u, .d.v { color: orange; } .g.p:hover, .g.q:hover { color: lime; } .upper.one, .upper.two { color: pink; } .e, .f { color: purple; } .w:where(.one,.two) { color: teal; } @media (min-width: 1px) { .m, .n { color: black; } } @supports (display: grid) { .s { display: grid; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["selector-is-where-compression", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_keyframe_selector_aliases() {
    let source = r#"@keyframes fade { from { opacity: 0; } 100% { opacity: 1; } 50%, TO { opacity: .5; } } @-webkit-keyframes spin { FROM { transform: rotate(0deg); } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::SelectorIsWhereCompression,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        r#"@keyframes fade { 0%{ opacity: 0; } to{ opacity: 1; } 50%,to{ opacity: .5; } } @-webkit-keyframes spin { 0%{ transform: rotate(0deg); } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["selector-is-where-compression", "print-css"]
    );
}

#[test]
fn execution_runtime_removes_only_plain_empty_rules() {
    let source = r#".empty { } @media (min-width: 1px) { .nested { } } .outer { .inner { } } .with-comment { /* keep */ } .filled { color: red; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::EmptyRuleRemoval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 5);
    assert_eq!(
        execution.output_css,
        r#"   .with-comment { /* keep */ } .filled { color: red; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["empty-rule-removal", "print-css"]
    );
}

#[test]
fn execution_runtime_removes_comment_only_rules_after_comment_strip() {
    let source = r#".empty { } @media (min-width: 1px) { .nested { } .filled { color: red; } } .outer { .inner { } } .with-comment { /* remove after comment strip */ } .filled { color: red; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::EmptyRuleRemoval,
            TransformPassKind::CommentStrip,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(
        execution.ordered_pass_ids,
        vec!["comment-strip", "empty-rule-removal", "print-css"]
    );
    assert_eq!(execution.mutation_count, 6);
    assert_eq!(
        execution.output_css,
        r#" @media (min-width: 1px) {  .filled { color: red; } }   .filled { color: red; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["comment-strip", "empty-rule-removal", "print-css"]
    );
}

#[test]
fn execution_runtime_preserves_empty_keyframe_frames() {
    let source = r#"@keyframes fade { 0% {} to { opacity: 1 } } .empty{}"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::EmptyRuleRemoval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#"@keyframes fade { 0% {} to { opacity: 1 } } "#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["empty-rule-removal", "print-css"]
    );
}
