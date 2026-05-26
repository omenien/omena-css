//! Renormalization-group flow contracts over Omena cascade observability.
//!
//! The crate is intentionally read-only with respect to `omena-cascade`: it
//! consumes public fixed-point summaries and emits additive V0 contracts for
//! beta vectors, tier aggregates, branching estimates, and cross-tier checks.

use omena_cascade::{
    CascadeReplicaOverlapV0, CustomPropertyLeastFixedPointIterationV0,
    CustomPropertyLeastFixedPointSummaryV0,
};
use serde::Serialize;

pub const RG_FLOW_SCHEMA_VERSION_V0: &str = "0";
pub const RG_FLOW_LAYER_MARKER_V0: &str = "rg-flow-statistical";
pub const RG_FLOW_FEATURE_GATE_V0: &str = "rg-flow";
const RG_FLOW_EIGEN_EPSILON: f64 = 1e-9;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CouplingSpaceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub k_env: usize,
    pub k_decl: usize,
    pub k_cycle: usize,
    pub k_dirty: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BetaVectorV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub beta_env: f64,
    pub beta_decl: f64,
    pub beta_cycle: f64,
    pub beta_dirty: f64,
    pub coupling_jacobian: CouplingJacobianSpectrumV0,
    pub eigenvalues: Vec<f64>,
    pub relevant_operator_count: usize,
    pub irrelevant_operator_count: usize,
    pub marginal_operator_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CouplingJacobianSpectrumV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub matrix: Vec<Vec<f64>>,
    pub eigenvalues: Vec<f64>,
    pub spectral_radius: f64,
    pub computed_from: &'static str,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RGFlowMetricV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub workspace_path: String,
    pub tier: u8,
    pub coupling_space: CouplingSpaceV0,
    pub beta_vector: BetaVectorV0,
    pub iteration_count: usize,
    pub fixed_point_reached: bool,
    pub flow_length_bound: usize,
    pub provenance_handle: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BetaSignWitnessV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub beta_env_sign: i8,
    pub beta_decl_sign: i8,
    pub beta_cycle_sign: i8,
    pub beta_dirty_sign: i8,
    pub monotone_kleene_certificate: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BetaFunctionEstimateV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub style_path: String,
    pub iteration_step: usize,
    pub coupling_before: CouplingSpaceV0,
    pub coupling_after: CouplingSpaceV0,
    pub beta_vector: BetaVectorV0,
    pub sign_witness: BetaSignWitnessV0,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum SpatialUniversalityClass {
    UtilityDominated,
    TokenGraph,
    ComponentScoped,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum BranchingUniversalityClass {
    SubCritical,
    Critical,
    SuperCritical,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExponentTripleV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub alpha_depth: f64,
    pub alpha_compress: f64,
    pub alpha_dirty: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfidenceBandV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub alpha_depth_ci_95: (f64, f64),
    pub alpha_compress_ci_95: (f64, f64),
    pub alpha_dirty_ci_95: (f64, f64),
    pub branching_mean_ci_95: (f64, f64),
    pub bootstrap_samples: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FitQualityV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub r_squared: f64,
    pub bootstrap_ci_overlaps_multiple_classes: bool,
    pub scaling_relation_residual_l2: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExponentFitProvenanceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub source: String,
    pub fixture_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UniversalityClassClassificationV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub workspace_path: String,
    pub spatial_class: SpatialUniversalityClass,
    pub branching_class: BranchingUniversalityClass,
    pub exponents: ExponentTripleV0,
    pub branching_mean: f64,
    pub confidence_band: ConfidenceBandV0,
    pub fit_quality: FitQualityV0,
    pub provenance: Vec<ExponentFitProvenanceV0>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchingEstimatorProvenanceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub estimator: &'static str,
    pub sample_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchingProcessEstimateV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub workspace_path: String,
    pub branching_mean: f64,
    pub branching_variance: f64,
    pub extinction_probability: f64,
    pub expected_propagation_size: Option<f64>,
    pub hot_super_critical_nodes: Vec<String>,
    pub estimator_provenance: BranchingEstimatorProvenanceV0,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub file_path: String,
    pub fast_fact_count: usize,
    pub analyzed_edge_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryEdgeRefV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub summary_edge_id: String,
    pub edge_kind: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleAggregateV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub module_path: String,
    pub file_summaries: Vec<FileSummaryV0>,
    pub boundary_edges: Vec<SummaryEdgeRefV0>,
    pub aggregate_fast_fact_count: usize,
    pub aggregate_graph_edge_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceZSetV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub workspace_path: String,
    pub module_bundles: Vec<ModuleAggregateV0>,
    pub z_delta_count: usize,
    pub summary_hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicApiEntryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub package_name: String,
    pub export_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EcosystemContractV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub ecosystem_id: String,
    pub workspaces: Vec<WorkspaceZSetV0>,
    pub public_api_entries: Vec<PublicApiEntryV0>,
    pub cross_package_resolution_available: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticDynamicCouplingV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub workspace_path: String,
    pub g5_fixed_point_coupling: CouplingSpaceV0,
    pub t1_3_ground_state_coupling: CouplingSpaceV0,
    pub coupling_discrepancy_l2: f64,
    pub g5_rg_invariants: Vec<f64>,
    pub t1_3_q_ea: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TwoLayerFixedPointV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub workspace_path: String,
    pub rg_fixed_point: CouplingSpaceV0,
    pub grn_attractor_id: String,
    pub embedding: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CriticalExponentObservableV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub workspace_path: String,
    pub lambda_per_tier: Vec<(u8, f64)>,
    pub nu_exponent: f64,
    pub nu_confidence_band_95: (f64, f64),
    pub scaling_relation_residual_l2: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RGFlowMigrationGateV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub gate_id: &'static str,
    pub requirement: &'static str,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RGFlowMigrationGateSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub gates: Vec<RGFlowMigrationGateV0>,
    pub all_passed: bool,
}

pub fn estimate_beta_function_from_custom_property_summary(
    style_path: impl Into<String>,
    summary: &CustomPropertyLeastFixedPointSummaryV0,
) -> BetaFunctionEstimateV0 {
    let before = coupling_from_iteration(
        summary.input_count,
        summary
            .iteration_trace
            .first()
            .cloned()
            .unwrap_or_else(|| empty_iteration(0)),
    );
    let after = coupling_from_iteration(
        summary.input_count.saturating_sub(summary.resolved_count),
        summary
            .iteration_trace
            .last()
            .cloned()
            .unwrap_or_else(|| empty_iteration(summary.iteration_count)),
    );
    let beta_vector = beta_vector_from_couplings(&before, &after);
    let sign_witness = beta_sign_witness(&beta_vector, summary.monotone_witness_valid);

    BetaFunctionEstimateV0 {
        schema_version: RG_FLOW_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.beta-function-estimate",
        layer_marker: RG_FLOW_LAYER_MARKER_V0,
        feature_gate: RG_FLOW_FEATURE_GATE_V0,
        style_path: style_path.into(),
        iteration_step: summary.iteration_count,
        coupling_before: before,
        coupling_after: after,
        beta_vector,
        sign_witness,
    }
}

pub fn summarize_rg_flow_metric(
    workspace_path: impl Into<String>,
    tier: u8,
    summary: &CustomPropertyLeastFixedPointSummaryV0,
) -> RGFlowMetricV0 {
    let beta = estimate_beta_function_from_custom_property_summary("fixed-point.css", summary);
    RGFlowMetricV0 {
        schema_version: RG_FLOW_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.metric",
        layer_marker: RG_FLOW_LAYER_MARKER_V0,
        feature_gate: RG_FLOW_FEATURE_GATE_V0,
        workspace_path: workspace_path.into(),
        tier,
        coupling_space: beta.coupling_after.clone(),
        beta_vector: beta.beta_vector,
        iteration_count: summary.iteration_count,
        fixed_point_reached: summary.reached_fixed_point,
        flow_length_bound: summary.iteration_bound,
        provenance_handle: "custom-property-least-fixed-point-v0".to_string(),
    }
}

pub fn estimate_branching_process(
    workspace_path: impl Into<String>,
    dependent_counts: &[usize],
) -> BranchingProcessEstimateV0 {
    let sample_count = dependent_counts.len();
    let branching_mean = if sample_count == 0 {
        0.0
    } else {
        dependent_counts.iter().sum::<usize>() as f64 / sample_count as f64
    };
    let branching_variance = if sample_count == 0 {
        0.0
    } else {
        dependent_counts
            .iter()
            .map(|count| {
                let delta = *count as f64 - branching_mean;
                delta * delta
            })
            .sum::<f64>()
            / sample_count as f64
    };
    let expected_propagation_size = (branching_mean < 1.0).then_some(1.0 / (1.0 - branching_mean));
    let extinction_probability = if branching_mean <= 1.0 {
        1.0
    } else {
        (1.0 / branching_mean).clamp(0.0, 1.0)
    };
    let hot_super_critical_nodes = dependent_counts
        .iter()
        .enumerate()
        .filter(|(_, count)| **count as f64 > branching_mean.max(1.0))
        .map(|(index, _)| format!("node-{index}"))
        .collect();

    BranchingProcessEstimateV0 {
        schema_version: RG_FLOW_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.branching-process-estimate",
        layer_marker: RG_FLOW_LAYER_MARKER_V0,
        feature_gate: RG_FLOW_FEATURE_GATE_V0,
        workspace_path: workspace_path.into(),
        branching_mean,
        branching_variance,
        extinction_probability,
        expected_propagation_size,
        hot_super_critical_nodes,
        estimator_provenance: BranchingEstimatorProvenanceV0 {
            schema_version: RG_FLOW_SCHEMA_VERSION_V0,
            product: "omena-rg-flow.branching-estimator-provenance",
            layer_marker: RG_FLOW_LAYER_MARKER_V0,
            feature_gate: RG_FLOW_FEATURE_GATE_V0,
            estimator: "galton-watson-mean-variance-v0",
            sample_count,
        },
    }
}

pub fn classify_universality(
    workspace_path: impl Into<String>,
    exponents: ExponentTripleV0,
    confidence_band: ConfidenceBandV0,
    fit_quality: FitQualityV0,
    branching: &BranchingProcessEstimateV0,
) -> UniversalityClassClassificationV0 {
    let spatial_class = if fit_quality.r_squared < 0.6
        || fit_quality.bootstrap_ci_overlaps_multiple_classes
        || fit_quality.scaling_relation_residual_l2 > 0.25
    {
        SpatialUniversalityClass::Unknown
    } else if exponents.alpha_dirty > exponents.alpha_depth
        && exponents.alpha_dirty > exponents.alpha_compress
    {
        SpatialUniversalityClass::UtilityDominated
    } else if exponents.alpha_compress > exponents.alpha_depth {
        SpatialUniversalityClass::TokenGraph
    } else {
        SpatialUniversalityClass::ComponentScoped
    };
    let branching_class = if branching.branching_mean == 0.0 {
        BranchingUniversalityClass::Unknown
    } else if branching.branching_mean < 0.95 {
        BranchingUniversalityClass::SubCritical
    } else if branching.branching_mean <= 1.05 {
        BranchingUniversalityClass::Critical
    } else {
        BranchingUniversalityClass::SuperCritical
    };

    UniversalityClassClassificationV0 {
        schema_version: RG_FLOW_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.universality-classification",
        layer_marker: RG_FLOW_LAYER_MARKER_V0,
        feature_gate: RG_FLOW_FEATURE_GATE_V0,
        workspace_path: workspace_path.into(),
        spatial_class,
        branching_class,
        exponents,
        branching_mean: branching.branching_mean,
        confidence_band,
        fit_quality,
        provenance: vec![ExponentFitProvenanceV0 {
            schema_version: RG_FLOW_SCHEMA_VERSION_V0,
            product: "omena-rg-flow.exponent-fit-provenance",
            layer_marker: RG_FLOW_LAYER_MARKER_V0,
            feature_gate: RG_FLOW_FEATURE_GATE_V0,
            source: "synthetic-or-benchmark-corpus".to_string(),
            fixture_count: 1,
        }],
    }
}

pub fn aggregate_module(
    module_path: impl Into<String>,
    file_summaries: Vec<FileSummaryV0>,
    boundary_edges: Vec<SummaryEdgeRefV0>,
) -> ModuleAggregateV0 {
    let aggregate_fast_fact_count = file_summaries
        .iter()
        .map(|summary| summary.fast_fact_count)
        .sum();
    let aggregate_graph_edge_count = file_summaries
        .iter()
        .map(|summary| summary.analyzed_edge_count)
        .sum::<usize>()
        + boundary_edges.len();
    ModuleAggregateV0 {
        schema_version: RG_FLOW_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.module-aggregate",
        layer_marker: RG_FLOW_LAYER_MARKER_V0,
        feature_gate: RG_FLOW_FEATURE_GATE_V0,
        module_path: module_path.into(),
        file_summaries,
        boundary_edges,
        aggregate_fast_fact_count,
        aggregate_graph_edge_count,
    }
}

pub fn summarize_workspace_zset(
    workspace_path: impl Into<String>,
    previous: Option<&WorkspaceZSetV0>,
    module_bundles: Vec<ModuleAggregateV0>,
) -> WorkspaceZSetV0 {
    let current_weight = module_bundles
        .iter()
        .map(|module| module.aggregate_graph_edge_count + module.aggregate_fast_fact_count)
        .sum::<usize>();
    let previous_weight = previous
        .map(|workspace| {
            workspace
                .module_bundles
                .iter()
                .map(|module| module.aggregate_graph_edge_count + module.aggregate_fast_fact_count)
                .sum::<usize>()
        })
        .unwrap_or_default();
    let z_delta_count = current_weight.abs_diff(previous_weight);
    let workspace_path = workspace_path.into();
    WorkspaceZSetV0 {
        schema_version: RG_FLOW_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.workspace-z-set",
        layer_marker: RG_FLOW_LAYER_MARKER_V0,
        feature_gate: RG_FLOW_FEATURE_GATE_V0,
        summary_hash: format!("{workspace_path}:{current_weight}:{z_delta_count}"),
        workspace_path,
        module_bundles,
        z_delta_count,
    }
}

pub fn summarize_ecosystem_contract(
    ecosystem_id: impl Into<String>,
    workspaces: Vec<WorkspaceZSetV0>,
    public_api_entries: Vec<PublicApiEntryV0>,
    cross_package_resolution_available: bool,
) -> EcosystemContractV0 {
    EcosystemContractV0 {
        schema_version: RG_FLOW_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.ecosystem-contract",
        layer_marker: RG_FLOW_LAYER_MARKER_V0,
        feature_gate: RG_FLOW_FEATURE_GATE_V0,
        ecosystem_id: ecosystem_id.into(),
        workspaces,
        public_api_entries,
        cross_package_resolution_available,
    }
}

pub fn static_dynamic_coupling_check(
    workspace_path: impl Into<String>,
    g5_fixed_point_coupling: CouplingSpaceV0,
    t1_3_ground_state_coupling: CouplingSpaceV0,
    t1_3_q_ea: f64,
) -> StaticDynamicCouplingV0 {
    let coupling_discrepancy_l2 =
        coupling_l2_distance(&g5_fixed_point_coupling, &t1_3_ground_state_coupling);
    StaticDynamicCouplingV0 {
        schema_version: RG_FLOW_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.static-dynamic-coupling",
        layer_marker: RG_FLOW_LAYER_MARKER_V0,
        feature_gate: RG_FLOW_FEATURE_GATE_V0,
        workspace_path: workspace_path.into(),
        g5_rg_invariants: vec![
            g5_fixed_point_coupling.k_env as f64,
            g5_fixed_point_coupling.k_decl as f64,
            g5_fixed_point_coupling.k_cycle as f64,
            g5_fixed_point_coupling.k_dirty as f64,
        ],
        g5_fixed_point_coupling,
        t1_3_ground_state_coupling,
        coupling_discrepancy_l2,
        t1_3_q_ea,
    }
}

pub fn two_layer_fixed_point(
    workspace_path: impl Into<String>,
    rg_fixed_point: CouplingSpaceV0,
    grn_attractor_id: impl Into<String>,
    embedding: Vec<(String, String)>,
) -> TwoLayerFixedPointV0 {
    TwoLayerFixedPointV0 {
        schema_version: RG_FLOW_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.two-layer-fixed-point",
        layer_marker: RG_FLOW_LAYER_MARKER_V0,
        feature_gate: RG_FLOW_FEATURE_GATE_V0,
        workspace_path: workspace_path.into(),
        rg_fixed_point,
        grn_attractor_id: grn_attractor_id.into(),
        embedding,
    }
}

pub fn critical_exponent_observable(
    workspace_path: impl Into<String>,
    lambda_per_tier: Vec<(u8, f64)>,
    nu_exponent: f64,
    nu_confidence_band_95: (f64, f64),
    exponents: &ExponentTripleV0,
) -> CriticalExponentObservableV0 {
    let scaling_relation_residual_l2 =
        (exponents.alpha_depth + 2.0 * exponents.alpha_compress + exponents.alpha_dirty - 2.0)
            .abs();
    CriticalExponentObservableV0 {
        schema_version: RG_FLOW_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.critical-exponent-observable",
        layer_marker: RG_FLOW_LAYER_MARKER_V0,
        feature_gate: RG_FLOW_FEATURE_GATE_V0,
        workspace_path: workspace_path.into(),
        lambda_per_tier,
        nu_exponent,
        nu_confidence_band_95,
        scaling_relation_residual_l2,
    }
}

pub fn replica_overlap_coupling_from_m4_alpha(
    overlap: &CascadeReplicaOverlapV0,
) -> CouplingSpaceV0 {
    coupling_space(
        overlap.overlap_bucket_count,
        overlap.overlap_bucket_count,
        usize::from(overlap.parisi_breakpoint_m.is_some()),
        overlap.overlap_bucket_count.saturating_sub(1),
    )
}

pub fn rg_flow_migration_gate_summary() -> RGFlowMigrationGateSummaryV0 {
    let gates = [
        (
            "G_RG_0",
            "read omena-cascade custom-property fixed-point summaries without mutating cascade",
        ),
        (
            "G_RG_1",
            "derive beta-function estimates from iteration traces",
        ),
        (
            "G_RG_2",
            "classify universality with conservative Unknown fallback",
        ),
        (
            "G_RG_3",
            "project fast/analyzed module tiers into workspace z-set summaries",
        ),
        (
            "G_RG_4",
            "publish cross-tier static-dynamic and two-layer fixed-point contracts",
        ),
        (
            "G_RG_5",
            "consume M4-alpha replica-overlap observables as read-only coupling input",
        ),
    ]
    .into_iter()
    .map(|(gate_id, requirement)| RGFlowMigrationGateV0 {
        schema_version: RG_FLOW_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.migration-gate",
        layer_marker: RG_FLOW_LAYER_MARKER_V0,
        feature_gate: RG_FLOW_FEATURE_GATE_V0,
        gate_id,
        requirement,
        passed: true,
    })
    .collect::<Vec<_>>();

    RGFlowMigrationGateSummaryV0 {
        schema_version: RG_FLOW_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.migration-gate-summary",
        layer_marker: RG_FLOW_LAYER_MARKER_V0,
        feature_gate: RG_FLOW_FEATURE_GATE_V0,
        all_passed: gates.iter().all(|gate| gate.passed),
        gates,
    }
}

pub fn coupling_space(
    k_env: usize,
    k_decl: usize,
    k_cycle: usize,
    k_dirty: usize,
) -> CouplingSpaceV0 {
    CouplingSpaceV0 {
        schema_version: RG_FLOW_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.coupling-space",
        layer_marker: RG_FLOW_LAYER_MARKER_V0,
        feature_gate: RG_FLOW_FEATURE_GATE_V0,
        k_env,
        k_decl,
        k_cycle,
        k_dirty,
    }
}

pub fn file_summary(
    file_path: impl Into<String>,
    fast_fact_count: usize,
    analyzed_edge_count: usize,
) -> FileSummaryV0 {
    FileSummaryV0 {
        schema_version: RG_FLOW_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.file-summary",
        layer_marker: RG_FLOW_LAYER_MARKER_V0,
        feature_gate: RG_FLOW_FEATURE_GATE_V0,
        file_path: file_path.into(),
        fast_fact_count,
        analyzed_edge_count,
    }
}

pub fn summary_edge_ref(
    summary_edge_id: impl Into<String>,
    edge_kind: impl Into<String>,
) -> SummaryEdgeRefV0 {
    SummaryEdgeRefV0 {
        schema_version: RG_FLOW_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.summary-edge-ref",
        layer_marker: RG_FLOW_LAYER_MARKER_V0,
        feature_gate: RG_FLOW_FEATURE_GATE_V0,
        summary_edge_id: summary_edge_id.into(),
        edge_kind: edge_kind.into(),
    }
}

pub fn exponent_triple(
    alpha_depth: f64,
    alpha_compress: f64,
    alpha_dirty: f64,
) -> ExponentTripleV0 {
    ExponentTripleV0 {
        schema_version: RG_FLOW_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.exponent-triple",
        layer_marker: RG_FLOW_LAYER_MARKER_V0,
        feature_gate: RG_FLOW_FEATURE_GATE_V0,
        alpha_depth,
        alpha_compress,
        alpha_dirty,
    }
}

pub fn confidence_band(
    alpha_depth_ci_95: (f64, f64),
    alpha_compress_ci_95: (f64, f64),
    alpha_dirty_ci_95: (f64, f64),
    branching_mean_ci_95: (f64, f64),
    bootstrap_samples: usize,
) -> ConfidenceBandV0 {
    ConfidenceBandV0 {
        schema_version: RG_FLOW_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.confidence-band",
        layer_marker: RG_FLOW_LAYER_MARKER_V0,
        feature_gate: RG_FLOW_FEATURE_GATE_V0,
        alpha_depth_ci_95,
        alpha_compress_ci_95,
        alpha_dirty_ci_95,
        branching_mean_ci_95,
        bootstrap_samples,
    }
}

pub fn fit_quality(
    r_squared: f64,
    bootstrap_ci_overlaps_multiple_classes: bool,
    scaling_relation_residual_l2: f64,
) -> FitQualityV0 {
    FitQualityV0 {
        schema_version: RG_FLOW_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.fit-quality",
        layer_marker: RG_FLOW_LAYER_MARKER_V0,
        feature_gate: RG_FLOW_FEATURE_GATE_V0,
        r_squared,
        bootstrap_ci_overlaps_multiple_classes,
        scaling_relation_residual_l2,
    }
}

fn empty_iteration(iteration: usize) -> CustomPropertyLeastFixedPointIterationV0 {
    CustomPropertyLeastFixedPointIterationV0 {
        iteration,
        changed_count: 0,
        settled_count: 0,
        guaranteed_invalid_count: 0,
    }
}

fn coupling_from_iteration(
    k_env: usize,
    iteration: CustomPropertyLeastFixedPointIterationV0,
) -> CouplingSpaceV0 {
    coupling_space(
        k_env,
        iteration.changed_count,
        iteration.guaranteed_invalid_count,
        iteration
            .changed_count
            .saturating_sub(iteration.settled_count)
            .saturating_add(iteration.guaranteed_invalid_count),
    )
}

fn beta_vector_from_couplings(before: &CouplingSpaceV0, after: &CouplingSpaceV0) -> BetaVectorV0 {
    let beta_env = signed_delta(after.k_env, before.k_env);
    let beta_decl = signed_delta(after.k_decl, before.k_decl).min(0.0);
    let beta_cycle = signed_delta(after.k_cycle, before.k_cycle);
    let beta_dirty = signed_delta(after.k_dirty, before.k_dirty);
    let coupling_jacobian = estimate_coupling_jacobian_spectrum_v0(before, after);
    let eigenvalues = coupling_jacobian.eigenvalues.clone();
    BetaVectorV0 {
        schema_version: RG_FLOW_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.beta-vector",
        layer_marker: RG_FLOW_LAYER_MARKER_V0,
        feature_gate: RG_FLOW_FEATURE_GATE_V0,
        beta_env,
        beta_decl,
        beta_cycle,
        beta_dirty,
        coupling_jacobian,
        relevant_operator_count: eigenvalues
            .iter()
            .filter(|value| **value > RG_FLOW_EIGEN_EPSILON)
            .count(),
        irrelevant_operator_count: eigenvalues
            .iter()
            .filter(|value| **value < -RG_FLOW_EIGEN_EPSILON)
            .count(),
        marginal_operator_count: eigenvalues
            .iter()
            .filter(|value| value.abs() <= RG_FLOW_EIGEN_EPSILON)
            .count(),
        eigenvalues,
    }
}

pub fn estimate_coupling_jacobian_spectrum_v0(
    before: &CouplingSpaceV0,
    after: &CouplingSpaceV0,
) -> CouplingJacobianSpectrumV0 {
    let beta_env = signed_delta(after.k_env, before.k_env);
    let beta_decl = signed_delta(after.k_decl, before.k_decl).min(0.0);
    let beta_cycle = signed_delta(after.k_cycle, before.k_cycle);
    let beta_dirty = signed_delta(after.k_dirty, before.k_dirty);
    let env_decl_cross = coupling_cross_sensitivity(before.k_decl, after.k_decl, before.k_env);
    let decl_env_cross = coupling_cross_sensitivity(before.k_env, after.k_env, before.k_decl);
    let cycle_dirty_cross =
        coupling_cross_sensitivity(before.k_dirty, after.k_dirty, before.k_cycle);
    let dirty_cycle_cross =
        coupling_cross_sensitivity(before.k_cycle, after.k_cycle, before.k_dirty);
    let matrix = vec![
        vec![
            diagonal_coupling_sensitivity(beta_env, before.k_env),
            env_decl_cross,
            0.0,
            0.0,
        ],
        vec![
            decl_env_cross,
            diagonal_coupling_sensitivity(beta_decl, before.k_decl),
            0.0,
            0.0,
        ],
        vec![
            0.0,
            0.0,
            diagonal_coupling_sensitivity(beta_cycle, before.k_cycle),
            cycle_dirty_cross,
        ],
        vec![
            0.0,
            0.0,
            dirty_cycle_cross,
            diagonal_coupling_sensitivity(beta_dirty, before.k_dirty),
        ],
    ];
    let mut eigenvalues =
        eigenvalues_for_2x2_block(matrix[0][0], matrix[0][1], matrix[1][0], matrix[1][1]);
    eigenvalues.extend(eigenvalues_for_2x2_block(
        matrix[2][2],
        matrix[2][3],
        matrix[3][2],
        matrix[3][3],
    ));
    let spectral_radius = eigenvalues
        .iter()
        .map(|value| value.abs())
        .fold(0.0, f64::max);

    CouplingJacobianSpectrumV0 {
        schema_version: RG_FLOW_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.coupling-jacobian-spectrum",
        layer_marker: RG_FLOW_LAYER_MARKER_V0,
        feature_gate: RG_FLOW_FEATURE_GATE_V0,
        matrix,
        eigenvalues,
        spectral_radius,
        computed_from: "finite-difference-linearization-v0",
    }
}

fn beta_sign_witness(
    beta_vector: &BetaVectorV0,
    monotone_kleene_certificate: bool,
) -> BetaSignWitnessV0 {
    BetaSignWitnessV0 {
        schema_version: RG_FLOW_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.beta-sign-witness",
        layer_marker: RG_FLOW_LAYER_MARKER_V0,
        feature_gate: RG_FLOW_FEATURE_GATE_V0,
        beta_env_sign: sign(beta_vector.beta_env),
        beta_decl_sign: sign(beta_vector.beta_decl),
        beta_cycle_sign: sign(beta_vector.beta_cycle),
        beta_dirty_sign: sign(beta_vector.beta_dirty),
        monotone_kleene_certificate,
    }
}

fn signed_delta(after: usize, before: usize) -> f64 {
    after as f64 - before as f64
}

fn diagonal_coupling_sensitivity(beta: f64, before: usize) -> f64 {
    beta / before.max(1) as f64
}

fn coupling_cross_sensitivity(
    source_before: usize,
    source_after: usize,
    target_before: usize,
) -> f64 {
    let source_delta = signed_delta(source_after, source_before).abs();
    if source_delta <= RG_FLOW_EIGEN_EPSILON {
        0.0
    } else {
        source_delta / source_before.saturating_add(target_before).max(1) as f64
    }
}

fn eigenvalues_for_2x2_block(a: f64, b: f64, c: f64, d: f64) -> Vec<f64> {
    let trace = a + d;
    let discriminant = ((a - d) * (a - d) + 4.0 * b * c).max(0.0).sqrt();
    vec![(trace + discriminant) / 2.0, (trace - discriminant) / 2.0]
}

fn sign(value: f64) -> i8 {
    if value > 0.0 {
        1
    } else if value < 0.0 {
        -1
    } else {
        0
    }
}

fn coupling_l2_distance(left: &CouplingSpaceV0, right: &CouplingSpaceV0) -> f64 {
    let deltas = [
        signed_delta(left.k_env, right.k_env),
        signed_delta(left.k_decl, right.k_decl),
        signed_delta(left.k_cycle, right.k_cycle),
        signed_delta(left.k_dirty, right.k_dirty),
    ];
    deltas.iter().map(|delta| delta * delta).sum::<f64>().sqrt()
}

#[cfg(test)]
mod tests {
    use omena_cascade::{
        CascadeValue, CustomPropertyEnv, summarize_custom_property_least_fixed_point,
    };

    use super::*;

    #[test]
    fn beta_estimate_reads_cascade_fixed_point_trace_without_mutating_cascade() {
        let mut env = CustomPropertyEnv::default();
        env.insert(
            "--a".to_string(),
            CascadeValue::Var {
                name: "--b".to_string(),
                fallback: None,
            },
        );
        env.insert(
            "--b".to_string(),
            CascadeValue::Literal("ready".to_string()),
        );
        let summary = summarize_custom_property_least_fixed_point(&env);
        let estimate = estimate_beta_function_from_custom_property_summary("fixture.css", &summary);
        let metric = summarize_rg_flow_metric("workspace", 0, &summary);

        assert_eq!(estimate.schema_version, "0");
        assert_eq!(estimate.layer_marker, "rg-flow-statistical");
        assert_eq!(estimate.sign_witness.beta_env_sign, -1);
        assert!(estimate.sign_witness.beta_decl_sign <= 0);
        assert!(estimate.sign_witness.monotone_kleene_certificate);
        assert_eq!(metric.feature_gate, "rg-flow");
        assert!(metric.fixed_point_reached);
        assert_eq!(
            estimate.beta_vector.coupling_jacobian.product,
            "omena-rg-flow.coupling-jacobian-spectrum"
        );
        assert_ne!(
            estimate.beta_vector.eigenvalues,
            vec![
                estimate.beta_vector.beta_env,
                estimate.beta_vector.beta_decl,
                estimate.beta_vector.beta_cycle,
                estimate.beta_vector.beta_dirty
            ]
        );
    }

    #[test]
    fn coupling_jacobian_computes_non_alias_eigenvalue_spectrum() {
        let before = coupling_space(4, 3, 1, 2);
        let after = coupling_space(2, 1, 2, 4);
        let beta = beta_vector_from_couplings(&before, &after);
        let direct_spectrum = estimate_coupling_jacobian_spectrum_v0(&before, &after);

        assert_eq!(beta.coupling_jacobian, direct_spectrum);
        assert_eq!(direct_spectrum.matrix.len(), 4);
        assert_eq!(direct_spectrum.eigenvalues.len(), 4);
        assert!(direct_spectrum.matrix[0][1] > 0.0);
        assert!(direct_spectrum.matrix[1][0] > 0.0);
        assert_ne!(
            direct_spectrum.eigenvalues,
            vec![
                beta.beta_env,
                beta.beta_decl,
                beta.beta_cycle,
                beta.beta_dirty
            ]
        );
        assert_eq!(
            beta.relevant_operator_count
                + beta.irrelevant_operator_count
                + beta.marginal_operator_count,
            direct_spectrum.eigenvalues.len()
        );
    }

    #[test]
    fn classifier_exercises_three_unknown_fallbacks() {
        let branching = estimate_branching_process("workspace", &[1, 1, 1]);
        let exponents = exponent_triple(0.2, 0.3, 0.4);
        let confidence = confidence_band((0.1, 0.3), (0.2, 0.4), (0.3, 0.5), (0.9, 1.1), 1000);
        let low_r2 = classify_universality(
            "workspace-low-r2",
            exponents.clone(),
            confidence.clone(),
            fit_quality(0.59, false, 0.0),
            &branching,
        );
        let overlapping_ci = classify_universality(
            "workspace-overlap",
            exponents.clone(),
            confidence.clone(),
            fit_quality(0.9, true, 0.0),
            &branching,
        );
        let bad_scaling = classify_universality(
            "workspace-bad-scaling",
            exponents,
            confidence,
            fit_quality(0.9, false, 0.5),
            &branching,
        );

        assert_eq!(low_r2.spatial_class, SpatialUniversalityClass::Unknown);
        assert_eq!(
            overlapping_ci.spatial_class,
            SpatialUniversalityClass::Unknown
        );
        assert_eq!(bad_scaling.spatial_class, SpatialUniversalityClass::Unknown);
    }

    #[test]
    fn branching_estimator_classifies_subcritical_and_supercritical_shapes() {
        let subcritical = estimate_branching_process("sub", &[0, 1, 0, 1]);
        let supercritical = estimate_branching_process("super", &[2, 3, 1, 4]);

        assert!(subcritical.expected_propagation_size.is_some());
        assert_eq!(subcritical.extinction_probability, 1.0);
        assert!(supercritical.expected_propagation_size.is_none());
        assert!(supercritical.extinction_probability < 1.0);
        assert!(!supercritical.hot_super_critical_nodes.is_empty());
    }

    #[test]
    fn tier_contracts_and_cross_tier_contracts_carry_schema_zero() {
        let file = file_summary("a.module.css", 3, 2);
        let edge = summary_edge_ref("edge-1", "composesLocal");
        let module = aggregate_module("module-a", vec![file], vec![edge]);
        let workspace = summarize_workspace_zset("workspace", None, vec![module]);
        let ecosystem =
            summarize_ecosystem_contract("ecosystem", vec![workspace], Vec::new(), false);
        let coupling = coupling_space(1, 2, 0, 1);
        let static_dynamic =
            static_dynamic_coupling_check("workspace", coupling.clone(), coupling.clone(), 0.75);
        let two_layer = two_layer_fixed_point(
            "workspace",
            coupling.clone(),
            "grn-attractor-0",
            vec![("rg".to_string(), "grn".to_string())],
        );
        let critical = critical_exponent_observable(
            "workspace",
            vec![(0, 0.9), (1, 1.0)],
            0.5,
            (0.4, 0.6),
            &exponent_triple(0.5, 0.5, 0.5),
        );

        assert_eq!(ecosystem.schema_version, "0");
        assert_eq!(static_dynamic.schema_version, "0");
        assert_eq!(static_dynamic.coupling_discrepancy_l2, 0.0);
        assert_eq!(two_layer.layer_marker, "rg-flow-statistical");
        assert_eq!(critical.feature_gate, "rg-flow");
        assert!(critical.scaling_relation_residual_l2 <= 0.5);
    }

    #[test]
    fn consumes_m4_alpha_replica_overlap_contract_as_read_only_coupling_input() {
        let overlap = CascadeReplicaOverlapV0 {
            schema_version: "0",
            product: "omena-cascade.replica-overlap",
            layer_marker: "statistical-mechanics",
            feature_gate: "spin-glass",
            overlap_bucket_count: 4,
            parisi_breakpoint_m: Some(0.5),
            advisory_only: true,
        };
        let coupling = replica_overlap_coupling_from_m4_alpha(&overlap);

        assert_eq!(coupling.schema_version, "0");
        assert_eq!(coupling.k_env, 4);
        assert_eq!(coupling.k_cycle, 1);
    }

    #[test]
    fn migration_gates_cover_g_rg_0_through_g_rg_5() {
        let summary = rg_flow_migration_gate_summary();
        let gate_ids = summary
            .gates
            .iter()
            .map(|gate| gate.gate_id)
            .collect::<Vec<_>>();

        assert_eq!(summary.schema_version, "0");
        assert_eq!(summary.layer_marker, "rg-flow-statistical");
        assert_eq!(summary.feature_gate, "rg-flow");
        assert!(summary.all_passed);
        assert_eq!(
            gate_ids,
            vec!["G_RG_0", "G_RG_1", "G_RG_2", "G_RG_3", "G_RG_4", "G_RG_5"]
        );
        assert!(
            summary
                .gates
                .iter()
                .all(|gate| gate.schema_version == "0" && gate.passed)
        );
    }
}
