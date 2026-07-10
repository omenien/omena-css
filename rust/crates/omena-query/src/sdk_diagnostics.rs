use crate::{
    OmenaError, OmenaErrorClassV0, OmenaErrorContextV0, OmenaErrorRecoverabilityV0,
    OmenaErrorSeverityV0, OmenaSdkDiagnosticsRequestV0, OmenaSdkDiagnosticsResponseV0,
    OmenaSdkDiagnosticsSummaryV0, OmenaSdkResponsePartitionV0, OmenaWorkspaceSnapshotIdV0,
    summarize_omena_query_consumer_check_style_source,
};

pub fn execute_omena_sdk_diagnostics_workflow(
    request: OmenaSdkDiagnosticsRequestV0,
    read_snapshot_id: OmenaWorkspaceSnapshotIdV0,
) -> Result<OmenaSdkDiagnosticsResponseV0, OmenaError> {
    if request.snapshot_id != read_snapshot_id {
        return Err(OmenaError::new(
            OmenaErrorClassV0::Workspace,
            "diagnostics request does not match the current workspace snapshot",
            OmenaErrorContextV0 {
                code: "workspace.snapshot-mismatch".to_string(),
                severity: OmenaErrorSeverityV0::Error,
                recoverability: OmenaErrorRecoverabilityV0::Retry,
            },
        ));
    }

    let summary = summarize_omena_query_consumer_check_style_source(
        &request.style_path,
        &request.style_source,
    );
    Ok(OmenaSdkDiagnosticsResponseV0 {
        snapshot_id: read_snapshot_id,
        partition: OmenaSdkResponsePartitionV0::Public,
        summary: OmenaSdkDiagnosticsSummaryV0 {
            schema_version: summary.schema_version.to_string(),
            product: summary.product.to_string(),
            style_path: summary.style_path,
            dialect: summary.dialect.to_string(),
            token_count: count_to_u64(summary.token_count)?,
            parser_error_count: count_to_u64(summary.parser_error_count)?,
            class_selector_count: count_to_u64(summary.class_selector_count)?,
            custom_property_count: count_to_u64(summary.custom_property_count)?,
            keyframe_count: count_to_u64(summary.keyframe_count)?,
            ready_surfaces: summary
                .ready_surfaces
                .into_iter()
                .map(str::to_string)
                .collect(),
        },
    })
}

fn count_to_u64(value: usize) -> Result<u64, OmenaError> {
    u64::try_from(value).map_err(|_| {
        OmenaError::new(
            OmenaErrorClassV0::Internal,
            "diagnostics count exceeds the workflow contract range",
            OmenaErrorContextV0 {
                code: "diagnostics.count-range".to_string(),
                severity: OmenaErrorSeverityV0::Error,
                recoverability: OmenaErrorRecoverabilityV0::NotRecoverable,
            },
        )
    })
}
