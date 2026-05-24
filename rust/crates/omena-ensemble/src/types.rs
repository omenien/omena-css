use std::collections::BTreeMap;

use omena_cascade::{CascadeOutcome, CascadeReplicaOverlapV0};
use serde::Serialize;

pub const REPLICA_ENSEMBLE_SCHEMA_VERSION_V0: &str = "0";
pub const REPLICA_ENSEMBLE_LAYER_MARKER_V0: &str = "replica-ensemble";
pub const REPLICA_ENSEMBLE_FEATURE_GATE_V0: &str = "replica-ensemble";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeSiteKeyV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub element_selector: String,
    pub property: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinearProvenanceTagV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub semiring_identifier: &'static str,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplicaSiteOutcomeV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub site: CascadeSiteKeyV0,
    pub outcome: CascadeOutcome,
    pub provenance: Option<LinearProvenanceTagV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplicaSnapshotV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub path: String,
    pub sites: Vec<ReplicaSiteOutcomeV0>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OutcomeMode {
    #[default]
    DefiniteOnly,
    WidenedRankedSet,
    FullStrict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SamplingPolicy {
    AllPairs,
    PageRankWeighted { max_pair_count: usize },
    RandomSubset { max_pair_count: usize },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplicaOverlapV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub replica_alpha_path: String,
    pub replica_beta_path: String,
    pub outcome_mode: OutcomeMode,
    pub shared_site_count: usize,
    pub agreeing_site_count: usize,
    pub overlap_q: f64,
    pub overlap_q_unit: &'static str,
    pub provenance_attributions: Vec<OverlapAttributionV0>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlapAttributionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub site_element_selector: String,
    pub site_property: String,
    pub winner_alpha: String,
    pub winner_beta: String,
    pub provenance_alpha: Option<LinearProvenanceTagV0>,
    pub provenance_beta: Option<LinearProvenanceTagV0>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplicaOverlapDistributionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub workspace_root: String,
    pub outcome_mode: OutcomeMode,
    pub replica_count: usize,
    pub pair_count: usize,
    pub histogram_bin_count: usize,
    pub histogram_bins: Vec<HistogramBinV0>,
    pub modality: DistributionModality,
    pub modality_definition: &'static str,
    pub peak_q_values: Vec<f64>,
    pub parisi_m_estimate: Option<f64>,
    pub parisi_m_source: ParisiSource,
    pub mean_q: f64,
    pub variance_q: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ParisiSource {
    M4AlphaCascadeReplicaOverlap,
    LocalEmFallback,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistogramBinV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub q_low: f64,
    pub q_high: f64,
    pub count: usize,
    pub normalized_density: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DistributionModality {
    Trivial,
    Unimodal,
    BimodalRSB,
    Continuous,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleGraphV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub workspace_root: String,
    pub nodes: Vec<String>,
    pub edges: Vec<ModuleGraphEdgeV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleGraphEdgeV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub from_module: String,
    pub to_module: String,
    pub edge_kind: &'static str,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SBMDetectabilityThresholdV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub workspace_root: String,
    pub node_count: usize,
    pub edge_count: usize,
    pub assortative_partition: PartitionEstimateV0,
    pub p_in_estimate: f64,
    pub p_out_estimate: f64,
    pub lambda_snr: f64,
    pub phase: DetectabilityPhase,
    pub spectral_method_used: SpectralMethod,
    pub k_community_estimate: usize,
    pub partition_hypothesis_results: Vec<PartitionHypothesisResultV0>,
    pub best_hypothesis: PartitionHypothesisLabel,
    pub best_lambda_snr: f64,
    pub critical_exponent_annotation: Option<CriticalExponentAnnotationV0>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartitionHypothesisResultV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub label: PartitionHypothesisLabel,
    pub partition: PartitionEstimateV0,
    pub lambda_snr_per_hypothesis: f64,
    pub log_likelihood: f64,
    pub aic_relative: f64,
    pub bic_relative: f64,
    pub likelihood_ratio_p_value: Option<f64>,
    pub k_communities: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PartitionHypothesisLabel {
    DirectoryTree,
    ComposesCluster,
    BrandTheme,
    AutoSpectral,
    UserSupplied(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartitionEstimateV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub partitions: BTreeMap<String, u32>,
    pub community_size_distribution: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DetectabilityPhase {
    Detectable,
    Borderline,
    Undetectable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SpectralMethod {
    Auto,
    DegreeCorrected,
    Spectral,
    NonBacktracking,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CriticalExponentAnnotationV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub rg_exponent_triple_handle: Option<RgExponentHandleV0>,
    pub detectability_exponent_beta_est: f64,
    pub universality_class_hint: Option<UniversalityClassHint>,
    pub agreement_with_rg_fixed_point: AgreementVerdict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UniversalityClassHint {
    UtilityDominated,
    TokenGraph,
    ComponentScoped,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AgreementVerdict {
    Agree,
    Disagree,
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RgExponentHandleV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub workspace_root: String,
    pub timestamp: String,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossFileInconsistencyReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub workspace_root: String,
    pub distribution: ReplicaOverlapDistributionV0,
    pub detectability: SBMDetectabilityThresholdV0,
    pub top_disagreement_pairs: Vec<ReplicaOverlapV0>,
    pub recommendation: ReportRecommendation,
    pub outcome_projection_policy: OutcomeProjectionPolicyV0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ReportRecommendation {
    NoActionNeeded,
    InvestigateRsbBroken,
    UndetectablePhase,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutcomeProjectionPolicyV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub consumer_id: ConsumerId,
    pub projection: ProjectionFamily,
    pub top_variant_treatment: TopVariantTreatment,
    pub ranked_set_treatment: RankedSetTreatment,
    pub inherit_treatment: InheritTreatment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ConsumerId {
    ReplicaOverlap,
    GrnCascade,
    VciVariational,
    MdlCompression,
    Other(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ProjectionFamily {
    BinaryAgreement,
    TernaryClassification,
    GeneralProjection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TopVariantTreatment {
    ExcludeFromOverlap,
    TreatAsDisagree,
    TreatAsUnknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RankedSetTreatment {
    ExcludeFromOverlap,
    WidenedAgreement,
    BimodalClassification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum InheritTreatment {
    ExcludeFromOverlap,
    StrictEquality,
    ZeroInflatedClassification,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportOptionsV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub partition_hypotheses: Vec<PartitionHypothesisLabel>,
    pub spectral_method: SpectralMethod,
    pub sampling_policy: Option<SamplingPolicy>,
    pub rg_exponent_handle: Option<RgExponentHandleV0>,
}

impl Default for ReportOptionsV0 {
    fn default() -> Self {
        Self {
            schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
            product: "omena-ensemble.report-options",
            layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
            feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
            partition_hypotheses: vec![
                PartitionHypothesisLabel::AutoSpectral,
                PartitionHypothesisLabel::ComposesCluster,
                PartitionHypothesisLabel::DirectoryTree,
            ],
            spectral_method: SpectralMethod::Auto,
            sampling_policy: None,
            rg_exponent_handle: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ParisiM4AlphaSource<'a> {
    pub replica_overlap: &'a CascadeReplicaOverlapV0,
}
