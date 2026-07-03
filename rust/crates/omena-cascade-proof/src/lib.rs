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
use omena_evidence_graph::{
    EvidenceDemandEdgeV0, EvidenceGraphBuildErrorV0, EvidenceGraphV0, EvidenceNodeKeyV0,
    EvidenceNodeSeedV0, GuaranteeKindV0, ObligationFamilyIdV0, build_evidence_graph_from_edges_v0,
};
use omena_refinement_trait::RefinementVerdictV0;
use serde::Serialize;

pub mod fuzz;

pub use fuzz::{
    SmtBisimulationFuzzCaseV0, SmtBisimulationFuzzReportV0, run_smt_bisimulation_fuzz_case_v0,
    run_smt_bisimulation_fuzz_seed_corpus_v0, smt_bisimulation_fuzz_case_v0,
};

pub const SMT_SCHEMA_VERSION_V0: &str = "0";
pub const SMT_LAYER_MARKER_V0: &str = "smt-cascade-verification";
pub const SMT_FEATURE_GATE_V0: &str = "smt-stub";
const REWRITE_PROOF_INPUT_EVIDENCE_QUERY_V0: &str = "omena-cascade-proof.transform-rewrite-input";
const CASCADE_PROOF_RECORD_EVIDENCE_QUERY_V0: &str = "omena-cascade-proof.cascade-proof-record";
const CASCADE_PROOF_EVIDENCE_EDGE_KIND_V0: &str = "cascade-proof-evidence";
pub const TRANSFORM_REWRITE_PROOF_INPUT_OBLIGATION_FAMILY_V0: ObligationFamilyIdV0 =
    ObligationFamilyIdV0::CascadeObligationDeclaration;

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

impl CascadeSMTProofV0 {
    pub fn evidence_node_key(&self) -> EvidenceNodeKeyV0 {
        EvidenceNodeKeyV0::new(
            CASCADE_PROOF_RECORD_EVIDENCE_QUERY_V0,
            self.obligation_id.clone(),
        )
    }

    pub fn evidence_node_seed(&self) -> EvidenceNodeSeedV0 {
        EvidenceNodeSeedV0::new(
            self.evidence_node_key(),
            vec![
                ["obligation:", self.obligation_id.as_str()].concat(),
                ["primitive:", self.l1_primitive].concat(),
                ["featureGate:", self.feature_gate].concat(),
            ],
            GuaranteeKindV0::for_label_less_family(),
        )
    }

    pub fn evidence_graph(&self) -> Result<EvidenceGraphV0, EvidenceGraphBuildErrorV0> {
        build_evidence_graph_from_edges_v0(
            [self.evidence_node_seed()],
            [EvidenceDemandEdgeV0::new(
                CASCADE_PROOF_RECORD_EVIDENCE_QUERY_V0,
                self.evidence_node_key(),
                CASCADE_PROOF_EVIDENCE_EDGE_KIND_V0,
            )],
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformRewriteProofInputV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub pass_id: String,
    pub cascade_obligation_declared: bool,
    pub provenance_recomputed: bool,
    pub provenance_preserved: bool,
    pub contains_bogus_or_trivia: bool,
    pub stable_post_semantic_ir: bool,
}

impl TransformRewriteProofInputV0 {
    pub fn new(
        pass_id: impl Into<String>,
        obligation_family: ObligationFamilyIdV0,
        provenance_recomputed: bool,
        provenance_preserved: bool,
        contains_bogus_or_trivia: bool,
        stable_post_semantic_ir: bool,
    ) -> Self {
        let cascade_obligation_declared = obligation_family.declares_cascade_obligation();
        Self {
            schema_version: SMT_SCHEMA_VERSION_V0,
            product: "omena-cascade-proof.transform-rewrite-input",
            pass_id: pass_id.into(),
            cascade_obligation_declared,
            provenance_recomputed,
            provenance_preserved,
            contains_bogus_or_trivia,
            stable_post_semantic_ir,
        }
    }

    pub fn evidence_node_key(&self) -> EvidenceNodeKeyV0 {
        EvidenceNodeKeyV0::new(REWRITE_PROOF_INPUT_EVIDENCE_QUERY_V0, self.pass_id.clone())
    }

    pub fn evidence_node_seed(&self) -> EvidenceNodeSeedV0 {
        EvidenceNodeSeedV0::new(
            self.evidence_node_key(),
            vec![
                ["pass:", self.pass_id.as_str()].concat(),
                [
                    "cascadeObligationDeclared:",
                    self.cascade_obligation_declared.to_string().as_str(),
                ]
                .concat(),
                [
                    "provenanceRecomputed:",
                    self.provenance_recomputed.to_string().as_str(),
                ]
                .concat(),
                [
                    "provenancePreserved:",
                    self.provenance_preserved.to_string().as_str(),
                ]
                .concat(),
            ],
            GuaranteeKindV0::for_label_less_family(),
        )
    }

    pub fn evidence_graph(&self) -> Result<EvidenceGraphV0, EvidenceGraphBuildErrorV0> {
        build_evidence_graph_from_edges_v0(
            [self.evidence_node_seed()],
            [EvidenceDemandEdgeV0::new(
                REWRITE_PROOF_INPUT_EVIDENCE_QUERY_V0,
                self.evidence_node_key(),
                CASCADE_PROOF_EVIDENCE_EDGE_KIND_V0,
            )],
        )
    }
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

pub fn smt_verify_transform_rewrite_candidate_v0<B: SmtBackendV0>(
    input: &TransformRewriteProofInputV0,
    backend: &B,
) -> CascadeSMTProofV0 {
    cascade_smt_proof_v0(
        canonical_transform_rewrite_candidate_input_v0(input),
        backend,
        "verify_transform_rewrite_candidate",
        Some(
            input.cascade_obligation_declared
                && input.provenance_recomputed
                && input.provenance_preserved
                && !input.contains_bogus_or_trivia
                && input.stable_post_semantic_ir,
        ),
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

fn canonical_transform_rewrite_candidate_input_v0(
    input: &TransformRewriteProofInputV0,
) -> CanonicalSmtInputV0 {
    canonical_smt_input_v0(
        "transform-rewrite-candidate",
        "verify_transform_rewrite_candidate",
        vec![
            format!("pass:{}", input.pass_id),
            smt_require_term_v0(
                "cascade-obligation-declared",
                input.cascade_obligation_declared,
            ),
            smt_require_term_v0("provenance-recomputed", input.provenance_recomputed),
            smt_require_term_v0("provenance-preserved", input.provenance_preserved),
            smt_require_term_v0("no-bogus-or-trivia", !input.contains_bogus_or_trivia),
            smt_require_term_v0("stable-post-semantic-ir", input.stable_post_semantic_ir),
        ],
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

#[cfg(test)]
mod tests {
    use super::*;
    use omena_cascade::{
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
    fn default_backend_matches_l1_box_shorthand_verdict() {
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
    fn transform_rewrite_verification_runs_backend_check() {
        let backend = StubSmtBackendV0::default();
        let proof_input = TransformRewriteProofInputV0::new(
            "rule-deduplication",
            ObligationFamilyIdV0::CascadeObligationDeclaration,
            true,
            true,
            false,
            true,
        );
        let proof = smt_verify_transform_rewrite_candidate_v0(&proof_input, &backend);

        assert_eq!(proof.verdict, SmtVerdictV0::Accepted);
        assert_eq!(proof.l1_primitive, "verify_transform_rewrite_candidate");
        assert_eq!(
            proof.canonical_input.obligation_id,
            "transform-rewrite-candidate"
        );
        assert!(
            proof
                .canonical_input
                .canonical_terms
                .contains(&"require:provenance-recomputed=true".to_string())
        );
        assert!(
            proof
                .canonical_input
                .canonical_terms
                .contains(&"require:no-bogus-or-trivia=true".to_string())
        );
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
    fn smt_bisimulation_fuzz_seed_corpus_covers_fixture_shapes() {
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

    #[test]
    fn rewrite_proof_input_evidence_graph_preserves_public_shape() -> Result<(), serde_json::Error>
    {
        let input = TransformRewriteProofInputV0::new(
            "number-compression",
            ObligationFamilyIdV0::CascadeObligationDeclaration,
            true,
            true,
            false,
            true,
        );

        let before = serde_json::to_value(&input)?;
        let graph = input
            .evidence_graph()
            .map_err(|_| serde::ser::Error::custom("input edge must target its node"))?;
        let after = serde_json::to_value(&input)?;

        assert_eq!(before, after);
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].key.input_identity, "number-compression");
        assert_eq!(graph.nodes[0].guarantee, GuaranteeKindV0::Floor);
        assert!(
            graph.nodes[0]
                .provenance
                .iter()
                .any(|item| item == "provenancePreserved:true")
        );
        Ok(())
    }

    #[test]
    fn rewrite_proof_input_family_derivation_preserves_legacy_json_contract()
    -> Result<(), serde_json::Error> {
        for (pass_id, family, expected_declared) in [
            (
                "number-compression",
                ObligationFamilyIdV0::CascadeObligationDeclaration,
                true,
            ),
            ("print-css", ObligationFamilyIdV0::CascadeSafetyFloor, false),
        ] {
            let input =
                TransformRewriteProofInputV0::new(pass_id, family, true, false, false, true);

            assert_eq!(
                serde_json::to_value(&input)?,
                serde_json::json!({
                    "schemaVersion": "0",
                    "product": "omena-cascade-proof.transform-rewrite-input",
                    "passId": pass_id,
                    "cascadeObligationDeclared": expected_declared,
                    "provenanceRecomputed": true,
                    "provenancePreserved": false,
                    "containsBogusOrTrivia": false,
                    "stablePostSemanticIr": true,
                })
            );
            assert_eq!(
                serde_json::to_value(input.evidence_node_seed())?,
                serde_json::json!({
                    "key": {
                        "queryIdentity": REWRITE_PROOF_INPUT_EVIDENCE_QUERY_V0,
                        "inputIdentity": pass_id,
                    },
                    "provenance": [
                        format!("pass:{pass_id}"),
                        format!("cascadeObligationDeclared:{expected_declared}"),
                        "provenanceRecomputed:true",
                        "provenancePreserved:false",
                    ],
                    "guarantee": "floor",
                })
            );
        }

        Ok(())
    }

    #[test]
    fn cascade_proof_record_evidence_graph_preserves_public_shape() -> Result<(), serde_json::Error>
    {
        let backend = StubSmtBackendV0::default();
        let proof = smt_verify_transform_rewrite_candidate_v0(
            &TransformRewriteProofInputV0::new(
                "number-compression",
                ObligationFamilyIdV0::CascadeObligationDeclaration,
                true,
                true,
                false,
                true,
            ),
            &backend,
        );

        let before = serde_json::to_value(&proof)?;
        let graph = proof
            .evidence_graph()
            .map_err(|_| serde::ser::Error::custom("proof edge must target its node"))?;
        let after = serde_json::to_value(&proof)?;

        assert_eq!(before, after);
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].key.input_identity, proof.obligation_id);
        assert_eq!(graph.nodes[0].guarantee, GuaranteeKindV0::Floor);
        assert!(
            graph.nodes[0]
                .provenance
                .iter()
                .any(|item| item == "primitive:verify_transform_rewrite_candidate")
        );
        Ok(())
    }
}
