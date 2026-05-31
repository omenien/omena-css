//! SMT-encoded cascade verification contracts.
//!
//! The crate keeps the M4-gamma three-layer split explicit:
//! L1 cascade proof primitives remain in `omena-cascade`, L2 refinement
//! delegates live beside the cascade crate, and this crate owns L3 encoding,
//! proof contracts, backend selection, and unsat-core audit metadata.
//!
//! Most obligations are propositional (`require:name=bool`) conjunctions the
//! Rust code already decided; the [`layer_inversion`] obligation is the
//! exception, emitting a QF_LIA cascade-ordering search the opt-in `smt-z3`
//! backend genuinely solves and the propositional stub cannot.
//!
//! claim_level: default stub plus opt-in solver-backed checking, where the
//! opt-in z3 backend genuinely solves one non-trivial QF_LIA cascade-ordering
//! obligation the stub cannot, while the remaining obligations stay
//! propositional and not default build SMT completeness.

pub mod backend;
pub mod encoder;
pub mod fuzz;
pub mod layer_inversion;
pub mod obligations;
pub mod proof;
pub mod theory;
pub mod unsat_core;

#[cfg(feature = "smt-z3")]
pub use backend::z3::Z3SmtBackendV0;
pub use backend::{
    SmtBackendCheckV0, SmtBackendKindV0, SmtBackendSatResultV0, SmtBackendV0, StubSmtBackendV0,
};
pub use encoder::{
    CanonicalSmtInputV0, canonical_smt_input_v0, canonical_smt_input_with_script_v0,
    canonical_smtlib2_script_v0,
};
pub use fuzz::{
    SmtBisimulationFuzzCaseV0, SmtBisimulationFuzzReportV0, run_smt_bisimulation_fuzz_case_v0,
    run_smt_bisimulation_fuzz_seed_corpus_v0, smt_bisimulation_fuzz_case_v0,
};
pub use layer_inversion::{
    LayerFlattenInversionVerdictV0, LayerInversionDeclarationV0,
    canonical_layer_flatten_inversion_input_v0, layer_inversion_declaration_v0,
    smt_check_layer_flatten_inversion_v0,
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
        let backend = StubSmtBackendV0::default();
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
        assert!(
            proof
                .canonical_input
                .smtlib2_script
                .contains("(set-logic QF_UF)")
        );
    }

    #[test]
    fn cascade_spec_digest_is_hash_of_canonical_spec_material() {
        let digest = cascade_spec_digest_v0();
        assert_eq!(
            digest,
            *blake3::hash(crate::proof::CASCADE_SMT_SPEC_MATERIAL_V0.as_bytes()).as_bytes()
        );
        assert_ne!(digest, *b"omena-cascade-smt-spec-v0-------");
    }

    #[test]
    fn proof_style_bisimulation_invariant_holds_for_all_l1_primitives() {
        let backend = StubSmtBackendV0::default();
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
        let backend = StubSmtBackendV0::default();
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

    fn three_layer_declarations() -> [LayerInversionDeclarationV0; 3] {
        [
            layer_inversion_declaration_v0("base", 0, 0),
            layer_inversion_declaration_v0("components", 1, 1),
            layer_inversion_declaration_v0("utilities", 2, 2),
        ]
    }

    #[test]
    fn layer_inversion_encoder_emits_real_qf_lia_search() {
        // The body is an arithmetic search, not a conjunction of pre-decided
        // booleans: the encoder never evaluates whether an inversion exists.
        let input = canonical_layer_flatten_inversion_input_v0(&three_layer_declarations());
        assert!(input.smtlib2_script.contains("(set-logic QF_LIA)"));
        assert!(input.smtlib2_script.contains("(declare-const rank_0 Int)"));
        assert!(
            input
                .smtlib2_script
                .contains(":named cascade_layer_flatten_inversion")
        );
        assert!(input.smtlib2_script.contains("(> rank_"));
        assert!(input.smtlib2_script.contains("(> source_"));
        // No `require:name=bool` literal terms: the propositional stub has
        // nothing to evaluate and degenerates to Sat.
        assert!(
            input
                .canonical_terms
                .iter()
                .all(|term| !term.starts_with("require:"))
        );
    }

    #[test]
    fn stub_backend_cannot_reason_about_layer_inversion() {
        // The propositional stub does not model integer ordering, so it returns
        // Sat for *every* layer-inversion formula regardless of whether an
        // inversion actually exists. Both a real inversion and a safe ordering
        // collapse to the same stub verdict, which is exactly why z3 is needed.
        let backend = StubSmtBackendV0::default();

        let mut inverted = three_layer_declarations();
        // Make `utilities` (highest layer rank) lose in source order -> inversion.
        inverted[2].source_order = -1;
        let stub_inverted = smt_check_layer_flatten_inversion_v0(&inverted, &backend);

        let safe = three_layer_declarations();
        let stub_safe = smt_check_layer_flatten_inversion_v0(&safe, &backend);

        assert_eq!(stub_inverted.backend, SmtBackendKindV0::Stub);
        assert_eq!(stub_inverted.sat_result, SmtBackendSatResultV0::Sat);
        assert_eq!(stub_safe.sat_result, SmtBackendSatResultV0::Sat);
        // The stub gives the identical verdict for an unsafe and a safe ordering.
        assert_eq!(stub_inverted.verdict, stub_safe.verdict);
    }

    #[cfg(feature = "smt-z3")]
    #[test]
    fn z3_decides_layer_inversion_and_disagrees_with_stub() {
        let z3 = Z3SmtBackendV0::default();
        let stub = StubSmtBackendV0::default();

        // Emit case: `utilities` has the highest layer rank but is moved before
        // `base`/`components` in source order, so flattening inverts the winner.
        // Only `source_order` of one declaration differs from the clear case.
        let mut inverted = three_layer_declarations();
        inverted[2].source_order = -1;
        let z3_inverted = smt_check_layer_flatten_inversion_v0(&inverted, &z3);

        // Clear case: same declarations, restored source order -> layered and
        // flattened winners coincide, so no inversion exists.
        let safe = three_layer_declarations();
        let z3_safe = smt_check_layer_flatten_inversion_v0(&safe, &z3);

        // z3 actually solves the QF_LIA search and the verdict flips on the one
        // changed source_order field.
        assert_eq!(z3_inverted.backend, SmtBackendKindV0::Z3);
        assert_eq!(z3_inverted.sat_result, SmtBackendSatResultV0::Sat);
        assert!(z3_inverted.inversion_exists);
        assert_eq!(z3_inverted.verdict, SmtVerdictV0::Rejected);

        assert_eq!(z3_safe.sat_result, SmtBackendSatResultV0::Unsat);
        assert!(!z3_safe.inversion_exists);
        assert_eq!(z3_safe.verdict, SmtVerdictV0::Accepted);

        // z3 adds reasoning: on the safe ordering z3 proves Unsat while the
        // propositional stub degenerates to Sat. They disagree, which is only
        // possible because z3 solves the formula the stub cannot.
        let stub_safe = smt_check_layer_flatten_inversion_v0(&safe, &stub);
        assert_eq!(stub_safe.sat_result, SmtBackendSatResultV0::Sat);
        assert_ne!(z3_safe.sat_result, stub_safe.sat_result);
        assert_ne!(z3_safe.verdict, stub_safe.verdict);
    }

    #[cfg(feature = "smt-z3")]
    #[test]
    fn z3_backend_solves_canonical_box_shorthand_obligation() {
        let backend = Z3SmtBackendV0::default();
        let accepted = smt_prove_box_shorthand_combination_v0(
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
        let rejected = smt_prove_box_shorthand_combination_v0(
            "margin",
            &[
                BoxLonghandInputV0 {
                    property: "margin-top".to_string(),
                    value: "1px".to_string(),
                    important: false,
                    source_order: 1,
                },
                BoxLonghandInputV0 {
                    property: "margin-bottom".to_string(),
                    value: "1px".to_string(),
                    important: false,
                    source_order: 2,
                },
                BoxLonghandInputV0 {
                    property: "margin-right".to_string(),
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

        assert_eq!(accepted.backend, SmtBackendKindV0::Z3);
        assert_eq!(accepted.solver_check.sat_result, SmtBackendSatResultV0::Sat);
        assert_eq!(accepted.verdict, SmtVerdictV0::Accepted);
        assert!(
            accepted
                .canonical_input
                .smtlib2_script
                .contains("(assert (! true :named req_canonical_longhand_quartet))")
        );
        assert_eq!(
            rejected.solver_check.sat_result,
            SmtBackendSatResultV0::Unsat
        );
        assert_eq!(rejected.verdict, SmtVerdictV0::Rejected);
        assert!(
            rejected
                .canonical_input
                .smtlib2_script
                .contains("(assert (! false :named req_canonical_longhand_quartet))")
        );
        assert_ne!(
            accepted.canonical_input.canonical_terms,
            rejected.canonical_input.canonical_terms
        );
    }
}
