use std::collections::BTreeSet;

use omena_reactive::{ReactiveDivergenceDispositionV0, ReactiveObservationPhaseV0};
use serde_json::{Value, json};

use crate::{
    LspLoopTurnV0, LspShellState, ScheduledLspOutput,
    reactive_shadow::{
        ReactiveShadowFlushReportV0, ReactiveShadowObserverV0, ReactiveShadowPublishTierV0,
        ReactiveShadowStampsV0,
    },
    reactive_shadow_contract::{
        ProposedAuthorityViolationV0, ReactiveShadowParityDimensionV0, divergence_disposition,
        evaluate_proposed_authority_reduction, evaluate_reactive_shadow_parity,
    },
};

type ParityProjectionMutation = (
    ReactiveShadowParityDimensionV0,
    fn(&mut ReactiveShadowFlushReportV0),
);

#[test]
fn four_projection_arena_settles_without_external_effects() -> Result<(), Box<dyn std::error::Error>>
{
    let observer = ReactiveShadowObserverV0::new().map_err(std::io::Error::other)?;
    let stamps = ReactiveShadowStampsV0 {
        corpus_revision: 7,
        style_snapshot_revision: 11,
        demand_generation: 3,
    };
    let Some(flush_id) = observer.begin_flush(stamps) else {
        return Err(std::io::Error::other("observer did not begin a flush").into());
    };
    let Some(receipt) = observer.record_tier_digest(
        Some(flush_id),
        "file:///workspace/App.module.scss",
        ReactiveShadowPublishTierV0::Baseline,
        "blake3:baseline",
    ) else {
        return Err(std::io::Error::other("observer did not issue a receipt").into());
    };
    observer.complete_flush(
        flush_id,
        BTreeSet::from(["file:///workspace/App.module.scss".to_string()]),
        stamps,
    );
    receipt.record_delivery_decision(true);

    let reports = observer.reports();
    let Some(report) = reports.first() else {
        return Err(std::io::Error::other("observer produced no report").into());
    };
    assert_eq!(report.expected_target_uris, report.projected_target_uris);
    assert_eq!(report.projected_stamps, Some(stamps));
    assert_eq!(
        report.expected_baseline_digests,
        report.projected_baseline_digests
    );
    assert_eq!(
        report.expected_optimizing_digests,
        report.projected_optimizing_digests
    );
    assert_eq!(
        report.expected_delivery_decisions,
        report.projected_delivery_decisions
    );
    assert!(report.delta_fold_matches_full_rebuild);
    assert!(report.settled_without_pending_work);
    assert!(report.observer_liveness_grounded);
    assert!(observer.failures().is_empty());
    Ok(())
}

#[test]
fn snapshot_generation_projection_uses_flush_completion_stamps()
-> Result<(), Box<dyn std::error::Error>> {
    let initial_stamps = authority_stamps();
    let final_stamps = ReactiveShadowStampsV0 {
        style_snapshot_revision: 12,
        demand_generation: 4,
        ..initial_stamps
    };
    let (observer, _, _) = observer_flush_with_final_stamps(initial_stamps, final_stamps)?;
    let reports = observer.reports();
    let Some(report) = reports.first() else {
        return Err(std::io::Error::other("observer produced no report").into());
    };
    assert_eq!(report.expected_stamps, final_stamps);
    assert_eq!(report.projected_stamps, Some(final_stamps));
    assert!(
        evaluate_proposed_authority_reduction(reports.as_slice())
            .violations
            .is_empty()
    );
    Ok(())
}

#[test]
fn observer_enabled_and_disabled_paths_emit_identical_lsp_values()
-> Result<(), Box<dyn std::error::Error>> {
    let mut disabled = LspShellState::default();
    let mut enabled = LspShellState::default();
    enabled
        .enable_reactive_shadow_observer()
        .map_err(std::io::Error::other)?;

    let messages = [
        open_message(
            "file:///workspace/App.module.scss",
            1,
            ".root { color: red; }",
        ),
        change_message(
            "file:///workspace/App.module.scss",
            2,
            ".root { color: blue; }",
        ),
        close_message("file:///workspace/App.module.scss"),
    ];

    let disabled_values = run_message_stream(&mut disabled, &messages);
    let enabled_values = run_message_stream(&mut enabled, &messages);
    assert_eq!(enabled_values, disabled_values);
    assert!(
        disabled
            .diagnostics_publish_digest_registry
            .reactive_shadow_reports_for_test()
            .is_none()
    );

    let Some(reports) = enabled
        .diagnostics_publish_digest_registry
        .reactive_shadow_reports_for_test()
    else {
        return Err(std::io::Error::other("enabled observer produced no reports").into());
    };
    let parity = evaluate_reactive_shadow_parity(reports.as_slice());
    assert_eq!(parity.flush_count, messages.len());
    assert!(parity.unclassified_divergences.is_empty());
    assert!(
        reports
            .iter()
            .flat_map(|report| report.expected_delivery_decisions.iter())
            .any(|decision| !decision.should_deliver),
        "the stream must exercise the writer-level suppression decision"
    );
    assert_eq!(
        enabled
            .diagnostics_publish_digest_registry
            .reactive_shadow_failures_for_test(),
        Some(Vec::new())
    );
    Ok(())
}

#[cfg(feature = "salsa-style-diagnostics")]
#[test]
fn deferred_digest_receipt_stays_attached_to_its_scheduler_flush()
-> Result<(), Box<dyn std::error::Error>> {
    let mut state = LspShellState::default();
    state
        .enable_reactive_shadow_observer()
        .map_err(std::io::Error::other)?;
    let turn = crate::handle_lsp_message_scheduled_outputs_or_dispatch(
        &mut state,
        open_message(
            "file:///workspace/Deferred.module.scss",
            1,
            ".root { color: red; }",
        ),
    );
    let LspLoopTurnV0::OutputsAndDeferredDiagnostics {
        deferred_diagnostics,
        ..
    } = turn
    else {
        return Err(std::io::Error::other("expected deferred diagnostics turn").into());
    };
    let Some(dispatch) = deferred_diagnostics.first() else {
        return Err(std::io::Error::other("missing deferred diagnostics dispatch").into());
    };
    let notification = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {
            "uri": dispatch.uri,
            "diagnostics": [{"code": "deferred-proof"}],
        },
    });
    let Some(receipt) = dispatch.optimizing_publish_receipt(&notification) else {
        return Err(std::io::Error::other("missing optimizing receipt").into());
    };
    assert!(receipt.should_deliver());
    receipt.record_delivered();

    let Some(reports) = state
        .diagnostics_publish_digest_registry
        .reactive_shadow_reports_for_test()
    else {
        return Err(std::io::Error::other("missing observer reports").into());
    };
    let Some(report) = reports.first() else {
        return Err(std::io::Error::other("missing scheduler flush report").into());
    };
    assert!(
        report
            .projected_optimizing_digests
            .contains_key("file:///workspace/Deferred.module.scss")
    );
    assert_eq!(
        report.expected_optimizing_digests,
        report.projected_optimizing_digests
    );
    assert_eq!(
        report.expected_delivery_decisions,
        report.projected_delivery_decisions
    );
    assert!(report.settled_without_pending_work);
    Ok(())
}

#[test]
fn seeded_event_stream_matches_all_flush_projections() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = LspShellState::default();
    state
        .enable_reactive_shadow_observer()
        .map_err(std::io::Error::other)?;
    let uris = [
        "file:///workspace/A.module.scss",
        "file:///workspace/B.module.scss",
        "file:///workspace/C.module.scss",
        "file:///workspace/D.module.scss",
    ];
    let diagnostic_text =
        ":root { --brand: red; }\n.btn { width: var(--missing); color: red; color: blue; }";
    let clean_text = ".root { color: red; }";
    let mut open = [false; 4];
    let mut versions = [0_i64; 4];
    let mut random = XorShift64(0x9e37_79b9_7f4a_7c15);

    for index in 0..uris.len() {
        versions[index] += 1;
        let message = open_message(uris[index], versions[index], diagnostic_text);
        let _ = run_message_stream(&mut state, &[message]);
        open[index] = true;
    }
    for _ in 0..128 {
        let index = random.below(uris.len() as u64) as usize;
        let action = random.below(3);
        let message = match (open[index], action) {
            (false, _) => {
                open[index] = true;
                versions[index] += 1;
                open_message(
                    uris[index],
                    versions[index],
                    if random.below(2) == 0 {
                        diagnostic_text
                    } else {
                        clean_text
                    },
                )
            }
            (true, 0) => {
                open[index] = false;
                close_message(uris[index])
            }
            (true, _) => {
                versions[index] += 1;
                change_message(
                    uris[index],
                    versions[index],
                    if random.below(2) == 0 {
                        diagnostic_text
                    } else {
                        clean_text
                    },
                )
            }
        };
        let _ = run_message_stream(&mut state, &[message]);
    }

    let Some(reports) = state
        .diagnostics_publish_digest_registry
        .reactive_shadow_reports_for_test()
    else {
        return Err(std::io::Error::other("missing random-stream reports").into());
    };
    let parity = evaluate_reactive_shadow_parity(reports.as_slice());
    assert_eq!(parity.flush_count, 132);
    assert_eq!(parity.target_set_equality_count, parity.flush_count);
    assert_eq!(
        parity.snapshot_generation_equality_count,
        parity.flush_count
    );
    assert_eq!(parity.tier_digest_equality_count, parity.flush_count);
    assert_eq!(parity.delivery_decision_equality_count, parity.flush_count);
    assert_eq!(parity.settled_without_pending_count, parity.flush_count);
    assert!(parity.baseline_digest_observation_count > 0);
    assert!(parity.optimizing_digest_observation_count > 0);
    assert!(parity.suppressed_delivery_observation_count > 0);
    assert!(parity.unclassified_divergences.is_empty());
    let authority = evaluate_proposed_authority_reduction(reports.as_slice());
    assert_eq!(authority.checked_flush_count, parity.flush_count);
    assert!(authority.violations.is_empty());
    assert_eq!(
        state
            .diagnostics_publish_digest_registry
            .reactive_shadow_failures_for_test(),
        Some(Vec::new())
    );
    Ok(())
}

#[test]
fn every_flush_equality_rejects_its_own_projection_drift() {
    let baseline = synthetic_report();
    let mutations: [ParityProjectionMutation; 5] = [
        (ReactiveShadowParityDimensionV0::TargetSet, |report| {
            report
                .projected_target_uris
                .insert("file:///drift.scss".to_string());
        }),
        (
            ReactiveShadowParityDimensionV0::SnapshotGeneration,
            |report| {
                report.projected_stamps = Some(ReactiveShadowStampsV0 {
                    corpus_revision: 99,
                    ..report.expected_stamps
                });
            },
        ),
        (ReactiveShadowParityDimensionV0::TierDigest, |report| {
            report
                .projected_baseline_digests
                .insert("file:///fixture.scss".to_string(), "wrong".to_string());
        }),
        (
            ReactiveShadowParityDimensionV0::DeliveryDecision,
            |report| report.projected_delivery_decisions.clear(),
        ),
        (ReactiveShadowParityDimensionV0::SettledWork, |report| {
            report.settled_without_pending_work = false
        }),
    ];

    for (expected_dimension, mutate) in mutations {
        let mut report = baseline.clone();
        mutate(&mut report);
        let parity = evaluate_reactive_shadow_parity(&[report]);
        assert!(
            parity
                .unclassified_divergences
                .contains(&expected_dimension)
        );
    }
}

#[test]
fn taxonomy_allows_only_reviewed_transient_timing_differences() {
    assert_eq!(
        divergence_disposition(
            "flushConeClosureTiming",
            ReactiveObservationPhaseV0::DuringWave
        ),
        ReactiveDivergenceDispositionV0::BenignUntilFlush
    );
    assert_eq!(
        divergence_disposition("flushConeClosureTiming", ReactiveObservationPhaseV0::Flush),
        ReactiveDivergenceDispositionV0::Blocker
    );
    assert_eq!(
        divergence_disposition(
            "unknownTimingDifference",
            ReactiveObservationPhaseV0::DuringWave
        ),
        ReactiveDivergenceDispositionV0::Blocker
    );
}

#[test]
fn proposed_authority_reduction_accepts_clean_observation() -> Result<(), Box<dyn std::error::Error>>
{
    let (clean_observer, clean_flush, _) = observer_flush(authority_stamps())?;
    let clean_reports = clean_observer.reports();
    let clean = evaluate_proposed_authority_reduction(clean_reports.as_slice());
    assert_eq!(clean.checked_flush_count, 1);
    assert!(clean.violations.is_empty());
    assert_eq!(clean_flush, 1);
    Ok(())
}

#[test]
fn proposed_authority_reduction_rejects_torn_corpus_revision()
-> Result<(), Box<dyn std::error::Error>> {
    let initial_stamps = authority_stamps();
    let final_stamps = ReactiveShadowStampsV0 {
        corpus_revision: 8,
        ..initial_stamps
    };
    let (torn_observer, _, _) = observer_flush_with_final_stamps(initial_stamps, final_stamps)?;
    assert_eq!(
        evaluate_proposed_authority_reduction(torn_observer.reports().as_slice()).violations,
        vec![ProposedAuthorityViolationV0::CorpusRevision]
    );
    Ok(())
}

#[test]
fn proposed_authority_reduction_rejects_snapshot_read_side_effect()
-> Result<(), Box<dyn std::error::Error>> {
    let (snapshot_observer, snapshot_flush, _) = observer_flush(authority_stamps())?;
    snapshot_observer.inject_snapshot_read_side_effect_for_test(snapshot_flush);
    assert_eq!(
        evaluate_proposed_authority_reduction(snapshot_observer.reports().as_slice()).violations,
        vec![ProposedAuthorityViolationV0::ImmutableReadSnapshot]
    );
    Ok(())
}

#[test]
fn proposed_authority_reduction_rejects_stale_live_demand() -> Result<(), Box<dyn std::error::Error>>
{
    let (stale_observer, stale_flush, stale_receipt) = observer_flush(authority_stamps())?;
    let next_stamps = ReactiveShadowStampsV0 {
        corpus_revision: 8,
        style_snapshot_revision: 12,
        demand_generation: 4,
    };
    let Some(next_flush) = stale_observer.begin_flush(next_stamps) else {
        return Err(std::io::Error::other("observer did not begin replacement flush").into());
    };
    stale_observer.complete_flush(
        next_flush,
        BTreeSet::from(["file:///fixture.scss".to_string()]),
        next_stamps,
    );
    stale_receipt.record_delivery_decision(true);
    let stale_reports = stale_observer.reports();
    let Some(stale_report) = stale_reports
        .iter()
        .find(|report| report.flush_id == stale_flush)
    else {
        return Err(std::io::Error::other("missing stale flush report").into());
    };
    assert_eq!(
        evaluate_proposed_authority_reduction(std::slice::from_ref(stale_report)).violations,
        vec![ProposedAuthorityViolationV0::DemandLedger]
    );
    Ok(())
}

fn authority_stamps() -> ReactiveShadowStampsV0 {
    ReactiveShadowStampsV0 {
        corpus_revision: 7,
        style_snapshot_revision: 11,
        demand_generation: 3,
    }
}

fn observer_flush(
    stamps: ReactiveShadowStampsV0,
) -> Result<
    (
        ReactiveShadowObserverV0,
        u64,
        crate::reactive_shadow::ReactiveShadowPublishReceiptV0,
    ),
    Box<dyn std::error::Error>,
> {
    observer_flush_with_final_stamps(stamps, stamps)
}

fn observer_flush_with_final_stamps(
    initial_stamps: ReactiveShadowStampsV0,
    final_stamps: ReactiveShadowStampsV0,
) -> Result<
    (
        ReactiveShadowObserverV0,
        u64,
        crate::reactive_shadow::ReactiveShadowPublishReceiptV0,
    ),
    Box<dyn std::error::Error>,
> {
    let observer = ReactiveShadowObserverV0::new().map_err(std::io::Error::other)?;
    let Some(flush_id) = observer.begin_flush(initial_stamps) else {
        return Err(std::io::Error::other("observer did not begin fixture flush").into());
    };
    let Some(receipt) = observer.record_tier_digest(
        Some(flush_id),
        "file:///fixture.scss",
        ReactiveShadowPublishTierV0::Baseline,
        "digest",
    ) else {
        return Err(std::io::Error::other("observer did not issue fixture receipt").into());
    };
    observer.complete_flush(
        flush_id,
        BTreeSet::from(["file:///fixture.scss".to_string()]),
        final_stamps,
    );
    Ok((observer, flush_id, receipt))
}

fn synthetic_report() -> ReactiveShadowFlushReportV0 {
    let stamps = ReactiveShadowStampsV0 {
        corpus_revision: 7,
        style_snapshot_revision: 11,
        demand_generation: 3,
    };
    let mut report = ReactiveShadowFlushReportV0::new(1, stamps);
    report.expected_target_uris = BTreeSet::from(["file:///fixture.scss".to_string()]);
    report.projected_target_uris = report.expected_target_uris.clone();
    report.projected_stamps = Some(stamps);
    report
        .expected_baseline_digests
        .insert("file:///fixture.scss".to_string(), "digest".to_string());
    report.projected_baseline_digests = report.expected_baseline_digests.clone();
    report.projected_optimizing_digests = report.expected_optimizing_digests.clone();
    report.expected_delivery_decisions =
        vec![crate::reactive_shadow::ReactiveShadowDeliveryDecisionV0 {
            candidate_id: 1,
            uri: "file:///fixture.scss".to_string(),
            tier: Some(ReactiveShadowPublishTierV0::Baseline),
            should_deliver: true,
        }];
    report.projected_delivery_decisions = report.expected_delivery_decisions.clone();
    report.delta_fold_matches_full_rebuild = true;
    report.settled_without_pending_work = true;
    report.corpus_revision_reads.push(stamps.corpus_revision);
    report.observer_liveness_grounded = true;
    report
}

fn run_message_stream(state: &mut LspShellState, messages: &[Value]) -> Vec<Value> {
    messages
        .iter()
        .flat_map(|message| {
            crate::handle_lsp_message_scheduled_outputs(state, message.clone())
                .into_iter()
                .filter_map(ScheduledLspOutput::into_delivered_value)
        })
        .collect()
}

fn open_message(uri: &str, version: i64, text: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": uri,
                "languageId": "scss",
                "version": version,
                "text": text,
            },
        },
    })
}

fn change_message(uri: &str, version: i64, text: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didChange",
        "params": {
            "textDocument": {
                "uri": uri,
                "version": version,
            },
            "contentChanges": [{"text": text}],
        },
    })
}

fn close_message(uri: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didClose",
        "params": {
            "textDocument": {
                "uri": uri,
            },
        },
    })
}

struct XorShift64(u64);

impl XorShift64 {
    fn next(&mut self) -> u64 {
        let mut value = self.0;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.0 = value;
        value
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
}
