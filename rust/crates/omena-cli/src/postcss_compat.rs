use crate::lock::sha256_hex;
use omena_evidence_graph::{
    EvidenceNodeKeyV0, EvidenceNodeSeedV0, ExternalToolRunWitnessV0, FamilyStampV0, GuaranteeKindV0,
};
use omena_query::{
    OmenaParserStyleDialect, OmenaQueryExternalCssSemanticChangeClassificationV0,
    OmenaQueryExternalCssSemanticChangeKindV0, OmenaQueryExternalCssSemanticChangeV0,
    OmenaQueryExternalCssSemanticDiffV0, OmenaQueryStyleFrameRefreshParseCacheV0,
    OmenaQueryTransformTargetQueryPlanV0, compare_omena_query_external_css_semantic_changes_v0,
    omena_query_external_css_adoption_boundary_is_complete_v0,
    summarize_omena_query_style_frame_refresh_facts_with_reuse,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PostcssCompatPluginConfigV0 {
    #[serde(default)]
    override_browserslist: Vec<String>,
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
    pub(crate) configured_targets: Vec<String>,
    pub(crate) input_digest: String,
    pub(crate) exit_status: i32,
    pub(crate) warning_count: usize,
    pub(crate) evidence: EvidenceNodeSeedV0,
    pub(crate) semantic_diff: OmenaQueryExternalCssSemanticDiffV0,
    pub(crate) adoption_verdict: PostcssCompatAdoptionVerdictV0,
    pub(crate) candidate_output_css: String,
    pub(crate) output_css: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum PostcssCompatAdoptionRefusalCauseV0 {
    InputParseErrors,
    CandidateParseErrors,
    UncoveredSyntaxChanges,
    UnunderstoodSemanticChanges,
}

impl PostcssCompatAdoptionRefusalCauseV0 {
    const fn label(self) -> &'static str {
        match self {
            Self::InputParseErrors => "inputParseErrors",
            Self::CandidateParseErrors => "candidateParseErrors",
            Self::UncoveredSyntaxChanges => "uncoveredSyntaxChanges",
            Self::UnunderstoodSemanticChanges => "ununderstoodSemanticChanges",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PostcssCompatAdoptionChecksV0 {
    pub(crate) input_parse_error_count: usize,
    pub(crate) candidate_parse_error_count: usize,
    pub(crate) adoption_boundary_complete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "status",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub(crate) enum PostcssCompatAdoptionVerdictV0 {
    Adopted {
        checks: PostcssCompatAdoptionChecksV0,
    },
    Refused {
        cause: PostcssCompatAdoptionRefusalCauseV0,
        checks: PostcssCompatAdoptionChecksV0,
    },
}

impl PostcssCompatAdoptionVerdictV0 {
    pub(crate) fn human_summary(&self) -> String {
        match self {
            Self::Adopted { .. } => "adopted".to_string(),
            Self::Refused { cause, .. } => format!("refused cause={}", cause.label()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum PostcssNativeDifferentialClassificationV0 {
    Equivalent,
    NativeConservative,
    InvestigationRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PostcssNativeDifferentialV0 {
    pub(crate) schema_version: &'static str,
    pub(crate) product: &'static str,
    pub(crate) comparison_basis: &'static str,
    pub(crate) plugin_id: String,
    pub(crate) native_target_query: String,
    pub(crate) stage1_targets: Vec<String>,
    pub(crate) target_sets_aligned: bool,
    pub(crate) native_output_digest: String,
    pub(crate) stage1_output_digest: String,
    pub(crate) classification: PostcssNativeDifferentialClassificationV0,
    pub(crate) matched_uncovered_feature_ids: Vec<String>,
    pub(crate) coverage_boundary_respected: bool,
    pub(crate) requires_investigation: bool,
    pub(crate) semantic_diff: OmenaQueryExternalCssSemanticDiffV0,
}

pub(crate) fn summarize_postcss_native_differential(
    target_query: &OmenaQueryTransformTargetQueryPlanV0,
    plugin_id: &str,
    stage1_targets: &[String],
    native_output_css: &str,
    stage1_output_css: &str,
    dialect: OmenaParserStyleDialect,
) -> PostcssNativeDifferentialV0 {
    let semantic_diff = compare_omena_query_external_css_semantic_changes_v0(
        native_output_css,
        stage1_output_css,
        dialect,
    );
    let native_targets = target_query
        .resolved_targets
        .iter()
        .map(|target| normalize_compat_target(target))
        .collect::<BTreeSet<_>>();
    let configured_targets = stage1_targets
        .iter()
        .map(|target| normalize_compat_target(target))
        .collect::<BTreeSet<_>>();
    let target_sets_aligned = !native_targets.is_empty() && native_targets == configured_targets;
    let matched_uncovered_feature_ids = semantic_diff
        .changes
        .iter()
        .filter_map(|change| {
            uncovered_feature_for_external_prefix(change, target_query).map(str::to_string)
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let all_changes_match_uncovered_prefixes = !semantic_diff.changes.is_empty()
        && semantic_diff
            .changes
            .iter()
            .all(|change| uncovered_feature_for_external_prefix(change, target_query).is_some());
    let classification = if target_sets_aligned && semantic_diff.total_change_count == 0 {
        PostcssNativeDifferentialClassificationV0::Equivalent
    } else if target_sets_aligned
        && semantic_diff.cst_coverage.complete
        && all_changes_match_uncovered_prefixes
    {
        PostcssNativeDifferentialClassificationV0::NativeConservative
    } else {
        PostcssNativeDifferentialClassificationV0::InvestigationRequired
    };
    let coverage_boundary_respected =
        classification != PostcssNativeDifferentialClassificationV0::InvestigationRequired;

    PostcssNativeDifferentialV0 {
        schema_version: "0",
        product: "omena-cli.postcss-native-differential",
        comparison_basis: "semanticObservationNotByteIdentity",
        plugin_id: plugin_id.to_string(),
        native_target_query: target_query.normalized_query.clone(),
        stage1_targets: stage1_targets.to_vec(),
        target_sets_aligned,
        native_output_digest: sha256_hex(native_output_css.as_bytes()),
        stage1_output_digest: sha256_hex(stage1_output_css.as_bytes()),
        classification,
        matched_uncovered_feature_ids,
        coverage_boundary_respected,
        requires_investigation: !coverage_boundary_respected,
        semantic_diff,
    }
}

fn uncovered_feature_for_external_prefix<'a>(
    change: &OmenaQueryExternalCssSemanticChangeV0,
    target_query: &'a OmenaQueryTransformTargetQueryPlanV0,
) -> Option<&'a str> {
    if change.kind != OmenaQueryExternalCssSemanticChangeKindV0::Added
        || change.classification != OmenaQueryExternalCssSemanticChangeClassificationV0::Understood
    {
        return None;
    }
    let property = change.after.as_ref()?.property.as_str();
    let unprefixed_property = ["-webkit-", "-moz-", "-ms-", "-o-"]
        .into_iter()
        .find_map(|prefix| property.strip_prefix(prefix))?;
    target_query
        .native_stage2_coverage
        .uncovered_features
        .iter()
        .find(|feature| {
            feature.fallback == "stage1"
                && feature
                    .observed_properties
                    .iter()
                    .any(|property| property == unprefixed_property)
        })
        .map(|feature| feature.feature_id.as_str())
}

fn normalize_compat_target(target: &str) -> String {
    target
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
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
    let configured_targets =
        serde_json::from_str::<PostcssCompatPluginConfigV0>(plugin.config_json.as_str())
            .map_err(|error| {
                failure(
                    plugin_id,
                    PostcssCompatFailureKindV0::InvalidManifest,
                    format!("plugin config is invalid: {error}"),
                )
            })?
            .override_browserslist;

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
    let adoption_verdict = decide_postcss_compat_adoption(
        source_css,
        response.output_css.as_str(),
        dialect,
        &semantic_diff,
    );
    let candidate_output_css = response.output_css;
    let output_css = if matches!(
        adoption_verdict,
        PostcssCompatAdoptionVerdictV0::Adopted { .. }
    ) {
        candidate_output_css.clone()
    } else {
        source_css.to_string()
    };

    Ok(PostcssCompatExecutionV0 {
        plugin_id: plugin.id.clone(),
        package_name: plugin.package_name.clone(),
        plugin_version: response.plugin_version,
        config_digest: plugin.config_digest.clone(),
        configured_targets,
        input_digest: input_digest.clone(),
        exit_status: process.exit_status,
        warning_count: response.warning_count,
        evidence: external_tool_evidence(plugin, input_digest.as_str(), process.exit_status),
        semantic_diff,
        adoption_verdict,
        candidate_output_css,
        output_css,
    })
}

fn decide_postcss_compat_adoption(
    source_css: &str,
    candidate_output_css: &str,
    dialect: OmenaParserStyleDialect,
    semantic_diff: &OmenaQueryExternalCssSemanticDiffV0,
) -> PostcssCompatAdoptionVerdictV0 {
    let input_parse_error_count = postcss_parse_error_count(source_css, dialect);
    let candidate_parse_error_count = postcss_parse_error_count(candidate_output_css, dialect);
    let adoption_boundary_complete =
        omena_query_external_css_adoption_boundary_is_complete_v0(semantic_diff);
    let checks = PostcssCompatAdoptionChecksV0 {
        input_parse_error_count,
        candidate_parse_error_count,
        adoption_boundary_complete,
    };
    let refusal_cause = if input_parse_error_count > 0 {
        Some(PostcssCompatAdoptionRefusalCauseV0::InputParseErrors)
    } else if candidate_parse_error_count > 0 {
        Some(PostcssCompatAdoptionRefusalCauseV0::CandidateParseErrors)
    } else if semantic_diff.passthrough_change_count > 0 {
        Some(PostcssCompatAdoptionRefusalCauseV0::UnunderstoodSemanticChanges)
    } else if !adoption_boundary_complete {
        Some(PostcssCompatAdoptionRefusalCauseV0::UncoveredSyntaxChanges)
    } else {
        None
    };
    refusal_cause.map_or_else(
        || PostcssCompatAdoptionVerdictV0::Adopted {
            checks: checks.clone(),
        },
        |cause| PostcssCompatAdoptionVerdictV0::Refused {
            cause,
            checks: checks.clone(),
        },
    )
}

fn postcss_parse_error_count(source_css: &str, dialect: OmenaParserStyleDialect) -> usize {
    let mut cache = OmenaQueryStyleFrameRefreshParseCacheV0::default();
    summarize_omena_query_style_frame_refresh_facts_with_reuse(source_css, dialect, &mut cache)
        .error_count
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

    fn legacy_target_query(source: &str) -> Result<OmenaQueryTransformTargetQueryPlanV0, String> {
        omena_query::summarize_omena_query_transform_plan_from_target_query(
            "input.css",
            source,
            "Firefox 20, Safari 8",
            omena_query::conservative_omena_query_target_options(),
            omena_query::default_omena_query_transform_print_options(),
        )
        .target_query
        .ok_or_else(|| "a valid Browserslist query must produce a target plan".to_string())
    }

    fn legacy_targets() -> Vec<String> {
        vec!["Firefox 20".to_string(), "Safari 8".to_string()]
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

        assert_eq!(outcome.plugin_version, "10.5.4");
        assert_eq!(outcome.configured_targets, legacy_targets());
        assert_eq!(outcome.output_css, input);
        assert_ne!(outcome.candidate_output_css, input);
        assert!(outcome.candidate_output_css.contains("-webkit-appearance"));
        assert!(matches!(
            outcome.adoption_verdict,
            PostcssCompatAdoptionVerdictV0::Refused {
                cause: PostcssCompatAdoptionRefusalCauseV0::UnunderstoodSemanticChanges,
                ..
            }
        ));
        assert!(outcome.semantic_diff.understood_change_count >= 1);
        assert!(outcome.semantic_diff.passthrough_change_count >= 1);
        assert_eq!(outcome.evidence.earned_via.describe(), "externalTool");
        Ok(())
    }

    #[test]
    fn native_differential_accepts_semantically_equivalent_output() -> Result<(), String> {
        let source = ".input { appearance: none; }";
        let target_query = legacy_target_query(source)?;
        let report = summarize_postcss_native_differential(
            &target_query,
            "autoprefixer-legacy-browsers",
            legacy_targets().as_slice(),
            source,
            ".input { appearance: none; }\n",
            OmenaParserStyleDialect::Css,
        );

        assert_eq!(
            report.classification,
            PostcssNativeDifferentialClassificationV0::Equivalent
        );
        assert!(report.target_sets_aligned);
        assert!(report.coverage_boundary_respected);
        assert!(!report.requires_investigation);
        assert_eq!(report.semantic_diff.total_change_count, 0);
        Ok(())
    }

    #[test]
    fn native_differential_accepts_only_declared_stage1_prefix_coverage() -> Result<(), String> {
        let source = ".input { hyphens: auto; }";
        let target_query = legacy_target_query(source)?;
        let report = summarize_postcss_native_differential(
            &target_query,
            "autoprefixer-legacy-browsers",
            legacy_targets().as_slice(),
            ".input { -webkit-hyphens: auto; hyphens: auto; }",
            ".input { -webkit-hyphens: auto; -moz-hyphens: auto; hyphens: auto; }",
            OmenaParserStyleDialect::Css,
        );

        assert_eq!(
            report.classification,
            PostcssNativeDifferentialClassificationV0::NativeConservative
        );
        assert_eq!(
            report.matched_uncovered_feature_ids,
            vec!["vendor-prefixing.hyphens"]
        );
        assert!(report.coverage_boundary_respected);
        assert!(!report.requires_investigation);
        Ok(())
    }

    #[test]
    fn native_differential_escalates_unexplained_or_misaligned_changes() -> Result<(), String> {
        let source = ".input { color: red; }";
        let target_query = legacy_target_query(source)?;
        let unexplained = summarize_postcss_native_differential(
            &target_query,
            "autoprefixer-legacy-browsers",
            legacy_targets().as_slice(),
            source,
            ".input { color: blue; }",
            OmenaParserStyleDialect::Css,
        );
        let misaligned = summarize_postcss_native_differential(
            &target_query,
            "autoprefixer-legacy-browsers",
            &["Chrome 123".to_string()],
            source,
            source,
            OmenaParserStyleDialect::Css,
        );

        assert_eq!(
            unexplained.classification,
            PostcssNativeDifferentialClassificationV0::InvestigationRequired
        );
        assert!(unexplained.requires_investigation);
        assert!(!unexplained.coverage_boundary_respected);
        assert_eq!(
            misaligned.classification,
            PostcssNativeDifferentialClassificationV0::InvestigationRequired
        );
        assert!(!misaligned.target_sets_aligned);
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
        assert!(matches!(
            first.adoption_verdict,
            PostcssCompatAdoptionVerdictV0::Adopted { .. }
        ));
        assert!(matches!(
            second.adoption_verdict,
            PostcssCompatAdoptionVerdictV0::Adopted { .. }
        ));
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

    #[test]
    fn parse_errors_refuse_candidate_adoption() {
        let source = ".a { color: red; }";
        let candidate = ".a { color: red;";
        let semantic_diff = compare_omena_query_external_css_semantic_changes_v0(
            source,
            candidate,
            OmenaParserStyleDialect::Css,
        );
        let verdict = decide_postcss_compat_adoption(
            source,
            candidate,
            OmenaParserStyleDialect::Css,
            &semantic_diff,
        );

        assert!(matches!(
            verdict,
            PostcssCompatAdoptionVerdictV0::Refused {
                cause: PostcssCompatAdoptionRefusalCauseV0::CandidateParseErrors,
                checks: PostcssCompatAdoptionChecksV0 {
                    input_parse_error_count: 0,
                    candidate_parse_error_count: 1..,
                    ..
                }
            }
        ));
    }

    #[test]
    fn unobserved_css_syntax_changes_refuse_candidate_adoption() {
        let cases = [
            (
                "font-face-deletion",
                "@font-face { font-family: Demo; src: url(demo.woff2); } .a { color: red; }",
                ".a { color: red; }",
            ),
            (
                "font-face-modification",
                "@font-face { font-family: Demo; src: url(demo.woff2); }",
                "@font-face { font-family: Demo; src: url(poisoned.woff2); }",
            ),
            (
                "counter-style-deletion",
                r#"@counter-style thumbs { system: cyclic; symbols: "👍"; } .a { color: red; }"#,
                ".a { color: red; }",
            ),
            (
                "page-deletion",
                "@page { margin: 1cm; } .a { color: red; }",
                ".a { color: red; }",
            ),
            (
                "same-rule-shorthand-longhand-reorder",
                ".box { margin-left: 5px; margin: 0; }",
                ".box { margin: 0; margin-left: 5px; }",
            ),
            (
                "understood-prefix-does-not-mask-font-face-deletion",
                "@font-face { font-family: Demo; src: url(demo.woff2); } .a { appearance: none; }",
                ".a { -webkit-appearance: none; appearance: none; }",
            ),
        ];

        let mut silently_adopted = Vec::new();
        for (name, source, candidate) in cases {
            let semantic_diff = compare_omena_query_external_css_semantic_changes_v0(
                source,
                candidate,
                OmenaParserStyleDialect::Css,
            );
            let verdict = decide_postcss_compat_adoption(
                source,
                candidate,
                OmenaParserStyleDialect::Css,
                &semantic_diff,
            );

            if !matches!(
                verdict,
                PostcssCompatAdoptionVerdictV0::Refused {
                    cause: PostcssCompatAdoptionRefusalCauseV0::UncoveredSyntaxChanges,
                    checks: PostcssCompatAdoptionChecksV0 {
                        adoption_boundary_complete: false,
                        ..
                    }
                }
            ) || semantic_diff.cst_coverage.complete
            {
                silently_adopted.push(format!(
                    "{name}: {verdict:?}, totalChangeCount={}",
                    semantic_diff.total_change_count
                ));
            }
        }
        assert!(
            silently_adopted.is_empty(),
            "unobserved syntax changes were silently adopted: {silently_adopted:#?}"
        );
    }
}
