use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    sync::{Arc, Mutex},
};

use omena_reactive::{
    ChangePolicyV0, ReactiveEngineV0, ReactiveGraphBuilderV0, ReactiveNodeIdV0, ReactiveStateV0,
    ReactiveValueV0, StabilizeStatusV0,
};

const STABILIZATION_RECOMPUTE_LIMIT: usize = 64;
const DELIVERY_EFFECT_CHANNEL: &str = "lspDiagnosticsDeliveryDecision";
pub const REACTIVE_SHADOW_ENV: &str = "OMENA_LSP_REACTIVE_SHADOW";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReactiveShadowPublishTierV0 {
    Baseline,
    Optimizing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReactiveShadowStampsV0 {
    pub(crate) corpus_revision: u64,
    pub(crate) style_snapshot_revision: u64,
    pub(crate) demand_generation: u64,
}

impl ReactiveShadowStampsV0 {
    fn as_state(self) -> ReactiveStateV0 {
        ReactiveStateV0::available(ReactiveValueV0::Tuple(vec![
            ReactiveValueV0::Counter(self.corpus_revision),
            ReactiveValueV0::Counter(self.style_snapshot_revision),
            ReactiveValueV0::Counter(self.demand_generation),
        ]))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReactiveShadowDeliveryDecisionV0 {
    pub(crate) candidate_id: u64,
    pub(crate) uri: String,
    pub(crate) tier: Option<ReactiveShadowPublishTierV0>,
    pub(crate) should_deliver: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReactiveShadowFlushReportV0 {
    pub(crate) flush_id: u64,
    pub(crate) expected_target_uris: BTreeSet<String>,
    pub(crate) projected_target_uris: BTreeSet<String>,
    pub(crate) expected_stamps: ReactiveShadowStampsV0,
    pub(crate) projected_stamps: Option<ReactiveShadowStampsV0>,
    pub(crate) expected_baseline_digests: BTreeMap<String, String>,
    pub(crate) projected_baseline_digests: BTreeMap<String, String>,
    pub(crate) expected_optimizing_digests: BTreeMap<String, String>,
    pub(crate) projected_optimizing_digests: BTreeMap<String, String>,
    pub(crate) expected_delivery_decisions: Vec<ReactiveShadowDeliveryDecisionV0>,
    pub(crate) projected_delivery_decisions: Vec<ReactiveShadowDeliveryDecisionV0>,
    pub(crate) delta_fold_matches_full_rebuild: bool,
    pub(crate) settled_without_pending_work: bool,
}

impl ReactiveShadowFlushReportV0 {
    fn new(flush_id: u64, stamps: ReactiveShadowStampsV0) -> Self {
        Self {
            flush_id,
            expected_target_uris: BTreeSet::new(),
            projected_target_uris: BTreeSet::new(),
            expected_stamps: stamps,
            projected_stamps: None,
            expected_baseline_digests: BTreeMap::new(),
            projected_baseline_digests: BTreeMap::new(),
            expected_optimizing_digests: BTreeMap::new(),
            projected_optimizing_digests: BTreeMap::new(),
            expected_delivery_decisions: Vec::new(),
            projected_delivery_decisions: Vec::new(),
            delta_fold_matches_full_rebuild: false,
            settled_without_pending_work: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReactiveShadowPublishCandidateV0 {
    candidate_id: u64,
    flush_id: u64,
    uri: String,
    tier: Option<ReactiveShadowPublishTierV0>,
}

#[derive(Clone)]
pub(crate) struct ReactiveShadowPublishReceiptV0 {
    observer: ReactiveShadowObserverV0,
    candidate: ReactiveShadowPublishCandidateV0,
}

impl fmt::Debug for ReactiveShadowPublishReceiptV0 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ReactiveShadowPublishReceiptV0")
            .field("candidate", &self.candidate)
            .finish_non_exhaustive()
    }
}

impl ReactiveShadowPublishReceiptV0 {
    pub(crate) fn record_delivery_decision(&self, should_deliver: bool) {
        self.observer
            .record_delivery_decision(&self.candidate, should_deliver);
    }
}

#[derive(Clone)]
pub(crate) struct ReactiveShadowObserverV0 {
    inner: Arc<Mutex<ReactiveShadowDriverV0>>,
}

impl fmt::Debug for ReactiveShadowObserverV0 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ReactiveShadowObserverV0")
            .field("process_local", &true)
            .finish_non_exhaustive()
    }
}

impl ReactiveShadowObserverV0 {
    pub(crate) fn new() -> Result<Self, String> {
        Ok(Self {
            inner: Arc::new(Mutex::new(ReactiveShadowDriverV0::new()?)),
        })
    }

    pub(crate) fn begin_flush(&self, stamps: ReactiveShadowStampsV0) -> Option<u64> {
        self.with_driver(|driver| driver.begin_flush(stamps))
            .flatten()
    }

    pub(crate) fn current_flush_id(&self) -> Option<u64> {
        self.with_driver(|driver| driver.active_flush_id).flatten()
    }

    pub(crate) fn record_tier_digest(
        &self,
        flush_id: Option<u64>,
        uri: &str,
        tier: ReactiveShadowPublishTierV0,
        digest: &str,
    ) -> Option<ReactiveShadowPublishReceiptV0> {
        self.with_driver(|driver| driver.record_tier_digest(flush_id?, uri, tier, digest))
            .flatten()
            .map(|candidate| ReactiveShadowPublishReceiptV0 {
                observer: self.clone(),
                candidate,
            })
    }

    pub(crate) fn record_clear(
        &self,
        flush_id: Option<u64>,
        uri: &str,
    ) -> Option<ReactiveShadowPublishReceiptV0> {
        self.with_driver(|driver| driver.record_clear(flush_id?, uri))
            .flatten()
            .map(|candidate| ReactiveShadowPublishReceiptV0 {
                observer: self.clone(),
                candidate,
            })
    }

    pub(crate) fn complete_flush(&self, flush_id: u64, target_uris: BTreeSet<String>) {
        let _ = self.with_driver(|driver| driver.complete_flush(flush_id, target_uris));
    }

    #[cfg(any(test, feature = "test-support"))]
    pub(crate) fn reports(&self) -> Vec<ReactiveShadowFlushReportV0> {
        self.with_driver(|driver| driver.flushes.values().cloned().collect())
            .unwrap_or_default()
    }

    #[cfg(any(test, feature = "test-support"))]
    pub(crate) fn failures(&self) -> Vec<String> {
        self.with_driver(|driver| driver.failures.clone())
            .unwrap_or_else(|| vec!["reactive shadow observer lock was poisoned".to_string()])
    }

    fn record_delivery_decision(
        &self,
        candidate: &ReactiveShadowPublishCandidateV0,
        should_deliver: bool,
    ) {
        let _ = self.with_driver(|driver| {
            driver.record_delivery_decision(candidate, should_deliver);
        });
    }

    fn with_driver<T>(
        &self,
        operation: impl FnOnce(&mut ReactiveShadowDriverV0) -> T,
    ) -> Option<T> {
        let mut driver = self.inner.lock().ok()?;
        Some(operation(&mut driver))
    }
}

struct ReactiveShadowDriverV0 {
    engine: ReactiveEngineV0,
    target_set_input: ReactiveNodeIdV0,
    target_set_projection: ReactiveNodeIdV0,
    stamp_input: ReactiveNodeIdV0,
    baseline_digest_input: ReactiveNodeIdV0,
    baseline_digest_projection: ReactiveNodeIdV0,
    optimizing_digest_input: ReactiveNodeIdV0,
    optimizing_digest_projection: ReactiveNodeIdV0,
    digest_fold: ReactiveNodeIdV0,
    delivery_decision_input: ReactiveNodeIdV0,
    active_flush_id: Option<u64>,
    next_flush_id: u64,
    next_candidate_id: u64,
    baseline_digests: BTreeMap<String, String>,
    optimizing_digests: BTreeMap<String, String>,
    flushes: BTreeMap<u64, ReactiveShadowFlushReportV0>,
    failures: Vec<String>,
}

impl ReactiveShadowDriverV0 {
    fn new() -> Result<Self, String> {
        let mut graph = ReactiveGraphBuilderV0::new();
        let target_set_input = graph.add_input(
            string_set_state(BTreeSet::new()),
            ChangePolicyV0::exact("affectedTargetSetDeposit"),
        );
        let target_set_projection = graph.add_map(
            target_set_input,
            clone_state,
            ChangePolicyV0::exact("affectedTargetSetProjection"),
        );
        let stamp_input = graph.add_input(
            ReactiveShadowStampsV0 {
                corpus_revision: 0,
                style_snapshot_revision: 0,
                demand_generation: 0,
            }
            .as_state(),
            ChangePolicyV0::exact("snapshotGenerationDeposit"),
        );
        let baseline_digest_input = graph.add_input(
            text_map_state(BTreeMap::new()),
            ChangePolicyV0::exact("baselineDigestDeposit"),
        );
        let baseline_digest_projection = graph.add_map(
            baseline_digest_input,
            clone_state,
            ChangePolicyV0::exact("baselineDigestProjection"),
        );
        let optimizing_digest_input = graph.add_input(
            text_map_state(BTreeMap::new()),
            ChangePolicyV0::exact("optimizingDigestDeposit"),
        );
        let optimizing_digest_projection = graph.add_map(
            optimizing_digest_input,
            clone_state,
            ChangePolicyV0::exact("optimizingDigestProjection"),
        );
        let digest_fold = graph
            .add_delta_fold(
                vec![
                    ("baseline".to_string(), baseline_digest_projection),
                    ("optimizing".to_string(), optimizing_digest_projection),
                ],
                ChangePolicyV0::exact("tierDigestFold"),
            )
            .map_err(|error| error.to_string())?;
        let delivery_decision_input = graph.add_input(
            delivery_state(0, false),
            ChangePolicyV0::exact("deliveryDecisionDeposit"),
        );
        let delivery_effect = graph.add_effect_boundary(
            delivery_decision_input,
            DELIVERY_EFFECT_CHANNEL,
            ChangePolicyV0::exact("deliveryDecisionReceipt"),
        );
        let mut engine = graph.build().map_err(|error| error.to_string())?;
        for node in [
            target_set_projection,
            stamp_input,
            digest_fold,
            delivery_effect,
        ] {
            engine.observe(node).map_err(|error| error.to_string())?;
        }
        let _ = engine
            .stabilize_until_settled(STABILIZATION_RECOMPUTE_LIMIT)
            .map_err(|error| error.to_string())?;
        let _ = engine.drain_effect_receipts();

        Ok(Self {
            engine,
            target_set_input,
            target_set_projection,
            stamp_input,
            baseline_digest_input,
            baseline_digest_projection,
            optimizing_digest_input,
            optimizing_digest_projection,
            digest_fold,
            delivery_decision_input,
            active_flush_id: None,
            next_flush_id: 0,
            next_candidate_id: 0,
            baseline_digests: BTreeMap::new(),
            optimizing_digests: BTreeMap::new(),
            flushes: BTreeMap::new(),
            failures: Vec::new(),
        })
    }

    fn begin_flush(&mut self, stamps: ReactiveShadowStampsV0) -> Option<u64> {
        if let Some(active_flush_id) = self.active_flush_id {
            self.failures.push(format!(
                "flush {active_flush_id} was still active when another flush began"
            ));
            return None;
        }
        self.next_flush_id = self.next_flush_id.saturating_add(1).max(1);
        let flush_id = self.next_flush_id;
        self.flushes
            .insert(flush_id, ReactiveShadowFlushReportV0::new(flush_id, stamps));
        self.active_flush_id = Some(flush_id);
        Some(flush_id)
    }

    fn record_tier_digest(
        &mut self,
        flush_id: u64,
        uri: &str,
        tier: ReactiveShadowPublishTierV0,
        digest: &str,
    ) -> Option<ReactiveShadowPublishCandidateV0> {
        if !self.flushes.contains_key(&flush_id) {
            self.failures
                .push(format!("tier digest referenced unknown flush {flush_id}"));
            return None;
        }
        match tier {
            ReactiveShadowPublishTierV0::Baseline => {
                self.baseline_digests
                    .insert(uri.to_string(), digest.to_string());
            }
            ReactiveShadowPublishTierV0::Optimizing => {
                self.optimizing_digests
                    .insert(uri.to_string(), digest.to_string());
            }
        }
        self.next_candidate_id = self.next_candidate_id.saturating_add(1).max(1);
        let candidate = ReactiveShadowPublishCandidateV0 {
            candidate_id: self.next_candidate_id,
            flush_id,
            uri: uri.to_string(),
            tier: Some(tier),
        };
        if self.active_flush_id != Some(flush_id) {
            self.sync_digest_projection(flush_id);
        }
        Some(candidate)
    }

    fn record_clear(
        &mut self,
        flush_id: u64,
        uri: &str,
    ) -> Option<ReactiveShadowPublishCandidateV0> {
        if !self.flushes.contains_key(&flush_id) {
            self.failures
                .push(format!("clear referenced unknown flush {flush_id}"));
            return None;
        }
        self.baseline_digests.remove(uri);
        self.optimizing_digests.remove(uri);
        self.next_candidate_id = self.next_candidate_id.saturating_add(1).max(1);
        Some(ReactiveShadowPublishCandidateV0 {
            candidate_id: self.next_candidate_id,
            flush_id,
            uri: uri.to_string(),
            tier: None,
        })
    }

    fn complete_flush(&mut self, flush_id: u64, target_uris: BTreeSet<String>) {
        if self.active_flush_id != Some(flush_id) {
            self.failures
                .push(format!("flush {flush_id} completed out of order"));
            return;
        }
        let Some(expected_stamps) = self
            .flushes
            .get(&flush_id)
            .map(|flush| flush.expected_stamps)
        else {
            self.failures
                .push(format!("flush {flush_id} completed without a record"));
            self.active_flush_id = None;
            return;
        };
        self.deposit(self.target_set_input, string_set_state(target_uris.clone()));
        self.deposit(self.stamp_input, expected_stamps.as_state());
        self.deposit(
            self.baseline_digest_input,
            text_map_state(self.baseline_digests.clone()),
        );
        self.deposit(
            self.optimizing_digest_input,
            text_map_state(self.optimizing_digests.clone()),
        );
        self.settle();

        let projected_target_uris =
            string_set_from_state(self.engine.state(self.target_set_projection).ok());
        let projected_stamps = stamps_from_state(self.engine.state(self.stamp_input).ok());
        let projected_baseline_digests =
            text_map_from_state(self.engine.state(self.baseline_digest_projection).ok());
        let projected_optimizing_digests =
            text_map_from_state(self.engine.state(self.optimizing_digest_projection).ok());
        let delta_fold_matches_full_rebuild =
            self.engine.verify_delta_fold(self.digest_fold).is_ok();
        let settled_without_pending_work = !self.engine.has_pending_work();

        if let Some(flush) = self.flushes.get_mut(&flush_id) {
            flush.expected_target_uris = target_uris;
            flush.projected_target_uris = projected_target_uris;
            flush.projected_stamps = projected_stamps;
            flush.expected_baseline_digests = self.baseline_digests.clone();
            flush.projected_baseline_digests = projected_baseline_digests;
            flush.expected_optimizing_digests = self.optimizing_digests.clone();
            flush.projected_optimizing_digests = projected_optimizing_digests;
            flush.delta_fold_matches_full_rebuild = delta_fold_matches_full_rebuild;
            flush.settled_without_pending_work = settled_without_pending_work;
        }
        self.active_flush_id = None;
    }

    fn record_delivery_decision(
        &mut self,
        candidate: &ReactiveShadowPublishCandidateV0,
        should_deliver: bool,
    ) {
        let decision = ReactiveShadowDeliveryDecisionV0 {
            candidate_id: candidate.candidate_id,
            uri: candidate.uri.clone(),
            tier: candidate.tier,
            should_deliver,
        };
        let Some(flush) = self.flushes.get_mut(&candidate.flush_id) else {
            self.failures.push(format!(
                "delivery decision referenced unknown flush {}",
                candidate.flush_id
            ));
            return;
        };
        if flush
            .expected_delivery_decisions
            .iter()
            .any(|existing| existing.candidate_id == candidate.candidate_id)
        {
            return;
        }
        flush.expected_delivery_decisions.push(decision.clone());
        self.deposit(
            self.delivery_decision_input,
            delivery_state(candidate.candidate_id, should_deliver),
        );
        self.settle();
        let receipts = self.engine.drain_effect_receipts();
        let observed = receipts.iter().any(|receipt| {
            receipt.channel == DELIVERY_EFFECT_CHANNEL
                && receipt.state == delivery_state(candidate.candidate_id, should_deliver)
        });
        if observed {
            if let Some(flush) = self.flushes.get_mut(&candidate.flush_id) {
                flush.projected_delivery_decisions.push(decision);
                flush.settled_without_pending_work = !self.engine.has_pending_work();
            }
        } else {
            self.failures.push(format!(
                "delivery decision {} produced no effect receipt",
                candidate.candidate_id
            ));
        }
    }

    fn sync_digest_projection(&mut self, flush_id: u64) {
        self.deposit(
            self.baseline_digest_input,
            text_map_state(self.baseline_digests.clone()),
        );
        self.deposit(
            self.optimizing_digest_input,
            text_map_state(self.optimizing_digests.clone()),
        );
        self.settle();
        let projected_baseline_digests =
            text_map_from_state(self.engine.state(self.baseline_digest_projection).ok());
        let projected_optimizing_digests =
            text_map_from_state(self.engine.state(self.optimizing_digest_projection).ok());
        let delta_fold_matches_full_rebuild =
            self.engine.verify_delta_fold(self.digest_fold).is_ok();
        if let Some(flush) = self.flushes.get_mut(&flush_id) {
            flush.expected_baseline_digests = self.baseline_digests.clone();
            flush.projected_baseline_digests = projected_baseline_digests;
            flush.expected_optimizing_digests = self.optimizing_digests.clone();
            flush.projected_optimizing_digests = projected_optimizing_digests;
            flush.delta_fold_matches_full_rebuild = delta_fold_matches_full_rebuild;
            flush.settled_without_pending_work = !self.engine.has_pending_work();
        }
    }

    fn deposit(&mut self, node: ReactiveNodeIdV0, state: ReactiveStateV0) {
        if let Err(error) = self.engine.deposit(node, state) {
            self.failures.push(error.to_string());
        }
    }

    fn settle(&mut self) {
        match self
            .engine
            .stabilize_until_settled(STABILIZATION_RECOMPUTE_LIMIT)
        {
            Ok(StabilizeStatusV0::Settled { .. }) => {}
            Ok(StabilizeStatusV0::Pending { .. }) => self
                .failures
                .push("reactive shadow exceeded its bounded stabilization budget".to_string()),
            Err(error) => self.failures.push(error.to_string()),
        }
    }
}

impl crate::LspShellState {
    /// Enables the process-local read-only observer. The stdio server calls
    /// this only when [`REACTIVE_SHADOW_ENV`] is explicitly set to `1`.
    pub fn enable_reactive_shadow_observer(&mut self) -> Result<(), String> {
        self.diagnostics_publish_digest_registry
            .enable_reactive_shadow()
    }
}

fn clone_state(state: &ReactiveStateV0) -> ReactiveStateV0 {
    state.clone()
}

fn string_set_state(values: BTreeSet<String>) -> ReactiveStateV0 {
    ReactiveStateV0::available(ReactiveValueV0::StringSet(values))
}

fn text_map_state(values: BTreeMap<String, String>) -> ReactiveStateV0 {
    ReactiveStateV0::available(ReactiveValueV0::TextMap(values))
}

fn delivery_state(candidate_id: u64, should_deliver: bool) -> ReactiveStateV0 {
    ReactiveStateV0::available(ReactiveValueV0::Tuple(vec![
        ReactiveValueV0::Counter(candidate_id),
        ReactiveValueV0::Bool(should_deliver),
    ]))
}

fn string_set_from_state(state: Option<&ReactiveStateV0>) -> BTreeSet<String> {
    match state {
        Some(ReactiveStateV0::Available(ReactiveValueV0::StringSet(values))) => values.clone(),
        _ => BTreeSet::new(),
    }
}

fn text_map_from_state(state: Option<&ReactiveStateV0>) -> BTreeMap<String, String> {
    match state {
        Some(ReactiveStateV0::Available(ReactiveValueV0::TextMap(values))) => values.clone(),
        _ => BTreeMap::new(),
    }
}

fn stamps_from_state(state: Option<&ReactiveStateV0>) -> Option<ReactiveShadowStampsV0> {
    let Some(ReactiveStateV0::Available(ReactiveValueV0::Tuple(values))) = state else {
        return None;
    };
    let [
        ReactiveValueV0::Counter(corpus_revision),
        ReactiveValueV0::Counter(style_snapshot_revision),
        ReactiveValueV0::Counter(demand_generation),
    ] = values.as_slice()
    else {
        return None;
    };
    Some(ReactiveShadowStampsV0 {
        corpus_revision: *corpus_revision,
        style_snapshot_revision: *style_snapshot_revision,
        demand_generation: *demand_generation,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn four_projection_arena_settles_without_external_effects()
    -> Result<(), Box<dyn std::error::Error>> {
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
        assert!(observer.failures().is_empty());
        Ok(())
    }

    #[test]
    fn observer_enabled_and_disabled_paths_emit_identical_lsp_values()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut disabled = crate::LspShellState::default();
        let mut enabled = crate::LspShellState::default();
        enabled
            .enable_reactive_shadow_observer()
            .map_err(std::io::Error::other)?;

        let messages = [
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": "file:///workspace/App.module.scss",
                        "languageId": "scss",
                        "version": 1,
                        "text": ".root { color: red; }",
                    },
                },
            }),
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didChange",
                "params": {
                    "textDocument": {
                        "uri": "file:///workspace/App.module.scss",
                        "version": 2,
                    },
                    "contentChanges": [
                        {
                            "text": ".root { color: blue; }",
                        },
                    ],
                },
            }),
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didClose",
                "params": {
                    "textDocument": {
                        "uri": "file:///workspace/App.module.scss",
                    },
                },
            }),
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
        assert_eq!(reports.len(), messages.len());
        assert!(reports.iter().all(|report| {
            report.expected_target_uris == report.projected_target_uris
                && report.projected_stamps == Some(report.expected_stamps)
                && report.expected_baseline_digests == report.projected_baseline_digests
                && report.expected_optimizing_digests == report.projected_optimizing_digests
                && report.expected_delivery_decisions == report.projected_delivery_decisions
                && report.delta_fold_matches_full_rebuild
                && report.settled_without_pending_work
        }));
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
        let mut state = crate::LspShellState::default();
        state
            .enable_reactive_shadow_observer()
            .map_err(std::io::Error::other)?;
        let turn = crate::handle_lsp_message_scheduled_outputs_or_dispatch(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": "file:///workspace/Deferred.module.scss",
                        "languageId": "scss",
                        "version": 1,
                        "text": ".root { color: red; }",
                    },
                },
            }),
        );
        let crate::LspLoopTurnV0::OutputsAndDeferredDiagnostics {
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

    fn run_message_stream(state: &mut crate::LspShellState, messages: &[Value]) -> Vec<Value> {
        messages
            .iter()
            .flat_map(|message| {
                crate::handle_lsp_message_scheduled_outputs(state, message.clone())
                    .into_iter()
                    .filter_map(crate::ScheduledLspOutput::into_delivered_value)
            })
            .collect()
    }
}
