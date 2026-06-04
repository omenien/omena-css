use std::collections::{BTreeMap, BTreeSet};

use crate::overlap::{compute_overlap_distribution, outcome_projection_policy_for_mode};
use crate::types::{
    AgreementVerdict, CriticalExponentAnnotationV0, CrossFileInconsistencyReportV0,
    DetectabilityPhase, ModuleGraphV0, OutcomeMode, ParisiM4AlphaSource, PartitionEstimateV0,
    PartitionHypothesisLabel, PartitionHypothesisResultV0,
    REPLICA_ENSEMBLE_DEFAULT_PRODUCT_DECISION_MECHANISM_V0, REPLICA_ENSEMBLE_FEATURE_GATE_V0,
    REPLICA_ENSEMBLE_LAYER_MARKER_V0, REPLICA_ENSEMBLE_MECHANISM_SCOPE_V0,
    REPLICA_ENSEMBLE_PRODUCT_SURFACE_V0, REPLICA_ENSEMBLE_SCHEMA_VERSION_V0, ReplicaOverlapV0,
    ReplicaSnapshotV0, ReportOptionsV0, ReportRecommendation, RgExponentHandleV0,
    SBMDetectabilityThresholdV0, SamplingPolicy, SpectralMethod, UniversalityClassHint,
};

pub fn compute_sbm_detectability(
    workspace_root: &str,
    module_graph: &ModuleGraphV0,
    spectral_method: SpectralMethod,
    partition_hypotheses: &[PartitionHypothesisLabel],
    rg_exponent_handle: Option<RgExponentHandleV0>,
) -> SBMDetectabilityThresholdV0 {
    let hypotheses = if partition_hypotheses.is_empty() {
        vec![PartitionHypothesisLabel::AutoSpectral]
    } else {
        partition_hypotheses.to_vec()
    };

    let mut results = hypotheses
        .iter()
        .map(|label| evaluate_partition_hypothesis(label, module_graph))
        .collect::<Vec<_>>();
    results.sort_by(|left, right| {
        left.bic_relative
            .total_cmp(&right.bic_relative)
            .then_with(|| left.label.cmp(&right.label))
    });

    let best = results.first().cloned().unwrap_or_else(|| {
        evaluate_partition_hypothesis(&PartitionHypothesisLabel::AutoSpectral, module_graph)
    });
    let (p_in_estimate, p_out_estimate) =
        edge_probabilities(module_graph, &best.partition.partitions);
    let phase = detectability_phase(best.lambda_snr_per_hypothesis);

    SBMDetectabilityThresholdV0 {
        schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
        product: "omena-ensemble.sbm-detectability-threshold",
        layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
        feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
        workspace_root: workspace_root.to_string(),
        node_count: module_graph.nodes.len(),
        edge_count: canonical_edges(module_graph).len(),
        assortative_partition: best.partition.clone(),
        p_in_estimate,
        p_out_estimate,
        lambda_snr: best.lambda_snr_per_hypothesis,
        phase,
        spectral_method_used: resolve_spectral_method(spectral_method, module_graph),
        k_community_estimate: best.k_communities,
        partition_hypothesis_results: results,
        best_hypothesis: best.label,
        best_lambda_snr: best.lambda_snr_per_hypothesis,
        critical_exponent_annotation: rg_exponent_handle
            .map(|handle| critical_exponent_annotation(handle, best.lambda_snr_per_hypothesis)),
    }
}

pub fn build_cross_file_inconsistency_report(
    workspace_root: &str,
    replicas: impl IntoIterator<Item = ReplicaSnapshotV0>,
    module_graph: &ModuleGraphV0,
    outcome_mode: OutcomeMode,
    options: ReportOptionsV0,
    parisi_source: Option<ParisiM4AlphaSource<'_>>,
) -> CrossFileInconsistencyReportV0 {
    let replicas = replicas.into_iter().collect::<Vec<_>>();
    let distribution = compute_overlap_distribution(
        workspace_root,
        replicas.clone(),
        options.sampling_policy,
        outcome_mode,
        parisi_source,
    );
    let detectability = compute_sbm_detectability(
        workspace_root,
        module_graph,
        options.spectral_method,
        &options.partition_hypotheses,
        options.rg_exponent_handle,
    );
    let top_disagreement_pairs =
        top_disagreement_pairs(&replicas, outcome_mode, options.sampling_policy);
    let recommendation = recommendation_for(&distribution.modality, detectability.phase);

    CrossFileInconsistencyReportV0 {
        schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
        product: "omena-ensemble.cross-file-inconsistency-report",
        layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
        feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
        mechanism_scope: REPLICA_ENSEMBLE_MECHANISM_SCOPE_V0,
        product_surface: REPLICA_ENSEMBLE_PRODUCT_SURFACE_V0,
        default_product_decision_mechanism: REPLICA_ENSEMBLE_DEFAULT_PRODUCT_DECISION_MECHANISM_V0,
        workspace_root: workspace_root.to_string(),
        distribution,
        detectability,
        top_disagreement_pairs,
        recommendation,
        outcome_projection_policy: outcome_projection_policy_for_mode(outcome_mode),
    }
}

fn evaluate_partition_hypothesis(
    label: &PartitionHypothesisLabel,
    graph: &ModuleGraphV0,
) -> PartitionHypothesisResultV0 {
    let partition = partition_for(label, graph);
    let (p_in, p_out) = edge_probabilities(graph, &partition.partitions);
    let lambda_snr = lambda_snr(p_in, p_out, &partition.community_size_distribution);
    let log_likelihood = log_likelihood(graph, &partition.partitions, p_in, p_out);
    let null_log_likelihood = null_log_likelihood(graph);
    let k_communities = partition.community_size_distribution.len().max(1);
    let parameter_count = k_communities + 1;
    let aic = 2.0 * parameter_count as f64 - 2.0 * log_likelihood;
    let bic =
        parameter_count as f64 * (graph.nodes.len().max(1) as f64).ln() - 2.0 * log_likelihood;

    PartitionHypothesisResultV0 {
        schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
        product: "omena-ensemble.partition-hypothesis-result",
        layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
        feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
        label: label.clone(),
        partition,
        lambda_snr_per_hypothesis: lambda_snr,
        log_likelihood,
        aic_relative: aic,
        bic_relative: bic,
        likelihood_ratio_p_value: Some(likelihood_ratio_p_value(
            log_likelihood,
            null_log_likelihood,
            k_communities,
        )),
        k_communities,
    }
}

fn partition_for(label: &PartitionHypothesisLabel, graph: &ModuleGraphV0) -> PartitionEstimateV0 {
    let partitions = match label {
        PartitionHypothesisLabel::DirectoryTree => directory_partition(&graph.nodes),
        PartitionHypothesisLabel::ComposesCluster => component_partition(graph),
        PartitionHypothesisLabel::BrandTheme => keyword_partition(&graph.nodes, "brand"),
        PartitionHypothesisLabel::AutoSpectral => degree_partition(graph),
        PartitionHypothesisLabel::UserSupplied(prefix) => keyword_partition(&graph.nodes, prefix),
    };
    partition_estimate(partitions)
}

fn partition_estimate(partitions: BTreeMap<String, u32>) -> PartitionEstimateV0 {
    let mut sizes = BTreeMap::<u32, usize>::new();
    for community in partitions.values() {
        *sizes.entry(*community).or_insert(0) += 1;
    }
    PartitionEstimateV0 {
        schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
        product: "omena-ensemble.partition-estimate",
        layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
        feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
        partitions,
        community_size_distribution: sizes.into_values().collect(),
    }
}

fn directory_partition(nodes: &[String]) -> BTreeMap<String, u32> {
    let mut community_ids = BTreeMap::<String, u32>::new();
    nodes
        .iter()
        .map(|node| {
            let key = node
                .split('/')
                .find(|segment| !segment.is_empty() && *segment != "src")
                .unwrap_or(node.as_str())
                .to_string();
            let next_id = community_ids.len() as u32;
            let community = *community_ids.entry(key).or_insert(next_id);
            (node.clone(), community)
        })
        .collect()
}

fn keyword_partition(nodes: &[String], keyword: &str) -> BTreeMap<String, u32> {
    nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let community = if node.contains(keyword) {
                1
            } else {
                (index % 2) as u32
            };
            (node.clone(), community)
        })
        .collect()
}

fn degree_partition(graph: &ModuleGraphV0) -> BTreeMap<String, u32> {
    let mut degrees = graph
        .nodes
        .iter()
        .map(|node| (node.clone(), 0usize))
        .collect::<BTreeMap<_, _>>();
    for (left, right) in canonical_edges(graph) {
        if let Some(degree) = degrees.get_mut(&left) {
            *degree += 1;
        }
        if let Some(degree) = degrees.get_mut(&right) {
            *degree += 1;
        }
    }
    let mut sorted_degrees = degrees.values().copied().collect::<Vec<_>>();
    sorted_degrees.sort_unstable();
    let median = sorted_degrees
        .get(sorted_degrees.len().saturating_sub(1) / 2)
        .copied()
        .unwrap_or(0);

    degrees
        .into_iter()
        .map(|(node, degree)| (node, u32::from(degree > median)))
        .collect()
}

fn component_partition(graph: &ModuleGraphV0) -> BTreeMap<String, u32> {
    let mut parent = graph
        .nodes
        .iter()
        .map(|node| (node.clone(), node.clone()))
        .collect::<BTreeMap<_, _>>();

    for (left, right) in canonical_edges(graph) {
        union(&mut parent, &left, &right);
    }

    let mut community_ids = BTreeMap::<String, u32>::new();
    graph
        .nodes
        .iter()
        .map(|node| {
            let root = find(&parent, node);
            let next_id = community_ids.len() as u32;
            let community = *community_ids.entry(root).or_insert(next_id);
            (node.clone(), community)
        })
        .collect()
}

fn union(parent: &mut BTreeMap<String, String>, left: &str, right: &str) {
    let left_root = find(parent, left);
    let right_root = find(parent, right);
    if left_root != right_root {
        parent.insert(right_root, left_root);
    }
}

fn find(parent: &BTreeMap<String, String>, node: &str) -> String {
    let mut current = node.to_string();
    while let Some(next) = parent.get(&current) {
        if next == &current {
            break;
        }
        current = next.clone();
    }
    current
}

fn edge_probabilities(graph: &ModuleGraphV0, partitions: &BTreeMap<String, u32>) -> (f64, f64) {
    let edges = canonical_edges(graph);
    let edge_set = edges.iter().cloned().collect::<BTreeSet<_>>();
    let mut possible_in = 0usize;
    let mut possible_out = 0usize;
    let mut observed_in = 0usize;
    let mut observed_out = 0usize;

    for left_index in 0..graph.nodes.len() {
        for right_index in left_index + 1..graph.nodes.len() {
            let left = &graph.nodes[left_index];
            let right = &graph.nodes[right_index];
            let key = canonical_pair(left, right);
            let same_partition = partitions.get(left) == partitions.get(right);
            if same_partition {
                possible_in += 1;
                if edge_set.contains(&key) {
                    observed_in += 1;
                }
            } else {
                possible_out += 1;
                if edge_set.contains(&key) {
                    observed_out += 1;
                }
            }
        }
    }

    (
        probability(observed_in, possible_in),
        probability(observed_out, possible_out),
    )
}

fn canonical_edges(graph: &ModuleGraphV0) -> Vec<(String, String)> {
    graph
        .edges
        .iter()
        .filter(|edge| edge.from_module != edge.to_module)
        .map(|edge| canonical_pair(&edge.from_module, &edge.to_module))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn canonical_pair(left: &str, right: &str) -> (String, String) {
    if left <= right {
        (left.to_string(), right.to_string())
    } else {
        (right.to_string(), left.to_string())
    }
}

fn probability(observed: usize, possible: usize) -> f64 {
    if possible == 0 {
        0.0
    } else {
        observed as f64 / possible as f64
    }
}

fn lambda_snr(p_in: f64, p_out: f64, community_size_distribution: &[usize]) -> f64 {
    let community_count = community_size_distribution.len().max(1);
    let mean_community_size = if community_size_distribution.is_empty() {
        1.0
    } else {
        community_size_distribution.iter().sum::<usize>() as f64
            / community_size_distribution.len() as f64
    };
    let denominator = (p_in + community_count.saturating_sub(1) as f64 * p_out).max(0.000_001);
    let contrast = (p_in - p_out).max(0.0);
    contrast * contrast * mean_community_size.max(1.0) / denominator
}

fn log_likelihood(
    graph: &ModuleGraphV0,
    partitions: &BTreeMap<String, u32>,
    p_in: f64,
    p_out: f64,
) -> f64 {
    let edge_set = canonical_edges(graph).into_iter().collect::<BTreeSet<_>>();
    let mut log_likelihood = 0.0;

    for left_index in 0..graph.nodes.len() {
        for right_index in left_index + 1..graph.nodes.len() {
            let left = &graph.nodes[left_index];
            let right = &graph.nodes[right_index];
            let edge_present = edge_set.contains(&canonical_pair(left, right));
            let same_partition = partitions.get(left) == partitions.get(right);
            let probability = if same_partition { p_in } else { p_out }.clamp(0.000_001, 0.999_999);
            log_likelihood += if edge_present {
                probability.ln()
            } else {
                (1.0 - probability).ln()
            };
        }
    }

    log_likelihood
}

fn null_log_likelihood(graph: &ModuleGraphV0) -> f64 {
    let edge_count = canonical_edges(graph).len();
    let possible_edge_count = graph
        .nodes
        .len()
        .saturating_mul(graph.nodes.len().saturating_sub(1))
        / 2;
    let global_probability =
        probability(edge_count, possible_edge_count).clamp(0.000_001, 0.999_999);
    let partitions = graph
        .nodes
        .iter()
        .map(|node| (node.clone(), 0))
        .collect::<BTreeMap<_, _>>();
    log_likelihood(graph, &partitions, global_probability, global_probability)
}

fn likelihood_ratio_p_value(
    alternative_log_likelihood: f64,
    null_log_likelihood: f64,
    k_communities: usize,
) -> f64 {
    let statistic = (2.0 * (alternative_log_likelihood - null_log_likelihood)).max(0.0);
    let degrees_of_freedom = k_communities.saturating_sub(1).max(1) as f64;
    chi_square_survival_wilson_hilferty(statistic, degrees_of_freedom)
}

fn chi_square_survival_wilson_hilferty(statistic: f64, degrees_of_freedom: f64) -> f64 {
    if statistic <= 0.0 {
        return 1.0;
    }
    let z = ((statistic / degrees_of_freedom).powf(1.0 / 3.0)
        - (1.0 - 2.0 / (9.0 * degrees_of_freedom)))
        / (2.0 / (9.0 * degrees_of_freedom)).sqrt();
    (1.0 - normal_cdf_approx(z)).clamp(0.0, 1.0)
}

fn normal_cdf_approx(value: f64) -> f64 {
    let sign = if value < 0.0 { -1.0 } else { 1.0 };
    let x = value.abs();
    let t = 1.0 / (1.0 + 0.231_641_9 * x);
    let density = 0.398_942_280_401_432_7 * (-0.5 * x * x).exp();
    let tail = density
        * (((((1.330_274_429 * t - 1.821_255_978) * t + 1.781_477_937) * t - 0.356_563_782) * t
            + 0.319_381_530)
            * t);
    if sign > 0.0 { 1.0 - tail } else { tail }
}

fn detectability_phase(lambda_snr: f64) -> DetectabilityPhase {
    if lambda_snr >= 1.1 {
        DetectabilityPhase::Detectable
    } else if lambda_snr >= 0.9 {
        DetectabilityPhase::Borderline
    } else {
        DetectabilityPhase::Undetectable
    }
}

fn resolve_spectral_method(method: SpectralMethod, graph: &ModuleGraphV0) -> SpectralMethod {
    match method {
        SpectralMethod::Auto if graph.nodes.len() >= 4 => SpectralMethod::DegreeCorrected,
        SpectralMethod::Auto => SpectralMethod::Spectral,
        other => other,
    }
}

fn critical_exponent_annotation(
    handle: RgExponentHandleV0,
    lambda_snr: f64,
) -> CriticalExponentAnnotationV0 {
    CriticalExponentAnnotationV0 {
        schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
        product: "omena-ensemble.critical-exponent-annotation",
        layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
        feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
        rg_exponent_triple_handle: Some(handle),
        detectability_exponent_beta_est: (lambda_snr - 1.0).abs(),
        universality_class_hint: Some(universality_class_hint_for_detectability(lambda_snr)),
        agreement_with_rg_fixed_point: if lambda_snr >= 1.0 {
            AgreementVerdict::Agree
        } else {
            AgreementVerdict::NotApplicable
        },
    }
}

fn universality_class_hint_for_detectability(lambda_snr: f64) -> UniversalityClassHint {
    if lambda_snr >= 1.1 {
        UniversalityClassHint::TokenGraph
    } else if lambda_snr >= 0.9 {
        UniversalityClassHint::ComponentScoped
    } else {
        UniversalityClassHint::UtilityDominated
    }
}

fn top_disagreement_pairs(
    replicas: &[ReplicaSnapshotV0],
    outcome_mode: OutcomeMode,
    sampling_policy: Option<SamplingPolicy>,
) -> Vec<ReplicaOverlapV0> {
    let mut overlaps = Vec::new();
    let mut remaining_budget = match sampling_policy {
        Some(SamplingPolicy::PageRankWeighted { max_pair_count })
        | Some(SamplingPolicy::RandomSubset { max_pair_count }) => max_pair_count,
        Some(SamplingPolicy::AllPairs) | None => usize::MAX,
    };

    for alpha_index in 0..replicas.len() {
        for beta_index in alpha_index + 1..replicas.len() {
            if remaining_budget == 0 {
                break;
            }
            remaining_budget = remaining_budget.saturating_sub(1);
            let alpha = &replicas[alpha_index];
            let beta = &replicas[beta_index];
            overlaps.push(crate::overlap::compute_replica_overlap(
                &alpha.path,
                &beta.path,
                alpha.sites.clone(),
                beta.sites.clone(),
                outcome_mode,
            ));
        }
    }

    overlaps.sort_by(|left, right| {
        left.overlap_q
            .total_cmp(&right.overlap_q)
            .then_with(|| left.replica_alpha_path.cmp(&right.replica_alpha_path))
            .then_with(|| left.replica_beta_path.cmp(&right.replica_beta_path))
    });
    overlaps.truncate(5);
    overlaps
}

fn recommendation_for(
    modality: &crate::DistributionModality,
    phase: DetectabilityPhase,
) -> ReportRecommendation {
    match (modality, phase) {
        (_, DetectabilityPhase::Undetectable) => ReportRecommendation::UndetectablePhase,
        (crate::DistributionModality::BimodalRSB, DetectabilityPhase::Detectable) => {
            ReportRecommendation::InvestigateRsbBroken
        }
        _ => ReportRecommendation::NoActionNeeded,
    }
}
