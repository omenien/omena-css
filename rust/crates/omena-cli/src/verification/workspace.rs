use std::{collections::BTreeMap, fs, path::PathBuf};

use omena_query::{
    OmenaQueryClosedWorldOutcomeV0, load_omena_query_workspace_style_resolution_inputs,
    summarize_omena_query_consumer_check_style_source,
    summarize_omena_query_workspace_cross_file_summary_with_resolution_inputs,
};
use serde_json::{Value, json};

use crate::{
    bundle::{BundleCommandOptions, plan_bundle},
    config::{OmenaTranslationValidationMode, find_omena_config_for_path, resolve_config_paths},
    format::build_format_report,
    io::{read_package_manifests, read_source, read_source_documents, read_style_sources},
    lint::{discover_style_paths, discover_workspace_files},
    modules::{has_css_module_sources, modules_check_report},
    paths::{path_string, style_resolution_workspace_uri_for_path},
    sass::sass_compile_report,
};

use super::{
    manifest::{
        VerificationAvailabilityV0, VerificationExecutorV0, VerificationScopeV0,
        VerificationTargetV0, translation_validation_binding, verification_manifest,
    },
    report::{VerificationItemReportV0, VerificationOutcomeV0, VerificationReportV0},
};

pub(crate) struct VerificationExecutionV0 {
    pub(crate) report: VerificationReportV0,
    pub(crate) config_content_digest: Option<String>,
    pub(crate) warnings: Vec<String>,
}

struct WorkspaceVerificationContextV0 {
    root: PathBuf,
    style_paths: Vec<PathBuf>,
    source_paths: Vec<PathBuf>,
    package_manifest_paths: Vec<PathBuf>,
    bundle_entries: Vec<PathBuf>,
    bundle_sources: Vec<PathBuf>,
    bundle_package_manifests: Vec<PathBuf>,
    evidence_policy: Option<String>,
    translation_validation: OmenaTranslationValidationMode,
    external_corpus_policy: Option<String>,
    config_content_digest: Option<String>,
    warnings: Vec<String>,
}

pub(crate) fn verify_workspace(
    root: Option<PathBuf>,
    engine_self: bool,
) -> Result<VerificationExecutionV0, String> {
    let context = load_context(root)?;
    let manifest = verification_manifest()?;
    let mut items = Vec::new();
    for target in manifest.targets {
        if target.scope == VerificationScopeV0::EngineSelf && !engine_self {
            continue;
        }
        items.push(execute_target(&context, target));
    }
    if context.evidence_policy.as_deref() == Some("required") {
        for item in &mut items {
            if item.outcome != VerificationOutcomeV0::Skipped && item.evidence.is_empty() {
                item.outcome = VerificationOutcomeV0::Indeterminate;
                item.summary = "required evidence references are missing".to_string();
            }
        }
    }
    let report = VerificationReportV0::new(path_string(context.root.as_path()), engine_self, items);
    Ok(VerificationExecutionV0 {
        report,
        config_content_digest: context.config_content_digest,
        warnings: context.warnings,
    })
}

fn load_context(root: Option<PathBuf>) -> Result<WorkspaceVerificationContextV0, String> {
    let requested_root = root.unwrap_or_else(|| PathBuf::from("."));
    let root = fs::canonicalize(&requested_root).map_err(|error| {
        format!(
            "failed to resolve verification root {}: {error}",
            path_string(requested_root.as_path())
        )
    })?;
    let files = discover_workspace_files(root.as_path())?;
    let loaded_config = find_omena_config_for_path(root.as_path())?;
    let config_directory = loaded_config
        .as_ref()
        .map(|loaded| loaded.directory.as_path())
        .unwrap_or(root.as_path());
    let bundle_entries = loaded_config
        .as_ref()
        .and_then(|loaded| loaded.config.build.bundle_entries.as_deref())
        .map(|paths| resolve_config_paths(config_directory, paths))
        .unwrap_or_default();
    let bundle_sources = loaded_config
        .as_ref()
        .and_then(|loaded| loaded.config.build.sources.as_deref())
        .map(|paths| resolve_config_paths(config_directory, paths))
        .unwrap_or_else(|| files.style_paths.clone());
    let bundle_package_manifests = loaded_config
        .as_ref()
        .and_then(|loaded| loaded.config.build.package_manifests.as_deref())
        .map(|paths| resolve_config_paths(config_directory, paths))
        .unwrap_or_else(|| files.package_manifest_paths.clone());
    let evidence_policy = loaded_config
        .as_ref()
        .and_then(|loaded| loaded.config.verify.evidence.clone());
    if evidence_policy
        .as_deref()
        .is_some_and(|value| value != "required")
    {
        return Err("unsupported [verify].evidence value; expected required".to_string());
    }
    let external_corpus_policy = loaded_config
        .as_ref()
        .and_then(|loaded| loaded.config.verify.external_corpus.clone());
    if external_corpus_policy
        .as_deref()
        .is_some_and(|value| value != "advisory")
    {
        return Err("unsupported [verify].externalCorpus value; expected advisory".to_string());
    }
    Ok(WorkspaceVerificationContextV0 {
        root,
        style_paths: files.style_paths,
        source_paths: files.source_paths,
        package_manifest_paths: files.package_manifest_paths,
        bundle_entries,
        bundle_sources,
        bundle_package_manifests,
        evidence_policy,
        translation_validation: loaded_config
            .as_ref()
            .map_or(OmenaTranslationValidationMode::Off, |loaded| {
                loaded.config.verify.translation_validation
            }),
        external_corpus_policy,
        config_content_digest: loaded_config
            .as_ref()
            .map(|loaded| loaded.config_content_digest.to_string()),
        warnings: loaded_config
            .as_ref()
            .map(|loaded| {
                loaded
                    .reports
                    .iter()
                    .map(|report| report.render_warning())
                    .collect()
            })
            .unwrap_or_default(),
    })
}

fn execute_target(
    context: &WorkspaceVerificationContextV0,
    target: VerificationTargetV0,
) -> VerificationItemReportV0 {
    let result = match target.availability {
        VerificationAvailabilityV0::NotYet => not_yet_or_skipped(context, &target),
        VerificationAvailabilityV0::Skipped => Ok(target_result(
            VerificationOutcomeV0::Skipped,
            "the target is disabled by the committed verification manifest",
        )),
        VerificationAvailabilityV0::Available => match target.executor {
            Some(VerificationExecutorV0::ParserFacts) => verify_parser_consistency(context),
            Some(VerificationExecutorV0::ModuleGraphDiagnostics) => {
                verify_module_graph_consistency(context)
            }
            Some(VerificationExecutorV0::FormatIdempotence) => verify_format_idempotence(context),
            Some(VerificationExecutorV0::BundleAdmission) => verify_bundle_admission(context),
            Some(VerificationExecutorV0::ModulesDrift) => verify_modules_drift(context),
            Some(VerificationExecutorV0::SassExternalComparison) => verify_external_sass(context),
            None => Err("available target has no executor".to_string()),
        },
    };
    let result = result.unwrap_or_else(|error| {
        target_result(
            VerificationOutcomeV0::Indeterminate,
            format!("verification could not produce a verdict: {error}"),
        )
    });
    VerificationItemReportV0 {
        id: target.id,
        scope: target.scope.as_str(),
        outcome: result.outcome,
        description: target.description,
        summary: result.summary,
        evidence: target.evidence,
        runtime_evidence: result.runtime_evidence,
        limitation: target.limitation,
        metrics: result.metrics,
    }
}

struct TargetExecutionResultV0 {
    outcome: VerificationOutcomeV0,
    summary: String,
    runtime_evidence: Vec<Value>,
    metrics: BTreeMap<String, Value>,
}

fn target_result(
    outcome: VerificationOutcomeV0,
    summary: impl Into<String>,
) -> TargetExecutionResultV0 {
    TargetExecutionResultV0 {
        outcome,
        summary: summary.into(),
        runtime_evidence: Vec::new(),
        metrics: BTreeMap::new(),
    }
}

fn not_yet_or_skipped(
    context: &WorkspaceVerificationContextV0,
    target: &VerificationTargetV0,
) -> Result<TargetExecutionResultV0, String> {
    if target.id == "translation-validation"
        && context.translation_validation == OmenaTranslationValidationMode::Off
    {
        return Ok(target_result(
            VerificationOutcomeV0::Skipped,
            "translation validation is disabled by [verify].translationValidation",
        ));
    }
    if target.id == "translation-validation" {
        let value = context.translation_validation.as_str();
        let binding = translation_validation_binding(value)?;
        return Ok(target_result(
            VerificationOutcomeV0::NotYetAvailable,
            format!(
                "{value} is bound to engine arm {} and report kind {}; the workspace observation report is not available",
                binding.engine_arm.as_deref().unwrap_or("none"),
                binding.report_kind.as_deref().unwrap_or("none")
            ),
        ));
    }
    Ok(target_result(
        VerificationOutcomeV0::NotYetAvailable,
        "the declared verification mechanism is not available on the product workspace path",
    ))
}

fn verify_parser_consistency(
    context: &WorkspaceVerificationContextV0,
) -> Result<TargetExecutionResultV0, String> {
    if context.style_paths.is_empty() {
        return Ok(target_result(
            VerificationOutcomeV0::Indeterminate,
            "no CSS-family source was discovered under the selected root",
        ));
    }
    let mut parser_error_count = 0usize;
    let mut runtime_evidence = Vec::new();
    for path in &context.style_paths {
        let source = read_source(path)?;
        let summary =
            summarize_omena_query_consumer_check_style_source(&path_string(path), &source);
        parser_error_count += summary.parser_error_count;
        runtime_evidence.push(json!({
            "stylePath": summary.style_path,
            "dialect": summary.dialect,
            "tokenCount": summary.token_count,
            "parserErrorCount": summary.parser_error_count,
        }));
    }
    let mut result = target_result(
        if parser_error_count == 0 {
            VerificationOutcomeV0::Passed
        } else {
            VerificationOutcomeV0::Failed
        },
        format!(
            "parsed {} source(s) with {parser_error_count} reported syntax error(s)",
            context.style_paths.len()
        ),
    );
    result.runtime_evidence = runtime_evidence;
    result.metrics.insert(
        "styleFileCount".to_string(),
        json!(context.style_paths.len()),
    );
    result
        .metrics
        .insert("parserErrorCount".to_string(), json!(parser_error_count));
    Ok(result)
}

fn verify_module_graph_consistency(
    context: &WorkspaceVerificationContextV0,
) -> Result<TargetExecutionResultV0, String> {
    if context.style_paths.is_empty() {
        return Ok(target_result(
            VerificationOutcomeV0::Skipped,
            "no style graph exists because no CSS-family source was discovered",
        ));
    }
    let style_sources = read_style_sources(context.style_paths.as_slice())?;
    let source_documents = read_source_documents(context.source_paths.as_slice())?;
    let package_manifests = read_package_manifests(context.package_manifest_paths.as_slice())?;
    let workspace_folder_uri = context
        .style_paths
        .first()
        .and_then(|path| style_resolution_workspace_uri_for_path(path));
    let resolution_inputs = load_omena_query_workspace_style_resolution_inputs(
        workspace_folder_uri.as_deref(),
        package_manifests.as_slice(),
    );
    let summary = summarize_omena_query_workspace_cross_file_summary_with_resolution_inputs(
        style_sources.as_slice(),
        source_documents.as_slice(),
        package_manifests.as_slice(),
        &resolution_inputs,
    );
    let unresolved_edges = summary
        .edges
        .iter()
        .filter(|edge| edge.status.starts_with("unresolved"))
        .collect::<Vec<_>>();
    let mut result = target_result(
        if unresolved_edges.is_empty() {
            VerificationOutcomeV0::Passed
        } else {
            VerificationOutcomeV0::Failed
        },
        format!(
            "resolved {} graph edge(s) with {} unresolved edge(s)",
            summary.summary_edge_count,
            unresolved_edges.len()
        ),
    );
    result.runtime_evidence = unresolved_edges
        .iter()
        .map(|edge| {
            json!({
                "edgeId": edge.edge_id,
                "edgeKind": edge.edge_kind,
                "fromPath": edge.from_path,
                "targetPath": edge.target_path,
                "status": edge.status,
            })
        })
        .collect();
    result.metrics.insert(
        "summaryEdgeCount".to_string(),
        json!(summary.summary_edge_count),
    );
    result.metrics.insert(
        "unresolvedEdgeCount".to_string(),
        json!(unresolved_edges.len()),
    );
    Ok(result)
}

fn verify_format_idempotence(
    context: &WorkspaceVerificationContextV0,
) -> Result<TargetExecutionResultV0, String> {
    let report = build_format_report(Some(context.root.clone()), None, true)?;
    let mut result = target_result(
        if report.non_idempotent_file_count == 0 {
            VerificationOutcomeV0::Passed
        } else {
            VerificationOutcomeV0::Failed
        },
        format!(
            "observed {} formatted source(s); {} were non-idempotent",
            report.file_count, report.non_idempotent_file_count
        ),
    );
    result
        .runtime_evidence
        .push(serde_json::to_value(&report).map_err(|error| error.to_string())?);
    result
        .metrics
        .insert("fileCount".to_string(), json!(report.file_count));
    result.metrics.insert(
        "nonIdempotentFileCount".to_string(),
        json!(report.non_idempotent_file_count),
    );
    Ok(result)
}

fn verify_bundle_admission(
    context: &WorkspaceVerificationContextV0,
) -> Result<TargetExecutionResultV0, String> {
    if context.bundle_entries.is_empty() {
        return Ok(target_result(
            VerificationOutcomeV0::Skipped,
            "no [build].bundleEntries are configured for closed-world admission",
        ));
    }
    let lockfile = context.root.join("omena.lock");
    let lockfile = lockfile.is_file().then_some(lockfile);
    let mut open_entry_count = 0usize;
    let mut blocker_count = 0usize;
    let mut runtime_evidence = Vec::new();
    for entry in &context.bundle_entries {
        let source_paths = context
            .bundle_sources
            .iter()
            .filter(|path| *path != entry)
            .cloned()
            .collect();
        let plan = plan_bundle(&BundleCommandOptions {
            entry: Some(entry.clone()),
            css_out: None,
            evidence_path: None,
            source_paths,
            package_manifest_paths: context.bundle_package_manifests.clone(),
            sif_paths: Vec::new(),
            lockfile: lockfile.clone(),
        })?;
        if let OmenaQueryClosedWorldOutcomeV0::Open { blockers } = &plan.result.closed_world_outcome
        {
            open_entry_count += 1;
            blocker_count += blockers.len();
        }
        runtime_evidence
            .push(serde_json::to_value(plan.evidence).map_err(|error| error.to_string())?);
    }
    let mut result = target_result(
        if open_entry_count == 0 {
            VerificationOutcomeV0::Passed
        } else {
            VerificationOutcomeV0::Failed
        },
        format!(
            "evaluated {} bundle entry or entries; {open_entry_count} remained open with {blocker_count} blocker(s)",
            context.bundle_entries.len()
        ),
    );
    result.runtime_evidence = runtime_evidence;
    result.metrics.insert(
        "bundleEntryCount".to_string(),
        json!(context.bundle_entries.len()),
    );
    result
        .metrics
        .insert("openEntryCount".to_string(), json!(open_entry_count));
    result
        .metrics
        .insert("blockerCount".to_string(), json!(blocker_count));
    Ok(result)
}

fn verify_modules_drift(
    context: &WorkspaceVerificationContextV0,
) -> Result<TargetExecutionResultV0, String> {
    if !has_css_module_sources(context.root.as_path())? {
        return Ok(target_result(
            VerificationOutcomeV0::Skipped,
            "no CSS Module source was discovered under the selected root",
        ));
    }
    let report = modules_check_report(Some(context.root.clone()))?;
    let mut result = target_result(
        if report.drift_count == 0 {
            VerificationOutcomeV0::Passed
        } else {
            VerificationOutcomeV0::Failed
        },
        format!(
            "checked {} module(s) and {} artifact(s); {} artifact(s) drifted",
            report.module_count, report.artifact_count, report.drift_count
        ),
    );
    result
        .runtime_evidence
        .push(serde_json::to_value(&report).map_err(|error| error.to_string())?);
    result
        .metrics
        .insert("moduleCount".to_string(), json!(report.module_count));
    result
        .metrics
        .insert("driftCount".to_string(), json!(report.drift_count));
    Ok(result)
}

fn verify_external_sass(
    context: &WorkspaceVerificationContextV0,
) -> Result<TargetExecutionResultV0, String> {
    if context.external_corpus_policy.as_deref() != Some("advisory") {
        return Ok(target_result(
            VerificationOutcomeV0::Skipped,
            "[verify].externalCorpus is not enabled",
        ));
    }
    let sass_paths = discover_style_paths(context.root.as_path())?
        .into_iter()
        .filter(|path| {
            matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("scss" | "sass")
            )
        })
        .collect::<Vec<_>>();
    if sass_paths.is_empty() {
        return Ok(target_result(
            VerificationOutcomeV0::Skipped,
            "external corpus verification is enabled but no Sass source was discovered",
        ));
    }
    let mut failed_count = 0usize;
    let mut runtime_evidence = Vec::new();
    for path in &sass_paths {
        let report = sass_compile_report(path.clone())?;
        if !report.compiled {
            failed_count += 1;
        }
        runtime_evidence.push(json!({
            "entry": report.entry,
            "authority": report.authority,
            "compiled": report.compiled,
            "stderr": report.stderr,
            "externalToolEvidence": report.external_tool_evidence,
        }));
    }
    let mut result = target_result(
        if failed_count == 0 {
            VerificationOutcomeV0::Passed
        } else {
            VerificationOutcomeV0::Failed
        },
        format!(
            "compiled {} Sass source(s) with the pinned external authority; {failed_count} failed",
            sass_paths.len()
        ),
    );
    result.runtime_evidence = runtime_evidence;
    result
        .metrics
        .insert("entryCount".to_string(), json!(sass_paths.len()));
    result
        .metrics
        .insert("failedEntryCount".to_string(), json!(failed_count));
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn workspace_verification_is_evidence_bearing_and_read_only() -> Result<(), String> {
        let root = fixture_root("read-only");
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        let path = root.join("app.css");
        let source = ".app {\n  color: red;\n}\n";
        fs::write(&path, source).map_err(|error| error.to_string())?;

        let execution = verify_workspace(Some(root.clone()), false)?;
        assert_eq!(execution.report.blocking_failure_count, 0);
        assert!(execution.report.passed_count > 0);
        assert!(
            execution
                .report
                .items
                .iter()
                .all(|item| item.scope == "userWorkspace")
        );
        assert!(
            execution
                .report
                .items
                .iter()
                .filter(|item| item.outcome != VerificationOutcomeV0::Skipped)
                .all(|item| !item.evidence.is_empty())
        );
        assert_eq!(
            fs::read_to_string(&path).map_err(|error| error.to_string())?,
            source
        );

        fs::remove_dir_all(root).map_err(|error| error.to_string())?;
        Ok(())
    }

    #[test]
    fn parser_errors_fail_closed() -> Result<(), String> {
        let root = fixture_root("parser-failure");
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        fs::write(root.join("broken.css"), ".app { color: red;")
            .map_err(|error| error.to_string())?;

        let report = verify_workspace(Some(root.clone()), false)?.report;
        let parser = report
            .items
            .iter()
            .find(|item| item.id == "parser-consistency")
            .ok_or_else(|| "parser verification item is missing".to_string())?;
        assert_eq!(parser.outcome, VerificationOutcomeV0::Failed);
        assert!(report.blocking_failure_count > 0);

        fs::remove_dir_all(root).map_err(|error| error.to_string())?;
        Ok(())
    }

    #[test]
    fn engine_self_roster_requires_explicit_scope() -> Result<(), String> {
        let root = fixture_root("engine-scope");
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        fs::write(root.join("app.css"), ".app {}\n").map_err(|error| error.to_string())?;

        let user_report = verify_workspace(Some(root.clone()), false)?.report;
        assert!(
            user_report
                .items
                .iter()
                .all(|item| item.scope == "userWorkspace")
        );
        let engine_report = verify_workspace(Some(root.clone()), true)?.report;
        assert_eq!(
            engine_report
                .items
                .iter()
                .filter(|item| item.scope == "engineSelf")
                .count(),
            5
        );
        assert!(
            engine_report
                .items
                .iter()
                .filter(|item| item.scope == "engineSelf")
                .all(|item| item.outcome == VerificationOutcomeV0::NotYetAvailable)
        );

        fs::remove_dir_all(root).map_err(|error| error.to_string())?;
        Ok(())
    }

    #[test]
    fn external_sass_comparison_retains_execution_witness() -> Result<(), String> {
        let root = fixture_root("external-sass");
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        fs::write(root.join("app.scss"), ".app { color: red; }\n")
            .map_err(|error| error.to_string())?;
        fs::write(
            root.join("omena.toml"),
            "[verify]\nexternalCorpus = \"advisory\"\n",
        )
        .map_err(|error| error.to_string())?;

        let report = verify_workspace(Some(root.clone()), false)?.report;
        let external = report
            .items
            .iter()
            .find(|item| item.id == "external-sass-compatibility")
            .ok_or_else(|| "external Sass verification item is missing".to_string())?;
        assert_eq!(external.outcome, VerificationOutcomeV0::Passed);
        let witness = external
            .runtime_evidence
            .first()
            .and_then(|value| value.get("externalToolEvidence"))
            .ok_or_else(|| "external execution witness is missing".to_string())?;
        assert_eq!(witness["earnedVia"], "externalTool");
        assert_eq!(witness["key"]["queryIdentity"], "omena-cli.sass.compile");

        fs::remove_dir_all(root).map_err(|error| error.to_string())?;
        Ok(())
    }

    fn fixture_root(label: &str) -> PathBuf {
        let id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "omena-verification-{label}-{}-{id}",
            std::process::id()
        ))
    }
}
