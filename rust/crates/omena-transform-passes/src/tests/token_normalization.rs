use super::execute_transform_passes_on_source;
use omena_transform_cst::TransformPassKind;

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
