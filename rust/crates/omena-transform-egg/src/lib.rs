//! Optional e-graph rewrite boundary for Omena CSS transforms.
//!
//! Selector and computed-value rewrites are the current e-graph candidates.
//! This crate keeps their proof requirements explicit without forcing an
//! e-graph dependency into the core transform path.

use egg::{
    AstSize, Extractor, Id, RecExpr, Rewrite, Runner, Symbol, define_language,
    rewrite as egg_rewrite,
};
use omena_transform_cst::TransformPassKind;
use omena_transform_passes::{TransformPassPlanV0, plan_transform_passes};
use serde::Serialize;

define_language! {
    enum CssRewriteLanguage {
        Num(i64),
        Symbol(Symbol),
        "+" = Add([Id; 2]),
        "*" = Mul([Id; 2]),
        "calc" = Calc(Id),
        "is" = Is(Id),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EggRewriteProofV0 {
    pub specificity_preserved: bool,
    pub computed_value_preserved: bool,
    pub provenance_preserved: bool,
    pub cascade_safe_witness: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EggRewriteCandidateV0 {
    pub pass_id: &'static str,
    pub before: String,
    pub after: String,
    pub proof: EggRewriteProofV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EggRewriteDecisionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub pass_id: &'static str,
    pub accepted: bool,
    pub blocked_reason: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EggRewriteExecutionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub pass_id: &'static str,
    pub accepted: bool,
    pub blocked_reason: Option<&'static str>,
    pub before: String,
    pub after: String,
    pub expected_after: String,
    pub after_matches_candidate: bool,
    pub engine: &'static str,
    pub iteration_limit: usize,
    pub iteration_count: usize,
    pub eclass_count: usize,
    pub enode_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformEggBoundarySummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub managed_pass_ids: Vec<&'static str>,
    pub optional_engine: &'static str,
    pub proof_obligations: Vec<&'static str>,
    pub planner_surface: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformEggPlanV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub requested_pass_ids: Vec<&'static str>,
    pub planned_pass_ids: Vec<&'static str>,
    pub pass_plan: TransformPassPlanV0,
}

pub fn summarize_omena_transform_egg_boundary() -> TransformEggBoundarySummaryV0 {
    TransformEggBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-transform-egg.boundary",
        managed_pass_ids: managed_egg_passes().iter().map(|pass| pass.id()).collect(),
        optional_engine: "egg-compatible equality saturation engine",
        proof_obligations: vec![
            "selector rewrites preserve specificity",
            "calc rewrites preserve computed value",
            "all rewrites preserve provenance",
            "all accepted rewrites carry a cascade-safe witness",
        ],
        planner_surface: "omena-transform-passes.plan",
    }
}

pub fn plan_egg_rewrite_passes(include_selector: bool, include_calc: bool) -> TransformEggPlanV0 {
    let mut requested_passes = Vec::new();
    if include_selector {
        requested_passes.push(TransformPassKind::SelectorIsWhereCompression);
    }
    if include_calc {
        requested_passes.push(TransformPassKind::CalcReduction);
    }
    let pass_plan = plan_transform_passes(&requested_passes);

    TransformEggPlanV0 {
        schema_version: "0",
        product: "omena-transform-egg.plan",
        requested_pass_ids: requested_passes.iter().map(|pass| pass.id()).collect(),
        planned_pass_ids: pass_plan.ordered_pass_ids.clone(),
        pass_plan,
    }
}

pub fn decide_egg_rewrite(candidate: EggRewriteCandidateV0) -> EggRewriteDecisionV0 {
    let blocked_reason = if !is_managed_egg_pass_id(candidate.pass_id) {
        Some("pass is not managed by omena-transform-egg")
    } else if candidate.proof.cascade_safe_witness.is_empty() {
        Some("missing cascade-safe witness")
    } else if !candidate.proof.provenance_preserved {
        Some("rewrite does not preserve provenance")
    } else if candidate.pass_id == TransformPassKind::SelectorIsWhereCompression.id()
        && !candidate.proof.specificity_preserved
    {
        Some("selector rewrite does not preserve specificity")
    } else if candidate.pass_id == TransformPassKind::CalcReduction.id()
        && !candidate.proof.computed_value_preserved
    {
        Some("calc rewrite does not preserve computed value")
    } else {
        None
    };

    EggRewriteDecisionV0 {
        schema_version: "0",
        product: "omena-transform-egg.decision",
        pass_id: candidate.pass_id,
        accepted: blocked_reason.is_none(),
        blocked_reason,
    }
}

pub fn execute_egg_rewrite(candidate: EggRewriteCandidateV0) -> EggRewriteExecutionV0 {
    let decision = decide_egg_rewrite(candidate.clone());
    if !decision.accepted {
        return blocked_execution(candidate, decision.blocked_reason);
    }

    let expression = match candidate.before.parse::<RecExpr<CssRewriteLanguage>>() {
        Ok(expression) => expression,
        Err(_) => return blocked_execution(candidate, Some("rewrite expression could not parse")),
    };
    let Some(rules) = rewrite_rules_for_pass(candidate.pass_id) else {
        return blocked_execution(
            candidate,
            Some("pass is not managed by omena-transform-egg"),
        );
    };

    let iteration_limit = 8;
    let runner = Runner::default()
        .with_expr(&expression)
        .with_iter_limit(iteration_limit)
        .run(rules.as_slice());
    let root = runner.roots[0];
    let extractor = Extractor::new(&runner.egraph, AstSize);
    let (_, extracted) = extractor.find_best(root);
    let after = extracted.to_string();
    let after_matches_candidate = after == candidate.after;

    EggRewriteExecutionV0 {
        schema_version: "0",
        product: "omena-transform-egg.execution",
        pass_id: candidate.pass_id,
        accepted: after_matches_candidate,
        blocked_reason: (!after_matches_candidate)
            .then_some("egg extraction did not match candidate output"),
        before: candidate.before,
        after,
        expected_after: candidate.after,
        after_matches_candidate,
        engine: "egg",
        iteration_limit,
        iteration_count: runner.iterations.len(),
        eclass_count: runner.egraph.number_of_classes(),
        enode_count: runner.egraph.total_size(),
    }
}

fn managed_egg_passes() -> [TransformPassKind; 2] {
    [
        TransformPassKind::SelectorIsWhereCompression,
        TransformPassKind::CalcReduction,
    ]
}

fn is_managed_egg_pass_id(pass_id: &str) -> bool {
    managed_egg_passes().iter().any(|pass| pass.id() == pass_id)
}

fn rewrite_rules_for_pass(pass_id: &'static str) -> Option<Vec<Rewrite<CssRewriteLanguage, ()>>> {
    if pass_id == TransformPassKind::SelectorIsWhereCompression.id() {
        return Some(vec![egg_rewrite!("single-is-selector"; "(is ?a)" => "?a")]);
    }
    if pass_id == TransformPassKind::CalcReduction.id() {
        return Some(vec![
            egg_rewrite!("unwrap-calc"; "(calc ?a)" => "?a"),
            egg_rewrite!("add-zero-right"; "(+ ?a 0)" => "?a"),
            egg_rewrite!("add-zero-left"; "(+ 0 ?a)" => "?a"),
            egg_rewrite!("mul-one-right"; "(* ?a 1)" => "?a"),
            egg_rewrite!("mul-one-left"; "(* 1 ?a)" => "?a"),
        ]);
    }
    None
}

fn blocked_execution(
    candidate: EggRewriteCandidateV0,
    blocked_reason: Option<&'static str>,
) -> EggRewriteExecutionV0 {
    EggRewriteExecutionV0 {
        schema_version: "0",
        product: "omena-transform-egg.execution",
        pass_id: candidate.pass_id,
        accepted: false,
        blocked_reason,
        before: candidate.before.clone(),
        after: candidate.before,
        expected_after: candidate.after,
        after_matches_candidate: false,
        engine: "egg",
        iteration_limit: 0,
        iteration_count: 0,
        eclass_count: 0,
        enode_count: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        EggRewriteCandidateV0, EggRewriteProofV0, decide_egg_rewrite, execute_egg_rewrite,
        plan_egg_rewrite_passes, summarize_omena_transform_egg_boundary,
    };
    use omena_transform_cst::TransformPassKind;

    #[test]
    fn exposes_selector_and_calc_optional_egg_boundary() {
        let boundary = summarize_omena_transform_egg_boundary();

        assert_eq!(boundary.product, "omena-transform-egg.boundary");
        assert_eq!(
            boundary.managed_pass_ids,
            vec!["selector-is-where-compression", "calc-reduction"]
        );
        assert_eq!(boundary.proof_obligations.len(), 4);
    }

    #[test]
    fn plans_requested_egg_passes_through_transform_pass_planner() {
        let plan = plan_egg_rewrite_passes(true, true);

        assert_eq!(
            plan.planned_pass_ids,
            vec!["selector-is-where-compression", "calc-reduction"]
        );
        assert_eq!(plan.pass_plan.violated_dag_edge_count, 0);
    }

    #[test]
    fn accepts_selector_rewrite_only_with_specificity_and_provenance_witnesses() {
        let decision = decide_egg_rewrite(EggRewriteCandidateV0 {
            pass_id: TransformPassKind::SelectorIsWhereCompression.id(),
            before: ":is(.a, .b)".to_string(),
            after: ".a,.b".to_string(),
            proof: EggRewriteProofV0 {
                specificity_preserved: true,
                computed_value_preserved: false,
                provenance_preserved: true,
                cascade_safe_witness: "specificity tuple preserved".to_string(),
            },
        });

        assert!(decision.accepted);
        assert_eq!(decision.blocked_reason, None);
    }

    #[test]
    fn rejects_calc_rewrite_without_computed_value_witness() {
        let decision = decide_egg_rewrite(EggRewriteCandidateV0 {
            pass_id: TransformPassKind::CalcReduction.id(),
            before: "calc(1rem + 2px)".to_string(),
            after: "1rem".to_string(),
            proof: EggRewriteProofV0 {
                specificity_preserved: false,
                computed_value_preserved: false,
                provenance_preserved: true,
                cascade_safe_witness: "candidate generated".to_string(),
            },
        });

        assert!(!decision.accepted);
        assert_eq!(
            decision.blocked_reason,
            Some("calc rewrite does not preserve computed value")
        );
    }

    #[test]
    fn executes_selector_rewrite_through_egg_engine() {
        let execution = execute_egg_rewrite(EggRewriteCandidateV0 {
            pass_id: TransformPassKind::SelectorIsWhereCompression.id(),
            before: "(is buttonPrimary)".to_string(),
            after: "buttonPrimary".to_string(),
            proof: EggRewriteProofV0 {
                specificity_preserved: true,
                computed_value_preserved: false,
                provenance_preserved: true,
                cascade_safe_witness: "single :is() argument keeps specificity".to_string(),
            },
        });

        assert!(execution.accepted);
        assert_eq!(execution.product, "omena-transform-egg.execution");
        assert_eq!(execution.engine, "egg");
        assert_eq!(execution.after, "buttonPrimary");
        assert_eq!(execution.iteration_limit, 8);
        assert!(execution.iteration_count > 0);
        assert!(execution.eclass_count > 0);
        assert!(execution.enode_count > 0);
    }

    #[test]
    fn executes_calc_rewrite_through_egg_engine() {
        let execution = execute_egg_rewrite(EggRewriteCandidateV0 {
            pass_id: TransformPassKind::CalcReduction.id(),
            before: "(calc (+ width 0))".to_string(),
            after: "width".to_string(),
            proof: EggRewriteProofV0 {
                specificity_preserved: false,
                computed_value_preserved: true,
                provenance_preserved: true,
                cascade_safe_witness: "additive identity preserves computed value".to_string(),
            },
        });

        assert!(execution.accepted);
        assert_eq!(execution.after, "width");
        assert!(execution.after_matches_candidate);
    }
}
