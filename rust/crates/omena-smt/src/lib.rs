//! SMT-encoded cascade verification contracts.
//!
//! The crate keeps the M4-gamma three-layer split explicit:
//! L1 cascade proof primitives remain in `omena-cascade`, L2 refinement
//! delegates live beside the cascade crate, and this crate owns L3 encoding,
//! proof contracts, backend selection, and unsat-core audit metadata.

pub mod backend;
pub mod encoder;
pub mod fuzz;
pub mod obligations;
pub mod proof;
pub mod theory;
pub mod unsat_core;

pub use backend::{SmtBackendKindV0, SmtBackendV0, StubSmtBackendV0};
pub use encoder::{CanonicalSmtInputV0, canonical_smt_input_v0};
pub use fuzz::{
    SmtBisimulationFuzzCaseV0, SmtBisimulationFuzzReportV0, run_smt_bisimulation_fuzz_case_v0,
    run_smt_bisimulation_fuzz_seed_corpus_v0, smt_bisimulation_fuzz_case_v0,
};
pub use obligations::{
    smt_evaluate_static_supports_condition_v0, smt_prove_box_shorthand_combination_v0,
    smt_prove_layer_flatten_candidate_v0, smt_prove_scope_flatten_candidate_v0,
};
pub use proof::{
    CascadeSMTProofAuditLogV0, CascadeSMTProofV0, SmtVerdictV0, cascade_spec_digest_v0,
};
pub use theory::{CascadeTheorySignatureV0, cascade_theory_signature_v0};
pub use unsat_core::{CascadeUnsatCoreLabelV0, cascade_unsat_core_label_v0};

pub const SMT_SCHEMA_VERSION_V0: &str = "0";
pub const SMT_LAYER_MARKER_V0: &str = "smt-cascade-verification";
pub const SMT_FEATURE_GATE_V0: &str = "smt-stub";

#[cfg(test)]
mod tests {
    use super::*;
    use omena_cascade::{
        BoxLonghandInputV0, LayerFlattenInputV0, ScopeFlattenInputV0, StaticSupportsAssumptionV0,
        StaticSupportsEvalVerdictV0, evaluate_static_supports_condition,
        prove_box_shorthand_combination, prove_layer_flatten_candidate,
        prove_scope_flatten_candidate,
    };

    fn accepted_verdict(accepted: bool) -> SmtVerdictV0 {
        if accepted {
            SmtVerdictV0::Accepted
        } else {
            SmtVerdictV0::Rejected
        }
    }

    #[test]
    fn default_stub_backend_matches_l1_proof_verdict() {
        let backend = StubSmtBackendV0;
        let proof = smt_prove_box_shorthand_combination_v0(
            "margin",
            &[
                BoxLonghandInputV0 {
                    property: "margin-top".to_string(),
                    value: "1px".to_string(),
                    important: false,
                    source_order: 1,
                },
                BoxLonghandInputV0 {
                    property: "margin-right".to_string(),
                    value: "1px".to_string(),
                    important: false,
                    source_order: 2,
                },
                BoxLonghandInputV0 {
                    property: "margin-bottom".to_string(),
                    value: "1px".to_string(),
                    important: false,
                    source_order: 3,
                },
                BoxLonghandInputV0 {
                    property: "margin-left".to_string(),
                    value: "1px".to_string(),
                    important: false,
                    source_order: 4,
                },
            ],
            &backend,
        );
        assert_eq!(proof.schema_version, "0");
        assert_eq!(proof.verdict, SmtVerdictV0::Accepted);
        assert_eq!(proof.backend, SmtBackendKindV0::Stub);
    }

    #[test]
    fn proof_style_bisimulation_invariant_holds_for_all_l1_primitives() {
        let backend = StubSmtBackendV0;
        let longhands = vec![
            BoxLonghandInputV0 {
                property: "margin-top".to_string(),
                value: "1px".to_string(),
                important: false,
                source_order: 1,
            },
            BoxLonghandInputV0 {
                property: "margin-right".to_string(),
                value: "1px".to_string(),
                important: false,
                source_order: 2,
            },
            BoxLonghandInputV0 {
                property: "margin-bottom".to_string(),
                value: "1px".to_string(),
                important: false,
                source_order: 3,
            },
            BoxLonghandInputV0 {
                property: "margin-left".to_string(),
                value: "1px".to_string(),
                important: false,
                source_order: 4,
            },
        ];
        let l1_box = prove_box_shorthand_combination("margin", &longhands);
        let l3_box = smt_prove_box_shorthand_combination_v0("margin", &longhands, &backend);
        assert_eq!(l3_box.verdict, accepted_verdict(l1_box.accepted));

        let scope_input = ScopeFlattenInputV0 {
            root_selector: ":root".to_string(),
            limit_selector: None,
            scoped_rule_count: 1,
            peer_scope_count: 0,
            competing_unscoped_rule_count: 0,
            inside_layer: false,
        };
        let l1_scope = prove_scope_flatten_candidate(scope_input.clone());
        let l3_scope = smt_prove_scope_flatten_candidate_v0(scope_input, &backend);
        assert_eq!(l3_scope.verdict, accepted_verdict(l1_scope.accepted));

        let layer_input = LayerFlattenInputV0 {
            layer_name: Some("components".to_string()),
            layer_rule_count: 1,
            peer_layer_count: 0,
            unlayered_rule_count: 0,
            important_declaration_count: 0,
            closed_bundle: true,
        };
        let l1_layer = prove_layer_flatten_candidate(layer_input.clone());
        let l3_layer = smt_prove_layer_flatten_candidate_v0(layer_input, &backend);
        assert_eq!(l3_layer.verdict, accepted_verdict(l1_layer.accepted));
    }

    #[test]
    fn static_supports_smt_equivalence_tracks_l1_verdict_shape() {
        let backend = StubSmtBackendV0;
        let l1 = evaluate_static_supports_condition(
            "(display: grid)",
            StaticSupportsAssumptionV0::ModernBrowser,
        );
        let l3 = smt_evaluate_static_supports_condition_v0(
            "(display: grid)",
            StaticSupportsAssumptionV0::ModernBrowser,
            &backend,
        );

        assert_eq!(l1.verdict, StaticSupportsEvalVerdictV0::AlwaysTrue);
        assert_eq!(l3.verdict, SmtVerdictV0::Accepted);
        assert_eq!(l3.l1_primitive, "evaluate_static_supports_condition");
    }

    #[test]
    fn smt_bisimulation_fuzz_seed_corpus_covers_m3_fixture_shapes() {
        let report = run_smt_bisimulation_fuzz_seed_corpus_v0(128);
        assert_eq!(report.schema_version, "0");
        assert_eq!(report.fixture_suite, "m3-cascade-proof-fixtures");
        assert_eq!(report.checked_obligation_count, 128 * 4);
        assert_eq!(report.l1_l3_mismatch_count, 0);
        assert!(report.passed);
    }

    #[test]
    fn smt_bisimulation_fuzz_case_is_a_schema_zero_contract() {
        let case = smt_bisimulation_fuzz_case_v0(42);
        assert_eq!(case.schema_version, "0");
        assert_eq!(case.layer_marker, "smt-cascade-verification");
        assert_eq!(case.feature_gate, "smt-stub");
        assert_eq!(case.seed, 42);
    }
}
