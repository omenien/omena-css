use std::{
    fs,
    path::{Path, PathBuf},
};

use globset::{Glob, GlobSet, GlobSetBuilder};
use omena_query::{
    OmenaQueryCssModulesInterfaceBundleV0, OmenaQueryCssModulesInterfaceSummaryViewV0,
    render_omena_query_css_module_typescript_declaration,
    render_omena_query_css_modules_interface_json, summarize_cross_file_summary_view_v0,
    summarize_omena_query_css_modules_export_usage,
    summarize_omena_query_css_modules_interface_bundle,
    summarize_omena_query_css_modules_interface_summary_view,
    summarize_omena_query_workspace_cross_file_summary,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{
    commands::ModulesCommand,
    config::{find_omena_config_for_path, resolve_config_path},
    io::{read_package_manifests, read_source_documents, read_style_sources},
    lint::discover_workspace_files,
    output::{CliOutputMetadataV0, print_json},
    paths::path_string,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModulesMode {
    Emit,
    Check,
}

impl ModulesMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Emit => "emit",
            Self::Check => "check",
        }
    }
}

#[derive(Debug)]
struct ModulesOptions {
    mode: ModulesMode,
    root: Option<PathBuf>,
    declaration_dir: Option<PathBuf>,
    interface_file: Option<PathBuf>,
    json: bool,
}

#[derive(Debug, Clone)]
struct ModuleArtifactPlanV0 {
    path: PathBuf,
    kind: &'static str,
    content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct ModuleArtifactReportV0 {
    path: String,
    kind: &'static str,
    status: &'static str,
    expected_byte_length: usize,
    expected_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct ModulesReportV0 {
    schema_version: &'static str,
    product: &'static str,
    mode: &'static str,
    workspace_root: String,
    hash_strategy: String,
    module_count: usize,
    class_export_count: usize,
    icss_export_count: usize,
    unused_export_count: usize,
    skipped_export_count: usize,
    artifact_count: usize,
    drift_count: usize,
    summary_view: OmenaQueryCssModulesInterfaceSummaryViewV0,
    artifacts: Vec<ModuleArtifactReportV0>,
}

pub(crate) fn modules_command(command: ModulesCommand) -> Result<(), String> {
    let options = match command {
        ModulesCommand::Emit {
            root,
            declaration_dir,
            interface_file,
            json,
        } => ModulesOptions {
            mode: ModulesMode::Emit,
            root,
            declaration_dir,
            interface_file,
            json,
        },
        ModulesCommand::Check {
            root,
            declaration_dir,
            interface_file,
            json,
        } => ModulesOptions {
            mode: ModulesMode::Check,
            root,
            declaration_dir,
            interface_file,
            json,
        },
    };
    run_modules(options)
}

fn run_modules(options: ModulesOptions) -> Result<(), String> {
    let root = options.root.unwrap_or_else(|| PathBuf::from("."));
    let workspace_root = fs::canonicalize(&root).map_err(|error| {
        format!(
            "failed to resolve modules workspace {}: {error}",
            path_string(root.as_path())
        )
    })?;
    let loaded_config = find_omena_config_for_path(&workspace_root)?;
    let config_directory = loaded_config
        .as_ref()
        .map(|loaded| loaded.directory.as_path())
        .unwrap_or(workspace_root.as_path());
    let config = loaded_config.as_ref().map(|loaded| &loaded.config.modules);
    let include =
        compile_include_globs(config.map_or(&[][..], |config| config.include.as_slice()))?;
    let files = discover_workspace_files(workspace_root.as_path())?;
    let module_paths = files
        .style_paths
        .iter()
        .filter(|path| is_css_module_path(path))
        .filter(|path| include_matches(&include, workspace_root.as_path(), path))
        .cloned()
        .collect::<Vec<_>>();
    if module_paths.is_empty() {
        return Err(format!(
            "no CSS Module sources matched under {}",
            path_string(workspace_root.as_path())
        ));
    }

    let style_sources = read_style_sources(module_paths.as_slice())?;
    let source_documents = read_source_documents(files.source_paths.as_slice())?;
    let package_manifests = read_package_manifests(files.package_manifest_paths.as_slice())?;
    let bundle = summarize_omena_query_css_modules_interface_bundle(
        style_sources.as_slice(),
        package_manifests.as_slice(),
    );
    let usage = summarize_omena_query_css_modules_export_usage(
        style_sources.as_slice(),
        source_documents.as_slice(),
        package_manifests.as_slice(),
        None,
    );
    let workspace_summary = summarize_omena_query_workspace_cross_file_summary(
        style_sources.as_slice(),
        source_documents.as_slice(),
        package_manifests.as_slice(),
    );
    let cross_file_summary_view = summarize_cross_file_summary_view_v0(&workspace_summary);
    let summary_view = summarize_omena_query_css_modules_interface_summary_view(
        &cross_file_summary_view,
        &bundle,
        &usage,
    );

    let configured_declaration_dir = config.and_then(|config| config.declaration_dir.as_deref());
    let declaration_dir = options
        .declaration_dir
        .as_deref()
        .or(configured_declaration_dir)
        .map(|path| resolve_config_path(config_directory, path));
    let configured_interface_file = config.and_then(|config| config.interface_file.as_deref());
    let interface_file = options
        .interface_file
        .as_deref()
        .or(configured_interface_file)
        .map(|path| resolve_config_path(config_directory, path))
        .unwrap_or_else(|| workspace_root.join("omena.modules.json"));
    let typed_definitions = config
        .and_then(|config| config.typed_definitions)
        .unwrap_or(true);
    let hash_strategy = config
        .and_then(|config| config.hash_strategy.clone())
        .unwrap_or_else(|| "stable".to_string());
    let plans = plan_module_artifacts(
        workspace_root.as_path(),
        declaration_dir.as_deref(),
        interface_file,
        typed_definitions,
        &bundle,
    )?;
    let artifacts = apply_or_check_module_artifacts(options.mode, plans.as_slice())?;
    let drift_count = artifacts
        .iter()
        .filter(|artifact| matches!(artifact.status, "missing" | "changed"))
        .count();
    let report = ModulesReportV0 {
        schema_version: "0",
        product: "omena-cli.modules",
        mode: options.mode.as_str(),
        workspace_root: path_string(workspace_root.as_path()),
        hash_strategy,
        module_count: bundle.module_count,
        class_export_count: bundle.class_export_count,
        icss_export_count: bundle.icss_export_count,
        unused_export_count: usage.unused_export_count,
        skipped_export_count: usage.skipped_export_count,
        artifact_count: artifacts.len(),
        drift_count,
        summary_view,
        artifacts,
    };

    if options.json {
        print_json(
            CliOutputMetadataV0::new("omena-cli.modules").with_config_content_digest(
                loaded_config
                    .as_ref()
                    .map(|loaded| loaded.config_content_digest.as_ref()),
            ),
            &report,
        )?;
    } else {
        println!(
            "modules {}: {} module(s), {} artifact(s), {} drifted",
            report.mode, report.module_count, report.artifact_count, report.drift_count
        );
        for artifact in report
            .artifacts
            .iter()
            .filter(|artifact| module_artifact_is_drifted(artifact))
        {
            println!("{}: {}", artifact.status, artifact.path);
        }
        for diagnostic in &usage.diagnostics {
            println!("{}: {}", diagnostic.style_path, diagnostic.message);
        }
    }

    if options.mode == ModulesMode::Check && drift_count > 0 {
        return Err(format!(
            "CSS Modules interface drift detected in {drift_count} artifact(s): {}",
            module_artifact_drift_summary(report.artifacts.as_slice())
        ));
    }
    Ok(())
}

fn module_artifact_is_drifted(artifact: &ModuleArtifactReportV0) -> bool {
    matches!(artifact.status, "missing" | "changed")
}

fn module_artifact_drift_summary(artifacts: &[ModuleArtifactReportV0]) -> String {
    artifacts
        .iter()
        .filter(|artifact| module_artifact_is_drifted(artifact))
        .map(|artifact| format!("{}:{}", artifact.status, artifact.path))
        .collect::<Vec<_>>()
        .join(", ")
}

fn plan_module_artifacts(
    workspace_root: &Path,
    declaration_dir: Option<&Path>,
    interface_file: PathBuf,
    typed_definitions: bool,
    bundle: &OmenaQueryCssModulesInterfaceBundleV0,
) -> Result<Vec<ModuleArtifactPlanV0>, String> {
    let mut plans = Vec::new();
    if typed_definitions {
        for module in &bundle.modules {
            let source_path = Path::new(module.style_path.as_str());
            let path = declaration_output_path(workspace_root, source_path, declaration_dir);
            plans.push(ModuleArtifactPlanV0 {
                path,
                kind: "typescriptDeclaration",
                content: render_omena_query_css_module_typescript_declaration(module),
            });
        }
    }
    plans.push(ModuleArtifactPlanV0 {
        path: interface_file,
        kind: "moduleInterfaceJson",
        content: render_omena_query_css_modules_interface_json(bundle)
            .map_err(|error| format!("failed to serialize module-interface JSON: {error}"))?,
    });
    plans.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(plans)
}

fn declaration_output_path(
    workspace_root: &Path,
    source_path: &Path,
    declaration_dir: Option<&Path>,
) -> PathBuf {
    let mut output = match declaration_dir {
        Some(directory) => directory.join(source_path.strip_prefix(workspace_root).unwrap_or_else(
            |_| {
                source_path
                    .file_name()
                    .map(Path::new)
                    .unwrap_or(source_path)
            },
        )),
        None => source_path.to_path_buf(),
    };
    let file_name = output
        .file_name()
        .map(|name| format!("{}.d.ts", name.to_string_lossy()))
        .unwrap_or_else(|| "module.css.d.ts".to_string());
    output.set_file_name(file_name);
    output
}

fn apply_or_check_module_artifacts(
    mode: ModulesMode,
    plans: &[ModuleArtifactPlanV0],
) -> Result<Vec<ModuleArtifactReportV0>, String> {
    let mut reports = Vec::with_capacity(plans.len());
    for plan in plans {
        let existing = fs::read(plan.path.as_path()).ok();
        let matches = existing
            .as_deref()
            .is_some_and(|bytes| bytes == plan.content.as_bytes());
        let status = match (mode, existing.is_some(), matches) {
            (_, _, true) => "matched",
            (ModulesMode::Check, false, false) => "missing",
            (ModulesMode::Check, true, false) => "changed",
            (ModulesMode::Emit, _, false) => {
                write_module_artifact(plan.path.as_path(), plan.content.as_bytes())?;
                "written"
            }
        };
        reports.push(ModuleArtifactReportV0 {
            path: path_string(plan.path.as_path()),
            kind: plan.kind,
            status,
            expected_byte_length: plan.content.len(),
            expected_sha256: sha256_hex(plan.content.as_bytes()),
        });
    }
    Ok(reports)
}

fn write_module_artifact(path: &Path, content: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create module artifact directory {}: {error}",
                path_string(parent)
            )
        })?;
    }
    fs::write(path, content)
        .map_err(|error| format!("failed to write {}: {error}", path_string(path)))
}

fn compile_include_globs(patterns: &[String]) -> Result<Option<GlobSet>, String> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(
            Glob::new(pattern)
                .map_err(|error| format!("invalid [modules].include glob '{pattern}': {error}"))?,
        );
    }
    builder
        .build()
        .map(Some)
        .map_err(|error| format!("failed to build [modules].include matcher: {error}"))
}

fn include_matches(include: &Option<GlobSet>, root: &Path, path: &Path) -> bool {
    include.as_ref().is_none_or(|include| {
        let relative = path.strip_prefix(root).unwrap_or(path);
        include.is_match(relative)
    })
}

fn is_css_module_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.contains(".module."))
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{commands::Cli, dispatch::run_with_exit};
    use clap::Parser;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn css_modules_interface_emit_then_check_detects_byte_drift() -> Result<(), String> {
        let root = fixture_root();
        fs::create_dir_all(root.join("src")).map_err(|error| error.to_string())?;
        fs::write(
            root.join("src/Card.module.css"),
            ":export { tone: #0af; } .used {} .ghost {}",
        )
        .map_err(|error| error.to_string())?;
        fs::write(
            root.join("src/Card.tsx"),
            r#"import styles from "./Card.module.css";
export const Card = () => <div className={styles.used} />;"#,
        )
        .map_err(|error| error.to_string())?;
        fs::write(
            root.join("omena.toml"),
            "[modules]\ntypedDefinitions = true\ninclude = [\"src/**/*.module.css\"]\ndeclarationDir = \"generated/types\"\ninterfaceFile = \"generated/modules.json\"\n",
        )
        .map_err(|error| error.to_string())?;

        let root_arg = root.to_string_lossy().into_owned();
        let emit = Cli::try_parse_from(["omena", "modules", "emit", root_arg.as_str()])
            .map_err(|error| error.to_string())?;
        run_with_exit(emit).map_err(|error| error.to_string())?;
        let declaration = root.join("generated/types/src/Card.module.css.d.ts");
        let interface = root.join("generated/modules.json");
        assert!(declaration.is_file());
        assert!(interface.is_file());
        assert!(
            fs::read_to_string(&declaration)
                .map_err(|error| error.to_string())?
                .contains("readonly \"used\": string;")
        );
        assert!(
            fs::read_to_string(&declaration)
                .map_err(|error| error.to_string())?
                .contains("readonly \"tone\": string;")
        );
        assert!(
            fs::read_to_string(&interface)
                .map_err(|error| error.to_string())?
                .contains("\"value\": \"#0af\"")
        );

        let check = Cli::try_parse_from(["omena", "modules", "check", root_arg.as_str()])
            .map_err(|error| error.to_string())?;
        run_with_exit(check).map_err(|error| error.to_string())?;
        fs::write(&declaration, "stale\n").map_err(|error| error.to_string())?;
        let stale_check = Cli::try_parse_from(["omena", "modules", "check", root_arg.as_str()])
            .map_err(|error| error.to_string())?;
        let error = match run_with_exit(stale_check) {
            Ok(()) => return Err("stale declaration unexpectedly passed check".to_string()),
            Err(error) => error.to_string(),
        };
        assert!(error.contains("interface drift detected in 1 artifact"));
        assert!(error.contains("changed:"));
        assert!(error.contains(declaration.to_string_lossy().as_ref()));

        fs::remove_dir_all(root).map_err(|error| error.to_string())?;
        Ok(())
    }

    fn fixture_root() -> PathBuf {
        let id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("omena-modules-{}-{id}", std::process::id()))
    }
}
