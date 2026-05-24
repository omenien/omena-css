use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BooleanGRNStateV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub top_policy: GrnTopHandlingPolicyV0,
    pub vertices: Vec<GrnVertexStateV0>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GrnTopHandlingPolicyV0 {
    ScBoolSeqUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GrnVertexV0 {
    pub vertex_id: String,
    pub selector: String,
    pub property: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GrnBooleanState {
    Applied,
    LosingButEligible,
    Inactive,
    Top,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GrnVertexStateV0 {
    pub vertex: GrnVertexV0,
    pub state: GrnBooleanState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GrnModeDistributionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub applied_count: usize,
    pub losing_but_eligible_count: usize,
    pub inactive_count: usize,
    pub top_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeAttractorBasinV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub basin_id: String,
    pub strategy: AttractorEnumerationStrategyV0,
    pub state_count: usize,
    pub fixed_point_count: usize,
    pub proof: CascadeAttractorBasinProofV0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AttractorEnumerationStrategyV0 {
    Explicit,
    Bdd,
    Lumped,
    Deferred,
    Sampled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RgFixedPointTagV0 {
    Exact,
    Lumped,
    Deferred,
    SampledAdvisory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeAttractorBasinProofV0 {
    pub deterministic: bool,
    pub fixed_point_tag: RgFixedPointTagV0,
    pub conservative: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeOutcomeProjectionRecordV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub lint_codes: Vec<&'static str>,
    pub mode_distribution: GrnModeDistributionV0,
}

pub fn summarize_grn_state(vertices: Vec<GrnVertexStateV0>) -> BooleanGRNStateV0 {
    BooleanGRNStateV0 {
        schema_version: "0",
        product: "omena-cascade.boolean-grn-state",
        layer_marker: "statistical-mechanics",
        top_policy: GrnTopHandlingPolicyV0::ScBoolSeqUnknown,
        vertices,
    }
}

pub fn choose_grn_attractor_strategy(variable_count: usize) -> AttractorEnumerationStrategyV0 {
    match variable_count {
        0..=16 => AttractorEnumerationStrategyV0::Explicit,
        17..=64 if cfg!(feature = "bdd-attractor") => AttractorEnumerationStrategyV0::Bdd,
        17..=256 if cfg!(feature = "naldi-lumping") => AttractorEnumerationStrategyV0::Lumped,
        _ => AttractorEnumerationStrategyV0::Deferred,
    }
}

pub fn prove_cascade_attractor_basin(variable_count: usize) -> CascadeAttractorBasinV0 {
    let strategy = choose_grn_attractor_strategy(variable_count);
    let fixed_point_tag = match strategy {
        AttractorEnumerationStrategyV0::Explicit | AttractorEnumerationStrategyV0::Bdd => {
            RgFixedPointTagV0::Exact
        }
        AttractorEnumerationStrategyV0::Lumped => RgFixedPointTagV0::Lumped,
        AttractorEnumerationStrategyV0::Deferred => RgFixedPointTagV0::Deferred,
        AttractorEnumerationStrategyV0::Sampled => RgFixedPointTagV0::SampledAdvisory,
    };

    CascadeAttractorBasinV0 {
        schema_version: "0",
        product: "omena-cascade.attractor-basin",
        basin_id: format!("grn-v0-{variable_count}"),
        strategy,
        state_count: variable_count,
        fixed_point_count: usize::from(variable_count > 0),
        proof: CascadeAttractorBasinProofV0 {
            deterministic: !matches!(
                strategy,
                AttractorEnumerationStrategyV0::Deferred | AttractorEnumerationStrategyV0::Sampled
            ),
            fixed_point_tag,
            conservative: true,
        },
    }
}

pub fn project_grn_outcome(vertices: &[GrnVertexStateV0]) -> CascadeOutcomeProjectionRecordV0 {
    let mut applied_count = 0;
    let mut losing_but_eligible_count = 0;
    let mut inactive_count = 0;
    let mut top_count = 0;

    for vertex in vertices {
        match vertex.state {
            GrnBooleanState::Applied => applied_count += 1,
            GrnBooleanState::LosingButEligible => losing_but_eligible_count += 1,
            GrnBooleanState::Inactive => inactive_count += 1,
            GrnBooleanState::Top => top_count += 1,
        }
    }

    CascadeOutcomeProjectionRecordV0 {
        schema_version: "0",
        product: "omena-cascade.grn-outcome-projection",
        lint_codes: vec!["cascade.deep-conflict", "cascade.unreachable-rule"],
        mode_distribution: GrnModeDistributionV0 {
            schema_version: "0",
            product: "omena-cascade.grn-mode-distribution",
            applied_count,
            losing_but_eligible_count,
            inactive_count,
            top_count,
        },
    }
}

pub fn grn_shadow_omena_verbs() -> Vec<&'static str> {
    vec![
        "shadow.omena.grnState",
        "shadow.omena.grnAttractorBasin",
        "shadow.omena.grnDeepConflict",
        "shadow.omena.grnUnreachableRule",
        "shadow.omena.grnModeDistribution",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grn_strategy_policy_matches_m4_alpha_bounds() {
        assert_eq!(
            choose_grn_attractor_strategy(16),
            AttractorEnumerationStrategyV0::Explicit
        );
        assert_eq!(
            choose_grn_attractor_strategy(257),
            AttractorEnumerationStrategyV0::Deferred
        );
    }

    #[test]
    fn grn_explicit_attractor_basin_proof_covers_all_n_le_16() {
        for variable_count in 0..=16 {
            let basin = prove_cascade_attractor_basin(variable_count);

            assert_eq!(basin.schema_version, "0");
            assert_eq!(basin.product, "omena-cascade.attractor-basin");
            assert_eq!(basin.strategy, AttractorEnumerationStrategyV0::Explicit);
            assert_eq!(basin.state_count, variable_count);
            assert!(basin.proof.deterministic);
            assert_eq!(basin.proof.fixed_point_tag, RgFixedPointTagV0::Exact);
            assert!(basin.proof.conservative);
        }

        assert_eq!(
            choose_grn_attractor_strategy(17),
            AttractorEnumerationStrategyV0::Deferred
        );
    }

    #[test]
    fn grn_projection_exposes_lint_codes() {
        let projection = project_grn_outcome(&[]);

        assert_eq!(
            projection.lint_codes,
            vec!["cascade.deep-conflict", "cascade.unreachable-rule"]
        );
    }

    #[test]
    fn grn_shadow_omena_verbs_include_required_surface() {
        let verbs = grn_shadow_omena_verbs();

        assert_eq!(verbs.len(), 5);
        assert!(verbs.iter().all(|verb| verb.starts_with("shadow.omena.")));
        assert!(verbs.contains(&"shadow.omena.grnAttractorBasin"));
        assert!(verbs.contains(&"shadow.omena.grnDeepConflict"));
    }

    #[cfg(feature = "bdd-attractor")]
    #[test]
    fn grn_bdd_strategy_is_feature_gated() {
        assert_eq!(
            choose_grn_attractor_strategy(32),
            AttractorEnumerationStrategyV0::Bdd
        );
    }

    #[cfg(feature = "naldi-lumping")]
    #[test]
    fn grn_lumping_strategy_is_feature_gated() {
        assert_eq!(
            choose_grn_attractor_strategy(65),
            AttractorEnumerationStrategyV0::Lumped
        );
    }
}
