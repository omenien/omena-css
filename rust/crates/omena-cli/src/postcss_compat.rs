use crate::lock::sha256_hex;
use omena_evidence_graph::{
    EvidenceNodeKeyV0, EvidenceNodeSeedV0, ExternalToolRunWitnessV0, FamilyStampV0, GuaranteeKindV0,
};
use omena_query::{
    OmenaParserStyleDialect, OmenaQueryExternalCssSemanticDiffV0,
    compare_omena_query_external_css_semantic_changes_v0,
    omena_query_external_css_semantic_diff_is_total_v0,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    io::Read,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const PLUGIN_MANIFEST_SOURCE: &str = include_str!("../postcss-compat-plugins.json");
const NODE_BRIDGE_SOURCE: &str = include_str!("../assets/postcss-compat-runner.cjs");
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PostcssCompatManifestV0 {
    schema_version: String,
    plugins: Vec<PostcssCompatPluginV0>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PostcssCompatPluginV0 {
    id: String,
    package_name: String,
    version: String,
    config_json: String,
    config_digest: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PostcssCompatNodeRequestV0<'a> {
    project_root: &'a str,
    source_path: &'a str,
    source_css: &'a str,
    package_name: &'a str,
    expected_version: &'a str,
    config_json: &'a str,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PostcssCompatNodeResponseV0 {
    schema_version: String,
    output_css: String,
    plugin_version: String,
    warning_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum PostcssCompatFailureKindV0 {
    InvalidManifest,
    UnknownPlugin,
    SpawnFailed,
    Timeout,
    ProcessFailed,
    InvalidOutput,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PostcssCompatFailureV0 {
    pub(crate) kind: PostcssCompatFailureKindV0,
    pub(crate) plugin_id: String,
    pub(crate) message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) evidence: Option<Box<EvidenceNodeSeedV0>>,
}

impl fmt::Display for PostcssCompatFailureV0 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "PostCSS compatibility plugin '{}' failed ({:?}): {}",
            self.plugin_id, self.kind, self.message
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PostcssCompatExecutionV0 {
    pub(crate) plugin_id: String,
    pub(crate) package_name: String,
    pub(crate) plugin_version: String,
    pub(crate) config_digest: String,
    pub(crate) input_digest: String,
    pub(crate) exit_status: i32,
    pub(crate) warning_count: usize,
    pub(crate) evidence: EvidenceNodeSeedV0,
    pub(crate) semantic_diff: OmenaQueryExternalCssSemanticDiffV0,
    pub(crate) adopted: bool,
    pub(crate) output_css: String,
}

pub(crate) fn run_postcss_compat_plugin(
    plugin_id: &str,
    project_root: &Path,
    source_path: &Path,
    source_css: &str,
    dialect: OmenaParserStyleDialect,
) -> Result<PostcssCompatExecutionV0, PostcssCompatFailureV0> {
    run_postcss_compat_plugin_with_timeout(
        plugin_id,
        project_root,
        source_path,
        source_css,
        dialect,
        DEFAULT_TIMEOUT,
    )
}

fn run_postcss_compat_plugin_with_timeout(
    plugin_id: &str,
    project_root: &Path,
    source_path: &Path,
    source_css: &str,
    dialect: OmenaParserStyleDialect,
    timeout: Duration,
) -> Result<PostcssCompatExecutionV0, PostcssCompatFailureV0> {
    let manifest = parse_manifest(plugin_id)?;
    let plugin = manifest
        .plugins
        .iter()
        .find(|plugin| plugin.id == plugin_id)
        .ok_or_else(|| {
            failure(
                plugin_id,
                PostcssCompatFailureKindV0::UnknownPlugin,
                "plugin id is not present in the compiled allowlist",
            )
        })?;
    let observed_config_digest = sha256_hex(plugin.config_json.as_bytes());
    if observed_config_digest != plugin.config_digest {
        return Err(failure(
            plugin_id,
            PostcssCompatFailureKindV0::InvalidManifest,
            format!(
                "config digest mismatch: expected {}, observed {observed_config_digest}",
                plugin.config_digest
            ),
        ));
    }

    let project_root = project_root.to_str().ok_or_else(|| {
        failure(
            plugin_id,
            PostcssCompatFailureKindV0::SpawnFailed,
            "project root is not valid UTF-8",
        )
    })?;
    let source_path = source_path.to_str().ok_or_else(|| {
        failure(
            plugin_id,
            PostcssCompatFailureKindV0::SpawnFailed,
            "source path is not valid UTF-8",
        )
    })?;
    let input_digest = sha256_hex(source_css.as_bytes());
    let request = serde_json::to_vec(&PostcssCompatNodeRequestV0 {
        project_root,
        source_path,
        source_css,
        package_name: plugin.package_name.as_str(),
        expected_version: plugin.version.as_str(),
        config_json: plugin.config_json.as_str(),
    })
    .map_err(|error| {
        failure(
            plugin_id,
            PostcssCompatFailureKindV0::InvalidManifest,
            format!("failed to serialize bridge request: {error}"),
        )
    })?;
    let process = run_node_bridge(
        NODE_BRIDGE_SOURCE,
        project_root,
        request.as_slice(),
        timeout,
    )
    .map_err(|error| {
        execution_failure(
            plugin,
            input_digest.as_str(),
            error.exit_status,
            error.kind,
            error.message,
        )
    })?;
    let response = serde_json::from_slice::<PostcssCompatNodeResponseV0>(&process.stdout).map_err(
        |error| {
            execution_failure(
                plugin,
                input_digest.as_str(),
                process.exit_status,
                PostcssCompatFailureKindV0::InvalidOutput,
                format!("bridge returned malformed JSON: {error}"),
            )
        },
    )?;
    if response.schema_version != manifest.schema_version
        || response.plugin_version != plugin.version
    {
        return Err(execution_failure(
            plugin,
            input_digest.as_str(),
            process.exit_status,
            PostcssCompatFailureKindV0::InvalidOutput,
            "bridge response does not match the compiled manifest",
        ));
    }
    let semantic_diff = compare_omena_query_external_css_semantic_changes_v0(
        source_css,
        response.output_css.as_str(),
        dialect,
    );
    if !semantic_diff.all_changes_classified
        || !omena_query_external_css_semantic_diff_is_total_v0(&semantic_diff)
    {
        return Err(execution_failure(
            plugin,
            input_digest.as_str(),
            process.exit_status,
            PostcssCompatFailureKindV0::InvalidOutput,
            "Omena semantic change classification is incomplete",
        ));
    }

    Ok(PostcssCompatExecutionV0 {
        plugin_id: plugin.id.clone(),
        package_name: plugin.package_name.clone(),
        plugin_version: response.plugin_version,
        config_digest: plugin.config_digest.clone(),
        input_digest: input_digest.clone(),
        exit_status: process.exit_status,
        warning_count: response.warning_count,
        evidence: external_tool_evidence(plugin, input_digest.as_str(), process.exit_status),
        semantic_diff,
        adopted: true,
        output_css: response.output_css,
    })
}

fn parse_manifest(plugin_id: &str) -> Result<PostcssCompatManifestV0, PostcssCompatFailureV0> {
    let manifest = serde_json::from_str::<PostcssCompatManifestV0>(PLUGIN_MANIFEST_SOURCE)
        .map_err(|error| {
            failure(
                plugin_id,
                PostcssCompatFailureKindV0::InvalidManifest,
                format!("compiled manifest is invalid: {error}"),
            )
        })?;
    if manifest.schema_version != "0" {
        return Err(failure(
            plugin_id,
            PostcssCompatFailureKindV0::InvalidManifest,
            format!(
                "unsupported manifest schema version '{}'",
                manifest.schema_version
            ),
        ));
    }
    Ok(manifest)
}

#[derive(Debug)]
struct NodeBridgeOutputV0 {
    exit_status: i32,
    stdout: Vec<u8>,
}

#[derive(Debug)]
struct NodeBridgeFailureV0 {
    kind: PostcssCompatFailureKindV0,
    exit_status: i32,
    message: String,
}

fn run_node_bridge(
    script: &str,
    working_directory: &str,
    request: &[u8],
    timeout: Duration,
) -> Result<NodeBridgeOutputV0, NodeBridgeFailureV0> {
    let mut child = Command::new("node")
        .arg("-e")
        .arg(script)
        .current_dir(working_directory)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| NodeBridgeFailureV0 {
            kind: PostcssCompatFailureKindV0::SpawnFailed,
            exit_status: -1,
            message: format!("failed to start Node.js: {error}"),
        })?;

    let mut stdin = child.stdin.take().ok_or_else(|| NodeBridgeFailureV0 {
        kind: PostcssCompatFailureKindV0::ProcessFailed,
        exit_status: -1,
        message: "Node.js bridge did not expose piped stdin".to_string(),
    })?;
    let mut request_reader = request;
    std::io::copy(&mut request_reader, &mut stdin).map_err(|error| NodeBridgeFailureV0 {
        kind: PostcssCompatFailureKindV0::ProcessFailed,
        exit_status: -1,
        message: format!("failed to write bridge request: {error}"),
    })?;
    drop(stdin);

    let mut stdout = child.stdout.take().ok_or_else(|| NodeBridgeFailureV0 {
        kind: PostcssCompatFailureKindV0::ProcessFailed,
        exit_status: -1,
        message: "Node.js bridge did not expose piped stdout".to_string(),
    })?;
    let stdout_reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout.read_to_end(&mut bytes).map(|_| bytes)
    });
    let mut stderr = child.stderr.take().ok_or_else(|| NodeBridgeFailureV0 {
        kind: PostcssCompatFailureKindV0::ProcessFailed,
        exit_status: -1,
        message: "Node.js bridge did not expose piped stderr".to_string(),
    })?;
    let stderr_reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        stderr.read_to_end(&mut bytes).map(|_| bytes)
    });

    let started_at = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|error| NodeBridgeFailureV0 {
            kind: PostcssCompatFailureKindV0::ProcessFailed,
            exit_status: -1,
            message: format!("failed to read bridge status: {error}"),
        })? {
            break status;
        }
        if started_at.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout_reader.join();
            let _ = stderr_reader.join();
            return Err(NodeBridgeFailureV0 {
                kind: PostcssCompatFailureKindV0::Timeout,
                exit_status: -1,
                message: format!("Node.js bridge exceeded {} ms", timeout.as_millis()),
            });
        }
        thread::sleep(Duration::from_millis(5));
    };
    let stdout = stdout_reader
        .join()
        .map_err(|_| NodeBridgeFailureV0 {
            kind: PostcssCompatFailureKindV0::ProcessFailed,
            exit_status: -1,
            message: "stdout reader panicked".to_string(),
        })?
        .map_err(|error| NodeBridgeFailureV0 {
            kind: PostcssCompatFailureKindV0::ProcessFailed,
            exit_status: -1,
            message: format!("failed to read bridge stdout: {error}"),
        })?;
    let stderr = stderr_reader
        .join()
        .map_err(|_| NodeBridgeFailureV0 {
            kind: PostcssCompatFailureKindV0::ProcessFailed,
            exit_status: -1,
            message: "stderr reader panicked".to_string(),
        })?
        .map_err(|error| NodeBridgeFailureV0 {
            kind: PostcssCompatFailureKindV0::ProcessFailed,
            exit_status: -1,
            message: format!("failed to read bridge stderr: {error}"),
        })?;
    let exit_status = status.code().unwrap_or(1);
    if !status.success() {
        return Err(NodeBridgeFailureV0 {
            kind: PostcssCompatFailureKindV0::ProcessFailed,
            exit_status,
            message: String::from_utf8_lossy(&stderr).trim().to_string(),
        });
    }
    Ok(NodeBridgeOutputV0 {
        exit_status,
        stdout,
    })
}

fn failure(
    plugin_id: &str,
    kind: PostcssCompatFailureKindV0,
    message: impl Into<String>,
) -> PostcssCompatFailureV0 {
    PostcssCompatFailureV0 {
        kind,
        plugin_id: plugin_id.to_string(),
        message: message.into(),
        evidence: None,
    }
}

fn execution_failure(
    plugin: &PostcssCompatPluginV0,
    input_digest: &str,
    exit_status: i32,
    kind: PostcssCompatFailureKindV0,
    message: impl Into<String>,
) -> PostcssCompatFailureV0 {
    PostcssCompatFailureV0 {
        kind,
        plugin_id: plugin.id.clone(),
        message: message.into(),
        evidence: Some(Box::new(external_tool_evidence(
            plugin,
            input_digest,
            exit_status,
        ))),
    }
}

fn external_tool_evidence(
    plugin: &PostcssCompatPluginV0,
    input_digest: &str,
    exit_status: i32,
) -> EvidenceNodeSeedV0 {
    let witness = ExternalToolRunWitnessV0 {
        tool_name: plugin.package_name.clone(),
        tool_version: plugin.version.clone(),
        input_digest: input_digest.to_string(),
        exit_status,
    };
    EvidenceNodeSeedV0::with_family(
        EvidenceNodeKeyV0::new(
            "omena-cli.build.postcss-compat",
            format!("{}:{input_digest}", plugin.id),
        ),
        vec![
            format!("externalTool:{}", plugin.package_name),
            format!("toolVersion:{}", plugin.version),
            format!("configDigest:{}", plugin.config_digest),
            format!("exitStatus:{exit_status}"),
        ],
        GuaranteeKindV0::for_label_less_family(),
        FamilyStampV0::external_tool(&witness),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repository_root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
    }

    #[test]
    fn manifest_allows_only_named_plugins() -> Result<(), String> {
        let Err(error) = run_postcss_compat_plugin(
            "arbitrary-package",
            repository_root().as_path(),
            Path::new("input.css"),
            ".a { color: red; }",
            OmenaParserStyleDialect::Css,
        ) else {
            return Err("unknown plugins must be rejected before execution".to_string());
        };
        assert_eq!(error.kind, PostcssCompatFailureKindV0::UnknownPlugin);
        Ok(())
    }

    #[test]
    fn process_failure_returns_no_candidate() -> Result<(), String> {
        let repository_root = repository_root();
        let working_directory = repository_root
            .to_str()
            .ok_or_else(|| "repository path is not utf8".to_string())?;
        let Err(error) = run_node_bridge(
            "process.stderr.write('injected failure'); process.exit(7);",
            working_directory,
            b"{}",
            Duration::from_secs(1),
        ) else {
            return Err("non-zero process status must fail".to_string());
        };
        assert_eq!(error.kind, PostcssCompatFailureKindV0::ProcessFailed);
        assert!(error.message.contains("injected failure"));
        Ok(())
    }

    #[test]
    fn timeout_terminates_the_bridge() -> Result<(), String> {
        let repository_root = repository_root();
        let working_directory = repository_root
            .to_str()
            .ok_or_else(|| "repository path is not utf8".to_string())?;
        let Err(error) = run_node_bridge(
            "setTimeout(() => {}, 1000);",
            working_directory,
            b"{}",
            Duration::from_millis(20),
        ) else {
            return Err("long-running process must time out".to_string());
        };
        assert_eq!(error.kind, PostcssCompatFailureKindV0::Timeout);
        Ok(())
    }

    #[test]
    fn pinned_autoprefixer_executes_through_the_compiled_bridge() -> Result<(), String> {
        let input =
            "::placeholder { color: gray; } .input { appearance: none; user-select: none; }";
        let outcome = run_postcss_compat_plugin(
            "autoprefixer-legacy-browsers",
            repository_root().as_path(),
            Path::new("input.css"),
            input,
            OmenaParserStyleDialect::Css,
        )
        .map_err(|error| error.to_string())?;

        assert_eq!(outcome.plugin_version, "10.5.2");
        assert_ne!(outcome.output_css, input);
        assert!(outcome.output_css.contains("-webkit-appearance"));
        assert!(outcome.adopted);
        assert!(outcome.semantic_diff.all_changes_classified);
        assert!(outcome.semantic_diff.understood_change_count >= 1);
        assert!(outcome.semantic_diff.passthrough_change_count >= 1);
        assert_eq!(outcome.evidence.earned_via.describe(), "externalTool");
        Ok(())
    }

    #[test]
    fn each_plugin_invocation_receives_its_own_input_bound_witness() -> Result<(), String> {
        let root = repository_root();
        let first = run_postcss_compat_plugin(
            "autoprefixer-legacy-browsers",
            root.as_path(),
            Path::new("first.css"),
            ".first { appearance: none; }",
            OmenaParserStyleDialect::Css,
        )
        .map_err(|error| error.to_string())?;
        let second = run_postcss_compat_plugin(
            "autoprefixer-legacy-browsers",
            root.as_path(),
            Path::new("second.css"),
            ".second { user-select: none; }",
            OmenaParserStyleDialect::Css,
        )
        .map_err(|error| error.to_string())?;

        assert_ne!(first.input_digest, second.input_digest);
        assert_ne!(first.evidence.key, second.evidence.key);
        assert!(
            first
                .evidence
                .key
                .input_identity
                .ends_with(first.input_digest.as_str())
        );
        assert!(
            second
                .evidence
                .key
                .input_identity
                .ends_with(second.input_digest.as_str())
        );
        Ok(())
    }
}
