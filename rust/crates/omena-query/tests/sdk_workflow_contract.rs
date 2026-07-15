use omena_query::{
    IncrementalRevisionV0, OmenaError, OmenaErrorClassV0, OmenaQueryStyleSourceInputV0,
    OmenaSdkBuildRequestV0, OmenaSdkDiagnosticsRequestV0, OmenaSdkExplainPositionV0,
    OmenaSdkExplainRequestV0, OmenaSdkQueryRequestV0, OmenaSdkResponsePartitionV0,
    OmenaSdkSnapshotRequestV0, OmenaSdkSnapshotResponseV0, OmenaSdkWorkspaceV0,
    OmenaWorkspaceSnapshotIdV0, execute_omena_sdk_diagnostics_debug_workflow,
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
fn diagnostics_debug_report_is_opt_in_and_keeps_the_public_response() -> Result<(), OmenaError> {
    let snapshot_id =
        OmenaWorkspaceSnapshotIdV0::from_revision(IncrementalRevisionV0 { value: 31 });
    let report = execute_omena_sdk_diagnostics_debug_workflow(
        OmenaSdkDiagnosticsRequestV0 {
            snapshot_id,
            style_path: "src/debug.module.css".to_string(),
            style_source: ".debug { color: green; }".to_string(),
        },
        snapshot_id,
    )?;

    assert_eq!(report.partition, OmenaSdkResponsePartitionV0::Debug);
    assert_eq!(
        report.public_response.partition,
        OmenaSdkResponsePartitionV0::Public
    );
    assert_eq!(report.public_response.snapshot_id, report.snapshot_id);
    assert!(report.analysis.get("readySurfaces").is_some());
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

fn workspace() -> Result<OmenaSdkWorkspaceV0, OmenaError> {
    OmenaSdkWorkspaceV0::open(
        OmenaSdkSnapshotRequestV0 {
            workspace_root: "/workspace".to_string(),
        },
        [OmenaQueryStyleSourceInputV0 {
            style_path: "src/card.module.scss".to_string(),
            style_source: ".card { --tone: red; color: var(--tone); }".to_string(),
        }],
    )
}

#[test]
fn workspace_runtime_executes_every_typed_workflow() -> Result<(), OmenaError> {
    let workspace = workspace()?;
    let snapshot = workspace.snapshot();
    let query = workspace.execute_query(OmenaSdkQueryRequestV0 {
        snapshot_id: snapshot.snapshot_id,
        query_kind: "styleSummary".to_string(),
        input: Some(serde_json::json!({ "stylePath": "src/card.module.scss" })),
    })?;
    let diagnostics = workspace.execute_diagnostics(OmenaSdkDiagnosticsRequestV0 {
        snapshot_id: snapshot.snapshot_id,
        style_path: "src/card.module.scss".to_string(),
        style_source: ".card { --tone: red; color: var(--tone); }".to_string(),
    })?;
    let build = workspace.execute_build(OmenaSdkBuildRequestV0 {
        snapshot_id: snapshot.snapshot_id,
        style_path: "src/card.module.scss".to_string(),
        style_source: ".card { --tone: red; color: var(--tone); }".to_string(),
        pass_ids: vec!["whitespace-normalize".to_string()],
        context: None,
    })?;
    let explain = workspace.execute_explain(OmenaSdkExplainRequestV0 {
        snapshot_id: snapshot.snapshot_id,
        style_path: "src/card.module.scss".to_string(),
        position: OmenaSdkExplainPositionV0 {
            line: 0,
            character: 9,
        },
    })?;

    assert_eq!(query.snapshot_id, snapshot.snapshot_id);
    assert_eq!(diagnostics.snapshot_id, snapshot.snapshot_id);
    assert_eq!(build.snapshot_id, snapshot.snapshot_id);
    assert_eq!(explain.snapshot_id, snapshot.snapshot_id);
    assert_eq!(query.payload["language"], "scss");
    assert!(build.summary["sourceMapV3"]["sources"].is_array());
    assert_eq!(
        explain.report["sourceIdentity"]["originalSource"],
        "src/card.module.scss"
    );
    Ok(())
}

#[test]
fn workspace_runtime_advances_only_for_changed_sources() -> Result<(), OmenaError> {
    let mut workspace = workspace()?;
    let initial = workspace.snapshot();
    let unchanged = workspace.replace_style_sources([OmenaQueryStyleSourceInputV0 {
        style_path: "src/card.module.scss".to_string(),
        style_source: ".card { --tone: red; color: var(--tone); }".to_string(),
    }])?;
    assert_eq!(unchanged.snapshot_id, initial.snapshot_id);

    let changed = workspace.replace_style_sources([OmenaQueryStyleSourceInputV0 {
        style_path: "src/card.module.scss".to_string(),
        style_source: ".card { color: blue; }".to_string(),
    }])?;
    assert_ne!(changed.snapshot_id, initial.snapshot_id);
    let stale = workspace.execute_query(OmenaSdkQueryRequestV0 {
        snapshot_id: initial.snapshot_id,
        query_kind: "styleSummary".to_string(),
        input: Some(serde_json::json!({ "stylePath": "src/card.module.scss" })),
    });
    let error = stale
        .err()
        .ok_or_else(|| OmenaError::unknown("stale query succeeded", "test.unexpected-success"))?;
    assert_eq!(error.class, OmenaErrorClassV0::Workspace);
    Ok(())
}

#[test]
fn workspace_runtime_normalizes_empty_style_paths() -> Result<(), OmenaError> {
    let workspace = OmenaSdkWorkspaceV0::open(
        OmenaSdkSnapshotRequestV0 {
            workspace_root: "/workspace".to_string(),
        },
        [OmenaQueryStyleSourceInputV0 {
            style_path: String::new(),
            style_source: ".root { color: red; }".to_string(),
        }],
    )?;
    let response = workspace.execute_diagnostics(OmenaSdkDiagnosticsRequestV0 {
        snapshot_id: workspace.snapshot_id(),
        style_path: String::new(),
        style_source: ".root { color: red; }".to_string(),
    })?;
    assert_eq!(response.summary.style_path, "style.css");
    Ok(())
}
