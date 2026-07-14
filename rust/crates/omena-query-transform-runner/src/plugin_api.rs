use omena_bridge::{
    StyleIntelligenceClassUniverseV0, StyleIntelligenceCompletionV0, StyleIntelligenceHoverV0,
    StyleIntelligenceProvider, StyleIntelligenceSnapshotV0, built_in_style_intelligence_provider,
    style_intelligence_completions_at_offset, style_intelligence_hover_at_offset,
};
use omena_incremental::OmenaWorkspaceSnapshotIdV0;
use omena_transform_cst::{
    IrEditRegionV0, IrNodeIdV0, IrNodeKindV0, IrTransactionErrorV0, IrTransactionV0,
    TransformIrPrintErrorV0, TransformIrV0, print_transform_ir_css,
};
use omena_transform_passes::{
    TransformDecision, TransformPassExecutionOutcomeV0, TransformPassRuntimeStatus,
};
use serde::Serialize;

pub use omena_abstract_value::FactPrecision;
pub use omena_evidence_graph::EvidenceNodeKeyV0;

pub const OMENA_PLUGIN_ABI_VERSION_V0: &str = "0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginMetadataV0 {
    pub plugin_id: &'static str,
    pub version: &'static str,
    pub abi_version: &'static str,
    pub stability: &'static str,
    pub capabilities: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginAnalysisV0 {
    pub summary: String,
    pub evidence_reference: EvidenceNodeKeyV0,
    pub precision: FactPrecision,
}

#[derive(Debug, Clone, Copy)]
pub struct PluginWorkspaceSnapshotV0<'snapshot> {
    snapshot_id: OmenaWorkspaceSnapshotIdV0,
    style_intelligence: StyleIntelligenceSnapshotV0<'snapshot>,
}

impl<'snapshot> PluginWorkspaceSnapshotV0<'snapshot> {
    pub const fn new(
        snapshot_id: OmenaWorkspaceSnapshotIdV0,
        style_intelligence: StyleIntelligenceSnapshotV0<'snapshot>,
    ) -> Self {
        Self {
            snapshot_id,
            style_intelligence,
        }
    }

    pub const fn snapshot_id(&self) -> OmenaWorkspaceSnapshotIdV0 {
        self.snapshot_id
    }

    pub fn class_universe(&self, provider_id: &str) -> Option<StyleIntelligenceClassUniverseV0> {
        built_in_style_intelligence_provider(provider_id)
            .map(|provider| provider.class_universe(&self.style_intelligence))
    }

    pub fn completions_at_offset(&self, byte_offset: usize) -> Vec<StyleIntelligenceCompletionV0> {
        style_intelligence_completions_at_offset(&self.style_intelligence, byte_offset)
    }

    pub fn hover_at_offset(&self, byte_offset: usize) -> Option<StyleIntelligenceHoverV0> {
        style_intelligence_hover_at_offset(&self.style_intelligence, byte_offset)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginTransformContextV0 {
    pub snapshot_id: OmenaWorkspaceSnapshotIdV0,
    pub required_precision: FactPrecision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginChangeSummaryV0 {
    pub observed_node_count: usize,
    pub mutation_count: usize,
    pub output_changed: bool,
}

/// The required result of every plugin transform.
///
/// Evidence and precision are fields rather than optional metadata. Omitting
/// either one is therefore a compile-time error.
///
/// ```compile_fail
/// use omena_query_transform_runner::{FactPrecision, PluginOutcomeV0};
///
/// let _ = PluginOutcomeV0 {
///     change_summary: todo!(),
///     precision: FactPrecision::Exact,
///     decision: todo!(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginOutcomeV0 {
    pub change_summary: PluginChangeSummaryV0,
    pub evidence_reference: EvidenceNodeKeyV0,
    pub precision: FactPrecision,
    pub decision: TransformDecision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginIrNodeV0 {
    pub node_id: IrNodeIdV0,
    pub kind: IrNodeKindV0,
    pub source_span_start: usize,
    pub source_span_end: usize,
}

pub struct PluginTransformIrV0<'ir> {
    ir: &'ir mut TransformIrV0,
    plugin_id: &'static str,
    mutation_count: usize,
}

impl<'ir> PluginTransformIrV0<'ir> {
    pub fn new(ir: &'ir mut TransformIrV0, plugin_id: &'static str) -> Self {
        Self {
            ir,
            plugin_id,
            mutation_count: 0,
        }
    }

    pub fn source_id(&self) -> &str {
        self.ir.source_id.as_str()
    }

    pub fn source_byte_len(&self) -> usize {
        self.ir.source_byte_len
    }

    pub fn nodes(&self) -> Vec<PluginIrNodeV0> {
        self.ir
            .nodes
            .iter()
            .map(|node| PluginIrNodeV0 {
                node_id: node.node_id,
                kind: node.kind,
                source_span_start: node.source_span_start,
                source_span_end: node.source_span_end,
            })
            .collect()
    }

    pub fn printed_css(&self) -> Result<String, TransformIrPrintErrorV0> {
        print_transform_ir_css(self.ir)
    }

    pub fn replace_node(
        &mut self,
        node_id: IrNodeIdV0,
        canonical_text: impl Into<String>,
    ) -> Result<(), IrTransactionErrorV0> {
        let region = IrEditRegionV0::full(self.ir.source_byte_len);
        let mut transaction = IrTransactionV0::new(self.ir, self.plugin_id, region);
        transaction.replace_node(node_id, canonical_text)?;
        transaction.commit()?;
        self.mutation_count += 1;
        Ok(())
    }

    pub const fn mutation_count(&self) -> usize {
        self.mutation_count
    }
}

pub trait OmenaPlugin: Sync {
    fn metadata(&self) -> &'static PluginMetadataV0;

    fn analyze(&self, snapshot: &PluginWorkspaceSnapshotV0<'_>) -> PluginAnalysisV0;

    fn transform(
        &self,
        ir: &mut PluginTransformIrV0<'_>,
        context: PluginTransformContextV0,
    ) -> PluginOutcomeV0;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PluginOutcomeValidationErrorV0 {
    SnapshotIdentityMismatch,
    AbiVersionMismatch,
    PassIdentityMismatch,
    ChangeSummaryMismatch,
    DecisionMismatch,
    EvidenceReferenceMismatch,
    PrecisionBelowRequired,
}

pub fn validate_plugin_outcome(
    metadata: &PluginMetadataV0,
    context: PluginTransformContextV0,
    observed_mutation_count: usize,
    outcome: &PluginOutcomeV0,
) -> Result<(), PluginOutcomeValidationErrorV0> {
    if metadata.abi_version != OMENA_PLUGIN_ABI_VERSION_V0 {
        return Err(PluginOutcomeValidationErrorV0::AbiVersionMismatch);
    }
    let execution_outcome = outcome.decision.compatibility_outcome();
    if execution_outcome.pass_id != metadata.plugin_id {
        return Err(PluginOutcomeValidationErrorV0::PassIdentityMismatch);
    }
    if outcome.change_summary.mutation_count != observed_mutation_count
        || execution_outcome.mutation_count != observed_mutation_count
    {
        return Err(PluginOutcomeValidationErrorV0::ChangeSummaryMismatch);
    }
    let decision_matches_change = match outcome.decision {
        TransformDecision::Applied { .. } => outcome.change_summary.output_changed,
        TransformDecision::NoChange { .. } => !outcome.change_summary.output_changed,
        TransformDecision::Blocked { .. } | TransformDecision::Rejected { .. } => {
            observed_mutation_count == 0 && !outcome.change_summary.output_changed
        }
    };
    if !decision_matches_change {
        return Err(PluginOutcomeValidationErrorV0::DecisionMismatch);
    }
    if outcome.evidence_reference != execution_outcome.evidence_node_key() {
        return Err(PluginOutcomeValidationErrorV0::EvidenceReferenceMismatch);
    }
    if !outcome.precision.satisfies(context.required_precision) {
        return Err(PluginOutcomeValidationErrorV0::PrecisionBelowRequired);
    }
    Ok(())
}

pub(crate) fn no_change_plugin_outcome(
    plugin_id: &'static str,
    observed_node_count: usize,
    precision: FactPrecision,
) -> PluginOutcomeV0 {
    let execution_outcome = TransformPassExecutionOutcomeV0 {
        pass_id: plugin_id,
        status: TransformPassRuntimeStatus::NoChange,
        input_byte_len: 0,
        output_byte_len: 0,
        mutation_count: 0,
        provenance_preserved: true,
        detail: "plugin observed the transform IR without changing emitted CSS",
    };
    PluginOutcomeV0 {
        change_summary: PluginChangeSummaryV0 {
            observed_node_count,
            mutation_count: 0,
            output_changed: false,
        },
        evidence_reference: execution_outcome.evidence_node_key(),
        precision,
        decision: TransformDecision::NoChange {
            reason: omena_transform_passes::TransformNoChangeReasonV0::NoMutation,
            outcome: execution_outcome,
        },
    }
}

#[cfg(test)]
mod tests {
    use omena_bridge::{StyleIntelligenceSnapshotV0, summarize_omena_bridge_source_syntax_index};
    use omena_incremental::{IncrementalRevisionV0, OmenaWorkspaceSnapshotIdV0};
    use omena_transform_cst::{StyleDialect, lower_transform_ir_from_source};

    use super::*;
    use crate::plugins::{built_in_omena_plugins, execute_built_in_omena_plugin};

    #[test]
    fn built_in_plugin_round_trips_analysis_and_fail_closed_transform() -> Result<(), &'static str>
    {
        let source = r#"import { cva } from "class-variance-authority";
const button = cva("btn", { variants: { intent: { primary: "a" } } });
const value = button({ intent: "primary" });"#;
        let source_index =
            summarize_omena_bridge_source_syntax_index(source, Vec::new(), Vec::new());
        let snapshot = PluginWorkspaceSnapshotV0::new(
            OmenaWorkspaceSnapshotIdV0::from_revision(IncrementalRevisionV0 { value: 7 }),
            StyleIntelligenceSnapshotV0::new(&source_index),
        );
        let mut transform_ir = lower_transform_ir_from_source(
            ".button { color: red; }",
            StyleDialect::Css,
            "input.css",
        );
        let mut plugin_ir = PluginTransformIrV0::new(&mut transform_ir, "semantic-observation");
        let context = PluginTransformContextV0 {
            snapshot_id: snapshot.snapshot_id(),
            required_precision: FactPrecision::Conservative,
        };
        let (analysis, outcome) = execute_built_in_omena_plugin(
            "semantic-observation",
            &snapshot,
            &mut plugin_ir,
            context,
        )
        .ok_or("built-in plugin should be registered")?
        .map_err(|_| "built-in plugin outcome should validate")?;

        assert!(
            analysis
                .summary
                .contains("snapshot 7 exposes 1 CVA class universes")
        );
        assert_eq!(analysis.precision, FactPrecision::Exact);
        assert_eq!(outcome.precision, FactPrecision::Exact);
        assert_eq!(outcome.change_summary.mutation_count, 0);
        assert!(matches!(
            outcome.decision,
            TransformDecision::NoChange { .. }
        ));
        assert_eq!(
            plugin_ir
                .printed_css()
                .map_err(|_| "plugin IR should remain printable")?,
            ".button { color: red; }"
        );
        Ok(())
    }

    #[test]
    fn outcome_validation_rejects_unbound_snapshot_precision_and_evidence()
    -> Result<(), &'static str> {
        let source_index = summarize_omena_bridge_source_syntax_index("", Vec::new(), Vec::new());
        let snapshot = PluginWorkspaceSnapshotV0::new(
            OmenaWorkspaceSnapshotIdV0::from_revision(IncrementalRevisionV0 { value: 1 }),
            StyleIntelligenceSnapshotV0::new(&source_index),
        );
        let mut transform_ir =
            lower_transform_ir_from_source("a {}", StyleDialect::Css, "input.css");
        let mut plugin_ir = PluginTransformIrV0::new(&mut transform_ir, "semantic-observation");
        let context = PluginTransformContextV0 {
            snapshot_id: snapshot.snapshot_id(),
            required_precision: FactPrecision::Exact,
        };
        assert_eq!(
            execute_built_in_omena_plugin(
                "semantic-observation",
                &snapshot,
                &mut plugin_ir,
                PluginTransformContextV0 {
                    snapshot_id: OmenaWorkspaceSnapshotIdV0 { value: 2 },
                    ..context
                },
            ),
            Some(Err(
                PluginOutcomeValidationErrorV0::SnapshotIdentityMismatch
            ))
        );
        let (_, mut outcome) = execute_built_in_omena_plugin(
            "semantic-observation",
            &snapshot,
            &mut plugin_ir,
            context,
        )
        .ok_or("built-in plugin should be registered")?
        .map_err(|_| "built-in plugin outcome should validate")?;
        let metadata = built_in_omena_plugins()
            .first()
            .ok_or("built-in plugin registry should be non-empty")?
            .metadata();

        outcome.precision = FactPrecision::Unknown;
        assert_eq!(
            validate_plugin_outcome(metadata, context, plugin_ir.mutation_count(), &outcome,),
            Err(PluginOutcomeValidationErrorV0::PrecisionBelowRequired)
        );
        outcome.precision = FactPrecision::Exact;
        outcome.evidence_reference = EvidenceNodeKeyV0::new("wrong", "evidence");
        assert_eq!(
            validate_plugin_outcome(metadata, context, plugin_ir.mutation_count(), &outcome,),
            Err(PluginOutcomeValidationErrorV0::EvidenceReferenceMismatch)
        );
        Ok(())
    }
}
