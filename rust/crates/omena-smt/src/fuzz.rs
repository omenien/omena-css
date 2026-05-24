use omena_cascade::{
    BoxLonghandInputV0, LayerFlattenInputV0, ScopeFlattenInputV0, StaticSupportsAssumptionV0,
    StaticSupportsEvalVerdictV0, evaluate_static_supports_condition,
    prove_box_shorthand_combination, prove_layer_flatten_candidate, prove_scope_flatten_candidate,
};
use serde::Serialize;

use crate::{
    SMT_FEATURE_GATE_V0, SMT_LAYER_MARKER_V0, SMT_SCHEMA_VERSION_V0, SmtVerdictV0,
    StubSmtBackendV0, smt_evaluate_static_supports_condition_v0,
    smt_prove_box_shorthand_combination_v0, smt_prove_layer_flatten_candidate_v0,
    smt_prove_scope_flatten_candidate_v0,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SmtBisimulationFuzzCaseV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub seed: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SmtBisimulationFuzzReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub fixture_suite: &'static str,
    pub seed: u64,
    pub checked_obligation_count: usize,
    pub l1_l3_mismatch_count: usize,
    pub passed: bool,
}

pub fn smt_bisimulation_fuzz_case_v0(seed: u64) -> SmtBisimulationFuzzCaseV0 {
    SmtBisimulationFuzzCaseV0 {
        schema_version: SMT_SCHEMA_VERSION_V0,
        product: "omena-smt.bisimulation-fuzz-case",
        layer_marker: SMT_LAYER_MARKER_V0,
        feature_gate: SMT_FEATURE_GATE_V0,
        seed,
    }
}

pub fn run_smt_bisimulation_fuzz_case_v0(
    case: SmtBisimulationFuzzCaseV0,
) -> SmtBisimulationFuzzReportV0 {
    let backend = StubSmtBackendV0;
    let mut checked_obligation_count = 0;
    let mut l1_l3_mismatch_count = 0;

    let longhands = generated_box_longhands(case.seed);
    let shorthand_property = if case.seed.is_multiple_of(7) {
        "unsupported"
    } else if case.seed.is_multiple_of(3) {
        "padding"
    } else {
        "margin"
    };
    let l1_box = prove_box_shorthand_combination(shorthand_property, &longhands);
    let l3_box = smt_prove_box_shorthand_combination_v0(shorthand_property, &longhands, &backend);
    checked_obligation_count += 1;
    if l3_box.verdict != expected_verdict_from_l1_accepted(Some(l1_box.accepted)) {
        l1_l3_mismatch_count += 1;
    }

    let supports_condition = generated_supports_condition(case.seed);
    let l1_supports = evaluate_static_supports_condition(
        supports_condition,
        StaticSupportsAssumptionV0::ModernBrowser,
    );
    let l3_supports = smt_evaluate_static_supports_condition_v0(
        supports_condition,
        StaticSupportsAssumptionV0::ModernBrowser,
        &backend,
    );
    checked_obligation_count += 1;
    if l3_supports.verdict != expected_verdict_from_supports(l1_supports.verdict) {
        l1_l3_mismatch_count += 1;
    }

    let scope_input = generated_scope_input(case.seed);
    let l1_scope = prove_scope_flatten_candidate(scope_input.clone());
    let l3_scope = smt_prove_scope_flatten_candidate_v0(scope_input, &backend);
    checked_obligation_count += 1;
    if l3_scope.verdict != expected_verdict_from_l1_accepted(Some(l1_scope.accepted)) {
        l1_l3_mismatch_count += 1;
    }

    let layer_input = generated_layer_input(case.seed);
    let l1_layer = prove_layer_flatten_candidate(layer_input.clone());
    let l3_layer = smt_prove_layer_flatten_candidate_v0(layer_input, &backend);
    checked_obligation_count += 1;
    if l3_layer.verdict != expected_verdict_from_l1_accepted(Some(l1_layer.accepted)) {
        l1_l3_mismatch_count += 1;
    }

    SmtBisimulationFuzzReportV0 {
        schema_version: SMT_SCHEMA_VERSION_V0,
        product: "omena-smt.bisimulation-fuzz-report",
        layer_marker: SMT_LAYER_MARKER_V0,
        feature_gate: SMT_FEATURE_GATE_V0,
        fixture_suite: "m3-cascade-proof-fixtures",
        seed: case.seed,
        checked_obligation_count,
        l1_l3_mismatch_count,
        passed: l1_l3_mismatch_count == 0,
    }
}

pub fn run_smt_bisimulation_fuzz_seed_corpus_v0(case_count: usize) -> SmtBisimulationFuzzReportV0 {
    let mut checked_obligation_count = 0;
    let mut l1_l3_mismatch_count = 0;

    for index in 0..case_count {
        let report = run_smt_bisimulation_fuzz_case_v0(smt_bisimulation_fuzz_case_v0(stable_seed(
            index as u64,
        )));
        checked_obligation_count += report.checked_obligation_count;
        l1_l3_mismatch_count += report.l1_l3_mismatch_count;
    }

    SmtBisimulationFuzzReportV0 {
        schema_version: SMT_SCHEMA_VERSION_V0,
        product: "omena-smt.bisimulation-fuzz-seed-corpus",
        layer_marker: SMT_LAYER_MARKER_V0,
        feature_gate: SMT_FEATURE_GATE_V0,
        fixture_suite: "m3-cascade-proof-fixtures",
        seed: 0,
        checked_obligation_count,
        l1_l3_mismatch_count,
        passed: l1_l3_mismatch_count == 0,
    }
}

fn expected_verdict_from_l1_accepted(accepted: Option<bool>) -> SmtVerdictV0 {
    match accepted {
        Some(true) => SmtVerdictV0::Accepted,
        Some(false) => SmtVerdictV0::Rejected,
        None => SmtVerdictV0::Unknown,
    }
}

fn expected_verdict_from_supports(verdict: StaticSupportsEvalVerdictV0) -> SmtVerdictV0 {
    match verdict {
        StaticSupportsEvalVerdictV0::AlwaysTrue => SmtVerdictV0::Accepted,
        StaticSupportsEvalVerdictV0::AlwaysFalse => SmtVerdictV0::Rejected,
        StaticSupportsEvalVerdictV0::Unknown => SmtVerdictV0::Unknown,
    }
}

fn generated_box_longhands(seed: u64) -> Vec<BoxLonghandInputV0> {
    let shorthand = if seed.is_multiple_of(3) {
        "padding"
    } else {
        "margin"
    };
    let properties = [
        format!("{shorthand}-top"),
        format!("{shorthand}-right"),
        format!("{shorthand}-bottom"),
        format!("{shorthand}-left"),
    ];
    let maybe_permuted = if seed.is_multiple_of(5) {
        vec![1, 0, 2, 3]
    } else {
        vec![0, 1, 2, 3]
    };

    maybe_permuted
        .into_iter()
        .enumerate()
        .map(|(source_index, property_index)| BoxLonghandInputV0 {
            property: properties[property_index].clone(),
            value: if seed.is_multiple_of(11) {
                String::new()
            } else {
                format!("{}px", (seed + source_index as u64) % 17)
            },
            important: seed.is_multiple_of(13),
            source_order: if seed.is_multiple_of(17) {
                (source_index as u32) * 2 + 1
            } else {
                source_index as u32 + 1
            },
        })
        .collect()
}

fn generated_supports_condition(seed: u64) -> &'static str {
    const CONDITIONS: &[&str] = &[
        "(display: grid)",
        "(display: unknown-omena)",
        "selector(:has(*))",
        "not (display: grid)",
        "((display: grid) or (display: unknown-omena))",
        "((display: grid) and selector(:has(*)))",
        "font-tech(color-COLRv1)",
    ];
    CONDITIONS[(seed as usize) % CONDITIONS.len()]
}

fn generated_scope_input(seed: u64) -> ScopeFlattenInputV0 {
    ScopeFlattenInputV0 {
        root_selector: if seed.is_multiple_of(2) {
            ":root".to_string()
        } else {
            ".scope".to_string()
        },
        limit_selector: seed.is_multiple_of(3).then(|| ".limit".to_string()),
        scoped_rule_count: (seed % 4) as usize,
        peer_scope_count: (seed % 3) as usize,
        competing_unscoped_rule_count: (seed % 2) as usize,
        inside_layer: seed.is_multiple_of(5),
    }
}

fn generated_layer_input(seed: u64) -> LayerFlattenInputV0 {
    LayerFlattenInputV0 {
        layer_name: seed.is_multiple_of(2).then(|| "components".to_string()),
        layer_rule_count: (seed % 5) as usize,
        peer_layer_count: (seed % 3) as usize,
        unlayered_rule_count: (seed % 4) as usize,
        important_declaration_count: (seed % 2) as usize,
        closed_bundle: !seed.is_multiple_of(7),
    }
}

fn stable_seed(index: u64) -> u64 {
    index
        .wrapping_mul(0x9e37_79b9_7f4a_7c15)
        .wrapping_add(0xd1b5_4a32_d192_ed03)
}
