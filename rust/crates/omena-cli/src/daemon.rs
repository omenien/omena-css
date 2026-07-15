use std::{
    collections::BTreeSet,
    fs,
    io::{BufRead, BufReader, ErrorKind, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, ExitCode, Stdio},
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
    OmenaSdkSnapshotRequestV0, OmenaSdkWorkspaceV0, OmenaWorkspaceSessionHandshakeRequestV0,
    OmenaWorkspaceSessionHandshakeResponseV0, OmenaWorkspaceSessionOperationV0,
    OmenaWorkspaceSessionRequestV0, OmenaWorkspaceSessionResponseV0,
    negotiate_omena_workspace_session_v0, omena_workspace_session_failure_v0,
    omena_workspace_session_success_v0,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    commands::{ExplainCommand, ExplainSymbolKind, FormatMode, LintProfile},
    config::find_omena_config_for_path,
    explain::resolve_explain_command,
    facts::facts_report_value,
    format::build_format_report,
    lint::{discover_style_paths, lint_report},
};

const DEFAULT_IDLE_TIMEOUT_MS: u64 = 300_000;
const MAX_TRANSPORT_LINE_BYTES: usize = 16 * 1024 * 1024;
const WATCH_POLL_INTERVAL: Duration = Duration::from_millis(150);
const WATCH_SESSION_LIMITS: omena_query::OmenaWorkspaceSessionLimitsV0 =
    omena_query::OmenaWorkspaceSessionLimitsV0 {
        deadline_ms: 30_000,
        max_response_bytes: 16 * 1024 * 1024,
    };

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum OmenadCliExplainRequestV0 {
    Diagnostic {
        path: PathBuf,
        code: String,
    },
    Transform {
        path: PathBuf,
        pass_id: String,
    },
    WhyNotTreeShaken {
        path: PathBuf,
        symbol_kind: String,
        symbol: String,
        context_json: PathBuf,
    },
    Precision {
        path: PathBuf,
        variable: String,
        byte_offset: usize,
        source_language: Option<String>,
    },
    Cascade {
        path: PathBuf,
        line: usize,
        character: usize,
    },
    Bundle {
        chunk: String,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OmenadCliExplainPayloadV0 {
    cli_request: OmenadCliExplainRequestV0,
}

#[derive(Debug)]
enum WatchCommandV0 {
    Check {
        path: PathBuf,
        json: bool,
    },
    Lint {
        root: Option<PathBuf>,
        profile: Option<LintProfile>,
        stylelint_config: Option<PathBuf>,
        json: bool,
    },
    Format {
        path: Option<PathBuf>,
        mode: Option<FormatMode>,
        json: bool,
    },
    Explain {
        request: OmenadCliExplainRequestV0,
        json: bool,
    },
}

struct WatchDaemonV0 {
    client: OmenadClientV0,
    snapshot_id: omena_query::OmenaWorkspaceSnapshotIdV0,
    _process: Option<OmenadProcessV0>,
}

struct OmenadProcessV0 {
    child: Child,
    endpoint_file: PathBuf,
}

impl Drop for OmenadProcessV0 {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        cleanup_endpoint(self.endpoint_file.as_path());
    }
}

pub(crate) fn watch_check(path: Option<PathBuf>, write: bool, json: bool) -> Result<(), String> {
    if write {
        return Err("omena check --watch does not write source files".to_string());
    }
    let path = path.ok_or_else(|| "omena check --watch requires a source path".to_string())?;
    run_watch_loop(WatchCommandV0::Check { path, json })
}

pub(crate) fn watch_lint(
    root: Option<PathBuf>,
    profile: Option<LintProfile>,
    stylelint_config: Option<PathBuf>,
    write: bool,
    json: bool,
) -> Result<(), String> {
    if write {
        return Err(
            "omena lint --watch is read-only; run a one-shot --write command explicitly"
                .to_string(),
        );
    }
    run_watch_loop(WatchCommandV0::Lint {
        root,
        profile,
        stylelint_config,
        json,
    })
}

pub(crate) fn watch_format(
    path: Option<PathBuf>,
    mode: Option<FormatMode>,
    _check: bool,
    json: bool,
) -> Result<(), String> {
    run_watch_loop(WatchCommandV0::Format { path, mode, json })
}

pub(crate) fn watch_explain(command: ExplainCommand) -> Result<(), String> {
    let (request, json) = OmenadCliExplainRequestV0::from_command(command);
    run_watch_loop(WatchCommandV0::Explain { request, json })
}

fn run_watch_loop(command: WatchCommandV0) -> Result<(), String> {
    let target = command.workspace_target();
    let workspace_root = workspace_root_for_target(target.as_path())?;
    let config_content_digest = find_omena_config_for_path(target.as_path())?
        .map(|loaded| loaded.config_content_digest.to_string());
    let mut style_sources = collect_style_sources(workspace_root.as_path())?;
    command.add_style_source_alias(&mut style_sources)?;
    let mut daemon = open_watch_daemon(
        workspace_root.as_path(),
        config_content_digest.as_deref(),
        style_sources.as_slice(),
    )
    .map_err(|error| {
        eprintln!("warning: {error}; using direct watch execution");
        error
    })
    .ok();
    let mut request_ordinal = 0_u64;

    let (route, payload) = execute_watch_command(&command, daemon.as_mut(), &mut request_ordinal)?;
    if !emit_watch_result(command.json(), route, daemon.as_ref(), payload)? {
        return Ok(());
    }

    loop {
        thread::sleep(WATCH_POLL_INTERVAL);
        let mut next_sources = collect_style_sources(workspace_root.as_path())?;
        command.add_style_source_alias(&mut next_sources)?;
        if next_sources == style_sources {
            continue;
        }
        style_sources = next_sources;

        if let Some(active_daemon) = daemon.as_mut() {
            request_ordinal = request_ordinal.saturating_add(1);
            let replacement = session_request(
                format!("watch-replace-{request_ordinal}"),
                active_daemon.snapshot_id,
                OmenaWorkspaceSessionOperationV0::ReplaceStyleSources,
                serde_json::to_value(style_sources.as_slice())
                    .map_err(|error| format!("failed to encode watched sources: {error}"))?,
            );
            match active_daemon.client.request(&replacement) {
                Ok(response) if response.ok => active_daemon.snapshot_id = response.snapshot_id,
                Ok(response) => {
                    eprintln!(
                        "warning: omenad rejected a watched snapshot update: {}",
                        response
                            .error
                            .as_ref()
                            .map_or("unknown session error", |error| error.message.as_str())
                    );
                    daemon = None;
                }
                Err(error) => {
                    eprintln!("warning: omenad became unavailable: {error}");
                    daemon = None;
                }
            }
        }

        let (route, payload) =
            execute_watch_command(&command, daemon.as_mut(), &mut request_ordinal)?;
        if !emit_watch_result(command.json(), route, daemon.as_ref(), payload)? {
            return Ok(());
        }
    }
}

fn execute_watch_command(
    command: &WatchCommandV0,
    daemon: Option<&mut WatchDaemonV0>,
    request_ordinal: &mut u64,
) -> Result<(&'static str, serde_json::Value), String> {
    if let Some(daemon) = daemon {
        *request_ordinal = request_ordinal.saturating_add(1);
        let (operation, payload) = command.daemon_operation()?;
        let request = session_request(
            format!("watch-request-{request_ordinal}"),
            daemon.snapshot_id,
            operation,
            payload,
        );
        match daemon.client.request(&request) {
            Ok(response) if response.ok => {
                return Ok((
                    "daemon",
                    response.payload.unwrap_or(serde_json::Value::Null),
                ));
            }
            Ok(response) => {
                eprintln!(
                    "warning: omenad operation failed: {}",
                    response
                        .error
                        .as_ref()
                        .map_or("unknown session error", |error| error.message.as_str())
                );
            }
            Err(error) => eprintln!("warning: omenad request failed: {error}"),
        }
    }
    Ok(("directFallback", command.direct_payload()?))
}

fn emit_watch_result(
    json: bool,
    route: &str,
    daemon: Option<&WatchDaemonV0>,
    payload: serde_json::Value,
) -> Result<bool, String> {
    let snapshot_id = daemon.map(|daemon| daemon.snapshot_id);
    let value = serde_json::json!({
        "schemaVersion": "0",
        "product": "omena-cli.watch",
        "route": route,
        "snapshotId": snapshot_id,
        "payload": payload,
    });
    let rendered = if json {
        serde_json::to_string(&value)
    } else {
        serde_json::to_string_pretty(&value)
    }
    .map_err(|error| format!("failed to render watch result: {error}"))?;
    match std::io::stdout()
        .lock()
        .write_all(format!("{rendered}\n").as_bytes())
    {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == ErrorKind::BrokenPipe => Ok(false),
        Err(error) => Err(format!("failed to write watch result: {error}")),
    }
}

fn open_watch_daemon(
    workspace_root: &Path,
    config_content_digest: Option<&str>,
    style_sources: &[OmenaQueryStyleSourceInputV0],
) -> Result<WatchDaemonV0, String> {
    let endpoint_file = watch_endpoint_path(workspace_root, config_content_digest);
    if endpoint_file.is_file()
        && let Ok(endpoint) = read_omenad_endpoint(endpoint_file.as_path())
    {
        let handshake = watch_handshake(workspace_root, config_content_digest, Vec::new());
        if let Ok((client, response)) = OmenadClientV0::connect(&endpoint, &handshake) {
            return Ok(WatchDaemonV0 {
                client,
                snapshot_id: response.snapshot_id,
                _process: None,
            });
        }
        cleanup_endpoint(endpoint_file.as_path());
    }

    let mut process = spawn_watch_daemon(endpoint_file.clone())?;
    let endpoint = wait_for_endpoint(endpoint_file.as_path(), &mut process.child)?;
    let handshake = watch_handshake(
        workspace_root,
        config_content_digest,
        style_sources.to_vec(),
    );
    let (client, response) = OmenadClientV0::connect(&endpoint, &handshake)?;
    Ok(WatchDaemonV0 {
        client,
        snapshot_id: response.snapshot_id,
        _process: Some(process),
    })
}

fn spawn_watch_daemon(endpoint_file: PathBuf) -> Result<OmenadProcessV0, String> {
    let binary = daemon_binary_path()?;
    let child = Command::new(&binary)
        .args([
            "--endpoint-file",
            endpoint_file.to_string_lossy().as_ref(),
            "--idle-timeout-ms",
            "2000",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("failed to spawn {}: {error}", binary.display()))?;
    Ok(OmenadProcessV0 {
        child,
        endpoint_file,
    })
}

fn daemon_binary_path() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("OMENA_DAEMON_BIN") {
        return Ok(PathBuf::from(path));
    }
    let executable = std::env::current_exe()
        .map_err(|error| format!("failed to locate the current Omena executable: {error}"))?;
    let file_name = if cfg!(windows) {
        "omenad.exe"
    } else {
        "omenad"
    };
    Ok(executable.with_file_name(file_name))
}

fn wait_for_endpoint(path: &Path, child: &mut Child) -> Result<OmenadEndpointV0, String> {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if path.is_file() {
            return read_omenad_endpoint(path);
        }
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("failed to inspect omenad: {error}"))?
        {
            return Err(format!(
                "omenad exited before publishing its endpoint: {status}"
            ));
        }
        thread::sleep(Duration::from_millis(10));
    }
    Err("omenad did not publish an endpoint before the startup deadline".to_string())
}

fn watch_endpoint_path(workspace_root: &Path, config_content_digest: Option<&str>) -> PathBuf {
    if let Some(path) = std::env::var_os("OMENA_DAEMON_ENDPOINT_FILE") {
        return PathBuf::from(path);
    }
    let mut digest = Sha256::new();
    digest.update(workspace_root.to_string_lossy().as_bytes());
    digest.update([0]);
    digest.update(config_content_digest.unwrap_or("<no-config>").as_bytes());
    let key = digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    std::env::temp_dir()
        .join("omena")
        .join(format!("omenad-{}.json", &key[..24]))
}

fn watch_handshake(
    workspace_root: &Path,
    config_content_digest: Option<&str>,
    style_sources: Vec<OmenaQueryStyleSourceInputV0>,
) -> OmenaWorkspaceSessionHandshakeRequestV0 {
    OmenaWorkspaceSessionHandshakeRequestV0 {
        protocol_version: OMENA_WORKSPACE_SESSION_PROTOCOL_VERSION_V0.to_string(),
        workspace_root: workspace_root.to_string_lossy().into_owned(),
        config_content_digest: config_content_digest.map(str::to_string),
        style_sources,
        limits: WATCH_SESSION_LIMITS,
    }
}

fn session_request(
    request_id: String,
    snapshot_id: omena_query::OmenaWorkspaceSnapshotIdV0,
    operation: OmenaWorkspaceSessionOperationV0,
    payload: serde_json::Value,
) -> OmenaWorkspaceSessionRequestV0 {
    OmenaWorkspaceSessionRequestV0 {
        request_id,
        protocol_version: OMENA_WORKSPACE_SESSION_PROTOCOL_VERSION_V0.to_string(),
        snapshot_id,
        operation,
        limits: WATCH_SESSION_LIMITS,
        payload: Some(payload),
    }
}

fn workspace_root_for_target(target: &Path) -> Result<PathBuf, String> {
    let absolute = fs::canonicalize(target).map_err(|error| {
        format!(
            "failed to resolve watch target {}: {error}",
            target.display()
        )
    })?;
    if absolute.is_file() {
        return absolute
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| "watched source has no parent directory".to_string());
    }
    Ok(absolute)
}

fn collect_style_sources(root: &Path) -> Result<Vec<OmenaQueryStyleSourceInputV0>, String> {
    discover_style_paths(root)?
        .into_iter()
        .map(|path| {
            let source = fs::read_to_string(&path).map_err(|error| {
                format!("failed to read watched source {}: {error}", path.display())
            })?;
            Ok(OmenaQueryStyleSourceInputV0 {
                style_path: path.to_string_lossy().into_owned(),
                style_source: source,
            })
        })
        .collect()
}

impl WatchCommandV0 {
    fn json(&self) -> bool {
        match self {
            Self::Check { json, .. }
            | Self::Lint { json, .. }
            | Self::Format { json, .. }
            | Self::Explain { json, .. } => *json,
        }
    }

    fn workspace_target(&self) -> PathBuf {
        match self {
            Self::Check { path, .. } => path.clone(),
            Self::Lint { root, .. } => root.clone().unwrap_or_else(|| PathBuf::from(".")),
            Self::Format { path, .. } => path.clone().unwrap_or_else(|| PathBuf::from(".")),
            Self::Explain { request, .. } => request
                .source_path()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from(".")),
        }
    }

    fn add_style_source_alias(
        &self,
        style_sources: &mut Vec<OmenaQueryStyleSourceInputV0>,
    ) -> Result<(), String> {
        let path = match self {
            Self::Check { path, .. } => Some(path.clone()),
            Self::Explain { request, .. } => request.source_path().map(Path::to_path_buf),
            Self::Lint { .. } | Self::Format { .. } => None,
        };
        let Some(path) = path else {
            return Ok(());
        };
        let path_string = path.to_string_lossy().into_owned();
        if style_sources
            .iter()
            .any(|source| source.style_path == path_string)
        {
            return Ok(());
        }
        if !path.is_file()
            || !matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("css" | "scss" | "sass" | "less")
            )
        {
            return Ok(());
        }
        let style_source = fs::read_to_string(&path).map_err(|error| {
            format!("failed to read watched source {}: {error}", path.display())
        })?;
        style_sources.push(OmenaQueryStyleSourceInputV0 {
            style_path: path_string,
            style_source,
        });
        style_sources.sort_by(|left, right| left.style_path.cmp(&right.style_path));
        Ok(())
    }

    fn daemon_operation(
        &self,
    ) -> Result<(OmenaWorkspaceSessionOperationV0, serde_json::Value), String> {
        match self {
            Self::Check { path, .. } => Ok((
                OmenaWorkspaceSessionOperationV0::Check,
                serde_json::json!({ "stylePath": path }),
            )),
            Self::Lint {
                root,
                profile,
                stylelint_config,
                ..
            } => Ok((
                OmenaWorkspaceSessionOperationV0::Lint,
                serde_json::json!({
                    "root": root,
                    "profile": profile.map(LintProfile::as_str),
                    "stylelintConfig": stylelint_config,
                }),
            )),
            Self::Format { path, mode, .. } => Ok((
                OmenaWorkspaceSessionOperationV0::Format,
                serde_json::json!({
                    "path": path,
                    "mode": mode.map(FormatMode::as_str),
                }),
            )),
            Self::Explain { request, .. } => Ok((
                OmenaWorkspaceSessionOperationV0::Explain,
                serde_json::json!({ "cliRequest": request }),
            )),
        }
    }

    fn direct_payload(&self) -> Result<serde_json::Value, String> {
        match self {
            Self::Check { path, .. } => facts_report_value(path),
            Self::Lint {
                root,
                profile,
                stylelint_config,
                ..
            } => serde_json::to_value(lint_report(
                root.clone(),
                *profile,
                stylelint_config.clone(),
            )?)
            .map_err(|error| format!("failed to serialize lint report: {error}")),
            Self::Format { path, mode, .. } => {
                serde_json::to_value(build_format_report(path.clone(), *mode, true)?)
                    .map_err(|error| format!("failed to serialize format report: {error}"))
            }
            Self::Explain { request, .. } => {
                let (response, _) = resolve_explain_command(request.clone().into_command())?;
                serde_json::to_value(response)
                    .map_err(|error| format!("failed to serialize explain response: {error}"))
            }
        }
    }
}

impl OmenadCliExplainRequestV0 {
    fn from_command(command: ExplainCommand) -> (Self, bool) {
        match command {
            ExplainCommand::Diagnostic { path, code, json } => {
                (Self::Diagnostic { path, code }, json)
            }
            ExplainCommand::Transform {
                path,
                pass_id,
                json,
            } => (Self::Transform { path, pass_id }, json),
            ExplainCommand::WhyNotTreeShaken {
                path,
                symbol_kind,
                symbol,
                context_json,
                json,
            } => (
                Self::WhyNotTreeShaken {
                    path,
                    symbol_kind: explain_symbol_kind_name(symbol_kind).to_string(),
                    symbol,
                    context_json,
                },
                json,
            ),
            ExplainCommand::Precision {
                path,
                variable,
                byte_offset,
                source_language,
                json,
            } => (
                Self::Precision {
                    path,
                    variable,
                    byte_offset,
                    source_language,
                },
                json,
            ),
            ExplainCommand::Cascade {
                path,
                line,
                character,
                json,
            } => (
                Self::Cascade {
                    path,
                    line,
                    character,
                },
                json,
            ),
            ExplainCommand::Bundle { chunk, json } => (Self::Bundle { chunk }, json),
        }
    }

    fn source_path(&self) -> Option<&Path> {
        match self {
            Self::Diagnostic { path, .. }
            | Self::Transform { path, .. }
            | Self::WhyNotTreeShaken { path, .. }
            | Self::Precision { path, .. }
            | Self::Cascade { path, .. } => Some(path),
            Self::Bundle { .. } => None,
        }
    }

    fn into_command(self) -> ExplainCommand {
        match self {
            Self::Diagnostic { path, code } => ExplainCommand::Diagnostic {
                path,
                code,
                json: true,
            },
            Self::Transform { path, pass_id } => ExplainCommand::Transform {
                path,
                pass_id,
                json: true,
            },
            Self::WhyNotTreeShaken {
                path,
                symbol_kind,
                symbol,
                context_json,
            } => ExplainCommand::WhyNotTreeShaken {
                path,
                symbol_kind: parse_explain_symbol_kind(symbol_kind.as_str()),
                symbol,
                context_json,
                json: true,
            },
            Self::Precision {
                path,
                variable,
                byte_offset,
                source_language,
            } => ExplainCommand::Precision {
                path,
                variable,
                byte_offset,
                source_language,
                json: true,
            },
            Self::Cascade {
                path,
                line,
                character,
            } => ExplainCommand::Cascade {
                path,
                line,
                character,
                json: true,
            },
            Self::Bundle { chunk } => ExplainCommand::Bundle { chunk, json: true },
        }
    }
}

fn explain_symbol_kind_name(kind: ExplainSymbolKind) -> &'static str {
    match kind {
        ExplainSymbolKind::Class => "class",
        ExplainSymbolKind::Keyframes => "keyframes",
        ExplainSymbolKind::Value => "value",
        ExplainSymbolKind::CustomProperty => "customProperty",
    }
}

fn parse_explain_symbol_kind(kind: &str) -> ExplainSymbolKind {
    match kind {
        "keyframes" => ExplainSymbolKind::Keyframes,
        "value" => ExplainSymbolKind::Value,
        "customProperty" => ExplainSymbolKind::CustomProperty,
        _ => ExplainSymbolKind::Class,
    }
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
            if request
                .payload
                .as_ref()
                .is_some_and(|payload| payload.get("cliRequest").is_some())
            {
                let payload = parse_payload::<OmenadCliExplainPayloadV0>(request, "explain")?;
                let (response, _) = resolve_explain_command(payload.cli_request.into_command())
                    .map_err(operation_error)?;
                return serde_json::to_value(response).map_err(serialization_error);
            }
            serde_json::to_value(workspace.execute_explain(parse_payload::<
                OmenaSdkExplainRequestV0,
            >(request, "explain")?)?)
            .map_err(serialization_error)
        }
        OmenaWorkspaceSessionOperationV0::Check => {
            let payload = parse_payload::<CheckRequestPayloadV0>(request, "check")?;
            workspace.execute_consumer_check(request.snapshot_id, payload.style_path.as_str())
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
