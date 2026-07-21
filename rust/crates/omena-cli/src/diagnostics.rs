use crate::{
    io::{
        read_package_manifests, read_source, read_source_diagnostic_candidates_json,
        read_source_documents, read_style_sources, read_workspace_sources,
    },
    lock::resolve_lock_relative_path,
    output::{CliOutputMetadataV0, print_json},
    paths::{path_string, style_resolution_workspace_uri_for_path},
};
use omena_query::{
    OmenaQueryDynamicClassnameMTierInputV0, OmenaQueryExternalModuleModeV0,
    OmenaQueryExternalSifInputV0, OmenaQuerySourceDiagnosticsForFileV0,
    OmenaQueryStyleDiagnosticV0, OmenaQueryStyleDiagnosticsForFileV0, OmenaQueryStyleMemoHostV0,
    OmenaQueryStyleResolutionInputsV0, OmenaQueryStyleSourceInputV0, ParserRangeV0,
    load_omena_query_workspace_style_resolution_inputs,
    resolve_omena_query_bridge_external_sifs_for_style_sources,
    summarize_omena_query_dynamic_classname_m_tier_diagnostics_with_context_depth,
    summarize_omena_query_source_diagnostics_for_file,
    summarize_omena_query_source_diagnostics_for_workspace_file_with_resolution_inputs,
    summarize_omena_query_style_diagnostics_for_file_with_local_composes_and_deep_analysis,
    summarize_omena_query_style_hover_candidates,
    summarize_omena_query_unified_cross_file_hypergraph,
};
#[cfg(test)]
use omena_query::{
    OmenaQuerySourceDocumentInputV0, OmenaQueryStylePackageManifestV0,
    summarize_omena_query_workspace_cross_file_summary_with_resolution_inputs,
};
use omena_sif::{read_omena_lock_json_v1, read_omena_sif_json_v1};
use omena_streaming_ifds::summarize_streaming_ifds_cross_file_reachability_v0;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn style_diagnostics(
    path: PathBuf,
    source_paths: Vec<PathBuf>,
    source_document_paths: Vec<PathBuf>,
    package_manifest_paths: Vec<PathBuf>,
    sif_paths: Vec<PathBuf>,
    lockfile: Option<PathBuf>,
    external: Option<String>,
    deep_analysis: bool,
    json: bool,
) -> Result<(), String> {
    let summary = style_diagnostics_summary(
        path,
        source_paths,
        source_document_paths,
        package_manifest_paths,
        sif_paths,
        lockfile,
        external,
        deep_analysis,
    )?;

    if json {
        print_json(
            CliOutputMetadataV0::new("omena-cli.style-diagnostics"),
            &summary,
        )?;
        return Ok(());
    }

    println!("file: {}", summary.file_uri);
    println!("diagnostics: {}", summary.diagnostic_count);
    for diagnostic in &summary.diagnostics {
        println!("{}\t{}", diagnostic.code, diagnostic.message);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn style_diagnostics_summary(
    path: PathBuf,
    source_paths: Vec<PathBuf>,
    source_document_paths: Vec<PathBuf>,
    package_manifest_paths: Vec<PathBuf>,
    sif_paths: Vec<PathBuf>,
    lockfile: Option<PathBuf>,
    external: Option<String>,
    deep_analysis: bool,
) -> Result<OmenaQueryStyleDiagnosticsForFileV0, String> {
    let source = read_source(&path)?;
    let style_path = path_string(&path);
    let package_manifests = read_package_manifests(&package_manifest_paths)?;
    let resolved_lockfile = lockfile.or_else(|| discover_omena_lockfile_for_path(&path));
    let external_mode = resolve_external_module_mode_for_style_diagnostics(
        external.as_deref(),
        &resolved_lockfile,
    )?;
    let uses_external_sif_path = external_mode == OmenaQueryExternalModuleModeV0::Sif;
    if source_paths.is_empty()
        && source_document_paths.is_empty()
        && package_manifests.is_empty()
        && sif_paths.is_empty()
        && !uses_external_sif_path
    {
        let Some(candidates) = summarize_omena_query_style_hover_candidates(&style_path, &source)
        else {
            return Err(format!(
                "failed to read style candidates for {}",
                path_string(&path)
            ));
        };
        Ok(
            summarize_omena_query_style_diagnostics_for_file_with_local_composes_and_deep_analysis(
                &style_path,
                &source,
                candidates.candidates.as_slice(),
                deep_analysis,
            ),
        )
    } else {
        let workspace_sources = read_workspace_sources(&path, &source, &source_paths)?;
        let source_documents = read_source_documents(&source_document_paths)?;
        let mut external_sifs = read_external_sifs(&sif_paths)?;
        let mut lockfile_diagnostics = Vec::new();
        if uses_external_sif_path && let Some(lockfile) = resolved_lockfile.as_ref() {
            match read_lock_external_sifs(lockfile) {
                Ok(lock_sifs) => external_sifs.extend(lock_sifs),
                Err(error) => lockfile_diagnostics
                    .push(lockfile_invalid_style_diagnostic(lockfile, error.as_str())),
            }
        }
        // #33: an `@use "file:///…"` edge now routes through the external-SIF branch
        // (resolver `is_external_style_module_source`). Generate the bridge SIF for each such
        // on-disk external edge in-process so an external `missingSassSymbol` is suppressed
        // without a manual `--sif`. Edges already covered by an explicit `--sif` (matching
        // canonical URL) keep the user-provided artifact; unreadable edges (a genuinely-missing
        // module, or a `http(s)://`/`sass:` scheme the bridge cannot read) are skipped so they
        // still surface their boundary state.
        let workspace_folder_uri = style_resolution_workspace_uri_for_path(&path);
        let resolution_inputs = load_omena_query_workspace_style_resolution_inputs(
            workspace_folder_uri.as_deref(),
            package_manifests.as_slice(),
        );
        let in_process_external_sifs = resolve_in_process_external_sifs(
            workspace_sources.as_slice(),
            external_sifs.as_slice(),
            &resolution_inputs,
        );
        external_sifs.extend(in_process_external_sifs);
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let mut summary = if let Some(selector) = host.workspace_revision_selector(
            workspace_sources.as_slice(),
            source_documents.as_slice(),
            package_manifests.as_slice(),
            external_sifs.as_slice(),
            &resolution_inputs,
        ) {
            let mut summary = selector
                .workspace_style_diagnostics_with_external_mode(&style_path, external_mode)
                .ok_or_else(|| {
                    format!("failed to read committed workspace style diagnostics for {style_path}")
                })?;
            summary.diagnostics.extend(
                summarize_cross_file_streaming_reachability_diagnostics_from_summary(
                    &style_path,
                    selector.workspace_cross_file_summary(),
                ),
            );
            summary
        } else {
            return Err(format!(
                "failed to commit workspace style diagnostics for {style_path}"
            ));
        };
        summary.diagnostics.extend(lockfile_diagnostics);
        summary.diagnostic_count = summary.diagnostics.len();
        Ok(summary)
    }
}

pub(crate) fn workspace_style_diagnostics_summaries(
    style_paths: &[PathBuf],
    source_document_paths: &[PathBuf],
    package_manifest_paths: &[PathBuf],
) -> Result<Vec<OmenaQueryStyleDiagnosticsForFileV0>, String> {
    let Some(first_style_path) = style_paths.first() else {
        return Ok(Vec::new());
    };
    let workspace_sources = read_style_sources(style_paths)?;
    let source_documents = read_source_documents(source_document_paths)?;
    let package_manifests = read_package_manifests(package_manifest_paths)?;
    let resolved_lockfile = discover_omena_lockfile_for_path(first_style_path);
    let mut lockfile_diagnostics = Vec::new();
    let mut external_sifs = if let Some(lockfile) = resolved_lockfile.as_ref() {
        match read_lock_external_sifs(lockfile) {
            Ok(sifs) => sifs,
            Err(error) => {
                lockfile_diagnostics
                    .push(lockfile_invalid_style_diagnostic(lockfile, error.as_str()));
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };
    let workspace_folder_uri = style_resolution_workspace_uri_for_path(first_style_path);
    let resolution_inputs = load_omena_query_workspace_style_resolution_inputs(
        workspace_folder_uri.as_deref(),
        package_manifests.as_slice(),
    );
    external_sifs.extend(resolve_in_process_external_sifs(
        workspace_sources.as_slice(),
        external_sifs.as_slice(),
        &resolution_inputs,
    ));

    let mut host = OmenaQueryStyleMemoHostV0::new();
    let selector = host
        .workspace_revision_selector(
            workspace_sources.as_slice(),
            source_documents.as_slice(),
            package_manifests.as_slice(),
            external_sifs.as_slice(),
            &resolution_inputs,
        )
        .ok_or_else(|| "failed to commit workspace lint diagnostics".to_string())?;
    let cross_file_summary = selector.workspace_cross_file_summary();
    let mut summaries = Vec::with_capacity(style_paths.len());
    for style_path in style_paths {
        let style_path = path_string(style_path);
        let mut summary = selector
            .workspace_style_diagnostics_with_external_mode(
                style_path.as_str(),
                OmenaQueryExternalModuleModeV0::Sif,
            )
            .ok_or_else(|| format!("failed to read committed lint diagnostics for {style_path}"))?;
        summary.diagnostics.extend(
            summarize_cross_file_streaming_reachability_diagnostics_from_summary(
                style_path.as_str(),
                cross_file_summary,
            ),
        );
        summary.diagnostics.extend(lockfile_diagnostics.clone());
        summary.diagnostic_count = summary.diagnostics.len();
        summaries.push(summary);
    }
    Ok(summaries)
}

/// Surface a real cross-file dataflow reachability fact through the product diagnostics.
///
/// The streaming-IFDS crate projects the resolved workspace cross-file summary
/// to the unified hypergraph — the SAME real `composes`/`@use`/`@forward`/
/// `@import`/value/icss/foreign-reference edges the analyzer already resolves.
/// It then owns the exact propagation report: every node owned by the target
/// file is seeded and foreign module paths are reached by facts over those
/// edges. A self-contained file has no foreign reachable path and no diagnostic
/// is emitted. No synthetic hyperedges are fed in.
#[cfg(test)]
pub(crate) fn summarize_cross_file_streaming_reachability_diagnostics(
    target_style_path: &str,
    workspace_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let workspace_folder_uri =
        style_resolution_workspace_uri_for_path(Path::new(target_style_path));
    let resolution_inputs = load_omena_query_workspace_style_resolution_inputs(
        workspace_folder_uri.as_deref(),
        package_manifests,
    );
    let summary = summarize_omena_query_workspace_cross_file_summary_with_resolution_inputs(
        workspace_sources,
        source_documents,
        package_manifests,
        &resolution_inputs,
    );
    summarize_cross_file_streaming_reachability_diagnostics_from_summary(
        target_style_path,
        &summary,
    )
}

pub(crate) fn summarize_cross_file_streaming_reachability_diagnostics_from_summary(
    target_style_path: &str,
    summary: &omena_query::OmenaQueryCrossFileSummaryV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let hypergraph = summarize_omena_query_unified_cross_file_hypergraph(summary);
    let report = summarize_streaming_ifds_cross_file_reachability_v0(
        target_style_path,
        hypergraph.hyperedges.as_slice(),
    );
    if report.reachable_foreign_paths.is_empty() {
        return Vec::new();
    }

    let reachable_modules = report.reachable_foreign_paths.join(", ");
    vec![OmenaQueryStyleDiagnosticV0 {
        code: "crossFileStreamingReachability",
        severity: "hint",
        provenance: vec![
            "omena-streaming-ifds.cross-file-reachability-report",
            "omena-streaming-ifds.analysis-report",
            "omena-query.unified-cross-file-hypergraph",
            "omena-query.cross-file-summary",
        ],
        range: ParserRangeV0::default(),
        message: format!(
            "cross-file dataflow reaches {} module(s) via resolved edges: {reachable_modules}",
            report.reachable_foreign_path_count
        ),
        tags: Vec::new(),
        create_custom_property: None,
        cascade_narrowing: None,
        cascade_confidence: None,
        polynomial_provenance: None,
        cross_file_scc: None,
    }]
}

fn lockfile_invalid_style_diagnostic(
    lockfile: &Path,
    message: &str,
) -> OmenaQueryStyleDiagnosticV0 {
    OmenaQueryStyleDiagnosticV0 {
        code: "lockfileInvalid",
        severity: "error",
        provenance: vec![
            "omena-cli.lockfile-loader",
            "omena-query.external-sif-boundary-diagnostics",
        ],
        range: ParserRangeV0::default(),
        message: format!(
            "Failed to load {} for external SIF diagnostics: {message}",
            path_string(lockfile)
        ),
        tags: Vec::new(),
        create_custom_property: None,
        cascade_narrowing: None,
        cascade_confidence: None,
        polynomial_provenance: None,
        cross_file_scc: None,
    }
}

pub(crate) fn read_external_sifs(
    paths: &[PathBuf],
) -> Result<Vec<OmenaQueryExternalSifInputV0>, String> {
    paths
        .iter()
        .map(|path| {
            let sif_json = read_source(path)?;
            let sif = read_omena_sif_json_v1(&sif_json)
                .map_err(|error| format!("failed to parse SIF {}: {error}", path_string(path)))?;
            Ok(OmenaQueryExternalSifInputV0 {
                canonical_url: sif.canonical_url.clone(),
                sif,
            })
        })
        .collect()
}

pub(crate) fn read_lock_external_sifs(
    lockfile: &Path,
) -> Result<Vec<OmenaQueryExternalSifInputV0>, String> {
    let lockfile_source = read_source(lockfile)?;
    let lock = read_omena_lock_json_v1(&lockfile_source)
        .map_err(|error| format!("failed to parse {}: {error}", path_string(lockfile)))?;
    lock.entries
        .iter()
        .map(|entry| {
            let sif_path = resolve_lock_relative_path(lockfile, &entry.sif_path);
            let sif_json = read_source(&sif_path)?;
            let sif = read_omena_sif_json_v1(&sif_json).map_err(|error| {
                format!("failed to parse SIF {}: {error}", path_string(&sif_path))
            })?;
            if sif.canonical_url != entry.canonical_url {
                return Err(format!(
                    "lock entry {} points to SIF {} with canonicalUrl {}",
                    entry.canonical_url,
                    path_string(&sif_path),
                    sif.canonical_url
                ));
            }
            Ok(OmenaQueryExternalSifInputV0 {
                canonical_url: entry.canonical_url.clone(),
                sif,
            })
        })
        .collect()
}

/// Generate, in-process, the external SIFs for readable external Sass module
/// edges in the workspace — both literal `file://` edges and package
/// specifiers that resolve to disk — plus every reachable `@forward` hop.
///
/// The bridge reads the resolved on-disk module and produces an
/// [`omena_sif::OmenaSifV1`]. The resulting
/// `OmenaQueryExternalSifInputV0.canonical_url` keeps the verbatim edge source
/// for package aliases, while the inner SIF carries the normalized file URL.
///
/// After generating a SIF the walk recurses into that SIF's `exports.forwards[].canonical_url`
/// (each a *raw* relative/bare specifier as written, e.g. `"./tokens"`), re-resolving every
/// forwarded specifier against the forwarding SIF's resolved inner `canonical_url` (a `file://`
/// URI) via the `omena_query` resolver facade and generating those modules' SIFs too. So a
/// transitively-forwarded module (A `@forward` B where B defines/re-exports the symbols) gets a
/// SIF generated even though no workspace source imports B directly. The walk is a breadth-first
/// worklist with cycle/diamond detection keyed on the *resolved* `file://` identity, so an
/// A↔B forward cycle terminates without hanging or duplicating SIFs.
///
/// Skipped, never fabricated:
/// - an edge already covered by an explicit `--sif` (matching canonical URL) — the user artifact
///   wins, so a stale/partial `--sif` is never silently overwritten by a fresh bridge SIF;
/// - a `file://` edge the bridge cannot read (a genuinely-missing module) — left out so it keeps
///   surfacing its `missingExternalSif`/`missingSassSymbol` boundary state (no over-correction);
/// - a forwarded specifier that does not resolve to an on-disk module (resolver returns `None`)
///   or that the bridge cannot read — left out so a genuinely-missing transitive forward still
///   flags;
/// - `http(s)://`/`sass:` schemes — not on-disk, so the bridge cannot read them in-process.
///
/// The query layer consumes the generated chain by flattening forwarded external-SIF exports when
/// it computes the visible Sass symbol set for the root external module.
pub(crate) fn resolve_in_process_external_sifs(
    workspace_sources: &[OmenaQueryStyleSourceInputV0],
    existing_external_sifs: &[OmenaQueryExternalSifInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Vec<OmenaQueryExternalSifInputV0> {
    resolve_omena_query_bridge_external_sifs_for_style_sources(
        workspace_sources,
        existing_external_sifs,
        resolution_inputs,
    )
    .external_sifs
}

pub(crate) fn parse_external_module_mode(
    external: &str,
) -> Result<OmenaQueryExternalModuleModeV0, String> {
    match external {
        "ignored" => Ok(OmenaQueryExternalModuleModeV0::Ignored),
        "sif" => Ok(OmenaQueryExternalModuleModeV0::Sif),
        _ => Err(format!(
            "unsupported external mode '{external}'; expected ignored or sif"
        )),
    }
}

fn resolve_external_module_mode_for_style_diagnostics(
    external: Option<&str>,
    _lockfile: &Option<PathBuf>,
) -> Result<OmenaQueryExternalModuleModeV0, String> {
    match external {
        Some(external) => parse_external_module_mode(external),
        None => Ok(OmenaQueryExternalModuleModeV0::Sif),
    }
}

fn discover_omena_lockfile_for_path(path: &Path) -> Option<PathBuf> {
    let mut current = path.parent();
    while let Some(directory) = current {
        let candidate = directory.join("omena.lock");
        if candidate.exists() {
            return Some(candidate);
        }
        current = directory.parent();
    }
    let cwd_candidate = PathBuf::from("omena.lock");
    cwd_candidate.exists().then_some(cwd_candidate)
}

pub(crate) fn source_diagnostics(
    source_uri: String,
    candidates_json: Option<PathBuf>,
    source_path: Option<PathBuf>,
    source_paths: Vec<PathBuf>,
    package_manifest_paths: Vec<PathBuf>,
    json: bool,
) -> Result<(), String> {
    let summary = source_diagnostics_summary(
        source_uri,
        candidates_json,
        source_path,
        source_paths,
        package_manifest_paths,
    )?;

    if json {
        print_json(
            CliOutputMetadataV0::new("omena-cli.source-diagnostics"),
            &summary,
        )?;
        return Ok(());
    }

    println!("file: {}", summary.file_uri);
    println!("diagnostics: {}", summary.diagnostic_count);
    for diagnostic in &summary.diagnostics {
        println!("{}\t{}", diagnostic.code, diagnostic.message);
    }
    Ok(())
}

pub(crate) fn dynamic_classname_diagnostics(input_json: PathBuf, json: bool) -> Result<(), String> {
    let summary = dynamic_classname_diagnostics_summary(&input_json)?;

    if json {
        print_json(
            CliOutputMetadataV0::new("omena-cli.dynamic-classname-diagnostics"),
            &summary,
        )?;
        return Ok(());
    }

    println!("file: {}", summary.file_uri);
    println!("diagnostics: {}", summary.diagnostic_count);
    for diagnostic in &summary.diagnostics {
        println!("{}\t{}", diagnostic.code, diagnostic.message);
    }
    Ok(())
}

pub(crate) fn source_diagnostics_summary(
    source_uri: String,
    candidates_json: Option<PathBuf>,
    source_path: Option<PathBuf>,
    source_paths: Vec<PathBuf>,
    package_manifest_paths: Vec<PathBuf>,
) -> Result<OmenaQuerySourceDiagnosticsForFileV0, String> {
    if let Some(candidates_json) = candidates_json {
        let candidates = read_source_diagnostic_candidates_json(&candidates_json)?;
        Ok(summarize_omena_query_source_diagnostics_for_file(
            source_uri.as_str(),
            candidates.as_slice(),
        ))
    } else {
        let source_path = source_path.ok_or_else(|| {
            "source-diagnostics requires either --candidates-json or --source-path".to_string()
        })?;
        let source_source = read_source(&source_path)?;
        let style_sources = read_style_sources(&source_paths)?;
        let package_manifests = read_package_manifests(&package_manifest_paths)?;
        let workspace_folder_uri = style_resolution_workspace_uri_for_path(&source_path);
        let resolution_inputs = load_omena_query_workspace_style_resolution_inputs(
            workspace_folder_uri.as_deref(),
            package_manifests.as_slice(),
        );
        Ok(
            summarize_omena_query_source_diagnostics_for_workspace_file_with_resolution_inputs(
                source_uri.as_str(),
                source_source.as_str(),
                style_sources.as_slice(),
                package_manifests.as_slice(),
                &resolution_inputs,
            ),
        )
    }
}

pub(crate) fn dynamic_classname_diagnostics_summary(
    input_json: &Path,
) -> Result<OmenaQuerySourceDiagnosticsForFileV0, String> {
    let json = fs::read_to_string(input_json).map_err(|error| {
        format!(
            "failed to read dynamic className diagnostics input JSON {}: {error}",
            path_string(input_json)
        )
    })?;
    let input: OmenaQueryDynamicClassnameMTierInputV0 =
        serde_json::from_str(&json).map_err(|error| {
            format!(
                "failed to parse dynamic className diagnostics input JSON {}: {error}",
                path_string(input_json)
            )
        })?;
    Ok(summarize_omena_query_dynamic_classname_m_tier_diagnostics_with_context_depth(&input))
}
