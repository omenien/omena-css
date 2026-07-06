use super::execute_transform_passes_on_source;
use omena_cascade_proof::DischargeLedgerLookupStatusV0;
use omena_transform_cst::TransformPassKind;

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
    assert_eq!(execution.cascade_proof_obligations.obligation_count, 3);
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
                        .canonical_smt_input
                        .as_ref()
                        .is_some_and(|input| {
                            input.product == "omena-smt.canonical-input"
                                && input.l1_primitive == "prove_longhand_merge"
                        })
                    && obligation
                        .checked_obligations
                        .contains(&"canonicalLonghandMergeSet")
                    && obligation
                        .discharge_ledger_lookup
                        .as_ref()
                        .is_some_and(|lookup| {
                            lookup.status == DischargeLedgerLookupStatusV0::Matched
                                && lookup.cell_family.as_deref() == Some("longhandMerge")
                                && lookup.can_apply_family_stamp()
                        })
            })
    );
}

#[test]
fn execution_summary_reports_deterministic_discharge_ledger_telemetry() {
    let source = r#".a { margin-top: 1px; margin-right: 2px; margin-bottom: 1px; margin-left: 2px; border-top-color: red; border-right-color: blue; border-bottom-color: red; border-left-color: blue; border-top-width: 1px; border-right-width: 2px; border-bottom-width: 3px; border-left-width: 2px; }"#;
    let snapshots = (0..3)
        .map(|_| {
            let execution = execute_transform_passes_on_source(
                source,
                &[
                    TransformPassKind::ShorthandCombining,
                    TransformPassKind::PrintCss,
                ],
            );
            assert_eq!(execution.discharge_ledger_telemetry.lookup_count, 3);
            assert_eq!(execution.discharge_ledger_telemetry.matched_lookup_count, 3);
            assert_eq!(execution.discharge_ledger_telemetry.accepted_stamp_count, 3);
            assert_eq!(execution.discharge_ledger_telemetry.blocked_lookup_count, 0);
            let snapshot = serde_json::to_string(&execution.discharge_ledger_telemetry);
            assert!(
                snapshot.is_ok(),
                "discharge ledger telemetry should serialize"
            );
            snapshot.unwrap_or_default()
        })
        .collect::<Vec<_>>();

    assert!(snapshots.windows(2).all(|pair| pair[0] == pair[1]));
}

#[test]
fn execution_runtime_emits_longhand_merge_witnesses_for_multiple_shorthand_families() {
    let source = r#".a { row-gap: 1px; column-gap: 2px; align-items: center; justify-items: stretch; flex-direction: row; flex-wrap: wrap; list-style-type: none; list-style-position: outside; list-style-image: none; background-image: url(hero.png); background-repeat: repeat no-repeat; background-color: red; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(
        execution.output_css,
        r#".a { gap: 1px 2px; place-items: center stretch; flex-flow: wrap; list-style: none; background: url(hero.png) repeat-x red; }"#
    );
    let families = execution
        .cascade_proof_obligations
        .obligations
        .iter()
        .filter(|obligation| obligation.accepted)
        .filter_map(|obligation| obligation.canonical_smt_input.as_ref())
        .flat_map(|input| input.canonical_terms.iter())
        .filter_map(|term| term.strip_prefix("merge-family:"))
        .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(
        families,
        std::collections::BTreeSet::from([
            "background",
            "flex-flow",
            "gap",
            "list-style",
            "place-items"
        ])
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
fn execution_runtime_compresses_box_shorthand_values_with_typed_zero_equality() {
    let source = r#".a { margin: 0px 0 0 0; padding: 0% 0 0 0; inset: 0deg 0 0 0; scroll-margin: 0em 0 0 0; }"#;
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
        r#".a { margin: 0; padding: 0% 0 0; inset: 0; scroll-margin: 0em 0 0; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["shorthand-combining", "print-css"]
    );
}

// Fence for the NON-ZERO box canonicalization (INV-2c). The emitted-CSS golden gate prints
// in Identity mode and never runs ShorthandCombining, so it cannot witness this; this
// pass-driven test exercises the real emit path (box_component_output -> canonicalize_*).
// Canonically-equal-but-byte-differ absolute peers (1.0px/+1px/1PX/01px == 1px) collapse to
// the canonical shorthand; context-relative (em), distinct (2px), and var() stay byte-faithful.
#[test]
fn execution_runtime_compresses_box_shorthand_values_with_typed_nonzero_equality() {
    let source = r#".a { margin: 1.0px 1px 1px 1px; padding: +1px 1px 1px 1px; inset: 1PX 1px 1px 1px; scroll-margin: 01px 1px 1px 1px; } .keep { margin: 1em 1px 2px 1px; padding: var(--x) 1px 1px 1px; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(
        execution.output_css,
        r#".a { margin: 1px; padding: 1px; inset: 1px; scroll-margin: 1px; } .keep { margin: 1em 1px 2px; padding: var(--x) 1px 1px; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["shorthand-combining", "print-css"]
    );
}

// Unit guard directly on the box collapser: canonically-equal non-zero absolute peers
// collapse to the canonical single value; unsound merges (context-relative / var) do not.
#[test]
fn compress_box_shorthand_values_collapses_canonically_equal_nonzero_peers() {
    use crate::domains::shorthand::compress_box_shorthand_values;
    assert_eq!(
        compress_box_shorthand_values(&["1.0px", "1px", "1px", "1px"]),
        Some("1px".to_string())
    );
    assert_eq!(
        compress_box_shorthand_values(&["+1px", "1px", "1px", "1px"]),
        Some("1px".to_string())
    );
    assert_eq!(
        compress_box_shorthand_values(&["1PX", "1px", "1px", "1px"]),
        Some("1px".to_string())
    );
    // context-relative + var must NOT canonically merge to a single value (allowlist
    // soundness): the standard left==right box rule may still 4->3 collapse, but the
    // first component stays distinct, so the result is never the all-equal "1px".
    assert_ne!(
        compress_box_shorthand_values(&["1em", "1px", "1px", "1px"]),
        Some("1px".to_string())
    );
    assert_ne!(
        compress_box_shorthand_values(&["var(--x)", "1px", "1px", "1px"]),
        Some("1px".to_string())
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
