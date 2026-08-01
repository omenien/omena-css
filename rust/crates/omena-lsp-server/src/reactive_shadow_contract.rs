use omena_reactive::{
    ReactiveDivergenceClassV0, ReactiveDivergenceDispositionV0, ReactiveObservationPhaseV0,
};

use crate::reactive_shadow::ReactiveShadowFlushReportV0;

/// Comparison dimensions require two independently sourced values or a real
/// oracle. Snapshot stamps are deliberately excluded: their projected value
/// is only an encode/decode round trip, while corpus revision stability is
/// checked by [`evaluate_proposed_authority_reduction`]. Adding a second
/// reactive projection node solely to increase this denominator would add
/// observer work without adding an independent source of truth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ReactiveShadowParityDimensionV0 {
    TargetSet,
    DeltaFoldRebuild,
    DeliveryDecision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReactiveShadowParitySummaryV0 {
    pub(crate) flush_count: usize,
    pub(crate) target_set_equality_count: usize,
    pub(crate) delta_fold_rebuild_count: usize,
    pub(crate) delivery_decision_equality_count: usize,
    /// One-armed liveness observation, not a parity dimension.
    pub(crate) settled_without_pending_count: usize,
    pub(crate) baseline_digest_observation_count: usize,
    pub(crate) optimizing_digest_observation_count: usize,
    pub(crate) suppressed_delivery_observation_count: usize,
    pub(crate) unclassified_divergences: Vec<ReactiveShadowParityDimensionV0>,
}

pub(crate) fn evaluate_reactive_shadow_parity(
    reports: &[ReactiveShadowFlushReportV0],
) -> ReactiveShadowParitySummaryV0 {
    let mut summary = ReactiveShadowParitySummaryV0 {
        flush_count: reports.len(),
        target_set_equality_count: 0,
        delta_fold_rebuild_count: 0,
        delivery_decision_equality_count: 0,
        settled_without_pending_count: 0,
        baseline_digest_observation_count: 0,
        optimizing_digest_observation_count: 0,
        suppressed_delivery_observation_count: 0,
        unclassified_divergences: Vec::new(),
    };

    for report in reports {
        if report.expected_target_uris == report.projected_target_uris {
            summary.target_set_equality_count += 1;
        } else {
            summary
                .unclassified_divergences
                .push(ReactiveShadowParityDimensionV0::TargetSet);
        }
        if report.delta_fold_matches_full_rebuild {
            summary.delta_fold_rebuild_count += 1;
        } else {
            summary
                .unclassified_divergences
                .push(ReactiveShadowParityDimensionV0::DeltaFoldRebuild);
        }
        if report.expected_delivery_decisions == report.projected_delivery_decisions {
            summary.delivery_decision_equality_count += 1;
        } else {
            summary
                .unclassified_divergences
                .push(ReactiveShadowParityDimensionV0::DeliveryDecision);
        }
        if report.settled_without_pending_work {
            summary.settled_without_pending_count += 1;
        }

        summary.baseline_digest_observation_count += report.expected_baseline_digests.len();
        summary.optimizing_digest_observation_count += report.expected_optimizing_digests.len();
        summary.suppressed_delivery_observation_count += report
            .expected_delivery_decisions
            .iter()
            .filter(|decision| !decision.should_deliver)
            .count();
    }
    summary
}

pub(crate) fn divergence_disposition(
    class_id: &str,
    phase: ReactiveObservationPhaseV0,
) -> ReactiveDivergenceDispositionV0 {
    ReactiveDivergenceClassV0::from_id(class_id)
        .map(|class| class.disposition(phase))
        .unwrap_or(ReactiveDivergenceDispositionV0::Blocker)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ProposedAuthorityViolationV0 {
    CorpusRevision,
    ImmutableReadSnapshot,
    DemandLedger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProposedAuthorityReductionSummaryV0 {
    pub(crate) checked_flush_count: usize,
    pub(crate) violations: Vec<ProposedAuthorityViolationV0>,
}

/// Checks the proposed reduction against existing Tide, snapshot, and
/// observer-liveness witnesses. It does not claim these names are product
/// authorities or transfer ownership to the observer.
pub(crate) fn evaluate_proposed_authority_reduction(
    reports: &[ReactiveShadowFlushReportV0],
) -> ProposedAuthorityReductionSummaryV0 {
    let mut violations = Vec::new();
    for report in reports {
        if report.corpus_revision_reads.len() < 2
            || report
                .corpus_revision_reads
                .iter()
                .any(|revision| *revision != report.expected_stamps.corpus_revision)
        {
            violations.push(ProposedAuthorityViolationV0::CorpusRevision);
        }
        if report.snapshot_read_side_effect_count != 0 {
            violations.push(ProposedAuthorityViolationV0::ImmutableReadSnapshot);
        }
        if report.stale_live_demand_count != 0 || !report.observer_liveness_grounded {
            violations.push(ProposedAuthorityViolationV0::DemandLedger);
        }
    }
    ProposedAuthorityReductionSummaryV0 {
        checked_flush_count: reports.len(),
        violations,
    }
}
