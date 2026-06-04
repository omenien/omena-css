//! Pure circuit-side contracts for zero-knowledge cascade audit.
//!
//! claim_level: constraint-generation substrate, not a standalone proving
//! backend.

use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

pub const ZK_CIRCUIT_SCHEMA_VERSION_V0: &str = "0";
pub const ZK_CIRCUIT_LAYER_MARKER_V0: &str = "cryptographic-implementation";
pub const ZK_CIRCUIT_FEATURE_GATE_V0: &str = "zk-circuit";
pub const ZK_CIRCUIT_MECHANISM_SCOPE_V0: &str = "constraintGenerationSubstrate";
pub const ZK_CIRCUIT_STANDALONE_PROVING_BACKEND_V0: bool = false;
pub const ZK_CIRCUIT_DEFAULT_PRODUCT_PROVING_V0: bool = false;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeCircuitSpecV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub mechanism_scope: &'static str,
    pub standalone_proving_backend: bool,
    pub default_product_proving: bool,
    pub circuit_id: String,
    pub arithmetization: ArithmetizationKindV0,
    pub constraint_count: usize,
    pub salsa_dependency_free: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ArithmetizationKindV0 {
    R1cs,
    Plonkish,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct R1CSConstraintV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub constraint_id: String,
    pub left: R1CSLinearCombinationV0,
    pub right: R1CSLinearCombinationV0,
    pub output: R1CSLinearCombinationV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct R1CSLinearCombinationV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub constant: i64,
    pub terms: Vec<R1CSLinearTermV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct R1CSLinearTermV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub wire_id: String,
    pub coefficient: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct R1CSWitnessAssignmentV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub values: Vec<R1CSWireValueV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct R1CSWireValueV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub wire_id: String,
    pub value: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct R1CSSatisfactionCheckV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub mechanism_scope: &'static str,
    pub standalone_proving_backend: bool,
    pub constraint_count: usize,
    pub witness_value_count: usize,
    pub satisfied: bool,
    pub unsatisfied_constraint_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlonkishGateV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub gate_id: String,
    pub selector_polynomial: String,
}

pub fn cascade_circuit_spec_v0(
    circuit_id: impl Into<String>,
    arithmetization: ArithmetizationKindV0,
) -> CascadeCircuitSpecV0 {
    CascadeCircuitSpecV0 {
        schema_version: ZK_CIRCUIT_SCHEMA_VERSION_V0,
        product: "omena-zk-circuit.cascade-circuit-spec",
        layer_marker: ZK_CIRCUIT_LAYER_MARKER_V0,
        feature_gate: ZK_CIRCUIT_FEATURE_GATE_V0,
        mechanism_scope: ZK_CIRCUIT_MECHANISM_SCOPE_V0,
        standalone_proving_backend: ZK_CIRCUIT_STANDALONE_PROVING_BACKEND_V0,
        default_product_proving: ZK_CIRCUIT_DEFAULT_PRODUCT_PROVING_V0,
        circuit_id: circuit_id.into(),
        arithmetization,
        constraint_count: 0,
        salsa_dependency_free: true,
    }
}

pub fn cascade_circuit_spec_from_canonical_terms_v0(
    circuit_id: impl Into<String>,
    canonical_terms: &[String],
) -> CascadeCircuitSpecV0 {
    let constraints = cascade_r1cs_constraints_from_canonical_terms_v0(canonical_terms);
    CascadeCircuitSpecV0 {
        constraint_count: constraints.len(),
        ..cascade_circuit_spec_v0(circuit_id, ArithmetizationKindV0::R1cs)
    }
}

pub fn cascade_r1cs_witness_from_canonical_terms_v0(
    canonical_terms: &[String],
) -> R1CSWitnessAssignmentV0 {
    let mut values = vec![r1cs_wire_value_v0("public.one", 1)];
    values.extend(
        canonical_terms
            .iter()
            .filter_map(|term| parse_requirement_term_v0(term))
            .map(|(name, value)| {
                r1cs_wire_value_v0(requirement_wire_id_v0(&name), bool_i64_v0(value))
            }),
    );
    values.sort_by(|left, right| left.wire_id.cmp(&right.wire_id));
    values.dedup_by(|left, right| left.wire_id == right.wire_id);

    R1CSWitnessAssignmentV0 {
        schema_version: ZK_CIRCUIT_SCHEMA_VERSION_V0,
        product: "omena-zk-circuit.r1cs-witness-assignment",
        layer_marker: ZK_CIRCUIT_LAYER_MARKER_V0,
        feature_gate: ZK_CIRCUIT_FEATURE_GATE_V0,
        values,
    }
}

pub fn cascade_r1cs_constraints_from_canonical_terms_v0(
    canonical_terms: &[String],
) -> Vec<R1CSConstraintV0> {
    canonical_terms
        .iter()
        .filter_map(|term| {
            let (name, _value) = parse_requirement_term_v0(term)?;
            Some(R1CSConstraintV0 {
                schema_version: ZK_CIRCUIT_SCHEMA_VERSION_V0,
                product: "omena-zk-circuit.r1cs-constraint",
                layer_marker: ZK_CIRCUIT_LAYER_MARKER_V0,
                feature_gate: ZK_CIRCUIT_FEATURE_GATE_V0,
                constraint_id: format!("requirement-{name}"),
                left: r1cs_linear_combination_v0(
                    0,
                    [r1cs_linear_term_v0(requirement_wire_id_v0(&name), 1)],
                ),
                right: r1cs_linear_combination_v0(0, [r1cs_linear_term_v0("public.one", 1)]),
                output: r1cs_linear_combination_v0(0, [r1cs_linear_term_v0("public.one", 1)]),
            })
        })
        .collect()
}

pub fn check_r1cs_witness_satisfaction_v0(
    constraints: &[R1CSConstraintV0],
    witness: &R1CSWitnessAssignmentV0,
) -> R1CSSatisfactionCheckV0 {
    let assignment = witness
        .values
        .iter()
        .map(|value| (value.wire_id.as_str(), value.value))
        .collect::<BTreeMap<_, _>>();
    let unsatisfied_constraint_ids = constraints
        .iter()
        .filter_map(|constraint| {
            let satisfied = match (
                evaluate_linear_combination_v0(&constraint.left, &assignment),
                evaluate_linear_combination_v0(&constraint.right, &assignment),
                evaluate_linear_combination_v0(&constraint.output, &assignment),
            ) {
                (Some(left), Some(right), Some(output)) => left * right == output,
                _ => false,
            };
            (!satisfied).then(|| constraint.constraint_id.clone())
        })
        .collect::<Vec<_>>();

    R1CSSatisfactionCheckV0 {
        schema_version: ZK_CIRCUIT_SCHEMA_VERSION_V0,
        product: "omena-zk-circuit.r1cs-satisfaction-check",
        layer_marker: ZK_CIRCUIT_LAYER_MARKER_V0,
        feature_gate: ZK_CIRCUIT_FEATURE_GATE_V0,
        mechanism_scope: ZK_CIRCUIT_MECHANISM_SCOPE_V0,
        standalone_proving_backend: ZK_CIRCUIT_STANDALONE_PROVING_BACKEND_V0,
        constraint_count: constraints.len(),
        witness_value_count: witness.values.len(),
        satisfied: unsatisfied_constraint_ids.is_empty(),
        unsatisfied_constraint_ids,
    }
}

pub fn r1cs_wire_ids_v0(constraints: &[R1CSConstraintV0]) -> Vec<String> {
    let mut wire_ids = BTreeSet::new();
    for constraint in constraints {
        wire_ids.extend(r1cs_linear_combination_wire_ids_v0(&constraint.left));
        wire_ids.extend(r1cs_linear_combination_wire_ids_v0(&constraint.right));
        wire_ids.extend(r1cs_linear_combination_wire_ids_v0(&constraint.output));
    }
    wire_ids.into_iter().collect()
}

fn r1cs_linear_combination_v0(
    constant: i64,
    terms: impl IntoIterator<Item = R1CSLinearTermV0>,
) -> R1CSLinearCombinationV0 {
    R1CSLinearCombinationV0 {
        schema_version: ZK_CIRCUIT_SCHEMA_VERSION_V0,
        product: "omena-zk-circuit.r1cs-linear-combination",
        layer_marker: ZK_CIRCUIT_LAYER_MARKER_V0,
        feature_gate: ZK_CIRCUIT_FEATURE_GATE_V0,
        constant,
        terms: terms.into_iter().collect(),
    }
}

fn r1cs_linear_term_v0(wire_id: impl Into<String>, coefficient: i64) -> R1CSLinearTermV0 {
    R1CSLinearTermV0 {
        schema_version: ZK_CIRCUIT_SCHEMA_VERSION_V0,
        product: "omena-zk-circuit.r1cs-linear-term",
        layer_marker: ZK_CIRCUIT_LAYER_MARKER_V0,
        feature_gate: ZK_CIRCUIT_FEATURE_GATE_V0,
        wire_id: wire_id.into(),
        coefficient,
    }
}

fn r1cs_wire_value_v0(wire_id: impl Into<String>, value: i64) -> R1CSWireValueV0 {
    R1CSWireValueV0 {
        schema_version: ZK_CIRCUIT_SCHEMA_VERSION_V0,
        product: "omena-zk-circuit.r1cs-wire-value",
        layer_marker: ZK_CIRCUIT_LAYER_MARKER_V0,
        feature_gate: ZK_CIRCUIT_FEATURE_GATE_V0,
        wire_id: wire_id.into(),
        value,
    }
}

fn r1cs_linear_combination_wire_ids_v0(
    combination: &R1CSLinearCombinationV0,
) -> impl Iterator<Item = String> + '_ {
    combination.terms.iter().map(|term| term.wire_id.clone())
}

fn evaluate_linear_combination_v0(
    combination: &R1CSLinearCombinationV0,
    assignment: &BTreeMap<&str, i64>,
) -> Option<i128> {
    let mut value = i128::from(combination.constant);
    for term in &combination.terms {
        value += i128::from(term.coefficient) * i128::from(*assignment.get(term.wire_id.as_str())?);
    }
    Some(value)
}

fn parse_requirement_term_v0(term: &str) -> Option<(String, bool)> {
    let (name, value) = term.strip_prefix("require:")?.rsplit_once('=')?;
    let value = match value {
        "true" => true,
        "false" => false,
        _ => return None,
    };
    Some((name.to_string(), value))
}

fn requirement_wire_id_v0(name: &str) -> String {
    format!("witness.{name}")
}

fn bool_i64_v0(value: bool) -> i64 {
    if value { 1 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circuit_contract_is_pure_and_schema_zero() {
        let spec = cascade_circuit_spec_v0("cascade", ArithmetizationKindV0::Plonkish);
        assert_eq!(spec.schema_version, "0");
        assert!(spec.salsa_dependency_free);
        assert_eq!(spec.mechanism_scope, ZK_CIRCUIT_MECHANISM_SCOPE_V0);
        assert_eq!(
            spec.standalone_proving_backend,
            ZK_CIRCUIT_STANDALONE_PROVING_BACKEND_V0
        );
        assert_eq!(
            spec.default_product_proving,
            ZK_CIRCUIT_DEFAULT_PRODUCT_PROVING_V0
        );
    }

    #[test]
    fn circuit_spec_counts_r1cs_constraints_from_canonical_terms() {
        let terms = vec![
            "require:supported-shorthand-property=true".to_string(),
            "require:canonical-longhand-quartet=true".to_string(),
            "unknown:supports-condition".to_string(),
        ];
        let constraints = cascade_r1cs_constraints_from_canonical_terms_v0(&terms);
        let spec = cascade_circuit_spec_from_canonical_terms_v0("cascade-smt", &terms);

        assert_eq!(constraints.len(), 2);
        assert_eq!(spec.arithmetization, ArithmetizationKindV0::R1cs);
        assert_eq!(spec.constraint_count, constraints.len());
        assert_eq!(spec.mechanism_scope, ZK_CIRCUIT_MECHANISM_SCOPE_V0);
        assert_eq!(
            spec.standalone_proving_backend,
            ZK_CIRCUIT_STANDALONE_PROVING_BACKEND_V0
        );
        assert_eq!(
            constraints[0].constraint_id,
            "requirement-supported-shorthand-property"
        );
        assert_eq!(
            constraints[0].left.terms[0].wire_id,
            "witness.supported-shorthand-property"
        );
        assert_eq!(constraints[0].right.terms[0].wire_id, "public.one");
        assert_eq!(constraints[0].output.terms[0].wire_id, "public.one");
    }

    #[test]
    fn r1cs_witness_satisfaction_is_constraint_semantics_not_term_presence() {
        let satisfiable_terms = vec![
            "require:supported-shorthand-property=true".to_string(),
            "require:canonical-longhand-quartet=true".to_string(),
        ];
        let unsatisfiable_terms = vec![
            "require:supported-shorthand-property=true".to_string(),
            "require:canonical-longhand-quartet=false".to_string(),
        ];
        let constraints = cascade_r1cs_constraints_from_canonical_terms_v0(&satisfiable_terms);
        let satisfiable_witness = cascade_r1cs_witness_from_canonical_terms_v0(&satisfiable_terms);
        let unsatisfiable_witness =
            cascade_r1cs_witness_from_canonical_terms_v0(&unsatisfiable_terms);
        let satisfiable = check_r1cs_witness_satisfaction_v0(&constraints, &satisfiable_witness);
        let unsatisfiable =
            check_r1cs_witness_satisfaction_v0(&constraints, &unsatisfiable_witness);
        let mut missing_witness = satisfiable_witness.clone();
        missing_witness
            .values
            .retain(|value| value.wire_id != "witness.canonical-longhand-quartet");
        let missing = check_r1cs_witness_satisfaction_v0(&constraints, &missing_witness);

        assert!(satisfiable.satisfied);
        assert_eq!(satisfiable.mechanism_scope, ZK_CIRCUIT_MECHANISM_SCOPE_V0);
        assert_eq!(
            satisfiable.standalone_proving_backend,
            ZK_CIRCUIT_STANDALONE_PROVING_BACKEND_V0
        );
        assert!(!unsatisfiable.satisfied);
        assert!(!missing.satisfied);
        assert_eq!(
            unsatisfiable.unsatisfied_constraint_ids,
            vec!["requirement-canonical-longhand-quartet"]
        );
        assert_eq!(
            missing.unsatisfied_constraint_ids,
            vec!["requirement-canonical-longhand-quartet"]
        );
        assert_eq!(
            r1cs_wire_ids_v0(&constraints),
            vec![
                "public.one".to_string(),
                "witness.canonical-longhand-quartet".to_string(),
                "witness.supported-shorthand-property".to_string()
            ]
        );
    }
}
