use crate::{
    commands::ReportCommand,
    diagnostics::{
        parse_external_module_mode, read_external_sifs, read_lock_external_sifs,
        resolve_in_process_external_sifs,
    },
    io::{read_package_manifests, read_source_documents, read_style_sources},
    output::print_json,
};
use omena_query::{
    OmenaQueryDiagnosticSuppressionModeV0, OmenaQueryDiagnosticSuppressionReasonV0,
    OmenaQueryStyleDiagnosticsForFileV0, OmenaQueryStyleResolutionInputsV0,
    summarize_omena_query_sass_module_conformance_v0,
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_suppression_mode,
    summarize_omena_query_style_resolution_policy_v0,
};
use serde::Serialize;
use std::path::PathBuf;

pub(crate) fn report_command(command: ReportCommand) -> Result<(), String> {
    match command {
        ReportCommand::Soundiness {
            source_paths,
            source_document_paths,
            package_manifest_paths,
            sif_paths,
            lockfile,
            external,
            no_suppress,
            max_suppressions,
            report_stale_suppressions,
            json,
        } => report_soundiness(
            source_paths,
            source_document_paths,
            package_manifest_paths,
            sif_paths,
            lockfile,
            external,
            no_suppress,
            max_suppressions,
            report_stale_suppressions,
            json,
        ),
        ReportCommand::ResolutionPolicy { json } => report_resolution_policy(json),
        ReportCommand::SassModuleConformance { json } => report_sass_module_conformance(json),
    }
}

fn report_resolution_policy(json: bool) -> Result<(), String> {
    let report = summarize_omena_query_style_resolution_policy_v0();
    if json {
        print_json(&report)?;
    } else {
        println!(
            "{} candidateStrategy={} networkAccess={}",
            report.product, report.candidate_strategy, report.network_access
        );
        for step in &report.steps {
            println!(
                "{} {}: {} ({})",
                step.order, step.key, step.precedence, step.candidate_semantics
            );
        }
    }
    Ok(())
}

fn report_sass_module_conformance(json: bool) -> Result<(), String> {
    let report = summarize_omena_query_sass_module_conformance_v0();
    if json {
        print_json(&report)?;
    } else {
        println!(
            "{} claimLevel={} theoremClaimed={}",
            report.product, report.claim_level, report.theorem_claimed
        );
        for row in &report.rows {
            println!("{} [{}]: {}", row.key, row.status, row.decision);
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SoundinessReportV0 {
    schema_version: &'static str,
    product: &'static str,
    file_count: usize,
    line_count: usize,
    original_diagnostic_count: usize,
    emitted_diagnostic_count: usize,
    suppressed_diagnostic_count: usize,
    unused_expect_error_count: usize,
    diagnostic_suppression_mode: &'static str,
    boundary_diagnostics: SoundinessBoundaryDiagnosticsV0,
    strictness_distribution: SoundinessStrictnessDistributionV0,
    suppression_reasons: Vec<OmenaQueryDiagnosticSuppressionReasonV0>,
    file_reports: Vec<SoundinessFileReportV0>,
    noise_budget: SoundinessNoiseBudgetV0,
    ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct SoundinessBoundaryDiagnosticsV0 {
    stale_external_sif: usize,
    partial_external_sif: usize,
    missing_external_sif: usize,
    unresolved_external_reference: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct SoundinessStrictnessDistributionV0 {
    relaxed: usize,
    standard: usize,
    strict: usize,
    closed: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SoundinessFileReportV0 {
    file_uri: String,
    line_count: usize,
    original_diagnostic_count: usize,
    emitted_diagnostic_count: usize,
    suppressed_diagnostic_count: usize,
    unused_expect_error_count: usize,
    diagnostic_suppression_mode: &'static str,
    suppression_reasons: Vec<OmenaQueryDiagnosticSuppressionReasonV0>,
    suppressed_per_100_loc: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SoundinessNoiseBudgetV0 {
    per_pr_suppressed_diagnostic_ratio: SoundinessNoiseBudgetCheckV0,
    per_file_suppressed_density: SoundinessNoiseBudgetCheckV0,
    project_suppression_rate: SoundinessNoiseBudgetCheckV0,
    within_budget: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SoundinessNoiseBudgetCheckV0 {
    metric: &'static str,
    value: f64,
    threshold: f64,
    status: &'static str,
}

#[allow(clippy::too_many_arguments)]
fn report_soundiness(
    source_paths: Vec<PathBuf>,
    source_document_paths: Vec<PathBuf>,
    package_manifest_paths: Vec<PathBuf>,
    sif_paths: Vec<PathBuf>,
    lockfile: Option<PathBuf>,
    external: String,
    no_suppress: bool,
    max_suppressions: Option<usize>,
    report_stale_suppressions: bool,
    json: bool,
) -> Result<(), String> {
    let report = summarize_soundiness_report(
        source_paths,
        source_document_paths,
        package_manifest_paths,
        sif_paths,
        lockfile,
        external,
        no_suppress,
    )?;
    enforce_soundiness_report_audit_flags(&report, max_suppressions, report_stale_suppressions)?;
    if json {
        print_json(&report)?;
    } else {
        println!("files analysed: {}", report.file_count);
        println!("diagnostics emitted: {}", report.emitted_diagnostic_count);
        println!(
            "diagnostics suppressed: {}",
            report.suppressed_diagnostic_count
        );
        println!(
            "noise budget: {}",
            if report.noise_budget.within_budget {
                "within limits"
            } else {
                "review recommended"
            }
        );
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn summarize_soundiness_report(
    source_paths: Vec<PathBuf>,
    source_document_paths: Vec<PathBuf>,
    package_manifest_paths: Vec<PathBuf>,
    sif_paths: Vec<PathBuf>,
    lockfile: Option<PathBuf>,
    external: String,
    no_suppress: bool,
) -> Result<SoundinessReportV0, String> {
    if source_paths.is_empty() {
        return Err("omena report soundiness requires at least one --source <path>".to_string());
    }

    let style_sources = read_style_sources(source_paths.as_slice())?;
    let source_documents = read_source_documents(source_document_paths.as_slice())?;
    let package_manifests = read_package_manifests(package_manifest_paths.as_slice())?;
    let mut external_sifs = read_external_sifs(sif_paths.as_slice())?;
    if let Some(lockfile) = lockfile.as_ref() {
        external_sifs.extend(read_lock_external_sifs(lockfile)?);
    }
    let resolution_inputs = OmenaQueryStyleResolutionInputsV0 {
        package_manifests: package_manifests.clone(),
        ..OmenaQueryStyleResolutionInputsV0::default()
    };
    let in_process_external_sifs = resolve_in_process_external_sifs(
        style_sources.as_slice(),
        external_sifs.as_slice(),
        &resolution_inputs,
    );
    external_sifs.extend(in_process_external_sifs);
    let external_mode = parse_external_module_mode(&external)?;
    let suppression_mode = if no_suppress {
        OmenaQueryDiagnosticSuppressionModeV0::ReportOnly
    } else {
        OmenaQueryDiagnosticSuppressionModeV0::Apply
    };

    let mut boundary_diagnostics = SoundinessBoundaryDiagnosticsV0::default();
    let mut strictness_distribution = SoundinessStrictnessDistributionV0::default();
    let mut file_reports = Vec::new();
    let mut original_diagnostic_count = 0usize;
    let mut emitted_diagnostic_count = 0usize;
    let mut suppressed_diagnostic_count = 0usize;
    let mut unused_expect_error_count = 0usize;
    let mut suppression_reasons = Vec::new();
    let mut line_count = 0usize;

    for source in &style_sources {
        let summary =
            summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_suppression_mode(
                source.style_path.as_str(),
                style_sources.as_slice(),
                source_documents.as_slice(),
                package_manifests.as_slice(),
                None,
                external_mode,
                external_sifs.as_slice(),
                suppression_mode,
            )
            .ok_or_else(|| {
                format!(
                    "failed to read workspace style diagnostics for {}",
                    source.style_path
                )
            })?;
        let file_line_count = source.style_source.lines().count().max(1);
        line_count += file_line_count;
        strictness_distribution.add(parse_report_strictness_label(&source.style_source));
        boundary_diagnostics.add_summary(&summary);
        let suppression = summary.suppression_summary.as_ref();
        let original = suppression
            .map(|summary| summary.original_diagnostic_count)
            .unwrap_or(summary.diagnostic_count);
        let emitted = summary.diagnostic_count;
        let suppressed = suppression
            .map(|summary| summary.suppressed_diagnostic_count)
            .unwrap_or(0);
        let unused_expect_errors = suppression
            .map(|summary| summary.unused_expect_error_count)
            .unwrap_or(0);
        let file_suppression_reasons = suppression
            .map(|summary| summary.suppression_reasons.clone())
            .unwrap_or_default();
        original_diagnostic_count += original;
        emitted_diagnostic_count += emitted;
        suppressed_diagnostic_count += suppressed;
        unused_expect_error_count += unused_expect_errors;
        suppression_reasons.extend(file_suppression_reasons.iter().cloned());
        file_reports.push(SoundinessFileReportV0 {
            file_uri: source.style_path.clone(),
            line_count: file_line_count,
            original_diagnostic_count: original,
            emitted_diagnostic_count: emitted,
            suppressed_diagnostic_count: suppressed,
            unused_expect_error_count: unused_expect_errors,
            diagnostic_suppression_mode: suppression_mode.as_str(),
            suppression_reasons: file_suppression_reasons,
            suppressed_per_100_loc: ratio_per_100(suppressed, file_line_count),
        });
    }

    let max_file_suppressed_density = file_reports
        .iter()
        .map(|report| report.suppressed_per_100_loc)
        .fold(0.0_f64, f64::max);
    let per_pr_ratio = percentage(suppressed_diagnostic_count, original_diagnostic_count);
    let project_suppression_rate = per_pr_ratio;
    let noise_budget = SoundinessNoiseBudgetV0 {
        per_pr_suppressed_diagnostic_ratio: noise_budget_check(
            "perPrSuppressedDiagnosticRatio",
            per_pr_ratio,
            30.0,
        ),
        per_file_suppressed_density: noise_budget_check(
            "perFileSuppressedDiagnosticsPer100Loc",
            max_file_suppressed_density,
            5.0,
        ),
        project_suppression_rate: noise_budget_check(
            "projectSuppressionRate",
            project_suppression_rate,
            20.0,
        ),
        within_budget: per_pr_ratio <= 30.0
            && max_file_suppressed_density <= 5.0
            && project_suppression_rate <= 20.0,
    };

    Ok(SoundinessReportV0 {
        schema_version: "0",
        product: "omena-cli.soundiness-report",
        file_count: style_sources.len(),
        line_count,
        original_diagnostic_count,
        emitted_diagnostic_count,
        suppressed_diagnostic_count,
        unused_expect_error_count,
        diagnostic_suppression_mode: suppression_mode.as_str(),
        boundary_diagnostics,
        strictness_distribution,
        suppression_reasons,
        file_reports,
        noise_budget,
        ready_surfaces: vec![
            "soundinessReport",
            "externalBoundaryStateSummary",
            "diagnosticSuppressionRateSummary",
            "diagnosticSuppressionReasonSummary",
            "noiseBudgetVisibilityGates",
        ],
    })
}

fn enforce_soundiness_report_audit_flags(
    report: &SoundinessReportV0,
    max_suppressions: Option<usize>,
    report_stale_suppressions: bool,
) -> Result<(), String> {
    if let Some(max_suppressions) = max_suppressions
        && report.suppressed_diagnostic_count > max_suppressions
    {
        return Err(format!(
            "suppression budget exceeded: {} suppressions observed, max {}",
            report.suppressed_diagnostic_count, max_suppressions
        ));
    }
    if report_stale_suppressions && report.unused_expect_error_count > 0 {
        return Err(format!(
            "stale suppressions observed: {} unused omena-expect-error directives",
            report.unused_expect_error_count
        ));
    }
    Ok(())
}

impl SoundinessBoundaryDiagnosticsV0 {
    fn add_summary(&mut self, summary: &OmenaQueryStyleDiagnosticsForFileV0) {
        for diagnostic in &summary.diagnostics {
            match diagnostic.code {
                "staleExternalSif" => self.stale_external_sif += 1,
                "partialExternalSif" => self.partial_external_sif += 1,
                "missingExternalSif" => self.missing_external_sif += 1,
                "unresolvedExternalReference" => self.unresolved_external_reference += 1,
                _ => {}
            }
        }
    }
}

impl SoundinessStrictnessDistributionV0 {
    fn add(&mut self, strictness: &'static str) {
        match strictness {
            "relaxed" => self.relaxed += 1,
            "strict" => self.strict += 1,
            "closed" => self.closed += 1,
            _ => self.standard += 1,
        }
    }
}

fn parse_report_strictness_label(source: &str) -> &'static str {
    let mut level = "standard";
    for line in source.lines() {
        let Some(offset) = line.find("@omena-strict") else {
            continue;
        };
        let tail = &line[offset + "@omena-strict".len()..];
        for token in tail
            .split(|character: char| !character.is_ascii_alphanumeric() && character != '-')
            .filter(|token| !token.is_empty())
        {
            match token {
                "relaxed" => level = "relaxed",
                "standard" => level = "standard",
                "strict" => level = "strict",
                "closed" => level = "closed",
                _ => {}
            }
        }
    }
    level
}

fn noise_budget_check(
    metric: &'static str,
    value: f64,
    threshold: f64,
) -> SoundinessNoiseBudgetCheckV0 {
    SoundinessNoiseBudgetCheckV0 {
        metric,
        value,
        threshold,
        status: if value <= threshold {
            "within"
        } else {
            "review"
        },
    }
}

fn percentage(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        return 0.0;
    }
    (numerator as f64 / denominator as f64) * 100.0
}

fn ratio_per_100(count: usize, line_count: usize) -> f64 {
    if line_count == 0 {
        return 0.0;
    }
    (count as f64 / line_count as f64) * 100.0
}
