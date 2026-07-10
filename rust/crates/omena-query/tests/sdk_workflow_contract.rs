use omena_query::{
    IncrementalRevisionV0, OmenaError, OmenaErrorClassV0, OmenaSdkDiagnosticsRequestV0,
    OmenaSdkResponsePartitionV0, OmenaSdkSnapshotResponseV0, OmenaWorkspaceSnapshotIdV0,
    execute_omena_sdk_diagnostics_workflow, omena_error_from_boundary_encoding,
};

#[test]
fn sdk_workflow_contract_round_trips_existing_snapshot_identity() -> Result<(), serde_json::Error> {
    let snapshot_id =
        OmenaWorkspaceSnapshotIdV0::from_revision(IncrementalRevisionV0 { value: 41 });
    let response = OmenaSdkSnapshotResponseV0 {
        snapshot_id,
        partition: OmenaSdkResponsePartitionV0::Public,
        workspace_root: "/workspace".to_string(),
    };

    let encoded = serde_json::to_string(&response)?;
    let decoded: OmenaSdkSnapshotResponseV0 = serde_json::from_str(&encoded)?;

    assert_eq!(decoded, response);
    assert_eq!(decoded.snapshot_id.revision().value, 41);
    Ok(())
}

#[test]
fn unified_error_round_trips_with_typed_context() -> Result<(), serde_json::Error> {
    let error = omena_error_from_boundary_encoding(
        "unsupported-mode",
        "external mode is not available",
        "build",
    );

    let encoded = serde_json::to_string(&error)?;
    let decoded: OmenaError = serde_json::from_str(&encoded)?;

    assert_eq!(decoded, error);
    assert_eq!(decoded.class, OmenaErrorClassV0::Unsupported);
    assert_eq!(decoded.context.code, "boundary.build.unsupported-mode");
    Ok(())
}

#[test]
fn diagnostics_workflow_preserves_the_read_snapshot_identity() -> Result<(), OmenaError> {
    let snapshot_id =
        OmenaWorkspaceSnapshotIdV0::from_revision(IncrementalRevisionV0 { value: 23 });
    let response = execute_omena_sdk_diagnostics_workflow(
        OmenaSdkDiagnosticsRequestV0 {
            snapshot_id,
            style_path: "src/card.module.scss".to_string(),
            style_source: ".card { --tone: red; color: var(--tone); }".to_string(),
        },
        snapshot_id,
    )?;

    assert_eq!(response.snapshot_id, snapshot_id);
    assert_eq!(response.partition, OmenaSdkResponsePartitionV0::Public);
    assert_eq!(response.summary.style_path, "src/card.module.scss");
    assert_eq!(response.summary.dialect, "scss");
    assert_eq!(response.summary.class_selector_count, 1);
    assert_eq!(response.summary.custom_property_count, 1);
    Ok(())
}

#[test]
fn diagnostics_workflow_rejects_a_stale_snapshot() {
    let requested = OmenaWorkspaceSnapshotIdV0::from_revision(IncrementalRevisionV0 { value: 8 });
    let current = OmenaWorkspaceSnapshotIdV0::from_revision(IncrementalRevisionV0 { value: 9 });

    let result = execute_omena_sdk_diagnostics_workflow(
        OmenaSdkDiagnosticsRequestV0 {
            snapshot_id: requested,
            style_path: "src/card.module.css".to_string(),
            style_source: ".card { color: red; }".to_string(),
        },
        current,
    );
    assert!(
        result.is_err(),
        "stale snapshot must not produce a diagnostics response"
    );
    let Some(error) = result.err() else {
        return;
    };

    assert_eq!(error.class, OmenaErrorClassV0::Workspace);
    assert_eq!(error.context.code, "workspace.snapshot-mismatch");
}

#[test]
fn diagnostics_request_carries_snapshot_identity() -> Result<(), serde_json::Error> {
    let request = OmenaSdkDiagnosticsRequestV0 {
        snapshot_id: OmenaWorkspaceSnapshotIdV0::from_revision(IncrementalRevisionV0 { value: 7 }),
        style_path: "src/card.module.scss".to_string(),
        style_source: ".card { color: red; }".to_string(),
    };

    let encoded = serde_json::to_value(&request)?;
    assert_eq!(encoded["snapshotId"]["value"], 7);
    assert_eq!(encoded["stylePath"], "src/card.module.scss");
    Ok(())
}
