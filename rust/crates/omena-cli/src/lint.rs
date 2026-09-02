use std::{
    collections::{BTreeSet, HashMap},
    env, fs,
    path::{Path, PathBuf},
};

use omena_checker::{
    OmenaCheckerLintTierV0, OmenaCheckerRuleDescriptorV0, OmenaCheckerRulePresetV0,
    list_omena_checker_lint_tier_mappings_v0, list_omena_checker_rule_code_names,
    list_omena_checker_rule_descriptors, summarize_omena_checker_lint_tier_coverage_v0,
};
use omena_query::{
    OmenaQueryCascadeRankedSetLossCaptureV0, capture_omena_query_cascade_ranked_set_losses,
};
use omena_query::{ParserRangeV0, omena_query_checker_rule_code_name_for_diagnostic_v0};
use serde::Serialize;

use crate::{
    commands::LintProfile,
    config::find_omena_config_for_path,
    diagnostics::{workspace_source_diagnostics_summaries, workspace_style_diagnostics_summaries},
    output::{CliOutputMetadataV0, commit_json_artifact, print_json},
    paths::path_string,
    workspace_edit_transaction::{ExpectedContentDigestV0, WorkspaceEditSafetyClassV0},
};

mod fixes;
mod stylelint_compat;
mod workspace;
use fixes::{LintWriteStatusV0, apply_lint_fix_requests, lint_fix_candidate};
use stylelint_compat::{StylelintCompatibilityReportV0, read_stylelint_compatibility_report};
pub(crate) use workspace::discover_workspace_files;

pub(crate) fn discover_style_paths(root: &Path) -> Result<Vec<PathBuf>, String> {
    Ok(discover_workspace_files(root)?.style_paths)
}

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
struct LintTierGroupV0 {
    tier: OmenaCheckerLintTierV0,
    tier_name: &'static str,
    active_rule_count: usize,
    finding_count: usize,
    findings: Vec<LintFindingV0>,
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
    pub(crate) finding_count: usize,
    lint_tier_coverage_passed: bool,
    tiers: Vec<LintTierGroupV0>,
    unmapped_diagnostic_codes: Vec<String>,
    rule_parity: LintRuleParityV0,
    stylelint_compatibility: Option<StylelintCompatibilityReportV0>,
    write: LintWriteStatusV0,
}

struct LintExecutionV0 {
    report: LintReportV0,
    config_content_digest: Option<String>,
    warnings: Vec<String>,
    ranked_set_loss_capture: Option<OmenaQueryCascadeRankedSetLossCaptureV0>,
}

pub(crate) fn lint_workspace(
    root: Option<PathBuf>,
    profile: Option<LintProfile>,
    stylelint_config: Option<PathBuf>,
    write: bool,
    json: bool,
) -> Result<(), String> {
    let execution = build_lint_execution(root, profile, stylelint_config, write)?;
    for warning in &execution.warnings {
        eprintln!("warning: {warning}");
    }
    if let Some(capture) = &execution.ranked_set_loss_capture {
        write_ranked_set_loss_capture_if_complete(capture, &execution.report.write)?;
    }
    let report = execution.report;
    if json {
        print_json(
            CliOutputMetadataV0::new("omena-cli.lint")
                .with_config_content_digest(execution.config_content_digest.as_deref()),
            &report,
        )?;
    } else {
        print_text_report(&report);
    }
    Ok(())
}

pub(crate) fn lint_check_report(root: Option<PathBuf>) -> Result<LintReportV0, String> {
    Ok(build_lint_execution(root, None, None, false)?.report)
}

pub(crate) fn lint_report(
    root: Option<PathBuf>,
    profile: Option<LintProfile>,
    stylelint_config: Option<PathBuf>,
) -> Result<LintReportV0, String> {
    Ok(build_lint_execution(root, profile, stylelint_config, false)?.report)
}

fn build_lint_execution(
    root: Option<PathBuf>,
    profile: Option<LintProfile>,
    stylelint_config: Option<PathBuf>,
    write: bool,
) -> Result<LintExecutionV0, String> {
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
    let configured_stylelint_compatibility = loaded_config
        .as_ref()
        .and_then(|loaded| loaded.config.lint.stylelint_compat)
        .unwrap_or(false);
    let stylelint_config = match stylelint_config {
        Some(path) => Some(path),
        None if configured_stylelint_compatibility => {
            Some(discover_stylelint_config(&absolute_root).ok_or_else(|| {
                format!(
                    "[lint].stylelintCompat is enabled but no .stylelintrc JSON/YAML file was found under {}",
                    path_string(&absolute_root)
                )
            })?)
        }
        None => None,
    };
    let stylelint_compatibility = stylelint_config
        .as_deref()
        .map(read_stylelint_compatibility_report)
        .transpose()?;
    let warnings = loaded_config
        .as_ref()
        .map(|config| {
            config
                .reports
                .iter()
                .map(|report| report.render_warning())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let capture_ranked_set_losses = env::var_os("OMENA_RANKED_SET_LOSS_CENSUS_PATH").is_some();
    let (report, ranked_set_loss_capture) = if capture_ranked_set_losses {
        let (report, capture) = capture_omena_query_cascade_ranked_set_losses(|| {
            build_lint_report(&absolute_root, profile, stylelint_compatibility, write)
        })
        .map_err(str::to_string)?;
        (report?, Some(capture))
    } else {
        (
            build_lint_report(&absolute_root, profile, stylelint_compatibility, write)?,
            None,
        )
    };
    Ok(LintExecutionV0 {
        report,
        config_content_digest: loaded_config
            .as_ref()
            .map(|config| config.config_content_digest.to_string()),
        warnings,
        ranked_set_loss_capture,
    })
}

fn write_ranked_set_loss_capture(
    capture: &OmenaQueryCascadeRankedSetLossCaptureV0,
) -> Result<(), String> {
    let path = env::var_os("OMENA_RANKED_SET_LOSS_CENSUS_PATH")
        .map(PathBuf::from)
        .ok_or_else(|| "ranked-set loss census path is missing".to_string())?;
    let expected = ExpectedContentDigestV0::observe(path.as_path())
        .map_err(|error| format!("failed to inspect ranked-set loss census: {error}"))?;
    commit_json_artifact(
        path.as_path(),
        expected,
        WorkspaceEditSafetyClassV0::EvidenceRequired,
        capture,
        None,
    )
    .map(|_| ())
}

fn build_lint_report(
    workspace_root: &Path,
    profile: LintProfile,
    stylelint_compatibility: Option<StylelintCompatibilityReportV0>,
    write: bool,
) -> Result<LintReportV0, String> {
    let files = discover_workspace_files(workspace_root)?;
    let stylelint_rule_ids = stylelint_compatibility
        .as_ref()
        .map(StylelintCompatibilityReportV0::enabled_omena_rule_ids)
        .unwrap_or_default();
    let descriptors = active_rule_descriptors(profile, &stylelint_rule_ids);
    let tier_coverage = summarize_omena_checker_lint_tier_coverage_v0();
    if !tier_coverage.coverage_passed {
        return Err(format!(
            "lint tier mapping is incomplete: missing={:?}, extra={:?}, duplicate={:?}",
            tier_coverage.missing_rule_names,
            tier_coverage.extra_rule_names,
            tier_coverage.duplicate_rule_names
        ));
    }
    let tier_by_rule = list_omena_checker_lint_tier_mappings_v0()
        .into_iter()
        .map(|mapping| (mapping.rule_code_name, mapping.lint_tier))
        .collect::<HashMap<_, _>>();
    let active_rule_ids = descriptors
        .iter()
        .map(|descriptor| descriptor.code_name)
        .collect::<Vec<_>>();
    let active_rule_set = active_rule_ids.iter().copied().collect::<BTreeSet<_>>();
    let mut findings = Vec::new();
    let mut fix_candidates = Vec::new();
    let mut unmapped_diagnostic_codes = BTreeSet::new();

    let style_summaries = workspace_style_diagnostics_summaries(
        files.style_paths.as_slice(),
        files.source_paths.as_slice(),
        files.package_manifest_paths.as_slice(),
    )?;
    for summary in style_summaries {
        for diagnostic in summary.diagnostics {
            let Some(rule_id) = checker_rule_id_for_diagnostic(diagnostic.code) else {
                unmapped_diagnostic_codes.insert(diagnostic.code.to_string());
                continue;
            };
            if !active_rule_set.contains(rule_id.as_str()) {
                continue;
            }
            if let Some(action) = diagnostic.create_custom_property.as_ref() {
                fix_candidates.push(lint_fix_candidate(
                    rule_id.as_str(),
                    action.uri.as_str(),
                    action.range,
                    action.new_text.as_str(),
                ));
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

    let source_summaries = workspace_source_diagnostics_summaries(
        files.source_paths.as_slice(),
        files.style_paths.as_slice(),
        files.package_manifest_paths.as_slice(),
    )?;
    for summary in source_summaries {
        for diagnostic in summary.diagnostics {
            let Some(rule_id) = checker_rule_id_for_diagnostic(diagnostic.code) else {
                unmapped_diagnostic_codes.insert(diagnostic.code.to_string());
                continue;
            };
            if !active_rule_set.contains(rule_id.as_str()) {
                continue;
            }
            if let Some(action) = diagnostic.create_selector.as_ref() {
                fix_candidates.push(lint_fix_candidate(
                    rule_id.as_str(),
                    action.uri.as_str(),
                    action.range,
                    action.new_text.as_str(),
                ));
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
    let write_report = apply_lint_fix_requests(fix_candidates.as_slice(), write)?;
    let finding_count = findings.len();
    let tiers = [
        OmenaCheckerLintTierV0::Syntax,
        OmenaCheckerLintTierV0::Semantic,
        OmenaCheckerLintTierV0::SourceAware,
    ]
    .into_iter()
    .map(|tier| {
        let active_rule_count = active_rule_ids
            .iter()
            .filter(|rule_id| tier_by_rule.get(**rule_id) == Some(&tier))
            .count();
        let tier_findings = findings
            .iter()
            .filter(|finding| tier_by_rule.get(finding.rule_id.as_str()) == Some(&tier))
            .cloned()
            .collect::<Vec<_>>();
        LintTierGroupV0 {
            tier,
            tier_name: tier.as_str(),
            active_rule_count,
            finding_count: tier_findings.len(),
            findings: tier_findings,
        }
    })
    .collect();

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
        finding_count,
        lint_tier_coverage_passed: tier_coverage.coverage_passed,
        tiers,
        unmapped_diagnostic_codes: unmapped_diagnostic_codes.into_iter().collect(),
        rule_parity,
        stylelint_compatibility,
        write: write_report,
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

fn active_rule_descriptors(
    profile: LintProfile,
    additional_rule_ids: &BTreeSet<&str>,
) -> Vec<OmenaCheckerRuleDescriptorV0> {
    list_omena_checker_rule_descriptors()
        .into_iter()
        .filter(|descriptor| {
            profile == LintProfile::Strict
                || descriptor
                    .presets
                    .contains(&OmenaCheckerRulePresetV0::Recommended)
                || additional_rule_ids.contains(descriptor.code_name)
        })
        .collect()
}

fn discover_stylelint_config(root: &Path) -> Option<PathBuf> {
    let directory = if root.is_dir() { root } else { root.parent()? };
    [
        ".stylelintrc",
        ".stylelintrc.json",
        ".stylelintrc.yaml",
        ".stylelintrc.yml",
    ]
    .into_iter()
    .map(|file_name| directory.join(file_name))
    .find(|path| path.is_file())
}

fn checker_rule_id_for_diagnostic(code: &str) -> Option<String> {
    omena_query_checker_rule_code_name_for_diagnostic_v0(code).map(str::to_string)
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

fn print_text_report(report: &LintReportV0) {
    println!("profile: {}", report.profile);
    println!("workspace: {}", report.workspace_root);
    println!("rules: {}", report.active_rule_count);
    println!("findings: {}", report.finding_count);
    for tier in &report.tiers {
        println!("{}: {}", tier.tier_name, tier.finding_count);
        let mut by_file = HashMap::<&str, Vec<&LintFindingV0>>::new();
        for finding in &tier.findings {
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
    }
    if report.write.requested && report.write.applied_edit_count == 0 {
        println!("write: {}", report.write.status);
    }
    if let Some(stylelint) = report.stylelint_compatibility.as_ref() {
        println!(
            "stylelint compatibility: mapped={} unsupported={}",
            stylelint.mapped_rule_count, stylelint.unsupported_rule_count
        );
        for unsupported in &stylelint.unsupported_rules {
            println!("  unsupported: {}", unsupported.stylelint_rule);
        }
    }
}

fn write_ranked_set_loss_capture_if_complete(
    capture: &OmenaQueryCascadeRankedSetLossCaptureV0,
    status: &LintWriteStatusV0,
) -> Result<(), String> {
    if allows_ranked_set_loss_capture_publish(status) {
        write_ranked_set_loss_capture(capture)
    } else {
        eprintln!(
            "warning: ranked-set loss census was not written because lint --write did not land every candidate fix"
        );
        Ok(())
    }
}

fn allows_ranked_set_loss_capture_publish(status: &LintWriteStatusV0) -> bool {
    !status.requested
        || (status.candidate_edit_count > 0
            && status.applied_edit_count == status.candidate_edit_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::BTreeMap, process::Command, time::SystemTime};

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
        let recommended = active_rule_descriptors(LintProfile::Recommended, &BTreeSet::new());
        let strict = active_rule_descriptors(LintProfile::Strict, &BTreeSet::new());
        assert!(recommended.len() < strict.len());
        assert_eq!(strict.len(), list_omena_checker_rule_descriptors().len());
    }

    #[test]
    fn stylelint_compatibility_can_enable_a_rule_outside_the_recommended_profile() {
        let additional = BTreeSet::from(["unused-selector"]);
        let active = active_rule_descriptors(LintProfile::Recommended, &additional);
        assert!(
            active
                .iter()
                .any(|descriptor| descriptor.code_name == "unused-selector")
        );
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
    fn diagnostic_codes_use_checker_rule_spelling() {
        assert_eq!(
            checker_rule_id_for_diagnostic("missing-module").as_deref(),
            Some("missing-module")
        );
        assert_eq!(
            checker_rule_id_for_diagnostic("missingModule").as_deref(),
            Some("missing-module")
        );
        assert_eq!(
            checker_rule_id_for_diagnostic("missingSelector").as_deref(),
            Some("missing-static-class")
        );
        assert_eq!(checker_rule_id_for_diagnostic("notCheckerOwned"), None);
    }

    #[test]
    #[ignore = "explicit full tracked-workspace lint census receipt"]
    fn tracked_workspace_recommended_lint_preserves_the_pinned_rule_census() -> Result<(), String> {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .map_err(|error| error.to_string())?;
        let nonce = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos();
        let scratch = std::env::temp_dir().join(format!(
            "omena-tracked-lint-census-{}-{nonce}",
            std::process::id()
        ));
        let worktree = scratch.join("worktree");
        fs::create_dir_all(&scratch).map_err(|error| error.to_string())?;
        let added = Command::new("git")
            .args(["worktree", "add", "--detach"])
            .arg(&worktree)
            .arg("HEAD")
            .current_dir(&repo_root)
            .output()
            .map_err(|error| error.to_string())?;
        if !added.status.success() {
            return Err(format!(
                "cannot create tracked lint worktree: {}",
                String::from_utf8_lossy(&added.stderr)
            ));
        }

        let measurement = lint_report(Some(worktree.clone()), Some(LintProfile::Recommended), None);
        let removed = Command::new("git")
            .args(["worktree", "remove", "--force"])
            .arg(&worktree)
            .current_dir(&repo_root)
            .output()
            .map_err(|error| error.to_string())?;
        let _ = fs::remove_dir_all(&scratch);
        if !removed.status.success() {
            return Err(format!(
                "cannot remove tracked lint worktree: {}",
                String::from_utf8_lossy(&removed.stderr)
            ));
        }

        let report = measurement?;
        let mut counts = BTreeMap::<String, usize>::new();
        for finding in report.tiers.iter().flat_map(|tier| &tier.findings) {
            *counts.entry(finding.rule_id.clone()).or_default() += 1;
        }
        println!(
            "trackedLintStyleFileCount={} trackedLintSourceFileCount={} trackedLintFindingCount={} ruleCounts={counts:?}",
            report.style_file_count, report.source_file_count, report.finding_count
        );
        assert_eq!(report.style_file_count, 200);
        assert_eq!(report.source_file_count, 1_021);
        assert_eq!(report.finding_count, 37);
        assert_eq!(
            counts,
            BTreeMap::from([
                ("invalid-property-value".to_string(), 13),
                ("missing-composed-module".to_string(), 2),
                ("missing-composed-selector".to_string(), 3),
                ("missing-imported-value".to_string(), 1),
                ("missing-keyframes".to_string(), 1),
                ("missing-module".to_string(), 5),
                ("missing-resolved-class-domain".to_string(), 1),
                ("missing-resolved-class-values".to_string(), 2),
                ("missing-sass-symbol".to_string(), 3),
                ("missing-static-class".to_string(), 4),
                ("missing-template-prefix".to_string(), 1),
                ("missing-value-module".to_string(), 1),
            ])
        );
        Ok(())
    }
}
