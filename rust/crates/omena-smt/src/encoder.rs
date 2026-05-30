use serde::Serialize;

use crate::{SMT_FEATURE_GATE_V0, SMT_LAYER_MARKER_V0, SMT_SCHEMA_VERSION_V0};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalSmtInputV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub obligation_id: String,
    pub l1_primitive: &'static str,
    pub canonical_terms: Vec<String>,
    pub smtlib2_script: String,
}

pub fn canonical_smt_input_v0(
    obligation_id: impl Into<String>,
    l1_primitive: &'static str,
    canonical_terms: Vec<String>,
) -> CanonicalSmtInputV0 {
    let smtlib2_script = canonical_smtlib2_script_v0(&canonical_terms);
    CanonicalSmtInputV0 {
        schema_version: SMT_SCHEMA_VERSION_V0,
        product: "omena-smt.canonical-input",
        layer_marker: SMT_LAYER_MARKER_V0,
        feature_gate: SMT_FEATURE_GATE_V0,
        obligation_id: obligation_id.into(),
        l1_primitive,
        canonical_terms,
        smtlib2_script,
    }
}

/// Build a canonical input whose SMT-LIB2 body is supplied directly rather than
/// derived from propositional `require:name=bool` terms.
///
/// This is used for obligations whose satisfiability is *not* a conjunction of
/// pre-decided booleans (e.g. the QF_LIA layer-ordering inversion search), so
/// the formula a solver consumes carries real arithmetic the encoder did not
/// evaluate. `canonical_terms` still records human-readable provenance for the
/// audit trail; the load-bearing reasoning lives in `smtlib2_script`.
pub fn canonical_smt_input_with_script_v0(
    obligation_id: impl Into<String>,
    l1_primitive: &'static str,
    canonical_terms: Vec<String>,
    smtlib2_script: String,
) -> CanonicalSmtInputV0 {
    CanonicalSmtInputV0 {
        schema_version: SMT_SCHEMA_VERSION_V0,
        product: "omena-smt.canonical-input",
        layer_marker: SMT_LAYER_MARKER_V0,
        feature_gate: SMT_FEATURE_GATE_V0,
        obligation_id: obligation_id.into(),
        l1_primitive,
        canonical_terms,
        smtlib2_script,
    }
}

pub fn canonical_smtlib2_script_v0(canonical_terms: &[String]) -> String {
    let mut script = String::from("(set-logic QF_UF)\n");
    for term in canonical_terms {
        if let Some((name, value)) = canonical_requirement_parts_v0(term) {
            let symbol = smtlib2_named_assertion_symbol_v0(name);
            let atom = if value { "true" } else { "false" };
            script.push_str(&format!("(assert (! {atom} :named {symbol}))\n"));
        } else {
            let comment = smtlib2_comment_v0(term);
            script.push_str(&format!("; {comment}\n"));
        }
    }
    script
}

pub(crate) fn canonical_requirement_value_v0(term: &str) -> Option<bool> {
    canonical_requirement_parts_v0(term).map(|(_, value)| value)
}

pub(crate) fn canonical_input_has_unknown_v0(input: &CanonicalSmtInputV0) -> bool {
    input
        .canonical_terms
        .iter()
        .any(|term| term.starts_with("unknown:"))
}

fn canonical_requirement_parts_v0(term: &str) -> Option<(&str, bool)> {
    let (name, value) = term.strip_prefix("require:")?.rsplit_once('=')?;
    match value {
        "true" => Some((name, true)),
        "false" => Some((name, false)),
        _ => None,
    }
}

fn smtlib2_named_assertion_symbol_v0(name: &str) -> String {
    let mut symbol = String::from("req_");
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            symbol.push(ch);
        } else {
            symbol.push('_');
        }
    }
    symbol
}

fn smtlib2_comment_v0(term: &str) -> String {
    term.chars()
        .map(|ch| match ch {
            '\n' | '\r' => ' ',
            _ => ch,
        })
        .collect()
}
