use super::*;

#[cfg(feature = "salsa-style-diagnostics")]
pub(crate) fn current_style_workspace_snapshot_id(
    state: &LspShellState,
) -> Option<omena_query::OmenaWorkspaceSnapshotIdV0> {
    let revision = state.style_workspace_snapshot_revision_hint();
    Some(omena_query::OmenaWorkspaceSnapshotIdV0::from_revision(
        revision,
    ))
}

#[cfg(not(feature = "salsa-style-diagnostics"))]
pub(crate) fn current_style_workspace_snapshot_id(
    _state: &LspShellState,
) -> Option<omena_query::OmenaWorkspaceSnapshotIdV0> {
    None
}

pub(crate) fn attach_workspace_snapshot_id_to_diagnostics(
    mut diagnostics: Value,
    snapshot_id: Option<omena_query::OmenaWorkspaceSnapshotIdV0>,
) -> Value {
    let Some(snapshot_id) = snapshot_id else {
        return diagnostics;
    };
    let Some(elements) = diagnostics.as_array_mut() else {
        return diagnostics;
    };
    for element in elements {
        let Some(diagnostic) = element.as_object_mut() else {
            continue;
        };
        let data = diagnostic
            .entry("data")
            .or_insert_with(|| json!({}))
            .as_object_mut();
        if let Some(data) = data {
            attach_workspace_snapshot_id_to_diagnostic_data(data, snapshot_id);
        }
    }
    diagnostics
}

fn attach_workspace_snapshot_id_to_diagnostic_data(
    data: &mut serde_json::Map<String, Value>,
    snapshot_id: omena_query::OmenaWorkspaceSnapshotIdV0,
) {
    const ORDERED_KEYS: &[&str] = &[
        "querySeverity",
        "provenance",
        "createCustomProperty",
        "runtimeState",
        "cascadeNarrowing",
        "cascadeConfidence",
        "polynomialProvenance",
        "crossFileScc",
    ];
    const SNAPSHOT_AFTER_KEY: &str = "provenance";

    let mut remaining = std::mem::take(data);
    remaining.remove("snapshotId");
    let mut reordered = serde_json::Map::new();
    let mut inserted = false;

    for key in ORDERED_KEYS {
        if let Some(value) = remaining.remove(*key) {
            reordered.insert((*key).to_string(), value);
        }
        if *key == SNAPSHOT_AFTER_KEY {
            reordered.insert("snapshotId".to_string(), json!(snapshot_id));
            inserted = true;
        }
    }
    for (key, value) in remaining {
        reordered.insert(key, value);
    }
    if !inserted {
        reordered.insert("snapshotId".to_string(), json!(snapshot_id));
    }
    *data = reordered;
}

/// The full argument surface of [`crate::style_diagnostics::finish_style_diagnostics_value`]:
/// plain `Send` data only, by design - no `&LspShellState`.
pub(crate) struct LspStyleDiagnosticsRenderInputsV0<'inputs> {
    pub(crate) document_uri: &'inputs str,
    pub(crate) document_text: &'inputs str,
    pub(crate) query_candidates: &'inputs [omena_query::OmenaQueryStyleHoverCandidateV0],
    pub(crate) snapshot_id: Option<omena_query::OmenaWorkspaceSnapshotIdV0>,
    pub(crate) deep_analysis: bool,
    pub(crate) configured_severity: u8,
}
