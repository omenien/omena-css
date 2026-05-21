use super::execute_transform_passes_on_source;
use omena_transform_cst::TransformPassKind;

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
