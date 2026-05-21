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
mod rule_optimization;
mod runtime_boundary;
mod scalar_normalization;
mod selector_structure;
mod shorthand_combining;
mod static_conditionals;
mod static_resolution;
mod token_normalization;
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
