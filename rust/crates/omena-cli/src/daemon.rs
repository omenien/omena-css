use std::{
    collections::BTreeSet,
    fs,
    io::{BufRead, BufReader, ErrorKind, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::ExitCode,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

use clap::Parser;
use omena_query::{
    OMENA_WORKSPACE_SESSION_PROTOCOL_VERSION_V0, OmenaError, OmenaErrorClassV0,
    OmenaErrorContextV0, OmenaErrorRecoverabilityV0, OmenaErrorSeverityV0,
    OmenaQueryStyleSourceInputV0, OmenaSdkDiagnosticsRequestV0, OmenaSdkExplainRequestV0,
    OmenaSdkQueryRequestV0, OmenaSdkSnapshotRequestV0, OmenaSdkWorkspaceV0,
    OmenaWorkspaceSessionHandshakeRequestV0, OmenaWorkspaceSessionHandshakeResponseV0,
    OmenaWorkspaceSessionOperationV0, OmenaWorkspaceSessionRequestV0,
    OmenaWorkspaceSessionResponseV0, negotiate_omena_workspace_session_v0,
    omena_workspace_session_failure_v0, omena_workspace_session_success_v0,
};
use serde::{Deserialize, Serialize};

use crate::{
    commands::{FormatMode, LintProfile},
    format::build_format_report,
    lint::lint_report,
};

const DEFAULT_IDLE_TIMEOUT_MS: u64 = 300_000;
const MAX_TRANSPORT_LINE_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Parser)]
#[command(
    name = "omenad",
    about = "Host one resident Omena workspace session for local clients"
)]
pub struct OmenadArgs {
    /// File used to publish the selected loopback endpoint.
    #[arg(long)]
    endpoint_file: PathBuf,
    /// Stop after this many milliseconds without a protocol message.
    #[arg(long, default_value_t = DEFAULT_IDLE_TIMEOUT_MS)]
    idle_timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenadEndpointV0 {
    pub protocol_version: String,
    pub address: String,
    pub process_id: u32,
}

pub struct OmenadClientV0 {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
}

impl OmenadClientV0 {
    pub fn connect(
        endpoint: &OmenadEndpointV0,
        handshake: &OmenaWorkspaceSessionHandshakeRequestV0,
    ) -> Result<(Self, OmenaWorkspaceSessionHandshakeResponseV0), String> {
        if endpoint.protocol_version != OMENA_WORKSPACE_SESSION_PROTOCOL_VERSION_V0 {
            return Err("omenad endpoint protocol version is unsupported".to_string());
        }
        let mut stream = TcpStream::connect(endpoint.address.as_str())
            .map_err(|error| format!("failed to connect to omenad: {error}"))?;
        stream
            .set_read_timeout(Some(Duration::from_secs(30)))
            .map_err(|error| format!("failed to configure omenad client: {error}"))?;
        let reader = BufReader::new(
            stream
                .try_clone()
                .map_err(|error| format!("failed to clone omenad client stream: {error}"))?,
        );
        write_wire_value(&mut stream, handshake)?;
        let mut client = Self { stream, reader };
        let response = client.read_wire_value::<serde_json::Value>()?;
        if let Some(error) = response.get("error") {
            return Err(format!("omenad handshake failed: {error}"));
        }
        let response = serde_json::from_value(response)
            .map_err(|error| format!("failed to decode omenad handshake response: {error}"))?;
        Ok((client, response))
    }

    pub fn request(
        &mut self,
        request: &OmenaWorkspaceSessionRequestV0,
    ) -> Result<OmenaWorkspaceSessionResponseV0, String> {
        write_wire_value(&mut self.stream, request)?;
        self.read_wire_value()
    }

    fn read_wire_value<T: serde::de::DeserializeOwned>(&mut self) -> Result<T, String> {
        let mut line = String::new();
        self.reader
            .read_line(&mut line)
            .map_err(|error| format!("failed to read omenad response: {error}"))?;
        if line.is_empty() {
            return Err("omenad closed the connection without a response".to_string());
        }
        serde_json::from_str(&line).map_err(|error| {
            format!(
                "failed to decode omenad response: {error}; wire={}",
                line.trim_end()
            )
        })
    }
}

pub fn read_omenad_endpoint(path: &Path) -> Result<OmenadEndpointV0, String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("failed to read omenad endpoint {}: {error}", path.display()))?;
    serde_json::from_str(&source).map_err(|error| {
        format!(
            "failed to decode omenad endpoint {}: {error}",
            path.display()
        )
    })
}

#[derive(Debug)]
struct ResidentWorkspaceSession {
    workspace_root: String,
    config_content_digest: Option<String>,
    workspace: OmenaSdkWorkspaceV0,
}

struct OmenadState {
    session: Mutex<Option<ResidentWorkspaceSession>>,
    cancelled_request_ids: Mutex<BTreeSet<String>>,
    last_activity: Mutex<Instant>,
    shutdown: AtomicBool,
}

impl OmenadState {
    fn new() -> Self {
        Self {
            session: Mutex::new(None),
            cancelled_request_ids: Mutex::new(BTreeSet::new()),
            last_activity: Mutex::new(Instant::now()),
            shutdown: AtomicBool::new(false),
        }
    }

    fn touch(&self) {
        if let Ok(mut last_activity) = self.last_activity.lock() {
            *last_activity = Instant::now();
        }
    }

    fn idle_for(&self) -> Duration {
        self.last_activity
            .lock()
            .map_or(Duration::ZERO, |last_activity| last_activity.elapsed())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FormatRequestPayloadV0 {
    path: Option<PathBuf>,
    mode: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LintRequestPayloadV0 {
    root: Option<PathBuf>,
    profile: Option<String>,
    stylelint_config: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CheckRequestPayloadV0 {
    style_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CancelRequestPayloadV0 {
    request_id: String,
}

pub fn run_omenad_from_env() -> ExitCode {
    match run_omenad(OmenadArgs::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

pub fn run_omenad(args: OmenadArgs) -> Result<(), String> {
    if args.idle_timeout_ms == 0 {
        return Err("omenad idle timeout must be positive".to_string());
    }
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|error| format!("failed to bind omenad loopback endpoint: {error}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("failed to configure omenad endpoint: {error}"))?;
    let endpoint = OmenadEndpointV0 {
        protocol_version: OMENA_WORKSPACE_SESSION_PROTOCOL_VERSION_V0.to_string(),
        address: listener
            .local_addr()
            .map_err(|error| format!("failed to read omenad endpoint: {error}"))?
            .to_string(),
        process_id: std::process::id(),
    };
    write_endpoint(args.endpoint_file.as_path(), &endpoint)?;

    let state = Arc::new(OmenadState::new());
    let idle_timeout = Duration::from_millis(args.idle_timeout_ms);
    let mut connections = Vec::new();
    while !state.shutdown.load(Ordering::Acquire) && state.idle_for() < idle_timeout {
        match listener.accept() {
            Ok((stream, _)) => {
                let state = Arc::clone(&state);
                connections.push(thread::spawn(move || serve_connection(stream, state)));
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => {
                state.shutdown.store(true, Ordering::Release);
                cleanup_endpoint(args.endpoint_file.as_path());
                return Err(format!("omenad accept failed: {error}"));
            }
        }
    }
    state.shutdown.store(true, Ordering::Release);
    for connection in connections {
        let _ = connection.join();
    }
    cleanup_endpoint(args.endpoint_file.as_path());
    Ok(())
}

fn write_endpoint(path: &Path, endpoint: &OmenadEndpointV0) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create omenad endpoint directory {}: {error}",
                parent.display()
            )
        })?;
    }
    let encoded = serde_json::to_vec(endpoint)
        .map_err(|error| format!("failed to encode omenad endpoint: {error}"))?;
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    fs::write(&temporary, encoded).map_err(|error| {
        format!(
            "failed to write omenad endpoint {}: {error}",
            temporary.display()
        )
    })?;
    fs::rename(&temporary, path).map_err(|error| {
        format!(
            "failed to publish omenad endpoint {}: {error}",
            path.display()
        )
    })
}

fn cleanup_endpoint(path: &Path) {
    match fs::remove_file(path) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(_) => {}
    }
}

fn serve_connection(mut stream: TcpStream, state: Arc<OmenadState>) {
    let _ = stream.set_read_timeout(Some(Duration::from_millis(100)));
    let reader_stream = match stream.try_clone() {
        Ok(stream) => stream,
        Err(_) => return,
    };
    let mut reader = BufReader::new(reader_stream);
    let mut handshaken = false;
    let mut line = String::new();
    loop {
        if state.shutdown.load(Ordering::Acquire) {
            return;
        }
        match reader.read_line(&mut line) {
            Ok(0) => return,
            Ok(_) => {}
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                continue;
            }
            Err(_) => return,
        }
        state.touch();
        if line.len() > MAX_TRANSPORT_LINE_BYTES {
            let _ = write_wire_value(
                &mut stream,
                &error_envelope(daemon_error(
                    OmenaErrorClassV0::Input,
                    "omenad protocol message exceeds the transport limit",
                    "daemon.message-too-large",
                    OmenaErrorRecoverabilityV0::UserAction,
                )),
            );
            return;
        }
        let message = std::mem::take(&mut line);
        if !handshaken {
            match serde_json::from_str::<OmenaWorkspaceSessionHandshakeRequestV0>(&message)
                .map_err(|error| format!("invalid omenad handshake: {error}"))
                .and_then(|request| negotiate_handshake(&state, request))
            {
                Ok(response) => {
                    if write_wire_value(&mut stream, &response).is_err() {
                        return;
                    }
                    handshaken = true;
                }
                Err(error) => {
                    let _ = write_wire_value(
                        &mut stream,
                        &error_envelope(daemon_error(
                            OmenaErrorClassV0::Workspace,
                            error,
                            "daemon.handshake-restart-required",
                            OmenaErrorRecoverabilityV0::Retry,
                        )),
                    );
                    return;
                }
            }
            continue;
        }
        let request = match serde_json::from_str::<OmenaWorkspaceSessionRequestV0>(&message) {
            Ok(request) => request,
            Err(error) => {
                let _ = write_wire_value(
                    &mut stream,
                    &error_envelope(daemon_error(
                        OmenaErrorClassV0::Input,
                        format!("invalid omenad request: {error}"),
                        "daemon.request-parse",
                        OmenaErrorRecoverabilityV0::UserAction,
                    )),
                );
                continue;
            }
        };
        let response = execute_session_request(&state, request);
        if write_limited_response(&mut stream, response).is_err() {
            return;
        }
    }
}

fn negotiate_handshake(
    state: &OmenadState,
    request: OmenaWorkspaceSessionHandshakeRequestV0,
) -> Result<OmenaWorkspaceSessionHandshakeResponseV0, String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "omenad workspace session lock was poisoned".to_string())?;
    if let Some(session) = session.as_ref() {
        if request.workspace_root != session.workspace_root
            || request.config_content_digest != session.config_content_digest
        {
            return Err("omenad is pinned to a different workspace or config snapshot".to_string());
        }
        if !request.style_sources.is_empty() {
            return Err(
                "reconnecting clients must use replaceStyleSources after handshake".to_string(),
            );
        }
        return negotiate_omena_workspace_session_v0(&request, session.workspace.snapshot_id())
            .map_err(|error| error.to_string());
    }

    let workspace = OmenaSdkWorkspaceV0::open(
        OmenaSdkSnapshotRequestV0 {
            workspace_root: request.workspace_root.clone(),
        },
        request.style_sources.clone(),
    )
    .map_err(|error| error.to_string())?;
    let response = negotiate_omena_workspace_session_v0(&request, workspace.snapshot_id())
        .map_err(|error| error.to_string())?;
    *session = Some(ResidentWorkspaceSession {
        workspace_root: request.workspace_root,
        config_content_digest: request.config_content_digest,
        workspace,
    });
    Ok(response)
}

fn execute_session_request(
    state: &OmenadState,
    request: OmenaWorkspaceSessionRequestV0,
) -> (OmenaWorkspaceSessionResponseV0, u64) {
    let max_response_bytes = request.limits.max_response_bytes;
    let response = execute_session_request_inner(state, &request).unwrap_or_else(|error| {
        let snapshot_id = state
            .session
            .lock()
            .ok()
            .and_then(|session| {
                session
                    .as_ref()
                    .map(|session| session.workspace.snapshot_id())
            })
            .unwrap_or(request.snapshot_id);
        omena_workspace_session_failure_v0(request.request_id.clone(), snapshot_id, error)
    });
    (response, max_response_bytes)
}

fn execute_session_request_inner(
    state: &OmenadState,
    request: &OmenaWorkspaceSessionRequestV0,
) -> Result<OmenaWorkspaceSessionResponseV0, OmenaError> {
    if request.protocol_version != OMENA_WORKSPACE_SESSION_PROTOCOL_VERSION_V0 {
        return Err(daemon_error(
            OmenaErrorClassV0::Unsupported,
            "omenad request protocol version does not match the session",
            "daemon.protocol-version",
            OmenaErrorRecoverabilityV0::Retry,
        ));
    }
    if request.limits.deadline_ms == 0 || request.limits.max_response_bytes == 0 {
        return Err(daemon_error(
            OmenaErrorClassV0::Input,
            "omenad request limits must be positive",
            "daemon.invalid-limits",
            OmenaErrorRecoverabilityV0::UserAction,
        ));
    }

    if request.operation == OmenaWorkspaceSessionOperationV0::Cancel {
        let payload = parse_payload::<CancelRequestPayloadV0>(request, "cancel")?;
        state
            .cancelled_request_ids
            .lock()
            .map_err(|_| internal_lock_error("cancellation"))?
            .insert(payload.request_id.clone());
        return Ok(omena_workspace_session_success_v0(
            request.request_id.clone(),
            request.snapshot_id,
            serde_json::json!({ "cancelledRequestId": payload.request_id }),
        ));
    }

    if state
        .cancelled_request_ids
        .lock()
        .map_err(|_| internal_lock_error("cancellation"))?
        .remove(request.request_id.as_str())
    {
        return Err(daemon_error(
            OmenaErrorClassV0::Analysis,
            "omenad request was cancelled before execution",
            "daemon.request-cancelled",
            OmenaErrorRecoverabilityV0::Retry,
        ));
    }

    let mut session_guard = state
        .session
        .lock()
        .map_err(|_| internal_lock_error("workspace session"))?;
    let session = session_guard.as_mut().ok_or_else(|| {
        daemon_error(
            OmenaErrorClassV0::Workspace,
            "omenad request arrived before a workspace handshake",
            "daemon.handshake-required",
            OmenaErrorRecoverabilityV0::Retry,
        )
    })?;
    if request.snapshot_id != session.workspace.snapshot_id() {
        return Err(daemon_error(
            OmenaErrorClassV0::Workspace,
            "omenad request snapshot is stale",
            "workspace.snapshot-mismatch",
            OmenaErrorRecoverabilityV0::Retry,
        ));
    }

    match request.operation {
        OmenaWorkspaceSessionOperationV0::ReplaceStyleSources => {
            let replacement =
                parse_payload::<Vec<OmenaQueryStyleSourceInputV0>>(request, "replaceStyleSources")?;
            let payload =
                serde_json::to_value(session.workspace.replace_style_sources(replacement)?)
                    .map_err(serialization_error)?;
            return Ok(omena_workspace_session_success_v0(
                request.request_id.clone(),
                session.workspace.snapshot_id(),
                payload,
            ));
        }
        OmenaWorkspaceSessionOperationV0::Shutdown => {
            state.shutdown.store(true, Ordering::Release);
            return Ok(omena_workspace_session_success_v0(
                request.request_id.clone(),
                session.workspace.snapshot_id(),
                serde_json::json!({ "shutdown": true }),
            ));
        }
        OmenaWorkspaceSessionOperationV0::Cancel => {
            unreachable!("cancel is handled before locking the session")
        }
        OmenaWorkspaceSessionOperationV0::Diagnostics
        | OmenaWorkspaceSessionOperationV0::Format
        | OmenaWorkspaceSessionOperationV0::Lint
        | OmenaWorkspaceSessionOperationV0::Check
        | OmenaWorkspaceSessionOperationV0::Explain => {}
    }

    let workspace = session.workspace.clone();
    let response_snapshot_id = session.workspace.snapshot_id();
    drop(session_guard);

    let worker_request = request.clone();
    let (sender, receiver) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let _ = sender.send(execute_read_operation(workspace, &worker_request));
    });

    let deadline = Duration::from_millis(request.limits.deadline_ms);
    let started_at = Instant::now();
    let payload = loop {
        if state
            .cancelled_request_ids
            .lock()
            .map_err(|_| internal_lock_error("cancellation"))?
            .remove(request.request_id.as_str())
        {
            return Err(daemon_error(
                OmenaErrorClassV0::Analysis,
                "omenad request was cancelled during execution",
                "daemon.request-cancelled",
                OmenaErrorRecoverabilityV0::Retry,
            ));
        }

        let elapsed = started_at.elapsed();
        if elapsed >= deadline {
            return Err(daemon_error(
                OmenaErrorClassV0::Analysis,
                "omenad request exceeded its deadline",
                "daemon.deadline-exceeded",
                OmenaErrorRecoverabilityV0::Retry,
            ));
        }
        let wait = (deadline - elapsed).min(Duration::from_millis(10));
        match receiver.recv_timeout(wait) {
            Ok(result) => break result?,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(daemon_error(
                    OmenaErrorClassV0::Internal,
                    "omenad request worker stopped without a response",
                    "daemon.worker-disconnected",
                    OmenaErrorRecoverabilityV0::Retry,
                ));
            }
        }
    };

    Ok(omena_workspace_session_success_v0(
        request.request_id.clone(),
        response_snapshot_id,
        payload,
    ))
}

fn execute_read_operation(
    workspace: OmenaSdkWorkspaceV0,
    request: &OmenaWorkspaceSessionRequestV0,
) -> Result<serde_json::Value, OmenaError> {
    match request.operation {
        OmenaWorkspaceSessionOperationV0::Diagnostics => {
            serde_json::to_value(workspace.execute_diagnostics(parse_payload::<
                OmenaSdkDiagnosticsRequestV0,
            >(
                request, "diagnostics"
            )?)?)
            .map_err(serialization_error)
        }
        OmenaWorkspaceSessionOperationV0::Explain => {
            serde_json::to_value(workspace.execute_explain(parse_payload::<
                OmenaSdkExplainRequestV0,
            >(request, "explain")?)?)
            .map_err(serialization_error)
        }
        OmenaWorkspaceSessionOperationV0::Check => {
            let payload = parse_payload::<CheckRequestPayloadV0>(request, "check")?;
            let response = workspace.execute_query(OmenaSdkQueryRequestV0 {
                snapshot_id: request.snapshot_id,
                query_kind: "styleSummary".to_string(),
                input: Some(serde_json::json!({ "stylePath": payload.style_path })),
            })?;
            serde_json::to_value(response).map_err(serialization_error)
        }
        OmenaWorkspaceSessionOperationV0::Format => {
            let payload = parse_payload::<FormatRequestPayloadV0>(request, "format")?;
            let mode = parse_format_mode(payload.mode.as_deref())?;
            serde_json::to_value(
                build_format_report(payload.path, mode, true).map_err(operation_error)?,
            )
            .map_err(serialization_error)
        }
        OmenaWorkspaceSessionOperationV0::Lint => {
            let payload = parse_payload::<LintRequestPayloadV0>(request, "lint")?;
            let profile = parse_lint_profile(payload.profile.as_deref())?;
            serde_json::to_value(
                lint_report(payload.root, profile, payload.stylelint_config)
                    .map_err(operation_error)?,
            )
            .map_err(serialization_error)
        }
        OmenaWorkspaceSessionOperationV0::ReplaceStyleSources
        | OmenaWorkspaceSessionOperationV0::Cancel
        | OmenaWorkspaceSessionOperationV0::Shutdown => Err(daemon_error(
            OmenaErrorClassV0::Internal,
            "mutable omenad operation reached the read-only worker",
            "daemon.invalid-worker-operation",
            OmenaErrorRecoverabilityV0::NotRecoverable,
        )),
    }
}

fn parse_payload<T: serde::de::DeserializeOwned>(
    request: &OmenaWorkspaceSessionRequestV0,
    operation: &str,
) -> Result<T, OmenaError> {
    serde_json::from_value(request.payload.clone().unwrap_or(serde_json::Value::Null)).map_err(
        |error| {
            daemon_error(
                OmenaErrorClassV0::Input,
                format!("invalid {operation} payload: {error}"),
                "daemon.invalid-payload",
                OmenaErrorRecoverabilityV0::UserAction,
            )
        },
    )
}

fn parse_format_mode(mode: Option<&str>) -> Result<Option<FormatMode>, OmenaError> {
    match mode {
        None => Ok(None),
        Some("pretty") => Ok(Some(FormatMode::Pretty)),
        Some("stable") => Ok(Some(FormatMode::Stable)),
        Some(mode) => Err(daemon_error(
            OmenaErrorClassV0::Input,
            format!("unsupported format mode {mode:?}"),
            "daemon.invalid-format-mode",
            OmenaErrorRecoverabilityV0::UserAction,
        )),
    }
}

fn parse_lint_profile(profile: Option<&str>) -> Result<Option<LintProfile>, OmenaError> {
    match profile {
        None => Ok(None),
        Some("recommended") => Ok(Some(LintProfile::Recommended)),
        Some("strict") => Ok(Some(LintProfile::Strict)),
        Some(profile) => Err(daemon_error(
            OmenaErrorClassV0::Input,
            format!("unsupported lint profile {profile:?}"),
            "daemon.invalid-lint-profile",
            OmenaErrorRecoverabilityV0::UserAction,
        )),
    }
}

fn write_limited_response(
    stream: &mut TcpStream,
    response: (OmenaWorkspaceSessionResponseV0, u64),
) -> Result<(), String> {
    let (response, max_response_bytes) = response;
    let encoded = serde_json::to_vec(&response)
        .map_err(|error| format!("failed to encode omenad response: {error}"))?;
    if u64::try_from(encoded.len()).unwrap_or(u64::MAX) <= max_response_bytes {
        return write_wire_bytes(stream, encoded.as_slice());
    }
    let limited = omena_workspace_session_failure_v0(
        response.request_id,
        response.snapshot_id,
        daemon_error(
            OmenaErrorClassV0::Analysis,
            "omenad response exceeds the request memory budget",
            "daemon.response-budget-exceeded",
            OmenaErrorRecoverabilityV0::UserAction,
        ),
    );
    write_wire_value(stream, &limited)
}

fn write_wire_value<T: Serialize>(stream: &mut TcpStream, value: &T) -> Result<(), String> {
    let encoded = serde_json::to_vec(value)
        .map_err(|error| format!("failed to encode omenad protocol value: {error}"))?;
    write_wire_bytes(stream, encoded.as_slice())
}

fn write_wire_bytes(stream: &mut TcpStream, encoded: &[u8]) -> Result<(), String> {
    stream
        .write_all(encoded)
        .and_then(|_| stream.write_all(b"\n"))
        .and_then(|_| stream.flush())
        .map_err(|error| format!("failed to write omenad protocol response: {error}"))
}

fn error_envelope(error: OmenaError) -> serde_json::Value {
    serde_json::json!({ "error": error })
}

fn operation_error(message: String) -> OmenaError {
    daemon_error(
        OmenaErrorClassV0::Analysis,
        message,
        "daemon.operation-failed",
        OmenaErrorRecoverabilityV0::UserAction,
    )
}

fn serialization_error(error: serde_json::Error) -> OmenaError {
    daemon_error(
        OmenaErrorClassV0::Internal,
        format!("failed to serialize omenad operation response: {error}"),
        "daemon.response-serialization",
        OmenaErrorRecoverabilityV0::Retry,
    )
}

fn internal_lock_error(lock: &str) -> OmenaError {
    daemon_error(
        OmenaErrorClassV0::Internal,
        format!("omenad {lock} lock was poisoned"),
        "daemon.lock-poisoned",
        OmenaErrorRecoverabilityV0::Retry,
    )
}

fn daemon_error(
    class: OmenaErrorClassV0,
    message: impl Into<String>,
    code: &str,
    recoverability: OmenaErrorRecoverabilityV0,
) -> OmenaError {
    OmenaError::new(
        class,
        message,
        OmenaErrorContextV0 {
            code: code.to_string(),
            severity: OmenaErrorSeverityV0::Error,
            recoverability,
            evidence: Vec::new(),
        },
    )
}
