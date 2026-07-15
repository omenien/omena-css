use crate::plugin_api::{
    EvidenceNodeKeyV0, FactPrecision, OMENA_PLUGIN_ABI_VERSION_V0, OmenaPlugin, PluginAnalysisV0,
    PluginKindV0, PluginMetadataV0, PluginOutcomeV0, PluginTransformContextV0, PluginTransformIrV0,
    PluginWorkspaceSnapshotV0, no_change_plugin_outcome,
};

pub(super) struct ViteBundleHostPluginV0;

pub(super) const VITE_BUNDLE_HOST_PLUGIN_METADATA: PluginMetadataV0 = PluginMetadataV0 {
    plugin_id: "vite-bundle-host",
    kind: PluginKindV0::BundleHost,
    version: "0",
    abi_version: OMENA_PLUGIN_ABI_VERSION_V0,
    stability: "inTreeExperimental",
    capabilities: &[
        "bundlerHostProtocol",
        "semanticClassMap",
        "namedExports",
        "exportDeltaHmr",
    ],
};

impl OmenaPlugin for ViteBundleHostPluginV0 {
    fn metadata(&self) -> &'static PluginMetadataV0 {
        &VITE_BUNDLE_HOST_PLUGIN_METADATA
    }

    fn analyze(&self, snapshot: &PluginWorkspaceSnapshotV0<'_>) -> PluginAnalysisV0 {
        PluginAnalysisV0 {
            summary: format!(
                "snapshot {} is available to the Vite bundler host",
                snapshot.snapshot_id().value
            ),
            evidence_reference: EvidenceNodeKeyV0::new(
                "omena_plugin_analysis",
                VITE_BUNDLE_HOST_PLUGIN_METADATA.plugin_id,
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
            VITE_BUNDLE_HOST_PLUGIN_METADATA.plugin_id,
            ir.nodes().len(),
            FactPrecision::Exact,
        )
    }
}

pub(super) static VITE_BUNDLE_HOST_PLUGIN: ViteBundleHostPluginV0 = ViteBundleHostPluginV0;
