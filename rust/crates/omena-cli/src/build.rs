use crate::{
    config::{
        apply_configured_target_options, find_omena_build_config_for_path, resolve_config_path,
        resolve_config_paths,
    },
    io::{
        read_context_json, read_engine_input_json, read_package_manifests, read_source,
        read_workspace_sources,
    },
    output::print_json,
    paths::{
        cli_file_uri_to_path, cli_path_to_file_uri, path_string,
        style_resolution_workspace_uri_for_path,
    },
};
use omena_query::{
    OmenaParserStyleDialect, OmenaQueryBundlePlanInputV0, OmenaQueryConsumerBuildSummaryV0,
    OmenaQueryStylePackageManifestV0, OmenaQueryStyleResolutionInputsV0,
    OmenaQueryStyleSourceInputV0, OmenaQueryTargetTransformOptionsV0,
    OmenaQueryTransformBundleAssetUrlRewriteSummaryV0, OmenaQueryTransformExecutionContextV0,
    OmenaQueryTransformSourceMapV3V0, TransformBundleEdgeKind,
    attach_omena_query_consumer_build_bundle_summary,
    attach_omena_query_consumer_build_source_map_v3_with_sources_and_resolution_inputs,
    compose_omena_query_transform_source_map_v3_with_upstream_map,
    execute_omena_query_consumer_build_style_source_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_source_with_context,
    execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options_and_resolution_inputs,
    execute_omena_query_consumer_build_style_sources_with_context_and_resolution_inputs,
    list_omena_query_transform_pass_summaries, load_omena_query_workspace_style_resolution_inputs,
    resolve_omena_query_style_uri_for_specifier_with_resolution_inputs,
    rewrite_omena_transform_bundle_asset_urls_in_source, run_omena_query_bundle,
    summarize_omena_query_bundle_code_split_source_map_v3,
    summarize_omena_query_bundle_code_split_workspace_plan,
    summarize_omena_query_transform_context_from_engine_input,
    summarize_omena_transform_bundle_from_source,
};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

pub(crate) struct BuildFileOptions {
    pub(crate) path: PathBuf,
    pub(crate) output: Option<PathBuf>,
    pub(crate) pass_ids: Vec<String>,
    pub(crate) minify: bool,
    pub(crate) target_query: Option<String>,
    pub(crate) context_json: Option<PathBuf>,
    pub(crate) engine_input_json: Option<PathBuf>,
    pub(crate) closed_style_world: bool,
    pub(crate) tree_shake: bool,
    pub(crate) bundle: bool,
    pub(crate) split_out_dir: Option<PathBuf>,
    pub(crate) bundle_entry_paths: Vec<PathBuf>,
    pub(crate) source_paths: Vec<PathBuf>,
    pub(crate) package_manifest_paths: Vec<PathBuf>,
    pub(crate) source_map: bool,
    pub(crate) input_source_maps: Vec<String>,
    pub(crate) target_options: OmenaQueryTargetTransformOptionsV0,
    pub(crate) json: bool,
}

pub(crate) fn build_file(options: BuildFileOptions) -> Result<(), String> {
    let BuildFileOptions {
        path,
        mut output,
        mut pass_ids,
        mut minify,
        mut target_query,
        mut context_json,
        mut engine_input_json,
        mut closed_style_world,
        mut tree_shake,
        mut bundle,
        mut split_out_dir,
        mut bundle_entry_paths,
        mut source_paths,
        mut package_manifest_paths,
        mut source_map,
        mut input_source_maps,
        mut target_options,
        json,
    } = options;

    if let Some(config) = find_omena_build_config_for_path(&path)? {
        for report in config.reports.iter() {
            eprintln!("{}", report.render_warning());
        }
        let _config_content_digest = config.config_content_digest;
        let build = config.build;
        let config_dir = config.directory;
        if output.is_none() {
            output = build
                .output
                .as_deref()
                .map(|path| resolve_config_path(&config_dir, path));
        }
        if pass_ids.is_empty()
            && let Some(configured_passes) = build.passes.as_ref()
        {
            pass_ids = configured_passes.clone();
        }
        if !minify {
            minify = build.minify.unwrap_or(false);
        }
        if target_query.is_none() {
            target_query = build.target_query.clone();
        }
        if context_json.is_none() {
            context_json = build
                .context_json
                .as_deref()
                .map(|path| resolve_config_path(&config_dir, path));
        }
        if engine_input_json.is_none() {
            engine_input_json = build
                .engine_input_json
                .as_deref()
                .map(|path| resolve_config_path(&config_dir, path));
        }
        if !closed_style_world {
            closed_style_world = build.closed_style_world.unwrap_or(false);
        }
        if !tree_shake {
            tree_shake = build.tree_shake.unwrap_or(false);
        }
        if !bundle {
            bundle = build.bundle.unwrap_or(false);
        }
        if split_out_dir.is_none() {
            split_out_dir = build
                .split_out_dir
                .as_deref()
                .map(|path| resolve_config_path(&config_dir, path));
        }
        if bundle_entry_paths.is_empty()
            && let Some(configured_entries) = build.bundle_entries.as_ref()
        {
            bundle_entry_paths = resolve_config_paths(&config_dir, configured_entries);
        }
        if source_paths.is_empty()
            && let Some(configured_sources) = build.sources.as_ref()
        {
            source_paths = resolve_config_paths(&config_dir, configured_sources);
        }
        if package_manifest_paths.is_empty()
            && let Some(configured_manifests) = build.package_manifests.as_ref()
        {
            package_manifest_paths = resolve_config_paths(&config_dir, configured_manifests);
        }
        if !source_map {
            source_map = build.source_map.unwrap_or(false);
        }
        if input_source_maps.is_empty()
            && let Some(configured_input_source_maps) = build.input_source_maps.as_ref()
        {
            input_source_maps = configured_input_source_maps.clone();
        }
        apply_configured_target_options(&mut target_options, &build);
    }

    if target_query.is_some() && !pass_ids.is_empty() {
        return Err("cannot combine --target-query with explicit --pass values".to_string());
    }
    if target_query.is_some() && minify {
        return Err(
            "cannot combine --target-query with --minify yet; use explicit --pass values for now"
                .to_string(),
        );
    }
    if target_query.is_some() && tree_shake {
        return Err(
            "cannot combine --target-query with --tree-shake; use --tree-shake without --target-query"
                .to_string(),
        );
    }
    if target_query.is_some() && bundle {
        return Err(
            "cannot combine --target-query with --bundle; use --bundle without --target-query"
                .to_string(),
        );
    }
    if split_out_dir.is_some() && !bundle {
        return Err("--split-out-dir requires --bundle".to_string());
    }
    if !bundle_entry_paths.is_empty() && split_out_dir.is_none() {
        return Err("--bundle-entry requires --split-out-dir".to_string());
    }
    if source_map && !json {
        return Err("--source-map requires --json".to_string());
    }
    if !input_source_maps.is_empty() && !source_map {
        return Err("--input-source-map requires --source-map".to_string());
    }

    let source = read_source(&path)?;
    let style_path = path_string(&path);
    let input_source_maps = read_input_source_maps(&input_source_maps, &style_path)?;
    let bundle_entry_style_paths = bundle_entry_paths
        .iter()
        .map(|entry_path| path_string(entry_path))
        .collect::<Vec<_>>();
    let mut workspace_source_paths = source_paths.clone();
    for entry_path in &bundle_entry_paths {
        if entry_path != &path && !workspace_source_paths.contains(entry_path) {
            workspace_source_paths.push(entry_path.clone());
        }
    }
    let mut context = read_context_json(context_json.as_deref())?;
    if tree_shake {
        append_tree_shake_build_passes(&mut pass_ids);
    }
    if bundle {
        append_bundle_build_passes(&mut pass_ids, &style_path, &source);
    }
    if minify {
        append_minify_build_passes(&mut pass_ids);
    }
    let used_engine_input = engine_input_json.is_some();
    if let Some(engine_input_path) = engine_input_json.as_deref() {
        let engine_input = read_engine_input_json(engine_input_path)?;
        let engine_context = summarize_omena_query_transform_context_from_engine_input(
            &engine_input,
            &style_path,
            closed_style_world || tree_shake,
        )
        .context;
        context = merge_cli_transform_context(context, &engine_context);
    }
    let original_workspace_sources =
        read_workspace_sources(&path, &source, &workspace_source_paths)?;
    let (workspace_sources, bundle_asset_url_rewrites) = if bundle {
        rewrite_bundle_asset_urls_for_build_sources(&original_workspace_sources)
    } else {
        (original_workspace_sources.clone(), Vec::new())
    };
    let bundle_asset_url_rewrite_count = bundle_asset_url_rewrites
        .iter()
        .map(|rewrite| rewrite.rewrite_count)
        .sum::<usize>();
    let mut split_transform_pass_ids = Vec::new();
    if tree_shake {
        append_tree_shake_build_passes(&mut split_transform_pass_ids);
    }
    let source_for_build = workspace_sources
        .iter()
        .find(|style_source| style_source.style_path == style_path)
        .map(|style_source| style_source.style_source.as_str())
        .unwrap_or(source.as_str());
    let package_manifests = read_package_manifests(&package_manifest_paths)?;
    let resolution_inputs = resolution_inputs_for_build_path(&path, package_manifests.as_slice());
    let bundle_artifact = if bundle {
        Some(run_omena_query_bundle(OmenaQueryBundlePlanInputV0 {
            target_style_path: &style_path,
            style_sources: &workspace_sources,
            source_map_sources: &original_workspace_sources,
            requested_pass_ids: &pass_ids,
            context: &context,
            resolution_inputs: &resolution_inputs,
            asset_rewrites: bundle_asset_url_rewrites.clone(),
            bundle_entry_style_paths: &bundle_entry_style_paths,
        })?)
    } else {
        None
    };
    let mut summary = if let Some(target_query) = target_query {
        if workspace_sources.len() > 1 {
            execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options_and_resolution_inputs(
                &style_path,
                &workspace_sources,
                &target_query,
                &context,
                target_options,
                &resolution_inputs,
            )?
        } else {
            execute_omena_query_consumer_build_style_source_for_target_query_with_context_and_options(
                &style_path,
                source_for_build,
                &target_query,
                &context,
                target_options,
            )
        }
    } else if workspace_sources.len() > 1 {
        execute_omena_query_consumer_build_style_sources_with_context_and_resolution_inputs(
            &style_path,
            &workspace_sources,
            &pass_ids,
            &context,
            &resolution_inputs,
        )?
    } else {
        execute_omena_query_consumer_build_style_source_with_context(
            &style_path,
            source_for_build,
            &pass_ids,
            &context,
        )
    };
    if used_engine_input {
        push_ready_surface(
            &mut summary.ready_surfaces,
            "semanticReachabilityTransformContext",
        );
        push_ready_surface(
            &mut summary.ready_surfaces,
            "expressionDomainSelectorProjection",
        );
    }
    if tree_shake {
        push_ready_surface(&mut summary.ready_surfaces, "treeShakeBuildMode");
    }
    if bundle {
        if let Some(artifact) = bundle_artifact.as_ref() {
            summary.bundle = Some(artifact.bundle.clone());
            push_ready_surface(&mut summary.ready_surfaces, "bundleAssetUrlResolution");
            if artifact.bundle.code_splitting_required {
                push_ready_surface(&mut summary.ready_surfaces, "bundleCodeSplitPlan");
            }
            push_ready_surface(&mut summary.ready_surfaces, "bundleOperationFacade");
        } else {
            attach_omena_query_consumer_build_bundle_summary(&mut summary, &source);
        }
        push_ready_surface(&mut summary.ready_surfaces, "bundleBuildMode");
        if bundle_asset_url_rewrite_count > 0 {
            push_ready_surface(&mut summary.ready_surfaces, "bundleAssetUrlRewrite");
        }
    }
    if source_map {
        if let Some(artifact) = bundle_artifact.as_ref() {
            summary.source_map_v3 = Some(artifact.source_map_v3.clone());
            push_ready_surface(&mut summary.ready_surfaces, "sourceMapV3Serializer");
            if artifact.source_map_v3.sources.len() > 1 {
                push_ready_surface(&mut summary.ready_surfaces, "bundleSourceMapOriginChain");
            }
        } else {
            attach_omena_query_consumer_build_source_map_v3_with_sources_and_resolution_inputs(
                &mut summary,
                &original_workspace_sources,
                &resolution_inputs,
            );
        }
        if compose_summary_source_map_with_input_source_maps(&mut summary, &input_source_maps) {
            push_ready_surface(
                &mut summary.ready_surfaces,
                "bundleUpstreamSourceMapComposition",
            );
        }
    }
    if let Some(split_out_dir) = split_out_dir.as_ref() {
        let split_emission = emit_bundle_code_split_outputs(BundleCodeSplitOutputOptions {
            out_dir: split_out_dir,
            entry_style_path: &style_path,
            sources: &workspace_sources,
            source_map_sources: &original_workspace_sources,
            resolution_inputs: &resolution_inputs,
            split_transform_pass_ids: &split_transform_pass_ids,
            input_source_maps: &input_source_maps,
            bundle_entry_style_paths: &bundle_entry_style_paths,
            context: &context,
            source_map,
        })?;
        if split_emission.upstream_maps_applied {
            push_ready_surface(
                &mut summary.ready_surfaces,
                "bundleUpstreamSourceMapComposition",
            );
        }
        push_ready_surface(&mut summary.ready_surfaces, "bundleCodeSplitEmission");
        push_ready_surface(
            &mut summary.ready_surfaces,
            "bundleCodeSplitManifestEmission",
        );
        push_ready_surface(
            &mut summary.ready_surfaces,
            "bundleCodeSplitBoundaryManifest",
        );
        if split_emission.configured_entry_count > 0 {
            push_ready_surface(&mut summary.ready_surfaces, "bundleCodeSplitEntryConfig");
        }
        if split_emission.shared_boundary_count > 0 {
            push_ready_surface(
                &mut summary.ready_surfaces,
                "bundleCodeSplitSharedChunkEmission",
            );
        }
        if tree_shake {
            push_ready_surface(
                &mut summary.ready_surfaces,
                "bundleCodeSplitTreeShakeEmission",
            );
        }
        if source_map {
            push_ready_surface(
                &mut summary.ready_surfaces,
                "bundleCodeSplitSourceMapEmission",
            );
        }
    }

    if !summary.unknown_pass_ids.is_empty() {
        return Err(format!(
            "unknown transform pass id: {}",
            summary.unknown_pass_ids.join(", ")
        ));
    }

    if let Some(output_path) = output {
        fs::write(&output_path, &summary.execution.output_css).map_err(|error| {
            format!(
                "failed to write transformed CSS to {}: {error}",
                path_string(&output_path)
            )
        })?;
    } else if !json {
        print!("{}", summary.execution.output_css);
    }

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    eprintln!(
        "executed passes: {}",
        summary.execution.executed_pass_ids.join(", ")
    );
    eprintln!(
        "planned-only passes: {}",
        summary.execution.planned_only_pass_ids.join(", ")
    );
    eprintln!("mutations: {}", summary.execution.mutation_count);
    Ok(())
}

fn read_input_source_maps(
    source_map_specs: &[String],
    entry_style_path: &str,
) -> Result<BTreeMap<String, String>, String> {
    let mut source_maps = BTreeMap::new();
    for spec in source_map_specs {
        let (style_path, source_map_path) = if let Some((style_path, source_map_path)) =
            spec.split_once('=')
        {
            if style_path.is_empty() || source_map_path.is_empty() {
                return Err(
                    "--input-source-map expects STYLE=MAP or MAP for the entry file".to_string(),
                );
            }
            (style_path.to_string(), PathBuf::from(source_map_path))
        } else {
            (entry_style_path.to_string(), PathBuf::from(spec))
        };
        let source_map_json = fs::read_to_string(&source_map_path).map_err(|error| {
            format!(
                "failed to read input source map {}: {error}",
                path_string(&source_map_path)
            )
        })?;
        source_maps.insert(style_path, source_map_json);
    }
    Ok(source_maps)
}

fn compose_summary_source_map_with_input_source_maps(
    summary: &mut OmenaQueryConsumerBuildSummaryV0,
    input_source_maps: &BTreeMap<String, String>,
) -> bool {
    let Some(source_map) = summary.source_map_v3.take() else {
        return false;
    };
    let (source_map, upstream_map_applied) =
        compose_source_map_with_input_source_maps(source_map, input_source_maps);
    summary.source_map_v3 = Some(source_map);
    upstream_map_applied
}

fn compose_source_map_with_input_source_maps(
    source_map: OmenaQueryTransformSourceMapV3V0,
    input_source_maps: &BTreeMap<String, String>,
) -> (OmenaQueryTransformSourceMapV3V0, bool) {
    let mut source_map = source_map;
    let mut upstream_map_applied = false;
    let source_paths = source_map.sources.clone();
    for source_path in source_paths {
        let Some(input_source_map) = input_source_maps.get(&source_path) else {
            continue;
        };
        let composition = compose_omena_query_transform_source_map_v3_with_upstream_map(
            &source_map,
            source_path.as_str(),
            input_source_map,
        );
        if composition.upstream_map_applied {
            source_map = composition.source_map;
            upstream_map_applied = true;
        }
    }
    (source_map, upstream_map_applied)
}

fn rewrite_bundle_asset_urls_for_build_sources(
    sources: &[OmenaQueryStyleSourceInputV0],
) -> (
    Vec<OmenaQueryStyleSourceInputV0>,
    Vec<OmenaQueryTransformBundleAssetUrlRewriteSummaryV0>,
) {
    let mut rewrites = Vec::new();
    let rewritten_sources = sources
        .iter()
        .map(|source| {
            let rewrite = rewrite_omena_transform_bundle_asset_urls_in_source(
                source.style_path.as_str(),
                source.style_source.as_str(),
            );
            let output_css = rewrite.output_css.clone();
            rewrites.push(rewrite);
            OmenaQueryStyleSourceInputV0 {
                style_path: source.style_path.clone(),
                style_source: output_css,
            }
        })
        .collect::<Vec<_>>();
    (rewritten_sources, rewrites)
}

fn resolution_inputs_for_build_path(
    path: &Path,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> OmenaQueryStyleResolutionInputsV0 {
    let workspace_folder_uri = style_resolution_workspace_uri_for_path(path);
    load_omena_query_workspace_style_resolution_inputs(
        workspace_folder_uri.as_deref(),
        package_manifests,
    )
}

struct BundleCodeSplitOutputOptions<'a> {
    out_dir: &'a Path,
    entry_style_path: &'a str,
    sources: &'a [OmenaQueryStyleSourceInputV0],
    source_map_sources: &'a [OmenaQueryStyleSourceInputV0],
    resolution_inputs: &'a OmenaQueryStyleResolutionInputsV0,
    split_transform_pass_ids: &'a [String],
    input_source_maps: &'a BTreeMap<String, String>,
    bundle_entry_style_paths: &'a [String],
    context: &'a OmenaQueryTransformExecutionContextV0,
    source_map: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BundleCodeSplitEmissionSummaryV0 {
    upstream_maps_applied: bool,
    configured_entry_count: usize,
    shared_boundary_count: usize,
}

pub(crate) const BUNDLE_CODE_SPLIT_MANIFEST_FILE_NAME: &str = "omena.bundle-split.manifest.json";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BundleCodeSplitManifestV0 {
    schema_version: u8,
    product: &'static str,
    entry_style_path: String,
    entry_file: String,
    output_count: usize,
    outputs: Vec<BundleCodeSplitManifestOutputV0>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BundleCodeSplitManifestOutputV0 {
    source_path: String,
    file_name: String,
    is_entry: bool,
    split_boundary: &'static str,
    source_map_file: Option<String>,
    imports: Vec<BundleCodeSplitManifestImportV0>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BundleCodeSplitManifestImportV0 {
    import_source: String,
    resolved_style_path: String,
    file_name: String,
}

fn emit_bundle_code_split_outputs(
    options: BundleCodeSplitOutputOptions<'_>,
) -> Result<BundleCodeSplitEmissionSummaryV0, String> {
    let BundleCodeSplitOutputOptions {
        out_dir,
        entry_style_path,
        sources,
        source_map_sources,
        resolution_inputs,
        split_transform_pass_ids,
        input_source_maps,
        bundle_entry_style_paths,
        context,
        source_map,
    } = options;

    fs::create_dir_all(out_dir).map_err(|error| {
        format!(
            "failed to create bundle split output directory {}: {error}",
            path_string(out_dir)
        )
    })?;
    let source_by_path = sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<BTreeMap<_, _>>();
    let source_map_source_by_path = source_map_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<BTreeMap<_, _>>();
    let file_name_by_path = sources
        .iter()
        .map(|source| {
            (
                source.style_path.as_str(),
                bundle_split_file_name(source.style_path.as_str()),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let source_path_lookup = bundle_split_source_path_lookup(sources);
    let workspace_split_plan = summarize_omena_query_bundle_code_split_workspace_plan(
        entry_style_path,
        bundle_entry_style_paths,
        sources,
        resolution_inputs,
    )?;
    let split_boundary_by_path = workspace_split_plan
        .outputs
        .iter()
        .map(|output| (output.source_path.as_str(), output.split_boundary))
        .collect::<BTreeMap<_, _>>();
    let entry_style_path_set = workspace_split_plan
        .outputs
        .iter()
        .filter(|output| output.is_entry)
        .map(|output| output.source_path.clone())
        .collect::<BTreeSet<_>>();
    let reachable_paths = workspace_split_plan
        .outputs
        .iter()
        .map(|output| output.source_path.clone())
        .collect::<Vec<_>>();

    let mut manifest_outputs = Vec::new();
    let mut upstream_maps_applied = false;
    for style_path in reachable_paths {
        let Some(source) = source_by_path.get(style_path.as_str()) else {
            continue;
        };
        let Some(file_name) = file_name_by_path.get(style_path.as_str()) else {
            continue;
        };
        let manifest_imports = bundle_code_split_manifest_imports_for_source(
            style_path.as_str(),
            source,
            &file_name_by_path,
            resolution_inputs,
            &source_path_lookup,
        );
        let mut output_css = source.to_string();
        let source_map_file = source_map.then(|| format!("{file_name}.map"));
        if !split_transform_pass_ids.is_empty() {
            let split_summary =
                execute_omena_query_consumer_build_style_sources_with_context_and_resolution_inputs(
                style_path.as_str(),
                sources,
                split_transform_pass_ids,
                context,
                resolution_inputs,
            )?;
            if !split_summary.unknown_pass_ids.is_empty() {
                return Err(format!(
                    "unknown transform pass id for bundle split output {}: {}",
                    style_path,
                    split_summary.unknown_pass_ids.join(", ")
                ));
            }
            output_css = split_summary.execution.output_css;
        }
        output_css = rewrite_bundle_code_split_imports_for_source(
            style_path.as_str(),
            output_css.as_str(),
            &file_name_by_path,
            resolution_inputs,
            &source_path_lookup,
        );
        if let Some(map_file_name) = source_map_file.as_deref() {
            let source_map_source = source_map_source_by_path
                .get(style_path.as_str())
                .copied()
                .unwrap_or(source);
            let source_map_v3 = summarize_omena_query_bundle_code_split_source_map_v3(
                file_name,
                output_css.as_str(),
                style_path.as_str(),
                source_map_source,
            );
            let (source_map_v3, upstream_map_applied) =
                compose_source_map_with_input_source_maps(source_map_v3, input_source_maps);
            upstream_maps_applied = upstream_maps_applied || upstream_map_applied;
            let map_output_path = out_dir.join(map_file_name);
            let source_map_json = serde_json::to_string_pretty(&source_map_v3)
                .map_err(|error| format!("failed to serialize split source map: {error}"))?;
            fs::write(&map_output_path, source_map_json).map_err(|error| {
                format!(
                    "failed to write bundle split source map {}: {error}",
                    path_string(&map_output_path)
                )
            })?;
            output_css.push_str("\n/*# sourceMappingURL=");
            output_css.push_str(map_file_name);
            output_css.push_str(" */\n");
        }
        let output_path = out_dir.join(file_name);
        fs::write(&output_path, output_css).map_err(|error| {
            format!(
                "failed to write bundle split output {}: {error}",
                path_string(&output_path)
            )
        })?;
        let split_boundary = split_boundary_by_path
            .get(style_path.as_str())
            .copied()
            .unwrap_or("styleDependency");
        manifest_outputs.push(BundleCodeSplitManifestOutputV0 {
            source_path: style_path.clone(),
            file_name: file_name.clone(),
            is_entry: entry_style_path_set.contains(style_path.as_str()),
            split_boundary,
            source_map_file,
            imports: manifest_imports,
        });
    }
    let entry_file = file_name_by_path
        .get(entry_style_path)
        .cloned()
        .unwrap_or_else(|| bundle_split_file_name(entry_style_path));
    let manifest = BundleCodeSplitManifestV0 {
        schema_version: 0,
        product: "omena-cli.bundle-code-split-manifest",
        entry_style_path: entry_style_path.to_string(),
        entry_file,
        output_count: manifest_outputs.len(),
        outputs: manifest_outputs,
    };
    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|error| format!("failed to serialize bundle split manifest: {error}"))?;
    let manifest_path = out_dir.join(BUNDLE_CODE_SPLIT_MANIFEST_FILE_NAME);
    fs::write(&manifest_path, manifest_json).map_err(|error| {
        format!(
            "failed to write bundle split manifest {}: {error}",
            path_string(&manifest_path)
        )
    })?;
    Ok(BundleCodeSplitEmissionSummaryV0 {
        upstream_maps_applied,
        configured_entry_count: workspace_split_plan.configured_entry_count,
        shared_boundary_count: workspace_split_plan.shared_boundary_count,
    })
}

fn rewrite_bundle_code_split_imports_for_source(
    style_path: &str,
    source: &str,
    file_name_by_path: &BTreeMap<&str, String>,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    source_path_lookup: &BTreeMap<String, String>,
) -> String {
    let bundle = summarize_omena_transform_bundle_from_source(
        style_path,
        source,
        infer_cli_style_dialect(style_path),
    );
    let mut output = source.to_string();
    for edge in bundle.bundle_edges.iter().rev() {
        if !matches!(
            edge.kind,
            TransformBundleEdgeKind::CssImport | TransformBundleEdgeKind::LessImport
        ) {
            continue;
        }
        let Some(import_source) = edge.import_source.as_deref() else {
            continue;
        };
        let Some(target_path) =
            resolve_bundle_code_split_import_path(style_path, import_source, resolution_inputs)
        else {
            continue;
        };
        let Some(source_path) = source_path_lookup.get(target_path.as_str()) else {
            continue;
        };
        let Some(target_file_name) = file_name_by_path.get(source_path.as_str()) else {
            continue;
        };
        let range_start = edge.range_start as usize;
        let range_end = edge.range_end as usize;
        if range_start > range_end || range_end > output.len() {
            continue;
        }
        let rule_text = &output[range_start..range_end];
        let Some(relative_source_start) = rule_text.find(import_source) else {
            continue;
        };
        let source_start = range_start + relative_source_start;
        let source_end = source_start + import_source.len();
        output.replace_range(source_start..source_end, target_file_name);
    }
    output
}

fn bundle_code_split_manifest_imports_for_source(
    style_path: &str,
    source: &str,
    file_name_by_path: &BTreeMap<&str, String>,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    source_path_lookup: &BTreeMap<String, String>,
) -> Vec<BundleCodeSplitManifestImportV0> {
    let bundle = summarize_omena_transform_bundle_from_source(
        style_path,
        source,
        infer_cli_style_dialect(style_path),
    );
    let mut imports = Vec::new();
    for edge in bundle.bundle_edges {
        if !matches!(
            edge.kind,
            TransformBundleEdgeKind::CssImport | TransformBundleEdgeKind::LessImport
        ) {
            continue;
        }
        let Some(import_source) = edge.import_source else {
            continue;
        };
        let Some(target_path) =
            resolve_bundle_code_split_import_path(style_path, &import_source, resolution_inputs)
        else {
            continue;
        };
        let Some(source_path) = source_path_lookup.get(target_path.as_str()) else {
            continue;
        };
        let Some(file_name) = file_name_by_path.get(source_path.as_str()) else {
            continue;
        };
        imports.push(BundleCodeSplitManifestImportV0 {
            import_source,
            resolved_style_path: source_path.clone(),
            file_name: file_name.clone(),
        });
    }
    imports
}

fn bundle_split_source_path_lookup(
    sources: &[OmenaQueryStyleSourceInputV0],
) -> BTreeMap<String, String> {
    let mut lookup = BTreeMap::new();
    for source in sources {
        lookup.insert(source.style_path.clone(), source.style_path.clone());
        if let Ok(canonical_path) = fs::canonicalize(&source.style_path) {
            lookup.insert(path_string(&canonical_path), source.style_path.clone());
        }
    }
    lookup
}

fn resolve_bundle_code_split_import_path(
    style_path: &str,
    import_source: &str,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Option<String> {
    let base_uri = cli_path_to_file_uri(Path::new(style_path));
    let workspace_folder_uri = Path::new(style_path).parent().map(cli_path_to_file_uri);
    let resolved_uri = resolve_omena_query_style_uri_for_specifier_with_resolution_inputs(
        base_uri.as_str(),
        workspace_folder_uri.as_deref(),
        import_source,
        resolution_inputs,
    )?;
    cli_file_uri_to_path(resolved_uri.as_str()).map(|path| path_string(&path))
}

pub(crate) fn bundle_split_file_name(style_path: &str) -> String {
    let path = Path::new(style_path);
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("css");
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("chunk");
    let mut sanitized = String::new();
    for ch in stem.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            sanitized.push(ch);
        } else {
            sanitized.push('-');
        }
    }
    if sanitized.is_empty() {
        sanitized.push_str("chunk");
    }
    let hash = bundle_split_path_hash(style_path);
    format!("{sanitized}-{hash:016x}.{extension}")
}

fn bundle_split_path_hash(value: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub(crate) fn list_passes(json: bool) -> Result<(), String> {
    let passes = list_omena_query_transform_pass_summaries();

    if json {
        print_json(&passes)?;
        return Ok(());
    }

    for pass in passes {
        println!("{}\t{}", pass.id, pass.title);
    }
    Ok(())
}

fn merge_cli_transform_context(
    mut base: OmenaQueryTransformExecutionContextV0,
    additional: &OmenaQueryTransformExecutionContextV0,
) -> OmenaQueryTransformExecutionContextV0 {
    base.drop_dark_mode_media_queries =
        base.drop_dark_mode_media_queries || additional.drop_dark_mode_media_queries;
    merge_cli_context_list(
        &mut base.reachable_class_names,
        &additional.reachable_class_names,
    );
    merge_cli_context_list(
        &mut base.reachable_keyframe_names,
        &additional.reachable_keyframe_names,
    );
    merge_cli_context_list(
        &mut base.reachable_value_names,
        &additional.reachable_value_names,
    );
    merge_cli_context_list(
        &mut base.reachable_custom_property_names,
        &additional.reachable_custom_property_names,
    );
    base
}

fn append_tree_shake_build_passes(pass_ids: &mut Vec<String>) {
    for pass_id in [
        "tree-shake-class",
        "tree-shake-keyframes",
        "tree-shake-value",
        "tree-shake-custom-property",
    ] {
        if !pass_ids.iter().any(|existing| existing == pass_id) {
            pass_ids.push(pass_id.to_string());
        }
    }
}

fn append_bundle_build_passes(pass_ids: &mut Vec<String>, style_path: &str, source: &str) {
    let bundle = summarize_omena_transform_bundle_from_source(
        style_path,
        source,
        infer_cli_style_dialect(style_path),
    );
    for pass_id in bundle.planned_pass_ids {
        if !pass_ids.iter().any(|existing| existing == pass_id) {
            pass_ids.push(pass_id.to_string());
        }
    }
}

fn append_minify_build_passes(pass_ids: &mut Vec<String>) {
    for pass_id in [
        "comment-strip",
        "whitespace-strip",
        "number-compression",
        "color-compression",
        "shorthand-combining",
        "rule-deduplication",
        "rule-merging",
        "selector-merging",
        "empty-rule-removal",
        "calc-reduction",
        "print-css",
    ] {
        if !pass_ids.iter().any(|existing| existing == pass_id) {
            pass_ids.push(pass_id.to_string());
        }
    }
}

fn infer_cli_style_dialect(style_path: &str) -> OmenaParserStyleDialect {
    match Path::new(style_path)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .as_deref()
    {
        Some("scss") => OmenaParserStyleDialect::Scss,
        Some("sass") => OmenaParserStyleDialect::Sass,
        Some("less") => OmenaParserStyleDialect::Less,
        _ => OmenaParserStyleDialect::Css,
    }
}

fn push_ready_surface(surfaces: &mut Vec<&'static str>, surface: &'static str) {
    if !surfaces.contains(&surface) {
        surfaces.push(surface);
    }
}

fn merge_cli_context_list(target: &mut Vec<String>, additional: &[String]) {
    for item in additional {
        if !target.contains(item) {
            target.push(item.clone());
        }
    }
    target.sort();
}
