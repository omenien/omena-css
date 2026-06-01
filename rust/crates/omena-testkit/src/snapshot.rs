use serde::Serialize;

/// Snapshot-governance policy locked by the M4 testkit substrate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaTestkitSnapshotGovernancePolicyV0 {
    /// Fixture grammar that owns snapshot identity.
    pub fixture_grammar: &'static str,
    /// Manifest schema that binds snapshots back to fixture ids.
    pub snapshot_manifest_schema: &'static str,
    /// Known-failure schema that requires per-fixture rationale and review.
    pub known_failure_schema: &'static str,
    /// Whether a single global snapshot disable switch is accepted.
    pub allow_global_disable: bool,
    /// Whether snapshot updates require explicit human review.
    pub update_requires_review: bool,
    /// Action for snapshots not referenced by any fixture.
    pub unreferenced_action: &'static str,
    /// Maximum age before a hot snapshot requires review.
    pub hot_snapshot_max_age_days: u32,
    /// Maximum age before a known failure requires review.
    pub known_failure_review_period_days: u32,
}

/// One known-failure record attached to a fixture-scoped snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaTestkitKnownFailureRecordV0 {
    /// Human-readable rationale for carrying the failure.
    pub rationale: &'static str,
    /// Owner or lane responsible for review.
    pub owner: &'static str,
    /// Age of the known-failure record.
    pub age_days: u32,
    /// Whether the entry has an explicit expiry or review policy.
    pub has_expiry_or_review_policy: bool,
}

/// Seed snapshot entry used to prove the governance decision table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaTestkitSnapshotGovernanceSeedV0 {
    /// Stable snapshot id.
    pub snapshot_id: &'static str,
    /// Fixture id that should own the snapshot.
    pub fixture_id: &'static str,
    /// Snapshot path relative to the corpus root.
    pub snapshot_path: &'static str,
    /// Whether this snapshot is referenced by a fixture manifest.
    pub referenced_by_fixture: bool,
    /// Whether this snapshot belongs to a frequently edited product path.
    pub hot_snapshot: bool,
    /// Snapshot age in days.
    pub age_days: u32,
    /// Optional known-failure metadata.
    pub known_failure: Option<OmenaTestkitKnownFailureRecordV0>,
}

/// Per-seed governance verdict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaTestkitSnapshotGovernanceVerdictV0 {
    /// Stable snapshot id.
    pub snapshot_id: &'static str,
    /// Fixture id that should own the snapshot.
    pub fixture_id: &'static str,
    /// Whether the seed is rejected as unreferenced.
    pub rejected_unreferenced: bool,
    /// Whether a hot snapshot requires age review.
    pub hot_snapshot_review_required: bool,
    /// Whether known-failure metadata requires review.
    pub known_failure_review_required: bool,
    /// Whether the known failure includes rationale plus expiry or review policy.
    pub known_failure_policy_complete: bool,
    /// Final machine action for the governance seed.
    pub action: &'static str,
}

/// Snapshot-governance foundation report for Axis A.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaTestkitSnapshotGovernanceReportV0 {
    /// Schema version.
    pub schema_version: &'static str,
    /// Product surface.
    pub product: &'static str,
    /// Fixture grammar that owns snapshot identity.
    pub fixture_grammar: &'static str,
    /// Policy applied by the report.
    pub policy: OmenaTestkitSnapshotGovernancePolicyV0,
    /// Governance seed count.
    pub seed_count: usize,
    /// Whether global snapshot-disable behavior is rejected by policy.
    pub global_disable_rejected: bool,
    /// Whether unreferenced snapshots are rejected.
    pub unreferenced_reject_ready: bool,
    /// Whether hot snapshots are age-audited.
    pub hot_snapshot_age_audit_ready: bool,
    /// Whether known failures require rationale and expiry or review policy.
    pub known_failure_policy_ready: bool,
    /// Rejected unreferenced snapshot count.
    pub rejected_unreferenced_count: usize,
    /// Hot snapshot review-required count.
    pub hot_snapshot_review_required_count: usize,
    /// Known-failure count.
    pub known_failure_count: usize,
    /// Known-failure review-required count.
    pub known_failure_review_required_count: usize,
    /// Per-seed verdicts.
    pub verdicts: Vec<OmenaTestkitSnapshotGovernanceVerdictV0>,
}

const SNAPSHOT_POLICY: OmenaTestkitSnapshotGovernancePolicyV0 =
    OmenaTestkitSnapshotGovernancePolicyV0 {
        fixture_grammar: "omena-fixture-v0",
        snapshot_manifest_schema: "omena-testkit-snapshot-manifest-v0",
        known_failure_schema: "omena-testkit-known-failures-v0",
        allow_global_disable: false,
        update_requires_review: true,
        unreferenced_action: "reject",
        hot_snapshot_max_age_days: 14,
        known_failure_review_period_days: 30,
    };

const SNAPSHOT_GOVERNANCE_SEEDS: &[OmenaTestkitSnapshotGovernanceSeedV0] = &[
    OmenaTestkitSnapshotGovernanceSeedV0 {
        snapshot_id: "style-facts-current",
        fixture_id: "shared-style-fixture",
        snapshot_path: "snapshots/shared-style-fixture/style-facts.snap.json",
        referenced_by_fixture: true,
        hot_snapshot: false,
        age_days: 3,
        known_failure: None,
    },
    OmenaTestkitSnapshotGovernanceSeedV0 {
        snapshot_id: "orphan-style-facts",
        fixture_id: "removed-fixture",
        snapshot_path: "snapshots/removed-fixture/style-facts.snap.json",
        referenced_by_fixture: false,
        hot_snapshot: false,
        age_days: 2,
        known_failure: None,
    },
    OmenaTestkitSnapshotGovernanceSeedV0 {
        snapshot_id: "hot-lsp-hover",
        fixture_id: "lsp-hover-scenario",
        snapshot_path: "snapshots/lsp-hover-scenario/hover.snap.json",
        referenced_by_fixture: true,
        hot_snapshot: true,
        age_days: 21,
        known_failure: None,
    },
    OmenaTestkitSnapshotGovernanceSeedV0 {
        snapshot_id: "known-failure-source-rename",
        fixture_id: "dynamic-source-rename-scenario",
        snapshot_path: "snapshots/dynamic-source-rename-scenario/rename.snap.json",
        referenced_by_fixture: true,
        hot_snapshot: true,
        age_days: 5,
        known_failure: Some(OmenaTestkitKnownFailureRecordV0 {
            rationale: "issue-38 dynamic identifier rename path is tracked as an M4 gate",
            owner: "m4-axis-a-testkit",
            age_days: 31,
            has_expiry_or_review_policy: true,
        }),
    },
];

/// Summarize the M4 snapshot-governance foundation.
pub fn summarize_omena_testkit_snapshot_governance_report() -> OmenaTestkitSnapshotGovernanceReportV0
{
    let verdicts = SNAPSHOT_GOVERNANCE_SEEDS
        .iter()
        .map(snapshot_governance_verdict)
        .collect::<Vec<_>>();
    let rejected_unreferenced_count = verdicts
        .iter()
        .filter(|verdict| verdict.rejected_unreferenced)
        .count();
    let hot_snapshot_review_required_count = verdicts
        .iter()
        .filter(|verdict| verdict.hot_snapshot_review_required)
        .count();
    let known_failure_count = SNAPSHOT_GOVERNANCE_SEEDS
        .iter()
        .filter(|seed| seed.known_failure.is_some())
        .count();
    let known_failure_review_required_count = verdicts
        .iter()
        .filter(|verdict| verdict.known_failure_review_required)
        .count();
    let known_failure_policy_ready = SNAPSHOT_GOVERNANCE_SEEDS
        .iter()
        .filter_map(|seed| seed.known_failure.as_ref())
        .all(|known_failure| {
            !known_failure.rationale.trim().is_empty()
                && !known_failure.owner.trim().is_empty()
                && known_failure.has_expiry_or_review_policy
        });

    OmenaTestkitSnapshotGovernanceReportV0 {
        schema_version: "0",
        product: "omena-testkit.snapshot-governance",
        fixture_grammar: SNAPSHOT_POLICY.fixture_grammar,
        policy: SNAPSHOT_POLICY,
        seed_count: SNAPSHOT_GOVERNANCE_SEEDS.len(),
        global_disable_rejected: !SNAPSHOT_POLICY.allow_global_disable,
        unreferenced_reject_ready: rejected_unreferenced_count > 0
            && SNAPSHOT_POLICY.unreferenced_action == "reject",
        hot_snapshot_age_audit_ready: hot_snapshot_review_required_count > 0,
        known_failure_policy_ready,
        rejected_unreferenced_count,
        hot_snapshot_review_required_count,
        known_failure_count,
        known_failure_review_required_count,
        verdicts,
    }
}

fn snapshot_governance_verdict(
    seed: &OmenaTestkitSnapshotGovernanceSeedV0,
) -> OmenaTestkitSnapshotGovernanceVerdictV0 {
    let rejected_unreferenced =
        !seed.referenced_by_fixture && SNAPSHOT_POLICY.unreferenced_action == "reject";
    let hot_snapshot_review_required =
        seed.hot_snapshot && seed.age_days > SNAPSHOT_POLICY.hot_snapshot_max_age_days;
    let known_failure_review_required = seed.known_failure.as_ref().is_some_and(|known_failure| {
        known_failure.age_days > SNAPSHOT_POLICY.known_failure_review_period_days
    });
    let known_failure_policy_complete = seed.known_failure.as_ref().is_none_or(|known_failure| {
        !known_failure.rationale.trim().is_empty()
            && !known_failure.owner.trim().is_empty()
            && known_failure.has_expiry_or_review_policy
    });
    let action = if rejected_unreferenced {
        "reject"
    } else if hot_snapshot_review_required || known_failure_review_required {
        "review"
    } else {
        "accept"
    };

    OmenaTestkitSnapshotGovernanceVerdictV0 {
        snapshot_id: seed.snapshot_id,
        fixture_id: seed.fixture_id,
        rejected_unreferenced,
        hot_snapshot_review_required,
        known_failure_review_required,
        known_failure_policy_complete,
        action,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_governance_rejects_unreferenced_snapshots() {
        let report = summarize_omena_testkit_snapshot_governance_report();

        assert_eq!(report.product, "omena-testkit.snapshot-governance");
        assert_eq!(report.fixture_grammar, "omena-fixture-v0");
        assert!(report.global_disable_rejected);
        assert!(report.policy.update_requires_review);
        assert!(report.unreferenced_reject_ready);
        assert_eq!(report.rejected_unreferenced_count, 1);
        assert!(
            report
                .verdicts
                .iter()
                .any(|verdict| verdict.snapshot_id == "orphan-style-facts"
                    && verdict.rejected_unreferenced
                    && verdict.action == "reject")
        );
    }

    #[test]
    fn snapshot_governance_audits_hot_snapshots_and_known_failures() {
        let report = summarize_omena_testkit_snapshot_governance_report();

        assert!(report.hot_snapshot_age_audit_ready);
        assert_eq!(report.hot_snapshot_review_required_count, 1);
        assert!(report.known_failure_policy_ready);
        assert_eq!(report.known_failure_count, 1);
        assert_eq!(report.known_failure_review_required_count, 1);
        assert!(report.verdicts.iter().any(|verdict| verdict.snapshot_id
            == "known-failure-source-rename"
            && verdict.known_failure_review_required
            && verdict.known_failure_policy_complete
            && verdict.action == "review"));
    }
}
