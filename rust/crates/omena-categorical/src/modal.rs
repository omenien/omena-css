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
    pub children: Vec<ModalFormulaV0>,
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
pub struct ModalImperativeDiagnosticProjectionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub diagnostic_code: &'static str,
    pub formula_id: String,
    pub witness_truth: OmegaCascadeTruthValueV0,
    pub imperative_action: &'static str,
    pub equivalent_to_modal_witness: bool,
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
    modal_formula_v0(
        formula_id,
        ModalFormulaKindV0::Atom,
        vec![atom.into()],
        Vec::new(),
    )
}

pub fn modal_formula_v0(
    formula_id: impl Into<String>,
    kind: ModalFormulaKindV0,
    atoms: Vec<String>,
    children: Vec<ModalFormulaV0>,
) -> ModalFormulaV0 {
    ModalFormulaV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.modal-formula",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        formula_id: formula_id.into(),
        kind,
        atoms,
        children,
    }
}

pub fn modal_diagnostic_schema_v0(
    diagnostic_code: &'static str,
    formula: ModalFormulaV0,
) -> ModalDiagnosticSchemaV0 {
    ModalDiagnosticSchemaV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.modal-diagnostic-schema",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        diagnostic_code,
        formula,
    }
}

pub fn evaluate_omena_categorical_modal_formula_v0(
    formula: &ModalFormulaV0,
    frame: &KripkeFrameV0,
) -> ModalEvaluationWitnessV0 {
    let truth_value = modal_evaluation_root_world_v0(frame)
        .and_then(|world| evaluate_modal_formula_at_world_v0(formula, frame, world))
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

fn evaluate_modal_formula_at_world_v0(
    formula: &ModalFormulaV0,
    frame: &KripkeFrameV0,
    world: &str,
) -> Option<OmegaCascadeTruthValueV0> {
    match formula.kind {
        ModalFormulaKindV0::Atom => {
            Some(modal_formula_atoms_hold_at_world_v0(formula, frame, world))
        }
        ModalFormulaKindV0::Not => Some(modal_truth_complement_v0(evaluate_modal_unary_child_v0(
            formula, frame, world,
        )?)),
        ModalFormulaKindV0::And => evaluate_modal_conjunction_v0(formula, frame, world),
        ModalFormulaKindV0::Or => evaluate_modal_disjunction_v0(formula, frame, world),
        ModalFormulaKindV0::Necessarily => {
            let reachable = modal_accessible_worlds_v0(frame, world);
            if reachable.is_empty() {
                return Some(OmegaCascadeTruthValueV0::Open);
            }
            Some(
                reachable
                    .iter()
                    .fold(OmegaCascadeTruthValueV0::Full, |truth, reachable_world| {
                        modal_truth_meet_v0(
                            truth,
                            evaluate_modal_unary_child_v0(formula, frame, reachable_world)
                                .unwrap_or(OmegaCascadeTruthValueV0::Open),
                        )
                    }),
            )
        }
        ModalFormulaKindV0::Possibly => Some(modal_accessible_worlds_v0(frame, world).iter().fold(
            OmegaCascadeTruthValueV0::Open,
            |truth, reachable_world| {
                modal_truth_join_v0(
                    truth,
                    evaluate_modal_unary_child_v0(formula, frame, reachable_world)
                        .unwrap_or(OmegaCascadeTruthValueV0::Open),
                )
            },
        )),
        ModalFormulaKindV0::Implies => evaluate_modal_implication_v0(formula, frame, world),
    }
}

fn evaluate_modal_unary_child_v0(
    formula: &ModalFormulaV0,
    frame: &KripkeFrameV0,
    world: &str,
) -> Option<OmegaCascadeTruthValueV0> {
    if let Some(child) = formula.children.first() {
        evaluate_modal_formula_at_world_v0(child, frame, world)
    } else {
        Some(modal_formula_atoms_hold_at_world_v0(formula, frame, world))
    }
}

fn evaluate_modal_conjunction_v0(
    formula: &ModalFormulaV0,
    frame: &KripkeFrameV0,
    world: &str,
) -> Option<OmegaCascadeTruthValueV0> {
    if !formula.children.is_empty() {
        return Some(formula.children.iter().fold(
            OmegaCascadeTruthValueV0::Full,
            |truth, child| {
                modal_truth_meet_v0(
                    truth,
                    evaluate_modal_formula_at_world_v0(child, frame, world)
                        .unwrap_or(OmegaCascadeTruthValueV0::Open),
                )
            },
        ));
    }
    Some(modal_formula_atoms_hold_at_world_v0(formula, frame, world))
}

fn evaluate_modal_disjunction_v0(
    formula: &ModalFormulaV0,
    frame: &KripkeFrameV0,
    world: &str,
) -> Option<OmegaCascadeTruthValueV0> {
    if !formula.children.is_empty() {
        return Some(formula.children.iter().fold(
            OmegaCascadeTruthValueV0::Open,
            |truth, child| {
                modal_truth_join_v0(
                    truth,
                    evaluate_modal_formula_at_world_v0(child, frame, world)
                        .unwrap_or(OmegaCascadeTruthValueV0::Open),
                )
            },
        ));
    }
    Some(modal_formula_atoms_hold_at_world_v0(formula, frame, world))
}

fn evaluate_modal_implication_v0(
    formula: &ModalFormulaV0,
    frame: &KripkeFrameV0,
    world: &str,
) -> Option<OmegaCascadeTruthValueV0> {
    if formula.children.len() >= 2 {
        let antecedent = evaluate_modal_formula_at_world_v0(&formula.children[0], frame, world)?;
        let consequent = evaluate_modal_formula_at_world_v0(&formula.children[1], frame, world)?;
        return Some(modal_truth_join_v0(
            modal_truth_complement_v0(antecedent),
            consequent,
        ));
    }
    if formula.atoms.len() >= 2 {
        let antecedent = modal_atom_holds_at_world_v0(&formula.atoms[0], frame, world);
        let consequent = modal_atom_holds_at_world_v0(&formula.atoms[1], frame, world);
        return Some(modal_truth_join_v0(
            modal_truth_complement_v0(antecedent),
            consequent,
        ));
    }
    None
}

fn modal_formula_atoms_hold_at_world_v0(
    formula: &ModalFormulaV0,
    frame: &KripkeFrameV0,
    world: &str,
) -> OmegaCascadeTruthValueV0 {
    if formula.atoms.is_empty() {
        return OmegaCascadeTruthValueV0::Open;
    }
    formula
        .atoms
        .iter()
        .fold(OmegaCascadeTruthValueV0::Full, |truth, atom| {
            modal_truth_meet_v0(truth, modal_atom_holds_at_world_v0(atom, frame, world))
        })
}

fn modal_atom_holds_at_world_v0(
    atom: &str,
    frame: &KripkeFrameV0,
    world: &str,
) -> OmegaCascadeTruthValueV0 {
    frame
        .valuations
        .iter()
        .filter(|valuation| valuation.world == world && valuation.atom == atom)
        .fold(OmegaCascadeTruthValueV0::Open, |truth, valuation| {
            modal_truth_join_v0(truth, valuation.truth_value)
        })
}

fn modal_evaluation_root_world_v0(frame: &KripkeFrameV0) -> Option<&str> {
    frame
        .worlds
        .iter()
        .max_by_key(|world| modal_accessible_worlds_v0(frame, world).len())
        .map(String::as_str)
}

fn modal_accessible_worlds_v0<'a>(frame: &'a KripkeFrameV0, world: &str) -> Vec<&'a str> {
    frame
        .edges
        .iter()
        .filter(|edge| edge.from_world == world)
        .map(|edge| edge.to_world.as_str())
        .collect()
}

fn modal_truth_rank_v0(truth_value: OmegaCascadeTruthValueV0) -> u8 {
    match truth_value {
        OmegaCascadeTruthValueV0::Open => 0,
        OmegaCascadeTruthValueV0::Boundary => 1,
        OmegaCascadeTruthValueV0::Closed => 2,
        OmegaCascadeTruthValueV0::Full => 3,
    }
}

fn modal_truth_meet_v0(
    left: OmegaCascadeTruthValueV0,
    right: OmegaCascadeTruthValueV0,
) -> OmegaCascadeTruthValueV0 {
    if modal_truth_rank_v0(left) <= modal_truth_rank_v0(right) {
        left
    } else {
        right
    }
}

fn modal_truth_join_v0(
    left: OmegaCascadeTruthValueV0,
    right: OmegaCascadeTruthValueV0,
) -> OmegaCascadeTruthValueV0 {
    if modal_truth_rank_v0(left) >= modal_truth_rank_v0(right) {
        left
    } else {
        right
    }
}

fn modal_truth_complement_v0(truth_value: OmegaCascadeTruthValueV0) -> OmegaCascadeTruthValueV0 {
    match truth_value {
        OmegaCascadeTruthValueV0::Open => OmegaCascadeTruthValueV0::Closed,
        OmegaCascadeTruthValueV0::Boundary => OmegaCascadeTruthValueV0::Boundary,
        OmegaCascadeTruthValueV0::Closed | OmegaCascadeTruthValueV0::Full => {
            OmegaCascadeTruthValueV0::Open
        }
    }
}

pub fn project_modal_witness_to_imperative_diagnostic_v0(
    schema: &ModalDiagnosticSchemaV0,
    witness: &ModalEvaluationWitnessV0,
) -> ModalImperativeDiagnosticProjectionV0 {
    let imperative_action = if modal_truth_emits_diagnostic_v0(witness.truth_value) {
        "emitDiagnostic"
    } else {
        "suppressDiagnostic"
    };
    ModalImperativeDiagnosticProjectionV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.modal-imperative-diagnostic-projection",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        diagnostic_code: schema.diagnostic_code,
        formula_id: witness.formula_id.clone(),
        witness_truth: witness.truth_value,
        imperative_action,
        equivalent_to_modal_witness: schema.formula.formula_id == witness.formula_id
            && witness.s4_fragment_only,
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

fn modal_truth_emits_diagnostic_v0(truth_value: OmegaCascadeTruthValueV0) -> bool {
    matches!(
        truth_value,
        OmegaCascadeTruthValueV0::Closed | OmegaCascadeTruthValueV0::Full
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_cascade_prefix_kripke_frame_v0;

    #[test]
    fn modal_evaluator_recurses_over_s4_formula_kinds() {
        let frame = build_cascade_prefix_kripke_frame_v0(
            "fixture.modal.recursive",
            "color",
            &[
                (Vec::new(), "red".to_string()),
                (vec!["media:min-width".to_string()], "blue".to_string()),
            ],
        );
        let red = modal_atom_formula_v0("atom.red", "color=red");
        let blue = modal_atom_formula_v0("atom.blue", "color=blue");
        let necessarily_red = modal_formula_v0(
            "box.red",
            ModalFormulaKindV0::Necessarily,
            Vec::new(),
            vec![red.clone()],
        );
        let possibly_blue = modal_formula_v0(
            "diamond.blue",
            ModalFormulaKindV0::Possibly,
            Vec::new(),
            vec![blue.clone()],
        );
        let red_and_blue = modal_formula_v0(
            "and.red-blue",
            ModalFormulaKindV0::And,
            Vec::new(),
            vec![red.clone(), blue.clone()],
        );
        let red_or_blue = modal_formula_v0(
            "or.red-blue",
            ModalFormulaKindV0::Or,
            Vec::new(),
            vec![red.clone(), blue.clone()],
        );
        let red_implies_blue = modal_formula_v0(
            "implies.red-blue",
            ModalFormulaKindV0::Implies,
            Vec::new(),
            vec![red, blue],
        );

        assert_eq!(
            evaluate_omena_categorical_modal_formula_v0(&necessarily_red, &frame).truth_value,
            OmegaCascadeTruthValueV0::Open
        );
        assert_eq!(
            evaluate_omena_categorical_modal_formula_v0(&possibly_blue, &frame).truth_value,
            OmegaCascadeTruthValueV0::Closed
        );
        assert_eq!(
            evaluate_omena_categorical_modal_formula_v0(&red_and_blue, &frame).truth_value,
            OmegaCascadeTruthValueV0::Open
        );
        assert_eq!(
            evaluate_omena_categorical_modal_formula_v0(&red_or_blue, &frame).truth_value,
            OmegaCascadeTruthValueV0::Closed
        );
        assert_eq!(
            evaluate_omena_categorical_modal_formula_v0(&red_implies_blue, &frame).truth_value,
            OmegaCascadeTruthValueV0::Open
        );
    }

    #[test]
    fn modal_evaluator_preserves_omega_truth_values() {
        let mut frame = build_cascade_prefix_kripke_frame_v0(
            "fixture.modal.omega-valued",
            "color",
            &[
                (Vec::new(), "red".to_string()),
                (vec!["media:min-width".to_string()], "blue".to_string()),
            ],
        );
        for valuation in &mut frame.valuations {
            match valuation.atom.as_str() {
                "color=red" => valuation.truth_value = OmegaCascadeTruthValueV0::Full,
                "color=blue" => valuation.truth_value = OmegaCascadeTruthValueV0::Boundary,
                _ => {}
            }
        }

        let possibly_blue = modal_formula_v0(
            "diamond.blue.boundary",
            ModalFormulaKindV0::Possibly,
            Vec::new(),
            vec![modal_atom_formula_v0("atom.blue", "color=blue")],
        );
        let red = modal_atom_formula_v0("atom.red", "color=red");

        assert_eq!(
            evaluate_omena_categorical_modal_formula_v0(&possibly_blue, &frame).truth_value,
            OmegaCascadeTruthValueV0::Boundary
        );
        assert_eq!(
            evaluate_omena_categorical_modal_formula_v0(&red, &frame).truth_value,
            OmegaCascadeTruthValueV0::Full
        );
    }
}
