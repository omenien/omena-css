use std::path::PathBuf;

use omena_query::{
    OmenaQueryBundleEvidenceManifestV0, OmenaQueryBundleExecutionScopeEvidenceV0,
    OmenaQueryBundlePlanInputV0, OmenaQueryBundleResultV0, OmenaQueryClosedWorldOutcomeV0,
    OmenaQueryConsumerBuildOptionsV0, OmenaQueryTransformExecutionContextV0,
    run_omena_query_bundle_with_execution_scope_evidence_and_options,
    summarize_omena_query_bundle_evidence,
};
use serde::Serialize;

use crate::{
    build::{
        append_bundle_build_passes, resolution_inputs_for_build_path,
        rewrite_bundle_asset_urls_for_build_sources,
    },
    config::{find_omena_build_config_for_path, resolve_config_paths},
    external_sif_authority::{
        CliExternalSifLockFailureModeV0, CliExternalSifSelectionV0,
        resolve_cli_external_sif_authority,
    },
    io::{read_package_manifests, read_source, read_workspace_sources},
    output::{commit_json_artifact, commit_text_artifact},
    paths::path_string,
    workspace_edit_transaction::{ExpectedContentDigestV0, WorkspaceEditSafetyClassV0},
};

pub(crate) struct BundleCommandOptions {
    pub entry: Option<PathBuf>,
    pub css_out: Option<PathBuf>,
    pub evidence_path: Option<PathBuf>,
    pub source_paths: Vec<PathBuf>,
    pub package_manifest_paths: Vec<PathBuf>,
    pub external_sif_selection: CliExternalSifSelectionV0,
}

pub(crate) struct BundlePlanV0 {
    pub(crate) result: OmenaQueryBundleResultV0,
    pub(crate) evidence: OmenaQueryBundleEvidenceManifestV0,
    pub(crate) execution_scope: Option<OmenaQueryBundleExecutionScopeEvidenceV0>,
}

pub(crate) fn bundle_command(options: BundleCommandOptions) -> Result<(), String> {
    let expected_evidence_digest = options
        .evidence_path
        .as_deref()
        .map(ExpectedContentDigestV0::observe)
        .transpose()
        .map_err(|error| error.to_string())?;
    let expected_css_digest = options
        .css_out
        .as_deref()
        .map(ExpectedContentDigestV0::observe)
        .transpose()
        .map_err(|error| error.to_string())?;
    let plan = plan_bundle(&options)?;
    if let Some(evidence_path) = options.evidence_path.as_deref() {
        commit_json_artifact(
            evidence_path,
            expected_evidence_digest.ok_or_else(|| {
                "bundle evidence digest was not captured before analysis".to_string()
            })?,
            WorkspaceEditSafetyClassV0::EvidenceRequired,
            &BundleEvidenceOutputV0 {
                evidence: &plan.evidence,
                execution_scope: plan.execution_scope.as_ref(),
            },
        )?;
    }

    if let OmenaQueryClosedWorldOutcomeV0::Open { blockers } = &plan.result.closed_world_outcome {
        let blockers = serde_json::to_string(blockers)
            .map_err(|error| format!("failed to serialize bundle blockers: {error}"))?;
        return Err(format!(
            "closed-world bundle admission failed with typed blockers: {blockers}"
        ));
    }

    if let Some(css_out) = options.css_out.as_deref() {
        commit_text_artifact(
            css_out,
            expected_css_digest
                .ok_or_else(|| "bundle CSS digest was not captured before analysis".to_string())?,
            WorkspaceEditSafetyClassV0::EvidenceRequired,
            plan.result.artifact.output_css.as_bytes(),
        )?;
    } else {
        print!("{}", plan.result.artifact.output_css);
    }
    Ok(())
}

pub(crate) fn plan_bundle(options: &BundleCommandOptions) -> Result<BundlePlanV0, String> {
    let entry = options
        .entry
        .as_ref()
        .ok_or_else(|| "omena bundle requires an entry stylesheet".to_string())?;
    let entry_source = read_source(entry)?;
    let entry_style_path = path_string(entry);
    let mut source_paths = options.source_paths.clone();
    let mut package_manifest_paths = options.package_manifest_paths.clone();
    if let Some(config) = find_omena_build_config_for_path(entry)? {
        for report in config.reports.iter() {
            eprintln!("{}", report.render_warning());
        }
        if source_paths.is_empty()
            && let Some(configured_sources) = config.build.sources.as_ref()
        {
            source_paths = resolve_config_paths(&config.directory, configured_sources);
        }
        if package_manifest_paths.is_empty()
            && let Some(configured_manifests) = config.build.package_manifests.as_ref()
        {
            package_manifest_paths = resolve_config_paths(&config.directory, configured_manifests);
        }
    }
    let original_sources = read_workspace_sources(entry, &entry_source, source_paths.as_slice())?;
    let (style_sources, asset_rewrites) =
        rewrite_bundle_asset_urls_for_build_sources(&original_sources);
    let package_manifests = read_package_manifests(package_manifest_paths.as_slice())?;
    let resolution_inputs = resolution_inputs_for_build_path(entry, &package_manifests);
    let external_sif_authority = resolve_cli_external_sif_authority(
        &options.external_sif_selection,
        original_sources.as_slice(),
        &resolution_inputs,
        CliExternalSifLockFailureModeV0::Refuse,
    )?;

    let mut pass_ids = Vec::new();
    append_bundle_build_passes(&mut pass_ids, &entry_style_path, &entry_source);
    let result = run_omena_query_bundle_with_execution_scope_evidence_and_options(
        OmenaQueryBundlePlanInputV0 {
            target_style_path: &entry_style_path,
            style_sources: &style_sources,
            source_map_sources: &original_sources,
            requested_pass_ids: &pass_ids,
            context: &OmenaQueryTransformExecutionContextV0::default(),
            resolution_inputs: &resolution_inputs,
            asset_rewrites,
            bundle_entry_style_paths: &[],
        },
        external_sif_authority.external_sifs(),
        &OmenaQueryConsumerBuildOptionsV0::default(),
    )?;
    let evidence = summarize_omena_query_bundle_evidence(&result.bundle_result);
    Ok(BundlePlanV0 {
        result: result.bundle_result,
        evidence,
        execution_scope: result.execution_scope,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BundleEvidenceOutputV0<'a> {
    #[serde(flatten)]
    evidence: &'a OmenaQueryBundleEvidenceManifestV0,
    execution_scope: Option<&'a OmenaQueryBundleExecutionScopeEvidenceV0>,
}
