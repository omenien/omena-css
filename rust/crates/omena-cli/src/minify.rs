use std::{fs, path::PathBuf};

use omena_query::{
    OmenaQueryClosedWorldOutcomeV0, OmenaQueryTransformBuildProfileV0,
    OmenaQueryTransformDecisionV0, closed_world_omena_query_minify_build_profile,
    execute_omena_query_consumer_build_style_source_with_context,
    safe_omena_query_minify_build_profile, semantic_omena_query_minify_build_profile,
    summarize_omena_query_closed_world_outcome_for_style_source,
};
use serde::Serialize;

use crate::{
    commands::{MinifyBackend, MinifyProfile},
    config::find_omena_config_for_path,
    io::{read_context_json, read_source},
    minify_backend::{MinifyDelegationReportV0, run_hybrid_lightning_lowering},
    output::{CliOutputMetadataV0, print_json},
    paths::path_string,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MinifyDecisionCoverageV0 {
    decision_count: usize,
    applied_decision_count: usize,
    blocked_decision_count: usize,
    rejected_decision_count: usize,
    incomplete_pass_ids: Vec<&'static str>,
    semantic_removal_count: usize,
    covered_semantic_removal_count: usize,
    decision_mutation_count: usize,
    execution_mutation_count: usize,
    all_mutations_have_typed_decisions: bool,
    all_semantic_removals_have_applied_decisions: bool,
    profile_execution_completed: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MinifyBackendReportV0 {
    requested: &'static str,
    applied: &'static str,
    delegated: bool,
    fallback_to_omena: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    delegation: Option<MinifyDelegationReportV0>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MinifyReportV0 {
    schema_version: &'static str,
    product: &'static str,
    input_path: String,
    profile: OmenaQueryTransformBuildProfileV0,
    backend: MinifyBackendReportV0,
    input_byte_len: usize,
    output_byte_len: usize,
    output_css: String,
    decision_coverage: MinifyDecisionCoverageV0,
    decisions: Vec<OmenaQueryTransformDecisionV0>,
}

pub(crate) fn minify_source(
    input: Option<PathBuf>,
    cli_profile: Option<MinifyProfile>,
    backend: Option<MinifyBackend>,
    context_json: Option<PathBuf>,
    output: Option<PathBuf>,
    json: bool,
) -> Result<(), String> {
    let input = input.ok_or_else(|| "omena minify requires an input stylesheet".to_string())?;
    let source = read_source(&input)?;
    let input_path = path_string(&input);
    let loaded_config = find_omena_config_for_path(&input)?;
    let configured_profile = loaded_config
        .as_ref()
        .and_then(|loaded| loaded.config.minify.profile.as_deref());
    let profile = resolve_minify_profile(cli_profile, configured_profile)?;
    let build_profile = build_profile(profile);
    let pass_ids = build_profile
        .pass_ids
        .iter()
        .map(|pass_id| (*pass_id).to_string())
        .collect::<Vec<_>>();
    let context = read_context_json(context_json.as_deref())?;
    let summary = execute_omena_query_consumer_build_style_source_with_context(
        input_path.as_str(),
        source.as_str(),
        pass_ids.as_slice(),
        &context,
    );

    if !summary.unknown_pass_ids.is_empty() {
        return Err(format!(
            "minify profile contains unknown pass ids: {}",
            summary.unknown_pass_ids.join(", ")
        ));
    }
    if profile == MinifyProfile::ClosedWorld
        && let OmenaQueryClosedWorldOutcomeV0::Open { blockers } =
            summarize_omena_query_closed_world_outcome_for_style_source(
                input_path.as_str(),
                source.as_str(),
                pass_ids.as_slice(),
                &context,
            )
    {
        let blockers = serde_json::to_string(&blockers)
            .map_err(|error| format!("failed to serialize closed-world blockers: {error}"))?;
        return Err(format!(
            "closed-world minification refused typed blockers: {blockers}"
        ));
    }

    let requested_backend = backend.unwrap_or(MinifyBackend::Omena);
    let decision_coverage = decision_coverage(&summary.execution);
    if !decision_coverage.all_mutations_have_typed_decisions
        || !decision_coverage.all_semantic_removals_have_applied_decisions
    {
        return Err(format!(
            "minify execution is missing typed decision evidence: blocked={}, rejected={}, incompletePasses={}, decisionMutations={}, executionMutations={}, uncoveredRemovals={}",
            decision_coverage.blocked_decision_count,
            decision_coverage.rejected_decision_count,
            decision_coverage.incomplete_pass_ids.join(","),
            decision_coverage.decision_mutation_count,
            decision_coverage.execution_mutation_count,
            decision_coverage.semantic_removal_count
                - decision_coverage.covered_semantic_removal_count,
        ));
    }

    let semantic_output_css = summary.execution.output_css;
    let (output_css, backend) = match requested_backend {
        MinifyBackend::Omena => (
            semantic_output_css,
            MinifyBackendReportV0 {
                requested: requested_backend.as_str(),
                applied: "omena",
                delegated: false,
                fallback_to_omena: false,
                delegation: None,
            },
        ),
        MinifyBackend::HybridLightning => {
            let outcome = run_hybrid_lightning_lowering(semantic_output_css.as_str())?;
            let adopted = outcome.report.adopted;
            (
                outcome.output_css,
                MinifyBackendReportV0 {
                    requested: requested_backend.as_str(),
                    applied: if adopted { "hybrid-lightning" } else { "omena" },
                    delegated: true,
                    fallback_to_omena: !adopted,
                    delegation: Some(outcome.report),
                },
            )
        }
        MinifyBackend::Lightning => {
            return Err(
                "the `lightning` backend is comparison-only; use `hybrid-lightning` for fail-closed delegated lowering"
                    .to_string(),
            );
        }
    };

    let report = MinifyReportV0 {
        schema_version: "0",
        product: "omena-cli.minify-report",
        input_path,
        profile: build_profile,
        backend,
        input_byte_len: summary.execution.input_byte_len,
        output_byte_len: output_css.len(),
        output_css,
        decision_coverage,
        decisions: summary.execution.decisions,
    };

    if let Some(output) = output {
        fs::write(&output, report.output_css.as_bytes()).map_err(|error| {
            format!(
                "failed to write minified CSS to {}: {error}",
                path_string(&output)
            )
        })?;
    } else if !json {
        print!("{}", report.output_css);
    }

    if json {
        print_json(
            CliOutputMetadataV0::new("omena-cli.minify").with_config_content_digest(
                loaded_config
                    .as_ref()
                    .map(|loaded| loaded.config_content_digest.as_ref()),
            ),
            &report,
        )?;
    } else {
        eprintln!(
            "minify profile: {}; typed decisions: {}; mutations: {}",
            report.profile.profile_id,
            report.decision_coverage.decision_count,
            report.decision_coverage.execution_mutation_count
        );
    }

    Ok(())
}

fn resolve_minify_profile(
    cli_profile: Option<MinifyProfile>,
    configured_profile: Option<&str>,
) -> Result<MinifyProfile, String> {
    if let Some(profile) = cli_profile {
        return Ok(profile);
    }
    match configured_profile {
        None | Some("semantic") => Ok(MinifyProfile::Semantic),
        Some("safe") => Ok(MinifyProfile::Safe),
        Some("closed-world") => Ok(MinifyProfile::ClosedWorld),
        Some(profile) => Err(format!(
            "unsupported [minify].profile `{profile}`; expected `safe`, `semantic`, or `closed-world`"
        )),
    }
}

fn build_profile(profile: MinifyProfile) -> OmenaQueryTransformBuildProfileV0 {
    match profile {
        MinifyProfile::Safe => safe_omena_query_minify_build_profile(),
        MinifyProfile::Semantic => semantic_omena_query_minify_build_profile(),
        MinifyProfile::ClosedWorld => closed_world_omena_query_minify_build_profile(),
    }
}

fn decision_coverage(
    execution: &omena_query::OmenaQueryTransformExecutionSummaryV0,
) -> MinifyDecisionCoverageV0 {
    let decision_mutation_count = execution
        .decisions
        .iter()
        .map(|decision| decision.compatibility_outcome().mutation_count)
        .sum();
    let applied_decision_count = execution
        .decisions
        .iter()
        .filter(|decision| matches!(decision, OmenaQueryTransformDecisionV0::Applied { .. }))
        .count();
    let blocked_decision_count = execution
        .decisions
        .iter()
        .filter(|decision| matches!(decision, OmenaQueryTransformDecisionV0::Blocked { .. }))
        .count();
    let rejected_decision_count = execution
        .decisions
        .iter()
        .filter(|decision| matches!(decision, OmenaQueryTransformDecisionV0::Rejected { .. }))
        .count();
    let incomplete_pass_ids = execution
        .decisions
        .iter()
        .filter(|decision| {
            matches!(
                decision,
                OmenaQueryTransformDecisionV0::Blocked { .. }
                    | OmenaQueryTransformDecisionV0::Rejected { .. }
            )
        })
        .map(|decision| decision.compatibility_outcome().pass_id)
        .collect();
    let covered_semantic_removal_count = execution
        .semantic_removals
        .iter()
        .filter(|removal| {
            execution.decisions.iter().any(|decision| {
                matches!(decision, OmenaQueryTransformDecisionV0::Applied { .. })
                    && decision.compatibility_outcome().pass_id == removal.pass_id
            })
        })
        .count();

    MinifyDecisionCoverageV0 {
        decision_count: execution.decisions.len(),
        applied_decision_count,
        blocked_decision_count,
        rejected_decision_count,
        incomplete_pass_ids,
        semantic_removal_count: execution.semantic_removals.len(),
        covered_semantic_removal_count,
        decision_mutation_count,
        execution_mutation_count: execution.mutation_count,
        all_mutations_have_typed_decisions: decision_mutation_count == execution.mutation_count,
        all_semantic_removals_have_applied_decisions: covered_semantic_removal_count
            == execution.semantic_removals.len(),
        profile_execution_completed: blocked_decision_count == 0 && rejected_decision_count == 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_line_profile_takes_precedence_over_config() -> Result<(), String> {
        assert_eq!(
            resolve_minify_profile(Some(MinifyProfile::Safe), Some("closed-world"))?,
            MinifyProfile::Safe
        );
        Ok(())
    }

    #[test]
    fn semantic_is_the_compatibility_default_profile() -> Result<(), String> {
        assert_eq!(resolve_minify_profile(None, None)?, MinifyProfile::Semantic);
        assert_eq!(
            resolve_minify_profile(None, Some("semantic"))?,
            MinifyProfile::Semantic
        );
        Ok(())
    }

    #[test]
    fn semantic_removals_are_covered_by_applied_typed_decisions() {
        let profile = closed_world_omena_query_minify_build_profile();
        let pass_ids = profile
            .pass_ids
            .iter()
            .map(|pass_id| (*pass_id).to_string())
            .collect::<Vec<_>>();
        let context = omena_query::OmenaQueryTransformExecutionContextV0 {
            reachable_class_names: vec!["used".to_string()],
            ..omena_query::OmenaQueryTransformExecutionContextV0::default()
        };
        let summary = execute_omena_query_consumer_build_style_source_with_context(
            "fixture.css",
            ".used { color: red; } .dead { color: blue; }",
            pass_ids.as_slice(),
            &context,
        );
        let coverage = decision_coverage(&summary.execution);

        assert!(coverage.semantic_removal_count > 0);
        assert_eq!(
            coverage.covered_semantic_removal_count,
            coverage.semantic_removal_count
        );
        assert!(coverage.all_semantic_removals_have_applied_decisions);
        assert!(coverage.all_mutations_have_typed_decisions);
    }
}
