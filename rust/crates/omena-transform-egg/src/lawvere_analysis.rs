//! Feature-gated Lawvere-style analysis data for optional e-graph execution.
//!
//! This module is not part of the default transform path; it documents the
//! metadata carried when the `lawvere-saturation` experiment is enabled.

use egg::{Analysis, DidMerge, EGraph, Extractor, Id, Language, RecExpr, Runner};
use omena_lawvere::{
    AbstractDomainTagV0, LawvereSaturationExecutionV0, summarize_lawvere_saturation_execution_v0,
};

use crate::{
    CssRewriteLanguage, EggRewriteCandidateV0, EggRewriteExecutionV0, MdlExtractionCostV0,
    blocked_execution, decide_egg_rewrite, rewrite_rules_for_pass,
};

#[derive(Debug, Default, Clone)]
pub struct LawvereAnalysis;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LawvereAnalysisDataV0 {
    pub abstract_domain_tags: Vec<AbstractDomainTagV0>,
    pub enode_count: usize,
    pub contains_terminal_projection: bool,
}

impl LawvereAnalysisDataV0 {
    fn from_tag(tag: AbstractDomainTagV0) -> Self {
        Self {
            abstract_domain_tags: vec![tag],
            enode_count: 1,
            contains_terminal_projection: tag == AbstractDomainTagV0::TerminalEmission,
        }
    }
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
            | CssRewriteLanguage::Unit(_) => AbstractDomainTagV0::TokenValue,
            CssRewriteLanguage::Is(_)
            | CssRewriteLanguage::Where(_)
            | CssRewriteLanguage::List(_) => AbstractDomainTagV0::SelectorShape,
        };
        let mut data = LawvereAnalysisDataV0::from_tag(tag);
        for child in enode.children() {
            let child_data = &egraph[*child].data;
            merge_domain_tags(
                &mut data.abstract_domain_tags,
                &child_data.abstract_domain_tags,
            );
            data.contains_terminal_projection |= child_data.contains_terminal_projection;
        }
        data
    }

    fn merge(&mut self, a: &mut Self::Data, b: Self::Data) -> DidMerge {
        let before = a.clone();
        merge_domain_tags(&mut a.abstract_domain_tags, &b.abstract_domain_tags);
        a.enode_count = a.enode_count.max(b.enode_count);
        a.contains_terminal_projection |= b.contains_terminal_projection;
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

#[cfg(test)]
mod tests {
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
    }
}
