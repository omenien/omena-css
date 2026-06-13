use crate::{
    io::{
        read_package_manifests, read_source, read_source_diagnostic_candidates_json,
        read_source_documents, read_style_sources, read_workspace_sources,
    },
    lock::resolve_lock_relative_path,
    output::print_json,
    paths::{path_string, style_resolution_workspace_uri_for_path},
};
use omena_query::generate_omena_bridge_sif_for_resolved_style_path;
use omena_query::{
    OmenaQueryDynamicClassnameMTierInputV0, OmenaQueryExternalModuleModeV0,
    OmenaQueryExternalSifInputV0, OmenaQuerySourceDiagnosticsForFileV0,
    OmenaQuerySourceDocumentInputV0, OmenaQueryStyleDiagnosticV0, OmenaQueryStylePackageManifestV0,
    OmenaQueryStyleSourceInputV0, ParserRangeV0,
    load_omena_query_workspace_style_resolution_inputs,
    summarize_omena_query_dynamic_classname_m_tier_diagnostics_with_context_depth,
    summarize_omena_query_sass_module_sources, summarize_omena_query_source_diagnostics_for_file,
    summarize_omena_query_source_diagnostics_for_workspace_file,
    summarize_omena_query_style_diagnostics_for_file_with_local_composes_and_deep_analysis,
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs,
    summarize_omena_query_style_hover_candidates,
};
use omena_sif::{read_omena_lock_json_v1, read_omena_sif_json_v1};
use omena_streaming_ifds::summarize_streaming_ifds_workspace_cross_file_reachability_v0;
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
    let source = read_source(&path)?;
    let style_path = path_string(&path);
    let package_manifests = read_package_manifests(&package_manifest_paths)?;
    let resolved_lockfile = lockfile.or_else(|| discover_omena_lockfile_for_path(&path));
    let external_mode = resolve_external_module_mode_for_style_diagnostics(
        external.as_deref(),
        &resolved_lockfile,
    )?;
    let uses_external_sif_path = external_mode == OmenaQueryExternalModuleModeV0::Sif;
    let summary = if source_paths.is_empty()
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
        summarize_omena_query_style_diagnostics_for_file_with_local_composes_and_deep_analysis(
            &style_path,
            &source,
            candidates.candidates.as_slice(),
            deep_analysis,
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
        let in_process_external_sifs = resolve_in_process_external_sifs(
            workspace_sources.as_slice(),
            external_sifs.as_slice(),
        );
        external_sifs.extend(in_process_external_sifs);
        let workspace_folder_uri = style_resolution_workspace_uri_for_path(&path);
        let resolution_inputs = load_omena_query_workspace_style_resolution_inputs(
            workspace_folder_uri.as_deref(),
            package_manifests.as_slice(),
        );
        let mut summary =
            summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs(
                &style_path,
                workspace_sources.as_slice(),
                source_documents.as_slice(),
                package_manifests.as_slice(),
                None,
                external_mode,
                external_sifs.as_slice(),
                &resolution_inputs,
            )
            .ok_or_else(|| {
                format!("failed to read workspace style diagnostics for {style_path}")
            })?;
        // Drive the crate-owned streaming-IFDS cross-file reachability report from
        // the resolved cross-file hypergraph, not a synthetic harness.
        summary
            .diagnostics
            .extend(summarize_cross_file_streaming_reachability_diagnostics(
                &style_path,
                workspace_sources.as_slice(),
                source_documents.as_slice(),
                package_manifests.as_slice(),
            ));
        summary.diagnostics.extend(lockfile_diagnostics);
        summary.diagnostic_count = summary.diagnostics.len();
        summary
    };

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("file: {}", summary.file_uri);
    println!("diagnostics: {}", summary.diagnostic_count);
    for diagnostic in &summary.diagnostics {
        println!("{}\t{}", diagnostic.code, diagnostic.message);
    }
    Ok(())
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
pub(crate) fn summarize_cross_file_streaming_reachability_diagnostics(
    target_style_path: &str,
    workspace_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let report = summarize_streaming_ifds_workspace_cross_file_reachability_v0(
        target_style_path,
        workspace_sources,
        source_documents,
        package_manifests,
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

/// Generate, in-process, the external SIFs for every on-disk (`file://`) external Sass module
/// edge in the workspace — and, transitively, every module reachable through a generated SIF's
/// `@forward` chain — so the existing SIF-mode query path can pair them against import targets
/// without a manual `--sif`. (#33)
///
/// Each `file://` `@use`/`@forward`/`@import` source now classifies as an external edge
/// (resolver `is_external_style_module_source`), so its symbols are otherwise invisible and every
/// reference flags `missingSassSymbol`. The bridge reads the resolved on-disk module and produces an
/// [`omena_sif::OmenaSifV1`]; we key the resulting `OmenaQueryExternalSifInputV0.canonical_url` to
/// the *verbatim* edge source so it matches the import target 1:1 in
/// `find_omena_query_external_sif` (the inner SIF still carries the bridge's normalized URL).
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
) -> Vec<OmenaQueryExternalSifInputV0> {
    // A single `file://`-namespace dedup/cycle set: seeded with the verbatim canonical URLs of any
    // explicit `--sif`, then extended with each generated SIF's resolved inner `file://` URI. A
    // workspace `file://` edge source already *is* its resolved URI, so the two keying schemes
    // coincide in that namespace.
    let mut covered = existing_external_sifs
        .iter()
        .map(|input| input.canonical_url.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let mut resolved = Vec::new();
    // Worklist of generated SIFs whose `exports.forwards` still need to be walked.
    let mut worklist: std::collections::VecDeque<omena_sif::OmenaSifV1> =
        std::collections::VecDeque::new();

    // Direct workspace pass: each `file://` `@use`/`@forward`/`@import` edge written in a workspace
    // source. Key to the verbatim edge source (which already IS its resolved `file://` URI).
    for source in workspace_sources {
        let Some(module_sources) =
            summarize_omena_query_sass_module_sources(&source.style_path, &source.style_source)
        else {
            continue;
        };
        let edge_sources = module_sources
            .module_use_edges
            .iter()
            .map(|edge| edge.source.as_str())
            .chain(
                module_sources
                    .module_forward_sources
                    .iter()
                    .map(String::as_str),
            );
        for edge_source in edge_sources {
            // Only `file://` edges are on-disk external modules the bridge can read in-process.
            if !edge_source.starts_with("file://") {
                continue;
            }
            if !covered.insert(edge_source.to_string()) {
                // Already covered by an explicit `--sif` or an earlier edge in this workspace.
                continue;
            }
            // The bridge errors gracefully (never panics) on an unreadable/missing module; we
            // simply skip it so the boundary state still surfaces — we never fabricate a SIF.
            if let Ok(sif) = generate_omena_bridge_sif_for_resolved_style_path(edge_source) {
                // The bridge normalizes the path (symlinks/`..`), so the inner `sif.canonical_url`
                // can differ from the verbatim `file://` edge source. Record BOTH in `covered`:
                // the verbatim key matches the workspace import 1:1 here, and the resolved key is
                // what the transitive walk dedups on — without it a forward cycle that resolves
                // back to this module would regenerate it.
                covered.insert(sif.canonical_url.clone());
                worklist.push_back(sif.clone());
                resolved.push(OmenaQueryExternalSifInputV0 {
                    canonical_url: edge_source.to_string(),
                    sif,
                });
            }
        }
    }

    // Transitive `@forward` walk: pop a generated SIF and resolve each forwarded specifier against
    // that SIF's resolved inner `file://` base, generating the forwarded module's SIF and enqueueing
    // it so the chain (and any diamond) is followed to a fixpoint.
    while let Some(sif) = worklist.pop_front() {
        let base_file_uri = sif.canonical_url.clone();
        for forward in &sif.exports.forwards {
            let specifier = forward.canonical_url.as_str();
            // `sass:` builtins and `http(s)://` modules are not on-disk; the bridge cannot read
            // them in-process, so they keep surfacing their boundary state.
            if specifier.starts_with("sass:")
                || specifier.starts_with("http://")
                || specifier.starts_with("https://")
            {
                continue;
            }
            // Resolve the raw forwarded specifier (e.g. `"./tokens"`) relative to the forwarding
            // module's resolved `file://` URI. `None` => genuinely unresolvable; never fabricate.
            let Some(child_url) = omena_query::resolve_omena_query_style_uri_for_specifier(
                base_file_uri.as_str(),
                None,
                specifier,
            ) else {
                continue;
            };
            // Cycle/diamond guard: dedup on the resolved `file://` identity. A relative specifier
            // can reach the same physical module via different strings, so the verbatim string is
            // not a sound key — the resolved URI is.
            if !covered.insert(child_url.clone()) {
                continue;
            }
            // Unreadable/missing forwarded module: skip so it keeps surfacing its boundary state.
            if let Ok(child) = generate_omena_bridge_sif_for_resolved_style_path(child_url.as_str())
            {
                worklist.push_back(child.clone());
                // Key the entry to the resolved `file://` URI; it equals `child.canonical_url`, so
                // `find_omena_query_external_sif` matches on either field.
                resolved.push(OmenaQueryExternalSifInputV0 {
                    canonical_url: child_url,
                    sif: child,
                });
            }
        }
    }

    resolved
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
        print_json(&summary)?;
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
        print_json(&summary)?;
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
        Ok(summarize_omena_query_source_diagnostics_for_workspace_file(
            source_uri.as_str(),
            source_source.as_str(),
            style_sources.as_slice(),
            package_manifests.as_slice(),
        ))
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
