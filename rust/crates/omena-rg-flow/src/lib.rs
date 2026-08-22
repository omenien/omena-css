//! Design-system complexity diagnostic scaffold over Omena cascade observability.
//!
//! The crate is intentionally read-only with respect to `omena-cascade`: it
//! consumes public fixed-point summaries and emits additive V0 contracts for
//! beta vectors, tier aggregates, branching estimates, and cross-tier checks.
//!
//! claim_level: opt-in deep-analysis Jacobian-spectrum approximation,
//! deduplicated against the circular-var warning, not a default product decision
//! mechanism and not a mathematical scale-transformation theorem.

use omena_cascade::{
    CascadeReplicaOverlapV0, CustomPropertyLeastFixedPointIterationV0,
    CustomPropertyLeastFixedPointSummaryV0,
};
use serde::Serialize;

pub const MULTISCALE_COMPLEXITY_HEURISTIC_SCHEMA_VERSION_V0: &str = "0";
#[deprecated(
    since = "0.4.0",
    note = "legacy layer byte owned by omena-rg-flow maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
const LEGACY_MULTISCALE_COMPLEXITY_LAYER_BYTES_V0: &str = "rg-flow-statistical";
#[deprecated(
    since = "0.4.0",
    note = "legacy feature byte owned by omena-rg-flow maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
const LEGACY_MULTISCALE_COMPLEXITY_FEATURE_BYTES_V0: &str = "rg-flow";
#[allow(deprecated)]
const MULTISCALE_COMPLEXITY_HEURISTIC_COMPATIBILITY_LAYER_MARKER_V0: &str =
    LEGACY_MULTISCALE_COMPLEXITY_LAYER_BYTES_V0;
#[allow(deprecated)]
const MULTISCALE_COMPLEXITY_HEURISTIC_COMPATIBILITY_FEATURE_GATE_V0: &str =
    LEGACY_MULTISCALE_COMPLEXITY_FEATURE_BYTES_V0;
pub const MULTISCALE_COMPLEXITY_HEURISTIC_LAYER_MARKER_V0: &str =
    "multiscale-complexity-heuristic-statistical";
pub const MULTISCALE_COMPLEXITY_HEURISTIC_FEATURE_GATE_V0: &str = "multiscale-complexity-heuristic";
pub const MULTISCALE_COMPLEXITY_HEURISTIC_MECHANISM_SCOPE_V0: &str =
    "optInDeepAnalysisJacobianSpectrumHintSubstrate";
pub const MULTISCALE_COMPLEXITY_HEURISTIC_PRODUCT_SURFACE_V0: &str =
    "deepAnalysisCascadeSensitivityHint";
pub const MULTISCALE_COMPLEXITY_HEURISTIC_DEFAULT_PRODUCT_DECISION_MECHANISM_V0: bool = false;
const MULTISCALE_COMPLEXITY_HEURISTIC_EIGEN_EPSILON: f64 = 1e-9;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MultiscaleComplexityWireV0 {
    Compatibility,
    Canonical,
}

impl MultiscaleComplexityWireV0 {
    const fn layer_marker(self) -> &'static str {
        match self {
            Self::Compatibility => MULTISCALE_COMPLEXITY_HEURISTIC_COMPATIBILITY_LAYER_MARKER_V0,
            Self::Canonical => MULTISCALE_COMPLEXITY_HEURISTIC_LAYER_MARKER_V0,
        }
    }

    const fn feature_gate(self) -> &'static str {
        match self {
            Self::Compatibility => MULTISCALE_COMPLEXITY_HEURISTIC_COMPATIBILITY_FEATURE_GATE_V0,
            Self::Canonical => MULTISCALE_COMPLEXITY_HEURISTIC_FEATURE_GATE_V0,
        }
    }
}

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
    pub mechanism_scope: &'static str,
    pub product_surface: &'static str,
    pub default_product_decision_mechanism: bool,
    pub matrix: Vec<Vec<f64>>,
    pub eigenvalues: Vec<f64>,
    pub spectral_radius: f64,
    pub computed_from: &'static str,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiscaleComplexityHeuristicMetricV0 {
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
    pub observed_flow_step_count: usize,
    pub observed_flow_length_l1: f64,
    pub fixed_point_residual_l1: usize,
    pub fixed_point_verified_from_trace: bool,
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
pub struct MultiscaleComplexityHeuristicMigrationGateV0 {
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
pub struct MultiscaleComplexityHeuristicMigrationGateSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub gates: Vec<MultiscaleComplexityHeuristicMigrationGateV0>,
    pub all_passed: bool,
}

#[deprecated(
    since = "0.4.0",
    note = "use estimate_multiscale_complexity_heuristic_beta_function_from_custom_property_summary; compatibility surface owned by omena-rg-flow maintainers and removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn estimate_beta_function_from_custom_property_summary(
    style_path: impl Into<String>,
    summary: &CustomPropertyLeastFixedPointSummaryV0,
) -> BetaFunctionEstimateV0 {
    estimate_beta_function_with_wire_v0(
        style_path,
        summary,
        MultiscaleComplexityWireV0::Compatibility,
    )
}

fn estimate_beta_function_with_wire_v0(
    style_path: impl Into<String>,
    summary: &CustomPropertyLeastFixedPointSummaryV0,
    wire: MultiscaleComplexityWireV0,
) -> BetaFunctionEstimateV0 {
    let before = coupling_from_iteration(
        summary.input_count,
        summary
            .iteration_trace
            .first()
            .cloned()
            .unwrap_or_else(|| empty_iteration(0)),
        wire,
    );
    let after = coupling_from_iteration(
        summary.input_count.saturating_sub(summary.resolved_count),
        summary
            .iteration_trace
            .last()
            .cloned()
            .unwrap_or_else(|| empty_iteration(summary.iteration_count)),
        wire,
    );
    let beta_vector = beta_vector_from_couplings(&before, &after, wire);
    let sign_witness = beta_sign_witness(&beta_vector, summary.monotone_witness_valid, wire);

    BetaFunctionEstimateV0 {
        schema_version: MULTISCALE_COMPLEXITY_HEURISTIC_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.beta-function-estimate",
        layer_marker: wire.layer_marker(),
        feature_gate: wire.feature_gate(),
        style_path: style_path.into(),
        iteration_step: summary.iteration_count,
        coupling_before: before,
        coupling_after: after,
        beta_vector,
        sign_witness,
    }
}

/// Additive canonical beta-function projection with accurate diagnostic-layer
/// bytes. The pre-existing neutral function above retains its V0 wire bytes.
pub fn estimate_multiscale_complexity_heuristic_beta_function_from_custom_property_summary(
    style_path: impl Into<String>,
    summary: &CustomPropertyLeastFixedPointSummaryV0,
) -> BetaFunctionEstimateV0 {
    estimate_beta_function_with_wire_v0(style_path, summary, MultiscaleComplexityWireV0::Canonical)
}

pub fn summarize_multiscale_complexity_heuristic_metric(
    workspace_path: impl Into<String>,
    tier: u8,
    summary: &CustomPropertyLeastFixedPointSummaryV0,
) -> MultiscaleComplexityHeuristicMetricV0 {
    let beta = estimate_multiscale_complexity_heuristic_beta_function_from_custom_property_summary(
        "fixed-point.css",
        summary,
    );
    let observed_flow = observed_fixed_point_flow(summary);
    MultiscaleComplexityHeuristicMetricV0 {
        schema_version: MULTISCALE_COMPLEXITY_HEURISTIC_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.metric",
        layer_marker: MULTISCALE_COMPLEXITY_HEURISTIC_LAYER_MARKER_V0,
        feature_gate: MULTISCALE_COMPLEXITY_HEURISTIC_FEATURE_GATE_V0,
        workspace_path: workspace_path.into(),
        tier,
        coupling_space: beta.coupling_after.clone(),
        beta_vector: beta.beta_vector,
        iteration_count: summary.iteration_count,
        fixed_point_reached: summary.reached_fixed_point,
        flow_length_bound: summary.iteration_bound,
        observed_flow_step_count: observed_flow.step_count,
        observed_flow_length_l1: observed_flow.length_l1,
        fixed_point_residual_l1: observed_flow.fixed_point_residual_l1,
        fixed_point_verified_from_trace: observed_flow.fixed_point_verified_from_trace,
        provenance_handle: "custom-property-least-fixed-point-v0".to_string(),
    }
}

#[deprecated(
    since = "0.4.0",
    note = "use estimate_multiscale_complexity_heuristic_branching_process; compatibility surface owned by omena-rg-flow maintainers and removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn estimate_branching_process(
    workspace_path: impl Into<String>,
    dependent_counts: &[usize],
) -> BranchingProcessEstimateV0 {
    estimate_branching_process_with_wire_v0(
        workspace_path,
        dependent_counts,
        MultiscaleComplexityWireV0::Compatibility,
    )
}

pub fn estimate_multiscale_complexity_heuristic_branching_process(
    workspace_path: impl Into<String>,
    dependent_counts: &[usize],
) -> BranchingProcessEstimateV0 {
    estimate_branching_process_with_wire_v0(
        workspace_path,
        dependent_counts,
        MultiscaleComplexityWireV0::Canonical,
    )
}

fn estimate_branching_process_with_wire_v0(
    workspace_path: impl Into<String>,
    dependent_counts: &[usize],
    wire: MultiscaleComplexityWireV0,
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
        schema_version: MULTISCALE_COMPLEXITY_HEURISTIC_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.branching-process-estimate",
        layer_marker: wire.layer_marker(),
        feature_gate: wire.feature_gate(),
        workspace_path: workspace_path.into(),
        branching_mean,
        branching_variance,
        extinction_probability,
        expected_propagation_size,
        hot_super_critical_nodes,
        estimator_provenance: BranchingEstimatorProvenanceV0 {
            schema_version: MULTISCALE_COMPLEXITY_HEURISTIC_SCHEMA_VERSION_V0,
            product: "omena-rg-flow.branching-estimator-provenance",
            layer_marker: wire.layer_marker(),
            feature_gate: wire.feature_gate(),
            estimator: "galton-watson-mean-variance-v0",
            sample_count,
        },
    }
}

#[deprecated(
    since = "0.4.0",
    note = "use classify_multiscale_complexity_heuristic_universality; compatibility surface owned by omena-rg-flow maintainers and removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn classify_universality(
    workspace_path: impl Into<String>,
    exponents: ExponentTripleV0,
    confidence_band: ConfidenceBandV0,
    fit_quality: FitQualityV0,
    branching: &BranchingProcessEstimateV0,
) -> UniversalityClassClassificationV0 {
    classify_universality_with_wire_v0(
        workspace_path,
        exponents,
        confidence_band,
        fit_quality,
        branching,
        MultiscaleComplexityWireV0::Compatibility,
    )
}

pub fn classify_multiscale_complexity_heuristic_universality(
    workspace_path: impl Into<String>,
    exponents: ExponentTripleV0,
    confidence_band: ConfidenceBandV0,
    fit_quality: FitQualityV0,
    branching: &BranchingProcessEstimateV0,
) -> UniversalityClassClassificationV0 {
    classify_universality_with_wire_v0(
        workspace_path,
        exponents,
        confidence_band,
        fit_quality,
        branching,
        MultiscaleComplexityWireV0::Canonical,
    )
}

fn classify_universality_with_wire_v0(
    workspace_path: impl Into<String>,
    mut exponents: ExponentTripleV0,
    mut confidence_band: ConfidenceBandV0,
    mut fit_quality: FitQualityV0,
    branching: &BranchingProcessEstimateV0,
    wire: MultiscaleComplexityWireV0,
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

    if wire == MultiscaleComplexityWireV0::Canonical {
        normalize_exponent_triple_wire_v0(&mut exponents, wire);
        normalize_confidence_band_wire_v0(&mut confidence_band, wire);
        normalize_fit_quality_wire_v0(&mut fit_quality, wire);
    }

    UniversalityClassClassificationV0 {
        schema_version: MULTISCALE_COMPLEXITY_HEURISTIC_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.universality-classification",
        layer_marker: wire.layer_marker(),
        feature_gate: wire.feature_gate(),
        workspace_path: workspace_path.into(),
        spatial_class,
        branching_class,
        exponents,
        branching_mean: branching.branching_mean,
        confidence_band,
        fit_quality,
        provenance: vec![ExponentFitProvenanceV0 {
            schema_version: MULTISCALE_COMPLEXITY_HEURISTIC_SCHEMA_VERSION_V0,
            product: "omena-rg-flow.exponent-fit-provenance",
            layer_marker: wire.layer_marker(),
            feature_gate: wire.feature_gate(),
            source: "synthetic-or-benchmark-corpus".to_string(),
            fixture_count: 1,
        }],
    }
}

#[deprecated(
    since = "0.4.0",
    note = "use aggregate_multiscale_complexity_heuristic_module; compatibility surface owned by omena-rg-flow maintainers and removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn aggregate_module(
    module_path: impl Into<String>,
    file_summaries: Vec<FileSummaryV0>,
    boundary_edges: Vec<SummaryEdgeRefV0>,
) -> ModuleAggregateV0 {
    aggregate_module_with_wire_v0(
        module_path,
        file_summaries,
        boundary_edges,
        MultiscaleComplexityWireV0::Compatibility,
    )
}

pub fn aggregate_multiscale_complexity_heuristic_module(
    module_path: impl Into<String>,
    file_summaries: Vec<FileSummaryV0>,
    boundary_edges: Vec<SummaryEdgeRefV0>,
) -> ModuleAggregateV0 {
    aggregate_module_with_wire_v0(
        module_path,
        file_summaries,
        boundary_edges,
        MultiscaleComplexityWireV0::Canonical,
    )
}

fn aggregate_module_with_wire_v0(
    module_path: impl Into<String>,
    mut file_summaries: Vec<FileSummaryV0>,
    mut boundary_edges: Vec<SummaryEdgeRefV0>,
    wire: MultiscaleComplexityWireV0,
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
    if wire == MultiscaleComplexityWireV0::Canonical {
        for summary in &mut file_summaries {
            normalize_file_summary_wire_v0(summary, wire);
        }
        for edge in &mut boundary_edges {
            normalize_summary_edge_wire_v0(edge, wire);
        }
    }

    ModuleAggregateV0 {
        schema_version: MULTISCALE_COMPLEXITY_HEURISTIC_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.module-aggregate",
        layer_marker: wire.layer_marker(),
        feature_gate: wire.feature_gate(),
        module_path: module_path.into(),
        file_summaries,
        boundary_edges,
        aggregate_fast_fact_count,
        aggregate_graph_edge_count,
    }
}

#[deprecated(
    since = "0.4.0",
    note = "use summarize_multiscale_complexity_heuristic_workspace_zset; compatibility surface owned by omena-rg-flow maintainers and removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn summarize_workspace_zset(
    workspace_path: impl Into<String>,
    previous: Option<&WorkspaceZSetV0>,
    module_bundles: Vec<ModuleAggregateV0>,
) -> WorkspaceZSetV0 {
    summarize_workspace_zset_with_wire_v0(
        workspace_path,
        previous,
        module_bundles,
        MultiscaleComplexityWireV0::Compatibility,
    )
}

pub fn summarize_multiscale_complexity_heuristic_workspace_zset(
    workspace_path: impl Into<String>,
    previous: Option<&WorkspaceZSetV0>,
    module_bundles: Vec<ModuleAggregateV0>,
) -> WorkspaceZSetV0 {
    summarize_workspace_zset_with_wire_v0(
        workspace_path,
        previous,
        module_bundles,
        MultiscaleComplexityWireV0::Canonical,
    )
}

fn summarize_workspace_zset_with_wire_v0(
    workspace_path: impl Into<String>,
    previous: Option<&WorkspaceZSetV0>,
    mut module_bundles: Vec<ModuleAggregateV0>,
    wire: MultiscaleComplexityWireV0,
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
    if wire == MultiscaleComplexityWireV0::Canonical {
        for module in &mut module_bundles {
            normalize_module_aggregate_wire_v0(module, wire);
        }
    }
    let workspace_path = workspace_path.into();
    WorkspaceZSetV0 {
        schema_version: MULTISCALE_COMPLEXITY_HEURISTIC_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.workspace-z-set",
        layer_marker: wire.layer_marker(),
        feature_gate: wire.feature_gate(),
        summary_hash: format!("{workspace_path}:{current_weight}:{z_delta_count}"),
        workspace_path,
        module_bundles,
        z_delta_count,
    }
}

#[deprecated(
    since = "0.4.0",
    note = "use summarize_multiscale_complexity_heuristic_ecosystem_contract; compatibility surface owned by omena-rg-flow maintainers and removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn summarize_ecosystem_contract(
    ecosystem_id: impl Into<String>,
    workspaces: Vec<WorkspaceZSetV0>,
    public_api_entries: Vec<PublicApiEntryV0>,
    cross_package_resolution_available: bool,
) -> EcosystemContractV0 {
    summarize_ecosystem_contract_with_wire_v0(
        ecosystem_id,
        workspaces,
        public_api_entries,
        cross_package_resolution_available,
        MultiscaleComplexityWireV0::Compatibility,
    )
}

pub fn summarize_multiscale_complexity_heuristic_ecosystem_contract(
    ecosystem_id: impl Into<String>,
    workspaces: Vec<WorkspaceZSetV0>,
    public_api_entries: Vec<PublicApiEntryV0>,
    cross_package_resolution_available: bool,
) -> EcosystemContractV0 {
    summarize_ecosystem_contract_with_wire_v0(
        ecosystem_id,
        workspaces,
        public_api_entries,
        cross_package_resolution_available,
        MultiscaleComplexityWireV0::Canonical,
    )
}

fn summarize_ecosystem_contract_with_wire_v0(
    ecosystem_id: impl Into<String>,
    mut workspaces: Vec<WorkspaceZSetV0>,
    mut public_api_entries: Vec<PublicApiEntryV0>,
    cross_package_resolution_available: bool,
    wire: MultiscaleComplexityWireV0,
) -> EcosystemContractV0 {
    if wire == MultiscaleComplexityWireV0::Canonical {
        for workspace in &mut workspaces {
            normalize_workspace_wire_v0(workspace, wire);
        }
        for entry in &mut public_api_entries {
            normalize_public_api_entry_wire_v0(entry, wire);
        }
    }
    EcosystemContractV0 {
        schema_version: MULTISCALE_COMPLEXITY_HEURISTIC_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.ecosystem-contract",
        layer_marker: wire.layer_marker(),
        feature_gate: wire.feature_gate(),
        ecosystem_id: ecosystem_id.into(),
        workspaces,
        public_api_entries,
        cross_package_resolution_available,
    }
}

#[deprecated(
    since = "0.4.0",
    note = "use multiscale_complexity_heuristic_static_dynamic_coupling_check; compatibility surface owned by omena-rg-flow maintainers and removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn static_dynamic_coupling_check(
    workspace_path: impl Into<String>,
    g5_fixed_point_coupling: CouplingSpaceV0,
    t1_3_ground_state_coupling: CouplingSpaceV0,
    t1_3_q_ea: f64,
) -> StaticDynamicCouplingV0 {
    static_dynamic_coupling_check_with_wire_v0(
        workspace_path,
        g5_fixed_point_coupling,
        t1_3_ground_state_coupling,
        t1_3_q_ea,
        MultiscaleComplexityWireV0::Compatibility,
    )
}

pub fn multiscale_complexity_heuristic_static_dynamic_coupling_check(
    workspace_path: impl Into<String>,
    g5_fixed_point_coupling: CouplingSpaceV0,
    t1_3_ground_state_coupling: CouplingSpaceV0,
    t1_3_q_ea: f64,
) -> StaticDynamicCouplingV0 {
    static_dynamic_coupling_check_with_wire_v0(
        workspace_path,
        g5_fixed_point_coupling,
        t1_3_ground_state_coupling,
        t1_3_q_ea,
        MultiscaleComplexityWireV0::Canonical,
    )
}

fn static_dynamic_coupling_check_with_wire_v0(
    workspace_path: impl Into<String>,
    mut g5_fixed_point_coupling: CouplingSpaceV0,
    mut t1_3_ground_state_coupling: CouplingSpaceV0,
    t1_3_q_ea: f64,
    wire: MultiscaleComplexityWireV0,
) -> StaticDynamicCouplingV0 {
    let coupling_discrepancy_l2 =
        coupling_l2_distance(&g5_fixed_point_coupling, &t1_3_ground_state_coupling);
    if wire == MultiscaleComplexityWireV0::Canonical {
        normalize_coupling_space_wire_v0(&mut g5_fixed_point_coupling, wire);
        normalize_coupling_space_wire_v0(&mut t1_3_ground_state_coupling, wire);
    }
    StaticDynamicCouplingV0 {
        schema_version: MULTISCALE_COMPLEXITY_HEURISTIC_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.static-dynamic-coupling",
        layer_marker: wire.layer_marker(),
        feature_gate: wire.feature_gate(),
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

#[deprecated(
    since = "0.4.0",
    note = "use multiscale_complexity_heuristic_two_layer_fixed_point; compatibility surface owned by omena-rg-flow maintainers and removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn two_layer_fixed_point(
    workspace_path: impl Into<String>,
    rg_fixed_point: CouplingSpaceV0,
    grn_attractor_id: impl Into<String>,
    embedding: Vec<(String, String)>,
) -> TwoLayerFixedPointV0 {
    two_layer_fixed_point_with_wire_v0(
        workspace_path,
        rg_fixed_point,
        grn_attractor_id,
        embedding,
        MultiscaleComplexityWireV0::Compatibility,
    )
}

pub fn multiscale_complexity_heuristic_two_layer_fixed_point(
    workspace_path: impl Into<String>,
    rg_fixed_point: CouplingSpaceV0,
    grn_attractor_id: impl Into<String>,
    embedding: Vec<(String, String)>,
) -> TwoLayerFixedPointV0 {
    two_layer_fixed_point_with_wire_v0(
        workspace_path,
        rg_fixed_point,
        grn_attractor_id,
        embedding,
        MultiscaleComplexityWireV0::Canonical,
    )
}

fn two_layer_fixed_point_with_wire_v0(
    workspace_path: impl Into<String>,
    mut rg_fixed_point: CouplingSpaceV0,
    grn_attractor_id: impl Into<String>,
    embedding: Vec<(String, String)>,
    wire: MultiscaleComplexityWireV0,
) -> TwoLayerFixedPointV0 {
    if wire == MultiscaleComplexityWireV0::Canonical {
        normalize_coupling_space_wire_v0(&mut rg_fixed_point, wire);
    }
    TwoLayerFixedPointV0 {
        schema_version: MULTISCALE_COMPLEXITY_HEURISTIC_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.two-layer-fixed-point",
        layer_marker: wire.layer_marker(),
        feature_gate: wire.feature_gate(),
        workspace_path: workspace_path.into(),
        rg_fixed_point,
        grn_attractor_id: grn_attractor_id.into(),
        embedding,
    }
}

#[deprecated(
    since = "0.4.0",
    note = "use multiscale_complexity_heuristic_critical_exponent_observable; compatibility surface owned by omena-rg-flow maintainers and removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn critical_exponent_observable(
    workspace_path: impl Into<String>,
    lambda_per_tier: Vec<(u8, f64)>,
    nu_exponent: f64,
    nu_confidence_band_95: (f64, f64),
    exponents: &ExponentTripleV0,
) -> CriticalExponentObservableV0 {
    critical_exponent_observable_with_wire_v0(
        workspace_path,
        lambda_per_tier,
        nu_exponent,
        nu_confidence_band_95,
        exponents,
        MultiscaleComplexityWireV0::Compatibility,
    )
}

pub fn multiscale_complexity_heuristic_critical_exponent_observable(
    workspace_path: impl Into<String>,
    lambda_per_tier: Vec<(u8, f64)>,
    nu_exponent: f64,
    nu_confidence_band_95: (f64, f64),
    exponents: &ExponentTripleV0,
) -> CriticalExponentObservableV0 {
    critical_exponent_observable_with_wire_v0(
        workspace_path,
        lambda_per_tier,
        nu_exponent,
        nu_confidence_band_95,
        exponents,
        MultiscaleComplexityWireV0::Canonical,
    )
}

fn critical_exponent_observable_with_wire_v0(
    workspace_path: impl Into<String>,
    lambda_per_tier: Vec<(u8, f64)>,
    nu_exponent: f64,
    nu_confidence_band_95: (f64, f64),
    exponents: &ExponentTripleV0,
    wire: MultiscaleComplexityWireV0,
) -> CriticalExponentObservableV0 {
    let scaling_relation_residual_l2 =
        (exponents.alpha_depth + 2.0 * exponents.alpha_compress + exponents.alpha_dirty - 2.0)
            .abs();
    CriticalExponentObservableV0 {
        schema_version: MULTISCALE_COMPLEXITY_HEURISTIC_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.critical-exponent-observable",
        layer_marker: wire.layer_marker(),
        feature_gate: wire.feature_gate(),
        workspace_path: workspace_path.into(),
        lambda_per_tier,
        nu_exponent,
        nu_confidence_band_95,
        scaling_relation_residual_l2,
    }
}

#[deprecated(
    since = "0.4.0",
    note = "use multiscale_complexity_heuristic_replica_overlap_coupling_from_m4_alpha; compatibility surface owned by omena-rg-flow maintainers and removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn replica_overlap_coupling_from_m4_alpha(
    overlap: &CascadeReplicaOverlapV0,
) -> CouplingSpaceV0 {
    coupling_space_with_wire_v0(
        overlap.overlap_bucket_count,
        overlap.overlap_bucket_count,
        usize::from(overlap.parisi_breakpoint_m.is_some()),
        overlap.overlap_bucket_count.saturating_sub(1),
        MultiscaleComplexityWireV0::Compatibility,
    )
}

pub fn multiscale_complexity_heuristic_replica_overlap_coupling_from_m4_alpha(
    overlap: &CascadeReplicaOverlapV0,
) -> CouplingSpaceV0 {
    coupling_space_with_wire_v0(
        overlap.overlap_bucket_count,
        overlap.overlap_bucket_count,
        usize::from(overlap.parisi_breakpoint_m.is_some()),
        overlap.overlap_bucket_count.saturating_sub(1),
        MultiscaleComplexityWireV0::Canonical,
    )
}

pub fn multiscale_complexity_heuristic_migration_gate_summary()
-> MultiscaleComplexityHeuristicMigrationGateSummaryV0 {
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
    .map(
        |(gate_id, requirement)| MultiscaleComplexityHeuristicMigrationGateV0 {
            schema_version: MULTISCALE_COMPLEXITY_HEURISTIC_SCHEMA_VERSION_V0,
            product: "omena-rg-flow.migration-gate",
            layer_marker: MULTISCALE_COMPLEXITY_HEURISTIC_LAYER_MARKER_V0,
            feature_gate: MULTISCALE_COMPLEXITY_HEURISTIC_FEATURE_GATE_V0,
            gate_id,
            requirement,
            passed: true,
        },
    )
    .collect::<Vec<_>>();

    MultiscaleComplexityHeuristicMigrationGateSummaryV0 {
        schema_version: MULTISCALE_COMPLEXITY_HEURISTIC_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.migration-gate-summary",
        layer_marker: MULTISCALE_COMPLEXITY_HEURISTIC_LAYER_MARKER_V0,
        feature_gate: MULTISCALE_COMPLEXITY_HEURISTIC_FEATURE_GATE_V0,
        all_passed: gates.iter().all(|gate| gate.passed),
        gates,
    }
}

#[deprecated(
    since = "0.4.0",
    note = "use multiscale_complexity_heuristic_coupling_space; compatibility surface owned by omena-rg-flow maintainers and removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn coupling_space(
    k_env: usize,
    k_decl: usize,
    k_cycle: usize,
    k_dirty: usize,
) -> CouplingSpaceV0 {
    coupling_space_with_wire_v0(
        k_env,
        k_decl,
        k_cycle,
        k_dirty,
        MultiscaleComplexityWireV0::Compatibility,
    )
}

pub fn multiscale_complexity_heuristic_coupling_space(
    k_env: usize,
    k_decl: usize,
    k_cycle: usize,
    k_dirty: usize,
) -> CouplingSpaceV0 {
    coupling_space_with_wire_v0(
        k_env,
        k_decl,
        k_cycle,
        k_dirty,
        MultiscaleComplexityWireV0::Canonical,
    )
}

fn coupling_space_with_wire_v0(
    k_env: usize,
    k_decl: usize,
    k_cycle: usize,
    k_dirty: usize,
    wire: MultiscaleComplexityWireV0,
) -> CouplingSpaceV0 {
    CouplingSpaceV0 {
        schema_version: MULTISCALE_COMPLEXITY_HEURISTIC_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.coupling-space",
        layer_marker: wire.layer_marker(),
        feature_gate: wire.feature_gate(),
        k_env,
        k_decl,
        k_cycle,
        k_dirty,
    }
}

#[deprecated(
    since = "0.4.0",
    note = "use multiscale_complexity_heuristic_file_summary; compatibility surface owned by omena-rg-flow maintainers and removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn file_summary(
    file_path: impl Into<String>,
    fast_fact_count: usize,
    analyzed_edge_count: usize,
) -> FileSummaryV0 {
    file_summary_with_wire_v0(
        file_path,
        fast_fact_count,
        analyzed_edge_count,
        MultiscaleComplexityWireV0::Compatibility,
    )
}

pub fn multiscale_complexity_heuristic_file_summary(
    file_path: impl Into<String>,
    fast_fact_count: usize,
    analyzed_edge_count: usize,
) -> FileSummaryV0 {
    file_summary_with_wire_v0(
        file_path,
        fast_fact_count,
        analyzed_edge_count,
        MultiscaleComplexityWireV0::Canonical,
    )
}

fn file_summary_with_wire_v0(
    file_path: impl Into<String>,
    fast_fact_count: usize,
    analyzed_edge_count: usize,
    wire: MultiscaleComplexityWireV0,
) -> FileSummaryV0 {
    FileSummaryV0 {
        schema_version: MULTISCALE_COMPLEXITY_HEURISTIC_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.file-summary",
        layer_marker: wire.layer_marker(),
        feature_gate: wire.feature_gate(),
        file_path: file_path.into(),
        fast_fact_count,
        analyzed_edge_count,
    }
}

#[deprecated(
    since = "0.4.0",
    note = "use multiscale_complexity_heuristic_summary_edge_ref; compatibility surface owned by omena-rg-flow maintainers and removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn summary_edge_ref(
    summary_edge_id: impl Into<String>,
    edge_kind: impl Into<String>,
) -> SummaryEdgeRefV0 {
    summary_edge_ref_with_wire_v0(
        summary_edge_id,
        edge_kind,
        MultiscaleComplexityWireV0::Compatibility,
    )
}

pub fn multiscale_complexity_heuristic_summary_edge_ref(
    summary_edge_id: impl Into<String>,
    edge_kind: impl Into<String>,
) -> SummaryEdgeRefV0 {
    summary_edge_ref_with_wire_v0(
        summary_edge_id,
        edge_kind,
        MultiscaleComplexityWireV0::Canonical,
    )
}

fn summary_edge_ref_with_wire_v0(
    summary_edge_id: impl Into<String>,
    edge_kind: impl Into<String>,
    wire: MultiscaleComplexityWireV0,
) -> SummaryEdgeRefV0 {
    SummaryEdgeRefV0 {
        schema_version: MULTISCALE_COMPLEXITY_HEURISTIC_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.summary-edge-ref",
        layer_marker: wire.layer_marker(),
        feature_gate: wire.feature_gate(),
        summary_edge_id: summary_edge_id.into(),
        edge_kind: edge_kind.into(),
    }
}

#[deprecated(
    since = "0.4.0",
    note = "use multiscale_complexity_heuristic_exponent_triple; compatibility surface owned by omena-rg-flow maintainers and removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn exponent_triple(
    alpha_depth: f64,
    alpha_compress: f64,
    alpha_dirty: f64,
) -> ExponentTripleV0 {
    exponent_triple_with_wire_v0(
        alpha_depth,
        alpha_compress,
        alpha_dirty,
        MultiscaleComplexityWireV0::Compatibility,
    )
}

pub fn multiscale_complexity_heuristic_exponent_triple(
    alpha_depth: f64,
    alpha_compress: f64,
    alpha_dirty: f64,
) -> ExponentTripleV0 {
    exponent_triple_with_wire_v0(
        alpha_depth,
        alpha_compress,
        alpha_dirty,
        MultiscaleComplexityWireV0::Canonical,
    )
}

fn exponent_triple_with_wire_v0(
    alpha_depth: f64,
    alpha_compress: f64,
    alpha_dirty: f64,
    wire: MultiscaleComplexityWireV0,
) -> ExponentTripleV0 {
    ExponentTripleV0 {
        schema_version: MULTISCALE_COMPLEXITY_HEURISTIC_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.exponent-triple",
        layer_marker: wire.layer_marker(),
        feature_gate: wire.feature_gate(),
        alpha_depth,
        alpha_compress,
        alpha_dirty,
    }
}

#[deprecated(
    since = "0.4.0",
    note = "use multiscale_complexity_heuristic_confidence_band; compatibility surface owned by omena-rg-flow maintainers and removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn confidence_band(
    alpha_depth_ci_95: (f64, f64),
    alpha_compress_ci_95: (f64, f64),
    alpha_dirty_ci_95: (f64, f64),
    branching_mean_ci_95: (f64, f64),
    bootstrap_samples: usize,
) -> ConfidenceBandV0 {
    confidence_band_with_wire_v0(
        alpha_depth_ci_95,
        alpha_compress_ci_95,
        alpha_dirty_ci_95,
        branching_mean_ci_95,
        bootstrap_samples,
        MultiscaleComplexityWireV0::Compatibility,
    )
}

pub fn multiscale_complexity_heuristic_confidence_band(
    alpha_depth_ci_95: (f64, f64),
    alpha_compress_ci_95: (f64, f64),
    alpha_dirty_ci_95: (f64, f64),
    branching_mean_ci_95: (f64, f64),
    bootstrap_samples: usize,
) -> ConfidenceBandV0 {
    confidence_band_with_wire_v0(
        alpha_depth_ci_95,
        alpha_compress_ci_95,
        alpha_dirty_ci_95,
        branching_mean_ci_95,
        bootstrap_samples,
        MultiscaleComplexityWireV0::Canonical,
    )
}

fn confidence_band_with_wire_v0(
    alpha_depth_ci_95: (f64, f64),
    alpha_compress_ci_95: (f64, f64),
    alpha_dirty_ci_95: (f64, f64),
    branching_mean_ci_95: (f64, f64),
    bootstrap_samples: usize,
    wire: MultiscaleComplexityWireV0,
) -> ConfidenceBandV0 {
    ConfidenceBandV0 {
        schema_version: MULTISCALE_COMPLEXITY_HEURISTIC_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.confidence-band",
        layer_marker: wire.layer_marker(),
        feature_gate: wire.feature_gate(),
        alpha_depth_ci_95,
        alpha_compress_ci_95,
        alpha_dirty_ci_95,
        branching_mean_ci_95,
        bootstrap_samples,
    }
}

#[deprecated(
    since = "0.4.0",
    note = "use multiscale_complexity_heuristic_fit_quality; compatibility surface owned by omena-rg-flow maintainers and removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn fit_quality(
    r_squared: f64,
    bootstrap_ci_overlaps_multiple_classes: bool,
    scaling_relation_residual_l2: f64,
) -> FitQualityV0 {
    fit_quality_with_wire_v0(
        r_squared,
        bootstrap_ci_overlaps_multiple_classes,
        scaling_relation_residual_l2,
        MultiscaleComplexityWireV0::Compatibility,
    )
}

pub fn multiscale_complexity_heuristic_fit_quality(
    r_squared: f64,
    bootstrap_ci_overlaps_multiple_classes: bool,
    scaling_relation_residual_l2: f64,
) -> FitQualityV0 {
    fit_quality_with_wire_v0(
        r_squared,
        bootstrap_ci_overlaps_multiple_classes,
        scaling_relation_residual_l2,
        MultiscaleComplexityWireV0::Canonical,
    )
}

fn fit_quality_with_wire_v0(
    r_squared: f64,
    bootstrap_ci_overlaps_multiple_classes: bool,
    scaling_relation_residual_l2: f64,
    wire: MultiscaleComplexityWireV0,
) -> FitQualityV0 {
    FitQualityV0 {
        schema_version: MULTISCALE_COMPLEXITY_HEURISTIC_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.fit-quality",
        layer_marker: wire.layer_marker(),
        feature_gate: wire.feature_gate(),
        r_squared,
        bootstrap_ci_overlaps_multiple_classes,
        scaling_relation_residual_l2,
    }
}

fn normalize_coupling_space_wire_v0(value: &mut CouplingSpaceV0, wire: MultiscaleComplexityWireV0) {
    value.layer_marker = wire.layer_marker();
    value.feature_gate = wire.feature_gate();
}

fn normalize_file_summary_wire_v0(value: &mut FileSummaryV0, wire: MultiscaleComplexityWireV0) {
    value.layer_marker = wire.layer_marker();
    value.feature_gate = wire.feature_gate();
}

fn normalize_summary_edge_wire_v0(value: &mut SummaryEdgeRefV0, wire: MultiscaleComplexityWireV0) {
    value.layer_marker = wire.layer_marker();
    value.feature_gate = wire.feature_gate();
}

fn normalize_exponent_triple_wire_v0(
    value: &mut ExponentTripleV0,
    wire: MultiscaleComplexityWireV0,
) {
    value.layer_marker = wire.layer_marker();
    value.feature_gate = wire.feature_gate();
}

fn normalize_confidence_band_wire_v0(
    value: &mut ConfidenceBandV0,
    wire: MultiscaleComplexityWireV0,
) {
    value.layer_marker = wire.layer_marker();
    value.feature_gate = wire.feature_gate();
}

fn normalize_fit_quality_wire_v0(value: &mut FitQualityV0, wire: MultiscaleComplexityWireV0) {
    value.layer_marker = wire.layer_marker();
    value.feature_gate = wire.feature_gate();
}

fn normalize_module_aggregate_wire_v0(
    value: &mut ModuleAggregateV0,
    wire: MultiscaleComplexityWireV0,
) {
    value.layer_marker = wire.layer_marker();
    value.feature_gate = wire.feature_gate();
    for summary in &mut value.file_summaries {
        normalize_file_summary_wire_v0(summary, wire);
    }
    for edge in &mut value.boundary_edges {
        normalize_summary_edge_wire_v0(edge, wire);
    }
}

fn normalize_workspace_wire_v0(value: &mut WorkspaceZSetV0, wire: MultiscaleComplexityWireV0) {
    value.layer_marker = wire.layer_marker();
    value.feature_gate = wire.feature_gate();
    for module in &mut value.module_bundles {
        normalize_module_aggregate_wire_v0(module, wire);
    }
}

fn normalize_public_api_entry_wire_v0(
    value: &mut PublicApiEntryV0,
    wire: MultiscaleComplexityWireV0,
) {
    value.layer_marker = wire.layer_marker();
    value.feature_gate = wire.feature_gate();
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
    wire: MultiscaleComplexityWireV0,
) -> CouplingSpaceV0 {
    coupling_space_with_wire_v0(
        k_env,
        iteration.changed_count,
        iteration.guaranteed_invalid_count,
        iteration
            .changed_count
            .saturating_sub(iteration.settled_count)
            .saturating_add(iteration.guaranteed_invalid_count),
        wire,
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ObservedFixedPointFlowV0 {
    step_count: usize,
    length_l1: f64,
    fixed_point_residual_l1: usize,
    fixed_point_verified_from_trace: bool,
}

fn observed_fixed_point_flow(
    summary: &CustomPropertyLeastFixedPointSummaryV0,
) -> ObservedFixedPointFlowV0 {
    let length_l1 = summary
        .iteration_trace
        .iter()
        .map(|iteration| iteration.changed_count + iteration.guaranteed_invalid_count)
        .map(|count| count as f64)
        .sum::<f64>();
    let fixed_point_residual_l1 = summary
        .input_count
        .saturating_sub(summary.resolved_count)
        .saturating_sub(summary.guaranteed_invalid_count);
    let fixed_point_verified_from_trace = summary.reached_fixed_point
        && summary.monotone_witness_valid
        && summary.iteration_count == summary.iteration_trace.len()
        && summary
            .iteration_trace
            .last()
            .is_some_and(|iteration| iteration.iteration == summary.iteration_count)
        && fixed_point_residual_l1 == 0;

    ObservedFixedPointFlowV0 {
        step_count: summary.iteration_trace.len(),
        length_l1,
        fixed_point_residual_l1,
        fixed_point_verified_from_trace,
    }
}

fn beta_vector_from_couplings(
    before: &CouplingSpaceV0,
    after: &CouplingSpaceV0,
    wire: MultiscaleComplexityWireV0,
) -> BetaVectorV0 {
    let beta_env = signed_delta(after.k_env, before.k_env);
    let beta_decl = signed_delta(after.k_decl, before.k_decl);
    let beta_cycle = signed_delta(after.k_cycle, before.k_cycle);
    let beta_dirty = signed_delta(after.k_dirty, before.k_dirty);
    let coupling_jacobian = estimate_coupling_jacobian_spectrum_with_wire_v0(before, after, wire);
    let eigenvalues = coupling_jacobian.eigenvalues.clone();
    BetaVectorV0 {
        schema_version: MULTISCALE_COMPLEXITY_HEURISTIC_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.beta-vector",
        layer_marker: wire.layer_marker(),
        feature_gate: wire.feature_gate(),
        beta_env,
        beta_decl,
        beta_cycle,
        beta_dirty,
        coupling_jacobian,
        relevant_operator_count: eigenvalues
            .iter()
            .filter(|value| **value > MULTISCALE_COMPLEXITY_HEURISTIC_EIGEN_EPSILON)
            .count(),
        irrelevant_operator_count: eigenvalues
            .iter()
            .filter(|value| **value < -MULTISCALE_COMPLEXITY_HEURISTIC_EIGEN_EPSILON)
            .count(),
        marginal_operator_count: eigenvalues
            .iter()
            .filter(|value| value.abs() <= MULTISCALE_COMPLEXITY_HEURISTIC_EIGEN_EPSILON)
            .count(),
        eigenvalues,
    }
}

#[deprecated(
    since = "0.4.0",
    note = "use estimate_multiscale_complexity_heuristic_coupling_jacobian_spectrum_v0; compatibility surface owned by omena-rg-flow maintainers and removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn estimate_coupling_jacobian_spectrum_v0(
    before: &CouplingSpaceV0,
    after: &CouplingSpaceV0,
) -> CouplingJacobianSpectrumV0 {
    estimate_coupling_jacobian_spectrum_with_wire_v0(
        before,
        after,
        MultiscaleComplexityWireV0::Compatibility,
    )
}

pub fn estimate_multiscale_complexity_heuristic_coupling_jacobian_spectrum_v0(
    before: &CouplingSpaceV0,
    after: &CouplingSpaceV0,
) -> CouplingJacobianSpectrumV0 {
    estimate_coupling_jacobian_spectrum_with_wire_v0(
        before,
        after,
        MultiscaleComplexityWireV0::Canonical,
    )
}

fn estimate_coupling_jacobian_spectrum_with_wire_v0(
    before: &CouplingSpaceV0,
    after: &CouplingSpaceV0,
    wire: MultiscaleComplexityWireV0,
) -> CouplingJacobianSpectrumV0 {
    let beta_env = signed_delta(after.k_env, before.k_env);
    let beta_decl = signed_delta(after.k_decl, before.k_decl);
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
        schema_version: MULTISCALE_COMPLEXITY_HEURISTIC_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.coupling-jacobian-spectrum",
        layer_marker: wire.layer_marker(),
        feature_gate: wire.feature_gate(),
        mechanism_scope: MULTISCALE_COMPLEXITY_HEURISTIC_MECHANISM_SCOPE_V0,
        product_surface: MULTISCALE_COMPLEXITY_HEURISTIC_PRODUCT_SURFACE_V0,
        default_product_decision_mechanism:
            MULTISCALE_COMPLEXITY_HEURISTIC_DEFAULT_PRODUCT_DECISION_MECHANISM_V0,
        matrix,
        eigenvalues,
        spectral_radius,
        computed_from: "finite-difference-linearization-v0",
    }
}

fn beta_sign_witness(
    beta_vector: &BetaVectorV0,
    monotone_kleene_certificate: bool,
    wire: MultiscaleComplexityWireV0,
) -> BetaSignWitnessV0 {
    BetaSignWitnessV0 {
        schema_version: MULTISCALE_COMPLEXITY_HEURISTIC_SCHEMA_VERSION_V0,
        product: "omena-rg-flow.beta-sign-witness",
        layer_marker: wire.layer_marker(),
        feature_gate: wire.feature_gate(),
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
    if source_delta <= MULTISCALE_COMPLEXITY_HEURISTIC_EIGEN_EPSILON {
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

/// Pre-1.0 nominal compatibility type.
/// Owner: `omena-rg-flow` maintainers. Removal condition: not before 1.0,
/// after downstream migration and zero audited non-compatibility uses.
#[deprecated(
    since = "0.4.0",
    note = "use MultiscaleComplexityHeuristicMetricV0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
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
    pub observed_flow_step_count: usize,
    pub observed_flow_length_l1: f64,
    pub fixed_point_residual_l1: usize,
    pub fixed_point_verified_from_trace: bool,
    pub provenance_handle: String,
}

/// Pre-1.0 nominal compatibility type.
/// Owner: `omena-rg-flow` maintainers. Removal condition: not before 1.0,
/// after downstream migration and zero audited in-repo non-compatibility uses.
#[deprecated(
    since = "0.4.0",
    note = "use MultiscaleComplexityHeuristicMigrationGateV0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
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

/// Pre-1.0 nominal compatibility type.
/// Owner: `omena-rg-flow` maintainers. Removal condition: not before 1.0,
/// after downstream migration and zero audited in-repo non-compatibility uses.
#[deprecated(
    since = "0.4.0",
    note = "use MultiscaleComplexityHeuristicMigrationGateSummaryV0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
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

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "nominal compatibility conversion owned by omena-rg-flow maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
fn into_rg_flow_metric_v0(metric: MultiscaleComplexityHeuristicMetricV0) -> RGFlowMetricV0 {
    RGFlowMetricV0 {
        schema_version: metric.schema_version,
        product: metric.product,
        layer_marker: metric.layer_marker,
        feature_gate: metric.feature_gate,
        workspace_path: metric.workspace_path,
        tier: metric.tier,
        coupling_space: metric.coupling_space,
        beta_vector: metric.beta_vector,
        iteration_count: metric.iteration_count,
        fixed_point_reached: metric.fixed_point_reached,
        flow_length_bound: metric.flow_length_bound,
        observed_flow_step_count: metric.observed_flow_step_count,
        observed_flow_length_l1: metric.observed_flow_length_l1,
        fixed_point_residual_l1: metric.fixed_point_residual_l1,
        fixed_point_verified_from_trace: metric.fixed_point_verified_from_trace,
        provenance_handle: metric.provenance_handle,
    }
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "nominal compatibility conversion owned by omena-rg-flow maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
fn into_rg_flow_migration_gate_v0(
    gate: MultiscaleComplexityHeuristicMigrationGateV0,
) -> RGFlowMigrationGateV0 {
    RGFlowMigrationGateV0 {
        schema_version: gate.schema_version,
        product: gate.product,
        layer_marker: gate.layer_marker,
        feature_gate: gate.feature_gate,
        gate_id: gate.gate_id,
        requirement: gate.requirement,
        passed: gate.passed,
    }
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "nominal compatibility conversion owned by omena-rg-flow maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
fn into_rg_flow_migration_gate_summary_v0(
    summary: MultiscaleComplexityHeuristicMigrationGateSummaryV0,
) -> RGFlowMigrationGateSummaryV0 {
    RGFlowMigrationGateSummaryV0 {
        schema_version: summary.schema_version,
        product: summary.product,
        layer_marker: summary.layer_marker,
        feature_gate: summary.feature_gate,
        gates: summary
            .gates
            .into_iter()
            .map(into_rg_flow_migration_gate_v0)
            .collect(),
        all_passed: summary.all_passed,
    }
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "use summarize_multiscale_complexity_heuristic_metric; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn summarize_rg_flow_metric(
    workspace_path: impl Into<String>,
    tier: u8,
    summary: &CustomPropertyLeastFixedPointSummaryV0,
) -> RGFlowMetricV0 {
    let mut metric =
        summarize_multiscale_complexity_heuristic_metric(workspace_path, tier, summary);
    metric.layer_marker = RG_FLOW_LAYER_MARKER_V0;
    metric.feature_gate = RG_FLOW_FEATURE_GATE_V0;
    metric.coupling_space.layer_marker = RG_FLOW_LAYER_MARKER_V0;
    metric.coupling_space.feature_gate = RG_FLOW_FEATURE_GATE_V0;
    metric.beta_vector.layer_marker = RG_FLOW_LAYER_MARKER_V0;
    metric.beta_vector.feature_gate = RG_FLOW_FEATURE_GATE_V0;
    metric.beta_vector.coupling_jacobian.layer_marker = RG_FLOW_LAYER_MARKER_V0;
    metric.beta_vector.coupling_jacobian.feature_gate = RG_FLOW_FEATURE_GATE_V0;
    into_rg_flow_metric_v0(metric)
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "use multiscale_complexity_heuristic_migration_gate_summary; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn rg_flow_migration_gate_summary() -> RGFlowMigrationGateSummaryV0 {
    let mut summary = multiscale_complexity_heuristic_migration_gate_summary();
    summary.layer_marker = RG_FLOW_LAYER_MARKER_V0;
    summary.feature_gate = RG_FLOW_FEATURE_GATE_V0;
    for gate in &mut summary.gates {
        gate.layer_marker = RG_FLOW_LAYER_MARKER_V0;
        gate.feature_gate = RG_FLOW_FEATURE_GATE_V0;
    }
    into_rg_flow_migration_gate_summary_v0(summary)
}

/// Deprecated V0 wire constants retained for byte compatibility.
/// Owner: `omena-rg-flow` maintainers. Removal condition: not before 1.0,
/// after downstream migration and zero audited non-compatibility uses.
#[deprecated(
    since = "0.4.0",
    note = "use MULTISCALE_COMPLEXITY_HEURISTIC_LAYER_MARKER_V0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub const RG_FLOW_LAYER_MARKER_V0: &str = "rg-flow-statistical";
#[deprecated(
    since = "0.4.0",
    note = "use MULTISCALE_COMPLEXITY_HEURISTIC_FEATURE_GATE_V0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub const RG_FLOW_FEATURE_GATE_V0: &str = "rg-flow";
#[deprecated(
    since = "0.4.0",
    note = "use MULTISCALE_COMPLEXITY_HEURISTIC_SCHEMA_VERSION_V0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub const RG_FLOW_SCHEMA_VERSION_V0: &str = "0";
#[deprecated(
    since = "0.4.0",
    note = "use MULTISCALE_COMPLEXITY_HEURISTIC_MECHANISM_SCOPE_V0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub const RG_FLOW_MECHANISM_SCOPE_V0: &str = "optInDeepAnalysisJacobianSpectrumHintSubstrate";
#[deprecated(
    since = "0.4.0",
    note = "use MULTISCALE_COMPLEXITY_HEURISTIC_PRODUCT_SURFACE_V0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub const RG_FLOW_PRODUCT_SURFACE_V0: &str = "deepAnalysisCascadeSensitivityHint";
#[deprecated(
    since = "0.4.0",
    note = "use MULTISCALE_COMPLEXITY_HEURISTIC_DEFAULT_PRODUCT_DECISION_MECHANISM_V0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub const RG_FLOW_DEFAULT_PRODUCT_DECISION_MECHANISM_V0: bool = false;

#[deprecated(
    since = "0.4.0",
    note = "neutral legacy-wire fixture owned by omena-rg-flow maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[cfg(test)]
const LEGACY_NEUTRAL_BETA_EXPECTED_WIRE_V0: &str = r#"{"layerMarker":"rg-flow-statistical","featureGate":"rg-flow","beforeLayerMarker":"rg-flow-statistical","afterFeatureGate":"rg-flow","betaLayerMarker":"rg-flow-statistical","jacobianFeatureGate":"rg-flow","signLayerMarker":"rg-flow-statistical"}"#;

#[deprecated(
    since = "0.4.0",
    note = "legacy neutral wire fragment owned by omena-rg-flow maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[cfg(test)]
const LEGACY_LAYER_MARKER_FIELD_FRAGMENT_V0: &str = r#""layerMarker":"rg-flow-statistical""#;

#[deprecated(
    since = "0.4.0",
    note = "legacy neutral wire fragment owned by omena-rg-flow maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[cfg(test)]
const LEGACY_FEATURE_GATE_FIELD_FRAGMENT_V0: &str = r#""featureGate":"rg-flow""#;

#[deprecated(
    since = "0.4.0",
    note = "legacy RG wrapper fixture owned by omena-rg-flow maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[cfg(test)]
const LEGACY_RG_METRIC_EXPECTED_WIRE_V0: &str = r#"{"layerMarker":"rg-flow-statistical","featureGate":"rg-flow","couplingLayerMarker":"rg-flow-statistical","betaFeatureGate":"rg-flow","jacobianLayerMarker":"rg-flow-statistical"}"#;

#[cfg(test)]
#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "legacy metric wire fixture adapter owned by omena-rg-flow maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
fn compatibility_multiscale_metric_serialized_projection_v0(
    summary: &CustomPropertyLeastFixedPointSummaryV0,
) -> Result<String, serde_json::Error> {
    let metric = summarize_rg_flow_metric("workspace", 0, summary);
    serde_json::to_string(&serde_json::json!({
        "layerMarker": metric.layer_marker,
        "featureGate": metric.feature_gate,
        "couplingLayerMarker": metric.coupling_space.layer_marker,
        "betaFeatureGate": metric.beta_vector.feature_gate,
        "jacobianLayerMarker": metric.beta_vector.coupling_jacobian.layer_marker,
    }))
}

#[cfg(test)]
mod tests {
    use omena_cascade::{
        CascadeValue, CustomPropertyEnv, summarize_custom_property_least_fixed_point,
    };
    use omena_syntax::ident::PropertyNameV0;
    use sha2::{Digest, Sha256};

    use super::*;

    fn sha256_hex(bytes: &[u8]) -> String {
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    #[allow(deprecated)]
    #[deprecated(
        since = "0.4.0",
        note = "legacy neutral surface fixture owned by omena-rg-flow maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
    )]
    fn compatibility_surface_bundle_v0(
        summary: &CustomPropertyLeastFixedPointSummaryV0,
        overlap: &CascadeReplicaOverlapV0,
    ) -> Result<String, serde_json::Error> {
        let beta = estimate_beta_function_from_custom_property_summary("fixture.css", summary);
        let branching = estimate_branching_process("workspace", &[0, 1, 2]);
        let exponents = exponent_triple(0.2, 0.3, 0.4);
        let confidence = confidence_band((0.1, 0.3), (0.2, 0.4), (0.3, 0.5), (0.8, 1.2), 32);
        let quality = fit_quality(0.9, false, 0.1);
        let classification = classify_universality(
            "workspace",
            exponents.clone(),
            confidence.clone(),
            quality.clone(),
            &branching,
        );
        let file = file_summary("a.module.css", 3, 2);
        let edge = summary_edge_ref("edge-1", "composesLocal");
        let module = aggregate_module("module-a", vec![file.clone()], vec![edge.clone()]);
        let workspace = summarize_workspace_zset("workspace", None, vec![module.clone()]);
        let ecosystem =
            summarize_ecosystem_contract("ecosystem", vec![workspace.clone()], Vec::new(), false);
        let coupling = coupling_space(1, 2, 0, 1);
        let static_dynamic =
            static_dynamic_coupling_check("workspace", coupling.clone(), coupling.clone(), 0.75);
        let two_layer =
            two_layer_fixed_point("workspace", coupling.clone(), "grn-attractor-0", Vec::new());
        let critical =
            critical_exponent_observable("workspace", vec![(0, 0.9)], 0.5, (0.4, 0.6), &exponents);
        let replica = replica_overlap_coupling_from_m4_alpha(overlap);
        let jacobian = estimate_coupling_jacobian_spectrum_v0(&coupling, &replica);
        serde_json::to_string(&serde_json::json!({
            "beta": beta,
            "branching": branching,
            "classification": classification,
            "file": file,
            "edge": edge,
            "module": module,
            "workspace": workspace,
            "ecosystem": ecosystem,
            "staticDynamic": static_dynamic,
            "twoLayer": two_layer,
            "critical": critical,
            "replica": replica,
            "jacobian": jacobian,
        }))
    }

    fn canonical_surface_bundle_v0(
        summary: &CustomPropertyLeastFixedPointSummaryV0,
        overlap: &CascadeReplicaOverlapV0,
    ) -> Result<String, serde_json::Error> {
        let beta =
            estimate_multiscale_complexity_heuristic_beta_function_from_custom_property_summary(
                "fixture.css",
                summary,
            );
        let branching =
            estimate_multiscale_complexity_heuristic_branching_process("workspace", &[0, 1, 2]);
        let exponents = multiscale_complexity_heuristic_exponent_triple(0.2, 0.3, 0.4);
        let confidence = multiscale_complexity_heuristic_confidence_band(
            (0.1, 0.3),
            (0.2, 0.4),
            (0.3, 0.5),
            (0.8, 1.2),
            32,
        );
        let quality = multiscale_complexity_heuristic_fit_quality(0.9, false, 0.1);
        let classification = classify_multiscale_complexity_heuristic_universality(
            "workspace",
            exponents.clone(),
            confidence.clone(),
            quality,
            &branching,
        );
        let file = multiscale_complexity_heuristic_file_summary("a.module.css", 3, 2);
        let edge = multiscale_complexity_heuristic_summary_edge_ref("edge-1", "composesLocal");
        let module = aggregate_multiscale_complexity_heuristic_module(
            "module-a",
            vec![file.clone()],
            vec![edge.clone()],
        );
        let workspace = summarize_multiscale_complexity_heuristic_workspace_zset(
            "workspace",
            None,
            vec![module.clone()],
        );
        let ecosystem = summarize_multiscale_complexity_heuristic_ecosystem_contract(
            "ecosystem",
            vec![workspace.clone()],
            Vec::new(),
            false,
        );
        let coupling = multiscale_complexity_heuristic_coupling_space(1, 2, 0, 1);
        let static_dynamic = multiscale_complexity_heuristic_static_dynamic_coupling_check(
            "workspace",
            coupling.clone(),
            coupling.clone(),
            0.75,
        );
        let two_layer = multiscale_complexity_heuristic_two_layer_fixed_point(
            "workspace",
            coupling.clone(),
            "grn-attractor-0",
            Vec::new(),
        );
        let critical = multiscale_complexity_heuristic_critical_exponent_observable(
            "workspace",
            vec![(0, 0.9)],
            0.5,
            (0.4, 0.6),
            &exponents,
        );
        let replica =
            multiscale_complexity_heuristic_replica_overlap_coupling_from_m4_alpha(overlap);
        let jacobian = estimate_multiscale_complexity_heuristic_coupling_jacobian_spectrum_v0(
            &coupling, &replica,
        );
        serde_json::to_string(&serde_json::json!({
            "beta": beta,
            "branching": branching,
            "classification": classification,
            "file": file,
            "edge": edge,
            "module": module,
            "workspace": workspace,
            "ecosystem": ecosystem,
            "staticDynamic": static_dynamic,
            "twoLayer": two_layer,
            "critical": critical,
            "replica": replica,
            "jacobian": jacobian,
        }))
    }

    #[test]
    #[allow(deprecated)]
    fn beta_estimate_reads_cascade_fixed_point_trace_without_mutating_cascade() {
        let mut env = CustomPropertyEnv::default();
        env.insert(
            PropertyNameV0::canonical_custom_key("--a"),
            CascadeValue::Var {
                name: PropertyNameV0::canonical_custom_key("--b"),
                fallback: None,
            },
        );
        env.insert(
            PropertyNameV0::canonical_custom_key("--b"),
            CascadeValue::Literal("ready".to_string()),
        );
        let summary = summarize_custom_property_least_fixed_point(&env);
        let estimate = estimate_beta_function_from_custom_property_summary("fixture.css", &summary);
        let canonical_estimate =
            estimate_multiscale_complexity_heuristic_beta_function_from_custom_property_summary(
                "fixture.css",
                &summary,
            );
        let metric = summarize_multiscale_complexity_heuristic_metric("workspace", 0, &summary);

        assert_eq!(estimate.schema_version, "0");
        assert_eq!(
            estimate.layer_marker,
            MULTISCALE_COMPLEXITY_HEURISTIC_COMPATIBILITY_LAYER_MARKER_V0
        );
        assert_eq!(
            estimate.feature_gate,
            MULTISCALE_COMPLEXITY_HEURISTIC_COMPATIBILITY_FEATURE_GATE_V0
        );
        assert_eq!(
            canonical_estimate.layer_marker,
            "multiscale-complexity-heuristic-statistical"
        );
        assert_eq!(
            canonical_estimate.feature_gate,
            "multiscale-complexity-heuristic"
        );
        assert_eq!(estimate.sign_witness.beta_env_sign, -1);
        assert!(estimate.sign_witness.beta_decl_sign <= 0);
        assert!(estimate.sign_witness.monotone_kleene_certificate);
        assert_eq!(metric.feature_gate, "multiscale-complexity-heuristic");
        assert!(metric.fixed_point_reached);
        assert_eq!(metric.fixed_point_residual_l1, 0);
        assert!(metric.fixed_point_verified_from_trace);
        assert_eq!(
            metric.observed_flow_step_count,
            summary.iteration_trace.len()
        );
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
    #[allow(deprecated)]
    fn neutral_beta_estimate_preserves_exact_legacy_wire_projection()
    -> Result<(), serde_json::Error> {
        let summary = summarize_custom_property_least_fixed_point(&CustomPropertyEnv::default());
        let estimate = estimate_beta_function_from_custom_property_summary("fixture.css", &summary);
        let canonical =
            estimate_multiscale_complexity_heuristic_beta_function_from_custom_property_summary(
                "fixture.css",
                &summary,
            );
        let compatibility_bytes = serde_json::to_vec(&estimate)?;
        let canonical_bytes = serde_json::to_vec(&canonical)?;
        assert_eq!(compatibility_bytes.len(), 1585);
        assert_eq!(
            sha256_hex(&compatibility_bytes),
            "2e9efce8c20df8bb51826507d0f7bce254e5cc854c972e2afc55358b2e09eb24"
        );
        assert_eq!(canonical_bytes.len(), 1873);
        assert_eq!(
            sha256_hex(&canonical_bytes),
            "2c689297c0137a88dce5739c7499bc73ea917f1d35f5109388e3dca746d0459f"
        );
        let actual = serde_json::to_string(&serde_json::json!({
            "layerMarker": estimate.layer_marker,
            "featureGate": estimate.feature_gate,
            "beforeLayerMarker": estimate.coupling_before.layer_marker,
            "afterFeatureGate": estimate.coupling_after.feature_gate,
            "betaLayerMarker": estimate.beta_vector.layer_marker,
            "jacobianFeatureGate": estimate.beta_vector.coupling_jacobian.feature_gate,
            "signLayerMarker": estimate.sign_witness.layer_marker,
        }))?;
        assert_eq!(actual, LEGACY_NEUTRAL_BETA_EXPECTED_WIRE_V0);
        Ok(())
    }

    #[test]
    #[allow(deprecated)]
    fn compatibility_and_canonical_surface_bundles_pin_full_bytes_and_wire_callgraphs()
    -> Result<(), serde_json::Error> {
        let summary = summarize_custom_property_least_fixed_point(&CustomPropertyEnv::default());
        let overlap = CascadeReplicaOverlapV0 {
            schema_version: "0",
            product: "omena-cascade.replica-overlap",
            layer_marker: "statistical-mechanics",
            feature_gate: "spin-glass",
            overlap_bucket_count: 4,
            parisi_breakpoint_m: Some(0.5),
            advisory_only: true,
        };
        let compatibility = compatibility_surface_bundle_v0(&summary, &overlap)?;
        let canonical = canonical_surface_bundle_v0(&summary, &overlap)?;
        assert_eq!(compatibility.len(), 8239);
        assert_eq!(
            sha256_hex(compatibility.as_bytes()),
            "113baaa17a66336a8675ff82991c93a0ea919aa465fb4a899028e4efb679dcdf"
        );
        assert_eq!(canonical.len(), 9919);
        assert_eq!(
            sha256_hex(canonical.as_bytes()),
            "9046559d402601c1c51110ffb8f39772bf0124b2f98710ac6a25f09267f7ee3b"
        );
        assert!(compatibility.contains(LEGACY_LAYER_MARKER_FIELD_FRAGMENT_V0));
        assert!(compatibility.contains(LEGACY_FEATURE_GATE_FIELD_FRAGMENT_V0));
        assert!(!canonical.contains(LEGACY_LAYER_MARKER_FIELD_FRAGMENT_V0));
        assert!(!canonical.contains(LEGACY_FEATURE_GATE_FIELD_FRAGMENT_V0));
        assert!(
            canonical.contains(r#""layerMarker":"multiscale-complexity-heuristic-statistical""#)
        );
        assert!(canonical.contains(r#""featureGate":"multiscale-complexity-heuristic""#));
        Ok(())
    }

    #[test]
    fn canonical_multiscale_metric_uses_exact_accurate_wire_projection()
    -> Result<(), serde_json::Error> {
        let summary = summarize_custom_property_least_fixed_point(&CustomPropertyEnv::default());
        let metric = summarize_multiscale_complexity_heuristic_metric("workspace", 0, &summary);
        let actual = serde_json::to_string(&serde_json::json!({
            "layerMarker": metric.layer_marker,
            "featureGate": metric.feature_gate,
            "couplingLayerMarker": metric.coupling_space.layer_marker,
            "betaFeatureGate": metric.beta_vector.feature_gate,
            "jacobianLayerMarker": metric.beta_vector.coupling_jacobian.layer_marker,
        }))?;
        assert_eq!(
            actual,
            r#"{"layerMarker":"multiscale-complexity-heuristic-statistical","featureGate":"multiscale-complexity-heuristic","couplingLayerMarker":"multiscale-complexity-heuristic-statistical","betaFeatureGate":"multiscale-complexity-heuristic","jacobianLayerMarker":"multiscale-complexity-heuristic-statistical"}"#
        );
        Ok(())
    }

    #[test]
    #[allow(deprecated)]
    fn compatibility_multiscale_wrapper_preserves_exact_wire_projection()
    -> Result<(), serde_json::Error> {
        let summary = summarize_custom_property_least_fixed_point(&CustomPropertyEnv::default());
        let actual = compatibility_multiscale_metric_serialized_projection_v0(&summary)?;
        assert_eq!(actual, LEGACY_RG_METRIC_EXPECTED_WIRE_V0);
        Ok(())
    }

    #[test]
    fn beta_vector_preserves_positive_declaration_growth() {
        let before = multiscale_complexity_heuristic_coupling_space(5, 4, 0, 0);
        let after = multiscale_complexity_heuristic_coupling_space(5, 10, 0, 0);
        let beta =
            beta_vector_from_couplings(&before, &after, MultiscaleComplexityWireV0::Canonical);
        let witness = beta_sign_witness(&beta, true, MultiscaleComplexityWireV0::Canonical);

        assert_eq!(beta.beta_decl, 6.0);
        assert_eq!(witness.beta_decl_sign, 1);
        assert!(beta.coupling_jacobian.spectral_radius > 1.0);
    }

    #[test]
    fn multiscale_complexity_heuristic_metric_records_trace_derived_flow_evidence() {
        let mut env = CustomPropertyEnv::default();
        env.insert(
            PropertyNameV0::canonical_custom_key("--a"),
            CascadeValue::Var {
                name: PropertyNameV0::canonical_custom_key("--b"),
                fallback: None,
            },
        );
        env.insert(
            PropertyNameV0::canonical_custom_key("--b"),
            CascadeValue::Var {
                name: PropertyNameV0::canonical_custom_key("--c"),
                fallback: None,
            },
        );
        env.insert(
            PropertyNameV0::canonical_custom_key("--c"),
            CascadeValue::Literal("ready".to_string()),
        );
        let summary = summarize_custom_property_least_fixed_point(&env);
        let metric = summarize_multiscale_complexity_heuristic_metric("workspace", 0, &summary);

        assert!(metric.observed_flow_step_count > 0);
        assert!(metric.observed_flow_length_l1 > 0.0);
        assert_eq!(metric.fixed_point_residual_l1, 0);
        assert!(metric.fixed_point_verified_from_trace);
        assert!(metric.flow_length_bound >= metric.observed_flow_step_count);
    }

    #[test]
    fn coupling_jacobian_computes_non_alias_eigenvalue_spectrum() {
        let before = multiscale_complexity_heuristic_coupling_space(4, 3, 1, 2);
        let after = multiscale_complexity_heuristic_coupling_space(2, 1, 2, 4);
        let beta =
            beta_vector_from_couplings(&before, &after, MultiscaleComplexityWireV0::Canonical);
        let direct_spectrum =
            estimate_multiscale_complexity_heuristic_coupling_jacobian_spectrum_v0(&before, &after);

        assert_eq!(beta.coupling_jacobian, direct_spectrum);
        assert_eq!(direct_spectrum.matrix.len(), 4);
        assert_eq!(direct_spectrum.eigenvalues.len(), 4);
        assert_eq!(
            direct_spectrum.mechanism_scope,
            MULTISCALE_COMPLEXITY_HEURISTIC_MECHANISM_SCOPE_V0
        );
        assert_eq!(
            direct_spectrum.product_surface,
            MULTISCALE_COMPLEXITY_HEURISTIC_PRODUCT_SURFACE_V0
        );
        assert!(!direct_spectrum.default_product_decision_mechanism);
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
        let branching =
            estimate_multiscale_complexity_heuristic_branching_process("workspace", &[1, 1, 1]);
        let exponents = multiscale_complexity_heuristic_exponent_triple(0.2, 0.3, 0.4);
        let confidence = multiscale_complexity_heuristic_confidence_band(
            (0.1, 0.3),
            (0.2, 0.4),
            (0.3, 0.5),
            (0.9, 1.1),
            1000,
        );
        let low_r2 = classify_multiscale_complexity_heuristic_universality(
            "workspace-low-r2",
            exponents.clone(),
            confidence.clone(),
            multiscale_complexity_heuristic_fit_quality(0.59, false, 0.0),
            &branching,
        );
        let overlapping_ci = classify_multiscale_complexity_heuristic_universality(
            "workspace-overlap",
            exponents.clone(),
            confidence.clone(),
            multiscale_complexity_heuristic_fit_quality(0.9, true, 0.0),
            &branching,
        );
        let bad_scaling = classify_multiscale_complexity_heuristic_universality(
            "workspace-bad-scaling",
            exponents,
            confidence,
            multiscale_complexity_heuristic_fit_quality(0.9, false, 0.5),
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
        let subcritical =
            estimate_multiscale_complexity_heuristic_branching_process("sub", &[0, 1, 0, 1]);
        let supercritical =
            estimate_multiscale_complexity_heuristic_branching_process("super", &[2, 3, 1, 4]);

        assert!(subcritical.expected_propagation_size.is_some());
        assert_eq!(subcritical.extinction_probability, 1.0);
        assert!(supercritical.expected_propagation_size.is_none());
        assert!(supercritical.extinction_probability < 1.0);
        assert!(!supercritical.hot_super_critical_nodes.is_empty());
    }

    #[test]
    fn tier_contracts_and_cross_tier_contracts_carry_schema_zero() {
        let file = multiscale_complexity_heuristic_file_summary("a.module.css", 3, 2);
        let edge = multiscale_complexity_heuristic_summary_edge_ref("edge-1", "composesLocal");
        let module =
            aggregate_multiscale_complexity_heuristic_module("module-a", vec![file], vec![edge]);
        let workspace = summarize_multiscale_complexity_heuristic_workspace_zset(
            "workspace",
            None,
            vec![module],
        );
        let ecosystem = summarize_multiscale_complexity_heuristic_ecosystem_contract(
            "ecosystem",
            vec![workspace],
            Vec::new(),
            false,
        );
        let coupling = multiscale_complexity_heuristic_coupling_space(1, 2, 0, 1);
        let static_dynamic = multiscale_complexity_heuristic_static_dynamic_coupling_check(
            "workspace",
            coupling.clone(),
            coupling.clone(),
            0.75,
        );
        let two_layer = multiscale_complexity_heuristic_two_layer_fixed_point(
            "workspace",
            coupling.clone(),
            "grn-attractor-0",
            vec![("rg".to_string(), "grn".to_string())],
        );
        let critical = multiscale_complexity_heuristic_critical_exponent_observable(
            "workspace",
            vec![(0, 0.9), (1, 1.0)],
            0.5,
            (0.4, 0.6),
            &multiscale_complexity_heuristic_exponent_triple(0.5, 0.5, 0.5),
        );

        assert_eq!(ecosystem.schema_version, "0");
        assert_eq!(static_dynamic.schema_version, "0");
        assert_eq!(static_dynamic.coupling_discrepancy_l2, 0.0);
        assert_eq!(
            two_layer.layer_marker,
            MULTISCALE_COMPLEXITY_HEURISTIC_LAYER_MARKER_V0
        );
        assert_eq!(
            critical.feature_gate,
            MULTISCALE_COMPLEXITY_HEURISTIC_FEATURE_GATE_V0
        );
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
        let coupling =
            multiscale_complexity_heuristic_replica_overlap_coupling_from_m4_alpha(&overlap);

        assert_eq!(coupling.schema_version, "0");
        assert_eq!(coupling.k_env, 4);
        assert_eq!(coupling.k_cycle, 1);
    }

    #[test]
    fn migration_gates_cover_g_rg_0_through_g_rg_5() {
        let summary = multiscale_complexity_heuristic_migration_gate_summary();
        let gate_ids = summary
            .gates
            .iter()
            .map(|gate| gate.gate_id)
            .collect::<Vec<_>>();

        assert_eq!(summary.schema_version, "0");
        assert_eq!(
            summary.layer_marker,
            "multiscale-complexity-heuristic-statistical"
        );
        assert_eq!(summary.feature_gate, "multiscale-complexity-heuristic");
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
