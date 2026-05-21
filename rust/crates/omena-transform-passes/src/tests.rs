use super::{
    TransformCssModuleValueResolutionV0, TransformExecutionContextV0, TransformPassRuntimeStatus,
    execute_transform_passes_on_source, execute_transform_passes_on_source_with_dialect,
    execute_transform_passes_on_source_with_dialect_and_context,
};
use omena_parser::StyleDialect;
use omena_transform_cst::TransformPassKind;

mod class_hashing;
mod design_tokens;
mod import_inline;
mod module_evaluation;
mod runtime_boundary;
mod static_conditionals;
mod static_resolution;
mod tree_shake_classes;
mod tree_shake_custom_properties;
mod tree_shake_keyframes;
mod tree_shake_values;

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
fn execution_runtime_compresses_numeric_tokens_only() {
    let source = r#".a { width: 0.50rem; opacity: 000.50; margin: -0.25px 10.00%; scale: 1.0E+03; flex-grow: 1e+00; z-index: 001; order: +001; translate: 0e+3px; rotate: -0deg; content: "0.50 1.0E+03"; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::NumberCompression,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 10);
    assert_eq!(
        execution.output_css,
        r#".a { width: .5rem; opacity: .5; margin: -.25px 10%; scale: 1e3; flex-grow: 1; z-index: 1; order: 1; translate: 0px; rotate: 0deg; content: "0.50 1.0E+03"; }"#
    );
}

#[test]
fn execution_runtime_normalizes_zero_length_units_with_property_context() {
    let source = r#".a { margin: 0px 0.0rem -0em; border: 0px solid #000; border-top: 0px solid #000; border-top-width: 0PX; border-radius: -0em; border-spacing: 0px 0px; letter-spacing: 0px; word-spacing: 0px; scroll-margin-inline: 0rem; outline: 0px solid #000; outline-width: 0pt; outline-offset: 0px; text-decoration: underline 0px #000; text-indent: 0px; line-height: 0em; stroke-width: 0px; stroke-dasharray: 0px; stroke-dashoffset: 0px; tab-size: 0px; vertical-align: 0px; perspective: 0px; border-image-width: 0px; flex-basis: 0px; grid-template-columns: 0px 1FR; grid-auto-rows: 0px; font-size: 0px; rotate: 1TURN; animation-delay: 200MS; transition-duration: .05s; transition-delay: 0ms; --x: 0PX; width: 10PX; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 34);
    assert_eq!(
        execution.output_css,
        r#".a { margin: 0 0 0; border: 0 solid #000; border-top: 0 solid #000; border-top-width: 0; border-radius: 0; border-spacing: 0 0; letter-spacing: 0; word-spacing: 0; scroll-margin-inline: 0; outline: 0 solid #000; outline-width: 0; outline-offset: 0px; text-decoration: underline 0 #000; text-indent: 0; line-height: 0; stroke-width: 0; stroke-dasharray: 0; stroke-dashoffset: 0; tab-size: 0; vertical-align: 0; perspective: 0; border-image-width: 0; flex-basis: 0; grid-template-columns: 0 1fr; grid-auto-rows: 0; font-size: 0; rotate: 1turn; animation-delay: .2s; transition-duration: 50ms; transition-delay: 0s; --x: 0PX; width: 10px; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_removes_adjacent_duplicate_unit_declarations() {
    let source = r#".a { tab-size: 0px; tab-size: 0; width: 0px; width: 0; opacity: 100%; opacity: 1; color: red; color: red; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 5);
    assert_eq!(
        execution.output_css,
        r#".a { tab-size: 0;  width: 0;  opacity: 1; opacity: 1; color: red; color: red; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_safe_zero_percent_position_values() {
    let source = r#".a { background-position: 0% 0%; background-size: auto auto; mask-position: 0% 0%; mask-size: auto auto; -webkit-mask-size: auto auto; perspective-origin: 0% 0%; transform-origin: 0% 0%; object-position: 0% 0%; width: 0%; opacity: 0%; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 12);
    assert_eq!(
        execution.output_css,
        r#".a { background-position: 0 0; background-size: auto; mask-position: 0 0; mask-size: auto; -webkit-mask-size: auto; perspective-origin: 0 0; transform-origin: 0 0; object-position: 0% 0%; width: 0%; opacity: 0; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_static_opacity_percentages() {
    let source = r#".a { opacity: 50%; } .b { opacity: 100%; } .c { opacity: 5%; } .d { opacity: 150%; } .e { width: 50%; } .f { fill-opacity: 100%; stroke-opacity: 50%; flood-opacity: 0%; stop-opacity: 5%; } .g { opacity: 25%; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 5);
    assert_eq!(
        execution.output_css,
        r#".a { opacity: .5; } .b { opacity: 1; } .c { opacity: 5%; } .d { opacity: 150%; } .e { width: 50%; } .f { fill-opacity: 1; stroke-opacity: .5; flood-opacity: 0%; stop-opacity: 5%; } .g { opacity: .25; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_static_aspect_ratio_spacing() {
    let source = r#".a { aspect-ratio: 16 / 9; } .b { aspect-ratio: auto 4 / 3; } .c { aspect-ratio: var(--ratio); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#".a { aspect-ratio: 16/9; } .b { aspect-ratio: auto 4/3; } .c { aspect-ratio: var(--ratio); }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_static_center_position_values() {
    let source = r#".a { background-position: center center; transform-origin: center; mask-position: CENTER CENTER; mask-position-x: center; mask-position-y: CENTER; object-position: center center; } .b { background-position: left center; transform-origin: center top; mask-position: bottom right; mask-position-x: right; } .c { background-position: 0% 50%; mask-position: 100% 50%; -webkit-mask-position: 50% 50%; transform-origin: 50% 0%; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 13);
    assert_eq!(
        execution.output_css,
        r#".a { background-position: 50%; transform-origin: 50%; mask-position: 50%; mask-position-x: 50%; mask-position-y: 50%; object-position: center center; } .b { background-position: 0; transform-origin: top; mask-position: 100% 100%; mask-position-x: right; } .c { background-position: 0%; mask-position: 100%; -webkit-mask-position: 50%; transform-origin: 50% 0; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_zero_transform_function_units() {
    let source = r#".a { transform: rotate(0deg) rotateX(-0turn) translate(0px) skew(0deg); } .b { rotate: 0deg; transform: rotate(1deg) translate(1px); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#".a { transform: rotate(0)rotateX(0)translate(0)skew(0deg); } .b { rotate: 0deg; transform: rotate(1deg) translate(1px); }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_repeated_transform_scale_values() {
    let source = r#".a { transform: scale(1, 1) scale(2, 2) scale(.5, .5) scale(1, 2); } .b { transform: scale(var(--x), var(--x)); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#".a { transform: scale(1)scale(2)scale(.5)scaleY(2); } .b { transform: scale(var(--x), var(--x)); }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_static_3d_transform_axes() {
    let source = r#".a { transform: scale(2, 1) scale3d(1, 1, 1) scale3d(2, 3, 1) scale3d(1, 1, 2) rotate3d(1, 0, 0, 0deg) rotate3d(0, 1, 0, 1turn) rotate3d(0, 0, 1, 10deg) translate3d(0px, 0px, 0px) translate3d(1px, 0px, 0px) translate3d(0px, 1px, 0px) translate3d(0px, 0px, 1px) translate3d(1px, 2px, 0px); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#".a { transform: scaleX(2)scale(1)scale(2,3)scaleZ(2)rotateX(0)rotateY(1turn)rotate(10deg)translate(0,0)translate(1px)translateY(1px)translateZ(1px)translate(1px,2px); }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_zero_transform_axis_lengths() {
    let source = r#".a { transform: translateX(0px) translateY(-0%) translateZ(0em) translate(0px, 0%) perspective(0px); } .b { transform: translateX(1px) translate(0px, 1px); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#".a { transform: translate(0)translateY(0)translateZ(0)translate(0)perspective(0); } .b { transform: translateX(1px) translate(0px, 1px); }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_static_transform_tail_zeros() {
    let source = r#".a { transform: translate(1px, 0px) skew(0deg, 0deg) skewX(0deg) skewY(-0turn); } .b { transform: translate(1px, 2px) skew(1deg, 2deg); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#".a { transform: translate(1px)skew(0deg)skew(0)skewY(0); } .b { transform: translate(1px, 2px) skew(1deg, 2deg); }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_static_filter_default_functions() {
    let source = r#".a { filter: opacity(100%) brightness(1) contrast(+1) saturate(0100%) blur(0px) hue-rotate(-0deg); } .b { backdrop-filter: opacity(.5) blur(1px); } .c { -webkit-filter: opacity(1.0); } .d { filter: drop-shadow(red 0px 0px 0px); } .e { filter: drop-shadow(1px 2px 0px #000); } .f { filter: grayscale(0) sepia(0%) invert(.0); } .g { filter: grayscale(1) invert(100%); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 7);
    assert_eq!(
        execution.output_css,
        r#".a { filter: opacity()brightness()contrast()saturate()blur()hue-rotate(); } .b { backdrop-filter: opacity(.5)blur(1px); } .c { -webkit-filter: opacity(); } .d { filter: drop-shadow(0 0 red); } .e { filter: drop-shadow(1px 2px #000); } .f { filter: none; } .g { filter: grayscale(1)invert(100%); }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_individual_transform_properties() {
    let source = r#".t0 { translate: 0px 0% 0px; } .t1 { translate: 1px 0px; } .t2 { translate: 0px 1px; } .t3 { translate: 1px 2px 0px; } .t4 { translate: 0px 0px 1px; } .s0 { scale: 1 1; } .s1 { scale: 2 2; } .s2 { scale: 1 2; } .s3 { scale: 2 3 1; } .s4 { scale: 1 1 2; } .s5 { scale: 1 1 1; } .s6 { scale: 50% 50%; } .r0 { rotate: z 0deg; } .r1 { rotate: 0 0 1 10deg; } .r2 { rotate: 1 0 0 .500turn; } .r3 { rotate: 0 1 0 10.0deg; } .r4 { rotate: 0rad; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 21);
    assert_eq!(
        execution.output_css,
        r#".t0 { translate: 0; } .t1 { translate: 1px; } .t2 { translate: 0 1px; } .t3 { translate: 1px 2px; } .t4 { translate: 0 0 1px; } .s0 { scale: 1; } .s1 { scale: 2; } .s2 { scale: 1 2; } .s3 { scale: 2 3; } .s4 { scale: 1 1 2; } .s5 { scale: 1; } .s6 { scale: .5; } .r0 { rotate: 0deg; } .r1 { rotate: 10deg; } .r2 { rotate: x .5turn; } .r3 { rotate: y 10deg; } .r4 { rotate: 0deg; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_static_shadow_zero_lengths() {
    let source = r#".a { box-shadow: 0px 0px 0px #000; } .b { box-shadow: inset 1px 2px 0px 0px #000; } .c { text-shadow: 1px 2px 0px #000; } .d { box-shadow: 1px 2px 0px 5px #000; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 10);
    assert_eq!(
        execution.output_css,
        r#".a { box-shadow: 0 0 #000; } .b { box-shadow: inset 1px 2px #000; } .c { text-shadow: 1px 2px #000; } .d { box-shadow: 1px 2px 0 5px #000; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_compresses_static_declaration_colors_only() {
    let source = r#".a { color: #FFFFFF; box-shadow: 0 0 #AABBCC, 0 0 blue; border: 1px solid black; font-family: blue; background: url(blue.svg); background-color: rgb(255 0 0); border-color: rgb(0, 128, 0); outline-color: rgb(50% 50% 50%); text-emphasis-color: rgb(128 0 128); text-decoration-color: hsl(240 100% 50%); caret-color: hsl(0, 0%, 0%); fill: hwb(0 0% 0%); stroke: hwb(120 0% 50%); column-rule-color: hwb(0 100% 0%); flood-color: white; lighting-color: black; stop-color: blue; scrollbar-color: hsl(.5TURN 100% 50%); border-block-color: hwb(200GRAD 0% 0%); border-left-color: rgb(255 0 0 / 100%); border-right-color: hsl(120 100% 25% / 1); border-top-color: hwb(240 0% 0% / 100%); background: linear-gradient(rgb(255 0 0), hsl(240 100% 50%)); filter: drop-shadow(0 0 1px hwb(0 100% 0%)); border-bottom-color: rgb(255 0 0 / .5); accent-color: hsl(0 0% 0% / 50%); --brand: rgb(255 0 0); } .alpha { color: #FFFFFFFF; background-color: #ffff; border-color: #00000000; outline-color: rgba(255, 0, 0, 1); text-decoration-color: hsla(240, 100%, 50%, 100%); accent-color: rgba(255, 0, 0, .5); text-shadow: 0 0 hsla(240, 100%, 50%, 50%); column-rule-color: hwb(0 0% 0% / 50%); fill: transparent; box-shadow: 0 0 transparent; } #FFFFFF { color: red; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ColorCompression,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 35);
    assert_eq!(
        execution.output_css,
        r#".a { color: #fff; box-shadow: 0 0 #abc, 0 0 #00f; border: 1px solid #000; font-family: blue; background: url(blue.svg); background-color: red; border-color: green; outline-color: gray; text-emphasis-color: purple; text-decoration-color: #00f; caret-color: #000; fill: red; stroke: green; column-rule-color: #fff; flood-color: #fff; lighting-color: #000; stop-color: #00f; scrollbar-color: #0ff; border-block-color: #0ff; border-left-color: red; border-right-color: green; border-top-color: #00f; background: linear-gradient(red, #00f); filter: drop-shadow(0 0 1px #fff); border-bottom-color: #ff000080; accent-color: #00000080; --brand: rgb(255 0 0); } .alpha { color: #fff; background-color: #fff; border-color: #0000; outline-color: red; text-decoration-color: #00f; accent-color: #ff000080; text-shadow: 0 0 #0000ff80; column-rule-color: #ff000080; fill: #0000; box-shadow: 0 0 #0000; } #FFFFFF { color: red; }"#
    );
}

#[test]
fn execution_runtime_compresses_default_linear_gradient_directions() {
    let source = r#".a { background: linear-gradient(to bottom, red, blue); background-image: repeating-linear-gradient(180deg, white, black); list-style-image: linear-gradient(0.5turn, red, blue); mask-image: linear-gradient(200grad, red, blue); border-image-source: linear-gradient(to top, red, blue); } .b { background: radial-gradient(circle at center, red, blue); } .c { background: radial-gradient(ellipse at center, red, blue); } .d { background: conic-gradient(from 0deg, red, blue); } .e { background: repeating-conic-gradient(from 0turn, red, blue); } .f { background: linear-gradient(0deg, red 10%, blue 90%); background-image: repeating-linear-gradient(0turn, white, black); } .g { background: linear-gradient(to right, red, blue); background-image: repeating-linear-gradient(to left, white, black); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ColorCompression,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 13);
    assert_eq!(
        execution.output_css,
        r#".a { background: linear-gradient(red,#00f); background-image: repeating-linear-gradient(#fff,#000); list-style-image: linear-gradient(red,#00f); mask-image: linear-gradient(red,#00f); border-image-source: linear-gradient(#00f,red); } .b { background: radial-gradient(circle,red,#00f); } .c { background: radial-gradient(red,#00f); } .d { background: conic-gradient(red,#00f); } .e { background: repeating-conic-gradient(red,#00f); } .f { background: linear-gradient(#00f 10%,red 90%); background-image: repeating-linear-gradient(#000,#fff); } .g { background: linear-gradient(90deg,red,#00f); background-image: repeating-linear-gradient(270deg,#fff,#000); }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["color-compression", "print-css"]
    );
}

#[test]
fn execution_runtime_compresses_hex_colors_to_shorter_named_colors() {
    let source = r#".card { color: #ff0000; outline-color: #808080; background: #0000ff; border-color: #FFFFFF; box-shadow: 0 0 1px rebeccapurple; text-shadow: 0 0 1px aliceblue; caret-color: darkgray; accent-color: #d2b48c; fill: LightGoldenRodYellow; column-rule-color: currentcolor; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ColorCompression,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 9);
    assert_eq!(
        execution.output_css,
        r#".card { color: red; outline-color: gray; background: #00f; border-color: #fff; box-shadow: 0 0 1px #639; text-shadow: 0 0 1px #f0f8ff; caret-color: #a9a9a9; accent-color: tan; fill: #fafad2; column-rule-color: currentcolor; }"#
    );
}

#[test]
fn execution_runtime_keeps_column_rule_color_case() {
    let source = r#".a { column-rule: medium none currentcolor; column-rule-color: currentcolor; color: currentcolor; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ColorCompression,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#".a { column-rule: medium none currentcolor; column-rule-color: currentcolor; color: currentColor; }"#
    );
}

#[test]
fn execution_runtime_removes_adjacent_duplicate_color_declarations_after_compression() {
    let source = r#".a { color: rgb(255 0 0); color: rgb(255 0 0 / 100%); background: blue; background: #0000FF; } .b { color: red; margin: 1px; color: red; } .important { color: red !important; color: red !important; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ColorCompression,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 6);
    assert_eq!(
        execution.output_css,
        r#".a { color: red;  background: #00f;  } .b { color: red; margin: 1px; color: red; } .important { color: red !important; color: red !important; }"#
    );
}

#[test]
fn execution_runtime_preserves_minified_declaration_shape_for_value_replacements() {
    let source = ".a{background:blue}.b{margin:calc(2rem + 3rem)}.c{width:calc(2px * 3);height:calc(6px / 2)}";
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ColorCompression,
            TransformPassKind::CalcReduction,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        ".a{background:#00f}.b{margin:5rem}.c{width:6px;height:3px}"
    );
}

#[test]
fn execution_runtime_strips_safe_url_quotes_only() {
    let source = r#".a { background: url("img/icon.svg"); mask: url("has space.svg"); content: "url(\"keep\")"; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UrlQuoteStrip,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#".a { background: url(img/icon.svg); mask: url("has space.svg"); content: "url(\"keep\")"; }"#
    );
}

#[test]
fn execution_runtime_normalizes_safe_strings_without_rewriting_semantic_strings() {
    let source = r#".a { font-family: 'Demo'; content: 'has "quote"'; background: url('asset.svg'); } .b { font-family: "serif"; } .c { font-family: "Open Sans", "Helvetica Neue", "system-ui"; } .d { font-family: "--brand"; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::StringQuoteNormalize,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        r#".a { font-family: Demo; content: 'has "quote"'; background: url("asset.svg"); } .b { font-family: "serif"; } .c { font-family: Open Sans,Helvetica Neue,"system-ui"; } .d { font-family: --brand; }"#
    );
}

#[test]
fn execution_runtime_normalizes_static_font_longhand_keywords() {
    let source = r#".a { font-weight: normal; font-stretch: normal; } .b { font-weight: bold; font-stretch: condensed; } .c { font-weight: bolder; font-stretch: 80%; } .d { font-stretch: 100%; color: red; font-stretch: 50%; font-weight: normal; font-weight: 700; } .important { font-stretch: 100% !important; font-stretch: 50%; } .bad { font-stretch: 100%; font-stretch: bad; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::StringQuoteNormalize,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 7);
    assert_eq!(
        execution.output_css,
        r#".a { font-weight: 400; font-stretch: 100%; } .b { font-weight: 700; font-stretch: 75%; } .c { font-weight: bolder; font-stretch: 80%; } .d {  color: red; font-stretch: 50%;  font-weight: 700; } .important { font-stretch: 100% !important; font-stretch: 50%; } .bad { font-stretch: 100%; font-stretch: bad; }"#
    );
}

#[test]
fn execution_runtime_normalizes_static_single_keyword_case() {
    let source = r#".a { cursor: POINTER; user-select: NONE; position: STICKY; text-align: MATCH-PARENT; visibility: HIDDEN; pointer-events: NONE; cursor: -WEBKIT-GRAB; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::StringQuoteNormalize,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 5);
    assert_eq!(
        execution.output_css,
        r#".a { cursor: pointer; user-select: none; position: sticky; text-align: match-parent; visibility: hidden; pointer-events: NONE; cursor: -WEBKIT-GRAB; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["string-quote-normalize", "print-css"]
    );
}

#[test]
fn execution_runtime_combines_static_font_longhands() {
    let source = r#".a { font-style: normal; font-variant-caps: normal; font-weight: normal; font-stretch: normal; font-size: 16px; line-height: normal; font-family: Arial; } .b { font-style: normal; font-variant-caps: normal; font-weight: bold; font-stretch: condensed; font-size: 16px; line-height: 1.5; font-family: Arial, sans-serif; } .c { font-style: italic; font-variant-caps: small-caps; font-weight: bold; font-stretch: condensed; font-size: 1rem; line-height: 120%; font-family: "Open Sans", serif; } .d { font-style: normal !important; font-variant-caps: normal !important; font-weight: normal !important; font-stretch: normal !important; font-size: 16px !important; line-height: normal !important; font-family: Arial !important; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::StringQuoteNormalize,
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        r#".a { font: 16px Arial; } .b { font: 700 75% 16px/1.5 Arial,sans-serif; } .c { font: italic small-caps 700 75% 1rem/120% Open Sans,serif; } .d { font: 16px Arial!important; }"#
    );
}

#[test]
fn execution_runtime_normalizes_static_display_multi_keywords() {
    let source = r#".a { display: block flow; } .b { display: inline flow; } .c { display: block flow-root; } .d { display: inline flow-root; } .e { display: inline flex; } .f { display: block grid; } .g { display: list-item block flow; } .h { display: block ruby; } .i { display: BLOCK; } .j { display: INLINE RUBY; } .k { display: list-item inline flow; } .l { display: block flow list-item; } .m { display: list-item flow-root; } .n { display: INITIAL; } .o { display: INLINE BLOCK; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::StringQuoteNormalize,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 12);
    assert_eq!(
        execution.output_css,
        r#".a { display: block; } .b { display: inline; } .c { display: flow-root; } .d { display: inline-block; } .e { display: inline-flex; } .f { display: grid; } .g { display: list-item; } .h { display: block ruby; } .i { display: block; } .j { display: ruby; } .k { display: inline list-item; } .l { display: list-item; } .m { display: flow-root list-item; } .n { display: INITIAL; } .o { display: INLINE BLOCK; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["string-quote-normalize", "print-css"]
    );
}

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

#[test]
fn execution_runtime_combines_adjacent_box_longhands_with_cascade_proof() {
    let source = r#".a { margin-top: 1px; margin-right: 2px; margin-bottom: 1px; margin-left: 2px; border-top-color: red; border-right-color: blue; border-bottom-color: red; border-left-color: blue; border-top-width: 1px; border-right-width: 2px; border-bottom-width: 3px; border-left-width: 2px; padding-top: 1px; color: red; padding-right: 2px; padding-bottom: 3px; padding-left: 4px; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 3);
    assert_eq!(
        execution.output_css,
        r#".a { margin: 1px 2px; border-color: red blue; border-width: 1px 2px 3px; padding-top: 1px; color: red; padding-right: 2px; padding-bottom: 3px; padding-left: 4px; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["shorthand-combining", "print-css"]
    );
    assert_eq!(
        execution.cascade_proof_obligations.product,
        "omena-transform-passes.cascade-proof-obligations"
    );
    assert_eq!(execution.cascade_proof_obligations.obligation_count, 4);
    assert_eq!(execution.cascade_proof_obligations.accepted_count, 3);
    assert!(
        execution
            .cascade_proof_obligations
            .obligations
            .iter()
            .any(|obligation| {
                obligation.pass_id == "shorthand-combining"
                    && obligation.proof_product == "omena-cascade.shorthand-combination-proof"
                    && obligation.accepted
                    && obligation
                        .checked_obligations
                        .contains(&"canonicalLonghandSet")
            })
    );
    assert!(
        execution
            .cascade_proof_obligations
            .obligations
            .iter()
            .any(|obligation| {
                obligation.pass_id == "shorthand-combining"
                    && !obligation.accepted
                    && obligation.blocked_reason.as_deref()
                        == Some("longhands are not in canonical top/right/bottom/left order")
            })
    );
}

#[test]
fn execution_runtime_compresses_box_shorthand_values() {
    let source = r#".a { margin: 1px 1px 1px 1px; padding: 1px 2px 3px 2px; border-color: red blue red blue; border-width: 1px 1px; border-style: solid solid solid solid; border-image-slice: 100% 100% 100% 100%; border-image-width: 1 1 1 1; border-image-outset: 0 0 0 0; border: medium none currentColor; border-top: currentColor medium none; outline: medium none currentColor; } .important { margin: 1px 1px 1px 1px !important; border: medium none currentColor !important; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 13);
    assert_eq!(
        execution.output_css,
        r#".a { margin: 1px; padding: 1px 2px 3px; border-color: red blue; border-width: 1px; border-style: solid; border-image-slice: 100%; border-image-width: 1; border-image-outset: 0; border: none; border-top: none; outline: none; } .important { margin: 1px!important; border: none!important; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["shorthand-combining", "print-css"]
    );
}

#[test]
fn execution_runtime_compresses_border_image_longhands() {
    let source = r#".a { border-image-source: url(a.png); border-image-slice: 10; border-image-width: 1; border-image-outset: 0; border-image-repeat: stretch; } .b { border-image-source: linear-gradient(red,#00f); border-image-slice: 10 20; border-image-width: auto; border-image-outset: 1; border-image-repeat: round; } .c { border-image-source: none; border-image-slice: 10; border-image-width: 1; border-image-outset: 0; border-image-repeat: stretch; } .d { border-image-source: url(a.png); border-image-slice: 10 fill; border-image-width: 2; border-image-outset: 0; border-image-repeat: round space; } .invalid { border-image-source: url(a.png); border-image-slice: 10; border-image-width: fill; border-image-outset: 0; border-image-repeat: stretch; } .default { border-image-source: none; border-image-slice: 100%; border-image-width: 1; border-image-outset: 0; border-image-repeat: stretch; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        r#".a { border-image: url(a.png) 10; } .b { border-image: linear-gradient(red,#00f) 10 20/auto/1 round; } .c { border-image: 10; } .d { border-image: url(a.png) 10 fill/2 round space; } .invalid { border-image-source: url(a.png); border-image-slice: 10; border-image-width: fill; border-image-outset: 0; border-image-repeat: stretch; } .default { border-image-source: none; border-image-slice: 100%; border-image-width: 1; border-image-outset: 0; border-image-repeat: stretch; }"#
    );
}

#[test]
fn execution_runtime_compresses_existing_font_shorthand_defaults() {
    let source = r#".a { font: normal normal normal 16px/normal Arial; } .b { font: italic normal normal 16px/normal Arial; } .c { font: normal normal 16px Arial; } .d { font: bold 16px/normal Arial; } .e { font: italic small-caps bold condensed 1rem/120% "Open Sans", serif; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 5);
    assert_eq!(
        execution.output_css,
        r#".a { font: 16px Arial; } .b { font: italic 16px Arial; } .c { font: 16px Arial; } .d { font: 700 16px Arial; } .e { font: italic small-caps 700 75% 1rem/120% Open Sans,serif; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["shorthand-combining", "print-css"]
    );
}

#[test]
fn execution_runtime_compresses_important_shorthand_values() {
    let source = r#".a { margin: 0 0 0 0 !important; padding: 1px 1px 1px 1px !important; border-radius: 1px 1px 1px 1px !important; background-repeat: repeat repeat !important; overflow: visible visible !important; gap: 1px 1px !important; text-decoration: underline solid currentcolor auto !important; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 7);
    assert_eq!(
        execution.output_css,
        r#".a { margin: 0!important; padding: 1px!important; border-radius: 1px!important; background-repeat: repeat!important; overflow: visible!important; gap: 1px!important; text-decoration: underline!important; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["shorthand-combining", "print-css"]
    );
}

#[test]
fn execution_runtime_compresses_overflow_and_background_repeat_shorthands() {
    let source = r#".a { overflow-x: visible; overflow-y: visible; background-repeat: repeat repeat; } .b { overflow-x: hidden; color: red; overflow-y: hidden; background-repeat: round space; } .c { background-repeat: Repeat Repeat; } .d { overflow: hidden hidden; background-repeat: repeat no-repeat; } .e { overflow: visible visible; background-repeat: no-repeat repeat; } .f { overflow-x: auto; overflow-y: hidden; } .g { overflow-y: scroll; overflow-x: clip; } .h { overflow: AUTO HIDDEN; } .pos { background-position-x: left; background-position-y: top; } .pos-center { background-position-x: center; background-position-y: center; } .pos-reverse { background-position-y: top; background-position-x: center; } .pos-important { background-position-x: left !important; background-position-y: top !important; } .important { overflow-x: auto !important; overflow-y: auto !important; background-repeat: no-repeat no-repeat !important; } .bg { background-image: url(hero.svg); background-repeat: no-repeat repeat; background-color: rgb(255 0 0); } .bg-guard { background-position: center; background-image: url(hero.svg); background-repeat: repeat; background-color: red; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 16);
    assert_eq!(
        execution.output_css,
        r#".a { overflow: visible; background-repeat: repeat; } .b { overflow-x: hidden; color: red; overflow-y: hidden; background-repeat: round space; } .c { background-repeat: repeat; } .d { overflow: hidden; background-repeat: repeat-x; } .e { overflow: visible; background-repeat: repeat-y; } .f { overflow: auto hidden; } .g { overflow: clip scroll; } .h { overflow: auto hidden; } .pos { background-position: 0 0; } .pos-center { background-position: 50%; } .pos-reverse { background-position: top; } .pos-important { background-position: 0 0!important; } .important { overflow-x: auto !important; overflow-y: auto !important; background-repeat: no-repeat!important; } .bg { background: url(hero.svg) repeat-y rgb(255 0 0); } .bg-guard { background-position: center; background-image: url(hero.svg); background-repeat: repeat; background-color: red; }"#
    );
}

#[test]
fn execution_runtime_compresses_place_axis_shorthands() {
    let source = r#".items { align-items: stretch; justify-items: stretch; } .content { align-content: center; justify-content: center; } .self { justify-self: end; align-self: start; } .important { align-items: start !important; justify-items: end !important; } .mixed { align-items: first baseline; justify-items: center; } .legacy { justify-items: legacy left; align-items: normal; } .safe { align-self: safe center; justify-self: unsafe end; } .content-multi { align-content: space-between; justify-content: first baseline; } .content-shorthand { place-content: normal normal; } .items-stretch { place-items: stretch stretch; } .self-auto { place-self: auto auto; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 9);
    assert_eq!(
        execution.output_css,
        r#".items { place-items: stretch stretch; } .content { place-content: center; } .self { place-self: start end; } .important { place-items: start end!important; } .mixed { place-items: baseline center; } .legacy { place-items: normal legacy left; } .safe { place-self: safe center unsafe end; } .content-multi { align-content: space-between; justify-content: first baseline; } .content-shorthand { place-content: normal; } .items-stretch { place-items: stretch stretch; } .self-auto { place-self: auto; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["shorthand-combining", "print-css"]
    );
}

#[test]
fn execution_runtime_compresses_gap_axis_shorthands() {
    let source = r#".a { row-gap: 1px; column-gap: 1px; } .b { gap: 2px 2px; } .c { column-gap: 2px; row-gap: 1px; } .important { row-gap: 1px !important; column-gap: 2px !important; } .mixed { row-gap: calc(1px + 1px); column-gap: 2px; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 5);
    assert_eq!(
        execution.output_css,
        r#".a { gap: 1px; } .b { gap: 2px; } .c { gap: 1px 2px; } .important { gap: 1px 2px!important; } .mixed { gap: calc(1px + 1px) 2px; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["shorthand-combining", "print-css"]
    );
}

#[test]
fn execution_runtime_compresses_scroll_box_shorthands() {
    let source = r#".a { scroll-margin-top: 1px; scroll-margin-right: 2px; scroll-margin-bottom: 1px; scroll-margin-left: 2px; } .b { scroll-padding-top: 1px; scroll-padding-right: 1px; scroll-padding-bottom: 1px; scroll-padding-left: 1px; } .c { scroll-margin: 3px 3px; } .important { scroll-margin-top: 1px !important; scroll-margin-right: 2px !important; scroll-margin-bottom: 1px !important; scroll-margin-left: 2px !important; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 3);
    assert_eq!(
        execution.output_css,
        r#".a { scroll-margin: 1px 2px; } .b { scroll-padding: 1px; } .c { scroll-margin: 3px; } .important { scroll-margin-top: 1px !important; scroll-margin-right: 2px !important; scroll-margin-bottom: 1px !important; scroll-margin-left: 2px !important; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["shorthand-combining", "print-css"]
    );
}

#[test]
fn execution_runtime_compresses_text_decoration_shorthands() {
    let source = r#".a { text-decoration-line: underline; text-decoration-style: solid; text-decoration-color: currentcolor; text-decoration-thickness: auto; } .b { text-decoration: underline solid red auto; } .c { text-decoration-line: underline; text-decoration-style: wavy; text-decoration-color: red; text-decoration-thickness: 1px; } .important { text-decoration-line: underline !important; text-decoration-style: solid !important; text-decoration-color: currentcolor !important; text-decoration-thickness: auto !important; } .mixed { text-decoration-line: underline overline; text-decoration-style: solid; text-decoration-color: currentcolor; text-decoration-thickness: auto; } .em-a { text-emphasis-style: none; text-emphasis-color: currentcolor; } .em-b { text-emphasis-style: filled dot; text-emphasis-color: red; } .em-c { text-emphasis-style: open sesame !important; text-emphasis-color: currentcolor !important; } .pos-a { text-emphasis-position: over right; } .pos-b { text-emphasis-position: left under; } .pos-c { text-emphasis-position: over left; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 10);
    assert_eq!(
        execution.output_css,
        r#".a { text-decoration: underline; } .b { text-decoration: underline red; } .c { text-decoration: underline 1px wavy red; } .important { text-decoration: underline!important; } .mixed { text-decoration: underline overline; } .em-a { text-emphasis: none; } .em-b { text-emphasis: dot red; } .em-c { text-emphasis: open sesame!important; } .pos-a { text-emphasis-position: over; } .pos-b { text-emphasis-position: under left; } .pos-c { text-emphasis-position: over left; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["shorthand-combining", "print-css"]
    );
}

#[test]
fn execution_runtime_compresses_logical_axis_shorthands() {
    let source = r#".a { padding-block-start: 1px; padding-block-end: 1px; } .b { margin-inline-start: 1px; margin-inline-end: 2px; } .c { inset-block-end: 2px; inset-block-start: 1px; } .d { border-block-start-color: red; border-block-end-color: red; } .e { border-inline-start-width: 1px; border-inline-end-width: 2px; } .f { scroll-margin-block-start: 1px; scroll-margin-block-end: 1px; } .g { scroll-padding-inline-end: 2px; scroll-padding-inline-start: 1px; } .h { inset-block-start: 1px; inset-inline-end: 2px; inset-block-end: 1px; inset-inline-start: 2px; } .border-all { border-block-start-width: 1px; border-block-end-width: 1px; border-inline-start-width: 1px; border-inline-end-width: 1px; } .important { padding-block-start: 1px !important; padding-block-end: 2px !important; } .mixed { padding-block-start: calc(1px + 1px); padding-block-end: 2px; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 11);
    assert_eq!(
        execution.output_css,
        r#".a { padding-block: 1px; } .b { margin-inline: 1px 2px; } .c { inset-block: 1px 2px; } .d { border-block-color: red; } .e { border-inline-width: 1px 2px; } .f { scroll-margin-block: 1px; } .g { scroll-padding-inline: 1px 2px; } .h { inset-block: 1px; inset-inline: 2px; } .border-all { border-width: 1px; } .important { padding-block: 1px 2px!important; } .mixed { padding-block: calc(1px + 1px) 2px; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["shorthand-combining", "print-css"]
    );
}

#[test]
fn execution_runtime_compresses_line_style_shorthands() {
    let source = r#".a { border-top-width: 1px; border-top-style: solid; border-top-color: red; } .b { border-width: medium; border-style: none; border-color: currentcolor; } .c { outline-width: medium; outline-style: solid; outline-color: currentcolor; } .d { outline-width: 1px; outline-style: none; outline-color: red; } .e { border-inline-width: medium !important; border-inline-style: none !important; border-inline-color: currentcolor !important; } .f { border-color: red; border-style: solid; border-width: 1px; } .g { border-top: 1px solid red; border-right: 1px solid red; border-bottom: 1px solid red; border-left: 1px solid red; } .h { border-width: 1px 1px 1px 1px; border-style: solid solid solid solid; border-color: red red red red; } .mixed { border-top-width: 1px; color: blue; border-top-style: solid; border-top-color: red; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 8);
    assert_eq!(
        execution.output_css,
        r#".a { border-top: 1px solid red; } .b { border: none; } .c { outline: solid; } .d { outline: 1px red; } .e { border-inline: none!important; } .f { border: 1px solid red; } .g { border: 1px solid red; } .h { border: 1px solid red; } .mixed { border-top-width: 1px; color: blue; border-top-style: solid; border-top-color: red; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["shorthand-combining", "print-css"]
    );
}

#[test]
fn execution_runtime_compresses_logical_border_line_shorthands() {
    let source = r#".a { border-block-start-width: 1px; border-block-start-style: solid; border-block-start-color: red; } .b { border-block-start: 1px solid red; border-block-end: 1px solid red; } .c { border-block-start-width: 1px; border-block-start-style: solid; border-block-start-color: red; border-block-end-width: 1px; border-block-end-style: solid; border-block-end-color: red; } .d { border-inline-end: 1px solid red; border-inline-start: 1px solid red; } .e { border-inline-end-width: medium !important; border-inline-end-style: none !important; border-inline-end-color: currentcolor !important; border-inline-start-width: medium !important; border-inline-start-style: none !important; border-inline-start-color: currentcolor !important; } .different { border-block-start: 1px solid red; border-block-end: 2px solid red; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 5);
    assert_eq!(
        execution.output_css,
        r#".a { border-block-start: 1px solid red; } .b { border-block: 1px solid red; } .c { border-block: 1px solid red; } .d { border-inline: 1px solid red; } .e { border-inline: none!important; } .different { border-block-start: 1px solid red; border-block-end: 2px solid red; }"#
    );
}

#[test]
fn execution_runtime_compresses_repeated_axis_shorthand_values() {
    let source = r#".a { mask-repeat: repeat repeat; -webkit-mask-repeat: no-repeat no-repeat; background-repeat: space round; } .b { border-spacing: 1px 1px; } .c { scroll-padding-inline: 1px 1px; scroll-margin-block: 1px 2px; } .d { padding-inline: 2px 2px; margin-block: 1px 2px; } .e { border-block-color: red red; border-inline-width: 1px 1px; } .f { background-repeat: repeat no-repeat; mask-repeat: no-repeat repeat; -webkit-mask-repeat: repeat no-repeat; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 10);
    assert_eq!(
        execution.output_css,
        r#".a { mask-repeat: repeat; -webkit-mask-repeat: no-repeat; background-repeat: space round; } .b { border-spacing: 1px; } .c { scroll-padding-inline: 1px; scroll-margin-block: 1px 2px; } .d { padding-inline: 2px; margin-block: 1px 2px; } .e { border-block-color: red; border-inline-width: 1px; } .f { background-repeat: repeat-x; mask-repeat: repeat-y; -webkit-mask-repeat: repeat-x; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["shorthand-combining", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_mask_default_values() {
    let source = r#".mask { mask-size: auto auto; mask-repeat: repeat repeat; -webkit-mask-size: auto auto; -webkit-mask-repeat: no-repeat no-repeat; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        r#".mask { mask-size: auto; mask-repeat: repeat; -webkit-mask-size: auto; -webkit-mask-repeat: no-repeat; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["shorthand-combining", "unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_compresses_static_flex_shorthands() {
    let source = r#".a { flex: 0 1 auto; } .b { flex: 1 1 0%; } .c { flex: 2 1 0%; } .d { flex: 1 2 0%; } .e { flex: var(--flex); } .f { flex: 0 0 auto; } .g { flex-flow: row nowrap; } .h { flex-flow: row wrap; } .i { flex-flow: nowrap row; } .j { flex-direction: row; flex-wrap: nowrap; } .k { flex-wrap: wrap; flex-direction: column; } .l { flex-direction: row !important; flex-wrap: nowrap !important; } .m { flex-basis: 0%; flex: 1 1 0%; } .n { flex-basis: 0% !important; flex: 1; } .o { flex-grow: 1; flex-shrink: 1; flex: 2 1 0%; } .p { flex-grow: 1; flex-shrink: 1; flex-basis: 0%; } .q { flex-grow: 1; flex-shrink: 1; flex-basis: 10px; } .r { flex: 1 1 0; } .s { flex: 1 1 0px; } .t { flex-grow: 1 !important; flex-shrink: 1 !important; flex-basis: 0% !important; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 20);
    assert_eq!(
        execution.output_css,
        r#".a { flex: 0 auto; } .b { flex: 1; } .c { flex: 2; } .d { flex: 1 2; } .e { flex: var(--flex); } .f { flex: none; } .g { flex-flow: row; } .h { flex-flow: wrap; } .i { flex-flow: row; } .j { flex-flow: row; } .k { flex-flow: column wrap; } .l { flex-flow: row!important; } .m {  flex: 1; } .n { flex-basis: 0% !important; flex: 1; } .o {   flex: 2; } .p { flex: 1; } .q { flex: 10px; } .r { flex: 1 1 0; } .s { flex: 1 1 0; } .t { flex: 1!important; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["shorthand-combining", "print-css"]
    );
}

#[test]
fn execution_runtime_compresses_static_motion_shorthands() {
    let source = r#".a { transition: all 0s ease 0s; } .b { transition: opacity 0s linear .1s; } .c { transition: opacity 0s ease 0s, color .2s ease 0s; } .d { animation: none 0s ease 0s 1 normal none running; } .e { animation: 0s ease 0s 1 normal none running fade; } .f { animation: fade .2s ease 0s 1 normal none running; } .g { transition-property: all; transition-duration: 0s; transition-timing-function: ease; transition-delay: 0s; } .h { transition-property: opacity; transition-duration: .2s; transition-timing-function: ease; transition-delay: 0s; } .i { transition-property: all !important; transition-duration: 0s !important; transition-timing-function: ease !important; transition-delay: 0s !important; } .j { animation-name: fade; animation-duration: 0s; animation-timing-function: ease; animation-delay: 0s; animation-iteration-count: 1; animation-direction: normal; animation-fill-mode: none; animation-play-state: running; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 8);
    assert_eq!(
        execution.output_css,
        r#".a { transition: all; } .b { transition: opacity 0s linear .1s; } .c { transition: opacity,color .2s; } .d { animation: none; } .e { animation: fade; } .f { animation: fade .2s ease 0s 1 normal none running; } .g { transition: all; } .h { transition: opacity .2s; } .i { transition: all!important; } .j { animation: fade; }"#
    );
}

#[test]
fn execution_runtime_compresses_border_radius_shorthands() {
    let source = r#".a { border-radius: 1px 1px 1px 1px; border-top-left-radius: 1px; border-top-right-radius: 2px; border-bottom-right-radius: 1px; border-bottom-left-radius: 2px; } .b { border-radius: 1px / 2px; border-top-left-radius: 1px 2px; border-top-right-radius: 2px; border-bottom-right-radius: 1px; border-bottom-left-radius: 2px; } .c { border-radius: 1px 1px 1px 1px / 2px 2px 2px 2px; } .d { border-radius: 1px 2px 1px 2px / 3px 4px 3px 4px; } .e { border-radius: 1px / 1px; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 7);
    assert_eq!(
        execution.output_css,
        r#".a { border-radius: 1px; border-radius: 1px 2px; } .b { border-radius: 1px/2px; border-radius: 1px 2px/2px 2px 1px; } .c { border-radius: 1px/2px; } .d { border-radius: 1px 2px/3px 4px; } .e { border-radius: 1px; }"#
    );
}

#[test]
fn execution_runtime_compresses_inset_shorthands() {
    let source = r#".a { inset: 1px 2px 1px 2px; top: 1px; right: 2px; bottom: 1px; left: 2px; } .b { top: 1px; color: red; right: 2px; bottom: 1px; left: 2px; } .important { top: 1px !important; right: 2px !important; bottom: 1px !important; left: 2px !important; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#".a { inset: 1px 2px; inset: 1px 2px; } .b { top: 1px; color: red; right: 2px; bottom: 1px; left: 2px; } .important { top: 1px !important; right: 2px !important; bottom: 1px !important; left: 2px !important; }"#
    );
}

#[test]
fn execution_runtime_compresses_list_style_shorthands() {
    let source = r#".a { list-style: disc outside none; list-style-type: none; list-style-position: outside; list-style-image: none; } .b { list-style-type: decimal; list-style-position: inside; list-style-image: none; } .c { list-style-type: disc; color: red; list-style-position: outside; list-style-image: none; } .d { list-style: none outside none; } .e { list-style: url(icon.svg) outside none; } .f { list-style: NONE OUTSIDE NONE; } .important { list-style-type: none !important; list-style-position: outside !important; list-style-image: none !important; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 6);
    assert_eq!(
        execution.output_css,
        r#".a { list-style: outside; list-style: none; } .b { list-style: inside decimal; } .c { list-style-type: disc; color: red; list-style-position: outside; list-style-image: none; } .d { list-style: none; } .e { list-style: url(icon.svg) none; } .f { list-style: none; } .important { list-style-type: none !important; list-style-position: outside !important; list-style-image: none !important; }"#
    );
}

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

#[test]
fn execution_runtime_adds_conservative_vendor_prefixes_when_absent() {
    let source = r#".a { user-select: none; -webkit-appearance: none; appearance: none; backdrop-filter: blur(2px); } .flex { display: flex; position: sticky; } .inline { display: -webkit-inline-box; display: inline-flex; } .extra { text-size-adjust: 100%; mask-image: linear-gradient(red, blue); hyphens: auto; } .print { print-color-adjust: exact; -webkit-mask-size: cover; mask-size: cover; } @keyframes fade { from { opacity: 0; } to { opacity: 1; } } @-webkit-keyframes spin { from { opacity: 0; } } @keyframes spin { from { opacity: 0; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::VendorPrefixing,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 15);
    assert_eq!(
        execution.output_css,
        r#".a { -webkit-user-select: none; -moz-user-select: none; -ms-user-select: none; user-select: none; -webkit-appearance: none; -moz-appearance: none; appearance: none; -webkit-backdrop-filter: blur(2px); backdrop-filter: blur(2px); } .flex { display: -webkit-box; display: -ms-flexbox; display: flex; position: -webkit-sticky; position: sticky; } .inline { display: -webkit-inline-box; display: -ms-inline-flexbox; display: inline-flex; } .extra { -webkit-text-size-adjust: 100%; text-size-adjust: 100%; -webkit-mask-image: linear-gradient(red, blue); mask-image: linear-gradient(red, blue); -webkit-hyphens: auto; -ms-hyphens: auto; hyphens: auto; } .print { -webkit-print-color-adjust: exact; print-color-adjust: exact; -webkit-mask-size: cover; mask-size: cover; } @-webkit-keyframes fade { from { opacity: 0; } to { opacity: 1; } } @keyframes fade { from { opacity: 0; } to { opacity: 1; } } @-webkit-keyframes spin { from { opacity: 0; } } @keyframes spin { from { opacity: 0; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["vendor-prefixing", "print-css"]
    );
}

#[test]
fn execution_runtime_lowers_whole_value_light_dark_declarations() {
    let source = r#".card { color: light-dark(#000, #fff); background: linear-gradient(light-dark(red, blue), white); border: 1px solid light-dark(red, blue); box-shadow: 0 0 1px light-dark(black, white); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::LightDarkLowering,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        r#".card { color: #000; background: linear-gradient(red, white); border: 1px solid red; box-shadow: 0 0 1px black; } @media (prefers-color-scheme: dark) { .card { color: #fff; } } @media (prefers-color-scheme: dark) { .card { background: linear-gradient(blue, white); } } @media (prefers-color-scheme: dark) { .card { border: 1px solid blue; } } @media (prefers-color-scheme: dark) { .card { box-shadow: 0 0 1px white; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["light-dark-lowering", "print-css"]
    );
}

#[test]
fn execution_runtime_lowers_static_srgb_color_mix_declarations() {
    let source = r#".card { color: color-mix(in srgb, red 50%, blue 50%); background-color: color-mix(in srgb, #000, #fff 25%); outline-color: color-mix(in srgb, rgb(255 0 0) 25%, hsl(240 100% 50%) 75%); text-decoration-color: color-mix(in srgb, hwb(120 0% 50%) 40%, white 60%); caret-color: color-mix(in srgb, black 12.5%, white 87.5%); background: linear-gradient(color-mix(in srgb, red 25%, blue 75%), white); accent-color: color-mix(in srgb, red 25%, blue 25%); fill: color-mix(in srgb, red 75%, blue 75%); stroke: color-mix(in srgb, red 0%, blue 0%); border: 1px solid color-mix(in srgb, red, blue); box-shadow: 0 0 1px color-mix(in srgb, red, blue); column-rule: 1px solid color-mix(in srgb, red, blue); border-color: color-mix(in oklab, red, blue); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ColorMixLowering,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 11);
    assert_eq!(
        execution.output_css,
        r#".card { color: rgb(128 0 128); background-color: rgb(64 64 64); outline-color: rgb(64 0 191); text-decoration-color: rgb(153 204 153); caret-color: rgb(223 223 223); background: linear-gradient(rgb(64 0 191), white); accent-color: rgb(128 0 128 / .5); fill: rgb(128 0 128); stroke: color-mix(in srgb, red 0%, blue 0%); border: 1px solid rgb(128 0 128); box-shadow: 0 0 1px rgb(128 0 128); column-rule: 1px solid rgb(128 0 128); border-color: color-mix(in oklab, red, blue); }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["color-mix-lowering", "print-css"]
    );
}

#[test]
fn execution_runtime_lowers_alpha_aware_srgb_color_mix_declarations() {
    let source = r#".card { color: color-mix(in srgb, 50% red, transparent 50%); background-color: color-mix(in srgb, 25% rgb(100% 0% 0% / .7), rgb(0% 100% 0% / .2)); outline-color: color-mix(in srgb, rgb(100% 0% 0% / .7) 20%, 60% rgb(0% 100% 0% / .2)); border-color: color-mix(in srgb, 50% #ff000080, 50% blue); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ColorMixLowering,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        r#".card { color: rgb(255 0 0 / .5); background-color: rgb(137 118 0 / .325); outline-color: rgb(137 118 0 / .26); border-color: rgb(85 0 170 / .75098); }"#
    );
}

#[test]
fn execution_runtime_lowers_linear_srgb_color_mix_declarations() {
    let source = r#".card { color: color-mix(in srgb-linear, red 50%, blue 50%); background-color: color-mix(in srgb-linear, 50% red, transparent 50%); outline-color: color-mix(in srgb-linear, 25% rgb(100% 0% 0% / .7), rgb(0% 100% 0% / .2)); border-color: color-mix(in srgb-linear, 50% #ff000080, 50% blue); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ColorMixLowering,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        r#".card { color: rgb(188 0 188); background-color: rgb(255 0 0 / .5); outline-color: rgb(194 181 0 / .325); border-color: rgb(156 0 213 / .75098); }"#
    );
}

#[test]
fn execution_runtime_lowers_in_gamut_oklab_oklch_declarations() {
    let source = r#".card { color: oklab(1 0 0); background-color: oklch(0% 0 0deg); outline-color: oklch(0% 0 0.5TURN); background: linear-gradient(oklch(0% 0 0deg), white); accent-color: oklch(0% 0 0deg / .5); box-shadow: 0 0 1px oklch(0% 0 0deg); column-rule: 1px solid oklab(1 0 0); border-color: oklch(70% 0.4 40deg); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::OklchOklabLowering,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 7);
    assert_eq!(
        execution.output_css,
        r#".card { color: rgb(255 255 255); background-color: rgb(0 0 0); outline-color: rgb(0 0 0); background: linear-gradient(rgb(0 0 0), white); accent-color: rgb(0 0 0 / .5); box-shadow: 0 0 1px rgb(0 0 0); column-rule: 1px solid rgb(255 255 255); border-color: oklch(70% 0.4 40deg); }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["oklch-oklab-lowering", "print-css"]
    );
}

#[test]
fn execution_runtime_lowers_static_srgb_color_function_declarations() {
    let source = r#".card { color: color(srgb 1 0 0); background-color: color(srgb 50% 25% 0% / 100%); outline-color: color(srgb 0 0 1 / 1); fill: color(display-p3 0.5 0.5 0.5 / 100%); background: linear-gradient(color(srgb 1 0 0), white); accent-color: color(srgb 1 0 0 / .5); box-shadow: 0 0 1px color(srgb 0 0 1); column-rule: 1px solid color(srgb 1 0 0); text-shadow: 0 0 1px color(srgb-linear 0.5 0 0.5); scrollbar-color: color(srgb-linear 1 0 0 / 50%) white; border-color: color(display-p3 1 0 0); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ColorFunctionLowering,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 10);
    assert_eq!(
        execution.output_css,
        r#".card { color: rgb(255 0 0); background-color: rgb(128 64 0); outline-color: rgb(0 0 255); fill: rgb(128 128 128); background: linear-gradient(rgb(255 0 0), white); accent-color: rgb(255 0 0 / .5); box-shadow: 0 0 1px rgb(0 0 255); column-rule: 1px solid rgb(255 0 0); text-shadow: 0 0 1px rgb(188 0 188); scrollbar-color: rgb(255 0 0 / .5) white; border-color: color(display-p3 1 0 0); }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["color-function-lowering", "print-css"]
    );
}

#[test]
fn execution_runtime_lowers_logical_properties_only_with_static_direction() {
    let source = r#".ltr { direction: ltr; margin-inline-start: 1px; padding-inline-end: 2px; inline-size: 10rem; margin-inline: 1px 2px; padding-inline: calc(1rem + 1px) 3px; border-inline-color: red blue; margin-block: 4px 5px; padding-block-start: 6px; border-block-color: green yellow; border-block: 1px solid blue; inset-block-end: 7px; border-start-start-radius: 1px; border-start-end-radius: 2px; border-end-start-radius: 3px; border-end-end-radius: 4px; } .unknown { margin-inline-start: 1px; } .rtl { direction: rtl; writing-mode: horizontal-tb; inset-inline-start: 3px; border-inline-end-color: red; inset-inline: 4px 5px; border-inline: 1px solid red; border-inline-start: 2px dashed blue; border-start-start-radius: 5px; border-start-end-radius: 6px; border-end-start-radius: 7px; border-end-end-radius: 8px; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::LogicalToPhysical,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 24);
    assert_eq!(
        execution.output_css,
        r#".ltr { direction: ltr; margin-left: 1px; padding-right: 2px; width: 10rem; margin-left: 1px; margin-right: 2px; padding-left: calc(1rem + 1px); padding-right: 3px; border-left-color: red; border-right-color: blue; margin-top: 4px; margin-bottom: 5px; padding-top: 6px; border-top-color: green; border-bottom-color: yellow; border-top: 1px solid blue; border-bottom: 1px solid blue; bottom: 7px; border-top-left-radius: 1px; border-top-right-radius: 2px; border-bottom-left-radius: 3px; border-bottom-right-radius: 4px; } .unknown { margin-inline-start: 1px; } .rtl { direction: rtl; writing-mode: horizontal-tb; right: 3px; border-left-color: red; right: 4px; left: 5px; border-right: 1px solid red; border-left: 1px solid red; border-right: 2px dashed blue; border-top-right-radius: 5px; border-top-left-radius: 6px; border-bottom-right-radius: 7px; border-bottom-left-radius: 8px; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["logical-to-physical", "print-css"]
    );
}

#[test]
fn execution_runtime_lowers_vertical_logical_properties_with_static_axes() {
    let source = r#".vrl { writing-mode: vertical-rl; direction: ltr; margin-block-start: 1px; margin-block-end: 2px; margin-inline-start: 3px; margin-inline-end: 4px; block-size: 10px; inline-size: 20px; border-start-start-radius: 1px; border-end-end-radius: 2px; inset-block: 5px 6px; padding-inline: 7px 8px; } .vlr-rtl { writing-mode: vertical-lr; direction: rtl; inset-inline-start: 9px; border-start-end-radius: 3px; } .sideways { writing-mode: sideways-rl; direction: ltr; margin-inline-start: 1px; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::LogicalToPhysical,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 12);
    assert_eq!(
        execution.output_css,
        r#".vrl { writing-mode: vertical-rl; direction: ltr; margin-right: 1px; margin-left: 2px; margin-top: 3px; margin-bottom: 4px; width: 10px; height: 20px; border-top-right-radius: 1px; border-bottom-left-radius: 2px; right: 5px; left: 6px; padding-top: 7px; padding-bottom: 8px; } .vlr-rtl { writing-mode: vertical-lr; direction: rtl; bottom: 9px; border-top-left-radius: 3px; } .sideways { writing-mode: sideways-rl; direction: ltr; margin-inline-start: 1px; }"#
    );
}

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
    assert_eq!(execution.cascade_proof_obligations.obligation_count, 1);
    assert_eq!(execution.cascade_proof_obligations.accepted_count, 1);
    assert_eq!(
        execution.cascade_proof_obligations.obligations[0].proof_product,
        "omena-cascade.layer-flatten-proof"
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
