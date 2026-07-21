#![allow(clippy::expect_used)]
use omena_cascade::{
    CascadeDeclaration, CascadeKey, CascadeLevel, CascadeOutcome, CascadeProof, CascadeValue,
    LayerRank, ModuleRank, Specificity, summarize_replica_overlap,
};

use crate::{ConsumerId, ProjectionFamily, TopVariantTreatment};
use crate::{
    DetectabilityPhase, DistributionModality, LinearProvenanceTagV0, ModuleGraphEdgeV0,
    ModuleGraphV0, OutcomeMode, ParisiM4AlphaSource, ParisiSource, PartitionHypothesisLabel,
    REPLICA_ENSEMBLE_DEFAULT_PRODUCT_DECISION_MECHANISM_V0, REPLICA_ENSEMBLE_FEATURE_GATE_V0,
    REPLICA_ENSEMBLE_LAYER_MARKER_V0, REPLICA_ENSEMBLE_MECHANISM_SCOPE_V0,
    REPLICA_ENSEMBLE_PRODUCT_SURFACE_V0, REPLICA_ENSEMBLE_SCHEMA_VERSION_V0, ReportOptionsV0,
    ReportRecommendation, RgExponentHandleV0, SamplingPolicy, SpectralMethod,
    UniversalityClassHint, build_cross_file_inconsistency_report, compute_overlap_distribution,
    compute_replica_overlap, compute_sbm_detectability, grn_outcome_projection_policy,
    outcome_projection_policy_for_mode, site,
};

#[test]
fn tier_one_pairwise_overlap_does_not_require_spin_glass_source() {
    let alpha = fixture_replica("alpha.module.css", ["a", "b", "c"]);
    let beta = fixture_replica("beta.module.css", ["a", "x", "c"]);

    let overlap = compute_replica_overlap(
        &alpha.path,
        &beta.path,
        alpha.sites,
        beta.sites,
        OutcomeMode::DefiniteOnly,
    );

    assert_eq!(overlap.schema_version, "0");
    assert_eq!(overlap.layer_marker, REPLICA_ENSEMBLE_LAYER_MARKER_V0);
    assert_eq!(overlap.feature_gate, REPLICA_ENSEMBLE_FEATURE_GATE_V0);
    assert_eq!(overlap.shared_site_count, 3);
    assert_eq!(overlap.agreeing_site_count, 2);
    assert!((overlap.overlap_q - (2.0 / 3.0)).abs() < 0.000_001);
    assert_eq!(overlap.provenance_attributions.len(), 1);
}

#[test]
fn distribution_uses_local_two_component_em_without_m4_alpha_parisi_source() {
    let replicas = vec![
        fixture_replica("brand-a/a.module.css", ["a", "a", "a"]),
        fixture_replica("brand-a/b.module.css", ["a", "a", "a"]),
        fixture_replica("brand-b/c.module.css", ["x", "x", "x"]),
        fixture_replica("brand-b/d.module.css", ["x", "x", "x"]),
    ];

    let distribution = compute_overlap_distribution(
        "/workspace",
        replicas,
        Some(SamplingPolicy::AllPairs),
        OutcomeMode::DefiniteOnly,
        None,
    );

    assert_eq!(distribution.schema_version, "0");
    assert_eq!(distribution.pair_count, 6);
    assert_eq!(distribution.histogram_bin_count, 10);
    assert_eq!(distribution.modality, DistributionModality::BimodalRSB);
    assert_eq!(
        distribution.parisi_m_source,
        ParisiSource::LocalTwoComponentEm
    );
    let parisi_m = distribution
        .parisi_m_estimate
        .expect("bimodal distribution should run local EM");
    assert!(parisi_m > 0.5);
    assert!((parisi_m - distribution.mean_q).abs() > 0.1);
}

#[test]
fn tier_two_consumes_m4_alpha_cascade_replica_overlap_when_available() {
    let replicas = vec![
        fixture_replica("a.module.css", ["a", "a", "a"]),
        fixture_replica("b.module.css", ["a", "a", "a"]),
        fixture_replica("c.module.css", ["x", "x", "x"]),
    ];
    let spin_glass_overlap = summarize_replica_overlap(4, Some(0.625));

    let distribution = compute_overlap_distribution(
        "/workspace",
        replicas,
        None,
        OutcomeMode::DefiniteOnly,
        Some(ParisiM4AlphaSource {
            replica_overlap: &spin_glass_overlap,
        }),
    );

    assert_eq!(
        distribution.parisi_m_source,
        ParisiSource::M4AlphaCascadeReplicaOverlap
    );
    assert_eq!(distribution.parisi_m_estimate, Some(0.625));
}

#[test]
fn sampling_policies_choose_distinct_pair_sets() {
    let replicas = vec![
        fixture_replica("heavy/a.module.css", ["a", "a", "a", "a"]),
        fixture_replica("heavy/b.module.css", ["a", "a", "a"]),
        fixture_replica("light/c.module.css", ["x"]),
        fixture_replica("light/d.module.css", ["x"]),
    ];

    let weighted = crate::overlap::selected_pair_indices(
        &replicas,
        Some(SamplingPolicy::PageRankWeighted { max_pair_count: 2 }),
    );
    let random = crate::overlap::selected_pair_indices(
        &replicas,
        Some(SamplingPolicy::RandomSubset { max_pair_count: 2 }),
    );

    assert_eq!(weighted[0], (0, 1));
    assert_ne!(weighted, random);
}

#[test]
fn sbm_detectability_ranks_partition_hypotheses_and_uses_unknown_safe_annotation() {
    let graph = planted_two_brand_graph();
    let threshold = compute_sbm_detectability(
        "/workspace",
        &graph,
        SpectralMethod::Auto,
        &[
            PartitionHypothesisLabel::DirectoryTree,
            PartitionHypothesisLabel::ComposesCluster,
            PartitionHypothesisLabel::AutoSpectral,
        ],
        None,
    );

    assert_eq!(threshold.schema_version, "0");
    assert_eq!(threshold.layer_marker, REPLICA_ENSEMBLE_LAYER_MARKER_V0);
    assert_eq!(threshold.node_count, 4);
    assert_eq!(threshold.partition_hypothesis_results.len(), 3);
    assert!(threshold.best_lambda_snr >= 0.0);
    assert_ne!(threshold.phase, DetectabilityPhase::Undetectable);
}

#[test]
fn sbm_detectability_recovers_planted_partition_with_likelihood_evidence() {
    let graph = planted_two_brand_graph();
    let threshold = compute_sbm_detectability(
        "/workspace",
        &graph,
        SpectralMethod::Auto,
        &[
            PartitionHypothesisLabel::ComposesCluster,
            PartitionHypothesisLabel::AutoSpectral,
        ],
        Some(RgExponentHandleV0 {
            schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
            product: "omena-ensemble.rg-exponent-handle",
            layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
            feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
            workspace_root: "/workspace".to_string(),
            timestamp: "2026-05-31T00:00:00Z".to_string(),
            digest: "planted".to_string(),
        }),
    );

    assert_eq!(
        threshold.best_hypothesis,
        PartitionHypothesisLabel::ComposesCluster
    );
    assert!(threshold.p_in_estimate > threshold.p_out_estimate);
    assert_eq!(threshold.phase, DetectabilityPhase::Detectable);
    let partitions = &threshold.assortative_partition.partitions;
    assert_eq!(
        partitions["brand-a/a.module.css"],
        partitions["brand-a/b.module.css"]
    );
    assert_eq!(
        partitions["brand-b/c.module.css"],
        partitions["brand-b/d.module.css"]
    );
    assert_ne!(
        partitions["brand-a/a.module.css"],
        partitions["brand-b/c.module.css"]
    );
    let best_result = threshold
        .partition_hypothesis_results
        .iter()
        .find(|result| result.label == threshold.best_hypothesis)
        .expect("best hypothesis result");
    assert!(
        best_result.likelihood_ratio_p_value.unwrap_or(1.0) < 0.25,
        "planted partition should improve over the one-community null: {best_result:?}"
    );
    assert_eq!(
        threshold
            .critical_exponent_annotation
            .as_ref()
            .and_then(|annotation| annotation.universality_class_hint),
        Some(UniversalityClassHint::TokenGraph)
    );
}

#[test]
fn integrated_report_exposes_projection_registry_for_replica_and_grn_consumers() {
    let replicas = vec![
        fixture_replica("brand-a/a.module.css", ["a", "a", "a"]),
        fixture_replica("brand-a/b.module.css", ["a", "a", "a"]),
        fixture_replica("brand-b/c.module.css", ["x", "x", "x"]),
        fixture_replica("brand-b/d.module.css", ["x", "x", "x"]),
    ];
    let report = build_cross_file_inconsistency_report(
        "/workspace",
        replicas,
        &planted_two_brand_graph(),
        OutcomeMode::DefiniteOnly,
        ReportOptionsV0::default(),
        None,
    );
    let default_options = ReportOptionsV0::default();

    assert_eq!(report.schema_version, "0");
    assert_eq!(report.layer_marker, REPLICA_ENSEMBLE_LAYER_MARKER_V0);
    assert_eq!(default_options.schema_version, "0");
    assert_eq!(
        default_options.layer_marker,
        REPLICA_ENSEMBLE_LAYER_MARKER_V0
    );
    assert_eq!(
        default_options.feature_gate,
        REPLICA_ENSEMBLE_FEATURE_GATE_V0
    );
    assert_eq!(
        report.outcome_projection_policy.consumer_id,
        ConsumerId::ReplicaOverlap
    );
    assert_eq!(
        report.outcome_projection_policy.projection,
        ProjectionFamily::BinaryAgreement
    );
    assert_eq!(
        report.recommendation,
        ReportRecommendation::InvestigateRsbBroken
    );
    assert_eq!(report.mechanism_scope, REPLICA_ENSEMBLE_MECHANISM_SCOPE_V0);
    assert_eq!(report.product_surface, REPLICA_ENSEMBLE_PRODUCT_SURFACE_V0);
    assert_eq!(
        report.default_product_decision_mechanism,
        REPLICA_ENSEMBLE_DEFAULT_PRODUCT_DECISION_MECHANISM_V0
    );

    let replica_policy = outcome_projection_policy_for_mode(OutcomeMode::DefiniteOnly);
    let grn_policy = grn_outcome_projection_policy();
    assert_eq!(
        replica_policy.top_variant_treatment,
        TopVariantTreatment::ExcludeFromOverlap
    );
    assert_eq!(
        grn_policy.top_variant_treatment,
        TopVariantTreatment::TreatAsUnknown
    );
}

#[test]
fn public_helper_v0_contracts_carry_replica_ensemble_metadata() {
    let mut snapshot = fixture_replica("alpha.module.css", ["a", "b"]);
    snapshot.sites[0].provenance = Some(LinearProvenanceTagV0 {
        schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
        product: "omena-ensemble.linear-provenance-tag",
        layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
        feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
        semiring_identifier: "lin01",
        label: "fixture".to_string(),
    });
    let graph = planted_two_brand_graph();
    let distribution = compute_overlap_distribution(
        "/workspace",
        vec![
            snapshot.clone(),
            fixture_replica("beta.module.css", ["a", "x"]),
        ],
        Some(SamplingPolicy::AllPairs),
        OutcomeMode::DefiniteOnly,
        None,
    );
    let detectability = compute_sbm_detectability(
        "/workspace",
        &graph,
        SpectralMethod::Auto,
        &[PartitionHypothesisLabel::AutoSpectral],
        None,
    );
    let handle = RgExponentHandleV0 {
        schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
        product: "omena-ensemble.rg-exponent-handle",
        layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
        feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
        workspace_root: "/workspace".to_string(),
        timestamp: "2026-05-25T00:00:00Z".to_string(),
        digest: "fixture".to_string(),
    };

    assert_replica_contract(
        snapshot.schema_version,
        snapshot.layer_marker,
        snapshot.feature_gate,
    );
    assert_replica_contract(
        snapshot.sites[0].schema_version,
        snapshot.sites[0].layer_marker,
        snapshot.sites[0].feature_gate,
    );
    assert_replica_contract(
        snapshot.sites[0].site.schema_version,
        snapshot.sites[0].site.layer_marker,
        snapshot.sites[0].site.feature_gate,
    );
    assert!(
        snapshot.sites[0].provenance.is_some(),
        "fixture site has replica provenance"
    );
    let Some(provenance) = snapshot.sites[0].provenance.as_ref() else {
        return;
    };
    assert_replica_contract(
        provenance.schema_version,
        provenance.layer_marker,
        provenance.feature_gate,
    );
    assert_replica_contract(graph.schema_version, graph.layer_marker, graph.feature_gate);
    assert_replica_contract(
        graph.edges[0].schema_version,
        graph.edges[0].layer_marker,
        graph.edges[0].feature_gate,
    );
    assert_replica_contract(
        distribution.histogram_bins[0].schema_version,
        distribution.histogram_bins[0].layer_marker,
        distribution.histogram_bins[0].feature_gate,
    );
    let partition_result = &detectability.partition_hypothesis_results[0];
    assert_replica_contract(
        partition_result.schema_version,
        partition_result.layer_marker,
        partition_result.feature_gate,
    );
    assert_replica_contract(
        partition_result.partition.schema_version,
        partition_result.partition.layer_marker,
        partition_result.partition.feature_gate,
    );
    assert_replica_contract(
        handle.schema_version,
        handle.layer_marker,
        handle.feature_gate,
    );
}

fn fixture_replica<const N: usize>(
    path: &str,
    winners: [&'static str; N],
) -> crate::ReplicaSnapshotV0 {
    let sites = winners
        .into_iter()
        .enumerate()
        .map(|(index, winner)| crate::ReplicaSiteOutcomeV0 {
            schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
            product: "omena-ensemble.replica-site-outcome",
            layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
            feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
            site: site(format!(".item-{index}"), "color"),
            outcome: definite_outcome(winner),
            provenance: None,
        })
        .collect::<Vec<_>>();

    crate::ReplicaSnapshotV0 {
        schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
        product: "omena-ensemble.replica-snapshot",
        layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
        feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
        path: path.to_string(),
        sites,
    }
}

fn definite_outcome(id: &str) -> CascadeOutcome {
    let declaration = CascadeDeclaration {
        id: id.to_string(),
        property: "color".to_string(),
        value: CascadeValue::Literal(id.to_string()),
        key: CascadeKey {
            level: CascadeLevel::AuthorNormal,
            layer_rank: LayerRank(0),
            scope_proximity: 0,
            specificity: Specificity {
                ids: 0,
                classes: 1,
                elements: 0,
            },
            module_rank: ModuleRank::ZERO,
            source_order: 0,
        },
        specificity_exactness: omena_cascade::SpecificityExactnessV0::Exact,
    };
    CascadeOutcome::Definite {
        proof: Box::new(CascadeProof::from_declaration(&declaration)),
        winner: declaration,
        also_considered: Vec::new(),
    }
}

fn planted_two_brand_graph() -> ModuleGraphV0 {
    ModuleGraphV0 {
        schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
        product: "omena-ensemble.module-graph",
        layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
        feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
        workspace_root: "/workspace".to_string(),
        nodes: vec![
            "brand-a/a.module.css".to_string(),
            "brand-a/b.module.css".to_string(),
            "brand-b/c.module.css".to_string(),
            "brand-b/d.module.css".to_string(),
        ],
        edges: vec![
            ModuleGraphEdgeV0 {
                schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
                product: "omena-ensemble.module-graph-edge",
                layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
                feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
                from_module: "brand-a/a.module.css".to_string(),
                to_module: "brand-a/b.module.css".to_string(),
                edge_kind: "composes",
            },
            ModuleGraphEdgeV0 {
                schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
                product: "omena-ensemble.module-graph-edge",
                layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
                feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
                from_module: "brand-b/c.module.css".to_string(),
                to_module: "brand-b/d.module.css".to_string(),
                edge_kind: "composes",
            },
        ],
    }
}

fn assert_replica_contract(
    schema_version: &'static str,
    layer_marker: &'static str,
    feature_gate: &'static str,
) {
    assert_eq!(schema_version, REPLICA_ENSEMBLE_SCHEMA_VERSION_V0);
    assert_eq!(layer_marker, REPLICA_ENSEMBLE_LAYER_MARKER_V0);
    assert_eq!(feature_gate, REPLICA_ENSEMBLE_FEATURE_GATE_V0);
}
