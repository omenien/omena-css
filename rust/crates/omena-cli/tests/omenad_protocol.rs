use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use omena_cli::daemon::{OmenadClientV0, read_omenad_endpoint};
use omena_query::{
    OmenaQueryStyleSourceInputV0, OmenaSdkDiagnosticsRequestV0,
    OmenaWorkspaceSessionHandshakeRequestV0, OmenaWorkspaceSessionLimitsV0,
    OmenaWorkspaceSessionOperationV0, OmenaWorkspaceSessionRequestV0, OmenaWorkspaceSnapshotIdV0,
};

const TEST_LIMITS: OmenaWorkspaceSessionLimitsV0 = OmenaWorkspaceSessionLimitsV0 {
    deadline_ms: 30_000,
    max_response_bytes: 1_048_576,
};

#[test]
fn spawned_daemon_preserves_snapshot_and_enforces_session_boundaries() -> Result<(), String> {
    let root = temp_dir("resident-session");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let style_path = root.join("app.css");
    let source = ".app { color: red; }";
    fs::write(&style_path, source).map_err(|error| error.to_string())?;
    let endpoint_path = root.join("omenad.endpoint.json");
    let mut daemon = spawn_daemon(&endpoint_path)?;
    let endpoint = wait_for_endpoint(&endpoint_path, &mut daemon)?;

    let initial_handshake = handshake(
        &root,
        Some("config-a"),
        vec![style_source(&style_path, source)],
    );
    let (mut client, opened) = OmenadClientV0::connect(&endpoint, &initial_handshake)?;
    assert_eq!(opened.snapshot_id.value, 1);
    assert!(
        opened
            .capabilities
            .iter()
            .any(|value| value == "diagnostics")
    );

    let diagnostics = client.request(&request(
        "diagnostics-1",
        opened.snapshot_id,
        OmenaWorkspaceSessionOperationV0::Diagnostics,
        serde_json::to_value(OmenaSdkDiagnosticsRequestV0 {
            snapshot_id: opened.snapshot_id,
            style_path: style_path.to_string_lossy().into_owned(),
            style_source: source.to_string(),
        })
        .map_err(|error| error.to_string())?,
        TEST_LIMITS,
    ))?;
    assert!(diagnostics.ok);
    assert_eq!(diagnostics.snapshot_id, opened.snapshot_id);
    assert_eq!(
        diagnostics
            .payload
            .as_ref()
            .and_then(|value| value.pointer("/summary/classSelectorCount")),
        Some(&serde_json::json!(1))
    );

    let reconnect = handshake(&root, Some("config-a"), Vec::new());
    let (mut second_client, reconnected) = OmenadClientV0::connect(&endpoint, &reconnect)?;
    assert_eq!(reconnected.snapshot_id, opened.snapshot_id);
    let wrong_workspace = handshake(&root.join("other"), Some("config-a"), Vec::new());
    let pin_error = OmenadClientV0::connect(&endpoint, &wrong_workspace)
        .err()
        .ok_or_else(|| "workspace pin mismatch unexpectedly connected".to_string())?;
    assert!(pin_error.contains("pinned to a different workspace"));

    let cancel = second_client.request(&request(
        "cancel-1",
        opened.snapshot_id,
        OmenaWorkspaceSessionOperationV0::Cancel,
        serde_json::json!({ "requestId": "cancelled-check" }),
        TEST_LIMITS,
    ))?;
    assert!(cancel.ok);
    let cancelled = client.request(&request(
        "cancelled-check",
        opened.snapshot_id,
        OmenaWorkspaceSessionOperationV0::Check,
        serde_json::json!({ "stylePath": style_path }),
        TEST_LIMITS,
    ))?;
    assert!(!cancelled.ok);
    assert_eq!(
        cancelled
            .error
            .as_ref()
            .map(|error| error.context.code.as_str()),
        Some("daemon.request-cancelled")
    );

    let replacement_source = ".app { color: blue; }";
    let replaced = client.request(&request(
        "replace-1",
        opened.snapshot_id,
        OmenaWorkspaceSessionOperationV0::ReplaceStyleSources,
        serde_json::to_value(vec![style_source(&style_path, replacement_source)])
            .map_err(|error| error.to_string())?,
        TEST_LIMITS,
    ))?;
    assert!(replaced.ok);
    assert_eq!(replaced.snapshot_id.value, 2);

    let stale = client.request(&request(
        "stale-check",
        opened.snapshot_id,
        OmenaWorkspaceSessionOperationV0::Check,
        serde_json::json!({ "stylePath": style_path }),
        TEST_LIMITS,
    ))?;
    assert!(!stale.ok);
    assert_eq!(stale.snapshot_id.value, 2);
    assert_eq!(
        stale
            .error
            .as_ref()
            .map(|error| error.context.code.as_str()),
        Some("workspace.snapshot-mismatch")
    );

    let budgeted = client.request(&request(
        "budget-check",
        replaced.snapshot_id,
        OmenaWorkspaceSessionOperationV0::Check,
        serde_json::json!({ "stylePath": style_path }),
        OmenaWorkspaceSessionLimitsV0 {
            deadline_ms: 30_000,
            max_response_bytes: 1,
        },
    ))?;
    assert!(!budgeted.ok);
    assert_eq!(
        budgeted
            .error
            .as_ref()
            .map(|error| error.context.code.as_str()),
        Some("daemon.response-budget-exceeded")
    );

    let large_source = ".item { color: red; }\n".repeat(150_000);
    fs::write(&style_path, large_source).map_err(|error| error.to_string())?;
    let reconnect_for_cancellation = handshake(&root, Some("config-a"), Vec::new());
    let (mut long_client, long_session) =
        OmenadClientV0::connect(&endpoint, &reconnect_for_cancellation)?;
    let long_path = style_path.clone();
    let long_request = thread::spawn(move || {
        long_client.request(&request(
            "long-format",
            long_session.snapshot_id,
            OmenaWorkspaceSessionOperationV0::Format,
            serde_json::json!({ "path": long_path, "mode": "pretty" }),
            TEST_LIMITS,
        ))
    });
    thread::sleep(Duration::from_millis(10));
    let cancel_in_flight = second_client.request(&request(
        "cancel-2",
        replaced.snapshot_id,
        OmenaWorkspaceSessionOperationV0::Cancel,
        serde_json::json!({ "requestId": "long-format" }),
        TEST_LIMITS,
    ))?;
    assert!(cancel_in_flight.ok);
    let cancelled_in_flight = long_request
        .join()
        .map_err(|_| "long-running daemon request panicked".to_string())??;
    assert!(!cancelled_in_flight.ok);
    assert_eq!(
        cancelled_in_flight
            .error
            .as_ref()
            .map(|error| error.context.code.as_str()),
        Some("daemon.request-cancelled")
    );

    let deadline = client.request(&request(
        "deadline-format",
        replaced.snapshot_id,
        OmenaWorkspaceSessionOperationV0::Format,
        serde_json::json!({ "path": style_path, "mode": "pretty" }),
        OmenaWorkspaceSessionLimitsV0 {
            deadline_ms: 1,
            max_response_bytes: 1_048_576,
        },
    ))?;
    assert!(!deadline.ok);
    assert_eq!(
        deadline
            .error
            .as_ref()
            .map(|error| error.context.code.as_str()),
        Some("daemon.deadline-exceeded")
    );

    let shutdown = client.request(&request(
        "shutdown-1",
        replaced.snapshot_id,
        OmenaWorkspaceSessionOperationV0::Shutdown,
        serde_json::Value::Null,
        TEST_LIMITS,
    ))?;
    assert!(shutdown.ok);
    wait_for_exit(&mut daemon)?;
    assert!(!endpoint_path.exists());
    fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn idle_daemon_removes_its_endpoint() -> Result<(), String> {
    let root = temp_dir("idle-timeout");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let endpoint_path = root.join("omenad.endpoint.json");
    let mut daemon = Command::new(env!("CARGO_BIN_EXE_omenad"))
        .args([
            "--endpoint-file",
            endpoint_path.to_string_lossy().as_ref(),
            "--idle-timeout-ms",
            "50",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to spawn idle omenad: {error}"))?;
    let _ = wait_for_endpoint(&endpoint_path, &mut daemon)?;
    wait_for_exit(&mut daemon)?;
    assert!(!endpoint_path.exists());
    fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    Ok(())
}

fn spawn_daemon(endpoint_path: &Path) -> Result<Child, String> {
    Command::new(env!("CARGO_BIN_EXE_omenad"))
        .args([
            "--endpoint-file",
            endpoint_path.to_string_lossy().as_ref(),
            "--idle-timeout-ms",
            "30000",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to spawn omenad: {error}"))
}

fn wait_for_endpoint(
    endpoint_path: &Path,
    daemon: &mut Child,
) -> Result<omena_cli::daemon::OmenadEndpointV0, String> {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if endpoint_path.is_file() {
            return read_omenad_endpoint(endpoint_path);
        }
        if let Some(status) = daemon.try_wait().map_err(|error| error.to_string())? {
            return Err(format!(
                "omenad exited before publishing its endpoint: {status}"
            ));
        }
        thread::sleep(Duration::from_millis(10));
    }
    Err("omenad did not publish its endpoint before the test deadline".to_string())
}

fn wait_for_exit(daemon: &mut Child) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if let Some(status) = daemon.try_wait().map_err(|error| error.to_string())? {
            if status.success() {
                return Ok(());
            }
            return Err(format!("omenad exited unsuccessfully: {status}"));
        }
        thread::sleep(Duration::from_millis(10));
    }
    let _ = daemon.kill();
    Err("omenad did not stop after shutdown".to_string())
}

fn handshake(
    root: &Path,
    config_content_digest: Option<&str>,
    style_sources: Vec<OmenaQueryStyleSourceInputV0>,
) -> OmenaWorkspaceSessionHandshakeRequestV0 {
    OmenaWorkspaceSessionHandshakeRequestV0 {
        protocol_version: "0".to_string(),
        workspace_root: root.to_string_lossy().into_owned(),
        config_content_digest: config_content_digest.map(str::to_string),
        style_sources,
        limits: TEST_LIMITS,
    }
}

fn request(
    request_id: &str,
    snapshot_id: OmenaWorkspaceSnapshotIdV0,
    operation: OmenaWorkspaceSessionOperationV0,
    payload: serde_json::Value,
    limits: OmenaWorkspaceSessionLimitsV0,
) -> OmenaWorkspaceSessionRequestV0 {
    OmenaWorkspaceSessionRequestV0 {
        request_id: request_id.to_string(),
        protocol_version: "0".to_string(),
        snapshot_id,
        operation,
        limits,
        payload: Some(payload),
    }
}

fn style_source(path: &Path, source: &str) -> OmenaQueryStyleSourceInputV0 {
    OmenaQueryStyleSourceInputV0 {
        style_path: path.to_string_lossy().into_owned(),
        style_source: source.to_string(),
    }
}

fn temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!("omena-{label}-{}-{nonce}", std::process::id()))
}
