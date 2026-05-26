use super::{
    TransformExecutionContextV0, TransformPassRuntimeStatus, execute_transform_passes_on_source,
    execute_transform_passes_on_source_with_dialect,
    execute_transform_passes_on_source_with_dialect_and_context,
};
#[cfg(feature = "lawvere-trace")]
use super::{
    evaluate_lawvere_reorderability_with_differential_corpus,
    execute_transform_passes_on_source_with_lawvere_trace,
    plan_transform_passes_parallel_lawvere_layers,
};
use omena_parser::StyleDialect;
use omena_transform_cst::TransformPassKind;

#[test]
fn execution_runtime_applies_comment_strip_without_touching_strings() {
    let source = r#".a { color: red; /* remove */ content: "/* keep */"; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::CommentStrip,
            TransformPassKind::HashCssModuleClassNames,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.product, "omena-transform-passes.execution");
    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#".a { color: red;  content: "/* keep */"; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["comment-strip", "print-css"]
    );
    assert_eq!(
        execution.planned_only_pass_ids,
        vec!["css-modules-class-hashing"]
    );
    assert!(execution.provenance_preserved);
    assert_eq!(execution.pass_plan.violated_dag_edge_count, 0);
    assert!(execution.outcomes.iter().any(|outcome| {
        outcome.pass_id == "comment-strip"
            && outcome.status == TransformPassRuntimeStatus::Applied
            && outcome.mutation_count == 1
    }));
    assert!(execution.outcomes.iter().any(|outcome| {
        outcome.pass_id == "css-modules-class-hashing"
            && outcome.status == TransformPassRuntimeStatus::PlannedOnly
    }));
    assert_eq!(
        execution.provenance_derivation_forest.product,
        "omena-transform-passes.provenance-derivation-forest"
    );
    assert_eq!(execution.provenance_derivation_forest.root_count, 1);
    assert_eq!(
        execution.provenance_derivation_forest.node_count,
        execution.outcomes.len()
    );
    let comment_node = execution
        .provenance_derivation_forest
        .nodes
        .iter()
        .find(|node| node.pass_id == "comment-strip");
    assert!(
        comment_node.is_some(),
        "comment strip provenance node should exist"
    );
    let Some(comment_node) = comment_node else {
        return;
    };
    assert_eq!(comment_node.status, TransformPassRuntimeStatus::Applied);
    assert_eq!(comment_node.mutation_count, 1);
    assert_eq!(comment_node.mutation_spans.len(), 1);
    assert_eq!(comment_node.source_span_start, 17);
    assert!(comment_node.source_span_end < comment_node.input_byte_len);
    assert_eq!(comment_node.generated_span_start, 17);
    assert_eq!(comment_node.generated_span_end, 17);
    assert_eq!(
        execution.provenance_derivation_forest.nodes[0].parent_index,
        None
    );
    for (index, node) in execution
        .provenance_derivation_forest
        .nodes
        .iter()
        .enumerate()
        .skip(1)
    {
        assert_eq!(node.parent_index, Some(index - 1));
    }
}

#[test]
fn execution_runtime_applies_conservative_whitespace_normalization() {
    let source = r#".a , .b { color : red ; content: "x y"; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::WhitespaceStrip,
            TransformPassKind::CommentStrip,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 11);
    assert_eq!(execution.output_css, r#".a,.b{color:red;content:"x y"}"#);
    assert_eq!(
        execution.executed_pass_ids,
        vec!["whitespace-strip", "comment-strip", "print-css"]
    );
}

#[cfg(feature = "lawvere-trace")]
#[test]
fn lawvere_trace_path_preserves_existing_executor_signature_and_marks_terminal_pass() {
    let requested = [
        TransformPassKind::ColorCompression,
        TransformPassKind::NumberCompression,
        TransformPassKind::PrintCss,
    ];
    let plain =
        execute_transform_passes_on_source(".a { color: #ffffff; width: 1.0px; }", &requested);
    let (traced, trace) = execute_transform_passes_on_source_with_lawvere_trace(
        ".a { color: #ffffff; width: 1.0px; }",
        &requested,
    );
    let parallel_plan = plan_transform_passes_parallel_lawvere_layers(&requested);

    assert_eq!(traced.output_css, plain.output_css);
    assert_eq!(traced.ordered_pass_ids, plain.ordered_pass_ids);
    assert!(trace.preserves_existing_executor_signature);
    assert_eq!(trace.terminal_pass_ids, vec!["print-css"]);
    assert!(!parallel_plan.executor_consumes_plan);
    assert_eq!(parallel_plan.scheduler_status, "scaffoldOnly");
}

#[cfg(feature = "lawvere-trace")]
#[test]
fn lawvere_reorderability_uses_differential_commutativity_corpus() {
    let (certificate, witness) = evaluate_lawvere_reorderability_with_differential_corpus(
        TransformPassKind::CommentStrip,
        TransformPassKind::WhitespaceStrip,
        &[
            r#".a { color : red ; /* remove */ content : "x y" ; }"#,
            r#".b , .c { margin : 0px ; /* remove */ padding : 1px ; }"#,
        ],
    );

    assert_eq!(
        certificate.commute_witness,
        "differentialCommutativityCorpus"
    );
    assert_eq!(witness.fixture_count, 2);
    assert_eq!(witness.mismatch_count, 0);
    assert_eq!(certificate.differential_equal_fixture_count, 2);
    assert!(certificate.accepted);
    assert!(witness.cases.iter().all(|case| case.equal_output));
    assert!(
        witness
            .cases
            .iter()
            .all(|case| !case.left_then_right_css.contains("/*"))
    );
}

#[test]
fn execution_runtime_normalizes_important_annotation_whitespace() {
    let source = r#".a { color : red ! important ; margin : 0px !important ; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::WhitespaceStrip,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(
        execution.output_css,
        r#".a{color:red!important;margin:0px!important}"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["whitespace-strip", "print-css"]
    );
}

#[test]
fn execution_runtime_reduces_simple_same_unit_calc_values() {
    let source = r#".card { width: calc(1px + 2px); height: calc(10rem - 2rem); margin: calc(1px + 2rem); padding: calc(2px + 3px + 4px); margin-block-start: calc(10px - 3px - 2px); color: calc(1 + 2); gap: calc(.5rem+.25rem); inset: calc(1px - -2px); letter-spacing: calc(2px * 1); border-width: calc(1 * 3px); z-index: calc(4 / 1); scale: calc(3 * 0); box-shadow: 0 0 calc(1px + 2px) red; transform: translate(calc(10px - 2px), calc(1rem + 1rem)); min-width: min(10px, 4px); max-width: max(1rem, 2rem); block-size: min(2em, 1rem); opacity: max(.2, .5); outline-width: calc((2px * 3)); flex-basis: calc(2px * 3 * 4); inline-size: min(10px, max(2px, 4px)); line-height: clamp(.1, .5, .9); stroke-width: abs(-2px); order: sign(-10px); top: round(nearest, 10px, 3px); right: round(up, 10px, 3px); bottom: round(down, 10px, 3px); left: round(to-zero, 10px, 3px); translate: round(10px, 6px); rotate: round(nearest, 5px, 2px); margin-left: mod(10px, 3px); margin-right: rem(10px, 4px); perspective: mod(-10px, 3px); border-spacing: hypot(3px, 4px); flex-grow: hypot(3, 4); margin-bottom: hypot(3px, 4rem); animation-duration: sqrt(.25)s; grid-row: pow(2, 3); filter: brightness(exp(0)); font-size: log(100, 10)rem; min-height: sqrt(4px); line-width: pow(2px, 2); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::CalcReduction,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 35);
    assert_eq!(
        execution.output_css,
        r#".card { width: 3px; height: 8rem; margin: calc(1px + 2rem); padding: 9px; margin-block-start: 5px; color: 3; gap: 0.75rem; inset: 3px; letter-spacing: 2px; border-width: 3px; z-index: 4; scale: 0; box-shadow: 0 0 3px red; transform: translate(8px, 2rem); min-width: 4px; max-width: 2rem; block-size: min(2em, 1rem); opacity: 0.5; outline-width: 6px; flex-basis: 24px; inline-size: 4px; line-height: 0.5; stroke-width: 2px; order: -1; top: 9px; right: 12px; bottom: 9px; left: 9px; translate: 12px; rotate: round(nearest, 5px, 2px); margin-left: 1px; margin-right: 2px; perspective: mod(-10px, 3px); border-spacing: 5px; flex-grow: 5; margin-bottom: hypot(3px, 4rem); animation-duration: 0.5s; grid-row: 8; filter: brightness(1); font-size: 2rem; min-height: sqrt(4px); line-width: pow(2px, 2); }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["calc-reduction", "print-css"]
    );
}

#[test]
fn execution_runtime_removes_dead_branches_through_semantic_pass_surfaces() {
    let source = r#"@media not all { .dead { color: red; } } @supports (display: grid) { .grid { display: grid; } } @supports (display: -ms-grid) { .ms { display: -ms-grid; } } @supports (display: grid) and (color: red) { .conjunction { color: red; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::DeadMediaBranchRemoval,
            TransformPassKind::DeadSupportsBranchRemoval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        r#" .grid { display: grid; }  .conjunction { color: red; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec![
            "dead-media-branch-removal",
            "dead-supports-branch-removal",
            "print-css"
        ]
    );
}

#[test]
fn execution_runtime_removes_dark_media_branches_with_workspace_context() {
    let source = r#"@media (prefers-color-scheme: dark) { .dark { color: white; } } @media (prefers-color-scheme: light) { .light { color: black; } } @media screen and (prefers-color-scheme: dark) { .screen-dark { color: white; } }"#;
    let context = TransformExecutionContextV0 {
        drop_dark_mode_media_queries: true,
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::DeadMediaBranchRemoval,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#" @media (prefers-color-scheme: light) { .light { color: black; } } "#
    );
    assert!(!execution.output_css.contains("prefers-color-scheme: dark"));
}

#[test]
fn execution_runtime_uses_dialect_lexer_for_scss_silent_comments() {
    let source = ".a { // remove\n  color: red;\n  content: \"// keep\";\n}";
    let execution = execute_transform_passes_on_source_with_dialect(
        source,
        StyleDialect::Scss,
        &[TransformPassKind::CommentStrip],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        ".a { \n  color: red;\n  content: \"// keep\";\n}"
    );
}
