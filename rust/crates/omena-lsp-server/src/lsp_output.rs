use crate::{LspQuerySnapshotV0, LspStyleHoverCandidate};
use omena_query::{
    OmenaQueryExternalSifInputV0, OmenaQuerySourceDocumentInputV0,
    OmenaQuerySourceMissingSelectorDiagnosticCandidateV0, OmenaQuerySourceSyntaxIndexV0,
    OmenaQueryStyleHoverCandidateV0, OmenaQueryStylePackageManifestV0,
    OmenaQueryStyleResolutionInputsV0, OmenaQueryStyleSelectorDefinitionV0,
    OmenaQueryStyleSourceInputV0,
};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

pub const OPTIMIZING_DIAGNOSTICS_DELAY_MS: u64 = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticsPublishTierV0 {
    Baseline,
    Optimizing,
}

#[derive(Debug, Clone, Default)]
pub struct DiagnosticsPublishDigestRegistryV0 {
    delivered: Arc<Mutex<DiagnosticsPublishDigestStateV0>>,
}

#[derive(Debug, Default)]
struct DiagnosticsPublishDigestStateV0 {
    by_tier: BTreeMap<(String, DiagnosticsPublishTierV0), omena_sif::OmenaSifDigestV1>,
    // A tier digest can repeat after another tier became client-visible. The
    // terminal payload must still restore that visible state.
    current_by_uri: BTreeMap<String, omena_sif::OmenaSifDigestV1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DiagnosticsPublishReceiptActionV0 {
    Tier {
        uri: String,
        tier: DiagnosticsPublishTierV0,
        digest: omena_sif::OmenaSifDigestV1,
        terminal_for_revision: bool,
    },
    ClearDocument {
        uri: String,
    },
}

#[derive(Debug, Clone)]
pub struct DiagnosticsPublishReceiptV0 {
    registry: DiagnosticsPublishDigestRegistryV0,
    action: DiagnosticsPublishReceiptActionV0,
}

impl PartialEq for DiagnosticsPublishReceiptV0 {
    fn eq(&self, other: &Self) -> bool {
        self.action == other.action
    }
}

impl Eq for DiagnosticsPublishReceiptV0 {}

impl DiagnosticsPublishDigestRegistryV0 {
    pub fn tier_receipt(
        &self,
        uri: &str,
        tier: DiagnosticsPublishTierV0,
        diagnostics: &Value,
        terminal_for_revision: bool,
    ) -> Option<DiagnosticsPublishReceiptV0> {
        let bytes = serde_json::to_vec(diagnostics).ok()?;
        Some(DiagnosticsPublishReceiptV0 {
            registry: self.clone(),
            action: DiagnosticsPublishReceiptActionV0::Tier {
                uri: uri.to_string(),
                tier,
                digest: omena_sif::compute_omena_sif_leaf_hash_v1(bytes.as_slice()),
                terminal_for_revision,
            },
        })
    }

    pub fn clear_document_receipt(&self, uri: &str) -> DiagnosticsPublishReceiptV0 {
        DiagnosticsPublishReceiptV0 {
            registry: self.clone(),
            action: DiagnosticsPublishReceiptActionV0::ClearDocument {
                uri: uri.to_string(),
            },
        }
    }
}

impl DiagnosticsPublishReceiptV0 {
    pub fn should_deliver(&self) -> bool {
        match &self.action {
            DiagnosticsPublishReceiptActionV0::Tier {
                uri,
                tier,
                digest,
                terminal_for_revision,
            } => {
                let delivered = self
                    .registry
                    .delivered
                    .lock()
                    .unwrap_or_else(|error| error.into_inner());
                if delivered.by_tier.get(&(uri.clone(), *tier)) != Some(digest) {
                    return true;
                }
                // An unchanged non-terminal baseline can be omitted because
                // the following optimizing tier owns the final revision
                // state. A terminal tier may be omitted only when the client
                // already holds the same bytes.
                *terminal_for_revision && delivered.current_by_uri.get(uri) != Some(digest)
            }
            DiagnosticsPublishReceiptActionV0::ClearDocument { .. } => true,
        }
    }

    pub fn record_delivered(&self) {
        let mut delivered = self
            .registry
            .delivered
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        match &self.action {
            DiagnosticsPublishReceiptActionV0::Tier {
                uri, tier, digest, ..
            } => {
                delivered
                    .by_tier
                    .insert((uri.clone(), *tier), digest.clone());
                delivered.current_by_uri.insert(uri.clone(), digest.clone());
            }
            DiagnosticsPublishReceiptActionV0::ClearDocument { uri } => {
                delivered
                    .by_tier
                    .retain(|(delivered_uri, _), _| delivered_uri != uri);
                delivered.current_by_uri.remove(uri);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledLspOutput {
    pub value: Value,
    pub delay_millis: Option<u64>,
    pub coalesce_key: Option<String>,
    pub diagnostics_publish_receipt: Option<DiagnosticsPublishReceiptV0>,
}

impl ScheduledLspOutput {
    pub fn immediate(value: Value) -> Self {
        Self {
            value,
            delay_millis: None,
            coalesce_key: None,
            diagnostics_publish_receipt: None,
        }
    }

    pub fn immediate_coalesced(value: Value, coalesce_key: String) -> Self {
        Self {
            value,
            delay_millis: None,
            coalesce_key: Some(coalesce_key),
            diagnostics_publish_receipt: None,
        }
    }

    pub fn delayed(value: Value, delay_millis: u64) -> Self {
        Self {
            value,
            delay_millis: Some(delay_millis),
            coalesce_key: None,
            diagnostics_publish_receipt: None,
        }
    }

    pub fn delayed_coalesced(value: Value, delay_millis: u64, coalesce_key: String) -> Self {
        Self {
            value,
            delay_millis: Some(delay_millis),
            coalesce_key: Some(coalesce_key),
            diagnostics_publish_receipt: None,
        }
    }

    pub fn with_diagnostics_publish_receipt(
        mut self,
        receipt: DiagnosticsPublishReceiptV0,
    ) -> Self {
        self.diagnostics_publish_receipt = Some(receipt);
        self
    }

    pub fn into_value(self) -> Value {
        self.value
    }

    pub fn into_delivered_value(self) -> Option<Value> {
        if let Some(receipt) = self.diagnostics_publish_receipt.as_ref() {
            if !receipt.should_deliver() {
                return None;
            }
            receipt.record_delivered();
        }
        Some(self.value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticsPipelineTierPlanV0 {
    pub baseline_evidence: &'static str,
    pub optimizing_evidence: &'static str,
    pub baseline_feedback_evidence: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct LspOwnedStyleDiagnosticsRenderInputsV0 {
    pub document_uri: String,
    pub document_text: String,
    pub query_candidates: Vec<OmenaQueryStyleHoverCandidateV0>,
    pub snapshot_id: Option<omena_query::OmenaWorkspaceSnapshotIdV0>,
    pub style_sources: Vec<OmenaQueryStyleSourceInputV0>,
    pub source_documents: Vec<OmenaQuerySourceDocumentInputV0>,
    pub package_manifests: Vec<OmenaQueryStylePackageManifestV0>,
    pub external_sifs: Vec<OmenaQueryExternalSifInputV0>,
    pub resolution_inputs: OmenaQueryStyleResolutionInputsV0,
    pub deep_analysis: bool,
    pub configured_severity: u8,
}

#[derive(Debug, Clone)]
pub struct LspOwnedSourceDiagnosticsRenderInputsV0 {
    pub document_uri: String,
    pub document_text: String,
    pub source_syntax_index: OmenaQuerySourceSyntaxIndexV0,
    pub source_selector_candidates: Vec<LspStyleHoverCandidate>,
    pub style_sources: Vec<OmenaQueryStyleSourceInputV0>,
    pub query_definitions: Vec<OmenaQueryStyleSelectorDefinitionV0>,
    pub source_selector_fallback_candidates:
        Vec<OmenaQuerySourceMissingSelectorDiagnosticCandidateV0>,
    pub global_class_fallthroughs: Vec<LspGlobalClassFallthroughCandidateV0>,
    pub configured_severity: u8,
}

/// A reference that failed the bound module's export set but resolved in
/// the GLOBAL class universe (tier two): rendered as the
/// `globalClassFallthrough` disclosure instead of a missing-selector
/// warning. Property accesses never produce one — they have no runtime
/// fall-through and stay strict.
#[derive(Debug, Clone)]
pub struct LspGlobalClassFallthroughCandidateV0 {
    pub selector_name: String,
    pub global_definition_uri: String,
    pub target_style_uri: String,
    pub target_style_source: String,
    pub source_reference_range: omena_query::ParserRangeV0,
}

#[derive(Debug)]
pub enum DeferredDiagnosticsRenderInputsV0 {
    StyleSnapshot(Box<LspQuerySnapshotV0>),
    Source(Box<LspOwnedSourceDiagnosticsRenderInputsV0>),
}

#[derive(Debug)]
pub struct LspDeferredDiagnosticsDispatchV0 {
    pub uri: String,
    pub coalesce_key: String,
    pub tier_plan: DiagnosticsPipelineTierPlanV0,
    pub diagnostics_publish_registry: DiagnosticsPublishDigestRegistryV0,
    pub workspace_snapshot_id: Option<omena_query::OmenaWorkspaceSnapshotIdV0>,
    pub render_inputs: DeferredDiagnosticsRenderInputsV0,
    /// Tide-ledger epoch at dispatch time: the reverse-dependency refresh
    /// this compute produces is stamped with it, so edits racing the
    /// worker keep the memo honestly stale.
    pub ledger_epoch: u64,
}

impl LspDeferredDiagnosticsDispatchV0 {
    pub fn optimizing_publish_receipt(
        &self,
        notification: &Value,
    ) -> Option<DiagnosticsPublishReceiptV0> {
        self.diagnostics_publish_registry.tier_receipt(
            self.uri.as_str(),
            DiagnosticsPublishTierV0::Optimizing,
            notification.pointer("/params/diagnostics")?,
            true,
        )
    }
}

/// A reverse-dependency memo refresh produced as a BYPRODUCT of an
/// off-loop selector build: the loop applies it from the completion
/// channel instead of ever building a selector itself.
#[derive(Debug, Clone)]
pub struct LspReverseDependencyRefreshV0 {
    pub revision: u64,
    pub ledger_epoch: u64,
    pub summary: omena_query::OmenaQueryCrossFileSummaryV0,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn deliver(
        registry: &DiagnosticsPublishDigestRegistryV0,
        uri: &str,
        tier: DiagnosticsPublishTierV0,
        diagnostics: &Value,
        terminal_for_revision: bool,
    ) -> bool {
        let Some(receipt) = registry.tier_receipt(uri, tier, diagnostics, terminal_for_revision)
        else {
            return false;
        };
        if !receipt.should_deliver() {
            return false;
        }
        receipt.record_delivered();
        true
    }

    #[test]
    fn identical_tier_payloads_are_suppressed_without_losing_a_changed_final_tier() {
        let registry = DiagnosticsPublishDigestRegistryV0::default();
        let uri = "file:///workspace/App.module.scss";
        let baseline = json!([{"code": "baseline-a"}]);
        let changed_baseline = json!([{"code": "baseline-c"}]);
        let optimizing = json!([{"code": "optimizing-b"}]);

        assert!(deliver(
            &registry,
            uri,
            DiagnosticsPublishTierV0::Baseline,
            &baseline,
            false,
        ));
        assert!(deliver(
            &registry,
            uri,
            DiagnosticsPublishTierV0::Optimizing,
            &optimizing,
            true,
        ));

        assert!(!deliver(
            &registry,
            uri,
            DiagnosticsPublishTierV0::Baseline,
            &baseline,
            false,
        ));
        assert!(!deliver(
            &registry,
            uri,
            DiagnosticsPublishTierV0::Optimizing,
            &optimizing,
            true,
        ));

        assert!(deliver(
            &registry,
            uri,
            DiagnosticsPublishTierV0::Baseline,
            &changed_baseline,
            false,
        ));
        assert!(
            deliver(
                &registry,
                uri,
                DiagnosticsPublishTierV0::Optimizing,
                &optimizing,
                true,
            ),
            "a changed baseline must force the final tier back to the client-visible optimizing payload"
        );
    }

    #[test]
    fn terminal_tier_transitions_and_close_reset_are_never_suppressed() {
        let registry = DiagnosticsPublishDigestRegistryV0::default();
        let uri = "file:///workspace/App.module.scss";
        let baseline = json!([{"code": "baseline"}]);
        let optimizing = json!([{"code": "optimizing"}]);

        assert!(deliver(
            &registry,
            uri,
            DiagnosticsPublishTierV0::Baseline,
            &baseline,
            false,
        ));
        assert!(deliver(
            &registry,
            uri,
            DiagnosticsPublishTierV0::Optimizing,
            &optimizing,
            true,
        ));
        assert!(
            deliver(
                &registry,
                uri,
                DiagnosticsPublishTierV0::Baseline,
                &baseline,
                true,
            ),
            "a terminal baseline must replace a previously visible optimizing payload"
        );
        assert!(
            deliver(
                &registry,
                uri,
                DiagnosticsPublishTierV0::Optimizing,
                &optimizing,
                true,
            ),
            "a later optimizing transition must restore its payload even when its tier digest was seen before"
        );

        let clear = registry.clear_document_receipt(uri);
        assert!(clear.should_deliver());
        clear.record_delivered();
        assert!(deliver(
            &registry,
            uri,
            DiagnosticsPublishTierV0::Baseline,
            &baseline,
            false,
        ));
        assert!(deliver(
            &registry,
            uri,
            DiagnosticsPublishTierV0::Optimizing,
            &optimizing,
            true,
        ));
    }
}
