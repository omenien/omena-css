use omena_evidence_graph::{EvidenceNodeKeyV0, GuaranteeKindV0};
use omena_parser::{ClosedWorldBundleV0, ParserPositionV0, ParserRangeV0};
use omena_query_core::{FactPrecision, fact_precision_from_analysis_precision};
use omena_query_transform_runner::TransformDecision;
use serde::Serialize;

use crate::{
    OmenaQueryCascadeAtPositionV0, OmenaQuerySourcePrecisionReferenceV0,
    OmenaQueryStyleDiagnosticV0,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaQueryExplainAvailabilityV0 {
    Available,
    NotYetAvailable,
    NotFound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaQueryExplainCapabilityV0 {
    Diagnostic,
    Transform,
    TreeShake,
    Precision,
    Cascade,
    Bundle,
    HoverTrace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaQueryExplainSymbolKindV0 {
    Class,
    Keyframes,
    Value,
    CustomProperty,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum OmenaQueryExplainTargetV0 {
    Diagnostic {
        style_path: String,
        code: String,
        range: ParserRangeV0,
    },
    Transform {
        pass_id: String,
        decision_ordinal: usize,
    },
    TreeShake {
        symbol_kind: OmenaQueryExplainSymbolKindV0,
        symbol_name: String,
    },
    Precision {
        source_path: String,
        variable_name: String,
        reference_byte_offset: usize,
    },
    Cascade {
        style_path: String,
        position: ParserPositionV0,
    },
    Bundle {
        chunk_reference: String,
    },
    HoverTrace {
        document_uri: String,
        position: Option<ParserPositionV0>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum OmenaQueryExplainFactReferenceV0 {
    Diagnostic {
        style_path: String,
        code: String,
        range: ParserRangeV0,
        evidence_node_key: EvidenceNodeKeyV0,
    },
    TransformOutcome {
        pass_id: String,
        decision_ordinal: usize,
        evidence_node_key: EvidenceNodeKeyV0,
    },
    ClosedWorldReachability {
        closure_hash: String,
        symbol_kind: OmenaQueryExplainSymbolKindV0,
        symbol_name: String,
        guarantee: GuaranteeKindV0,
    },
    PrecisionFact {
        source_path: String,
        variable_name: String,
        reference_byte_offset: usize,
    },
    CascadeResolution {
        style_path: String,
        position: ParserPositionV0,
        winner_range: Option<ParserRangeV0>,
    },
    CapabilityGate {
        capability: OmenaQueryExplainCapabilityV0,
    },
    HoverResolution {
        document_uri: String,
        position: Option<ParserPositionV0>,
        reason_code: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum OmenaQueryExplainFactValueV0 {
    DiagnosticIdentity {
        severity: String,
    },
    ProvenanceLabel {
        label: String,
    },
    TransformDecision {
        decision_kind: &'static str,
        status: String,
        mutation_count: usize,
        provenance_preserved: bool,
    },
    ReachabilityMembership {
        reachable: bool,
    },
    PrecisionClassification {
        precision: FactPrecision,
        resolved_tier: String,
    },
    CascadeResolution {
        status: String,
        candidate_count: usize,
        winner_source_order: Option<usize>,
    },
    CapabilityAvailability {
        availability: OmenaQueryExplainAvailabilityV0,
    },
    HoverResolution {
        matched: bool,
        candidate_count: usize,
        definition_count: usize,
    },
}

/// A fact-backed explanation component.
///
/// The fields are intentionally private so callers cannot construct a prose-only
/// explanation without a typed reference.
///
/// ```compile_fail
/// use omena_query::OmenaQueryExplainFactV0;
///
/// let _ = OmenaQueryExplainFactV0 {};
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryExplainFactV0 {
    reference: OmenaQueryExplainFactReferenceV0,
    value: OmenaQueryExplainFactValueV0,
}

impl OmenaQueryExplainFactV0 {
    fn new(
        reference: OmenaQueryExplainFactReferenceV0,
        value: OmenaQueryExplainFactValueV0,
    ) -> Self {
        Self { reference, value }
    }

    pub fn reference(&self) -> &OmenaQueryExplainFactReferenceV0 {
        &self.reference
    }

    pub fn value(&self) -> &OmenaQueryExplainFactValueV0 {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryExplainSourceSpanV0 {
    source_path: String,
    range: ParserRangeV0,
    fact_reference: OmenaQueryExplainFactReferenceV0,
}

impl OmenaQueryExplainSourceSpanV0 {
    fn new(
        source_path: impl Into<String>,
        range: ParserRangeV0,
        fact_reference: OmenaQueryExplainFactReferenceV0,
    ) -> Self {
        Self {
            source_path: source_path.into(),
            range,
            fact_reference,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryExplainResponseV0 {
    schema_version: &'static str,
    product: &'static str,
    target: OmenaQueryExplainTargetV0,
    availability: OmenaQueryExplainAvailabilityV0,
    primary_fact: OmenaQueryExplainFactV0,
    supporting_facts: Vec<OmenaQueryExplainFactV0>,
    related_spans: Vec<OmenaQueryExplainSourceSpanV0>,
}

impl OmenaQueryExplainResponseV0 {
    fn new(
        target: OmenaQueryExplainTargetV0,
        availability: OmenaQueryExplainAvailabilityV0,
        primary_fact: OmenaQueryExplainFactV0,
        supporting_facts: Vec<OmenaQueryExplainFactV0>,
        related_spans: Vec<OmenaQueryExplainSourceSpanV0>,
    ) -> Self {
        Self {
            schema_version: "0",
            product: "omena-query.explain",
            target,
            availability,
            primary_fact,
            supporting_facts,
            related_spans,
        }
    }

    pub fn target(&self) -> &OmenaQueryExplainTargetV0 {
        &self.target
    }

    pub const fn availability(&self) -> OmenaQueryExplainAvailabilityV0 {
        self.availability
    }

    pub fn primary_fact(&self) -> &OmenaQueryExplainFactV0 {
        &self.primary_fact
    }

    pub fn supporting_facts(&self) -> &[OmenaQueryExplainFactV0] {
        self.supporting_facts.as_slice()
    }

    pub fn related_spans(&self) -> &[OmenaQueryExplainSourceSpanV0] {
        self.related_spans.as_slice()
    }
}

pub enum OmenaQueryExplainInputV0<'a> {
    Diagnostic {
        style_path: &'a str,
        diagnostic: &'a OmenaQueryStyleDiagnosticV0,
    },
    Transform {
        decision: &'a TransformDecision,
        decision_ordinal: usize,
    },
    TreeShake {
        bundle: &'a ClosedWorldBundleV0,
        symbol_kind: OmenaQueryExplainSymbolKindV0,
        symbol_name: &'a str,
    },
    Precision {
        reference: &'a OmenaQuerySourcePrecisionReferenceV0,
    },
    Cascade {
        result: &'a OmenaQueryCascadeAtPositionV0,
    },
    BundleUnavailable {
        chunk_reference: &'a str,
    },
    HoverTrace {
        document_uri: &'a str,
        position: Option<ParserPositionV0>,
        reason_code: &'a str,
        matched: bool,
        candidate_count: usize,
        definition_count: usize,
    },
}

pub fn explain_omena_query(input: OmenaQueryExplainInputV0<'_>) -> OmenaQueryExplainResponseV0 {
    match input {
        OmenaQueryExplainInputV0::Diagnostic {
            style_path,
            diagnostic,
        } => explain_diagnostic(style_path, diagnostic),
        OmenaQueryExplainInputV0::Transform {
            decision,
            decision_ordinal,
        } => explain_transform(decision, decision_ordinal),
        OmenaQueryExplainInputV0::TreeShake {
            bundle,
            symbol_kind,
            symbol_name,
        } => explain_tree_shake(bundle, symbol_kind, symbol_name),
        OmenaQueryExplainInputV0::Precision { reference } => explain_precision(reference),
        OmenaQueryExplainInputV0::Cascade { result } => explain_cascade(result),
        OmenaQueryExplainInputV0::BundleUnavailable { chunk_reference } => {
            explain_bundle_unavailable(chunk_reference)
        }
        OmenaQueryExplainInputV0::HoverTrace {
            document_uri,
            position,
            reason_code,
            matched,
            candidate_count,
            definition_count,
        } => explain_hover_trace(
            document_uri,
            position,
            reason_code,
            matched,
            candidate_count,
            definition_count,
        ),
    }
}

fn explain_diagnostic(
    style_path: &str,
    diagnostic: &OmenaQueryStyleDiagnosticV0,
) -> OmenaQueryExplainResponseV0 {
    let evidence_node_key = EvidenceNodeKeyV0::new("diagnosticProvenance", diagnostic.code);
    let reference = OmenaQueryExplainFactReferenceV0::Diagnostic {
        style_path: style_path.to_string(),
        code: diagnostic.code.to_string(),
        range: diagnostic.range,
        evidence_node_key,
    };
    let supporting_facts = diagnostic
        .provenance
        .iter()
        .map(|label| {
            OmenaQueryExplainFactV0::new(
                reference.clone(),
                OmenaQueryExplainFactValueV0::ProvenanceLabel {
                    label: (*label).to_string(),
                },
            )
        })
        .collect();

    OmenaQueryExplainResponseV0::new(
        OmenaQueryExplainTargetV0::Diagnostic {
            style_path: style_path.to_string(),
            code: diagnostic.code.to_string(),
            range: diagnostic.range,
        },
        OmenaQueryExplainAvailabilityV0::Available,
        OmenaQueryExplainFactV0::new(
            reference.clone(),
            OmenaQueryExplainFactValueV0::DiagnosticIdentity {
                severity: diagnostic.severity.to_string(),
            },
        ),
        supporting_facts,
        vec![OmenaQueryExplainSourceSpanV0::new(
            style_path,
            diagnostic.range,
            reference,
        )],
    )
}

fn explain_transform(
    decision: &TransformDecision,
    decision_ordinal: usize,
) -> OmenaQueryExplainResponseV0 {
    let outcome = decision.compatibility_outcome();
    let decision_kind = match decision {
        TransformDecision::Applied { .. } => "applied",
        TransformDecision::NoChange { .. } => "noChange",
        TransformDecision::Blocked { .. } => "blocked",
        TransformDecision::Rejected { .. } => "rejected",
    };
    let reference = OmenaQueryExplainFactReferenceV0::TransformOutcome {
        pass_id: outcome.pass_id.to_string(),
        decision_ordinal,
        evidence_node_key: outcome.evidence_node_key(),
    };
    OmenaQueryExplainResponseV0::new(
        OmenaQueryExplainTargetV0::Transform {
            pass_id: outcome.pass_id.to_string(),
            decision_ordinal,
        },
        OmenaQueryExplainAvailabilityV0::Available,
        OmenaQueryExplainFactV0::new(
            reference,
            OmenaQueryExplainFactValueV0::TransformDecision {
                decision_kind,
                status: format!("{:?}", outcome.status),
                mutation_count: outcome.mutation_count,
                provenance_preserved: outcome.provenance_preserved,
            },
        ),
        Vec::new(),
        Vec::new(),
    )
}

fn explain_tree_shake(
    bundle: &ClosedWorldBundleV0,
    symbol_kind: OmenaQueryExplainSymbolKindV0,
    symbol_name: &str,
) -> OmenaQueryExplainResponseV0 {
    let reachable = match symbol_kind {
        OmenaQueryExplainSymbolKindV0::Class => bundle
            .reachability()
            .class_names()
            .iter()
            .any(|candidate| candidate == symbol_name),
        OmenaQueryExplainSymbolKindV0::Keyframes => bundle
            .reachability()
            .keyframe_names()
            .iter()
            .any(|candidate| candidate == symbol_name),
        OmenaQueryExplainSymbolKindV0::Value => bundle
            .reachability()
            .value_names()
            .iter()
            .any(|candidate| candidate == symbol_name),
        OmenaQueryExplainSymbolKindV0::CustomProperty => bundle
            .reachability()
            .custom_property_names()
            .iter()
            .any(|candidate| candidate == symbol_name),
    };
    let reference = OmenaQueryExplainFactReferenceV0::ClosedWorldReachability {
        closure_hash: bundle.closure_hash().to_string(),
        symbol_kind,
        symbol_name: symbol_name.to_string(),
        guarantee: GuaranteeKindV0::NotClaimedExactTraversal,
    };
    OmenaQueryExplainResponseV0::new(
        OmenaQueryExplainTargetV0::TreeShake {
            symbol_kind,
            symbol_name: symbol_name.to_string(),
        },
        OmenaQueryExplainAvailabilityV0::Available,
        OmenaQueryExplainFactV0::new(
            reference,
            OmenaQueryExplainFactValueV0::ReachabilityMembership { reachable },
        ),
        Vec::new(),
        Vec::new(),
    )
}

fn explain_precision(
    precision_reference: &OmenaQuerySourcePrecisionReferenceV0,
) -> OmenaQueryExplainResponseV0 {
    let reference = OmenaQueryExplainFactReferenceV0::PrecisionFact {
        source_path: precision_reference.source_path.clone(),
        variable_name: precision_reference.variable_name.clone(),
        reference_byte_offset: precision_reference.reference_byte_offset,
    };
    OmenaQueryExplainResponseV0::new(
        OmenaQueryExplainTargetV0::Precision {
            source_path: precision_reference.source_path.clone(),
            variable_name: precision_reference.variable_name.clone(),
            reference_byte_offset: precision_reference.reference_byte_offset,
        },
        OmenaQueryExplainAvailabilityV0::Available,
        OmenaQueryExplainFactV0::new(
            reference,
            OmenaQueryExplainFactValueV0::PrecisionClassification {
                precision: fact_precision_from_analysis_precision(&precision_reference.precision),
                resolved_tier: precision_reference.resolved_tier.to_string(),
            },
        ),
        Vec::new(),
        Vec::new(),
    )
}

fn explain_cascade(result: &OmenaQueryCascadeAtPositionV0) -> OmenaQueryExplainResponseV0 {
    let reference = OmenaQueryExplainFactReferenceV0::CascadeResolution {
        style_path: result.style_path.clone(),
        position: result.query_position,
        winner_range: result.winner_declaration_range,
    };
    let related_spans = result
        .winner_declaration_range
        .map(|range| {
            vec![OmenaQueryExplainSourceSpanV0::new(
                result
                    .winner_declaration_file_path
                    .as_deref()
                    .unwrap_or(result.style_path.as_str()),
                range,
                reference.clone(),
            )]
        })
        .unwrap_or_default();
    OmenaQueryExplainResponseV0::new(
        OmenaQueryExplainTargetV0::Cascade {
            style_path: result.style_path.clone(),
            position: result.query_position,
        },
        OmenaQueryExplainAvailabilityV0::Available,
        OmenaQueryExplainFactV0::new(
            reference,
            OmenaQueryExplainFactValueV0::CascadeResolution {
                status: result.status.to_string(),
                candidate_count: result.candidate_declaration_count,
                winner_source_order: result.winner_declaration_source_order,
            },
        ),
        Vec::new(),
        related_spans,
    )
}

fn explain_bundle_unavailable(chunk_reference: &str) -> OmenaQueryExplainResponseV0 {
    OmenaQueryExplainResponseV0::new(
        OmenaQueryExplainTargetV0::Bundle {
            chunk_reference: chunk_reference.to_string(),
        },
        OmenaQueryExplainAvailabilityV0::NotYetAvailable,
        OmenaQueryExplainFactV0::new(
            OmenaQueryExplainFactReferenceV0::CapabilityGate {
                capability: OmenaQueryExplainCapabilityV0::Bundle,
            },
            OmenaQueryExplainFactValueV0::CapabilityAvailability {
                availability: OmenaQueryExplainAvailabilityV0::NotYetAvailable,
            },
        ),
        Vec::new(),
        Vec::new(),
    )
}

fn explain_hover_trace(
    document_uri: &str,
    position: Option<ParserPositionV0>,
    reason_code: &str,
    matched: bool,
    candidate_count: usize,
    definition_count: usize,
) -> OmenaQueryExplainResponseV0 {
    OmenaQueryExplainResponseV0::new(
        OmenaQueryExplainTargetV0::HoverTrace {
            document_uri: document_uri.to_string(),
            position,
        },
        OmenaQueryExplainAvailabilityV0::Available,
        OmenaQueryExplainFactV0::new(
            OmenaQueryExplainFactReferenceV0::HoverResolution {
                document_uri: document_uri.to_string(),
                position,
                reason_code: reason_code.to_string(),
            },
            OmenaQueryExplainFactValueV0::HoverResolution {
                matched,
                candidate_count,
                definition_count,
            },
        ),
        Vec::new(),
        Vec::new(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use omena_parser::{ConfigurationHashV0, ModuleIdV0, ModuleInstanceKeyV0};

    #[test]
    fn transform_explanation_references_the_production_outcome_evidence_key() {
        let execution = crate::execute_omena_query_transform_passes_from_source(
            "fixture.css",
            ".button {}",
            &["print-css".to_string()],
        );
        let decision = &execution.execution.decisions[0];
        let outcome = decision.compatibility_outcome();
        let response = explain_omena_query(OmenaQueryExplainInputV0::Transform {
            decision,
            decision_ordinal: 0,
        });

        assert_eq!(
            response.availability(),
            OmenaQueryExplainAvailabilityV0::Available
        );
        assert!(matches!(
            response.primary_fact().reference(),
            OmenaQueryExplainFactReferenceV0::TransformOutcome {
                evidence_node_key,
                ..
            } if evidence_node_key == &outcome.evidence_node_key()
        ));
    }

    #[test]
    fn tree_shake_explanation_inherits_the_non_exact_traversal_guarantee() -> Result<(), String> {
        let module = ModuleInstanceKeyV0::new(
            ModuleIdV0::new("src/App.module.css"),
            ConfigurationHashV0::new("default"),
        );
        let bundle = ClosedWorldBundleV0::try_from_linked_modules(
            vec![module.clone()],
            vec![omena_parser::ClosedWorldLinkedModuleV0::new(module).with_class_name("button")],
        )
        .map_err(|error| format!("failed to build closed-world fixture: {error:?}"))?;
        let response = explain_omena_query(OmenaQueryExplainInputV0::TreeShake {
            bundle: &bundle,
            symbol_kind: OmenaQueryExplainSymbolKindV0::Class,
            symbol_name: "button",
        });

        assert!(matches!(
            response.primary_fact().reference(),
            OmenaQueryExplainFactReferenceV0::ClosedWorldReachability {
                guarantee: GuaranteeKindV0::NotClaimedExactTraversal,
                ..
            }
        ));
        assert!(matches!(
            response.primary_fact().value(),
            OmenaQueryExplainFactValueV0::ReachabilityMembership { reachable: true }
        ));
        Ok(())
    }

    #[test]
    fn unavailable_bundle_explanation_is_still_fact_backed() {
        let response = explain_omena_query(OmenaQueryExplainInputV0::BundleUnavailable {
            chunk_reference: "main",
        });
        assert_eq!(
            response.availability(),
            OmenaQueryExplainAvailabilityV0::NotYetAvailable
        );
        assert!(matches!(
            response.primary_fact().reference(),
            OmenaQueryExplainFactReferenceV0::CapabilityGate {
                capability: OmenaQueryExplainCapabilityV0::Bundle
            }
        ));
    }

    #[test]
    fn diagnostic_explanation_reuses_product_diagnostic_provenance() -> Result<(), String> {
        let diagnostics = crate::summarize_omena_query_style_diagnostics_for_file(
            "file:///fixture.scss",
            "@import 'legacy';",
            &[],
        );
        let diagnostic = diagnostics
            .diagnostics
            .first()
            .ok_or_else(|| "fixture should produce a Sass import diagnostic".to_string())?;
        let response = explain_omena_query(OmenaQueryExplainInputV0::Diagnostic {
            style_path: "file:///fixture.scss",
            diagnostic,
        });

        assert_eq!(
            response.supporting_facts().len(),
            diagnostic.provenance.len()
        );
        assert!(matches!(
            response.primary_fact().reference(),
            OmenaQueryExplainFactReferenceV0::Diagnostic {
                code,
                evidence_node_key,
                ..
            } if code == diagnostic.code
                && evidence_node_key == &EvidenceNodeKeyV0::new("diagnosticProvenance", diagnostic.code)
        ));
        assert_eq!(response.related_spans().len(), 1);
        Ok(())
    }

    #[test]
    fn precision_explanation_uses_the_authoritative_precision_adapter() -> Result<(), String> {
        let source = "const className = 'button';\nclassName;";
        let reference_byte_offset = source
            .rfind("className")
            .ok_or_else(|| "fixture reference should exist".to_string())?;
        let reference = crate::resolve_omena_query_source_precision_for_source(
            "fixture.ts",
            source,
            Some("typescript"),
            "className",
            reference_byte_offset,
        );
        let response = explain_omena_query(OmenaQueryExplainInputV0::Precision {
            reference: &reference,
        });

        assert!(matches!(
            response.primary_fact().value(),
            OmenaQueryExplainFactValueV0::PrecisionClassification {
                precision: FactPrecision::Conservative,
                ..
            }
        ));
        Ok(())
    }
}
