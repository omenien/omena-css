use std::{
    collections::{BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
};

use omena_checker::{
    OmenaCheckerLintTierV0, OmenaCheckerRuleDescriptorV0, OmenaCheckerRulePresetV0,
    list_omena_checker_lint_tier_mappings_v0, list_omena_checker_rule_code_names,
    list_omena_checker_rule_descriptors, summarize_omena_checker_lint_tier_coverage_v0,
};
use omena_query::{ParserRangeV0, omena_query_checker_rule_code_name_for_diagnostic_v0};
use serde::Serialize;

use crate::{
    commands::LintProfile,
    config::find_omena_config_for_path,
    diagnostics::{source_diagnostics_summary, workspace_style_diagnostics_summaries},
    output::{CliOutputMetadataV0, print_json},
    paths::path_string,
};

mod fixes;
mod stylelint_compat;
mod workspace;
use fixes::{LintWriteStatusV0, apply_lint_fix_requests, lint_fix_candidate};
use stylelint_compat::{StylelintCompatibilityReportV0, read_stylelint_compatibility_report};
use workspace::discover_workspace_files;

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
    finding_count: usize,
    lint_tier_coverage_passed: bool,
    tiers: Vec<LintTierGroupV0>,
    unmapped_diagnostic_codes: Vec<String>,
    rule_parity: LintRuleParityV0,
    stylelint_compatibility: Option<StylelintCompatibilityReportV0>,
    write: LintWriteStatusV0,
}

pub(crate) fn lint_workspace(
    root: Option<PathBuf>,
    profile: Option<LintProfile>,
    stylelint_config: Option<PathBuf>,
    write: bool,
    json: bool,
) -> Result<(), String> {
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
    if let Some(config) = loaded_config.as_ref() {
        for report in config.reports.iter() {
            eprintln!("warning: {}", report.render_warning());
        }
    }

    let report = build_lint_report(&absolute_root, profile, stylelint_compatibility, write)?;
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

    for source_path in &files.source_paths {
        let summary = source_diagnostics_summary(
            path_string(source_path),
            None,
            Some(source_path.clone()),
            files.style_paths.clone(),
            files.package_manifest_paths.clone(),
        )?;
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

#[cfg(test)]
mod tests {
    use super::*;

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
            checker_rule_id_for_diagnostic("missingModule").as_deref(),
            Some("missing-module")
        );
        assert_eq!(
            checker_rule_id_for_diagnostic("missingSelector").as_deref(),
            Some("missing-static-class")
        );
        assert_eq!(checker_rule_id_for_diagnostic("notCheckerOwned"), None);
    }
}
