//! Non-trivial cascade-ordering obligation for `@layer` flattening.
//!
//! The propositional obligations in [`crate::obligations`] hand a solver a
//! conjunction of `require:name=bool` literals the Rust code already decided, so
//! any backend (including [`crate::StubSmtBackendV0`]) reaches the same verdict
//! and the solver adds no reasoning. This module emits a genuinely non-trivial
//! formula instead: a QF_LIA *search* over declaration orderings whose
//! satisfiability the encoder does not evaluate.
//!
//! Flattening `@layer` boundaries rewrites the cascade order for a property from
//! `(layer_rank, source_order)` to `source_order` alone. The rewrite is unsafe
//! exactly when some ordered pair of declarations inverts: declaration `a` wins
//! under the layered order (`layer_rank[a] > layer_rank[b]`) while `b` wins after
//! flattening (`source_order[b] > source_order[a]`). The encoder asserts the
//! *existence* of such a pair and lets the solver decide it. `Sat` means an
//! inversion exists, so the flatten is unsafe; `Unsat` means no ordering inverts,
//! so the flatten is cascade-safe.

use serde::Serialize;

use crate::{
    CanonicalSmtInputV0, SMT_FEATURE_GATE_V0, SMT_LAYER_MARKER_V0, SMT_SCHEMA_VERSION_V0,
    SmtBackendKindV0, SmtBackendSatResultV0, SmtBackendV0, SmtVerdictV0,
    encoder::canonical_smt_input_with_script_v0,
};

/// A competing declaration for a single property inside a layered bundle.
///
/// `layer_rank` is the cascade rank contributed by the declaration's `@layer`
/// (higher rank wins before flattening); `source_order` is its position in
/// source order (higher position wins after the layer boundary is erased).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LayerInversionDeclarationV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub declaration_id: String,
    pub layer_rank: i64,
    pub source_order: i64,
}

/// Schema-zero constructor for a layered-bundle declaration.
pub fn layer_inversion_declaration_v0(
    declaration_id: impl Into<String>,
    layer_rank: i64,
    source_order: i64,
) -> LayerInversionDeclarationV0 {
    LayerInversionDeclarationV0 {
        schema_version: SMT_SCHEMA_VERSION_V0,
        product: "omena-smt.layer-inversion-declaration",
        layer_marker: SMT_LAYER_MARKER_V0,
        feature_gate: SMT_FEATURE_GATE_V0,
        declaration_id: declaration_id.into(),
        layer_rank,
        source_order,
    }
}

/// Verdict of the layer-flatten inversion search.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LayerFlattenInversionVerdictV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub backend: SmtBackendKindV0,
    /// `true` when the solver found an inversion pair (flatten unsafe).
    pub inversion_exists: bool,
    /// `Accepted` means cascade-safe (no inversion); `Rejected` means an
    /// inversion was found; `Unknown` means the solver could not decide.
    pub verdict: SmtVerdictV0,
    pub canonical_input: CanonicalSmtInputV0,
    pub sat_result: SmtBackendSatResultV0,
}

/// Encode the layer-flatten inversion search as a QF_LIA formula.
///
/// The body fixes each declaration's `rank`/`source` integer constants and
/// asserts the disjunction of inversion conditions over all ordered pairs. The
/// encoder never computes whether the disjunction is satisfiable — that is left
/// to the backend.
pub fn canonical_layer_flatten_inversion_input_v0(
    declarations: &[LayerInversionDeclarationV0],
) -> CanonicalSmtInputV0 {
    let mut script = String::from("(set-logic QF_LIA)\n");
    for (index, declaration) in declarations.iter().enumerate() {
        script.push_str(&format!("(declare-const rank_{index} Int)\n"));
        script.push_str(&format!("(declare-const source_{index} Int)\n"));
        script.push_str(&format!(
            "(assert (= rank_{index} {}))\n",
            smtlib2_int_v0(declaration.layer_rank)
        ));
        script.push_str(&format!(
            "(assert (= source_{index} {}))\n",
            smtlib2_int_v0(declaration.source_order)
        ));
    }

    let mut inversion_clauses = Vec::new();
    for a in 0..declarations.len() {
        for b in 0..declarations.len() {
            if a == b {
                continue;
            }
            inversion_clauses.push(format!(
                "(and (> rank_{a} rank_{b}) (> source_{b} source_{a}))"
            ));
        }
    }

    let inversion_assertion = match inversion_clauses.len() {
        0 => "false".to_string(),
        1 => inversion_clauses.remove(0),
        _ => format!("(or {})", inversion_clauses.join(" ")),
    };
    script.push_str(&format!(
        "(assert (! {inversion_assertion} :named cascade_layer_flatten_inversion))\n"
    ));

    let canonical_terms = declarations
        .iter()
        .map(|declaration| {
            format!(
                "decl:{}:rank={}:source={}",
                declaration.declaration_id, declaration.layer_rank, declaration.source_order
            )
        })
        .collect();

    canonical_smt_input_with_script_v0(
        "layer-flatten-cascade-inversion",
        "prove_layer_flatten_candidate",
        canonical_terms,
        script,
    )
}

/// Discharge the layer-flatten inversion obligation through `backend`.
///
/// `Sat` (inversion found) maps to a `Rejected` cascade-safety verdict; `Unsat`
/// (no inversion exists) maps to `Accepted`; `Unknown` is preserved.
pub fn smt_check_layer_flatten_inversion_v0<B: SmtBackendV0>(
    declarations: &[LayerInversionDeclarationV0],
    backend: &B,
) -> LayerFlattenInversionVerdictV0 {
    let canonical_input = canonical_layer_flatten_inversion_input_v0(declarations);
    let check = backend.check_canonical_input_v0(&canonical_input);
    let inversion_exists = matches!(check.sat_result, SmtBackendSatResultV0::Sat);
    let verdict = match check.sat_result {
        SmtBackendSatResultV0::Sat => SmtVerdictV0::Rejected,
        SmtBackendSatResultV0::Unsat => SmtVerdictV0::Accepted,
        SmtBackendSatResultV0::Unknown => SmtVerdictV0::Unknown,
    };
    LayerFlattenInversionVerdictV0 {
        schema_version: crate::SMT_SCHEMA_VERSION_V0,
        product: "omena-smt.layer-flatten-inversion",
        layer_marker: crate::SMT_LAYER_MARKER_V0,
        feature_gate: crate::SMT_FEATURE_GATE_V0,
        backend: backend.backend_kind(),
        inversion_exists,
        verdict,
        canonical_input,
        sat_result: check.sat_result,
    }
}

fn smtlib2_int_v0(value: i64) -> String {
    if value < 0 {
        format!("(- {})", value.unsigned_abs())
    } else {
        value.to_string()
    }
}
