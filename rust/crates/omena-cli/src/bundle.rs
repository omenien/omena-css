use std::path::PathBuf;

use omena_query::{
    OmenaQueryBundlePlanInputV0, OmenaQueryClosedWorldOutcomeV0,
    OmenaQueryTransformExecutionContextV0, run_omena_query_bundle_with_semantic_inputs,
    summarize_omena_query_bundle_evidence,
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

pub(crate) fn bundle_command(options: BundleCommandOptions) -> Result<(), String> {
    let BundleCommandOptions {
        entry,
        css_out,
        evidence_path,
        source_paths,
        package_manifest_paths,
        sif_paths,
        lockfile,
    } = options;
    let entry = entry.ok_or_else(|| "omena bundle requires an entry stylesheet".to_string())?;
    let entry_source = read_source(&entry)?;
    let entry_style_path = path_string(&entry);
    let original_sources = read_workspace_sources(&entry, &entry_source, &source_paths)?;
    let (style_sources, asset_rewrites) =
        rewrite_bundle_asset_urls_for_build_sources(&original_sources);
    let package_manifests = read_package_manifests(&package_manifest_paths)?;
    let resolution_inputs = resolution_inputs_for_build_path(&entry, &package_manifests);
    let mut external_sifs = read_external_sifs(&sif_paths)?;
    if let Some(lockfile) = lockfile.as_deref() {
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
    if let Some(evidence_path) = evidence_path.as_deref() {
        write_json_artifact(evidence_path, &evidence)?;
    }

    if let OmenaQueryClosedWorldOutcomeV0::Open { blockers } = &result.closed_world_outcome {
        let blockers = serde_json::to_string(blockers)
            .map_err(|error| format!("failed to serialize bundle blockers: {error}"))?;
        return Err(format!(
            "closed-world bundle admission failed with typed blockers: {blockers}"
        ));
    }

    if let Some(css_out) = css_out.as_deref() {
        write_artifact(css_out, result.artifact.output_css.as_bytes())?;
    } else {
        print!("{}", result.artifact.output_css);
    }
    Ok(())
}
