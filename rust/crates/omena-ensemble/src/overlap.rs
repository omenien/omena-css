use std::collections::BTreeMap;

use omena_cascade::CascadeOutcome;

use crate::types::{
    CascadeSiteKeyV0, DistributionModality, HistogramBinV0, OutcomeMode, OutcomeProjectionPolicyV0,
    OverlapAttributionV0, ParisiM4AlphaSource, ParisiSource, REPLICA_ENSEMBLE_FEATURE_GATE_V0,
    REPLICA_ENSEMBLE_LAYER_MARKER_V0, REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
    ReplicaOverlapDistributionV0, ReplicaOverlapV0, ReplicaSiteOutcomeV0, ReplicaSnapshotV0,
    SamplingPolicy,
};
use crate::{
    ConsumerId, InheritTreatment, ProjectionFamily, RankedSetTreatment, TopVariantTreatment,
};

pub fn compute_replica_overlap<I, J>(
    alpha: &str,
    beta: &str,
    cascade_alpha: I,
    cascade_beta: J,
    mode: OutcomeMode,
) -> ReplicaOverlapV0
where
    I: IntoIterator<Item = ReplicaSiteOutcomeV0>,
    J: IntoIterator<Item = ReplicaSiteOutcomeV0>,
{
    let alpha_by_site = cascade_alpha
        .into_iter()
        .map(|entry| (entry.site.clone(), entry))
        .collect::<BTreeMap<_, _>>();
    let beta_by_site = cascade_beta
        .into_iter()
        .map(|entry| (entry.site.clone(), entry))
        .collect::<BTreeMap<_, _>>();

    let mut shared_site_count = 0usize;
    let mut agreeing_site_count = 0usize;
    let mut provenance_attributions = Vec::new();

    for (site, alpha_entry) in &alpha_by_site {
        let Some(beta_entry) = beta_by_site.get(site) else {
            continue;
        };

        let Some(alpha_projection) = project_outcome(&alpha_entry.outcome, mode) else {
            continue;
        };
        let Some(beta_projection) = project_outcome(&beta_entry.outcome, mode) else {
            continue;
        };

        shared_site_count += 1;
        if alpha_projection == beta_projection {
            agreeing_site_count += 1;
        } else {
            provenance_attributions.push(OverlapAttributionV0 {
                schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
                product: "omena-ensemble.overlap-attribution",
                layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
                feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
                site_element_selector: site.element_selector.clone(),
                site_property: site.property.clone(),
                winner_alpha: alpha_projection,
                winner_beta: beta_projection,
                provenance_alpha: alpha_entry.provenance.clone(),
                provenance_beta: beta_entry.provenance.clone(),
            });
        }
    }

    let overlap_q = if shared_site_count == 0 {
        0.0
    } else {
        agreeing_site_count as f64 / shared_site_count as f64
    };

    ReplicaOverlapV0 {
        schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
        product: "omena-ensemble.replica-overlap",
        layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
        feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
        replica_alpha_path: alpha.to_string(),
        replica_beta_path: beta.to_string(),
        outcome_mode: mode,
        shared_site_count,
        agreeing_site_count,
        overlap_q,
        overlap_q_unit: "unitless",
        provenance_attributions,
    }
}

pub fn compute_overlap_distribution(
    workspace_root: &str,
    replicas: impl IntoIterator<Item = ReplicaSnapshotV0>,
    sampling_policy: Option<SamplingPolicy>,
    outcome_mode: OutcomeMode,
    parisi_source: Option<ParisiM4AlphaSource<'_>>,
) -> ReplicaOverlapDistributionV0 {
    let replicas = replicas.into_iter().collect::<Vec<_>>();
    let pair_indices = selected_pair_indices(replicas.len(), sampling_policy);
    let overlaps = pair_indices
        .iter()
        .map(|(alpha_index, beta_index)| {
            let alpha = &replicas[*alpha_index];
            let beta = &replicas[*beta_index];
            compute_replica_overlap(
                &alpha.path,
                &beta.path,
                alpha.sites.clone(),
                beta.sites.clone(),
                outcome_mode,
            )
        })
        .collect::<Vec<_>>();

    distribution_from_overlaps(
        workspace_root,
        replicas.len(),
        outcome_mode,
        &overlaps,
        parisi_source,
    )
}

pub fn outcome_projection_policy_for_mode(mode: OutcomeMode) -> OutcomeProjectionPolicyV0 {
    let (top_variant_treatment, ranked_set_treatment, inherit_treatment) = match mode {
        OutcomeMode::DefiniteOnly => (
            TopVariantTreatment::ExcludeFromOverlap,
            RankedSetTreatment::ExcludeFromOverlap,
            InheritTreatment::ExcludeFromOverlap,
        ),
        OutcomeMode::WidenedRankedSet => (
            TopVariantTreatment::ExcludeFromOverlap,
            RankedSetTreatment::WidenedAgreement,
            InheritTreatment::ExcludeFromOverlap,
        ),
        OutcomeMode::FullStrict => (
            TopVariantTreatment::TreatAsDisagree,
            RankedSetTreatment::WidenedAgreement,
            InheritTreatment::StrictEquality,
        ),
    };

    OutcomeProjectionPolicyV0 {
        schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
        product: "omena-ensemble.outcome-projection-policy",
        layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
        feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
        consumer_id: ConsumerId::ReplicaOverlap,
        projection: ProjectionFamily::BinaryAgreement,
        top_variant_treatment,
        ranked_set_treatment,
        inherit_treatment,
    }
}

pub fn grn_outcome_projection_policy() -> OutcomeProjectionPolicyV0 {
    OutcomeProjectionPolicyV0 {
        schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
        product: "omena-ensemble.outcome-projection-policy",
        layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
        feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
        consumer_id: ConsumerId::GrnCascade,
        projection: ProjectionFamily::TernaryClassification,
        top_variant_treatment: TopVariantTreatment::TreatAsUnknown,
        ranked_set_treatment: RankedSetTreatment::BimodalClassification,
        inherit_treatment: InheritTreatment::ZeroInflatedClassification,
    }
}

fn distribution_from_overlaps(
    workspace_root: &str,
    replica_count: usize,
    outcome_mode: OutcomeMode,
    overlaps: &[ReplicaOverlapV0],
    parisi_source: Option<ParisiM4AlphaSource<'_>>,
) -> ReplicaOverlapDistributionV0 {
    let pair_count = overlaps.len();
    let q_values = overlaps
        .iter()
        .map(|overlap| overlap.overlap_q)
        .collect::<Vec<_>>();
    let mean_q = mean(&q_values);
    let variance_q = variance(&q_values, mean_q);
    let histogram_bins = histogram(&q_values, 10);
    let modality = classify_modality(pair_count, variance_q, &histogram_bins);
    let (parisi_m_estimate, parisi_m_source) = parisi_estimate(modality, parisi_source, &q_values);

    ReplicaOverlapDistributionV0 {
        schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
        product: "omena-ensemble.replica-overlap-distribution",
        layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
        feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
        workspace_root: workspace_root.to_string(),
        outcome_mode,
        replica_count,
        pair_count,
        histogram_bin_count: histogram_bins.len(),
        histogram_bins,
        modality,
        modality_definition: modality_definition(modality, parisi_m_source),
        peak_q_values: peak_q_values(&q_values),
        parisi_m_estimate,
        parisi_m_source,
        mean_q,
        variance_q,
    }
}

fn selected_pair_indices(
    replica_count: usize,
    sampling_policy: Option<SamplingPolicy>,
) -> Vec<(usize, usize)> {
    let mut pairs = Vec::new();
    for alpha_index in 0..replica_count {
        for beta_index in alpha_index + 1..replica_count {
            pairs.push((alpha_index, beta_index));
        }
    }

    let max_pair_count = match sampling_policy {
        Some(SamplingPolicy::PageRankWeighted { max_pair_count })
        | Some(SamplingPolicy::RandomSubset { max_pair_count }) => Some(max_pair_count),
        Some(SamplingPolicy::AllPairs) | None => None,
    };

    if let Some(max_pair_count) = max_pair_count {
        pairs.truncate(max_pair_count);
    }
    pairs
}

fn project_outcome(outcome: &CascadeOutcome, mode: OutcomeMode) -> Option<String> {
    match (outcome, mode) {
        (CascadeOutcome::Definite { winner, .. }, _) => Some(format!("definite:{}", winner.id)),
        (CascadeOutcome::RankedSet(declarations), OutcomeMode::WidenedRankedSet)
        | (CascadeOutcome::RankedSet(declarations), OutcomeMode::FullStrict) => {
            let mut ids = declarations
                .iter()
                .map(|declaration| declaration.id.as_str())
                .collect::<Vec<_>>();
            ids.sort_unstable();
            Some(format!("ranked:{}", ids.join("|")))
        }
        (CascadeOutcome::Inherit, OutcomeMode::FullStrict) => Some("inherit".to_string()),
        (CascadeOutcome::Top, OutcomeMode::FullStrict) => Some("top".to_string()),
        (CascadeOutcome::RankedSet(_), OutcomeMode::DefiniteOnly)
        | (CascadeOutcome::Inherit, OutcomeMode::DefiniteOnly | OutcomeMode::WidenedRankedSet)
        | (CascadeOutcome::Top, OutcomeMode::DefiniteOnly | OutcomeMode::WidenedRankedSet) => None,
    }
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

fn variance(values: &[f64], mean: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values
        .iter()
        .map(|value| {
            let delta = value - mean;
            delta * delta
        })
        .sum::<f64>()
        / values.len() as f64
}

fn histogram(values: &[f64], bin_count: usize) -> Vec<HistogramBinV0> {
    let mut counts = vec![0usize; bin_count];
    for value in values {
        let clamped = value.clamp(0.0, 1.0);
        let mut bin_index = (clamped * bin_count as f64).floor() as usize;
        if bin_index == bin_count {
            bin_index = bin_count.saturating_sub(1);
        }
        counts[bin_index] += 1;
    }

    counts
        .into_iter()
        .enumerate()
        .map(|(index, count)| HistogramBinV0 {
            schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
            product: "omena-ensemble.histogram-bin",
            layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
            feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
            q_low: index as f64 / bin_count as f64,
            q_high: (index + 1) as f64 / bin_count as f64,
            count,
            normalized_density: if values.is_empty() {
                0.0
            } else {
                count as f64 / values.len() as f64
            },
        })
        .collect()
}

fn classify_modality(
    pair_count: usize,
    variance_q: f64,
    histogram_bins: &[HistogramBinV0],
) -> DistributionModality {
    if pair_count < 3 {
        return DistributionModality::Trivial;
    }
    if variance_q < 0.01 {
        return DistributionModality::Unimodal;
    }

    let low_peak = histogram_bins
        .iter()
        .any(|bin| bin.count > 0 && bin.q_high <= 0.5);
    let high_peak = histogram_bins
        .iter()
        .any(|bin| bin.count > 0 && bin.q_low >= 0.7);
    if low_peak && high_peak {
        DistributionModality::BimodalRSB
    } else {
        DistributionModality::Continuous
    }
}

fn parisi_estimate(
    modality: DistributionModality,
    parisi_source: Option<ParisiM4AlphaSource<'_>>,
    q_values: &[f64],
) -> (Option<f64>, ParisiSource) {
    if let Some(source) = parisi_source
        && let Some(m_estimate) = source.replica_overlap.parisi_breakpoint_m
    {
        return (Some(m_estimate), ParisiSource::M4AlphaCascadeReplicaOverlap);
    }

    if modality == DistributionModality::BimodalRSB {
        return (
            two_component_em_low_overlap_weight(q_values),
            ParisiSource::LocalTwoComponentEm,
        );
    }

    (None, ParisiSource::Unavailable)
}

fn two_component_em_low_overlap_weight(q_values: &[f64]) -> Option<f64> {
    if q_values.len() < 3 {
        return None;
    }
    let mut sorted = q_values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let mut low_mean = sorted[0].clamp(0.0, 1.0);
    let mut high_mean = sorted[sorted.len() - 1].clamp(0.0, 1.0);
    if (high_mean - low_mean).abs() < 0.000_001 {
        return None;
    }

    let mut low_weight = 0.5;
    let mut variance = variance(q_values, mean(q_values)).max(0.000_1);
    for _ in 0..32 {
        let mut low_responsibility_sum = 0.0;
        let mut high_responsibility_sum = 0.0;
        let mut low_weighted_sum = 0.0;
        let mut high_weighted_sum = 0.0;

        for value in q_values {
            let low_density = low_weight * gaussian_density(*value, low_mean, variance);
            let high_density = (1.0 - low_weight) * gaussian_density(*value, high_mean, variance);
            let total_density = (low_density + high_density).max(0.000_001);
            let low_responsibility = low_density / total_density;
            let high_responsibility = 1.0 - low_responsibility;
            low_responsibility_sum += low_responsibility;
            high_responsibility_sum += high_responsibility;
            low_weighted_sum += low_responsibility * value;
            high_weighted_sum += high_responsibility * value;
        }

        low_weight = (low_responsibility_sum / q_values.len() as f64).clamp(0.001, 0.999);
        if low_responsibility_sum > 0.000_001 {
            low_mean = (low_weighted_sum / low_responsibility_sum).clamp(0.0, 1.0);
        }
        if high_responsibility_sum > 0.000_001 {
            high_mean = (high_weighted_sum / high_responsibility_sum).clamp(0.0, 1.0);
        }

        let mut variance_sum = 0.0;
        for value in q_values {
            let low_density = low_weight * gaussian_density(*value, low_mean, variance);
            let high_density = (1.0 - low_weight) * gaussian_density(*value, high_mean, variance);
            let total_density = (low_density + high_density).max(0.000_001);
            let low_responsibility = low_density / total_density;
            let high_responsibility = 1.0 - low_responsibility;
            variance_sum += low_responsibility * (value - low_mean).powi(2);
            variance_sum += high_responsibility * (value - high_mean).powi(2);
        }
        variance = (variance_sum / q_values.len() as f64).max(0.000_1);
    }

    if low_mean <= high_mean {
        Some(low_weight.clamp(0.0, 1.0))
    } else {
        Some((1.0 - low_weight).clamp(0.0, 1.0))
    }
}

fn gaussian_density(value: f64, mean: f64, variance: f64) -> f64 {
    let variance = variance.max(0.000_1);
    (-((value - mean).powi(2)) / (2.0 * variance)).exp() / variance.sqrt()
}

fn modality_definition(
    modality: DistributionModality,
    parisi_source: ParisiSource,
) -> &'static str {
    match (modality, parisi_source) {
        (DistributionModality::Trivial, _) => {
            "Fewer than 3 replica pairs available; modality undefined"
        }
        (DistributionModality::Unimodal, _) => {
            "Single peak in P(q) histogram; replica-symmetric descriptive shape"
        }
        (DistributionModality::BimodalRSB, ParisiSource::M4AlphaCascadeReplicaOverlap) => {
            "Two peaks in P(q) with M4-alpha spin-glass Parisi estimate attached"
        }
        (DistributionModality::BimodalRSB, ParisiSource::LocalTwoComponentEm) => {
            "Two peaks in P(q) histogram; local two-component EM estimates the low-overlap mixture weight"
        }
        (DistributionModality::BimodalRSB, _) => {
            "Two peaks in P(q) histogram; spin-glass source unavailable for Parisi estimate"
        }
        (DistributionModality::Continuous, _) => {
            "Smooth P(q) histogram; peak detection fails the bimodal threshold"
        }
    }
}

fn peak_q_values(values: &[f64]) -> Vec<f64> {
    if values.is_empty() {
        return Vec::new();
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let low = sorted[0];
    let high = sorted[sorted.len() - 1];
    if (high - low).abs() < f64::EPSILON {
        vec![low]
    } else {
        vec![low, high]
    }
}

pub fn site(element_selector: impl Into<String>, property: impl Into<String>) -> CascadeSiteKeyV0 {
    CascadeSiteKeyV0 {
        schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
        product: "omena-ensemble.cascade-site-key",
        layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
        feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
        element_selector: element_selector.into(),
        property: property.into(),
    }
}
