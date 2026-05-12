//! Optional e-graph rewrite boundary for Omena CSS transforms.
//!
//! Selector and computed-value rewrites are the current e-graph candidates.
//! This crate keeps their proof requirements explicit without forcing an
//! e-graph dependency into the core transform path.

use omena_transform_cst::TransformPassKind;
use omena_transform_passes::{TransformPassPlanV0, plan_transform_passes};
use serde::Serialize;

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

fn managed_egg_passes() -> [TransformPassKind; 2] {
    [
        TransformPassKind::SelectorIsWhereCompression,
        TransformPassKind::CalcReduction,
    ]
}

fn is_managed_egg_pass_id(pass_id: &str) -> bool {
    managed_egg_passes().iter().any(|pass| pass.id() == pass_id)
}

#[cfg(test)]
mod tests {
    use super::{
        EggRewriteCandidateV0, EggRewriteProofV0, decide_egg_rewrite, plan_egg_rewrite_passes,
        summarize_omena_transform_egg_boundary,
    };
    use omena_transform_cst::TransformPassKind;

    #[test]
    fn exposes_p08_and_p25_optional_egg_boundary() {
        let boundary = summarize_omena_transform_egg_boundary();

        assert_eq!(boundary.product, "omena-transform-egg.boundary");
        assert_eq!(
            boundary.managed_pass_ids,
            vec!["p08-selector-is-where-compression", "p25-calc-reduction"]
        );
        assert_eq!(boundary.proof_obligations.len(), 4);
    }

    #[test]
    fn plans_requested_egg_passes_through_transform_pass_planner() {
        let plan = plan_egg_rewrite_passes(true, true);

        assert_eq!(
            plan.planned_pass_ids,
            vec!["p08-selector-is-where-compression", "p25-calc-reduction"]
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
}
