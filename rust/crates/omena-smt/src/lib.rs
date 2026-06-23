//! SMT-backed cascade verification contracts.
//!
//! The crate keeps the M4-gamma three-layer split explicit:
//! L1 cascade proof primitives remain in `omena-cascade`, L2 refinement
//! delegates live beside the cascade crate, and this crate owns L3 encoding,
//! proof contracts, z3 backend selection, and unsat-core audit metadata.
//!
//! Solver-free propositional proof evaluation is product-owned by
//! `omena-cascade-proof`. This lab crate retains the opt-in `smt-z3` path and
//! the non-trivial QF_LIA layer-inversion search.
//!
//! claim_level: opt-in solver-backed checking for one non-trivial
//! QF_LIA cascade-ordering obligation, not default build SMT completeness.

pub mod backend;
pub mod encoder;
pub mod layer_inversion;
pub mod obligations;
pub mod proof;
pub mod theory;
pub mod unsat_core;

#[cfg(feature = "smt-z3")]
pub use backend::z3::Z3SmtBackendV0;
pub use backend::{SmtBackendCheckV0, SmtBackendKindV0, SmtBackendSatResultV0, SmtBackendV0};
pub use encoder::{
    CanonicalSmtInputV0, canonical_smt_input_v0, canonical_smt_input_with_script_v0,
    canonical_smtlib2_script_v0,
};
pub use layer_inversion::{
    LayerFlattenInversionVerdictV0, LayerInversionDeclarationV0,
    canonical_layer_flatten_inversion_input_v0, layer_inversion_declaration_v0,
    smt_check_layer_flatten_inversion_v0,
};
pub use obligations::{
    smt_evaluate_static_supports_condition_v0, smt_prove_box_shorthand_combination_v0,
    smt_prove_layer_flatten_candidate_v0, smt_prove_longhand_merge_v0,
    smt_prove_scope_flatten_candidate_v0,
};
pub use proof::{
    CascadeSMTProofAuditLogV0, CascadeSMTProofV0, SmtVerdictV0, cascade_spec_digest_v0,
};
pub use theory::{CascadeTheorySignatureV0, cascade_theory_signature_v0};
pub use unsat_core::{CascadeUnsatCoreLabelV0, cascade_unsat_core_label_v0};

pub const SMT_SCHEMA_VERSION_V0: &str = "0";
pub const SMT_LAYER_MARKER_V0: &str = "smt-cascade-verification";
pub const SMT_FEATURE_GATE_V0: &str = "smt-z3";

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "smt-z3")]
    use omena_cascade::BoxLonghandInputV0;

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
    fn canonical_smt_encoder_emits_named_assertions() {
        let input = canonical_smt_input_v0(
            "box-shorthand-combination",
            "prove_box_shorthand_combination",
            vec![
                "require:supported-shorthand-property=true".to_string(),
                "require:canonical-longhand-quartet=false".to_string(),
                "metadata:preserved".to_string(),
            ],
        );

        assert_eq!(input.feature_gate, "smt-z3");
        assert!(
            input
                .smtlib2_script
                .contains("(assert (! true :named req_supported_shorthand_property))")
        );
        assert!(
            input
                .smtlib2_script
                .contains("(assert (! false :named req_canonical_longhand_quartet))")
        );
        assert!(input.smtlib2_script.contains("; metadata:preserved"));
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
        assert!(
            input
                .canonical_terms
                .iter()
                .all(|term| !term.starts_with("require:"))
        );
    }

    #[cfg(feature = "smt-z3")]
    #[test]
    fn z3_decides_layer_inversion() {
        let z3 = Z3SmtBackendV0::default();

        let mut inverted = three_layer_declarations();
        inverted[2].source_order = -1;
        let z3_inverted = smt_check_layer_flatten_inversion_v0(&inverted, &z3);

        let safe = three_layer_declarations();
        let z3_safe = smt_check_layer_flatten_inversion_v0(&safe, &z3);

        assert_eq!(z3_inverted.backend, SmtBackendKindV0::Z3);
        assert_eq!(z3_inverted.sat_result, SmtBackendSatResultV0::Sat);
        assert!(z3_inverted.inversion_exists);
        assert_eq!(z3_inverted.verdict, SmtVerdictV0::Rejected);

        assert_eq!(z3_safe.sat_result, SmtBackendSatResultV0::Unsat);
        assert!(!z3_safe.inversion_exists);
        assert_eq!(z3_safe.verdict, SmtVerdictV0::Accepted);
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
