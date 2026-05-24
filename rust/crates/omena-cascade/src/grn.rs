use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BooleanGRNStateV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
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
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
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
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub basin_id: String,
    pub strategy: AttractorEnumerationStrategyV0,
    pub variable_count: usize,
    pub state_count: usize,
    pub transition_count: usize,
    pub fixed_point_count: usize,
    pub fixed_point_states: Vec<u64>,
    pub transition_digest: Option<String>,
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
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub deterministic: bool,
    pub fixed_point_tag: RgFixedPointTagV0,
    pub conservative: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GrnTransitionRecordV0 {
    pub from_state: u64,
    pub to_state: u64,
    pub fixed_point: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GrnExplicitAttractorEnumerationV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub variable_count: usize,
    pub state_count: usize,
    pub transition_count: usize,
    pub fixed_point_count: usize,
    pub fixed_point_states: Vec<u64>,
    pub transition_digest: String,
    pub complete: bool,
    pub transitions: Vec<GrnTransitionRecordV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeOutcomeProjectionRecordV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub lint_codes: Vec<&'static str>,
    pub mode_distribution: GrnModeDistributionV0,
    pub deep_conflict_report: CascadeDeepConflictReportV0,
    pub kauffman_regime: KauffmanRegimeV0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum KauffmanRegimeKindV0 {
    Ordered,
    Critical,
    Chaotic,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KauffmanRegimeV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub variable_count: usize,
    pub regime: KauffmanRegimeKindV0,
    pub conservative: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeDeepConflictReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub lint_code: &'static str,
    pub conflicting_vertex_count: usize,
    pub conservative: bool,
    pub regime: KauffmanRegimeV0,
}

pub fn summarize_grn_state(vertices: Vec<GrnVertexStateV0>) -> BooleanGRNStateV0 {
    BooleanGRNStateV0 {
        schema_version: "0",
        product: "omena-cascade.boolean-grn-state",
        layer_marker: "statistical-mechanics",
        feature_gate: "grn",
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

pub fn transition_cascade_grn_state_v0(variable_count: usize, state: u64) -> u64 {
    let mask = grn_state_mask(variable_count);
    let active = state & mask;
    active & active.wrapping_neg()
}

pub fn enumerate_explicit_grn_attractor_v0(
    variable_count: usize,
) -> GrnExplicitAttractorEnumerationV0 {
    assert!(
        variable_count <= 16,
        "explicit GRN state enumeration is bounded to n <= 16"
    );
    let state_count = 1usize << variable_count;
    let mut fixed_point_states = Vec::new();
    let mut transitions = Vec::with_capacity(state_count);

    for from_state in 0..state_count {
        let from_state = from_state as u64;
        let to_state = transition_cascade_grn_state_v0(variable_count, from_state);
        let fixed_point = from_state == to_state;
        if fixed_point {
            fixed_point_states.push(from_state);
        }
        transitions.push(GrnTransitionRecordV0 {
            from_state,
            to_state,
            fixed_point,
        });
    }

    let transition_digest = digest_grn_transitions(
        variable_count,
        state_count,
        fixed_point_states.len(),
        &transitions,
    );

    GrnExplicitAttractorEnumerationV0 {
        schema_version: "0",
        product: "omena-cascade.grn-explicit-attractor-enumeration",
        layer_marker: "statistical-mechanics",
        feature_gate: "grn",
        variable_count,
        state_count,
        transition_count: transitions.len(),
        fixed_point_count: fixed_point_states.len(),
        fixed_point_states,
        transition_digest,
        complete: true,
        transitions,
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
    let explicit = matches!(strategy, AttractorEnumerationStrategyV0::Explicit)
        .then(|| enumerate_explicit_grn_attractor_v0(variable_count));
    let state_count = explicit
        .as_ref()
        .map_or(0, |enumeration| enumeration.state_count);
    let transition_count = explicit
        .as_ref()
        .map_or(0, |enumeration| enumeration.transition_count);
    let fixed_point_states = explicit.as_ref().map_or_else(Vec::new, |enumeration| {
        enumeration.fixed_point_states.clone()
    });
    let fixed_point_count = fixed_point_states.len();
    let transition_digest = explicit
        .as_ref()
        .map(|enumeration| enumeration.transition_digest.clone());

    CascadeAttractorBasinV0 {
        schema_version: "0",
        product: "omena-cascade.attractor-basin",
        layer_marker: "statistical-mechanics",
        feature_gate: "grn",
        basin_id: format!("grn-v0-{variable_count}"),
        strategy,
        variable_count,
        state_count,
        transition_count,
        fixed_point_count,
        fixed_point_states,
        transition_digest,
        proof: CascadeAttractorBasinProofV0 {
            schema_version: "0",
            product: "omena-cascade.attractor-basin-proof",
            layer_marker: "statistical-mechanics",
            feature_gate: "grn",
            deterministic: !matches!(
                strategy,
                AttractorEnumerationStrategyV0::Deferred | AttractorEnumerationStrategyV0::Sampled
            ),
            fixed_point_tag,
            conservative: true,
        },
    }
}

fn grn_state_mask(variable_count: usize) -> u64 {
    assert!(
        variable_count <= 16,
        "GRN state bitset helper is bounded to n <= 16"
    );
    if variable_count == 0 {
        0
    } else {
        (1u64 << variable_count) - 1
    }
}

fn digest_grn_transitions(
    variable_count: usize,
    state_count: usize,
    fixed_point_count: usize,
    transitions: &[GrnTransitionRecordV0],
) -> String {
    let mut digest = 0xcbf29ce484222325u64;
    for transition in transitions {
        digest ^= transition.from_state;
        digest = digest.wrapping_mul(0x100000001b3);
        digest ^= transition.to_state.rotate_left(17);
        digest = digest.wrapping_mul(0x100000001b3);
        digest ^= u64::from(transition.fixed_point);
        digest = digest.wrapping_mul(0x100000001b3);
    }
    format!("grn-v0-{variable_count}-{state_count}-{fixed_point_count}-{digest:016x}")
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

    let kauffman_regime = classify_kauffman_regime(vertices.len());

    CascadeOutcomeProjectionRecordV0 {
        schema_version: "0",
        product: "omena-cascade.grn-outcome-projection",
        layer_marker: "statistical-mechanics",
        feature_gate: "grn",
        lint_codes: vec!["cascade.deep-conflict", "cascade.unreachable-rule"],
        mode_distribution: GrnModeDistributionV0 {
            schema_version: "0",
            product: "omena-cascade.grn-mode-distribution",
            layer_marker: "statistical-mechanics",
            feature_gate: "grn",
            applied_count,
            losing_but_eligible_count,
            inactive_count,
            top_count,
        },
        deep_conflict_report: CascadeDeepConflictReportV0 {
            schema_version: "0",
            product: "omena-cascade.deep-conflict-report",
            layer_marker: "statistical-mechanics",
            feature_gate: "grn",
            lint_code: "cascade.deep-conflict",
            conflicting_vertex_count: losing_but_eligible_count,
            conservative: true,
            regime: kauffman_regime.clone(),
        },
        kauffman_regime,
    }
}

pub fn classify_kauffman_regime(variable_count: usize) -> KauffmanRegimeV0 {
    let regime = match variable_count {
        0..=16 => KauffmanRegimeKindV0::Ordered,
        17..=64 => KauffmanRegimeKindV0::Critical,
        65..=256 => KauffmanRegimeKindV0::Chaotic,
        _ => KauffmanRegimeKindV0::Unknown,
    };

    KauffmanRegimeV0 {
        schema_version: "0",
        product: "omena-cascade.kauffman-regime",
        layer_marker: "statistical-mechanics",
        feature_gate: "grn",
        variable_count,
        regime,
        conservative: true,
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
        let state = summarize_grn_state(Vec::new());

        assert_eq!(state.layer_marker, "statistical-mechanics");
        assert_eq!(state.feature_gate, "grn");
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
            let expected_state_count = 1usize << variable_count;

            assert_eq!(basin.schema_version, "0");
            assert_eq!(basin.product, "omena-cascade.attractor-basin");
            assert_eq!(basin.layer_marker, "statistical-mechanics");
            assert_eq!(basin.feature_gate, "grn");
            assert_eq!(basin.strategy, AttractorEnumerationStrategyV0::Explicit);
            assert_eq!(basin.variable_count, variable_count);
            assert_eq!(basin.state_count, expected_state_count);
            assert_eq!(basin.transition_count, expected_state_count);
            assert_eq!(basin.fixed_point_count, variable_count + 1);
            assert_eq!(basin.fixed_point_states.len(), variable_count + 1);
            assert!(basin.fixed_point_states.contains(&0));
            assert!(basin.transition_digest.as_deref().is_some_and(|digest| {
                digest.starts_with(&format!(
                    "grn-v0-{variable_count}-{expected_state_count}-{}-",
                    variable_count + 1
                ))
            }));
            assert_eq!(basin.proof.schema_version, "0");
            assert_eq!(basin.proof.feature_gate, "grn");
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
    fn grn_explicit_transition_function_enumerates_full_state_space() {
        let enumeration = enumerate_explicit_grn_attractor_v0(3);

        assert_eq!(
            enumeration.product,
            "omena-cascade.grn-explicit-attractor-enumeration"
        );
        assert!(enumeration.complete);
        assert_eq!(enumeration.variable_count, 3);
        assert_eq!(enumeration.state_count, 8);
        assert_eq!(enumeration.transition_count, 8);
        assert_eq!(enumeration.fixed_point_states, vec![0, 1, 2, 4]);
        assert_eq!(transition_cascade_grn_state_v0(3, 0b000), 0b000);
        assert_eq!(transition_cascade_grn_state_v0(3, 0b001), 0b001);
        assert_eq!(transition_cascade_grn_state_v0(3, 0b110), 0b010);
        assert_eq!(transition_cascade_grn_state_v0(3, 0b111), 0b001);
        assert_eq!(
            enumeration
                .transitions
                .iter()
                .filter(|transition| transition.fixed_point)
                .count(),
            4
        );
    }

    #[test]
    fn grn_projection_exposes_lint_codes() {
        let projection = project_grn_outcome(&[]);

        assert_eq!(projection.feature_gate, "grn");
        assert_eq!(projection.layer_marker, "statistical-mechanics");
        assert_eq!(projection.mode_distribution.feature_gate, "grn");
        assert_eq!(projection.deep_conflict_report.schema_version, "0");
        assert_eq!(
            projection.deep_conflict_report.lint_code,
            "cascade.deep-conflict"
        );
        assert_eq!(projection.kauffman_regime.schema_version, "0");
        assert_eq!(projection.kauffman_regime.feature_gate, "grn");
        assert_eq!(
            projection.kauffman_regime.regime,
            KauffmanRegimeKindV0::Ordered
        );
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
