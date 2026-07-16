use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::mpsc,
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
    let source = ".app {\n  color: red;\n}\n";
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

    let parity_cases = [
        (
            "check",
            OmenaWorkspaceSessionOperationV0::Check,
            serde_json::json!({ "stylePath": style_path }),
            vec![
                "check".to_string(),
                style_path.to_string_lossy().into_owned(),
                "--json".to_string(),
            ],
        ),
        (
            "lint",
            OmenaWorkspaceSessionOperationV0::Lint,
            serde_json::json!({ "root": root }),
            vec![
                "lint".to_string(),
                root.to_string_lossy().into_owned(),
                "--json".to_string(),
            ],
        ),
        (
            "format",
            OmenaWorkspaceSessionOperationV0::Format,
            serde_json::json!({ "path": style_path, "mode": "pretty" }),
            vec![
                "fmt".to_string(),
                style_path.to_string_lossy().into_owned(),
                "--mode".to_string(),
                "pretty".to_string(),
                "--check".to_string(),
                "--json".to_string(),
            ],
        ),
        (
            "explain",
            OmenaWorkspaceSessionOperationV0::Explain,
            serde_json::json!({
                "cliRequest": {
                    "kind": "cascade",
                    "path": style_path,
                    "line": 1,
                    "character": 8
                }
            }),
            vec![
                "explain".to_string(),
                "cascade".to_string(),
                style_path.to_string_lossy().into_owned(),
                "--line".to_string(),
                "1".to_string(),
                "--character".to_string(),
                "8".to_string(),
                "--json".to_string(),
            ],
        ),
    ];
    for (name, operation, payload, direct_args) in parity_cases {
        let resident = client.request(&request(
            format!("parity-{name}").as_str(),
            opened.snapshot_id,
            operation,
            payload,
            TEST_LIMITS,
        ))?;
        assert!(
            resident.ok,
            "resident {name} operation failed: {resident:?}"
        );
        assert_eq!(
            resident.payload,
            Some(direct_payload(direct_args.as_slice())?),
            "resident and direct {name} payloads diverged"
        );
    }

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

#[test]
fn watch_command_falls_back_when_the_resident_process_stops() -> Result<(), String> {
    let root = temp_dir("watch-fallback");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let style_path = root.join("app.css");
    fs::write(&style_path, ".app {\n  color: red;\n}\n").map_err(|error| error.to_string())?;
    let endpoint_path = root.join("watch.endpoint.json");
    let mut watch = Command::new(env!("CARGO_BIN_EXE_omena"))
        .args([
            "check",
            style_path.to_string_lossy().as_ref(),
            "--watch",
            "--json",
        ])
        .env("OMENA_DAEMON_BIN", env!("CARGO_BIN_EXE_omenad"))
        .env("OMENA_DAEMON_ENDPOINT_FILE", &endpoint_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| format!("failed to spawn watch command: {error}"))?;
    let stdout = watch
        .stdout
        .take()
        .ok_or_else(|| "watch command stdout was not piped".to_string())?;
    let (sender, receiver) = mpsc::sync_channel(4);
    let reader = thread::spawn(move || {
        let mut reader = std::io::BufReader::new(stdout);
        loop {
            let mut line = String::new();
            match std::io::BufRead::read_line(&mut reader, &mut line) {
                Ok(0) | Err(_) => return,
                Ok(_) => {
                    if sender.send(line).is_err() {
                        return;
                    }
                }
            }
        }
    });

    let first = receiver
        .recv_timeout(Duration::from_secs(15))
        .map_err(|error| format!("watch command produced no initial result: {error}"))?;
    let first: serde_json::Value =
        serde_json::from_str(&first).map_err(|error| error.to_string())?;
    assert_eq!(first["route"], "daemon");
    let endpoint = read_omenad_endpoint(&endpoint_path)?;
    terminate_process(endpoint.process_id)?;

    fs::write(
        &style_path,
        ".app {\n  color: blue;\n}\n.panel {\n  display: block;\n}\n",
    )
    .map_err(|error| error.to_string())?;
    let second = receiver
        .recv_timeout(Duration::from_secs(15))
        .map_err(|error| format!("watch command produced no fallback result: {error}"))?;
    let second: serde_json::Value =
        serde_json::from_str(&second).map_err(|error| error.to_string())?;
    assert_eq!(second["route"], "directFallback");
    assert_eq!(
        second["payload"],
        direct_payload(&[
            "check".to_string(),
            style_path.to_string_lossy().into_owned(),
            "--json".to_string(),
        ])?
    );

    let _ = watch.kill();
    let _ = watch.wait();
    drop(receiver);
    let _ = reader.join();
    fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn watch_command_respects_the_disabled_workspace_session_route() -> Result<(), String> {
    let root = temp_dir("watch-session-disabled");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let style_path = root.join("app.css");
    fs::write(&style_path, ".app {\n  color: red;\n}\n").map_err(|error| error.to_string())?;
    fs::write(
        root.join("omena.toml"),
        "[workspace.session]\nenabled = false\nidleTimeoutMs = 5000\nrequestDeadlineMs = 1000\nmaxResponseBytes = 1048576\n",
    )
    .map_err(|error| error.to_string())?;
    let endpoint_path = root.join("disabled.endpoint.json");
    let mut watch = Command::new(env!("CARGO_BIN_EXE_omena"))
        .args([
            "check",
            style_path.to_string_lossy().as_ref(),
            "--watch",
            "--json",
        ])
        .env("OMENA_DAEMON_BIN", env!("CARGO_BIN_EXE_omenad"))
        .env("OMENA_DAEMON_ENDPOINT_FILE", &endpoint_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| format!("failed to spawn disabled-session watch command: {error}"))?;
    let stdout = watch
        .stdout
        .take()
        .ok_or_else(|| "watch command stdout was not piped".to_string())?;
    let (sender, receiver) = mpsc::sync_channel(1);
    let reader = thread::spawn(move || {
        let mut reader = std::io::BufReader::new(stdout);
        let mut line = String::new();
        if std::io::BufRead::read_line(&mut reader, &mut line).is_ok() {
            let _ = sender.send(line);
        }
    });

    let first = receiver
        .recv_timeout(Duration::from_secs(15))
        .map_err(|error| format!("watch command produced no initial result: {error}"))?;
    let first: serde_json::Value =
        serde_json::from_str(&first).map_err(|error| error.to_string())?;
    assert_eq!(first["route"], "directFallback");
    assert_eq!(first["snapshotId"], serde_json::Value::Null);
    assert!(!endpoint_path.exists());

    let _ = watch.kill();
    let _ = watch.wait();
    let _ = reader.join();
    fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn watch_command_rejects_a_session_budget_above_the_transport_limit() -> Result<(), String> {
    let root = temp_dir("watch-session-budget");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let style_path = root.join("app.css");
    fs::write(&style_path, ".app { color: red; }\n").map_err(|error| error.to_string())?;
    fs::write(
        root.join("omena.toml"),
        "[workspace.session]\nmaxResponseBytes = 16777217\n",
    )
    .map_err(|error| error.to_string())?;
    let endpoint_path = root.join("rejected.endpoint.json");
    let output = Command::new(env!("CARGO_BIN_EXE_omena"))
        .args([
            "check",
            style_path.to_string_lossy().as_ref(),
            "--watch",
            "--json",
        ])
        .env("OMENA_DAEMON_BIN", env!("CARGO_BIN_EXE_omenad"))
        .env("OMENA_DAEMON_ENDPOINT_FILE", &endpoint_path)
        .output()
        .map_err(|error| format!("failed to run invalid-budget watch command: {error}"))?;
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("workspace session maxResponseBytes must not exceed 16777216")
    );
    assert!(!endpoint_path.exists());
    fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn spawned_watch_daemon_survives_its_initial_client() -> Result<(), String> {
    let root = temp_dir("watch-resident-lifetime");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let style_path = root.join("app.css");
    fs::write(&style_path, ".app {\n  color: red;\n}\n").map_err(|error| error.to_string())?;
    fs::write(
        root.join("omena.toml"),
        "[workspace.session]\nidleTimeoutMs = 30000\nrequestDeadlineMs = 1000\nmaxResponseBytes = 1048576\n",
    )
    .map_err(|error| error.to_string())?;
    let endpoint_path = root.join("resident.endpoint.json");

    let mut first = spawn_watch_command(&style_path, &endpoint_path)?;
    assert_eq!(read_watch_result(&mut first)?["route"], "daemon");
    let first_endpoint = read_omenad_endpoint(&endpoint_path)?;
    first.kill().map_err(|error| error.to_string())?;
    first.wait().map_err(|error| error.to_string())?;

    fs::write(
        &style_path,
        ".app {\n  color: blue;\n}\n.panel {\n  display: block;\n}\n",
    )
    .map_err(|error| error.to_string())?;
    let mut second = spawn_watch_command(&style_path, &endpoint_path)?;
    let second_result = read_watch_result(&mut second)?;
    assert_eq!(second_result["route"], "daemon");
    assert_eq!(
        second_result["payload"],
        direct_payload(&[
            "check".to_string(),
            style_path.to_string_lossy().into_owned(),
            "--json".to_string(),
        ])?
    );
    let second_endpoint = read_omenad_endpoint(&endpoint_path)?;
    assert_eq!(second_endpoint.process_id, first_endpoint.process_id);

    let _ = second.kill();
    let _ = second.wait();
    terminate_process(first_endpoint.process_id)?;
    let _ = fs::remove_file(&endpoint_path);
    fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    Ok(())
}

fn spawn_watch_command(style_path: &Path, endpoint_path: &Path) -> Result<Child, String> {
    Command::new(env!("CARGO_BIN_EXE_omena"))
        .args([
            "check",
            style_path.to_string_lossy().as_ref(),
            "--watch",
            "--json",
        ])
        .env("OMENA_DAEMON_BIN", env!("CARGO_BIN_EXE_omenad"))
        .env("OMENA_DAEMON_ENDPOINT_FILE", endpoint_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| format!("failed to spawn watch command: {error}"))
}

fn read_watch_result(watch: &mut Child) -> Result<serde_json::Value, String> {
    let stdout = watch
        .stdout
        .take()
        .ok_or_else(|| "watch command stdout was not piped".to_string())?;
    let (sender, receiver) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let mut reader = std::io::BufReader::new(stdout);
        let mut line = String::new();
        if std::io::BufRead::read_line(&mut reader, &mut line).is_ok() {
            let _ = sender.send(line);
        }
    });
    let line = receiver
        .recv_timeout(Duration::from_secs(15))
        .map_err(|error| format!("watch command produced no initial result: {error}"))?;
    serde_json::from_str(&line).map_err(|error| error.to_string())
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

fn direct_payload(args: &[String]) -> Result<serde_json::Value, String> {
    let output = Command::new(env!("CARGO_BIN_EXE_omena"))
        .args(args)
        .output()
        .map_err(|error| format!("failed to run direct omena command: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "direct omena command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let envelope: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("failed to decode direct omena output: {error}"))?;
    envelope
        .get("payload")
        .cloned()
        .ok_or_else(|| "direct omena output omitted its payload".to_string())
}

#[cfg(unix)]
fn terminate_process(process_id: u32) -> Result<(), String> {
    let status = Command::new("kill")
        .args(["-9", process_id.to_string().as_str()])
        .status()
        .map_err(|error| format!("failed to terminate omenad: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("kill exited with {status}"))
    }
}

#[cfg(windows)]
fn terminate_process(process_id: u32) -> Result<(), String> {
    let status = Command::new("taskkill")
        .args(["/PID", process_id.to_string().as_str(), "/F"])
        .status()
        .map_err(|error| format!("failed to terminate omenad: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("taskkill exited with {status}"))
    }
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
