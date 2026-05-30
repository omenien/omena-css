use serde::Serialize;

use crate::{
    CATEGORICAL_FEATURE_GATE_V0, CATEGORICAL_LAYER_MARKER_V0, CATEGORICAL_SCHEMA_VERSION_V0,
    KripkeFrameV0, OmegaCascadeTruthValueV0,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModalFormulaV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub formula_id: String,
    pub kind: ModalFormulaKindV0,
    pub atoms: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ModalFormulaKindV0 {
    Atom,
    Not,
    And,
    Or,
    Necessarily,
    Possibly,
    Implies,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModalDiagnosticSchemaV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub diagnostic_code: &'static str,
    pub formula: ModalFormulaV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModalEvaluationWitnessV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub formula_id: String,
    pub frame_id: String,
    pub truth_value: OmegaCascadeTruthValueV0,
    pub s4_fragment_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModalAxiomCheckV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub axiom: &'static str,
    pub accepted: bool,
}

pub fn modal_atom_formula_v0(
    formula_id: impl Into<String>,
    atom: impl Into<String>,
) -> ModalFormulaV0 {
    ModalFormulaV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.modal-formula",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        formula_id: formula_id.into(),
        kind: ModalFormulaKindV0::Atom,
        atoms: vec![atom.into()],
    }
}

pub fn evaluate_omena_categorical_modal_formula_v0(
    formula: &ModalFormulaV0,
    frame: &KripkeFrameV0,
) -> ModalEvaluationWitnessV0 {
    let truth_value = frame
        .valuations
        .iter()
        .find(|valuation| formula.atoms.contains(&valuation.atom))
        .map(|valuation| valuation.truth_value)
        .unwrap_or(OmegaCascadeTruthValueV0::Open);
    ModalEvaluationWitnessV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.modal-evaluation-witness",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        formula_id: formula.formula_id.clone(),
        frame_id: frame.frame_id.clone(),
        truth_value,
        s4_fragment_only: true,
    }
}

pub fn modal_axiom_check_v0(axiom: &'static str, accepted: bool) -> ModalAxiomCheckV0 {
    ModalAxiomCheckV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.modal-axiom-check",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        axiom,
        accepted,
    }
}

/// Verify the S4 frame conditions on a concrete Kripke frame, computed directly
/// from its edge set: axiom T (reflexivity — every world carries a self-edge) and
/// axiom 4 (transitivity — `w -> v` and `v -> u` imply `w -> u`). Because both are
/// read off `frame.edges`, a corrupted frame that drops a self-edge or a
/// transitive-closure edge is rejected. That corrupted-edge sensitivity is the
/// discriminating control: a well-formed prefix frame satisfies both by
/// construction, so only a real edge defect can flip the verdict.
pub fn verify_s4_frame_axioms_v0(frame: &KripkeFrameV0) -> Vec<ModalAxiomCheckV0> {
    let has_edge = |from: &str, to: &str| {
        frame
            .edges
            .iter()
            .any(|edge| edge.from_world == from && edge.to_world == to)
    };
    let reflexive =
        !frame.worlds.is_empty() && frame.worlds.iter().all(|world| has_edge(world, world));
    let transitive = frame.edges.iter().all(|first| {
        frame
            .edges
            .iter()
            .filter(|second| second.from_world == first.to_world)
            .all(|second| has_edge(&first.from_world, &second.to_world))
    });
    vec![
        modal_axiom_check_v0("reflexivity-t", reflexive),
        modal_axiom_check_v0("transitivity-4", transitive),
    ]
}
