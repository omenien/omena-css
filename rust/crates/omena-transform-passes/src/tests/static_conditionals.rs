use super::{execute_transform_passes_on_source, execute_transform_passes_on_source_with_dialect};
use omena_parser::StyleDialect;
use omena_transform_cst::TransformPassKind;

#[test]
fn execution_runtime_evaluates_literal_media_branches() {
    let source = r#"@media all { .a { color: red; } } @media not all { .b { color: blue; } } @media (max-width: 0px) { .zero { color: red; } } @media not (max-width: 0px) { .not-zero { color: lime; } } @media not all and (max-width: 0px) { .not-impossible { color: teal; } } @media all and (max-width: 0px) { .dead-and { color: red; } } @media (min-width: 10px) and (max-width: 5px) { .impossible { color: red; } } @media (min-width: calc(4px + 4px)) and (max-width: 5px) { .impossible-calc { color: red; } } @media not all, (max-width: 0px) { .dead-list { color: blue; } } @media all, screen { .list-true { color: purple; } } @media screen, (max-width: 0px) { .unknown-list { color: orange; } } @media screen { .c { color: green; } } @supports (display: grid) { @media all { @media all { .d { color: black; } } } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::MediaStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 13);
    assert_eq!(
        execution.output_css,
        r#".a { color: red; }   .not-zero { color: lime; } .not-impossible { color: teal; }     .list-true { color: purple; } @media screen, (width<=0px) { .unknown-list { color: orange; } } @media screen { .c { color: green; } } @supports (display: grid) { .d { color: black; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["media-static-eval", "print-css"]
    );
}

#[test]
fn execution_runtime_evaluates_only_media_modifier() {
    let source = r#"@media only all { .a { color: red; } } @media only screen and (max-width: 0px) { .dead { color: blue; } } @media only screen { .unknown { color: green; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::MediaStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#".a { color: red; }  @media only screen { .unknown { color: green; } }"#
    );
}

#[test]
fn execution_runtime_evaluates_media_or_disjunctions() {
    let source = r#"@media (max-width: 0px) or all { .live { color: red; } } @media (max-width: 0px) or (height<=0px) { .dead { color: blue; } } @media screen or (max-width: 0px) { .unknown { color: green; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::MediaStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 3);
    assert_eq!(
        execution.output_css,
        r#".live { color: red; }  @media screen or (width<=0px) { .unknown { color: green; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["media-static-eval", "print-css"]
    );
}

#[test]
fn execution_runtime_evaluates_strict_media_range_comparisons() {
    let source = r#"@media (width > 10px) and (width < 5px) { .dead { color: red; } } @media (width > 10px) and (width < 10px) { .dead-strict { color: blue; } } @media (10px <= width) and (width <= 10px) { .maybe-point { color: green; } } @media (height < 0px) { .negative { color: orange; } } @media (0px < width) { .live { color: purple; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::MediaStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 6);
    assert_eq!(
        execution.output_css,
        r#"  @media (width>=10px) and (width<=10px) { .maybe-point { color: green; } }  @media (width>0px) { .live { color: purple; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["media-static-eval", "print-css"]
    );
}

#[test]
fn execution_runtime_evaluates_chained_media_range_comparisons() {
    let source = r#"@media (400px < width < 800px) { .fluid { color: red; } } @media (800px < width < 400px) { .dead { color: blue; } } @media (10px <= width <= 10px) { .point { color: green; } } @media (10px < width <= 10px) { .dead-strict { color: orange; } } @media (100px > height > 20px) { .reverse { color: purple; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::MediaStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 7);
    assert_eq!(
        execution.output_css,
        r#"@media (width>400px) and (width<800px) { .fluid { color: red; } }  @media (width>=10px) and (width<=10px) { .point { color: green; } }  @media (height<100px) and (height>20px) { .reverse { color: purple; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["media-static-eval", "print-css"]
    );
}

#[test]
fn execution_runtime_evaluates_media_range_equality_comparisons() {
    let source = r#"@media (width = 10px) { .point { color: red; } } @media (10px = height) { .reverse-point { color: blue; } } @media (width = 10px) and (width > 10px) { .dead-high { color: green; } } @media (height = 20px) and (height < 20px) { .dead-low { color: orange; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::MediaStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        r#"@media (width=10px) { .point { color: red; } } @media (height=10px) { .reverse-point { color: blue; } }  "#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["media-static-eval", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_simple_media_range_features() {
    let source = r#"@media screen and (min-width: 1px) and (max-width: 10px) { .a { color: red; } } @media (min-height: 2rem) { .b { color: blue; } } @media (min-width: calc(1px + 1px)) { .c { color: green; } } @media (max-height: clamp(1rem, 2rem, 3rem)) { .d { color: orange; } } @media (WIDTH >= 1PX) { .e { color: black; } } @media (10REM <= HEIGHT <= 20REM) { .f { color: gray; } } @media (min-width: +01PX) { .g { color: white; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::MediaStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 7);
    assert_eq!(
        execution.output_css,
        r#"@media screen and (width>=1px) and (width<=10px) { .a { color: red; } } @media (height>=2rem) { .b { color: blue; } } @media (width>=2px) { .c { color: green; } } @media (height<=2rem) { .d { color: orange; } } @media (width>=1px) { .e { color: black; } } @media (height>=10rem) and (height<=20rem) { .f { color: gray; } } @media (width>=1px) { .g { color: white; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["media-static-eval", "print-css"]
    );
}

#[test]
fn execution_runtime_evaluates_simple_supports_branches_with_cascade_witness() {
    let source = r#"@supports (display: grid) { .a { display: grid; } } @supports not (display: grid) { .b { display: block; } } @supports (display: grid) and (color: red) { .c { color: red; } } @supports (display: grid) or (selector(:has(*))) { .or { display: grid; } } @supports ((display: grid) or (display: -ms-grid)) and (color: red) { .grouped { display: grid; } } @supports not ((display: -ms-grid) or (-ms-ime-align: auto)) { .not-grouped { display: grid; } } @supports not ((display: grid) or (display: -ms-grid)) { .not-dead { display: grid; } } @media all { @supports (display: grid) { @supports (display: grid) { .d { display: grid; } } } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::SupportsStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 9);
    assert_eq!(
        execution.output_css,
        r#".a { display: grid; }  .c { color: red; } .or { display: grid; } .grouped { display: grid; } .not-grouped { display: grid; }  @media all { .d { display: grid; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["supports-static-eval", "print-css"]
    );
    assert_eq!(execution.cascade_proof_obligations.obligation_count, 8);
    assert_eq!(execution.cascade_proof_obligations.accepted_count, 8);
    assert!(
        execution
            .cascade_proof_obligations
            .obligations
            .iter()
            .any(|obligation| {
                obligation.proof_product == "omena-cascade.supports-static-eval"
                    && obligation.accepted
                    && obligation
                        .canonical_smt_input
                        .as_ref()
                        .is_some_and(|input| {
                            input.product == "omena-smt.canonical-input"
                                && input.l1_primitive == "evaluate_static_supports_condition"
                        })
                    && obligation
                        .checked_obligations
                        .contains(&"staticSupportsCondition")
            })
    );
}

#[test]
fn execution_runtime_evaluates_case_insensitive_supports_conditions() {
    let source = r#"@supports NOT (display: -MS-grid) { .ok { display: grid; } } @supports SELECTOR(:-MS-input-placeholder) { .dead { color: red; } } @supports FONT-TECH(COLOR-COLRv1) OR (display: -ms-grid) { .font { color: blue; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::SupportsStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 3);
    assert_eq!(
        execution.output_css,
        r#".ok { display: grid; }  .font { color: blue; }"#
    );
}

#[test]
fn execution_runtime_evaluates_selector_supports_branches_with_cascade_witness() {
    let source = r#"@supports selector(:has(*)) { .has { color: red; } } @supports not selector(:has(*)) { .not-has { color: blue; } } @supports selector(:-ms-input-placeholder) { .ms { color: green; } } @supports not selector(:-ms-input-placeholder) { .not-ms { color: purple; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::SupportsStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        r#".has { color: red; }   .not-ms { color: purple; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["supports-static-eval", "print-css"]
    );
}

#[test]
fn execution_runtime_evaluates_function_value_supports_branches_with_cascade_witness() {
    let source = r#"@supports (color: color(display-p3 1 0 0)) { .p3 { color: red; } } @supports not (background-image: linear-gradient(red, blue)) { .not-gradient { color: blue; } } @supports (width: min(10px, 20px)) and (display: grid) { .math { color: green; } } @supports (color: color(display-p3 1 0 0) { .malformed { color: orange; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::SupportsStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 3);
    assert_eq!(
        execution.output_css,
        r#".p3 { color: red; }  .math { color: green; } @supports (color: color(display-p3 1 0 0) { .malformed { color: orange; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["supports-static-eval", "print-css"]
    );
}

#[test]
fn execution_runtime_evaluates_font_supports_branches_with_cascade_witness() {
    let source = r#"@supports font-tech(color-COLRv1) { .color-font { color: red; } } @supports not font-format(woff2) { .not-woff2 { color: blue; } } @supports font-format(embedded-opentype) { .eot { color: green; } } @supports not font-tech(-ms-color) { .not-ms { color: purple; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::SupportsStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        r#".color-font { color: red; }   .not-ms { color: purple; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["supports-static-eval", "print-css"]
    );
}

#[test]
fn execution_runtime_folds_static_native_css_if_and_function_values() {
    let source = r#"@function --gap(--size <length>: 1rem) returns <length> { result: var(--size); } .card { gap: --gap(2rem); display: if(supports(display: grid): grid; else: block); margin: if(media(width >= 1px): 1rem; else: 2rem); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::NativeCssStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 2);
    assert!(execution.output_css.contains("gap: 2rem"));
    assert!(execution.output_css.contains("display: grid"));
    assert!(
        execution
            .output_css
            .contains("margin: if(media(width >= 1px): 1rem; else: 2rem)")
    );
    assert!(!execution.output_css.contains("--gap(2rem)"));
    assert!(!execution.output_css.contains("display: if(supports"));
    assert_eq!(
        execution.executed_pass_ids,
        vec!["native-css-static-eval", "print-css"]
    );
}

#[test]
fn execution_runtime_preserves_native_css_function_type_mismatches() {
    let source = r#"@function --tone(--size <length>) returns <color> { result: var(--size); } .card { color: --tone(2rem); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::NativeCssStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 0);
    assert_eq!(execution.output_css, source);
    assert_eq!(
        execution.executed_pass_ids,
        vec!["native-css-static-eval", "print-css"]
    );
}

#[test]
fn execution_runtime_preserves_native_css_function_cycles() {
    let source =
        r#"@function --loop() returns <length> { result: --loop(); } .card { width: --loop(); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::NativeCssStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 0);
    assert_eq!(execution.output_css, source);
    assert_eq!(
        execution.executed_pass_ids,
        vec!["native-css-static-eval", "print-css"]
    );
}

#[test]
fn execution_runtime_preserves_native_css_function_body_calls() {
    let source = r#"@function --inner() returns <length> { result: 1px; } @function --outer() returns <length> { result: --inner(); } .card { width: --inner(); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::NativeCssStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert!(execution.output_css.contains("result: --inner();"));
    assert!(execution.output_css.contains("width: 1px"));
    assert_eq!(
        execution.executed_pass_ids,
        vec!["native-css-static-eval", "print-css"]
    );
}

#[test]
fn execution_runtime_folds_static_native_css_when_rule_branch() {
    let source = r#"@when supports(display: grid) { .grid { display: grid; } } @else { .fallback { display: block; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::NativeCssStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert!(execution.output_css.contains(".grid { display: grid; }"));
    assert!(!execution.output_css.contains("@when"));
    assert!(!execution.output_css.contains(".fallback"));
    assert_eq!(
        execution.executed_pass_ids,
        vec!["native-css-static-eval", "print-css"]
    );
}

#[test]
fn execution_runtime_preserves_runtime_native_css_static_values() {
    let source = r#"@function --gap(--size <length>: 1rem) returns <length> { result: var(--size); } .card { gap: --gap(var(--space)); margin: if(media(width >= 1px): 1rem; else: 2rem); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::NativeCssStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 0);
    assert_eq!(execution.output_css, source);
    assert_eq!(
        execution.executed_pass_ids,
        vec!["native-css-static-eval", "print-css"]
    );
}

#[test]
fn execution_runtime_keeps_native_css_static_eval_css_dialect_only() {
    let source = r#".card { display: if(true, grid, block); }"#;
    let execution = execute_transform_passes_on_source_with_dialect(
        source,
        StyleDialect::Scss,
        &[
            TransformPassKind::NativeCssStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 0);
    assert_eq!(execution.output_css, source);
    assert_eq!(
        execution.executed_pass_ids,
        vec!["native-css-static-eval", "print-css"]
    );
}
