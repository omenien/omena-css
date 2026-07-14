use crate::plugin_api::{
    EvidenceNodeKeyV0, FactPrecision, OMENA_PLUGIN_ABI_VERSION_V0, OmenaPlugin, PluginAnalysisV0,
    PluginMetadataV0, PluginOutcomeV0, PluginTransformContextV0, PluginTransformIrV0,
    PluginWorkspaceSnapshotV0, no_change_plugin_outcome,
};

pub(super) struct SemanticObservationPluginV0;

pub(super) const SEMANTIC_OBSERVATION_PLUGIN_METADATA: PluginMetadataV0 = PluginMetadataV0 {
    plugin_id: "semantic-observation",
    version: "0",
    abi_version: OMENA_PLUGIN_ABI_VERSION_V0,
    stability: "inTreeExperimental",
    capabilities: &["analyze", "transform"],
};

impl OmenaPlugin for SemanticObservationPluginV0 {
    fn metadata(&self) -> &'static PluginMetadataV0 {
        &SEMANTIC_OBSERVATION_PLUGIN_METADATA
    }

    fn analyze(&self, snapshot: &PluginWorkspaceSnapshotV0<'_>) -> PluginAnalysisV0 {
        let indexed_universe_count = snapshot
            .class_universe("cva-recipe-domain")
            .map_or(0, |universe| universe.entries.len());
        PluginAnalysisV0 {
            summary: format!(
                "snapshot {} exposes {indexed_universe_count} CVA class universes",
                snapshot.snapshot_id().value
            ),
            evidence_reference: EvidenceNodeKeyV0::new(
                "omena_plugin_analysis",
                SEMANTIC_OBSERVATION_PLUGIN_METADATA.plugin_id,
            ),
            precision: FactPrecision::Exact,
        }
    }

    fn transform(
        &self,
        ir: &mut PluginTransformIrV0<'_>,
        _context: PluginTransformContextV0,
    ) -> PluginOutcomeV0 {
        no_change_plugin_outcome(
            SEMANTIC_OBSERVATION_PLUGIN_METADATA.plugin_id,
            ir.nodes().len(),
            FactPrecision::Exact,
        )
    }
}

pub(super) static SEMANTIC_OBSERVATION_PLUGIN: SemanticObservationPluginV0 =
    SemanticObservationPluginV0;
