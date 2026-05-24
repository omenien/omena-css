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
