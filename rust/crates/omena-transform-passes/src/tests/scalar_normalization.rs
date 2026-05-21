use super::execute_transform_passes_on_source;
use omena_transform_cst::TransformPassKind;

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
