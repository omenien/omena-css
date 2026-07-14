use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use omena_evidence_graph::{
    EvidenceNodeKeyV0, EvidenceNodeSeedV0, ExternalToolRunWitnessV0, FamilyStampV0, GuaranteeKindV0,
};
use serde::{Deserialize, Serialize};

use super::{SassModuleGraphViewV0, build_sass_graph_view};
use crate::{
    config::find_omena_config_for_path,
    io::read_source,
    lock::sha256_hex,
    output::{CliOutputMetadataV0, print_json, write_artifact},
    paths::path_string,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct DartSassCompilerV0 {
    name: String,
    package: String,
    version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DartSassBridgeResultV0 {
    schema_version: String,
    product: String,
    compiler: DartSassCompilerV0,
    entry: String,
    exit_status: i32,
    css: Option<String>,
    stderr: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct SassCompileGraphReferenceV0 {
    from_style_path: String,
    source: String,
    status: &'static str,
    resolved_style_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct SassCompileBridgeReportV0 {
    schema_version: &'static str,
    product: &'static str,
    authority: String,
    compiler: DartSassCompilerV0,
    entry: String,
    exit_status: i32,
    compiled: bool,
    css: Option<String>,
    stderr: String,
    unresolved_module_edge_count: usize,
    visibility_filter_count: usize,
    invalid_configuration_count: usize,
    graph_references: Vec<SassCompileGraphReferenceV0>,
    external_tool_evidence: EvidenceNodeSeedV0,
}

pub(super) fn sass_compile(
    entry: PathBuf,
    output: Option<PathBuf>,
    json: bool,
) -> Result<(), String> {
    let entry = std::fs::canonicalize(&entry).map_err(|error| {
        format!(
            "failed to resolve Sass entry {}: {error}",
            path_string(entry.as_path())
        )
    })?;
    let loaded_config = find_omena_config_for_path(entry.as_path())?;
    if let Some(oracle) = loaded_config
        .as_ref()
        .and_then(|loaded| loaded.config.sass.oracle.as_deref())
        && oracle != "dart-sass"
    {
        return Err(format!(
            "unsupported Sass compile authority '{oracle}'; expected dart-sass"
        ));
    }
    let graph = build_sass_graph_view(entry.parent().map(Path::to_path_buf), Some(entry.clone()))?;
    let source = read_source(entry.as_path())?;
    let bridge = run_pinned_dart_sass(entry.as_path())?;
    let report = build_compile_report(&source, graph, bridge)?;

    if json {
        print_json(
            CliOutputMetadataV0::new("omena-cli.sass.compile").with_config_content_digest(
                loaded_config
                    .as_ref()
                    .map(|loaded| loaded.config_content_digest.as_ref()),
            ),
            &report,
        )?;
    } else if report.compiled {
        let css = report.css.as_deref().unwrap_or_default();
        if let Some(output) = output.as_deref() {
            write_artifact(output, css.as_bytes())?;
            println!("{}; wrote {}", report.authority, path_string(output));
        } else {
            eprintln!("{}", report.authority);
            print!("{css}");
        }
    }

    if !report.compiled {
        return Err(format!(
            "{} failed for {}: {}{}",
            report.authority,
            report.entry,
            compact_stderr(report.stderr.as_str()),
            failure_graph_context(&report),
        ));
    }
    Ok(())
}

fn build_compile_report(
    source: &str,
    graph: SassModuleGraphViewV0,
    bridge: DartSassBridgeResultV0,
) -> Result<SassCompileBridgeReportV0, String> {
    if bridge.schema_version != "0"
        || bridge.product != "omena-cli.sass-compile-bridge-result"
        || bridge.compiler.name != "dart-sass"
        || bridge.compiler.package != "sass"
        || bridge.compiler.version != "1.101.0"
    {
        return Err("Dart Sass compile bridge returned an unsupported contract".to_string());
    }
    if graph.selected_module.as_deref() != Some(bridge.entry.as_str()) {
        return Err("Dart Sass compile bridge returned a mismatched entry identity".to_string());
    }
    let compiled = bridge.exit_status == 0;
    if compiled != bridge.css.is_some() {
        return Err("Dart Sass compile bridge returned inconsistent CSS coverage".to_string());
    }
    let input_digest = sha256_hex(source.as_bytes());
    let external_tool_evidence = external_tool_evidence(
        bridge.compiler.version.as_str(),
        input_digest.as_str(),
        bridge.exit_status,
    );
    let graph_references = graph
        .edges
        .iter()
        .map(|edge| SassCompileGraphReferenceV0 {
            from_style_path: edge.from_style_path.clone(),
            source: edge.source.clone(),
            status: edge.status,
            resolved_style_path: edge.resolved_style_path.clone(),
        })
        .collect::<Vec<_>>();
    let invalid_configuration_count = graph
        .edges
        .iter()
        .map(|edge| edge.invalid_configuration_variable_names.len())
        .sum();
    Ok(SassCompileBridgeReportV0 {
        schema_version: "0",
        product: "omena-cli.sass.compile",
        authority: format!("compiled by dart-sass {}", bridge.compiler.version),
        compiler: bridge.compiler,
        entry: bridge.entry,
        exit_status: bridge.exit_status,
        compiled,
        css: bridge.css,
        stderr: bridge.stderr,
        unresolved_module_edge_count: graph.unresolved_module_edge_count,
        visibility_filter_count: graph.visibility_filter_count,
        invalid_configuration_count,
        graph_references,
        external_tool_evidence,
    })
}

fn run_pinned_dart_sass(entry: &Path) -> Result<DartSassBridgeResultV0, String> {
    let repo_root = find_toolchain_root()?;
    let script = repo_root.join("scripts/run-sass-compile-bridge.ts");
    let output = Command::new("node")
        .args(["--import", "tsx"])
        .arg(script)
        .arg(entry)
        .current_dir(repo_root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| format!("failed to start the pinned Dart Sass bridge: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "pinned Dart Sass bridge failed: {}",
            compact_stderr(String::from_utf8_lossy(output.stderr.as_slice()).as_ref())
        ));
    }
    serde_json::from_slice(output.stdout.as_slice())
        .map_err(|error| format!("failed to parse the Dart Sass bridge result: {error}"))
}

fn find_toolchain_root() -> Result<PathBuf, String> {
    if let Some(root) = std::env::var_os("OMENA_TOOLCHAIN_ROOT").map(PathBuf::from)
        && root.join("scripts/run-sass-compile-bridge.ts").is_file()
    {
        return Ok(root);
    }
    let current = std::env::current_dir()
        .map_err(|error| format!("failed to resolve the current directory: {error}"))?;
    if let Some(root) = current.ancestors().find(|candidate| {
        candidate
            .join("scripts/run-sass-compile-bridge.ts")
            .is_file()
    }) {
        return Ok(root.to_path_buf());
    }
    let build_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    if build_root
        .join("scripts/run-sass-compile-bridge.ts")
        .is_file()
    {
        return Ok(build_root);
    }
    Err("could not locate the shared pinned Dart Sass bridge; set OMENA_TOOLCHAIN_ROOT".to_string())
}

fn external_tool_evidence(
    version: &str,
    input_digest: &str,
    exit_status: i32,
) -> EvidenceNodeSeedV0 {
    let witness = ExternalToolRunWitnessV0 {
        tool_name: "dart-sass".to_string(),
        tool_version: version.to_string(),
        input_digest: input_digest.to_string(),
        exit_status,
    };
    EvidenceNodeSeedV0::with_family(
        EvidenceNodeKeyV0::new("omena-cli.sass.compile", input_digest),
        vec![
            "externalTool:dart-sass".to_string(),
            format!("toolVersion:{version}"),
            format!("exitStatus:{exit_status}"),
        ],
        GuaranteeKindV0::for_label_less_family(),
        FamilyStampV0::external_tool(&witness),
    )
}

fn compact_stderr(stderr: &str) -> String {
    stderr
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn failure_graph_context(report: &SassCompileBridgeReportV0) -> String {
    let references = report
        .graph_references
        .iter()
        .filter(|reference| reference.status == "unresolved")
        .map(|reference| format!("{} -> {}", reference.from_style_path, reference.source))
        .collect::<Vec<_>>();
    if references.is_empty() {
        String::new()
    } else {
        format!(
            "; Omena graph unresolved references: {}",
            references.join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omena_evidence_graph::GuaranteeFamilyV0;
    use omena_query::OmenaQuerySassModuleCrossFileResolutionCapabilitiesV0;

    #[test]
    fn compile_report_names_authority_and_carries_one_execution_witness() -> Result<(), String> {
        let report = build_compile_report(
            ".app { color: red; }",
            empty_graph(),
            DartSassBridgeResultV0 {
                schema_version: "0".to_string(),
                product: "omena-cli.sass-compile-bridge-result".to_string(),
                compiler: DartSassCompilerV0 {
                    name: "dart-sass".to_string(),
                    package: "sass".to_string(),
                    version: "1.101.0".to_string(),
                },
                entry: "/workspace/app.scss".to_string(),
                exit_status: 0,
                css: Some(".app {\n  color: red;\n}\n".to_string()),
                stderr: String::new(),
            },
        )?;
        assert_eq!(report.authority, "compiled by dart-sass 1.101.0");
        assert!(report.compiled);
        assert_eq!(
            report.external_tool_evidence.earned_via,
            GuaranteeFamilyV0::ExternalTool
        );
        assert_eq!(report.external_tool_evidence.provenance.len(), 3);
        Ok(())
    }

    fn empty_graph() -> SassModuleGraphViewV0 {
        SassModuleGraphViewV0 {
            schema_version: "0",
            product: "omena-cli.sass.graph",
            workspace_root: "/workspace".to_string(),
            selected_module: Some("/workspace/app.scss".to_string()),
            style_count: 1,
            module_edge_count: 0,
            resolved_module_edge_count: 0,
            unresolved_module_edge_count: 0,
            external_module_edge_count: 0,
            configured_module_instance_count: 0,
            visibility_filter_count: 0,
            graph_closure_edge_count: 0,
            cycle_count: 0,
            edges: Vec::new(),
            graph_closure_edges: Vec::new(),
            cycles: Vec::new(),
            capabilities: OmenaQuerySassModuleCrossFileResolutionCapabilitiesV0 {
                omena_parser_module_edge_consumption_ready: true,
                resolver_backed_source_resolution_ready: true,
                package_manifest_resolution_ready: true,
                external_module_filtering_ready: true,
                graph_closure_ready: true,
                cycle_detection_ready: true,
                namespace_show_hide_filter_ready: true,
                configured_module_instance_identity_ready: true,
                symlink_chain_metadata_ready: true,
            },
        }
    }
}
