//! Feature-gated TransformCatalog-style analysis data for optional e-graph execution.
//!
//! This module is not part of the default transform path; it documents the
//! metadata carried when the `transform-catalog-saturation` experiment is enabled.

use egg::{Analysis, DidMerge, EGraph, Extractor, Id, Language, RecExpr, Runner};
#[allow(deprecated)]
use omena_lawvere::LawvereSaturationExecutionV0;
use omena_lawvere::{
    AbstractDomainTagV0, TransformCatalogSaturationExecutionV0,
    summarize_transform_catalog_saturation_execution_v0,
};
use serde::Serialize;

use crate::{
    CssRewriteLanguage, EggRewriteCandidateV0, EggRewriteExecutionV0, MdlExtractionCostV0,
    blocked_execution, decide_egg_rewrite, rewrite_rules_for_pass,
};

#[derive(Debug, Default, Clone)]
pub struct TransformCatalogAnalysis;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformCatalogAnalysisDataV0 {
    pub abstract_domain_tags: Vec<AbstractDomainTagV0>,
    pub enode_count: usize,
    pub contains_terminal_projection: bool,
    pub specificity_carrier: TransformCatalogSpecificityCarrierV0,
    pub computed_value_carrier: TransformCatalogComputedValueCarrierV0,
    pub var_state_carrier: TransformCatalogVarStateCarrierV0,
    pub provenance_carrier: TransformCatalogProvenanceCarrierV0,
}

impl TransformCatalogAnalysisDataV0 {
    fn from_enode(tag: AbstractDomainTagV0, enode: &CssRewriteLanguage) -> Self {
        Self {
            abstract_domain_tags: vec![tag],
            enode_count: 1,
            contains_terminal_projection: tag == AbstractDomainTagV0::TerminalEmission,
            specificity_carrier: TransformCatalogSpecificityCarrierV0::from_enode(enode),
            computed_value_carrier: TransformCatalogComputedValueCarrierV0::from_enode(enode),
            var_state_carrier: TransformCatalogVarStateCarrierV0::from_enode(enode),
            provenance_carrier: TransformCatalogProvenanceCarrierV0::from_enode(enode),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformCatalogSpecificityCarrierV0 {
    pub selector_context_count: usize,
    pub selector_atom_count: usize,
    pub zero_specificity_context_seen: bool,
    pub selector_specificity_obligation_ready: bool,
}

impl TransformCatalogSpecificityCarrierV0 {
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
pub struct TransformCatalogComputedValueCarrierV0 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exact_numeric_value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exact_unit: Option<String>,
    pub exact_value_candidates: Vec<String>,
    pub computed_value_obligation_ready: bool,
    pub expression_kinds: Vec<&'static str>,
}

impl TransformCatalogComputedValueCarrierV0 {
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
pub struct TransformCatalogVarStateCarrierV0 {
    pub symbolic_reference_count: usize,
    pub symbol_tokens: Vec<String>,
    pub unresolved_var_reference_seen: bool,
}

impl TransformCatalogVarStateCarrierV0 {
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
pub struct TransformCatalogProvenanceCarrierV0 {
    pub enode_kinds: Vec<&'static str>,
    pub provenance_obligation_ready: bool,
}

impl TransformCatalogProvenanceCarrierV0 {
    fn from_enode(enode: &CssRewriteLanguage) -> Self {
        Self {
            enode_kinds: vec![transform_catalog_enode_kind(enode)],
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
pub struct TransformCatalogAnalysisCarrierWitnessV0 {
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
    pub root_data: TransformCatalogAnalysisDataV0,
}

impl Analysis<CssRewriteLanguage> for TransformCatalogAnalysis {
    type Data = TransformCatalogAnalysisDataV0;

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
            CssRewriteLanguage::Declaration(_) | CssRewriteLanguage::StalePrefixDeclaration(_) => {
                AbstractDomainTagV0::TerminalEmission
            }
        };
        let mut data = TransformCatalogAnalysisDataV0::from_enode(tag, enode);
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

fn transform_catalog_enode_kind(enode: &CssRewriteLanguage) -> &'static str {
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
        CssRewriteLanguage::Declaration(_) => "decl",
        CssRewriteLanguage::StalePrefixDeclaration(_) => "stale-prefix-decl",
    }
}

fn refine_computed_value_carrier(
    enode: &CssRewriteLanguage,
    data: &mut TransformCatalogAnalysisDataV0,
    egraph: &EGraph<CssRewriteLanguage, TransformCatalogAnalysis>,
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
    left: &TransformCatalogAnalysisDataV0,
    right: &TransformCatalogAnalysisDataV0,
    operation: impl FnOnce(i64, i64) -> i64,
) -> TransformCatalogComputedValueCarrierV0 {
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
    TransformCatalogComputedValueCarrierV0 {
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

pub fn execute_egg_rewrite_with_transform_catalog_analysis(
    candidate: EggRewriteCandidateV0,
) -> (EggRewriteExecutionV0, TransformCatalogSaturationExecutionV0) {
    let decision = decide_egg_rewrite(candidate.clone());
    if !decision.accepted {
        let execution = blocked_execution(candidate.clone(), decision.blocked_reason);
        let saturation = summarize_transform_catalog_saturation_execution_v0(
            candidate.pass_id,
            0,
            0,
            0,
            0,
            false,
        );
        return (execution, saturation);
    }

    let expression = match candidate.before.parse::<RecExpr<CssRewriteLanguage>>() {
        Ok(expression) => expression,
        Err(_) => {
            let execution = blocked_execution(
                candidate.clone(),
                Some("rewrite expression could not parse"),
            );
            let saturation = summarize_transform_catalog_saturation_execution_v0(
                candidate.pass_id,
                0,
                0,
                0,
                0,
                false,
            );
            return (execution, saturation);
        }
    };
    let Some(rules) = rewrite_rules_for_pass::<TransformCatalogAnalysis>(candidate.pass_id) else {
        let execution = blocked_execution(
            candidate.clone(),
            Some("pass is not managed by omena-transform-egg"),
        );
        let saturation = summarize_transform_catalog_saturation_execution_v0(
            candidate.pass_id,
            0,
            0,
            0,
            0,
            false,
        );
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
    let saturation = summarize_transform_catalog_saturation_execution_v0(
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
            .then_some("transform-catalog analysis extraction did not match candidate output"),
        before: candidate.before,
        after,
        expected_after: candidate.after,
        after_matches_candidate,
        engine: "egg+transform-catalog-analysis",
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

pub fn summarize_transform_catalog_analysis_carrier_witness_v0(
    candidate: EggRewriteCandidateV0,
) -> Option<TransformCatalogAnalysisCarrierWitnessV0> {
    let decision = decide_egg_rewrite(candidate.clone());
    if !decision.accepted {
        return None;
    }

    let expression = candidate
        .before
        .parse::<RecExpr<CssRewriteLanguage>>()
        .ok()?;
    let rules = rewrite_rules_for_pass::<TransformCatalogAnalysis>(candidate.pass_id)?;
    let runner = Runner::default()
        .with_expr(&expression)
        .with_iter_limit(8)
        .run(rules.as_slice());
    let root = runner.roots[0];
    let extractor = Extractor::new(&runner.egraph, MdlExtractionCostV0::default_ast_size());
    let (_, extracted) = extractor.find_best(root);
    let root_data = runner.egraph[root].data.clone();
    Some(TransformCatalogAnalysisCarrierWitnessV0 {
        schema_version: "0",
        product: "omena-transform-egg.transform-catalog-analysis-carrier-witness",
        feature_gate: "transform-catalog-saturation",
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

/// Published 0.3 compatibility analysis marker.
/// Owner: `omena-transform-egg` maintainers. Removal condition: not before 1.0,
/// after downstream migration and zero audited non-compatibility uses.
#[deprecated(
    since = "0.4.0",
    note = "use TransformCatalogAnalysis; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[derive(Debug, Default, Clone)]
pub struct LawvereAnalysis;

#[deprecated(
    since = "0.4.0",
    note = "use TransformCatalogSpecificityCarrierV0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LawvereSpecificityCarrierV0 {
    pub selector_context_count: usize,
    pub selector_atom_count: usize,
    pub zero_specificity_context_seen: bool,
    pub selector_specificity_obligation_ready: bool,
}

#[deprecated(
    since = "0.4.0",
    note = "use TransformCatalogComputedValueCarrierV0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
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

#[deprecated(
    since = "0.4.0",
    note = "use TransformCatalogVarStateCarrierV0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LawvereVarStateCarrierV0 {
    pub symbolic_reference_count: usize,
    pub symbol_tokens: Vec<String>,
    pub unresolved_var_reference_seen: bool,
}

#[deprecated(
    since = "0.4.0",
    note = "use TransformCatalogProvenanceCarrierV0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LawvereProvenanceCarrierV0 {
    pub enode_kinds: Vec<&'static str>,
    pub provenance_obligation_ready: bool,
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "use TransformCatalogAnalysisDataV0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
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

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "use TransformCatalogAnalysisCarrierWitnessV0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
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

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "compatibility conversion owned by omena-transform-egg maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
fn analysis_witness_into_compatibility_wire_v0(
    witness: TransformCatalogAnalysisCarrierWitnessV0,
) -> LawvereAnalysisCarrierWitnessV0 {
    LawvereAnalysisCarrierWitnessV0 {
        schema_version: witness.schema_version,
        product: "omena-transform-egg.lawvere-analysis-carrier-witness",
        feature_gate: "lawvere-saturation",
        claim_level: witness.claim_level,
        pass_id: witness.pass_id,
        specificity_carrier_ready: witness.specificity_carrier_ready,
        computed_value_carrier_ready: witness.computed_value_carrier_ready,
        var_state_carrier_ready: witness.var_state_carrier_ready,
        provenance_carrier_ready: witness.provenance_carrier_ready,
        theorem_claimed: witness.theorem_claimed,
        extracted_matches_candidate: witness.extracted_matches_candidate,
        root_data: LawvereAnalysisDataV0 {
            abstract_domain_tags: witness.root_data.abstract_domain_tags,
            enode_count: witness.root_data.enode_count,
            contains_terminal_projection: witness.root_data.contains_terminal_projection,
            specificity_carrier: LawvereSpecificityCarrierV0 {
                selector_context_count: witness
                    .root_data
                    .specificity_carrier
                    .selector_context_count,
                selector_atom_count: witness.root_data.specificity_carrier.selector_atom_count,
                zero_specificity_context_seen: witness
                    .root_data
                    .specificity_carrier
                    .zero_specificity_context_seen,
                selector_specificity_obligation_ready: witness
                    .root_data
                    .specificity_carrier
                    .selector_specificity_obligation_ready,
            },
            computed_value_carrier: LawvereComputedValueCarrierV0 {
                exact_numeric_value: witness.root_data.computed_value_carrier.exact_numeric_value,
                exact_unit: witness.root_data.computed_value_carrier.exact_unit,
                exact_value_candidates: witness
                    .root_data
                    .computed_value_carrier
                    .exact_value_candidates,
                computed_value_obligation_ready: witness
                    .root_data
                    .computed_value_carrier
                    .computed_value_obligation_ready,
                expression_kinds: witness.root_data.computed_value_carrier.expression_kinds,
            },
            var_state_carrier: LawvereVarStateCarrierV0 {
                symbolic_reference_count: witness
                    .root_data
                    .var_state_carrier
                    .symbolic_reference_count,
                symbol_tokens: witness.root_data.var_state_carrier.symbol_tokens,
                unresolved_var_reference_seen: witness
                    .root_data
                    .var_state_carrier
                    .unresolved_var_reference_seen,
            },
            provenance_carrier: LawvereProvenanceCarrierV0 {
                enode_kinds: witness.root_data.provenance_carrier.enode_kinds,
                provenance_obligation_ready: witness
                    .root_data
                    .provenance_carrier
                    .provenance_obligation_ready,
            },
        },
    }
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "compatibility conversion owned by omena-transform-egg maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
fn saturation_into_compatibility_wire_v0(
    saturation: TransformCatalogSaturationExecutionV0,
) -> LawvereSaturationExecutionV0 {
    LawvereSaturationExecutionV0 {
        schema_version: saturation.schema_version,
        product: saturation.product,
        layer_marker: saturation.layer_marker,
        feature_gate: "lawvere-saturation",
        mechanism_scope: saturation.mechanism_scope,
        product_path_evidence_ready: saturation.product_path_evidence_ready,
        global_transform_theorem_claimed: saturation.global_transform_theorem_claimed,
        theory_version: "lawvere-css-transform-catalog-v0",
        pass_id: saturation.pass_id,
        analysis_slot: "LawvereAnalysis",
        original_unit_analysis_path_preserved: saturation.original_unit_analysis_path_preserved,
        differential_tier: saturation.differential_tier,
        differential_fixture_count: saturation.differential_fixture_count,
        iteration_limit: saturation.iteration_limit,
        iteration_count: saturation.iteration_count,
        eclass_count: saturation.eclass_count,
        enode_count: saturation.enode_count,
        accepted: saturation.accepted,
        extracted_matches_candidate: saturation.extracted_matches_candidate,
    }
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "use execute_egg_rewrite_with_transform_catalog_analysis; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn execute_egg_rewrite_with_lawvere_analysis(
    candidate: EggRewriteCandidateV0,
) -> (EggRewriteExecutionV0, LawvereSaturationExecutionV0) {
    let (mut execution, saturation) =
        execute_egg_rewrite_with_transform_catalog_analysis(candidate);
    if execution.blocked_reason
        == Some("transform-catalog analysis extraction did not match candidate output")
    {
        execution.blocked_reason =
            Some("lawvere analysis extraction did not match candidate output");
    }
    execution.engine = "egg+lawvere-analysis";
    (execution, saturation_into_compatibility_wire_v0(saturation))
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "use summarize_transform_catalog_analysis_carrier_witness_v0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn summarize_lawvere_analysis_carrier_witness_v0(
    candidate: EggRewriteCandidateV0,
) -> Option<LawvereAnalysisCarrierWitnessV0> {
    summarize_transform_catalog_analysis_carrier_witness_v0(candidate)
        .map(analysis_witness_into_compatibility_wire_v0)
}

#[cfg(test)]
mod tests {
    use omena_evidence_graph::ObligationFamilyIdV0;
    use omena_lawvere::{
        TRANSFORM_CATALOG_GLOBAL_TRANSFORM_THEOREM_CLAIMED_V0,
        TRANSFORM_CATALOG_MECHANISM_SCOPE_V0, TRANSFORM_CATALOG_PRODUCT_PATH_EVIDENCE_READY_V0,
    };
    use omena_transform_cst::TransformPassKind;
    use sha2::{Digest, Sha256};

    use crate::{EggRewriteCandidateV0, EggRewriteProofV0};

    use super::*;

    #[allow(deprecated)]
    #[deprecated(
        since = "0.4.0",
        note = "compatibility test adapter owned by omena-transform-egg maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
    )]
    fn compatibility_analysis_serialized_v0(
        candidate: EggRewriteCandidateV0,
    ) -> Result<String, serde_json::Error> {
        let execution = execute_egg_rewrite_with_lawvere_analysis(candidate.clone());
        let witness = summarize_lawvere_analysis_carrier_witness_v0(candidate);
        serde_json::to_string(&(execution, witness))
    }

    #[test]
    #[allow(deprecated)]
    fn compatibility_and_canonical_analysis_surfaces_keep_distinct_exact_wire_bytes()
    -> Result<(), serde_json::Error> {
        let candidate = EggRewriteCandidateV0 {
            pass_id: TransformPassKind::CalcReduction.id(),
            before: "(calc (+ (unit 1 px) (unit 2 px)))".to_string(),
            after: "(unit 3 px)".to_string(),
            proof: EggRewriteProofV0::new(
                false,
                ObligationFamilyIdV0::ComputedValuePreservation,
                true,
                "same-unit calc arithmetic preserves computed value",
            ),
        };
        let compatibility_json = compatibility_analysis_serialized_v0(candidate.clone())?;
        let canonical_execution =
            execute_egg_rewrite_with_transform_catalog_analysis(candidate.clone());
        let canonical_witness = summarize_transform_catalog_analysis_carrier_witness_v0(candidate);
        let canonical_json = serde_json::to_string(&(canonical_execution, canonical_witness))?;
        let digest = |bytes: &[u8]| {
            Sha256::digest(bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        };
        assert_eq!(compatibility_json.len(), 2_050);
        assert_eq!(
            digest(compatibility_json.as_bytes()),
            "eed58058f36a6f0903b3eda4c3d739eb69321b22ba3742ba11c8edc6771a2d82"
        );
        assert_eq!(canonical_json.len(), 2_091);
        assert_eq!(
            digest(canonical_json.as_bytes()),
            "9b3144364d808c6052b29ebbfd1fd3d8b7bbc153cb757a62ce4d964104b3adfa"
        );
        assert_ne!(compatibility_json, canonical_json);
        Ok(())
    }

    #[test]
    fn transform_catalog_analysis_fills_parallel_egg_analysis_slot() {
        let (execution, saturation) =
            execute_egg_rewrite_with_transform_catalog_analysis(EggRewriteCandidateV0 {
                pass_id: TransformPassKind::CalcReduction.id(),
                before: "(calc (+ (unit 1 px) (unit 2 px)))".to_string(),
                after: "(unit 3 px)".to_string(),
                proof: EggRewriteProofV0::new(
                    false,
                    ObligationFamilyIdV0::ComputedValuePreservation,
                    true,
                    "same-unit calc arithmetic preserves computed value",
                ),
            });

        assert!(execution.accepted);
        assert_eq!(execution.engine, "egg+transform-catalog-analysis");
        assert_eq!(saturation.schema_version, "0");
        assert_eq!(saturation.layer_marker, "enriched-algebraic");
        assert_eq!(saturation.analysis_slot, "TransformCatalogAnalysis");
        assert!(saturation.original_unit_analysis_path_preserved);
        assert_eq!(saturation.differential_fixture_count, 10);
        assert_eq!(
            saturation.mechanism_scope,
            TRANSFORM_CATALOG_MECHANISM_SCOPE_V0
        );
        assert_eq!(
            saturation.product_path_evidence_ready,
            TRANSFORM_CATALOG_PRODUCT_PATH_EVIDENCE_READY_V0
        );
        assert_eq!(
            saturation.global_transform_theorem_claimed,
            TRANSFORM_CATALOG_GLOBAL_TRANSFORM_THEOREM_CLAIMED_V0
        );
    }

    #[test]
    fn transform_catalog_analysis_widens_eclass_carriers_under_fixture_witness()
    -> Result<(), &'static str> {
        let witness =
            summarize_transform_catalog_analysis_carrier_witness_v0(EggRewriteCandidateV0 {
                pass_id: TransformPassKind::CalcReduction.id(),
                before: "(calc (+ (unit 1 px) (unit 2 px)))".to_string(),
                after: "(unit 3 px)".to_string(),
                proof: EggRewriteProofV0::new(
                    false,
                    ObligationFamilyIdV0::ComputedValuePreservation,
                    true,
                    "same-unit calc arithmetic preserves computed value",
                ),
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
    fn transform_catalog_analysis_carries_selector_specificity_context() -> Result<(), &'static str>
    {
        let witness =
            summarize_transform_catalog_analysis_carrier_witness_v0(EggRewriteCandidateV0 {
                pass_id: TransformPassKind::SelectorIsWhereCompression.id(),
                before: "(where (list ready ready))".to_string(),
                after: "(where ready)".to_string(),
                proof: EggRewriteProofV0::new(
                    true,
                    ObligationFamilyIdV0::CascadeSafetyFloor,
                    true,
                    "duplicate :where() argument keeps zero specificity",
                ),
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
