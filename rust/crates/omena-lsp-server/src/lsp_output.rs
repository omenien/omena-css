use crate::LspStyleHoverCandidate;
use omena_query::{
    OmenaQueryExternalSifInputV0, OmenaQuerySourceDocumentInputV0,
    OmenaQuerySourceMissingSelectorDiagnosticCandidateV0, OmenaQuerySourceSyntaxIndexV0,
    OmenaQueryStyleHoverCandidateV0, OmenaQueryStylePackageManifestV0,
    OmenaQueryStyleResolutionInputsV0, OmenaQueryStyleSelectorDefinitionV0,
    OmenaQueryStyleSourceInputV0,
};
use serde_json::Value;

pub const OPTIMIZING_DIAGNOSTICS_DELAY_MS: u64 = 200;

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledLspOutput {
    pub value: Value,
    pub delay_millis: Option<u64>,
    pub coalesce_key: Option<String>,
}

impl ScheduledLspOutput {
    pub fn immediate(value: Value) -> Self {
        Self {
            value,
            delay_millis: None,
            coalesce_key: None,
        }
    }

    pub fn immediate_coalesced(value: Value, coalesce_key: String) -> Self {
        Self {
            value,
            delay_millis: None,
            coalesce_key: Some(coalesce_key),
        }
    }

    pub fn delayed(value: Value, delay_millis: u64) -> Self {
        Self {
            value,
            delay_millis: Some(delay_millis),
            coalesce_key: None,
        }
    }

    pub fn delayed_coalesced(value: Value, delay_millis: u64, coalesce_key: String) -> Self {
        Self {
            value,
            delay_millis: Some(delay_millis),
            coalesce_key: Some(coalesce_key),
        }
    }

    pub fn into_value(self) -> Value {
        self.value
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
    pub configured_severity: u8,
}

#[derive(Debug, Clone)]
pub enum DeferredDiagnosticsRenderInputsV0 {
    Style(LspOwnedStyleDiagnosticsRenderInputsV0),
    Source(LspOwnedSourceDiagnosticsRenderInputsV0),
}

#[derive(Debug, Clone)]
pub struct LspDeferredDiagnosticsDispatchV0 {
    pub uri: String,
    pub coalesce_key: String,
    pub tier_plan: DiagnosticsPipelineTierPlanV0,
    pub render_inputs: DeferredDiagnosticsRenderInputsV0,
}
