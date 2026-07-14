mod semantic_observation;

use crate::plugin_api::{
    OmenaPlugin, PluginAnalysisV0, PluginOutcomeV0, PluginOutcomeValidationErrorV0,
    PluginTransformContextV0, PluginTransformIrV0, PluginWorkspaceSnapshotV0,
    validate_plugin_outcome,
};
use semantic_observation::SEMANTIC_OBSERVATION_PLUGIN;

static BUILT_IN_OMENA_PLUGINS: [&'static dyn OmenaPlugin; 1] = [&SEMANTIC_OBSERVATION_PLUGIN];

pub fn built_in_omena_plugins() -> &'static [&'static dyn OmenaPlugin] {
    &BUILT_IN_OMENA_PLUGINS
}

pub fn execute_built_in_omena_plugin(
    plugin_id: &str,
    snapshot: &PluginWorkspaceSnapshotV0<'_>,
    ir: &mut PluginTransformIrV0<'_>,
    context: PluginTransformContextV0,
) -> Option<Result<(PluginAnalysisV0, PluginOutcomeV0), PluginOutcomeValidationErrorV0>> {
    let plugin = BUILT_IN_OMENA_PLUGINS
        .iter()
        .find(|plugin| plugin.metadata().plugin_id == plugin_id)?;
    if context.snapshot_id != snapshot.snapshot_id() {
        return Some(Err(
            PluginOutcomeValidationErrorV0::SnapshotIdentityMismatch,
        ));
    }
    let analysis = plugin.analyze(snapshot);
    let outcome = plugin.transform(ir, context);
    Some(
        validate_plugin_outcome(plugin.metadata(), context, ir.mutation_count(), &outcome)
            .map(|()| (analysis, outcome)),
    )
}
