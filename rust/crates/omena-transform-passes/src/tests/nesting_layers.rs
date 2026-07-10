use super::{
    TransformExecutionContextV0, execute_transform_passes_on_source,
    execute_transform_passes_on_source_with_closed_world_context,
};
use crate::TransformDecision;
use omena_cascade_proof::{
    DischargeLedgerLookupStatusV0, SmtBackendSatResultV0, SmtBackendV0, StubSmtBackendV0,
};
use omena_evidence_graph::GuaranteeFamilyV0;
use omena_parser::StyleDialect;
use omena_transform_cst::TransformPassKind;

#[test]
fn nesting_scope_and_layer_structural_paths_match_ir_transactions() -> Result<(), String> {
    let nesting_sources = [
        r#".card { color: red; & .title { color: blue; } &:hover { color: green; } }"#,
        r#"@media (min-width: 40rem) { .card { color: red; & .title { color: blue; } } }"#,
        r#".card { color: red; @nest .theme & { color: blue; & .title { color: green; } } }"#,
    ];
    for source in nesting_sources {
        let execution = execute_transform_passes_on_source_with_closed_world_context(
            source,
            StyleDialect::Css,
            &[TransformPassKind::NestingUnwrap],
            &TransformExecutionContextV0::default(),
        );
        let expected =
            crate::domains::nesting::unwrap_css_nesting_with_lexer(source, StyleDialect::Css);

        assert_eq!(
            (execution.output_css, execution.mutation_count),
            expected,
            "nesting executor path should match the legacy lexer oracle"
        );
        assert!(
            execution
                .structural_ir_transaction_telemetry
                .transaction_commit_count
                > 0
        );
    }

    let scope_source = r#"@scope (:root) { .card { color: red; } }"#;
    let scope_execution = execute_transform_passes_on_source_with_closed_world_context(
        scope_source,
        StyleDialect::Css,
        &[TransformPassKind::ScopeFlatten],
        &TransformExecutionContextV0::default(),
    );
    assert_eq!(
        (scope_execution.output_css, scope_execution.mutation_count),
        crate::domains::cascade_flatten::flatten_css_scopes_with_lexer(
            scope_source,
            StyleDialect::Css
        ),
        "scope executor path should match the legacy lexer oracle"
    );
    assert!(
        scope_execution
            .structural_ir_transaction_telemetry
            .transaction_commit_count
            > 0
    );

    let layer_source = r#"@layer theme { .card { color: red; } }"#;
    let layer_context = TransformExecutionContextV0 {
        ..TransformExecutionContextV0::default()
    };
    let layer_execution = execute_transform_passes_on_source_with_closed_world_context(
        layer_source,
        StyleDialect::Css,
        &[TransformPassKind::LayerFlatten],
        &layer_context,
    );
    assert_eq!(
        (layer_execution.output_css, layer_execution.mutation_count),
        crate::domains::cascade_flatten::flatten_css_layers_with_lexer(
            layer_source,
            StyleDialect::Css,
            true
        ),
        "layer executor path should match the legacy lexer oracle"
    );
    assert!(
        layer_execution
            .structural_ir_transaction_telemetry
            .transaction_commit_count
            > 0
    );
    Ok(())
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
        accepted.cascade_proof_obligations.obligations[0]
            .canonical_smt_input
            .as_ref()
            .is_some_and(|input| input.l1_primitive == "prove_scope_flatten_candidate")
    );
    let scope_decision = accepted
        .decisions
        .iter()
        .find(|decision| decision.compatibility_outcome().pass_id == "scope-flatten");
    assert!(matches!(
        scope_decision,
        Some(TransformDecision::Applied {
            discharge_evidence,
            ..
        }) if discharge_evidence.len() == 1
            && discharge_evidence[0].guarantee_family
                == GuaranteeFamilyV0::LedgerBackedObligationDischarge
            && !discharge_evidence[0].evidence_node_key.input_identity.is_empty()
            && discharge_evidence[0].ledger_cell_key.len() == 64
            && !discharge_evidence[0].boundedness_kind.is_empty()
    ));
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
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_closed_world_context(
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
    assert!(
        planned.cascade_proof_obligations.obligations[0]
            .canonical_smt_input
            .as_ref()
            .is_some_and(|input| input.l1_primitive == "prove_layer_flatten_candidate")
    );
    assert!(
        planned.cascade_proof_obligations.obligations[0]
            .discharge_ledger_lookup
            .as_ref()
            .is_some_and(|lookup| {
                lookup.status == DischargeLedgerLookupStatusV0::Missing
                    && lookup.floor_reason == Some("ledger cell is absent")
                    && !lookup.can_apply_family_stamp()
            })
    );
    assert_eq!(execution.cascade_proof_obligations.obligation_count, 1);
    assert_eq!(execution.cascade_proof_obligations.accepted_count, 1);
    assert_eq!(
        execution.cascade_proof_obligations.obligations[0].proof_product,
        "omena-cascade.layer-flatten-proof"
    );
    assert!(
        execution.cascade_proof_obligations.obligations[0]
            .canonical_smt_input
            .as_ref()
            .is_some_and(|input| input.l1_primitive == "prove_layer_flatten_candidate")
    );
}

#[test]
fn layer_flatten_inversion_obligation_tracks_co_matching_selector_pairs() {
    let context = TransformExecutionContextV0 {
        ..TransformExecutionContextV0::default()
    };
    let source = concat!(
        "@layer utilities, base; ",
        "@layer base { .btn { color: red; } } ",
        "@layer utilities { button.btn { color: blue; } }"
    );

    let execution = execute_transform_passes_on_source_with_closed_world_context(
        source,
        StyleDialect::Css,
        &[TransformPassKind::LayerFlatten, TransformPassKind::PrintCss],
        &context,
    );
    let inversion_obligations = execution
        .cascade_proof_obligations
        .obligations
        .iter()
        .filter(|obligation| {
            obligation.proof_product == "omena-cascade.layer-flatten-inversion-proof"
        })
        .collect::<Vec<_>>();

    assert_eq!(
        inversion_obligations.len(),
        1,
        "co-matching cross-layer selector pair must produce one inversion obligation: {inversion_obligations:?}"
    );
    let canonical_terms = inversion_obligations[0]
        .canonical_smt_input
        .as_ref()
        .map(|input| input.canonical_terms.as_slice())
        .unwrap_or(&[]);
    assert!(
        canonical_terms
            .iter()
            .any(|term| term.contains(".btn|color@")),
        "base selector declaration must reach the inversion obligation: {canonical_terms:?}"
    );
    assert!(
        canonical_terms
            .iter()
            .any(|term| term.contains("button.btn|color@")),
        "tag-qualified selector declaration must reach the inversion obligation: {canonical_terms:?}"
    );
}

#[test]
fn layer_flatten_inversion_obligation_keeps_maybe_co_matches_competing() {
    let context = TransformExecutionContextV0 {
        ..TransformExecutionContextV0::default()
    };
    let source = concat!(
        "@layer utilities, base; ",
        "@layer base { .btn .icon { color: red; } } ",
        "@layer utilities { .btn:is(.active) { color: blue; } }"
    );

    let execution = execute_transform_passes_on_source_with_closed_world_context(
        source,
        StyleDialect::Css,
        &[TransformPassKind::LayerFlatten, TransformPassKind::PrintCss],
        &context,
    );
    let inversion_obligations = execution
        .cascade_proof_obligations
        .obligations
        .iter()
        .filter(|obligation| {
            obligation.proof_product == "omena-cascade.layer-flatten-inversion-proof"
        })
        .collect::<Vec<_>>();

    assert_eq!(
        inversion_obligations.len(),
        1,
        "unsupported selector structure must stay possibly competing: {inversion_obligations:?}"
    );
}

/// Mechanism-depth guard: the layer-flatten obligation's `accepted` flag is the
/// SMT solver's sat verdict over the obligation's own canonical input, not an
/// independent L1 flag.
///
/// The two sources differ ONLY in the load-bearing peer-layer field: a single
/// closed-bundle layer makes every cascade-safety requirement `true`, so the
/// `StubSmtBackendV0` returns `Sat` and the obligation is accepted; adding a
/// peer layer flips `require:no-peer-layer` to `false`, the solver returns
/// `Unsat`, and the same obligation is rejected. Re-running the real backend on
/// the carried canonical input proves the recorded `accepted` is exactly the
/// solver verdict (`Sat` => accepted). Replacing the solver with a constant or
/// ignoring `sat_result` would break one of the two halves.
#[test]
fn layer_flatten_obligation_acceptance_tracks_smt_sat_result() {
    let context = TransformExecutionContextV0 {
        ..TransformExecutionContextV0::default()
    };
    let backend = StubSmtBackendV0::default();

    // Sat half: one closed-bundle layer with no peers => all requirements hold,
    // so the stub solver returns `Sat` and the obligation is accepted. The
    // assertion re-runs the real backend on the obligation's own carried
    // canonical input and proves `accepted` is exactly `sat_result == Sat`.
    let sat = execute_transform_passes_on_source_with_closed_world_context(
        r#"@layer theme { .card { color: red; } }"#,
        StyleDialect::Css,
        &[TransformPassKind::LayerFlatten, TransformPassKind::PrintCss],
        &context,
    );
    assert!(
        sat.cascade_proof_obligations
            .obligations
            .iter()
            .any(|obligation| {
                obligation.proof_product == "omena-cascade.layer-flatten-proof"
                    && obligation.accepted
                    && obligation.blocked_reason.is_none()
                    && obligation
                        .canonical_smt_input
                        .as_ref()
                        .is_some_and(|input| {
                            input
                                .canonical_terms
                                .iter()
                                .any(|term| term == "require:no-peer-layer=true")
                                && matches!(
                                    backend.check_canonical_input_v0(input).sat_result,
                                    SmtBackendSatResultV0::Sat
                                )
                        })
            })
    );
    assert_eq!(sat.output_css, r#".card { color: red; }"#);

    // Unsat half: a peer layer flips ONLY the no-peer-layer requirement to
    // `false`, the stub solver returns `Unsat`, and the same obligation is
    // rejected. If the solver result were ignored (constant accepted) this half
    // would fail.
    let unsat = execute_transform_passes_on_source_with_closed_world_context(
        r#"@layer theme { .card { color: red; } } @layer util { .btn { color: blue; } }"#,
        StyleDialect::Css,
        &[TransformPassKind::LayerFlatten, TransformPassKind::PrintCss],
        &context,
    );
    assert!(
        unsat
            .cascade_proof_obligations
            .obligations
            .iter()
            .any(|obligation| {
                obligation.proof_product == "omena-cascade.layer-flatten-proof"
                    && !obligation.accepted
                    && obligation.blocked_reason.is_some()
                    && obligation
                        .canonical_smt_input
                        .as_ref()
                        .is_some_and(|input| {
                            input
                                .canonical_terms
                                .iter()
                                .any(|term| term == "require:no-peer-layer=false")
                                && matches!(
                                    backend.check_canonical_input_v0(input).sat_result,
                                    SmtBackendSatResultV0::Unsat
                                )
                        })
            })
    );
    // The product mutation follows the solver: the rejected layer is preserved.
    assert_eq!(
        unsat.output_css,
        r#"@layer theme { .card { color: red; } } @layer util { .btn { color: blue; } }"#
    );
    assert_eq!(unsat.cascade_proof_obligations.accepted_count, 0);
}

/// The cross-layer flatten inversion obligation is the real z3 search, not a
/// local flag.
///
/// Both inputs declare `.card { color }` in two `@layer` blocks whose source
/// order is identical; the ONLY byte that differs is the `@layer …, …;`
/// pre-declaration that fixes layer precedence. When precedence (`utilities,
/// base` => `base` wins) disagrees with source order (`utilities` block is last,
/// so it wins after flattening), the layered and flattened winners diverge and
/// z3 proves the QF_LIA inversion `Sat` => the flatten is blocked. Restoring the
/// precedence to match source order (`base, utilities`) makes z3 prove `Unsat`
/// => the flatten is accepted.
///
/// Litmus: replace z3 with a constant/identity backend and the verdict can no
/// longer flip on the pre-declaration order — the propositional stub returns
/// `Sat` for *both* inputs (it cannot model integer ordering), so the safe-case
/// acceptance below is only reachable because z3 actually solves the search. No
/// literal inversion flag is fed; the discriminating `(layer_rank, source_order)`
/// pairs are read from the real token stream.
#[cfg(feature = "smt-z3")]
#[test]
fn cross_layer_flatten_inversion_obligation_tracks_z3_verdict() {
    let context = TransformExecutionContextV0 {
        ..TransformExecutionContextV0::default()
    };

    let find_inversion_obligation = |execution: &crate::TransformExecutionSummaryV0| {
        let obligation = execution
            .cascade_proof_obligations
            .obligations
            .iter()
            .find(|obligation| {
                obligation.proof_product == "omena-cascade.layer-flatten-inversion-proof"
            })
            .cloned();
        assert!(
            obligation.is_some(),
            "multi-layer bundle must emit a cross-layer inversion obligation"
        );
        obligation.unwrap_or_else(|| unreachable!())
    };

    // Inverted: pre-declaration `utilities, base` makes `base` (declared first in
    // source) win the cascade, but flattening hands the win to the later
    // `utilities` block — the winners diverge, so z3 proves the inversion `Sat`.
    let inverted_source = concat!(
        "@layer utilities, base; ",
        "@layer base { .card { color: red; } } ",
        "@layer utilities { .card { color: blue; } }"
    );
    let inverted = execute_transform_passes_on_source_with_closed_world_context(
        inverted_source,
        StyleDialect::Css,
        &[TransformPassKind::LayerFlatten, TransformPassKind::PrintCss],
        &context,
    );
    let inverted_obligation = find_inversion_obligation(&inverted);
    assert!(
        !inverted_obligation.accepted,
        "z3 must reject the flatten when a cross-layer ordering inversion exists"
    );
    assert_eq!(
        inverted_obligation.blocked_reason.as_deref(),
        Some(
            "smt solver found a cross-layer cascade-ordering inversion: flattening would change the winning declaration"
        )
    );

    // Safe: pre-declaration `base, utilities` makes precedence agree with source
    // order, so the layered and flattened winners coincide and z3 proves `Unsat`.
    // The ONLY difference from `inverted_source` is the pre-declaration order.
    let safe_source = concat!(
        "@layer base, utilities; ",
        "@layer base { .card { color: red; } } ",
        "@layer utilities { .card { color: blue; } }"
    );
    let safe = execute_transform_passes_on_source_with_closed_world_context(
        safe_source,
        StyleDialect::Css,
        &[TransformPassKind::LayerFlatten, TransformPassKind::PrintCss],
        &context,
    );
    let safe_obligation = find_inversion_obligation(&safe);
    assert!(
        safe_obligation.accepted,
        "z3 must accept the flatten when no cross-layer ordering inverts"
    );
    assert!(safe_obligation.blocked_reason.is_none());

    // The verdict genuinely flipped on the single differing pre-declaration line.
    assert_ne!(inverted_obligation.accepted, safe_obligation.accepted);
}
