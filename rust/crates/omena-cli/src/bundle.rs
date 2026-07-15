use std::path::PathBuf;

use omena_query::{
    OmenaQueryBundleEvidenceManifestV0, OmenaQueryBundlePlanInputV0, OmenaQueryBundleResultV0,
    OmenaQueryClosedWorldOutcomeV0, OmenaQueryTransformExecutionContextV0,
    run_omena_query_bundle_with_semantic_inputs, summarize_omena_query_bundle_evidence,
};

use crate::{
    build::{
        append_bundle_build_passes, resolution_inputs_for_build_path,
        rewrite_bundle_asset_urls_for_build_sources,
    },
    diagnostics::{read_external_sifs, read_lock_external_sifs},
    io::{read_package_manifests, read_source, read_workspace_sources},
    output::{write_artifact, write_json_artifact},
    paths::path_string,
};

pub(crate) struct BundleCommandOptions {
    pub entry: Option<PathBuf>,
    pub css_out: Option<PathBuf>,
    pub evidence_path: Option<PathBuf>,
    pub source_paths: Vec<PathBuf>,
    pub package_manifest_paths: Vec<PathBuf>,
    pub sif_paths: Vec<PathBuf>,
    pub lockfile: Option<PathBuf>,
}

pub(crate) struct BundlePlanV0 {
    pub(crate) result: OmenaQueryBundleResultV0,
    pub(crate) evidence: OmenaQueryBundleEvidenceManifestV0,
}

pub(crate) fn bundle_command(options: BundleCommandOptions) -> Result<(), String> {
    let plan = plan_bundle(&options)?;
    if let Some(evidence_path) = options.evidence_path.as_deref() {
        write_json_artifact(evidence_path, &plan.evidence)?;
    }

    if let OmenaQueryClosedWorldOutcomeV0::Open { blockers } = &plan.result.closed_world_outcome {
        let blockers = serde_json::to_string(blockers)
            .map_err(|error| format!("failed to serialize bundle blockers: {error}"))?;
        return Err(format!(
            "closed-world bundle admission failed with typed blockers: {blockers}"
        ));
    }

    if let Some(css_out) = options.css_out.as_deref() {
        write_artifact(css_out, plan.result.artifact.output_css.as_bytes())?;
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
    let original_sources =
        read_workspace_sources(entry, &entry_source, options.source_paths.as_slice())?;
    let (style_sources, asset_rewrites) =
        rewrite_bundle_asset_urls_for_build_sources(&original_sources);
    let package_manifests = read_package_manifests(options.package_manifest_paths.as_slice())?;
    let resolution_inputs = resolution_inputs_for_build_path(entry, &package_manifests);
    let mut external_sifs = read_external_sifs(options.sif_paths.as_slice())?;
    if let Some(lockfile) = options.lockfile.as_deref() {
        external_sifs.extend(read_lock_external_sifs(lockfile)?);
    }
    external_sifs.sort_by(|left, right| left.canonical_url.cmp(&right.canonical_url));
    external_sifs.dedup_by(|left, right| left.canonical_url == right.canonical_url);

    let mut pass_ids = Vec::new();
    append_bundle_build_passes(&mut pass_ids, &entry_style_path, &entry_source);
    let result = run_omena_query_bundle_with_semantic_inputs(
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
        &external_sifs,
    )?;
    let evidence = summarize_omena_query_bundle_evidence(&result);
    Ok(BundlePlanV0 { result, evidence })
}
