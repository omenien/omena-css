use omena_cascade::{
    CascadeDeclaration, CascadeKey, CascadeLevel, CascadeOutcome, CascadeProof, CascadeValue,
    LayerRank, Specificity, summarize_replica_overlap,
};

use crate::{ConsumerId, ProjectionFamily, TopVariantTreatment};
use crate::{
    DetectabilityPhase, DistributionModality, ModuleGraphEdgeV0, ModuleGraphV0, OutcomeMode,
    ParisiM4AlphaSource, ParisiSource, PartitionHypothesisLabel, REPLICA_ENSEMBLE_FEATURE_GATE_V0,
    REPLICA_ENSEMBLE_LAYER_MARKER_V0, ReportOptionsV0, ReportRecommendation, SamplingPolicy,
    SpectralMethod, build_cross_file_inconsistency_report, compute_overlap_distribution,
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
fn distribution_uses_local_fallback_without_m4_alpha_parisi_source() {
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
    assert_eq!(distribution.parisi_m_source, ParisiSource::LocalEmFallback);
    assert!(distribution.parisi_m_estimate.is_some());
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

    assert_eq!(report.schema_version, "0");
    assert_eq!(report.layer_marker, REPLICA_ENSEMBLE_LAYER_MARKER_V0);
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

fn fixture_replica<const N: usize>(
    path: &str,
    winners: [&'static str; N],
) -> crate::ReplicaSnapshotV0 {
    let sites = winners
        .into_iter()
        .enumerate()
        .map(|(index, winner)| crate::ReplicaSiteOutcomeV0 {
            site: site(format!(".item-{index}"), "color"),
            outcome: definite_outcome(winner),
            provenance: None,
        })
        .collect::<Vec<_>>();

    crate::ReplicaSnapshotV0 {
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
            source_order: 0,
        },
    };
    CascadeOutcome::Definite {
        proof: CascadeProof::from_declaration(&declaration),
        winner: declaration,
        also_considered: Vec::new(),
    }
}

fn planted_two_brand_graph() -> ModuleGraphV0 {
    ModuleGraphV0 {
        workspace_root: "/workspace".to_string(),
        nodes: vec![
            "brand-a/a.module.css".to_string(),
            "brand-a/b.module.css".to_string(),
            "brand-b/c.module.css".to_string(),
            "brand-b/d.module.css".to_string(),
        ],
        edges: vec![
            ModuleGraphEdgeV0 {
                from_module: "brand-a/a.module.css".to_string(),
                to_module: "brand-a/b.module.css".to_string(),
                edge_kind: "composes",
            },
            ModuleGraphEdgeV0 {
                from_module: "brand-b/c.module.css".to_string(),
                to_module: "brand-b/d.module.css".to_string(),
                edge_kind: "composes",
            },
        ],
    }
}
