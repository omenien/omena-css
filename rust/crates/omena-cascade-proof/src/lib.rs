//! Product-owned cascade proof contracts.
//!
//! The default solver-free proof path is part of the shipped product surface:
//! product diagnostics and transform safety checks rely on it even when no
//! external solver is enabled. Solver-backed experiments live outside this crate.

use omena_cascade::{
    BoxLonghandInputV0, LayerFlattenInputV0, LonghandMergeInputV0, ScopeFlattenInputV0,
    StaticSupportsAssumptionV0, StaticSupportsEvalVerdictV0, evaluate_static_supports_condition,
    prove_box_shorthand_combination, prove_layer_flatten_candidate, prove_longhand_merge,
    prove_scope_flatten_candidate,
};
use omena_refinement_trait::RefinementVerdictV0;
use serde::Serialize;

pub const SMT_SCHEMA_VERSION_V0: &str = "0";
pub const SMT_LAYER_MARKER_V0: &str = "smt-cascade-verification";
pub const SMT_FEATURE_GATE_V0: &str = "smt-stub";

const CASCADE_SMT_SPEC_MATERIAL_V0: &str = "\
schema=0\n\
theory=cascade-smt-theory-v0\n\
encoding=canonical-smt-input-v0\n\
default-backend=stub-propositional\n\
opt-in-backend=smt-z3-qf-lia-layer-inversion\n\
obligations=box-shorthand-combination,scope-flatten-candidate,layer-flatten-candidate,static-supports-condition\n\
";

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

pub fn canonical_requirement_value_v0(term: &str) -> Option<bool> {
    canonical_requirement_parts_v0(term).map(|(_, value)| value)
}

pub fn canonical_input_has_unknown_v0(input: &CanonicalSmtInputV0) -> bool {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SmtBackendKindV0 {
    Stub,
    Z3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SmtBackendSatResultV0 {
    Sat,
    Unsat,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SmtBackendCheckV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub backend: SmtBackendKindV0,
    pub obligation_id: String,
    pub formula_count: usize,
    pub sat_result: SmtBackendSatResultV0,
    pub model_available: bool,
}

pub trait SmtBackendV0 {
    fn backend_kind(&self) -> SmtBackendKindV0;

    fn quantifier_elimination_tactic(&self) -> Option<&'static str> {
        None
    }

    fn check_canonical_input_v0(&self, input: &CanonicalSmtInputV0) -> SmtBackendCheckV0 {
        let sat_result = if canonical_input_has_unknown_v0(input) {
            SmtBackendSatResultV0::Unknown
        } else if input
            .canonical_terms
            .iter()
            .all(|term| canonical_requirement_value_v0(term).unwrap_or(true))
        {
            SmtBackendSatResultV0::Sat
        } else {
            SmtBackendSatResultV0::Unsat
        };
        SmtBackendCheckV0 {
            schema_version: SMT_SCHEMA_VERSION_V0,
            product: "omena-smt.backend-check",
            layer_marker: SMT_LAYER_MARKER_V0,
            feature_gate: SMT_FEATURE_GATE_V0,
            backend: self.backend_kind(),
            obligation_id: input.obligation_id.clone(),
            formula_count: input.canonical_terms.len(),
            sat_result,
            model_available: matches!(sat_result, SmtBackendSatResultV0::Sat),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StubSmtBackendV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
}

impl Default for StubSmtBackendV0 {
    fn default() -> Self {
        Self {
            schema_version: SMT_SCHEMA_VERSION_V0,
            product: "omena-smt.backend.stub",
            layer_marker: SMT_LAYER_MARKER_V0,
            feature_gate: SMT_FEATURE_GATE_V0,
        }
    }
}

impl SmtBackendV0 for StubSmtBackendV0 {
    fn backend_kind(&self) -> SmtBackendKindV0 {
        SmtBackendKindV0::Stub
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SmtVerdictV0 {
    Accepted,
    Rejected,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeSMTProofV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub obligation_id: String,
    pub backend: SmtBackendKindV0,
    pub verdict: SmtVerdictV0,
    pub l1_primitive: &'static str,
    pub l1_accepted: Option<bool>,
    pub canonical_input: CanonicalSmtInputV0,
    pub solver_check: SmtBackendCheckV0,
    pub refinement_verdict: Option<RefinementVerdictV0>,
    pub cascade_spec_digest: [u8; 32],
}

pub fn cascade_spec_digest_v0() -> [u8; 32] {
    *blake3::hash(CASCADE_SMT_SPEC_MATERIAL_V0.as_bytes()).as_bytes()
}

fn cascade_smt_proof_v0<B: SmtBackendV0>(
    canonical_input: CanonicalSmtInputV0,
    backend: &B,
    l1_primitive: &'static str,
    l1_accepted: Option<bool>,
) -> CascadeSMTProofV0 {
    let solver_check = backend.check_canonical_input_v0(&canonical_input);
    CascadeSMTProofV0 {
        schema_version: SMT_SCHEMA_VERSION_V0,
        product: "omena-smt.cascade-proof",
        layer_marker: SMT_LAYER_MARKER_V0,
        feature_gate: SMT_FEATURE_GATE_V0,
        obligation_id: canonical_input.obligation_id.clone(),
        backend: backend.backend_kind(),
        verdict: smt_verdict_from_backend_check_v0(solver_check.sat_result),
        l1_primitive,
        l1_accepted,
        canonical_input,
        solver_check,
        refinement_verdict: None,
        cascade_spec_digest: cascade_spec_digest_v0(),
    }
}

fn smt_verdict_from_backend_check_v0(sat_result: SmtBackendSatResultV0) -> SmtVerdictV0 {
    match sat_result {
        SmtBackendSatResultV0::Sat => SmtVerdictV0::Accepted,
        SmtBackendSatResultV0::Unsat => SmtVerdictV0::Rejected,
        SmtBackendSatResultV0::Unknown => SmtVerdictV0::Unknown,
    }
}

pub fn smt_prove_box_shorthand_combination_v0<B: SmtBackendV0>(
    shorthand_property: &str,
    longhands: &[BoxLonghandInputV0],
    backend: &B,
) -> CascadeSMTProofV0 {
    let proof = prove_box_shorthand_combination(shorthand_property, longhands);
    let canonical_input =
        canonical_box_shorthand_combination_input_v0(shorthand_property, longhands);
    cascade_smt_proof_v0(
        canonical_input,
        backend,
        "prove_box_shorthand_combination",
        Some(proof.accepted),
    )
}

pub fn smt_prove_longhand_merge_v0<B, S>(
    shorthand_property: &str,
    expected_longhands: &[S],
    longhands: &[LonghandMergeInputV0],
    backend: &B,
) -> CascadeSMTProofV0
where
    B: SmtBackendV0,
    S: AsRef<str>,
{
    let proof = prove_longhand_merge(shorthand_property, expected_longhands, longhands);
    let canonical_input =
        canonical_longhand_merge_input_v0(shorthand_property, expected_longhands, longhands);
    cascade_smt_proof_v0(
        canonical_input,
        backend,
        "prove_longhand_merge",
        Some(proof.accepted),
    )
}

pub fn smt_prove_scope_flatten_candidate_v0<B: SmtBackendV0>(
    input: ScopeFlattenInputV0,
    backend: &B,
) -> CascadeSMTProofV0 {
    let canonical_input = canonical_scope_flatten_candidate_input_v0(&input);
    let proof = prove_scope_flatten_candidate(input);
    cascade_smt_proof_v0(
        canonical_input,
        backend,
        "prove_scope_flatten_candidate",
        Some(proof.accepted),
    )
}

pub fn smt_prove_layer_flatten_candidate_v0<B: SmtBackendV0>(
    input: LayerFlattenInputV0,
    backend: &B,
) -> CascadeSMTProofV0 {
    let canonical_input = canonical_layer_flatten_candidate_input_v0(&input);
    let proof = prove_layer_flatten_candidate(input);
    cascade_smt_proof_v0(
        canonical_input,
        backend,
        "prove_layer_flatten_candidate",
        Some(proof.accepted),
    )
}

pub fn smt_evaluate_static_supports_condition_v0<B: SmtBackendV0>(
    condition: &str,
    assumption: StaticSupportsAssumptionV0,
    backend: &B,
) -> CascadeSMTProofV0 {
    let witness = evaluate_static_supports_condition(condition, assumption);
    let l1_accepted = match witness.verdict {
        StaticSupportsEvalVerdictV0::AlwaysTrue => Some(true),
        StaticSupportsEvalVerdictV0::AlwaysFalse => Some(false),
        StaticSupportsEvalVerdictV0::Unknown => None,
    };
    cascade_smt_proof_v0(
        canonical_static_supports_condition_input_v0(&witness.verdict),
        backend,
        "evaluate_static_supports_condition",
        l1_accepted,
    )
}

fn canonical_box_shorthand_combination_input_v0(
    shorthand_property: &str,
    longhands: &[BoxLonghandInputV0],
) -> CanonicalSmtInputV0 {
    let expected = smt_box_shorthand_longhands_v0(shorthand_property);
    let canonical_order = expected.is_some_and(|expected| {
        longhands.len() == expected.len()
            && longhands
                .iter()
                .zip(expected.iter())
                .all(|(actual, expected)| actual.property == *expected)
    });
    canonical_smt_input_v0(
        "box-shorthand-combination",
        "prove_box_shorthand_combination",
        vec![
            smt_require_term_v0("supported-shorthand-property", expected.is_some()),
            smt_require_term_v0("canonical-longhand-quartet", canonical_order),
            smt_require_term_v0(
                "no-important-longhand",
                longhands.iter().all(|longhand| !longhand.important),
            ),
            smt_require_term_v0(
                "no-empty-longhand-value",
                longhands.iter().all(|longhand| !longhand.value.is_empty()),
            ),
            smt_require_term_v0(
                "adjacent-source-order",
                longhands
                    .windows(2)
                    .all(|pair| pair[1].source_order == pair[0].source_order + 1),
            ),
        ],
    )
}

fn canonical_longhand_merge_input_v0<S>(
    shorthand_property: &str,
    expected_longhands: &[S],
    longhands: &[LonghandMergeInputV0],
) -> CanonicalSmtInputV0
where
    S: AsRef<str>,
{
    let canonical_order = !expected_longhands.is_empty()
        && longhands.len() == expected_longhands.len()
        && longhands
            .iter()
            .zip(expected_longhands.iter())
            .all(|(actual, expected)| actual.property == expected.as_ref());
    canonical_smt_input_v0(
        "longhand-merge",
        "prove_longhand_merge",
        vec![
            smt_require_term_v0("supported-merge-family", !expected_longhands.is_empty()),
            smt_require_term_v0("canonical-longhand-order", canonical_order),
            smt_require_term_v0(
                "no-important-longhand",
                longhands.iter().all(|longhand| !longhand.important),
            ),
            smt_require_term_v0(
                "no-empty-longhand-value",
                longhands.iter().all(|longhand| !longhand.value.is_empty()),
            ),
            smt_require_term_v0(
                "adjacent-source-order",
                longhands
                    .windows(2)
                    .all(|pair| pair[1].source_order == pair[0].source_order + 1),
            ),
            format!("merge-family:{shorthand_property}"),
        ],
    )
}

fn canonical_scope_flatten_candidate_input_v0(input: &ScopeFlattenInputV0) -> CanonicalSmtInputV0 {
    canonical_smt_input_v0(
        "scope-flatten-candidate",
        "prove_scope_flatten_candidate",
        vec![
            smt_require_term_v0("no-limit-selector", input.limit_selector.is_none()),
            smt_require_term_v0("root-scope", input.root_selector.trim() == ":root"),
            smt_require_term_v0("no-peer-scope", input.peer_scope_count == 0),
            smt_require_term_v0(
                "no-competing-unscoped-rule",
                input.competing_unscoped_rule_count == 0,
            ),
            smt_require_term_v0("not-inside-layer", !input.inside_layer),
        ],
    )
}

fn canonical_layer_flatten_candidate_input_v0(input: &LayerFlattenInputV0) -> CanonicalSmtInputV0 {
    canonical_smt_input_v0(
        "layer-flatten-candidate",
        "prove_layer_flatten_candidate",
        vec![
            smt_require_term_v0("closed-bundle", input.closed_bundle),
            smt_require_term_v0("no-peer-layer", input.peer_layer_count == 0),
            smt_require_term_v0("no-unlayered-rule", input.unlayered_rule_count == 0),
            smt_require_term_v0(
                "no-important-declaration",
                input.important_declaration_count == 0,
            ),
        ],
    )
}

fn canonical_static_supports_condition_input_v0(
    verdict: &StaticSupportsEvalVerdictV0,
) -> CanonicalSmtInputV0 {
    let canonical_terms = match verdict {
        StaticSupportsEvalVerdictV0::AlwaysTrue => {
            vec![smt_require_term_v0("supports-condition-known-true", true)]
        }
        StaticSupportsEvalVerdictV0::AlwaysFalse => {
            vec![smt_require_term_v0("supports-condition-known-true", false)]
        }
        StaticSupportsEvalVerdictV0::Unknown => vec!["unknown:supports-condition".to_string()],
    };
    canonical_smt_input_v0(
        "static-supports-condition",
        "evaluate_static_supports_condition",
        canonical_terms,
    )
}

fn smt_require_term_v0(name: &str, value: bool) -> String {
    format!("require:{name}={value}")
}

fn smt_box_shorthand_longhands_v0(shorthand_property: &str) -> Option<[&'static str; 4]> {
    match shorthand_property {
        "margin" => Some(["margin-top", "margin-right", "margin-bottom", "margin-left"]),
        "padding" => Some([
            "padding-top",
            "padding-right",
            "padding-bottom",
            "padding-left",
        ]),
        "border-color" => Some([
            "border-top-color",
            "border-right-color",
            "border-bottom-color",
            "border-left-color",
        ]),
        "border-style" => Some([
            "border-top-style",
            "border-right-style",
            "border-bottom-style",
            "border-left-style",
        ]),
        "border-width" => Some([
            "border-top-width",
            "border-right-width",
            "border-bottom-width",
            "border-left-width",
        ]),
        "scroll-margin" => Some([
            "scroll-margin-top",
            "scroll-margin-right",
            "scroll-margin-bottom",
            "scroll-margin-left",
        ]),
        "scroll-padding" => Some([
            "scroll-padding-top",
            "scroll-padding-right",
            "scroll-padding-bottom",
            "scroll-padding-left",
        ]),
        _ => None,
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LayerFlattenInversionVerdictV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub backend: SmtBackendKindV0,
    pub inversion_exists: bool,
    pub verdict: SmtVerdictV0,
    pub canonical_input: CanonicalSmtInputV0,
    pub sat_result: SmtBackendSatResultV0,
}

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
        schema_version: SMT_SCHEMA_VERSION_V0,
        product: "omena-smt.layer-flatten-inversion",
        layer_marker: SMT_LAYER_MARKER_V0,
        feature_gate: SMT_FEATURE_GATE_V0,
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
