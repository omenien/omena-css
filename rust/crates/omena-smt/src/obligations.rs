use omena_cascade::{
    BoxLonghandInputV0, LayerFlattenInputV0, LonghandMergeInputV0, ScopeFlattenInputV0,
    StaticSupportsAssumptionV0, StaticSupportsEvalVerdictV0, evaluate_static_supports_condition,
    prove_box_shorthand_combination, prove_layer_flatten_candidate, prove_longhand_merge,
    prove_scope_flatten_candidate,
};

use crate::{
    CanonicalSmtInputV0, CascadeSMTProofV0, SmtBackendV0, canonical_smt_input_v0,
    proof::cascade_smt_proof_v0,
};

pub fn smt_prove_box_shorthand_combination_v0<B: SmtBackendV0>(
    shorthand_property: &str,
    longhands: &[BoxLonghandInputV0],
    backend: &B,
) -> CascadeSMTProofV0 {
    let proof = prove_box_shorthand_combination(shorthand_property, longhands);
    let canonical_input =
        canonical_box_shorthand_combination_input_v0(shorthand_property, longhands);
    cascade_smt_proof_v0(
        canonical_input,
        backend,
        "prove_box_shorthand_combination",
        Some(proof.accepted),
    )
}

pub fn smt_prove_longhand_merge_v0<B: SmtBackendV0>(
    shorthand_property: &str,
    expected_longhands: &[&str],
    longhands: &[LonghandMergeInputV0],
    backend: &B,
) -> CascadeSMTProofV0 {
    let proof = prove_longhand_merge(shorthand_property, expected_longhands, longhands);
    let canonical_input =
        canonical_longhand_merge_input_v0(shorthand_property, expected_longhands, longhands);
    cascade_smt_proof_v0(
        canonical_input,
        backend,
        "prove_longhand_merge",
        Some(proof.accepted),
    )
}

pub fn smt_prove_scope_flatten_candidate_v0<B: SmtBackendV0>(
    input: ScopeFlattenInputV0,
    backend: &B,
) -> CascadeSMTProofV0 {
    let canonical_input = canonical_scope_flatten_candidate_input_v0(&input);
    let proof = prove_scope_flatten_candidate(input);
    cascade_smt_proof_v0(
        canonical_input,
        backend,
        "prove_scope_flatten_candidate",
        Some(proof.accepted),
    )
}

pub fn smt_prove_layer_flatten_candidate_v0<B: SmtBackendV0>(
    input: LayerFlattenInputV0,
    backend: &B,
) -> CascadeSMTProofV0 {
    let canonical_input = canonical_layer_flatten_candidate_input_v0(&input);
    let proof = prove_layer_flatten_candidate(input);
    cascade_smt_proof_v0(
        canonical_input,
        backend,
        "prove_layer_flatten_candidate",
        Some(proof.accepted),
    )
}

pub fn smt_evaluate_static_supports_condition_v0<B: SmtBackendV0>(
    condition: &str,
    assumption: StaticSupportsAssumptionV0,
    backend: &B,
) -> CascadeSMTProofV0 {
    let witness = evaluate_static_supports_condition(condition, assumption);
    let l1_accepted = match witness.verdict {
        StaticSupportsEvalVerdictV0::AlwaysTrue => Some(true),
        StaticSupportsEvalVerdictV0::AlwaysFalse => Some(false),
        StaticSupportsEvalVerdictV0::Unknown => None,
    };
    cascade_smt_proof_v0(
        canonical_static_supports_condition_input_v0(&witness.verdict),
        backend,
        "evaluate_static_supports_condition",
        l1_accepted,
    )
}

fn canonical_box_shorthand_combination_input_v0(
    shorthand_property: &str,
    longhands: &[BoxLonghandInputV0],
) -> CanonicalSmtInputV0 {
    let expected = smt_box_shorthand_longhands_v0(shorthand_property);
    let canonical_order = expected.is_some_and(|expected| {
        longhands.len() == expected.len()
            && longhands
                .iter()
                .zip(expected.iter())
                .all(|(actual, expected)| actual.property == *expected)
    });
    canonical_smt_input_v0(
        "box-shorthand-combination",
        "prove_box_shorthand_combination",
        vec![
            smt_require_term_v0("supported-shorthand-property", expected.is_some()),
            smt_require_term_v0("canonical-longhand-quartet", canonical_order),
            smt_require_term_v0(
                "no-important-longhand",
                longhands.iter().all(|longhand| !longhand.important),
            ),
            smt_require_term_v0(
                "no-empty-longhand-value",
                longhands.iter().all(|longhand| !longhand.value.is_empty()),
            ),
            smt_require_term_v0(
                "adjacent-source-order",
                longhands
                    .windows(2)
                    .all(|pair| pair[1].source_order == pair[0].source_order + 1),
            ),
        ],
    )
}

fn canonical_longhand_merge_input_v0(
    shorthand_property: &str,
    expected_longhands: &[&str],
    longhands: &[LonghandMergeInputV0],
) -> CanonicalSmtInputV0 {
    let canonical_order = !expected_longhands.is_empty()
        && longhands.len() == expected_longhands.len()
        && longhands
            .iter()
            .zip(expected_longhands.iter())
            .all(|(actual, expected)| actual.property == *expected);
    canonical_smt_input_v0(
        "longhand-merge",
        "prove_longhand_merge",
        vec![
            smt_require_term_v0("supported-merge-family", !expected_longhands.is_empty()),
            smt_require_term_v0("canonical-longhand-order", canonical_order),
            smt_require_term_v0(
                "no-important-longhand",
                longhands.iter().all(|longhand| !longhand.important),
            ),
            smt_require_term_v0(
                "no-empty-longhand-value",
                longhands.iter().all(|longhand| !longhand.value.is_empty()),
            ),
            smt_require_term_v0(
                "adjacent-source-order",
                longhands
                    .windows(2)
                    .all(|pair| pair[1].source_order == pair[0].source_order + 1),
            ),
            format!("merge-family:{shorthand_property}"),
        ],
    )
}

fn canonical_scope_flatten_candidate_input_v0(input: &ScopeFlattenInputV0) -> CanonicalSmtInputV0 {
    canonical_smt_input_v0(
        "scope-flatten-candidate",
        "prove_scope_flatten_candidate",
        vec![
            smt_require_term_v0("no-limit-selector", input.limit_selector.is_none()),
            smt_require_term_v0("root-scope", input.root_selector.trim() == ":root"),
            smt_require_term_v0("no-peer-scope", input.peer_scope_count == 0),
            smt_require_term_v0(
                "no-competing-unscoped-rule",
                input.competing_unscoped_rule_count == 0,
            ),
            smt_require_term_v0("not-inside-layer", !input.inside_layer),
        ],
    )
}

fn canonical_layer_flatten_candidate_input_v0(input: &LayerFlattenInputV0) -> CanonicalSmtInputV0 {
    canonical_smt_input_v0(
        "layer-flatten-candidate",
        "prove_layer_flatten_candidate",
        vec![
            smt_require_term_v0("closed-bundle", input.closed_bundle),
            smt_require_term_v0("no-peer-layer", input.peer_layer_count == 0),
            smt_require_term_v0("no-unlayered-rule", input.unlayered_rule_count == 0),
            smt_require_term_v0(
                "no-important-declaration",
                input.important_declaration_count == 0,
            ),
        ],
    )
}

fn canonical_static_supports_condition_input_v0(
    verdict: &StaticSupportsEvalVerdictV0,
) -> CanonicalSmtInputV0 {
    let canonical_terms = match verdict {
        StaticSupportsEvalVerdictV0::AlwaysTrue => {
            vec![smt_require_term_v0("supports-condition-known-true", true)]
        }
        StaticSupportsEvalVerdictV0::AlwaysFalse => {
            vec![smt_require_term_v0("supports-condition-known-true", false)]
        }
        StaticSupportsEvalVerdictV0::Unknown => vec!["unknown:supports-condition".to_string()],
    };
    canonical_smt_input_v0(
        "static-supports-condition",
        "evaluate_static_supports_condition",
        canonical_terms,
    )
}

fn smt_require_term_v0(name: &str, value: bool) -> String {
    format!("require:{name}={value}")
}

fn smt_box_shorthand_longhands_v0(shorthand_property: &str) -> Option<[&'static str; 4]> {
    match shorthand_property {
        "margin" => Some(["margin-top", "margin-right", "margin-bottom", "margin-left"]),
        "padding" => Some([
            "padding-top",
            "padding-right",
            "padding-bottom",
            "padding-left",
        ]),
        "border-color" => Some([
            "border-top-color",
            "border-right-color",
            "border-bottom-color",
            "border-left-color",
        ]),
        "border-style" => Some([
            "border-top-style",
            "border-right-style",
            "border-bottom-style",
            "border-left-style",
        ]),
        "border-width" => Some([
            "border-top-width",
            "border-right-width",
            "border-bottom-width",
            "border-left-width",
        ]),
        "scroll-margin" => Some([
            "scroll-margin-top",
            "scroll-margin-right",
            "scroll-margin-bottom",
            "scroll-margin-left",
        ]),
        "scroll-padding" => Some([
            "scroll-padding-top",
            "scroll-padding-right",
            "scroll-padding-bottom",
            "scroll-padding-left",
        ]),
        _ => None,
    }
}
