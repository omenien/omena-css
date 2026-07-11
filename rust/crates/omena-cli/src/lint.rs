use std::{
    collections::{BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
};

use omena_checker::{
    FixSafetyAssessmentV0, FixSafetyEvidenceInputV0, FixSafetyV0, OmenaCheckerLintTierV0,
    OmenaCheckerRuleDescriptorV0, OmenaCheckerRulePresetV0, compute_fix_safety,
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
    paths::{cli_file_uri_to_path, path_string},
    write_safety::{
        SourceWriteErrorV0, SourceWriteEvidenceV0, SourceWriteModeV0, SourceWriteRejectionV0,
        apply_write_with_safety,
    },
};

mod stylelint_compat;
use stylelint_compat::{StylelintCompatibilityReportV0, read_stylelint_compatibility_report};

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
    candidate_edit_count: usize,
    safe_edit_count: usize,
    conservative_edit_count: usize,
    manual_review_edit_count: usize,
    applied_edit_count: usize,
    rejection_count: usize,
    status: &'static str,
    owner: &'static str,
    suggestions: Vec<LintFixSuggestionV0>,
    rejections: Vec<SourceWriteRejectionV0>,
}

#[derive(Debug, Clone)]
struct LintFixCandidateV0 {
    rule_id: String,
    output_path: PathBuf,
    range: ParserRangeV0,
    new_text: String,
    assessment: FixSafetyAssessmentV0,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LintFixSuggestionV0 {
    rule_id: String,
    output_path: String,
    range: ParserRangeV0,
    new_text: String,
    safety: FixSafetyV0,
    precision_backed: bool,
    rationale: Vec<&'static str>,
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

fn lint_fix_candidate(
    rule_id: &str,
    output_uri: &str,
    range: ParserRangeV0,
    new_text: &str,
) -> LintFixCandidateV0 {
    let output_path = cli_file_uri_to_path(output_uri).unwrap_or_else(|| PathBuf::from(output_uri));
    let assessment = compute_fix_safety(FixSafetyEvidenceInputV0 {
        syntax_preserving: true,
        local_semantics_required: true,
        local_semantics_ready: false,
        closed_world_required: true,
        closed_world_ready: false,
        reference_precision_required: true,
        reference_precision: None,
    });
    LintFixCandidateV0 {
        rule_id: rule_id.to_string(),
        output_path,
        range,
        new_text: new_text.to_string(),
        assessment,
    }
}

fn apply_lint_fix_requests(
    candidates: &[LintFixCandidateV0],
    write: bool,
) -> Result<LintWriteStatusV0, String> {
    let suggestions = candidates
        .iter()
        .map(|candidate| LintFixSuggestionV0 {
            rule_id: candidate.rule_id.clone(),
            output_path: path_string(candidate.output_path.as_path()),
            range: candidate.range,
            new_text: candidate.new_text.clone(),
            safety: candidate.assessment.safety,
            precision_backed: candidate.assessment.precision_backed,
            rationale: candidate.assessment.rationale.clone(),
        })
        .collect::<Vec<_>>();
    let mut applied_edit_count = 0;
    let mut rejections = Vec::new();
    if write {
        for candidate in candidates {
            let source = fs::read_to_string(candidate.output_path.as_path()).map_err(|error| {
                format!(
                    "failed to read lint fix target {}: {error}",
                    path_string(candidate.output_path.as_path())
                )
            })?;
            let edited = apply_text_edit(
                source.as_str(),
                candidate.range,
                candidate.new_text.as_str(),
            )?;
            match apply_write_with_safety(
                candidate.output_path.as_path(),
                edited.as_bytes(),
                &candidate.assessment,
                SourceWriteModeV0::SafeOnly,
                SourceWriteEvidenceV0::LintFix,
            ) {
                Ok(_) => applied_edit_count += 1,
                Err(SourceWriteErrorV0::Rejected(rejection)) => rejections.push(rejection),
                Err(error) => return Err(error.to_string()),
            }
        }
    }
    let safe_edit_count = count_fix_safety(candidates, FixSafetyV0::Safe);
    let conservative_edit_count = count_fix_safety(candidates, FixSafetyV0::Conservative);
    let manual_review_edit_count = count_fix_safety(candidates, FixSafetyV0::ManualReview);
    let status = if applied_edit_count > 0 {
        "appliedSafeEdits"
    } else if write && !rejections.is_empty() {
        "rejectedByFixSafety"
    } else if candidates.is_empty() {
        "waitingForRuleLinkedSourceEdit"
    } else {
        "manualReviewOnly"
    };
    Ok(LintWriteStatusV0 {
        requested: write,
        candidate_edit_count: candidates.len(),
        safe_edit_count,
        conservative_edit_count,
        manual_review_edit_count,
        applied_edit_count,
        rejection_count: rejections.len(),
        status,
        owner: "omena lint",
        suggestions,
        rejections,
    })
}

fn count_fix_safety(candidates: &[LintFixCandidateV0], safety: FixSafetyV0) -> usize {
    candidates
        .iter()
        .filter(|candidate| candidate.assessment.safety == safety)
        .count()
}

fn apply_text_edit(source: &str, range: ParserRangeV0, new_text: &str) -> Result<String, String> {
    let start = byte_offset_for_position(source, range.start.line, range.start.character)
        .ok_or_else(|| "lint fix start position is outside the target source".to_string())?;
    let end = byte_offset_for_position(source, range.end.line, range.end.character)
        .ok_or_else(|| "lint fix end position is outside the target source".to_string())?;
    if start > end {
        return Err("lint fix range is reversed".to_string());
    }
    let mut edited = String::with_capacity(source.len() + new_text.len());
    edited.push_str(&source[..start]);
    edited.push_str(new_text);
    edited.push_str(&source[end..]);
    Ok(edited)
}

fn byte_offset_for_position(
    source: &str,
    target_line: usize,
    target_character: usize,
) -> Option<usize> {
    let mut line = 0;
    let mut line_start = 0;
    for (offset, character) in source.char_indices() {
        if line == target_line {
            line_start = offset;
            break;
        }
        if character == '\n' {
            line += 1;
            line_start = offset + character.len_utf8();
        }
    }
    if line != target_line {
        if target_line == line && line_start == source.len() {
            return (target_character == 0).then_some(source.len());
        }
        return None;
    }
    let line_source = source[line_start..]
        .split_once('\n')
        .map_or(&source[line_start..], |(line_source, _)| line_source);
    let mut utf16_offset = 0;
    for (byte_offset, character) in line_source.char_indices() {
        if utf16_offset == target_character {
            return Some(line_start + byte_offset);
        }
        utf16_offset += character.len_utf16();
        if utf16_offset > target_character {
            return None;
        }
    }
    (utf16_offset == target_character).then_some(line_start + line_source.len())
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
    fn manual_review_lint_fix_is_reported_and_rejected_without_writing() -> Result<(), String> {
        let path = fixture_root("manual-review.css");
        fs::write(&path, ".known {}\n").map_err(|error| error.to_string())?;
        let candidate = lint_fix_candidate(
            "missing-static-class",
            path_string(path.as_path()).as_str(),
            ParserRangeV0 {
                start: omena_query::ParserPositionV0 {
                    line: 1,
                    character: 0,
                },
                end: omena_query::ParserPositionV0 {
                    line: 1,
                    character: 0,
                },
            },
            ".missing {}\n",
        );
        let preview = apply_lint_fix_requests(std::slice::from_ref(&candidate), false)?;
        assert_eq!(preview.manual_review_edit_count, 1);
        assert_eq!(preview.status, "manualReviewOnly");

        let denied = apply_lint_fix_requests(std::slice::from_ref(&candidate), true)?;
        assert_eq!(denied.applied_edit_count, 0);
        assert_eq!(denied.rejection_count, 1);
        assert_eq!(denied.status, "rejectedByFixSafety");
        assert_eq!(
            fs::read_to_string(&path).map_err(|error| error.to_string())?,
            ".known {}\n"
        );
        fs::remove_file(path).map_err(|error| error.to_string())?;
        Ok(())
    }

    #[test]
    fn conservative_lint_fix_requires_an_explicit_write_mode() -> Result<(), String> {
        let path = fixture_root("conservative.css");
        fs::write(&path, ".known {}\n").map_err(|error| error.to_string())?;
        let mut candidate = lint_fix_candidate(
            "missing-static-class",
            path_string(path.as_path()).as_str(),
            ParserRangeV0 {
                start: omena_query::ParserPositionV0 {
                    line: 1,
                    character: 0,
                },
                end: omena_query::ParserPositionV0 {
                    line: 1,
                    character: 0,
                },
            },
            ".missing {}\n",
        );
        candidate.assessment = compute_fix_safety(FixSafetyEvidenceInputV0 {
            syntax_preserving: true,
            local_semantics_required: false,
            local_semantics_ready: false,
            closed_world_required: false,
            closed_world_ready: false,
            reference_precision_required: true,
            reference_precision: Some(omena_query::FactPrecision::Conservative),
        });
        let denied = apply_lint_fix_requests(&[candidate], true)?;
        assert_eq!(denied.conservative_edit_count, 1);
        assert_eq!(denied.applied_edit_count, 0);
        assert_eq!(denied.rejection_count, 1);
        assert_eq!(
            fs::read_to_string(&path).map_err(|error| error.to_string())?,
            ".known {}\n"
        );
        fs::remove_file(path).map_err(|error| error.to_string())?;
        Ok(())
    }

    #[test]
    fn utf16_text_edit_positions_preserve_non_ascii_prefixes() -> Result<(), String> {
        let source = "/* 🍊 */\n.known {}\n";
        let range = ParserRangeV0 {
            start: omena_query::ParserPositionV0 {
                line: 1,
                character: 9,
            },
            end: omena_query::ParserPositionV0 {
                line: 1,
                character: 9,
            },
        };
        assert_eq!(
            apply_text_edit(source, range, "\n.missing {}")?,
            "/* 🍊 */\n.known {}\n.missing {}\n"
        );
        Ok(())
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
