use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    io::{self, Read, Write},
    process::{Child, Command, Stdio},
};

use serde::{Deserialize, Serialize};

pub const TSGO_TYPE_FLAGS_UNION: u64 = 134_217_728;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaTsgoClientBoundarySummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub client_name: &'static str,
    pub runtime_model: &'static str,
    pub workspace_process_policy: WorkspaceProcessPolicyV0,
    pub request_path_policy: Vec<&'static str>,
    pub api_methods: Vec<TsgoApiMethodV0>,
    pub type_fact_contract: TypeFactContractV0,
    pub lifecycle: TsgoClientLifecycleV0,
    pub recovery_policy: TsgoRecoveryPolicyV0,
    pub ready_surfaces: Vec<&'static str>,
    pub cme_coupled_surfaces: Vec<&'static str>,
    pub next_decoupling_targets: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceProcessPolicyV0 {
    pub process_scope: &'static str,
    pub startup_mode: &'static str,
    pub shutdown_owner: &'static str,
    pub max_workspace_processes: usize,
    pub default_checker_workers: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TsgoApiMethodV0 {
    pub method: &'static str,
    pub phase: &'static str,
    pub purpose: &'static str,
    pub request_group: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeFactContractV0 {
    pub input_contract: &'static str,
    pub target_identity: Vec<&'static str>,
    pub output_contract: &'static str,
    pub unresolved_behavior: &'static str,
    pub project_miss_behavior: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TsgoClientLifecycleV0 {
    pub open_project_method: &'static str,
    pub snapshot_release_method: &'static str,
    pub cancellation_boundary: &'static str,
    pub stale_result_policy: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TsgoRecoveryPolicyV0 {
    pub retry_scope: &'static str,
    pub max_batch_attempts: usize,
    pub recoverable_errors: Vec<&'static str>,
    pub recovery_action: &'static str,
    pub snapshot_policy: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TsgoProviderRetryPolicyV0 {
    pub max_batch_attempts: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TsgoTypeFactRequestV0 {
    pub workspace_root: String,
    pub config_path: String,
    pub targets: Vec<TsgoTypeFactTargetV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TsgoTypeFactTargetV0 {
    pub file_path: String,
    pub expression_id: String,
    pub position: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TsgoTypeFactResultEntryV0 {
    pub file_path: String,
    pub expression_id: String,
    pub resolved_type: TsgoResolvedTypeV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TsgoResolvedTypeV0 {
    pub kind: &'static str,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TsgoTypeFactCollectionPlanV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub workspace_root: String,
    pub config_path: String,
    pub target_count: usize,
    pub unique_file_count: usize,
    pub request_sequence: Vec<TsgoClientRequestStepV0>,
    pub parallelizable_groups: Vec<&'static str>,
    pub request_path_blocking_allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TsgoClientRequestStepV0 {
    pub order: usize,
    pub method: &'static str,
    pub request_count: usize,
    pub blocking_policy: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TsgoProcessCommandV0 {
    pub command: String,
    pub args: Vec<String>,
    pub cwd: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TsgoWorkspaceProcessConfigV0 {
    pub workspace_root: String,
    pub command: TsgoProcessCommandV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TsgoWorkspaceProcessSnapshotV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub workspace_root: String,
    pub command: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub generation: u64,
    pub process_id: Option<u32>,
    pub running: bool,
    pub reused: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TsgoJsonRpcTransportSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub framing: &'static str,
    pub request_version: &'static str,
    pub supports_partial_reads: bool,
    pub error_policy: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TsgoJsonRpcOutboundRequestV0 {
    pub id: u64,
    pub method: String,
    pub frame: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct TsgoTypeFactRpcClientV0 {
    next_id: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TsgoJsonRpcFrameErrorV0 {
    InvalidHeaderUtf8,
    MissingContentLength,
    InvalidContentLength(String),
    InvalidBodyUtf8,
    InvalidJson(String),
}

#[derive(Debug)]
pub enum TsgoJsonRpcIoErrorV0 {
    Io(io::Error),
    Frame(TsgoJsonRpcFrameErrorV0),
    MissingChildStdin,
    MissingChildStdout,
}

#[derive(Debug)]
pub enum TsgoJsonRpcProviderErrorV0 {
    Io(TsgoJsonRpcIoErrorV0),
    Encode(serde_json::Error),
    MissingResponse { method: String },
    UnexpectedResponseId { method: String },
    RpcError { method: String, message: String },
    MissingSnapshot,
}

#[derive(Debug, Default)]
pub struct TsgoWorkspaceProcessPoolV0 {
    processes: BTreeMap<String, ManagedTsgoWorkspaceProcessV0>,
    generation_counter: u64,
}

pub trait TsgoJsonRpcProviderTransportV0 {
    fn send_json_rpc_request(
        &mut self,
        workspace_root: &str,
        request: &TsgoJsonRpcOutboundRequestV0,
    ) -> Result<Option<serde_json::Value>, TsgoJsonRpcIoErrorV0>;

    fn recover_workspace(&mut self, _workspace_root: &str) -> Result<bool, TsgoJsonRpcIoErrorV0> {
        Ok(false)
    }
}

#[derive(Debug)]
pub struct TsgoJsonRpcTypeFactProviderV0<TTransport> {
    transport: TTransport,
    rpc_client: TsgoTypeFactRpcClientV0,
}

pub fn summarize_omena_tsgo_client_boundary() -> OmenaTsgoClientBoundarySummaryV0 {
    OmenaTsgoClientBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-tsgo-client.boundary",
        client_name: "omena-tsgo-client",
        runtime_model: "longLivedWorkspaceProcess",
        workspace_process_policy: WorkspaceProcessPolicyV0 {
            process_scope: "oneTsgoApiProcessPerWorkspace",
            startup_mode: "backgroundWarmup",
            shutdown_owner: "omena-lsp-server",
            max_workspace_processes: 1,
            default_checker_workers: 2,
        },
        request_path_policy: vec![
            "noTypeScriptCreateProgramOnRequestPath",
            "noSyncWorkspaceFallbackOnRequestPath",
            "returnUnresolvedWhenTsgoUnavailable",
            "cooperativeCancellationBeforeTsgoRequest",
            "releaseSnapshotsAfterBatch",
        ],
        api_methods: tsgo_api_methods(),
        type_fact_contract: TypeFactContractV0 {
            input_contract: "TsgoTypeFactRequestV0",
            target_identity: vec!["filePath", "expressionId", "position"],
            output_contract: "TsgoTypeFactResultEntryV0[]",
            unresolved_behavior: "unavailable or non-literal types become unresolvable",
            project_miss_behavior: "project miss returns unresolvable without entering current-ts fallback",
        },
        lifecycle: TsgoClientLifecycleV0 {
            open_project_method: "updateSnapshot",
            snapshot_release_method: "release",
            cancellation_boundary: "before getTypeAtPosition batch",
            stale_result_policy: "discard when document version or config hash changes",
        },
        recovery_policy: TsgoRecoveryPolicyV0 {
            retry_scope: "wholeTypeFactBatch",
            max_batch_attempts: 2,
            recoverable_errors: vec!["io", "missingResponse", "unexpectedResponseId"],
            recovery_action: "restartWorkspaceProcessThenReplayBatch",
            snapshot_policy: "discardPreviousSnapshotAndOpenFreshSnapshot",
        },
        ready_surfaces: vec![
            "tsgoApiInvocationArgs",
            "typeFactRequestContract",
            "typeFactResultContract",
            "requestSequencePlan",
            "persistentWorkspaceProcessPool",
            "jsonRpcContentLengthTransport",
            "jsonRpcProcessIo",
            "jsonRpcTypeFactProviderImplementation",
            "recoverableBatchRetry",
            "workspaceProcessRecovery",
            "typeFactRpcClient",
            "typeFactResultReducer",
            "phase3SourceProviderExitGate",
        ],
        cme_coupled_surfaces: vec![
            "server/engine-host-node/src/tsgo-type-fact-collector.ts",
            "server/engine-host-node/src/type-fact-collector.ts",
        ],
        next_decoupling_targets: vec![
            "lspCancellationTokenWireup",
            "incrementalSnapshotDiff",
            "sourceProviderDirectRustAdapter",
        ],
    }
}

impl Default for TsgoProviderRetryPolicyV0 {
    fn default() -> Self {
        Self {
            max_batch_attempts: 2,
        }
    }
}

pub fn unresolvable_tsgo_type() -> TsgoResolvedTypeV0 {
    TsgoResolvedTypeV0 {
        kind: "unresolvable",
        values: Vec::new(),
    }
}

pub fn union_tsgo_type(values: impl IntoIterator<Item = String>) -> TsgoResolvedTypeV0 {
    TsgoResolvedTypeV0 {
        kind: "union",
        values: values
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
    }
}

pub fn reduce_tsgo_type_response(
    type_response: &serde_json::Value,
    union_members: &[serde_json::Value],
) -> TsgoResolvedTypeV0 {
    if let Some(value) = literal_type_response_value(type_response) {
        return union_tsgo_type([value]);
    }

    let flags = type_response
        .get("flags")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_default();
    if flags & TSGO_TYPE_FLAGS_UNION == 0 {
        return unresolvable_tsgo_type();
    }

    let mut values = Vec::new();
    for member in union_members {
        let resolved_member = reduce_tsgo_type_response(member, &[]);
        if resolved_member.kind != "union" || resolved_member.values.len() != 1 {
            return unresolvable_tsgo_type();
        }
        values.extend(resolved_member.values);
    }

    if values.is_empty() {
        unresolvable_tsgo_type()
    } else {
        union_tsgo_type(values)
    }
}

pub fn build_tsgo_type_fact_result_entry(
    target: &TsgoTypeFactTargetV0,
    type_response: &serde_json::Value,
    union_members: &[serde_json::Value],
) -> TsgoTypeFactResultEntryV0 {
    TsgoTypeFactResultEntryV0 {
        file_path: target.file_path.clone(),
        expression_id: target.expression_id.clone(),
        resolved_type: reduce_tsgo_type_response(type_response, union_members),
    }
}

impl TsgoTypeFactRpcClientV0 {
    pub fn initialize(&mut self) -> Result<TsgoJsonRpcOutboundRequestV0, serde_json::Error> {
        self.request("initialize", None)
    }

    pub fn update_snapshot(
        &mut self,
        config_path: &str,
    ) -> Result<TsgoJsonRpcOutboundRequestV0, serde_json::Error> {
        self.request(
            "updateSnapshot",
            Some(serde_json::json!({ "openProject": config_path })),
        )
    }

    pub fn get_default_project_for_file(
        &mut self,
        snapshot: &str,
        file_path: &str,
    ) -> Result<TsgoJsonRpcOutboundRequestV0, serde_json::Error> {
        self.request(
            "getDefaultProjectForFile",
            Some(serde_json::json!({
                "snapshot": snapshot,
                "file": file_path,
            })),
        )
    }

    pub fn get_type_at_position(
        &mut self,
        snapshot: &str,
        project: &str,
        target: &TsgoTypeFactTargetV0,
    ) -> Result<TsgoJsonRpcOutboundRequestV0, serde_json::Error> {
        self.request(
            "getTypeAtPosition",
            Some(serde_json::json!({
                "snapshot": snapshot,
                "project": project,
                "file": target.file_path,
                "position": target.position,
            })),
        )
    }

    pub fn get_types_of_type(
        &mut self,
        snapshot: &str,
        type_id: &str,
    ) -> Result<TsgoJsonRpcOutboundRequestV0, serde_json::Error> {
        self.request(
            "getTypesOfType",
            Some(serde_json::json!({
                "snapshot": snapshot,
                "type": type_id,
            })),
        )
    }

    pub fn release(
        &mut self,
        handle: &str,
    ) -> Result<TsgoJsonRpcOutboundRequestV0, serde_json::Error> {
        self.request("release", Some(serde_json::json!({ "handle": handle })))
    }

    fn request(
        &mut self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<TsgoJsonRpcOutboundRequestV0, serde_json::Error> {
        self.next_id += 1;
        Ok(TsgoJsonRpcOutboundRequestV0 {
            id: self.next_id,
            method: method.to_string(),
            frame: encode_tsgo_json_rpc_request(self.next_id, method, params)?,
        })
    }
}

pub fn summarize_tsgo_json_rpc_transport() -> TsgoJsonRpcTransportSummaryV0 {
    TsgoJsonRpcTransportSummaryV0 {
        schema_version: "0",
        product: "omena-tsgo-client.json-rpc-transport",
        framing: "Content-Length header followed by UTF-8 JSON body",
        request_version: "2.0",
        supports_partial_reads: true,
        error_policy: "malformed frames fail fast; incomplete frames stay buffered",
    }
}

pub fn tsgo_api_methods() -> Vec<TsgoApiMethodV0> {
    vec![
        TsgoApiMethodV0 {
            method: "initialize",
            phase: "startup",
            purpose: "start tsgo API session",
            request_group: "session",
        },
        TsgoApiMethodV0 {
            method: "updateSnapshot",
            phase: "workspaceSnapshot",
            purpose: "open project config and obtain snapshot handle",
            request_group: "snapshot",
        },
        TsgoApiMethodV0 {
            method: "getDefaultProjectForFile",
            phase: "projectMapping",
            purpose: "map each source file to a tsgo project",
            request_group: "perUniqueFile",
        },
        TsgoApiMethodV0 {
            method: "getTypeAtPosition",
            phase: "typeFacts",
            purpose: "read source expression type at target offset",
            request_group: "perTarget",
        },
        TsgoApiMethodV0 {
            method: "getTypesOfType",
            phase: "typeFacts",
            purpose: "expand union members into finite string facts",
            request_group: "perUnionType",
        },
        TsgoApiMethodV0 {
            method: "release",
            phase: "cleanup",
            purpose: "release snapshot handles after a batch",
            request_group: "snapshot",
        },
    ]
}

pub fn encode_tsgo_json_rpc_request(
    id: u64,
    method: &str,
    params: Option<serde_json::Value>,
) -> Result<Vec<u8>, serde_json::Error> {
    let mut request = serde_json::Map::new();
    request.insert("jsonrpc".to_string(), serde_json::json!("2.0"));
    request.insert("id".to_string(), serde_json::json!(id));
    request.insert("method".to_string(), serde_json::json!(method));
    if let Some(params) = params {
        request.insert("params".to_string(), params);
    }
    encode_tsgo_json_rpc_message(&serde_json::Value::Object(request))
}

pub fn encode_tsgo_json_rpc_message(
    message: &serde_json::Value,
) -> Result<Vec<u8>, serde_json::Error> {
    let body = serde_json::to_vec(message)?;
    let mut frame = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
    frame.extend(body);
    Ok(frame)
}

pub fn drain_tsgo_json_rpc_frames(
    buffer: &mut Vec<u8>,
) -> Result<Vec<serde_json::Value>, TsgoJsonRpcFrameErrorV0> {
    let mut messages = Vec::new();

    while let Some(header_end) = find_header_end(buffer) {
        let header = std::str::from_utf8(&buffer[..header_end])
            .map_err(|_| TsgoJsonRpcFrameErrorV0::InvalidHeaderUtf8)?;
        let length = content_length_from_header(header)?;
        let body_start = header_end + 4;
        let body_end = body_start + length;
        if buffer.len() < body_end {
            break;
        }

        let body = std::str::from_utf8(&buffer[body_start..body_end])
            .map_err(|_| TsgoJsonRpcFrameErrorV0::InvalidBodyUtf8)?;
        let message = serde_json::from_str(body)
            .map_err(|error| TsgoJsonRpcFrameErrorV0::InvalidJson(error.to_string()))?;
        messages.push(message);
        buffer.drain(..body_end);
    }

    Ok(messages)
}

pub fn write_tsgo_json_rpc_request(
    writer: &mut impl Write,
    request: &TsgoJsonRpcOutboundRequestV0,
) -> io::Result<()> {
    writer.write_all(&request.frame)?;
    writer.flush()
}

pub fn read_tsgo_json_rpc_message(
    reader: &mut impl Read,
    buffer: &mut Vec<u8>,
) -> Result<Option<serde_json::Value>, TsgoJsonRpcIoErrorV0> {
    if let Some(message) = drain_tsgo_json_rpc_frames(buffer)?.into_iter().next() {
        return Ok(Some(message));
    }

    let mut chunk = [0; 8192];
    let read = reader.read(&mut chunk)?;
    if read == 0 {
        return Ok(None);
    }
    buffer.extend_from_slice(&chunk[..read]);

    Ok(drain_tsgo_json_rpc_frames(buffer)?.into_iter().next())
}

pub fn build_tsgo_api_args(workspace_root: &str, checkers: Option<usize>) -> Vec<String> {
    let mut args = vec![
        "--api".to_string(),
        "--async".to_string(),
        "--cwd".to_string(),
        workspace_root.to_string(),
    ];

    if let Some(checkers) = checkers {
        args.push("--checkers".to_string());
        args.push(checkers.to_string());
    }

    args
}

pub fn build_tsgo_process_command(
    tsgo_path: &str,
    workspace_root: &str,
    checkers: Option<usize>,
) -> TsgoProcessCommandV0 {
    TsgoProcessCommandV0 {
        command: tsgo_path.to_string(),
        args: build_tsgo_api_args(workspace_root, checkers),
        cwd: workspace_root.to_string(),
    }
}

pub fn plan_tsgo_type_fact_collection(
    request: &TsgoTypeFactRequestV0,
) -> TsgoTypeFactCollectionPlanV0 {
    let unique_files = request
        .targets
        .iter()
        .map(|target| target.file_path.clone())
        .collect::<BTreeSet<_>>();
    let target_count = request.targets.len();

    TsgoTypeFactCollectionPlanV0 {
        schema_version: "0",
        product: "omena-tsgo-client.type-fact-collection-plan",
        workspace_root: request.workspace_root.clone(),
        config_path: request.config_path.clone(),
        target_count,
        unique_file_count: unique_files.len(),
        request_sequence: vec![
            request_step(1, "initialize", 1, "startupOnly"),
            request_step(2, "updateSnapshot", 1, "backgroundOrBatch"),
            request_step(
                3,
                "getDefaultProjectForFile",
                unique_files.len(),
                "cancelBeforeBatch",
            ),
            request_step(4, "getTypeAtPosition", target_count, "cancelBeforeBatch"),
            request_step(5, "getTypesOfType", target_count, "onlyWhenUnion"),
            request_step(6, "release", 1, "alwaysAfterSnapshotBatch"),
        ],
        parallelizable_groups: vec![
            "getDefaultProjectForFileByUniqueFile",
            "getTypeAtPositionByTarget",
            "getTypesOfTypeByUnionMember",
        ],
        request_path_blocking_allowed: false,
    }
}

impl fmt::Display for TsgoJsonRpcFrameErrorV0 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHeaderUtf8 => formatter.write_str("tsgo JSON-RPC header is not UTF-8"),
            Self::MissingContentLength => {
                formatter.write_str("tsgo JSON-RPC frame is missing Content-Length")
            }
            Self::InvalidContentLength(value) => {
                write!(
                    formatter,
                    "tsgo JSON-RPC Content-Length is invalid: {value}"
                )
            }
            Self::InvalidBodyUtf8 => formatter.write_str("tsgo JSON-RPC body is not UTF-8"),
            Self::InvalidJson(value) => write!(formatter, "tsgo JSON-RPC body is invalid: {value}"),
        }
    }
}

impl std::error::Error for TsgoJsonRpcFrameErrorV0 {}

impl fmt::Display for TsgoJsonRpcIoErrorV0 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "tsgo JSON-RPC I/O failed: {error}"),
            Self::Frame(error) => write!(formatter, "tsgo JSON-RPC frame failed: {error}"),
            Self::MissingChildStdin => formatter.write_str("tsgo child process stdin is missing"),
            Self::MissingChildStdout => formatter.write_str("tsgo child process stdout is missing"),
        }
    }
}

impl std::error::Error for TsgoJsonRpcIoErrorV0 {}

impl fmt::Display for TsgoJsonRpcProviderErrorV0 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "tsgo JSON-RPC provider I/O failed: {error}"),
            Self::Encode(error) => {
                write!(
                    formatter,
                    "tsgo JSON-RPC provider request encode failed: {error}"
                )
            }
            Self::MissingResponse { method } => {
                write!(
                    formatter,
                    "tsgo JSON-RPC provider missing response for {method}"
                )
            }
            Self::UnexpectedResponseId { method } => {
                write!(
                    formatter,
                    "tsgo JSON-RPC provider received unexpected response id for {method}"
                )
            }
            Self::RpcError { method, message } => {
                write!(
                    formatter,
                    "tsgo JSON-RPC provider request {method} failed: {message}"
                )
            }
            Self::MissingSnapshot => formatter.write_str("tsgo JSON-RPC provider missing snapshot"),
        }
    }
}

impl std::error::Error for TsgoJsonRpcProviderErrorV0 {}

impl TsgoJsonRpcProviderErrorV0 {
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Io(_) | Self::MissingResponse { .. } | Self::UnexpectedResponseId { .. }
        )
    }
}

impl From<io::Error> for TsgoJsonRpcIoErrorV0 {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<TsgoJsonRpcFrameErrorV0> for TsgoJsonRpcIoErrorV0 {
    fn from(error: TsgoJsonRpcFrameErrorV0) -> Self {
        Self::Frame(error)
    }
}

impl From<TsgoJsonRpcIoErrorV0> for TsgoJsonRpcProviderErrorV0 {
    fn from(error: TsgoJsonRpcIoErrorV0) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for TsgoJsonRpcProviderErrorV0 {
    fn from(error: serde_json::Error) -> Self {
        Self::Encode(error)
    }
}

impl TsgoWorkspaceProcessPoolV0 {
    pub fn ensure_workspace_process(
        &mut self,
        config: TsgoWorkspaceProcessConfigV0,
    ) -> io::Result<TsgoWorkspaceProcessSnapshotV0> {
        if let Some(existing) = self.processes.get_mut(config.workspace_root.as_str())
            && existing.config.command == config.command
            && existing.is_running()?
        {
            return Ok(existing.snapshot(true));
        }

        if let Some(mut previous) = self.processes.remove(config.workspace_root.as_str()) {
            previous.shutdown()?;
        }

        self.generation_counter += 1;
        let process = ManagedTsgoWorkspaceProcessV0::spawn(config, self.generation_counter)?;
        let snapshot = process.snapshot(false);
        self.processes
            .insert(process.config.workspace_root.clone(), process);
        Ok(snapshot)
    }

    pub fn restart_workspace_process(
        &mut self,
        workspace_root: &str,
    ) -> io::Result<Option<TsgoWorkspaceProcessSnapshotV0>> {
        let Some(mut previous) = self.processes.remove(workspace_root) else {
            return Ok(None);
        };
        let config = previous.config.clone();
        previous.shutdown()?;

        self.generation_counter += 1;
        let process = ManagedTsgoWorkspaceProcessV0::spawn(config, self.generation_counter)?;
        let snapshot = process.snapshot(false);
        self.processes
            .insert(process.config.workspace_root.clone(), process);
        Ok(Some(snapshot))
    }

    pub fn shutdown_workspace(&mut self, workspace_root: &str) -> io::Result<bool> {
        let Some(mut process) = self.processes.remove(workspace_root) else {
            return Ok(false);
        };
        process.shutdown()?;
        Ok(true)
    }

    pub fn send_json_rpc_request(
        &mut self,
        workspace_root: &str,
        request: &TsgoJsonRpcOutboundRequestV0,
    ) -> Result<Option<serde_json::Value>, TsgoJsonRpcIoErrorV0> {
        let Some(process) = self.processes.get_mut(workspace_root) else {
            return Ok(None);
        };
        process.send_json_rpc_request(request)
    }

    pub fn shutdown_all(&mut self) -> io::Result<usize> {
        let mut count = 0;
        let workspace_roots = self.processes.keys().cloned().collect::<Vec<_>>();
        for workspace_root in workspace_roots {
            if self.shutdown_workspace(workspace_root.as_str())? {
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn len(&self) -> usize {
        self.processes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.processes.is_empty()
    }
}

impl TsgoJsonRpcProviderTransportV0 for TsgoWorkspaceProcessPoolV0 {
    fn send_json_rpc_request(
        &mut self,
        workspace_root: &str,
        request: &TsgoJsonRpcOutboundRequestV0,
    ) -> Result<Option<serde_json::Value>, TsgoJsonRpcIoErrorV0> {
        TsgoWorkspaceProcessPoolV0::send_json_rpc_request(self, workspace_root, request)
    }

    fn recover_workspace(&mut self, workspace_root: &str) -> Result<bool, TsgoJsonRpcIoErrorV0> {
        self.restart_workspace_process(workspace_root)
            .map(|snapshot| snapshot.is_some())
            .map_err(TsgoJsonRpcIoErrorV0::Io)
    }
}

impl Drop for TsgoWorkspaceProcessPoolV0 {
    fn drop(&mut self) {
        let _ = self.shutdown_all();
    }
}

impl<TTransport> TsgoJsonRpcTypeFactProviderV0<TTransport>
where
    TTransport: TsgoJsonRpcProviderTransportV0,
{
    pub fn new(transport: TTransport) -> Self {
        Self {
            transport,
            rpc_client: TsgoTypeFactRpcClientV0::default(),
        }
    }

    pub fn into_transport(self) -> TTransport {
        self.transport
    }

    pub fn collect_type_facts(
        &mut self,
        request: &TsgoTypeFactRequestV0,
    ) -> Result<Vec<TsgoTypeFactResultEntryV0>, TsgoJsonRpcProviderErrorV0> {
        self.collect_type_facts_with_policy(request, TsgoProviderRetryPolicyV0::default())
    }

    pub fn collect_type_facts_with_policy(
        &mut self,
        request: &TsgoTypeFactRequestV0,
        retry_policy: TsgoProviderRetryPolicyV0,
    ) -> Result<Vec<TsgoTypeFactResultEntryV0>, TsgoJsonRpcProviderErrorV0> {
        let max_attempts = retry_policy.max_batch_attempts.max(1);
        let mut attempt = 1;

        loop {
            match self.collect_type_facts_once(request) {
                Ok(entries) => return Ok(entries),
                Err(error) if attempt < max_attempts && error.is_recoverable() => {
                    if !self
                        .transport
                        .recover_workspace(request.workspace_root.as_str())?
                    {
                        return Err(error);
                    }
                    attempt += 1;
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn collect_type_facts_once(
        &mut self,
        request: &TsgoTypeFactRequestV0,
    ) -> Result<Vec<TsgoTypeFactResultEntryV0>, TsgoJsonRpcProviderErrorV0> {
        let initialize_request = self.rpc_client.initialize()?;
        self.send_result(request.workspace_root.as_str(), &initialize_request)?;

        let snapshot_request = self
            .rpc_client
            .update_snapshot(request.config_path.as_str())?;
        let snapshot_result =
            self.send_result(request.workspace_root.as_str(), &snapshot_request)?;
        let snapshot = snapshot_result
            .get("snapshot")
            .and_then(serde_json::Value::as_str)
            .ok_or(TsgoJsonRpcProviderErrorV0::MissingSnapshot)?
            .to_string();

        let collection_result = self.collect_type_facts_for_snapshot(request, snapshot.as_str());
        let release_result =
            self.release_snapshot(request.workspace_root.as_str(), snapshot.as_str());

        match (collection_result, release_result) {
            (Ok(entries), Ok(())) => Ok(entries),
            (Err(error), _) => Err(error),
            (Ok(_), Err(error)) => Err(error),
        }
    }

    fn collect_type_facts_for_snapshot(
        &mut self,
        request: &TsgoTypeFactRequestV0,
        snapshot: &str,
    ) -> Result<Vec<TsgoTypeFactResultEntryV0>, TsgoJsonRpcProviderErrorV0> {
        let project_by_file = self.resolve_projects_by_file(request, snapshot)?;
        request
            .targets
            .iter()
            .map(|target| {
                let Some(Some(project)) = project_by_file.get(target.file_path.as_str()) else {
                    return Ok(TsgoTypeFactResultEntryV0 {
                        file_path: target.file_path.clone(),
                        expression_id: target.expression_id.clone(),
                        resolved_type: unresolvable_tsgo_type(),
                    });
                };
                let type_request =
                    self.rpc_client
                        .get_type_at_position(snapshot, project.as_str(), target)?;
                let type_response =
                    self.send_result(request.workspace_root.as_str(), &type_request)?;
                let resolved_type = self.resolve_type_response(
                    request.workspace_root.as_str(),
                    snapshot,
                    &type_response,
                )?;
                Ok(TsgoTypeFactResultEntryV0 {
                    file_path: target.file_path.clone(),
                    expression_id: target.expression_id.clone(),
                    resolved_type,
                })
            })
            .collect()
    }

    fn resolve_projects_by_file(
        &mut self,
        request: &TsgoTypeFactRequestV0,
        snapshot: &str,
    ) -> Result<BTreeMap<String, Option<String>>, TsgoJsonRpcProviderErrorV0> {
        let unique_files = request
            .targets
            .iter()
            .map(|target| target.file_path.as_str())
            .collect::<BTreeSet<_>>();
        let mut projects = BTreeMap::new();
        for file_path in unique_files {
            let project_request = self
                .rpc_client
                .get_default_project_for_file(snapshot, file_path)?;
            let project_response =
                self.send_result(request.workspace_root.as_str(), &project_request)?;
            projects.insert(
                file_path.to_string(),
                project_response
                    .get("id")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string),
            );
        }
        Ok(projects)
    }

    fn resolve_type_response(
        &mut self,
        workspace_root: &str,
        snapshot: &str,
        type_response: &serde_json::Value,
    ) -> Result<TsgoResolvedTypeV0, TsgoJsonRpcProviderErrorV0> {
        if let Some(value) = literal_type_response_value(type_response) {
            return Ok(union_tsgo_type([value]));
        }

        let flags = type_response
            .get("flags")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_default();
        if flags & TSGO_TYPE_FLAGS_UNION == 0 {
            return Ok(unresolvable_tsgo_type());
        }

        let Some(type_id) = type_response.get("id").and_then(serde_json::Value::as_str) else {
            return Ok(unresolvable_tsgo_type());
        };
        let members_request = self.rpc_client.get_types_of_type(snapshot, type_id)?;
        let members_response = self.send_result(workspace_root, &members_request)?;
        let Some(members) = members_response.as_array() else {
            return Ok(unresolvable_tsgo_type());
        };

        let mut values = Vec::new();
        for member in members {
            let resolved = self.resolve_type_response(workspace_root, snapshot, member)?;
            if resolved.kind != "union" || resolved.values.len() != 1 {
                return Ok(unresolvable_tsgo_type());
            }
            values.extend(resolved.values);
        }

        if values.is_empty() {
            Ok(unresolvable_tsgo_type())
        } else {
            Ok(union_tsgo_type(values))
        }
    }

    fn release_snapshot(
        &mut self,
        workspace_root: &str,
        snapshot: &str,
    ) -> Result<(), TsgoJsonRpcProviderErrorV0> {
        let release_request = self.rpc_client.release(snapshot)?;
        self.send_result(workspace_root, &release_request)?;
        Ok(())
    }

    fn send_result(
        &mut self,
        workspace_root: &str,
        request: &TsgoJsonRpcOutboundRequestV0,
    ) -> Result<serde_json::Value, TsgoJsonRpcProviderErrorV0> {
        let message = self
            .transport
            .send_json_rpc_request(workspace_root, request)?
            .ok_or_else(|| TsgoJsonRpcProviderErrorV0::MissingResponse {
                method: request.method.clone(),
            })?;
        if message.get("id").and_then(serde_json::Value::as_u64) != Some(request.id) {
            return Err(TsgoJsonRpcProviderErrorV0::UnexpectedResponseId {
                method: request.method.clone(),
            });
        }
        if let Some(error) = message.get("error") {
            return Err(TsgoJsonRpcProviderErrorV0::RpcError {
                method: request.method.clone(),
                message: error
                    .get("message")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| error.to_string()),
            });
        }
        Ok(message
            .get("result")
            .cloned()
            .unwrap_or(serde_json::Value::Null))
    }
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn literal_type_response_value(type_response: &serde_json::Value) -> Option<String> {
    let value = type_response.get("value")?;
    if let Some(value) = value.as_str() {
        return Some(value.to_string());
    }
    if let Some(value) = value.as_i64() {
        return Some(value.to_string());
    }
    if let Some(value) = value.as_u64() {
        return Some(value.to_string());
    }
    if let Some(value) = value.as_f64()
        && value.is_finite()
        && value.fract() == 0.0
    {
        return Some(format!("{value:.0}"));
    }
    None
}

fn content_length_from_header(header: &str) -> Result<usize, TsgoJsonRpcFrameErrorV0> {
    for line in header.lines() {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        if name.eq_ignore_ascii_case("content-length") {
            return value
                .trim()
                .parse::<usize>()
                .map_err(|_| TsgoJsonRpcFrameErrorV0::InvalidContentLength(value.to_string()));
        }
    }

    Err(TsgoJsonRpcFrameErrorV0::MissingContentLength)
}

fn request_step(
    order: usize,
    method: &'static str,
    request_count: usize,
    blocking_policy: &'static str,
) -> TsgoClientRequestStepV0 {
    TsgoClientRequestStepV0 {
        order,
        method,
        request_count,
        blocking_policy,
    }
}

#[derive(Debug)]
struct ManagedTsgoWorkspaceProcessV0 {
    config: TsgoWorkspaceProcessConfigV0,
    child: Child,
    generation: u64,
    stdout_buffer: Vec<u8>,
}

impl ManagedTsgoWorkspaceProcessV0 {
    fn spawn(config: TsgoWorkspaceProcessConfigV0, generation: u64) -> io::Result<Self> {
        let child = Command::new(config.command.command.as_str())
            .args(&config.command.args)
            .current_dir(config.command.cwd.as_str())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Ok(Self {
            config,
            child,
            generation,
            stdout_buffer: Vec::new(),
        })
    }

    fn is_running(&mut self) -> io::Result<bool> {
        Ok(self.child.try_wait()?.is_none())
    }

    fn snapshot(&self, reused: bool) -> TsgoWorkspaceProcessSnapshotV0 {
        TsgoWorkspaceProcessSnapshotV0 {
            schema_version: "0",
            product: "omena-tsgo-client.workspace-process",
            workspace_root: self.config.workspace_root.clone(),
            command: self.config.command.command.clone(),
            args: self.config.command.args.clone(),
            cwd: self.config.command.cwd.clone(),
            generation: self.generation,
            process_id: Some(self.child.id()),
            running: true,
            reused,
        }
    }

    fn shutdown(&mut self) -> io::Result<()> {
        if self.child.try_wait()?.is_none() {
            self.child.kill()?;
        }
        self.child.wait()?;
        Ok(())
    }

    fn send_json_rpc_request(
        &mut self,
        request: &TsgoJsonRpcOutboundRequestV0,
    ) -> Result<Option<serde_json::Value>, TsgoJsonRpcIoErrorV0> {
        let stdin = self
            .child
            .stdin
            .as_mut()
            .ok_or(TsgoJsonRpcIoErrorV0::MissingChildStdin)?;
        write_tsgo_json_rpc_request(stdin, request)?;

        let stdout = self
            .child
            .stdout
            .as_mut()
            .ok_or(TsgoJsonRpcIoErrorV0::MissingChildStdout)?;
        read_tsgo_json_rpc_message(stdout, &mut self.stdout_buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        TSGO_TYPE_FLAGS_UNION, TsgoJsonRpcIoErrorV0, TsgoJsonRpcOutboundRequestV0,
        TsgoJsonRpcProviderTransportV0, TsgoJsonRpcTypeFactProviderV0, TsgoProcessCommandV0,
        TsgoProviderRetryPolicyV0, TsgoTypeFactRequestV0, TsgoTypeFactRpcClientV0,
        TsgoTypeFactTargetV0, TsgoWorkspaceProcessConfigV0, TsgoWorkspaceProcessPoolV0,
        build_tsgo_api_args, build_tsgo_process_command, drain_tsgo_json_rpc_frames,
        encode_tsgo_json_rpc_message, encode_tsgo_json_rpc_request, plan_tsgo_type_fact_collection,
        read_tsgo_json_rpc_message, reduce_tsgo_type_response,
        summarize_omena_tsgo_client_boundary, summarize_tsgo_json_rpc_transport,
        write_tsgo_json_rpc_request,
    };
    use serde_json::json;
    use std::{collections::VecDeque, io};

    #[test]
    fn declares_long_lived_tsgo_client_boundary() {
        let summary = summarize_omena_tsgo_client_boundary();

        assert_eq!(summary.schema_version, "0");
        assert_eq!(summary.product, "omena-tsgo-client.boundary");
        assert_eq!(summary.runtime_model, "longLivedWorkspaceProcess");
        assert!(
            summary
                .request_path_policy
                .contains(&"noSyncWorkspaceFallbackOnRequestPath")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"phase3SourceProviderExitGate")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"persistentWorkspaceProcessPool")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"jsonRpcContentLengthTransport")
        );
        assert!(summary.ready_surfaces.contains(&"typeFactRpcClient"));
        assert!(summary.ready_surfaces.contains(&"typeFactResultReducer"));
        assert!(summary.ready_surfaces.contains(&"recoverableBatchRetry"));
        assert!(summary.ready_surfaces.contains(&"workspaceProcessRecovery"));
        assert_eq!(summary.recovery_policy.retry_scope, "wholeTypeFactBatch");
        assert_eq!(summary.recovery_policy.max_batch_attempts, 2);
        assert!(
            summary
                .recovery_policy
                .recoverable_errors
                .contains(&"missingResponse")
        );
        assert!(
            summary
                .next_decoupling_targets
                .contains(&"sourceProviderDirectRustAdapter")
        );
    }

    #[test]
    fn builds_tsgo_api_invocation_args() {
        assert_eq!(
            build_tsgo_api_args("/workspace", Some(2)),
            vec!["--api", "--async", "--cwd", "/workspace", "--checkers", "2"]
        );
    }

    #[test]
    fn builds_tsgo_workspace_process_command() {
        assert_eq!(
            build_tsgo_process_command("/bin/tsgo", "/workspace", Some(2)),
            TsgoProcessCommandV0 {
                command: "/bin/tsgo".to_string(),
                args: vec![
                    "--api".to_string(),
                    "--async".to_string(),
                    "--cwd".to_string(),
                    "/workspace".to_string(),
                    "--checkers".to_string(),
                    "2".to_string()
                ],
                cwd: "/workspace".to_string(),
            }
        );
    }

    #[test]
    fn summarizes_json_rpc_transport() {
        let summary = summarize_tsgo_json_rpc_transport();

        assert_eq!(summary.product, "omena-tsgo-client.json-rpc-transport");
        assert!(summary.supports_partial_reads);
    }

    #[test]
    fn encodes_tsgo_json_rpc_request_with_content_length_frame() -> Result<(), serde_json::Error> {
        let frame = encode_tsgo_json_rpc_request(
            7,
            "getTypeAtPosition",
            Some(json!({ "file": "/repo/App.tsx", "position": 12 })),
        )?;
        let frame = String::from_utf8_lossy(&frame);

        assert!(frame.starts_with("Content-Length: "));
        assert!(frame.contains("\r\n\r\n"));
        assert!(frame.contains("\"jsonrpc\":\"2.0\""));
        assert!(frame.contains("\"method\":\"getTypeAtPosition\""));
        Ok(())
    }

    #[test]
    fn drains_complete_and_partial_tsgo_json_rpc_frames() -> Result<(), Box<dyn std::error::Error>>
    {
        let first = encode_tsgo_json_rpc_message(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": { "snapshot": "s1" }
        }))?;
        let second = encode_tsgo_json_rpc_message(&json!({
            "jsonrpc": "2.0",
            "id": 2,
            "result": { "id": "project" }
        }))?;
        let split = second.len() / 2;
        let mut buffer = Vec::new();
        buffer.extend(first);
        buffer.extend(&second[..split]);

        let messages = drain_tsgo_json_rpc_frames(&mut buffer)?;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].pointer("/result/snapshot"), Some(&json!("s1")));
        assert!(!buffer.is_empty());

        buffer.extend(&second[split..]);
        let messages = drain_tsgo_json_rpc_frames(&mut buffer)?;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].pointer("/result/id"), Some(&json!("project")));
        assert!(buffer.is_empty());
        Ok(())
    }

    #[test]
    fn rejects_tsgo_json_rpc_frames_without_content_length() {
        let mut buffer = b"X-Test: 1\r\n\r\n{}".to_vec();

        let error = drain_tsgo_json_rpc_frames(&mut buffer).err();
        assert!(matches!(
            error,
            Some(super::TsgoJsonRpcFrameErrorV0::MissingContentLength)
        ));
    }

    #[test]
    fn writes_and_reads_tsgo_json_rpc_messages_over_generic_io()
    -> Result<(), Box<dyn std::error::Error>> {
        let request = encode_tsgo_json_rpc_request(1, "initialize", None)?;
        let outbound = super::TsgoJsonRpcOutboundRequestV0 {
            id: 1,
            method: "initialize".to_string(),
            frame: request,
        };
        let mut writer = Vec::new();

        write_tsgo_json_rpc_request(&mut writer, &outbound)?;

        let mut reader = std::io::Cursor::new(encode_tsgo_json_rpc_message(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": { "ok": true }
        }))?);
        let mut buffer = Vec::new();
        let response = read_tsgo_json_rpc_message(&mut reader, &mut buffer)?;

        assert!(String::from_utf8_lossy(&writer).contains("\"method\":\"initialize\""));
        assert_eq!(
            response.and_then(|message| message.pointer("/result/ok").cloned()),
            Some(json!(true))
        );
        Ok(())
    }

    #[test]
    fn type_fact_rpc_client_emits_node_compatible_requests()
    -> Result<(), Box<dyn std::error::Error>> {
        let target = TsgoTypeFactTargetV0 {
            file_path: "/repo/src/App.tsx".to_string(),
            expression_id: "expr-1".to_string(),
            position: 42,
        };
        let mut client = TsgoTypeFactRpcClientV0::default();
        let requests = vec![
            client.initialize()?,
            client.update_snapshot("/repo/tsconfig.json")?,
            client.get_default_project_for_file("snapshot-1", "/repo/src/App.tsx")?,
            client.get_type_at_position("snapshot-1", "project-1", &target)?,
            client.get_types_of_type("snapshot-1", "type-1")?,
            client.release("snapshot-1")?,
        ];

        let mut methods = Vec::new();
        let mut payloads = Vec::new();
        for request in requests {
            let mut frame = request.frame;
            let drained = drain_tsgo_json_rpc_frames(&mut frame)?;
            assert_eq!(drained.len(), 1);
            methods.push(
                drained[0]
                    .get("method")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            );
            payloads.push(drained[0].clone());
        }

        assert_eq!(
            methods,
            vec![
                "initialize",
                "updateSnapshot",
                "getDefaultProjectForFile",
                "getTypeAtPosition",
                "getTypesOfType",
                "release",
            ]
        );
        assert_eq!(
            payloads[1].pointer("/params/openProject"),
            Some(&json!("/repo/tsconfig.json"))
        );
        assert_eq!(payloads[3].pointer("/params/position"), Some(&json!(42)));
        assert_eq!(payloads[4].pointer("/params/type"), Some(&json!("type-1")));
        assert_eq!(
            payloads[5].pointer("/params/handle"),
            Some(&json!("snapshot-1"))
        );
        Ok(())
    }

    #[test]
    fn reduces_tsgo_literal_type_response_to_union() {
        let resolved = reduce_tsgo_type_response(&json!({ "value": "root" }), &[]);

        assert_eq!(resolved.kind, "union");
        assert_eq!(resolved.values, vec!["root"]);
    }

    #[test]
    fn reduces_numeric_literal_type_response_to_string_union() {
        let resolved = reduce_tsgo_type_response(
            &json!({ "id": "type-1", "flags": TSGO_TYPE_FLAGS_UNION }),
            &[json!({ "value": 10 }), json!({ "value": 12 })],
        );

        assert_eq!(resolved.kind, "union");
        assert_eq!(resolved.values, vec!["10", "12"]);
    }

    #[test]
    fn reduces_tsgo_union_type_response_to_deduped_union() {
        let resolved = reduce_tsgo_type_response(
            &json!({ "id": "type-1", "flags": TSGO_TYPE_FLAGS_UNION }),
            &[
                json!({ "value": "primary" }),
                json!({ "value": "secondary" }),
                json!({ "value": "primary" }),
            ],
        );

        assert_eq!(resolved.kind, "union");
        assert_eq!(resolved.values, vec!["primary", "secondary"]);
    }

    #[test]
    fn marks_non_literal_tsgo_type_response_unresolvable() {
        let resolved = reduce_tsgo_type_response(
            &json!({ "id": "type-1", "flags": TSGO_TYPE_FLAGS_UNION }),
            &[json!({ "id": "non-literal" })],
        );

        assert_eq!(resolved.kind, "unresolvable");
        assert!(resolved.values.is_empty());
    }

    #[test]
    fn json_rpc_type_fact_provider_collects_literal_and_union_facts()
    -> Result<(), Box<dyn std::error::Error>> {
        let request = TsgoTypeFactRequestV0 {
            workspace_root: "/repo".to_string(),
            config_path: "/repo/tsconfig.json".to_string(),
            targets: vec![
                TsgoTypeFactTargetV0 {
                    file_path: "/repo/src/App.tsx".to_string(),
                    expression_id: "expr-1".to_string(),
                    position: 12,
                },
                TsgoTypeFactTargetV0 {
                    file_path: "/repo/src/App.tsx".to_string(),
                    expression_id: "expr-2".to_string(),
                    position: 24,
                },
            ],
        };
        let transport = FakeTsgoTransport::new(vec![
            json!(null),
            json!({ "snapshot": "snapshot-1" }),
            json!({ "id": "project-1" }),
            json!({ "value": "primary" }),
            json!({ "id": "type-1", "flags": TSGO_TYPE_FLAGS_UNION }),
            json!([{ "value": "secondary" }, { "value": "primary" }]),
            json!(null),
        ]);
        let mut provider = TsgoJsonRpcTypeFactProviderV0::new(transport);

        let entries = provider.collect_type_facts(&request)?;
        let transport = provider.into_transport();

        assert_eq!(
            transport.methods,
            vec![
                "initialize",
                "updateSnapshot",
                "getDefaultProjectForFile",
                "getTypeAtPosition",
                "getTypeAtPosition",
                "getTypesOfType",
                "release",
            ]
        );
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].resolved_type.kind, "union");
        assert_eq!(entries[0].resolved_type.values, vec!["primary"]);
        assert_eq!(entries[1].resolved_type.kind, "union");
        assert_eq!(
            entries[1].resolved_type.values,
            vec!["primary", "secondary"]
        );
        Ok(())
    }

    #[test]
    fn json_rpc_type_fact_provider_returns_unresolvable_on_project_miss()
    -> Result<(), Box<dyn std::error::Error>> {
        let request = TsgoTypeFactRequestV0 {
            workspace_root: "/repo".to_string(),
            config_path: "/repo/tsconfig.json".to_string(),
            targets: vec![TsgoTypeFactTargetV0 {
                file_path: "/repo/src/App.tsx".to_string(),
                expression_id: "expr-1".to_string(),
                position: 12,
            }],
        };
        let transport = FakeTsgoTransport::new(vec![
            json!(null),
            json!({ "snapshot": "snapshot-1" }),
            json!({}),
            json!(null),
        ]);
        let mut provider = TsgoJsonRpcTypeFactProviderV0::new(transport);

        let entries = provider.collect_type_facts(&request)?;
        let transport = provider.into_transport();

        assert_eq!(
            transport.methods,
            vec![
                "initialize",
                "updateSnapshot",
                "getDefaultProjectForFile",
                "release"
            ]
        );
        assert_eq!(entries[0].resolved_type.kind, "unresolvable");
        assert!(entries[0].resolved_type.values.is_empty());
        Ok(())
    }

    #[test]
    fn json_rpc_type_fact_provider_recovers_whole_batch_after_missing_response()
    -> Result<(), Box<dyn std::error::Error>> {
        let request = TsgoTypeFactRequestV0 {
            workspace_root: "/repo".to_string(),
            config_path: "/repo/tsconfig.json".to_string(),
            targets: vec![TsgoTypeFactTargetV0 {
                file_path: "/repo/src/App.tsx".to_string(),
                expression_id: "expr-1".to_string(),
                position: 12,
            }],
        };
        let transport = FakeTsgoTransport::new_script(vec![
            Some(json!(null)),
            None,
            Some(json!(null)),
            Some(json!({ "snapshot": "snapshot-2" })),
            Some(json!({ "id": "project-1" })),
            Some(json!({ "value": "recovered" })),
            Some(json!(null)),
        ]);
        let mut provider = TsgoJsonRpcTypeFactProviderV0::new(transport);

        let entries = provider.collect_type_facts_with_policy(
            &request,
            TsgoProviderRetryPolicyV0 {
                max_batch_attempts: 2,
            },
        )?;
        let transport = provider.into_transport();

        assert_eq!(transport.reset_count, 1);
        assert_eq!(
            transport.methods,
            vec![
                "initialize",
                "updateSnapshot",
                "initialize",
                "updateSnapshot",
                "getDefaultProjectForFile",
                "getTypeAtPosition",
                "release",
            ]
        );
        assert_eq!(entries[0].resolved_type.kind, "union");
        assert_eq!(entries[0].resolved_type.values, vec!["recovered"]);
        Ok(())
    }

    #[test]
    fn process_pool_reuses_running_workspace_process() -> io::Result<()> {
        let mut pool = TsgoWorkspaceProcessPoolV0::default();
        let config = test_process_config("reuse", 30);

        let first = pool.ensure_workspace_process(config.clone())?;
        let second = pool.ensure_workspace_process(config)?;

        assert_eq!(pool.len(), 1);
        assert!(!first.reused);
        assert!(second.reused);
        assert_eq!(first.workspace_root, second.workspace_root);
        assert_eq!(first.process_id, second.process_id);
        assert!(pool.shutdown_workspace(first.workspace_root.as_str())?);
        assert!(pool.is_empty());
        Ok(())
    }

    #[test]
    fn process_pool_restarts_when_invocation_changes() -> io::Result<()> {
        let mut pool = TsgoWorkspaceProcessPoolV0::default();
        let first_config = test_process_config("restart", 30);
        let second_config = test_process_config("restart", 29);

        let first = pool.ensure_workspace_process(first_config)?;
        let second = pool.ensure_workspace_process(second_config)?;

        assert_eq!(pool.len(), 1);
        assert!(!first.reused);
        assert!(!second.reused);
        assert_ne!(first.generation, second.generation);
        assert_ne!(first.args, second.args);
        assert!(pool.shutdown_workspace(first.workspace_root.as_str())?);
        Ok(())
    }

    #[test]
    fn process_pool_restarts_existing_workspace_for_recovery() -> io::Result<()> {
        let mut pool = TsgoWorkspaceProcessPoolV0::default();
        let config = test_process_config("recover", 30);

        let first = pool.ensure_workspace_process(config)?;
        let Some(recovered) = pool.restart_workspace_process(first.workspace_root.as_str())? else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "expected existing workspace process to restart",
            ));
        };

        assert_eq!(pool.len(), 1);
        assert!(!recovered.reused);
        assert_eq!(first.workspace_root, recovered.workspace_root);
        assert_ne!(first.generation, recovered.generation);
        assert!(pool.shutdown_workspace(first.workspace_root.as_str())?);
        Ok(())
    }

    #[test]
    fn plans_type_fact_collection_without_request_path_blocking() {
        let request = TsgoTypeFactRequestV0 {
            workspace_root: "/repo".to_string(),
            config_path: "/repo/tsconfig.json".to_string(),
            targets: vec![
                TsgoTypeFactTargetV0 {
                    file_path: "/repo/src/App.tsx".to_string(),
                    expression_id: "expr-1".to_string(),
                    position: 12,
                },
                TsgoTypeFactTargetV0 {
                    file_path: "/repo/src/App.tsx".to_string(),
                    expression_id: "expr-2".to_string(),
                    position: 24,
                },
                TsgoTypeFactTargetV0 {
                    file_path: "/repo/src/Card.tsx".to_string(),
                    expression_id: "expr-3".to_string(),
                    position: 8,
                },
            ],
        };

        let plan = plan_tsgo_type_fact_collection(&request);

        assert_eq!(plan.product, "omena-tsgo-client.type-fact-collection-plan");
        assert_eq!(plan.target_count, 3);
        assert_eq!(plan.unique_file_count, 2);
        assert!(!plan.request_path_blocking_allowed);
        assert_eq!(plan.request_sequence.len(), 6);
        assert_eq!(plan.request_sequence[2].method, "getDefaultProjectForFile");
        assert_eq!(plan.request_sequence[2].request_count, 2);
        assert_eq!(plan.request_sequence[3].method, "getTypeAtPosition");
        assert_eq!(plan.request_sequence[3].request_count, 3);
    }

    fn test_process_config(label: &str, seconds: u64) -> TsgoWorkspaceProcessConfigV0 {
        let workspace_root = std::env::temp_dir()
            .join(format!("omena-tsgo-client-{label}"))
            .to_string_lossy()
            .to_string();
        let _ = std::fs::create_dir_all(workspace_root.as_str());

        TsgoWorkspaceProcessConfigV0 {
            workspace_root: workspace_root.clone(),
            command: test_process_command(workspace_root, seconds),
        }
    }

    #[cfg(not(windows))]
    fn test_process_command(cwd: String, seconds: u64) -> TsgoProcessCommandV0 {
        TsgoProcessCommandV0 {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), format!("sleep {seconds}")],
            cwd,
        }
    }

    #[cfg(windows)]
    fn test_process_command(cwd: String, seconds: u64) -> TsgoProcessCommandV0 {
        TsgoProcessCommandV0 {
            command: "cmd".to_string(),
            args: vec![
                "/C".to_string(),
                format!("ping -n {seconds} 127.0.0.1 > NUL"),
            ],
            cwd,
        }
    }

    #[derive(Debug)]
    struct FakeTsgoTransport {
        responses: VecDeque<Option<serde_json::Value>>,
        methods: Vec<String>,
        reset_count: usize,
    }

    impl FakeTsgoTransport {
        fn new(responses: Vec<serde_json::Value>) -> Self {
            Self::new_script(responses.into_iter().map(Some).collect())
        }

        fn new_script(responses: Vec<Option<serde_json::Value>>) -> Self {
            Self {
                responses: VecDeque::from(responses),
                methods: Vec::new(),
                reset_count: 0,
            }
        }
    }

    impl TsgoJsonRpcProviderTransportV0 for FakeTsgoTransport {
        fn send_json_rpc_request(
            &mut self,
            _workspace_root: &str,
            request: &TsgoJsonRpcOutboundRequestV0,
        ) -> Result<Option<serde_json::Value>, TsgoJsonRpcIoErrorV0> {
            self.methods.push(request.method.clone());
            let Some(result) = self
                .responses
                .pop_front()
                .unwrap_or(Some(serde_json::Value::Null))
            else {
                return Ok(None);
            };
            Ok(Some(json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "result": result,
            })))
        }

        fn recover_workspace(
            &mut self,
            _workspace_root: &str,
        ) -> Result<bool, TsgoJsonRpcIoErrorV0> {
            self.reset_count += 1;
            Ok(true)
        }
    }
}
