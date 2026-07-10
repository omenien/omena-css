use omena_query::{
    IncrementalRevisionV0, OmenaError, OmenaErrorClassV0, OmenaSdkDiagnosticsRequestV0,
    OmenaSdkResponsePartitionV0, OmenaSdkSnapshotResponseV0, OmenaWorkspaceSnapshotIdV0,
    omena_error_from_boundary_encoding,
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
