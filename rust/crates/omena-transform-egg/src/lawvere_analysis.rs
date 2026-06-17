//! Feature-gated Lawvere-style analysis data for optional e-graph execution.
//!
//! This module is not part of the default transform path; it documents the
//! metadata carried when the `lawvere-saturation` experiment is enabled.

use egg::{Analysis, DidMerge, EGraph, Extractor, Id, Language, RecExpr, Runner};
use omena_lawvere::{
    AbstractDomainTagV0, LawvereSaturationExecutionV0, summarize_lawvere_saturation_execution_v0,
};
use serde::Serialize;

use crate::{
    CssRewriteLanguage, EggRewriteCandidateV0, EggRewriteExecutionV0, MdlExtractionCostV0,
    blocked_execution, decide_egg_rewrite, rewrite_rules_for_pass,
};

#[derive(Debug, Default, Clone)]
pub struct LawvereAnalysis;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LawvereAnalysisDataV0 {
    pub abstract_domain_tags: Vec<AbstractDomainTagV0>,
    pub enode_count: usize,
    pub contains_terminal_projection: bool,
    pub specificity_carrier: LawvereSpecificityCarrierV0,
    pub computed_value_carrier: LawvereComputedValueCarrierV0,
    pub var_state_carrier: LawvereVarStateCarrierV0,
    pub provenance_carrier: LawvereProvenanceCarrierV0,
}

impl LawvereAnalysisDataV0 {
    fn from_enode(tag: AbstractDomainTagV0, enode: &CssRewriteLanguage) -> Self {
        Self {
            abstract_domain_tags: vec![tag],
            enode_count: 1,
            contains_terminal_projection: tag == AbstractDomainTagV0::TerminalEmission,
            specificity_carrier: LawvereSpecificityCarrierV0::from_enode(enode),
            computed_value_carrier: LawvereComputedValueCarrierV0::from_enode(enode),
            var_state_carrier: LawvereVarStateCarrierV0::from_enode(enode),
            provenance_carrier: LawvereProvenanceCarrierV0::from_enode(enode),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LawvereSpecificityCarrierV0 {
    pub selector_context_count: usize,
    pub selector_atom_count: usize,
    pub zero_specificity_context_seen: bool,
    pub selector_specificity_obligation_ready: bool,
}

impl LawvereSpecificityCarrierV0 {
    fn from_enode(enode: &CssRewriteLanguage) -> Self {
        match enode {
            CssRewriteLanguage::Is(_) => Self {
                selector_context_count: 1,
                selector_atom_count: 0,
                zero_specificity_context_seen: false,
                selector_specificity_obligation_ready: true,
            },
            CssRewriteLanguage::Where(_) => Self {
                selector_context_count: 1,
                selector_atom_count: 0,
                zero_specificity_context_seen: true,
                selector_specificity_obligation_ready: true,
            },
            CssRewriteLanguage::List(_) => Self {
                selector_context_count: 1,
                selector_atom_count: 0,
                zero_specificity_context_seen: false,
                selector_specificity_obligation_ready: true,
            },
            _ => Self::default(),
        }
    }

    fn merge_from(&mut self, other: &Self) {
        self.selector_context_count = self
            .selector_context_count
            .saturating_add(other.selector_context_count);
        self.selector_atom_count = self
            .selector_atom_count
            .saturating_add(other.selector_atom_count);
        self.zero_specificity_context_seen |= other.zero_specificity_context_seen;
        self.selector_specificity_obligation_ready |= other.selector_specificity_obligation_ready;
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LawvereComputedValueCarrierV0 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exact_numeric_value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exact_unit: Option<String>,
    pub exact_value_candidates: Vec<String>,
    pub computed_value_obligation_ready: bool,
    pub expression_kinds: Vec<&'static str>,
}

impl LawvereComputedValueCarrierV0 {
    fn from_enode(enode: &CssRewriteLanguage) -> Self {
        match enode {
            CssRewriteLanguage::Num(value) => Self {
                exact_numeric_value: Some(*value),
                exact_unit: None,
                exact_value_candidates: vec![value.to_string()],
                computed_value_obligation_ready: true,
                expression_kinds: vec!["numericLiteral"],
            },
            CssRewriteLanguage::Symbol(symbol) => Self {
                exact_numeric_value: None,
                exact_unit: Some(symbol.to_string()),
                exact_value_candidates: Vec::new(),
                computed_value_obligation_ready: false,
                expression_kinds: vec!["symbolToken"],
            },
            CssRewriteLanguage::Calc(_) => Self {
                exact_numeric_value: None,
                exact_unit: None,
                exact_value_candidates: Vec::new(),
                computed_value_obligation_ready: true,
                expression_kinds: vec!["calcExpression"],
            },
            CssRewriteLanguage::Unit(_) => Self {
                exact_numeric_value: None,
                exact_unit: None,
                exact_value_candidates: Vec::new(),
                computed_value_obligation_ready: true,
                expression_kinds: vec!["unitExpression"],
            },
            CssRewriteLanguage::Add(_) => Self {
                exact_numeric_value: None,
                exact_unit: None,
                exact_value_candidates: Vec::new(),
                computed_value_obligation_ready: true,
                expression_kinds: vec!["addExpression"],
            },
            CssRewriteLanguage::Sub(_) => Self {
                exact_numeric_value: None,
                exact_unit: None,
                exact_value_candidates: Vec::new(),
                computed_value_obligation_ready: true,
                expression_kinds: vec!["subExpression"],
            },
            CssRewriteLanguage::Mul(_) => Self {
                exact_numeric_value: None,
                exact_unit: None,
                exact_value_candidates: Vec::new(),
                computed_value_obligation_ready: true,
                expression_kinds: vec!["mulExpression"],
            },
            CssRewriteLanguage::Div(_) => Self {
                exact_numeric_value: None,
                exact_unit: None,
                exact_value_candidates: Vec::new(),
                computed_value_obligation_ready: true,
                expression_kinds: vec!["divExpression"],
            },
            CssRewriteLanguage::Box1(_)
            | CssRewriteLanguage::Box2(_)
            | CssRewriteLanguage::Box3(_)
            | CssRewriteLanguage::Box4(_) => Self {
                exact_numeric_value: None,
                exact_unit: None,
                exact_value_candidates: Vec::new(),
                computed_value_obligation_ready: true,
                expression_kinds: vec!["boxShorthandExpression"],
            },
            _ => Self::default(),
        }
    }

    fn merge_from(&mut self, other: &Self) {
        self.computed_value_obligation_ready |= other.computed_value_obligation_ready;
        merge_labels(&mut self.expression_kinds, &other.expression_kinds);
        merge_strings(
            &mut self.exact_value_candidates,
            &other.exact_value_candidates,
        );
        self.exact_numeric_value = None;
        if self.exact_unit != other.exact_unit {
            self.exact_unit = None;
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LawvereVarStateCarrierV0 {
    pub symbolic_reference_count: usize,
    pub symbol_tokens: Vec<String>,
    pub unresolved_var_reference_seen: bool,
}

impl LawvereVarStateCarrierV0 {
    fn from_enode(enode: &CssRewriteLanguage) -> Self {
        let CssRewriteLanguage::Symbol(symbol) = enode else {
            return Self::default();
        };
        let symbol = symbol.to_string();
        let unresolved_var_reference_seen = symbol.contains("--") || symbol.starts_with("var_");
        Self {
            symbolic_reference_count: 1,
            symbol_tokens: vec![symbol],
            unresolved_var_reference_seen,
        }
    }

    fn merge_from(&mut self, other: &Self) {
        self.symbolic_reference_count = self
            .symbolic_reference_count
            .saturating_add(other.symbolic_reference_count);
        for symbol in &other.symbol_tokens {
            if !self.symbol_tokens.contains(symbol) {
                self.symbol_tokens.push(symbol.clone());
            }
        }
        self.symbol_tokens.sort();
        self.unresolved_var_reference_seen |= other.unresolved_var_reference_seen;
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LawvereProvenanceCarrierV0 {
    pub enode_kinds: Vec<&'static str>,
    pub provenance_obligation_ready: bool,
}

impl LawvereProvenanceCarrierV0 {
    fn from_enode(enode: &CssRewriteLanguage) -> Self {
        Self {
            enode_kinds: vec![lawvere_enode_kind(enode)],
            provenance_obligation_ready: true,
        }
    }

    fn merge_from(&mut self, other: &Self) {
        merge_labels(&mut self.enode_kinds, &other.enode_kinds);
        self.provenance_obligation_ready |= other.provenance_obligation_ready;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LawvereAnalysisCarrierWitnessV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub feature_gate: &'static str,
    pub claim_level: &'static str,
    pub pass_id: &'static str,
    pub specificity_carrier_ready: bool,
    pub computed_value_carrier_ready: bool,
    pub var_state_carrier_ready: bool,
    pub provenance_carrier_ready: bool,
    pub theorem_claimed: bool,
    pub extracted_matches_candidate: bool,
    pub root_data: LawvereAnalysisDataV0,
}

impl Analysis<CssRewriteLanguage> for LawvereAnalysis {
    type Data = LawvereAnalysisDataV0;

    fn make(
        egraph: &mut EGraph<CssRewriteLanguage, Self>,
        enode: &CssRewriteLanguage,
        _id: Id,
    ) -> Self::Data {
        let tag = match enode {
            CssRewriteLanguage::Num(_)
            | CssRewriteLanguage::Symbol(_)
            | CssRewriteLanguage::Add(_)
            | CssRewriteLanguage::Sub(_)
            | CssRewriteLanguage::Mul(_)
            | CssRewriteLanguage::Div(_)
            | CssRewriteLanguage::Calc(_)
            | CssRewriteLanguage::Unit(_)
            | CssRewriteLanguage::Box1(_)
            | CssRewriteLanguage::Box2(_)
            | CssRewriteLanguage::Box3(_)
            | CssRewriteLanguage::Box4(_) => AbstractDomainTagV0::TokenValue,
            CssRewriteLanguage::Is(_)
            | CssRewriteLanguage::Where(_)
            | CssRewriteLanguage::List(_) => AbstractDomainTagV0::SelectorShape,
        };
        let mut data = LawvereAnalysisDataV0::from_enode(tag, enode);
        for child in enode.children() {
            let child_data = &egraph[*child].data;
            merge_domain_tags(
                &mut data.abstract_domain_tags,
                &child_data.abstract_domain_tags,
            );
            data.contains_terminal_projection |= child_data.contains_terminal_projection;
            data.specificity_carrier
                .merge_from(&child_data.specificity_carrier);
            data.computed_value_carrier
                .merge_from(&child_data.computed_value_carrier);
            data.var_state_carrier
                .merge_from(&child_data.var_state_carrier);
            data.provenance_carrier
                .merge_from(&child_data.provenance_carrier);
        }
        refine_computed_value_carrier(enode, &mut data, egraph);
        data
    }

    fn merge(&mut self, a: &mut Self::Data, b: Self::Data) -> DidMerge {
        let before = a.clone();
        merge_domain_tags(&mut a.abstract_domain_tags, &b.abstract_domain_tags);
        a.enode_count = a.enode_count.max(b.enode_count);
        a.contains_terminal_projection |= b.contains_terminal_projection;
        a.specificity_carrier.merge_from(&b.specificity_carrier);
        a.computed_value_carrier
            .merge_from(&b.computed_value_carrier);
        a.var_state_carrier.merge_from(&b.var_state_carrier);
        a.provenance_carrier.merge_from(&b.provenance_carrier);
        DidMerge(before != *a, *a != b)
    }

    fn allow_ematching_cycles(&self) -> bool {
        false
    }
}

fn merge_domain_tags(target: &mut Vec<AbstractDomainTagV0>, source: &[AbstractDomainTagV0]) {
    for tag in source {
        if !target.contains(tag) {
            target.push(*tag);
        }
    }
    target.sort();
}

fn merge_labels(target: &mut Vec<&'static str>, source: &[&'static str]) {
    for label in source {
        if !target.contains(label) {
            target.push(*label);
        }
    }
    target.sort();
}

fn merge_strings(target: &mut Vec<String>, source: &[String]) {
    for value in source {
        if !target.contains(value) {
            target.push(value.clone());
        }
    }
    target.sort();
}

fn lawvere_enode_kind(enode: &CssRewriteLanguage) -> &'static str {
    match enode {
        CssRewriteLanguage::Num(_) => "num",
        CssRewriteLanguage::Symbol(_) => "symbol",
        CssRewriteLanguage::Add(_) => "add",
        CssRewriteLanguage::Sub(_) => "sub",
        CssRewriteLanguage::Mul(_) => "mul",
        CssRewriteLanguage::Div(_) => "div",
        CssRewriteLanguage::Calc(_) => "calc",
        CssRewriteLanguage::Unit(_) => "unit",
        CssRewriteLanguage::Box1(_) => "box1",
        CssRewriteLanguage::Box2(_) => "box2",
        CssRewriteLanguage::Box3(_) => "box3",
        CssRewriteLanguage::Box4(_) => "box4",
        CssRewriteLanguage::Is(_) => "is",
        CssRewriteLanguage::Where(_) => "where",
        CssRewriteLanguage::List(_) => "list",
    }
}

fn refine_computed_value_carrier(
    enode: &CssRewriteLanguage,
    data: &mut LawvereAnalysisDataV0,
    egraph: &EGraph<CssRewriteLanguage, LawvereAnalysis>,
) {
    match enode {
        CssRewriteLanguage::Calc(child) => {
            data.computed_value_carrier = egraph[*child].data.computed_value_carrier.clone();
            data.computed_value_carrier
                .expression_kinds
                .push("calcExpression");
            data.computed_value_carrier.expression_kinds.sort();
            data.computed_value_carrier.expression_kinds.dedup();
        }
        CssRewriteLanguage::Unit([value, unit]) => {
            let value = &egraph[*value].data.computed_value_carrier;
            let unit = &egraph[*unit].data.var_state_carrier;
            data.computed_value_carrier.exact_numeric_value = value.exact_numeric_value;
            data.computed_value_carrier.exact_unit = unit.symbol_tokens.first().cloned();
            if let Some(candidate) = exact_computed_candidate_label(
                data.computed_value_carrier.exact_numeric_value,
                data.computed_value_carrier.exact_unit.as_deref(),
            ) {
                merge_strings(
                    &mut data.computed_value_carrier.exact_value_candidates,
                    &[candidate],
                );
            }
        }
        CssRewriteLanguage::Add([left, right]) => {
            data.computed_value_carrier =
                combine_binary_computed_value(&egraph[*left].data, &egraph[*right].data, |a, b| {
                    a + b
                });
            data.computed_value_carrier
                .expression_kinds
                .push("addExpression");
            data.computed_value_carrier.expression_kinds.sort();
            data.computed_value_carrier.expression_kinds.dedup();
        }
        CssRewriteLanguage::Sub([left, right]) => {
            data.computed_value_carrier =
                combine_binary_computed_value(&egraph[*left].data, &egraph[*right].data, |a, b| {
                    a - b
                });
            data.computed_value_carrier
                .expression_kinds
                .push("subExpression");
            data.computed_value_carrier.expression_kinds.sort();
            data.computed_value_carrier.expression_kinds.dedup();
        }
        _ => {}
    }
}

fn combine_binary_computed_value(
    left: &LawvereAnalysisDataV0,
    right: &LawvereAnalysisDataV0,
    operation: impl FnOnce(i64, i64) -> i64,
) -> LawvereComputedValueCarrierV0 {
    let left = &left.computed_value_carrier;
    let right = &right.computed_value_carrier;
    let exact_numeric_value = left
        .exact_numeric_value
        .zip(right.exact_numeric_value)
        .and_then(|(left_value, right_value)| {
            (left.exact_unit == right.exact_unit).then_some(operation(left_value, right_value))
        });
    let exact_unit = exact_numeric_value.and_then(|_| left.exact_unit.clone());
    let exact_value_candidates =
        exact_computed_candidate_label(exact_numeric_value, exact_unit.as_deref())
            .into_iter()
            .collect();
    let mut expression_kinds = vec!["computedBinaryExpression"];
    merge_labels(&mut expression_kinds, &left.expression_kinds);
    merge_labels(&mut expression_kinds, &right.expression_kinds);
    LawvereComputedValueCarrierV0 {
        exact_numeric_value,
        exact_unit,
        exact_value_candidates,
        computed_value_obligation_ready: true,
        expression_kinds,
    }
}

fn exact_computed_candidate_label(
    exact_numeric_value: Option<i64>,
    exact_unit: Option<&str>,
) -> Option<String> {
    let value = exact_numeric_value?;
    Some(format!("{}{}", value, exact_unit.unwrap_or("")))
}

pub fn execute_egg_rewrite_with_lawvere_analysis(
    candidate: EggRewriteCandidateV0,
) -> (EggRewriteExecutionV0, LawvereSaturationExecutionV0) {
    let decision = decide_egg_rewrite(candidate.clone());
    if !decision.accepted {
        let execution = blocked_execution(candidate.clone(), decision.blocked_reason);
        let saturation =
            summarize_lawvere_saturation_execution_v0(candidate.pass_id, 0, 0, 0, 0, false);
        return (execution, saturation);
    }

    let expression = match candidate.before.parse::<RecExpr<CssRewriteLanguage>>() {
        Ok(expression) => expression,
        Err(_) => {
            let execution = blocked_execution(
                candidate.clone(),
                Some("rewrite expression could not parse"),
            );
            let saturation =
                summarize_lawvere_saturation_execution_v0(candidate.pass_id, 0, 0, 0, 0, false);
            return (execution, saturation);
        }
    };
    let Some(rules) = rewrite_rules_for_pass::<LawvereAnalysis>(candidate.pass_id) else {
        let execution = blocked_execution(
            candidate.clone(),
            Some("pass is not managed by omena-transform-egg"),
        );
        let saturation =
            summarize_lawvere_saturation_execution_v0(candidate.pass_id, 0, 0, 0, 0, false);
        return (execution, saturation);
    };

    let iteration_limit = 8;
    let runner = Runner::default()
        .with_expr(&expression)
        .with_iter_limit(iteration_limit)
        .run(rules.as_slice());
    let root = runner.roots[0];
    let extractor = Extractor::new(&runner.egraph, MdlExtractionCostV0::default_ast_size());
    let (_, extracted) = extractor.find_best(root);
    let after = extracted.to_string();
    let after_matches_candidate = after == candidate.after;
    let iteration_count = runner.iterations.len();
    let eclass_count = runner.egraph.number_of_classes();
    let enode_count = runner.egraph.total_size();
    let saturation = summarize_lawvere_saturation_execution_v0(
        candidate.pass_id,
        iteration_limit,
        iteration_count,
        eclass_count,
        enode_count,
        after_matches_candidate,
    );

    let execution = EggRewriteExecutionV0 {
        schema_version: "0",
        product: "omena-transform-egg.execution",
        pass_id: candidate.pass_id,
        accepted: after_matches_candidate,
        blocked_reason: (!after_matches_candidate)
            .then_some("lawvere analysis extraction did not match candidate output"),
        before: candidate.before,
        after,
        expected_after: candidate.after,
        after_matches_candidate,
        engine: "egg+lawvere-analysis",
        iteration_limit,
        iteration_count,
        eclass_count,
        enode_count,
        mdl_bits: None,
        mdl_residual_bits: None,
        mdl_unit: None,
    };
    (execution, saturation)
}

pub fn summarize_lawvere_analysis_carrier_witness_v0(
    candidate: EggRewriteCandidateV0,
) -> Option<LawvereAnalysisCarrierWitnessV0> {
    let decision = decide_egg_rewrite(candidate.clone());
    if !decision.accepted {
        return None;
    }

    let expression = candidate
        .before
        .parse::<RecExpr<CssRewriteLanguage>>()
        .ok()?;
    let rules = rewrite_rules_for_pass::<LawvereAnalysis>(candidate.pass_id)?;
    let runner = Runner::default()
        .with_expr(&expression)
        .with_iter_limit(8)
        .run(rules.as_slice());
    let root = runner.roots[0];
    let extractor = Extractor::new(&runner.egraph, MdlExtractionCostV0::default_ast_size());
    let (_, extracted) = extractor.find_best(root);
    let root_data = runner.egraph[root].data.clone();
    Some(LawvereAnalysisCarrierWitnessV0 {
        schema_version: "0",
        product: "omena-transform-egg.lawvere-analysis-carrier-witness",
        feature_gate: "lawvere-saturation",
        claim_level: "fixtureWitnessEclassCarrierWidening",
        pass_id: candidate.pass_id,
        specificity_carrier_ready: root_data
            .specificity_carrier
            .selector_specificity_obligation_ready,
        computed_value_carrier_ready: root_data
            .computed_value_carrier
            .computed_value_obligation_ready,
        var_state_carrier_ready: !root_data.var_state_carrier.symbol_tokens.is_empty(),
        provenance_carrier_ready: root_data.provenance_carrier.provenance_obligation_ready,
        theorem_claimed: false,
        extracted_matches_candidate: extracted.to_string() == candidate.after,
        root_data,
    })
}

#[cfg(test)]
mod tests {
    use omena_lawvere::{
        LAWVERE_GLOBAL_TRANSFORM_THEOREM_CLAIMED_V0, LAWVERE_MECHANISM_SCOPE_V0,
        LAWVERE_PRODUCT_PATH_EVIDENCE_READY_V0,
    };
    use omena_transform_cst::TransformPassKind;

    use crate::{EggRewriteCandidateV0, EggRewriteProofV0};

    use super::*;

    #[test]
    fn lawvere_analysis_fills_parallel_egg_analysis_slot() {
        let (execution, saturation) =
            execute_egg_rewrite_with_lawvere_analysis(EggRewriteCandidateV0 {
                pass_id: TransformPassKind::CalcReduction.id(),
                before: "(calc (+ (unit 1 px) (unit 2 px)))".to_string(),
                after: "(unit 3 px)".to_string(),
                proof: EggRewriteProofV0 {
                    specificity_preserved: false,
                    computed_value_preserved: true,
                    provenance_preserved: true,
                    cascade_safe_witness: "same-unit calc arithmetic preserves computed value"
                        .to_string(),
                },
            });

        assert!(execution.accepted);
        assert_eq!(execution.engine, "egg+lawvere-analysis");
        assert_eq!(saturation.schema_version, "0");
        assert_eq!(saturation.layer_marker, "enriched-algebraic");
        assert_eq!(saturation.analysis_slot, "LawvereAnalysis");
        assert!(saturation.original_unit_analysis_path_preserved);
        assert_eq!(saturation.differential_fixture_count, 10);
        assert_eq!(saturation.mechanism_scope, LAWVERE_MECHANISM_SCOPE_V0);
        assert_eq!(
            saturation.product_path_evidence_ready,
            LAWVERE_PRODUCT_PATH_EVIDENCE_READY_V0
        );
        assert_eq!(
            saturation.global_transform_theorem_claimed,
            LAWVERE_GLOBAL_TRANSFORM_THEOREM_CLAIMED_V0
        );
    }

    #[test]
    fn lawvere_analysis_widens_eclass_carriers_under_fixture_witness() -> Result<(), &'static str> {
        let witness = summarize_lawvere_analysis_carrier_witness_v0(EggRewriteCandidateV0 {
            pass_id: TransformPassKind::CalcReduction.id(),
            before: "(calc (+ (unit 1 px) (unit 2 px)))".to_string(),
            after: "(unit 3 px)".to_string(),
            proof: EggRewriteProofV0 {
                specificity_preserved: false,
                computed_value_preserved: true,
                provenance_preserved: true,
                cascade_safe_witness: "same-unit calc arithmetic preserves computed value"
                    .to_string(),
            },
        })
        .ok_or("carrier witness should be produced for a managed accepted rewrite")?;

        assert_eq!(witness.claim_level, "fixtureWitnessEclassCarrierWidening");
        assert!(witness.computed_value_carrier_ready);
        assert!(witness.var_state_carrier_ready);
        assert!(witness.provenance_carrier_ready);
        assert!(!witness.theorem_claimed);
        assert!(witness.extracted_matches_candidate);
        assert!(
            witness
                .root_data
                .computed_value_carrier
                .exact_value_candidates
                .contains(&"3px".to_string())
        );
        assert!(
            witness
                .root_data
                .provenance_carrier
                .enode_kinds
                .contains(&"add")
        );
        Ok(())
    }

    #[test]
    fn lawvere_analysis_carries_selector_specificity_context() -> Result<(), &'static str> {
        let witness = summarize_lawvere_analysis_carrier_witness_v0(EggRewriteCandidateV0 {
            pass_id: TransformPassKind::SelectorIsWhereCompression.id(),
            before: "(where (list ready ready))".to_string(),
            after: "(where ready)".to_string(),
            proof: EggRewriteProofV0 {
                specificity_preserved: true,
                computed_value_preserved: false,
                provenance_preserved: true,
                cascade_safe_witness: "duplicate :where() argument keeps zero specificity"
                    .to_string(),
            },
        })
        .ok_or("selector carrier witness should be produced for an accepted rewrite")?;

        assert!(witness.specificity_carrier_ready);
        assert!(witness.var_state_carrier_ready);
        assert!(witness.provenance_carrier_ready);
        assert!(!witness.theorem_claimed);
        assert!(
            witness
                .root_data
                .specificity_carrier
                .zero_specificity_context_seen
        );
        assert!(
            witness
                .root_data
                .var_state_carrier
                .symbol_tokens
                .contains(&"ready".to_string())
        );
        Ok(())
    }
}
