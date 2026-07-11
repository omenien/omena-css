use std::{
    collections::{BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
};

use omena_checker::{
    OmenaCheckerRuleDescriptorV0, OmenaCheckerRulePresetV0, is_omena_checker_rule_code,
    list_omena_checker_rule_code_names, list_omena_checker_rule_descriptors,
};
use omena_query::ParserRangeV0;
use serde::Serialize;

use crate::{
    commands::LintProfile,
    config::find_omena_config_for_path,
    diagnostics::{source_diagnostics_summary, workspace_style_diagnostics_summaries},
    output::{CliOutputMetadataV0, print_json},
    paths::path_string,
};

const SHARED_CHECKER_RULES: &[&str] = &[
    "missing-module",
    "missing-static-class",
    "missing-template-prefix",
    "missing-resolved-class-values",
    "missing-resolved-class-domain",
    "unused-selector",
    "missing-composed-module",
    "missing-composed-selector",
    "missing-value-module",
    "missing-imported-value",
    "missing-keyframes",
    "missing-custom-property",
    "missing-sass-symbol",
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LintFindingV0 {
    file_path: String,
    category: &'static str,
    rule_id: String,
    severity: &'static str,
    range: ParserRangeV0,
    message: String,
    provenance: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LintRuleParityV0 {
    shared_rule_count: usize,
    shared_rule_ids: Vec<&'static str>,
    rust_only_rule_ids: Vec<&'static str>,
    typescript_only_rule_ids: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LintWriteStatusV0 {
    requested: bool,
    applied_edit_count: usize,
    status: &'static str,
    owner: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LintReportV0 {
    schema_version: &'static str,
    product: &'static str,
    workspace_root: String,
    profile: &'static str,
    style_file_count: usize,
    source_file_count: usize,
    package_manifest_count: usize,
    active_rule_count: usize,
    active_rule_ids: Vec<&'static str>,
    finding_count: usize,
    findings: Vec<LintFindingV0>,
    unmapped_diagnostic_codes: Vec<String>,
    rule_parity: LintRuleParityV0,
    write: LintWriteStatusV0,
}

struct WorkspaceFiles {
    style_paths: Vec<PathBuf>,
    source_paths: Vec<PathBuf>,
    package_manifest_paths: Vec<PathBuf>,
}

pub(crate) fn lint_workspace(
    root: Option<PathBuf>,
    profile: Option<LintProfile>,
    stylelint_config: Option<PathBuf>,
    write: bool,
    json: bool,
) -> Result<(), String> {
    if let Some(path) = stylelint_config {
        return Err(format!(
            "Stylelint compatibility config ingestion is not available yet: {}",
            path_string(&path)
        ));
    }

    let root = root.unwrap_or_else(|| PathBuf::from("."));
    let absolute_root = fs::canonicalize(&root).map_err(|error| {
        format!(
            "failed to resolve lint root {}: {error}",
            path_string(&root)
        )
    })?;
    let loaded_config = find_omena_config_for_path(&absolute_root)?;
    let configured_profile = loaded_config
        .as_ref()
        .and_then(|loaded| loaded.config.lint.profile.as_deref());
    let profile = resolve_lint_profile(profile, configured_profile)?;
    if let Some(config) = loaded_config.as_ref() {
        for report in config.reports.iter() {
            eprintln!("warning: {}", report.render_warning());
        }
    }

    let report = build_lint_report(&absolute_root, profile, write)?;
    if json {
        print_json(
            CliOutputMetadataV0::new("omena-cli.lint").with_config_content_digest(
                loaded_config
                    .as_ref()
                    .map(|config| config.config_content_digest.as_ref()),
            ),
            &report,
        )?;
    } else {
        print_text_report(&report);
    }
    Ok(())
}

fn build_lint_report(
    workspace_root: &Path,
    profile: LintProfile,
    write: bool,
) -> Result<LintReportV0, String> {
    let files = discover_workspace_files(workspace_root)?;
    let descriptors = active_rule_descriptors(profile);
    let active_rule_ids = descriptors
        .iter()
        .map(|descriptor| descriptor.code_name)
        .collect::<Vec<_>>();
    let active_rule_set = active_rule_ids.iter().copied().collect::<BTreeSet<_>>();
    let mut findings = Vec::new();
    let mut unmapped_diagnostic_codes = BTreeSet::new();

    let style_summaries = workspace_style_diagnostics_summaries(
        files.style_paths.as_slice(),
        files.source_paths.as_slice(),
        files.package_manifest_paths.as_slice(),
    )?;
    for summary in style_summaries {
        for diagnostic in summary.diagnostics {
            let rule_id = checker_rule_id_for_diagnostic(diagnostic.code);
            if !is_omena_checker_rule_code(rule_id.as_str()) {
                unmapped_diagnostic_codes.insert(diagnostic.code.to_string());
                continue;
            }
            if !active_rule_set.contains(rule_id.as_str()) {
                continue;
            }
            findings.push(LintFindingV0 {
                file_path: summary.file_uri.clone(),
                category: "style",
                rule_id,
                severity: diagnostic.severity,
                range: diagnostic.range,
                message: diagnostic.message,
                provenance: diagnostic.provenance,
            });
        }
    }

    for source_path in &files.source_paths {
        let summary = source_diagnostics_summary(
            path_string(source_path),
            None,
            Some(source_path.clone()),
            files.style_paths.clone(),
            files.package_manifest_paths.clone(),
        )?;
        for diagnostic in summary.diagnostics {
            let rule_id = checker_rule_id_for_diagnostic(diagnostic.code);
            if !is_omena_checker_rule_code(rule_id.as_str()) {
                unmapped_diagnostic_codes.insert(diagnostic.code.to_string());
                continue;
            }
            if !active_rule_set.contains(rule_id.as_str()) {
                continue;
            }
            findings.push(LintFindingV0 {
                file_path: summary.file_uri.clone(),
                category: "source",
                rule_id,
                severity: diagnostic.severity,
                range: diagnostic.range,
                message: diagnostic.message,
                provenance: diagnostic.provenance,
            });
        }
    }

    findings.sort_by(|left, right| {
        (
            left.file_path.as_str(),
            left.range.start.line,
            left.range.start.character,
            left.rule_id.as_str(),
        )
            .cmp(&(
                right.file_path.as_str(),
                right.range.start.line,
                right.range.start.character,
                right.rule_id.as_str(),
            ))
    });
    let rule_parity = rule_parity();

    Ok(LintReportV0 {
        schema_version: "0",
        product: "omena-cli.lint-report",
        workspace_root: path_string(workspace_root),
        profile: profile.as_str(),
        style_file_count: files.style_paths.len(),
        source_file_count: files.source_paths.len(),
        package_manifest_count: files.package_manifest_paths.len(),
        active_rule_count: active_rule_ids.len(),
        active_rule_ids,
        finding_count: findings.len(),
        findings,
        unmapped_diagnostic_codes: unmapped_diagnostic_codes.into_iter().collect(),
        rule_parity,
        write: LintWriteStatusV0 {
            requested: write,
            applied_edit_count: 0,
            status: "waitingForRuleLinkedSourceEdit",
            owner: "omena lint",
        },
    })
}

fn resolve_lint_profile(
    cli_profile: Option<LintProfile>,
    configured_profile: Option<&str>,
) -> Result<LintProfile, String> {
    if let Some(profile) = cli_profile {
        return Ok(profile);
    }
    match configured_profile {
        None | Some("recommended") => Ok(LintProfile::Recommended),
        Some("strict") => Ok(LintProfile::Strict),
        Some(value) => Err(format!(
            "unsupported lint profile '{value}'; expected recommended or strict"
        )),
    }
}

fn active_rule_descriptors(profile: LintProfile) -> Vec<OmenaCheckerRuleDescriptorV0> {
    list_omena_checker_rule_descriptors()
        .into_iter()
        .filter(|descriptor| {
            profile == LintProfile::Strict
                || descriptor
                    .presets
                    .contains(&OmenaCheckerRulePresetV0::Recommended)
        })
        .collect()
}

fn checker_rule_id_for_diagnostic(code: &str) -> String {
    let mut output = String::with_capacity(code.len() + 4);
    for (index, character) in code.char_indices() {
        if character.is_ascii_uppercase() {
            if index > 0 {
                output.push('-');
            }
            output.push(character.to_ascii_lowercase());
        } else {
            output.push(character);
        }
    }
    output
}

fn rule_parity() -> LintRuleParityV0 {
    let rust_rules = list_omena_checker_rule_code_names()
        .into_iter()
        .collect::<BTreeSet<_>>();
    let shared_rules = SHARED_CHECKER_RULES
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    LintRuleParityV0 {
        shared_rule_count: shared_rules.len(),
        shared_rule_ids: shared_rules.iter().copied().collect(),
        rust_only_rule_ids: rust_rules.difference(&shared_rules).copied().collect(),
        typescript_only_rule_ids: shared_rules.difference(&rust_rules).copied().collect(),
    }
}

fn discover_workspace_files(root: &Path) -> Result<WorkspaceFiles, String> {
    let mut files = WorkspaceFiles {
        style_paths: Vec::new(),
        source_paths: Vec::new(),
        package_manifest_paths: Vec::new(),
    };
    if root.is_file() {
        classify_file(root.to_path_buf(), &mut files);
    } else {
        visit_directory(root, &mut files)?;
    }
    files.style_paths.sort();
    files.source_paths.sort();
    files.package_manifest_paths.sort();
    Ok(files)
}

fn visit_directory(directory: &Path, files: &mut WorkspaceFiles) -> Result<(), String> {
    let entries = fs::read_dir(directory)
        .map_err(|error| format!("failed to read {}: {error}", path_string(directory)))?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            format!(
                "failed to read an entry under {}: {error}",
                path_string(directory)
            )
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| {
            format!("failed to inspect {}: {error}", path_string(path.as_path()))
        })?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            if !ignored_directory(entry.file_name().to_string_lossy().as_ref()) {
                visit_directory(path.as_path(), files)?;
            }
        } else if file_type.is_file() {
            classify_file(path, files);
        }
    }
    Ok(())
}

fn ignored_directory(name: &str) -> bool {
    matches!(
        name,
        ".git"
            | ".hg"
            | ".svn"
            | "node_modules"
            | "target"
            | "dist"
            | "build"
            | "coverage"
            | ".next"
            | ".turbo"
    )
}

fn classify_file(path: PathBuf, files: &mut WorkspaceFiles) {
    let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
        return;
    };
    if file_name == "package.json" {
        files.package_manifest_paths.push(path);
        return;
    }
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if matches!(extension, "css" | "scss" | "sass" | "less") {
        files.style_paths.push(path);
    } else if matches!(
        extension,
        "ts" | "tsx"
            | "mts"
            | "cts"
            | "js"
            | "jsx"
            | "mjs"
            | "cjs"
            | "vue"
            | "svelte"
            | "astro"
            | "html"
    ) {
        files.source_paths.push(path);
    }
}

fn print_text_report(report: &LintReportV0) {
    println!("profile: {}", report.profile);
    println!("workspace: {}", report.workspace_root);
    println!("rules: {}", report.active_rule_count);
    println!("findings: {}", report.finding_count);
    let mut by_file = HashMap::<&str, Vec<&LintFindingV0>>::new();
    for finding in &report.findings {
        by_file
            .entry(finding.file_path.as_str())
            .or_default()
            .push(finding);
    }
    let mut paths = by_file.keys().copied().collect::<Vec<_>>();
    paths.sort_unstable();
    for path in paths {
        println!("{path}");
        for finding in &by_file[path] {
            println!(
                "  {}:{} {} {}",
                finding.range.start.line + 1,
                finding.range.start.character + 1,
                finding.rule_id,
                finding.message
            );
        }
    }
    if report.write.requested && report.write.applied_edit_count == 0 {
        println!("write: {}", report.write.status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn config_profile_is_used_and_cli_profile_wins() -> Result<(), String> {
        assert_eq!(
            resolve_lint_profile(None, Some("strict"))?,
            LintProfile::Strict
        );
        assert_eq!(
            resolve_lint_profile(Some(LintProfile::Recommended), Some("strict"))?,
            LintProfile::Recommended
        );
        assert!(resolve_lint_profile(None, Some("unknown")).is_err());
        Ok(())
    }

    #[test]
    fn strict_profile_contains_the_complete_registered_rule_set() {
        let recommended = active_rule_descriptors(LintProfile::Recommended);
        let strict = active_rule_descriptors(LintProfile::Strict);
        assert!(recommended.len() < strict.len());
        assert_eq!(strict.len(), list_omena_checker_rule_descriptors().len());
    }

    #[test]
    fn shared_checker_contract_is_a_real_subset() {
        let parity = rule_parity();
        assert_eq!(parity.shared_rule_count, 13);
        assert!(parity.typescript_only_rule_ids.is_empty());
        assert_eq!(
            parity.shared_rule_count + parity.rust_only_rule_ids.len(),
            list_omena_checker_rule_code_names().len()
        );
    }

    #[test]
    fn workspace_discovery_is_sorted_and_skips_generated_directories() -> Result<(), String> {
        let root = fixture_root("discovery");
        fs::create_dir_all(root.join("src")).map_err(|error| error.to_string())?;
        fs::create_dir_all(root.join("node_modules/pkg")).map_err(|error| error.to_string())?;
        fs::write(root.join("src/a.module.scss"), ".a {}\n").map_err(|error| error.to_string())?;
        fs::write(root.join("src/a.tsx"), "export {};\n").map_err(|error| error.to_string())?;
        fs::write(root.join("package.json"), "{}\n").map_err(|error| error.to_string())?;
        fs::write(root.join("node_modules/pkg/ignored.css"), ".ignored {}\n")
            .map_err(|error| error.to_string())?;

        let files = discover_workspace_files(root.as_path())?;
        assert_eq!(files.style_paths.len(), 1);
        assert_eq!(files.source_paths.len(), 1);
        assert_eq!(files.package_manifest_paths.len(), 1);
        fs::remove_dir_all(root).map_err(|error| error.to_string())?;
        Ok(())
    }

    #[test]
    fn diagnostic_codes_use_checker_rule_spelling() {
        assert_eq!(
            checker_rule_id_for_diagnostic("missingModule"),
            "missing-module"
        );
        assert_eq!(
            checker_rule_id_for_diagnostic("cascade.deepConflict"),
            "cascade.deep-conflict"
        );
    }

    fn fixture_root(label: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        let sequence = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "omena-lint-{label}-{}-{timestamp}-{sequence}",
            std::process::id()
        ))
    }
}
