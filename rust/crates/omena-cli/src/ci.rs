use std::{collections::BTreeMap, fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    config::find_omena_config_for_path,
    format::build_format_report,
    lint::lint_check_report,
    modules::{has_css_module_sources, modules_check_report},
    output::{CliOutputMetadataV0, print_json},
    paths::path_string,
    verification::{ci_adapters, verify_workspace},
};

const VERB_MANIFEST_SOURCE: &str = include_str!("../verb-census.json");

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VerbManifestV0 {
    schema_version: String,
    product: String,
    verbs: Vec<VerbRowV0>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VerbRowV0 {
    verb: String,
    status: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
enum CiComponentOutcomeV0 {
    Passed,
    Failed,
    Indeterminate,
    NotYetAvailable,
    Skipped,
}

impl CiComponentOutcomeV0 {
    const fn is_blocking_failure(self) -> bool {
        matches!(self, Self::Failed | Self::Indeterminate)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CiComponentReportV0 {
    verb: String,
    verb_status: String,
    outcome: CiComponentOutcomeV0,
    summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CiPolicyReportV0 {
    policy: &'static str,
    configured_value: Option<String>,
    outcome: CiComponentOutcomeV0,
    summary: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CiReportV0 {
    schema_version: &'static str,
    product: &'static str,
    workspace_root: String,
    component_count: usize,
    policy_count: usize,
    check_count: usize,
    executed_count: usize,
    passed_count: usize,
    failed_count: usize,
    indeterminate_count: usize,
    not_yet_available_count: usize,
    skipped_count: usize,
    blocking_failure_count: usize,
    components: Vec<CiComponentReportV0>,
    configured_policies: Vec<CiPolicyReportV0>,
}

pub(crate) fn ci_command(root: Option<PathBuf>, json: bool) -> Result<(), String> {
    let requested_root = root.unwrap_or_else(|| PathBuf::from("."));
    let root = fs::canonicalize(&requested_root).map_err(|error| {
        format!(
            "failed to resolve CI root {}: {error}",
            path_string(requested_root.as_path())
        )
    })?;
    let manifest = verb_manifest()?;
    let adapters = ci_adapters()?
        .into_iter()
        .map(|adapter| (adapter.verb, adapter.executor))
        .collect::<BTreeMap<_, _>>();
    let components = compose_components(root.as_path(), manifest.verbs, &adapters);
    let loaded_config = find_omena_config_for_path(root.as_path())?;
    let configured_policies = configured_policy_reports(loaded_config.as_deref());
    let report = build_ci_report(path_string(root.as_path()), components, configured_policies);
    if json {
        print_json(
            CliOutputMetadataV0::new("omena-cli.ci").with_config_content_digest(
                loaded_config
                    .as_ref()
                    .map(|loaded| loaded.config_content_digest.as_ref()),
            ),
            &report,
        )?;
    } else {
        println!(
            "CI verification: {} passed, {} failed, {} indeterminate, {} not yet available, {} skipped",
            report.passed_count,
            report.failed_count,
            report.indeterminate_count,
            report.not_yet_available_count,
            report.skipped_count
        );
        for component in &report.components {
            println!(
                "{:?}\t{}\t{}",
                component.outcome, component.verb, component.summary
            );
        }
    }
    if report.blocking_failure_count > 0 {
        return Err(format!(
            "CI verification failed closed with {} blocking component(s)",
            report.blocking_failure_count
        ));
    }
    Ok(())
}

fn build_ci_report(
    workspace_root: String,
    components: Vec<CiComponentReportV0>,
    configured_policies: Vec<CiPolicyReportV0>,
) -> CiReportV0 {
    let component_count = components.len();
    let policy_count = configured_policies.len();
    let count_outcome = |outcome| {
        count_components(&components, outcome) + count_policies(&configured_policies, outcome)
    };
    let passed_count = count_outcome(CiComponentOutcomeV0::Passed);
    let failed_count = count_outcome(CiComponentOutcomeV0::Failed);
    let indeterminate_count = count_outcome(CiComponentOutcomeV0::Indeterminate);
    let not_yet_available_count = count_outcome(CiComponentOutcomeV0::NotYetAvailable);
    let skipped_count = count_outcome(CiComponentOutcomeV0::Skipped);
    let blocking_failure_count = components
        .iter()
        .filter(|component| component.outcome.is_blocking_failure())
        .count()
        + configured_policies
            .iter()
            .filter(|policy| policy.outcome.is_blocking_failure())
            .count();
    CiReportV0 {
        schema_version: "0",
        product: "omena-cli.ci",
        workspace_root,
        component_count,
        policy_count,
        check_count: component_count + policy_count,
        executed_count: passed_count + failed_count + indeterminate_count,
        passed_count,
        failed_count,
        indeterminate_count,
        not_yet_available_count,
        skipped_count,
        blocking_failure_count,
        components,
        configured_policies,
    }
}

fn compose_components(
    root: &std::path::Path,
    rows: Vec<VerbRowV0>,
    adapters: &BTreeMap<String, String>,
) -> Vec<CiComponentReportV0> {
    let mut components = Vec::with_capacity(rows.len());
    for row in rows {
        let component = if row.status != "wired" {
            CiComponentReportV0 {
                verb: row.verb,
                verb_status: row.status,
                outcome: CiComponentOutcomeV0::Skipped,
                summary: "the product verb is not directly wired".to_string(),
                details: None,
            }
        } else if let Some(executor) = adapters.get(row.verb.as_str()) {
            execute_adapter(root, row, executor.as_str())
        } else {
            CiComponentReportV0 {
                verb: row.verb,
                verb_status: row.status,
                outcome: CiComponentOutcomeV0::Skipped,
                summary: "the wired product verb has no read-only CI check contract".to_string(),
                details: None,
            }
        };
        components.push(component);
    }
    components
}

fn execute_adapter(root: &std::path::Path, row: VerbRowV0, executor: &str) -> CiComponentReportV0 {
    let result = match executor {
        "verify" => verify_workspace(Some(root.to_path_buf()), false).and_then(|execution| {
            let report = execution.report;
            let outcome = if report.blocking_failure_count > 0 {
                CiComponentOutcomeV0::Failed
            } else if report.not_yet_available_count > 0 && report.passed_count == 0 {
                CiComponentOutcomeV0::NotYetAvailable
            } else {
                CiComponentOutcomeV0::Passed
            };
            component_result(
                outcome,
                format!(
                    "{} verification item(s), {} blocking failure(s), {} not yet available",
                    report.item_count,
                    report.blocking_failure_count,
                    report.not_yet_available_count
                ),
                &report,
            )
        }),
        "lint" => lint_check_report(Some(root.to_path_buf())).and_then(|report| {
            component_result(
                if report.finding_count == 0 {
                    CiComponentOutcomeV0::Passed
                } else {
                    CiComponentOutcomeV0::Failed
                },
                format!("{} lint finding(s)", report.finding_count),
                &report,
            )
        }),
        "formatCheck" => {
            build_format_report(Some(root.to_path_buf()), None, true).and_then(|report| {
                let failure_count = report.changed_file_count + report.non_idempotent_file_count;
                component_result(
                    if failure_count == 0 {
                        CiComponentOutcomeV0::Passed
                    } else {
                        CiComponentOutcomeV0::Failed
                    },
                    format!(
                        "{} source(s) require formatting and {} source(s) were non-idempotent",
                        report.changed_file_count, report.non_idempotent_file_count
                    ),
                    &report,
                )
            })
        }
        "modulesCheck" => match has_css_module_sources(root) {
            Ok(false) => Ok((
                CiComponentOutcomeV0::Skipped,
                "no CSS Module source was discovered".to_string(),
                None,
            )),
            Ok(true) => modules_check_report(Some(root.to_path_buf())).and_then(|report| {
                component_result(
                    if report.drift_count == 0 {
                        CiComponentOutcomeV0::Passed
                    } else {
                        CiComponentOutcomeV0::Failed
                    },
                    format!("{} module artifact(s) drifted", report.drift_count),
                    &report,
                )
            }),
            Err(error) => Err(error),
        },
        _ => Err(format!("unsupported CI adapter executor {executor}")),
    };
    match result {
        Ok((outcome, summary, details)) => CiComponentReportV0 {
            verb: row.verb,
            verb_status: row.status,
            outcome,
            summary,
            details,
        },
        Err(error) => CiComponentReportV0 {
            verb: row.verb,
            verb_status: row.status,
            outcome: CiComponentOutcomeV0::Indeterminate,
            summary: format!("CI component could not produce a verdict: {error}"),
            details: None,
        },
    }
}

fn component_result<T: Serialize>(
    outcome: CiComponentOutcomeV0,
    summary: String,
    report: &T,
) -> Result<(CiComponentOutcomeV0, String, Option<Value>), String> {
    serde_json::to_value(report)
        .map(|details| (outcome, summary, Some(details)))
        .map_err(|error| format!("failed to serialize CI component evidence: {error}"))
}

fn configured_policy_reports(
    loaded: Option<&crate::config::LoadedOmenaConfig>,
) -> Vec<CiPolicyReportV0> {
    let precision = loaded.and_then(|loaded| loaded.config.ci.precision_regression.clone());
    let rejection = loaded.and_then(|loaded| loaded.config.ci.transform_rejection.clone());
    vec![
        policy_report(
            "precisionRegression",
            precision,
            "warn",
            "a persisted workspace precision comparison report is not available",
        ),
        policy_report(
            "transformRejection",
            rejection,
            "error",
            "a workspace-level transform rejection aggregate is not available",
        ),
    ]
}

fn policy_report(
    policy: &'static str,
    configured_value: Option<String>,
    expected_value: &'static str,
    limitation: &'static str,
) -> CiPolicyReportV0 {
    let (outcome, summary) = match configured_value.as_deref() {
        None => (
            CiComponentOutcomeV0::Skipped,
            "the policy is not configured".to_string(),
        ),
        Some(value) if value == expected_value => (
            CiComponentOutcomeV0::NotYetAvailable,
            limitation.to_string(),
        ),
        Some(value) => (
            CiComponentOutcomeV0::Indeterminate,
            format!("unsupported value {value}; expected {expected_value}"),
        ),
    };
    CiPolicyReportV0 {
        policy,
        configured_value,
        outcome,
        summary,
    }
}

fn count_components(components: &[CiComponentReportV0], outcome: CiComponentOutcomeV0) -> usize {
    components
        .iter()
        .filter(|component| component.outcome == outcome)
        .count()
}

fn count_policies(policies: &[CiPolicyReportV0], outcome: CiComponentOutcomeV0) -> usize {
    policies
        .iter()
        .filter(|policy| policy.outcome == outcome)
        .count()
}

fn verb_manifest() -> Result<VerbManifestV0, String> {
    let manifest: VerbManifestV0 = serde_json::from_str(VERB_MANIFEST_SOURCE)
        .map_err(|error| format!("failed to decode the product verb manifest: {error}"))?;
    if manifest.schema_version != "0" || manifest.product != "omena-cli.product-verb-census" {
        return Err("unsupported product verb manifest contract".to_string());
    }
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn workspace_verification_failure_propagates_to_ci_exit() -> Result<(), String> {
        let root = fixture_root("failure-propagation");
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        fs::write(root.join("broken.css"), ".app { color: red;")
            .map_err(|error| error.to_string())?;

        let Err(error) = ci_command(Some(root.clone()), false) else {
            return Err("CI accepted a workspace verification failure".to_string());
        };
        assert!(error.contains("CI verification failed closed"));

        fs::remove_dir_all(root).map_err(|error| error.to_string())?;
        Ok(())
    }

    #[test]
    fn unknown_ci_policy_value_is_indeterminate() {
        let report = policy_report(
            "precisionRegression",
            Some("ignore".to_string()),
            "warn",
            "fixture limitation",
        );
        assert_eq!(report.outcome, CiComponentOutcomeV0::Indeterminate);
        assert!(report.outcome.is_blocking_failure());
    }

    #[test]
    fn product_manifest_rows_are_composed_once_and_non_wired_verbs_stay_skipped()
    -> Result<(), String> {
        let root = fixture_root("manifest-composition");
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        fs::write(root.join("valid.css"), ".app { color: red; }\n")
            .map_err(|error| error.to_string())?;

        let manifest = verb_manifest()?;
        let expected_verbs = manifest
            .verbs
            .iter()
            .map(|row| row.verb.clone())
            .collect::<BTreeSet<_>>();
        let expected_count = manifest.verbs.len();
        let mut adapters = ci_adapters()?
            .into_iter()
            .map(|adapter| (adapter.verb, adapter.executor))
            .collect::<BTreeMap<_, _>>();
        adapters.insert("check".to_string(), "unsupported".to_string());

        let components = compose_components(root.as_path(), manifest.verbs, &adapters);
        let actual_verbs = components
            .iter()
            .map(|component| component.verb.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(components.len(), expected_count);
        assert_eq!(actual_verbs, expected_verbs);
        let reserved = components
            .iter()
            .find(|component| component.verb == "check")
            .ok_or_else(|| "reserved alias is missing from the CI report".to_string())?;
        assert_eq!(reserved.verb_status, "reserved-alias");
        assert_eq!(reserved.outcome, CiComponentOutcomeV0::Skipped);
        assert_eq!(reserved.summary, "the product verb is not directly wired");
        let verify_details = components
            .iter()
            .find(|component| component.verb == "verify")
            .and_then(|component| component.details.as_ref());
        assert_eq!(
            verify_details
                .and_then(|details| details.get("product"))
                .and_then(Value::as_str),
            Some("omena-cli.verify")
        );
        assert!(
            verify_details
                .and_then(|details| details.get("notYetAvailableCount"))
                .and_then(Value::as_u64)
                .is_some_and(|count| count > 0)
        );
        let report = build_ci_report(
            path_string(root.as_path()),
            components,
            configured_policy_reports(None),
        );
        assert_eq!(report.component_count, expected_count);
        assert_eq!(report.policy_count, 2);
        assert_eq!(report.check_count, expected_count + 2);
        assert_eq!(
            report.passed_count
                + report.failed_count
                + report.indeterminate_count
                + report.not_yet_available_count
                + report.skipped_count,
            report.check_count
        );

        fs::remove_dir_all(root).map_err(|error| error.to_string())?;
        Ok(())
    }

    fn fixture_root(label: &str) -> PathBuf {
        let id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("omena-ci-{label}-{}-{id}", std::process::id()))
    }
}
