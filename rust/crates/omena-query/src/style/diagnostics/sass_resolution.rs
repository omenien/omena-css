use std::collections::{BTreeMap, BTreeSet};

use super::render::whole_file_omena_query_style_range;
use super::shared::*;
#[cfg(feature = "salsa-memo")]
use crate::style::salsa_memo::OmenaQueryStyleMemoHostV0;

pub fn summarize_omena_query_sass_module_resolution_identity_diagnostics_for_workspace(
    target_style_path: &str,
    workspace_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    if !workspace_sources
        .iter()
        .any(|source| source.style_path == target_style_path)
    {
        return Vec::new();
    }
    #[cfg(feature = "salsa-memo")]
    {
        let mut host = OmenaQueryStyleMemoHostV0::new();
        if let Some(selector) = host.workspace_revision_selector(
            workspace_sources,
            &[],
            package_manifests,
            &[],
            resolution_inputs,
        ) {
            return selector
                .sass_module_resolution_identity_diagnostics_for_workspace(target_style_path);
        }
    }

    let resolution = summarize_omena_query_sass_module_cross_file_resolution_for_workspace(
        workspace_sources,
        package_manifests,
        resolution_inputs.bundler_path_mappings.as_slice(),
        resolution_inputs.tsconfig_path_mappings.as_slice(),
    );
    summarize_omena_query_sass_module_resolution_identity_diagnostics_for_workspace_from_resolution(
        target_style_path,
        workspace_sources,
        &resolution,
    )
}

pub(in crate::style) fn summarize_omena_query_sass_module_resolution_identity_diagnostics_for_workspace_from_resolution(
    target_style_path: &str,
    workspace_sources: &[OmenaQueryStyleSourceInputV0],
    resolution: &OmenaQuerySassModuleCrossFileResolutionV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let Some(target) = workspace_sources
        .iter()
        .find(|source| source.style_path == target_style_path)
    else {
        return Vec::new();
    };
    let range = whole_file_omena_query_style_range(target.style_source.as_str());
    let mut emitted = BTreeSet::new();
    let mut diagnostics = Vec::new();

    for edge in resolution
        .edges
        .iter()
        .filter(|edge| edge.from_style_path == target_style_path)
    {
        let visible_symlink_links = edge
            .symlink_chain_links
            .iter()
            .filter(|link| !is_platform_alias_omena_query_symlink_link(link))
            .collect::<Vec<_>>();
        if !visible_symlink_links.is_empty()
            && emitted.insert((
                "sassModuleSymlinkResolution",
                edge.source.clone(),
                edge.resolved_style_path.clone(),
            ))
        {
            let target_path = edge
                .resolved_style_path
                .as_deref()
                .unwrap_or(edge.source.as_str());
            let link_summary = visible_symlink_links
                .first()
                .map(|link| format!("; first link {} -> {}", link.link_path, link.target_path))
                .unwrap_or_default();
            diagnostics.push(OmenaQueryStyleDiagnosticV0 {
                code: "sassModuleSymlinkResolution",
                severity: "hint",
                provenance: omena_query_evidence_graph_provenance![
                    "omena-query.sass-module-cross-file-resolution",
                    "omena-resolver.symlink-chain-metadata",
                    "omena-query.style-diagnostics",
                ],
                range,
                message: format!(
                    "Sass module '{}' resolves to '{}' through {} symlink link(s){}.",
                    edge.source,
                    target_path,
                    visible_symlink_links.len(),
                    link_summary
                ),
                tags: Vec::new(),
                create_custom_property: None,
                cascade_narrowing: None,
                cascade_confidence: None,
                polynomial_provenance: None,
                cross_file_scc: None,
            });
        }

        if edge.configuration_variable_count > 0
            && let Some(identity_key) = edge.module_instance_identity_key.as_ref()
            && emitted.insert((
                "sassModuleInstanceIdentity",
                edge.source.clone(),
                Some(identity_key.clone()),
            ))
        {
            diagnostics.push(OmenaQueryStyleDiagnosticV0 {
                code: "sassModuleInstanceIdentity",
                severity: "hint",
                provenance: omena_query_evidence_graph_provenance![
                    "omena-query.sass-module-cross-file-resolution",
                    "omena-query.module-instance-identity",
                    "omena-query.style-diagnostics",
                ],
                range,
                message: format!(
                    "Sass module '{}' uses {} configured variable(s); module instance identity is {}.",
                    edge.source, edge.configuration_variable_count, identity_key
                ),
                tags: Vec::new(),
                create_custom_property: None,
                cascade_narrowing: None,
                cascade_confidence: None,
                polynomial_provenance: None,
                cross_file_scc: None,
            });
        }

        if !edge.invalid_configuration_variable_names.is_empty()
            && emitted.insert((
                "sassModuleInvalidConfiguration",
                edge.source.clone(),
                edge.resolved_style_path.clone(),
            ))
        {
            let target_path = edge
                .resolved_style_path
                .as_deref()
                .unwrap_or(edge.source.as_str());
            diagnostics.push(OmenaQueryStyleDiagnosticV0 {
                code: "sassModuleInvalidConfiguration",
                severity: "error",
                provenance: omena_query_evidence_graph_provenance![
                    "omena-query.sass-module-cross-file-resolution",
                    "omena-query.module-instance-identity",
                    "omena-query.style-diagnostics",
                ],
                range,
                message: format!(
                    "Sass module '{}' configures {} on '{}', but Sass @use/@forward with(...) can configure only public !default variables.",
                    edge.source,
                    format_omena_query_sass_configuration_variable_names(
                        edge.invalid_configuration_variable_names.as_slice()
                    ),
                    target_path
                ),
                tags: Vec::new(),
                create_custom_property: None,
                cascade_narrowing: None,
                cascade_confidence: None,
                polynomial_provenance: None,
                cross_file_scc: None,
            });
        }
    }

    for edge in resolution
        .graph_closure_edges
        .iter()
        .filter(|edge| edge.from_style_path == target_style_path)
        .filter(|edge| edge.configuration_variable_count > 0)
    {
        let Some(identity_key) = edge.module_instance_identity_key.as_ref() else {
            continue;
        };
        if !emitted.insert((
            "sassModuleInstanceIdentity",
            edge.target_style_path.clone(),
            Some(identity_key.clone()),
        )) {
            continue;
        }
        diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code: "sassModuleInstanceIdentity",
            severity: "hint",
            provenance: omena_query_evidence_graph_provenance![
                "omena-query.sass-module-cross-file-resolution",
                "omena-query.module-instance-identity",
                "omena-query.style-diagnostics",
            ],
            range,
            message: format!(
                "Sass module graph reaches configured module instance '{}' in {} hop(s); module instance identity is {}.",
                edge.target_style_path, edge.depth, identity_key
            ),
            tags: Vec::new(),
            create_custom_property: None,
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
        });
    }
    for edge in resolution
        .graph_closure_edges
        .iter()
        .filter(|edge| edge.from_style_path == target_style_path)
        .filter(|edge| !edge.invalid_configuration_variable_names.is_empty())
    {
        if !emitted.insert((
            "sassModuleInvalidConfiguration",
            edge.target_style_path.clone(),
            Some(edge.configuration_signature.clone()),
        )) {
            continue;
        }
        diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code: "sassModuleInvalidConfiguration",
            severity: "error",
            provenance: omena_query_evidence_graph_provenance![
                "omena-query.sass-module-cross-file-resolution",
                "omena-query.module-instance-identity",
                "omena-query.style-diagnostics",
            ],
            range,
            message: format!(
                "Sass module graph reaches invalid configuration for '{}': {} are not public !default variables.",
                edge.target_style_path,
                format_omena_query_sass_configuration_variable_names(
                    edge.invalid_configuration_variable_names.as_slice()
                )
            ),
            tags: Vec::new(),
            create_custom_property: None,
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
        });
    }
    diagnostics.extend(
        summarize_omena_query_sass_module_configuration_conflict_diagnostics(
            target_style_path,
            workspace_sources,
            resolution,
            range,
        ),
    );

    diagnostics
}

fn summarize_omena_query_sass_module_configuration_conflict_diagnostics(
    target_style_path: &str,
    workspace_sources: &[OmenaQueryStyleSourceInputV0],
    resolution: &OmenaQuerySassModuleCrossFileResolutionV0,
    range: ParserRangeV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let mut signatures_by_target = BTreeMap::<String, BTreeSet<String>>::new();
    for edge in resolution
        .graph_closure_edges
        .iter()
        .filter(|edge| edge.from_style_path == target_style_path)
        .filter(|edge| edge.configuration_variable_count > 0)
    {
        signatures_by_target
            .entry(edge.target_style_path.clone())
            .or_default()
            .insert(edge.configuration_signature.clone());
    }
    for (target, signatures) in collect_omena_query_sass_module_load_order_configuration_conflicts(
        target_style_path,
        workspace_sources,
        resolution,
    ) {
        signatures_by_target
            .entry(target)
            .or_default()
            .extend(signatures);
    }

    signatures_by_target
        .into_iter()
        .filter(|(_, signatures)| signatures.len() > 1)
        .map(|(target, signatures)| OmenaQueryStyleDiagnosticV0 {
            code: "sassModuleConfigurationConflict",
            severity: "error",
            provenance: omena_query_evidence_graph_provenance![
                "omena-query.sass-module-cross-file-resolution",
                "omena-query.module-instance-identity",
                "omena-query.style-diagnostics",
            ],
            range,
            message: format!(
                "Sass module '{target}' is reached with {} different configurations ({}); Sass modules can be configured only once per compilation.",
                signatures.len(),
                signatures.into_iter().collect::<Vec<_>>().join(", ")
            ),
            tags: Vec::new(),
            create_custom_property: None,
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
        })
        .collect()
}

fn collect_omena_query_sass_module_load_order_configuration_conflicts(
    target_style_path: &str,
    workspace_sources: &[OmenaQueryStyleSourceInputV0],
    resolution: &OmenaQuerySassModuleCrossFileResolutionV0,
) -> BTreeMap<String, BTreeSet<String>> {
    let source_by_path = workspace_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut edges_by_from = BTreeMap::<&str, Vec<&OmenaQuerySassModuleEdgeResolutionV0>>::new();
    for edge in resolution
        .edges
        .iter()
        .filter(|edge| edge.status == "resolved" && edge.resolved_style_path.is_some())
    {
        edges_by_from
            .entry(edge.from_style_path.as_str())
            .or_default()
            .push(edge);
    }
    for (style_path, edges) in &mut edges_by_from {
        let style_source = source_by_path.get(style_path).copied().unwrap_or_default();
        edges.sort_by_key(|edge| {
            (
                omena_query_sass_module_edge_source_offset(
                    style_source,
                    edge.edge_kind,
                    edge.source.as_str(),
                ),
                edge.edge_kind,
                edge.rule_ordinal,
                edge.source.clone(),
            )
        });
    }

    let mut loaded_signatures_by_target = BTreeMap::new();
    let mut active_stack = BTreeSet::new();
    let mut conflicts_by_target = BTreeMap::new();
    collect_omena_query_sass_module_load_order_configuration_conflicts_for_style(
        target_style_path,
        &edges_by_from,
        &mut loaded_signatures_by_target,
        &mut active_stack,
        &mut conflicts_by_target,
    );
    conflicts_by_target
}

fn collect_omena_query_sass_module_load_order_configuration_conflicts_for_style(
    style_path: &str,
    edges_by_from: &BTreeMap<&str, Vec<&OmenaQuerySassModuleEdgeResolutionV0>>,
    loaded_signatures_by_target: &mut BTreeMap<String, String>,
    active_stack: &mut BTreeSet<String>,
    conflicts_by_target: &mut BTreeMap<String, BTreeSet<String>>,
) {
    if !active_stack.insert(style_path.to_string()) {
        return;
    }
    if let Some(edges) = edges_by_from.get(style_path) {
        for edge in edges {
            let Some(target_style_path) = edge.resolved_style_path.as_ref() else {
                continue;
            };
            let requested_signature = edge.configuration_signature.clone();
            let should_visit_target =
                match loaded_signatures_by_target.get(target_style_path.as_str()) {
                    Some(existing_signature)
                        if is_unconfigured_omena_query_sass_module_signature(
                            requested_signature.as_str(),
                        ) || existing_signature == &requested_signature =>
                    {
                        false
                    }
                    Some(existing_signature) => {
                        let signatures = conflicts_by_target
                            .entry(target_style_path.clone())
                            .or_default();
                        signatures.insert(existing_signature.clone());
                        signatures.insert(requested_signature);
                        false
                    }
                    None => {
                        loaded_signatures_by_target
                            .insert(target_style_path.clone(), requested_signature);
                        true
                    }
                };
            if should_visit_target {
                collect_omena_query_sass_module_load_order_configuration_conflicts_for_style(
                    target_style_path.as_str(),
                    edges_by_from,
                    loaded_signatures_by_target,
                    active_stack,
                    conflicts_by_target,
                );
            }
        }
    }
    active_stack.remove(style_path);
}

fn is_unconfigured_omena_query_sass_module_signature(signature: &str) -> bool {
    signature == "with:none"
}

fn format_omena_query_sass_configuration_variable_names(names: &[String]) -> String {
    names
        .iter()
        .map(|name| format!("${name}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn omena_query_sass_module_edge_source_offset(
    style_source: &str,
    edge_kind: &str,
    source: &str,
) -> usize {
    let keyword = match edge_kind {
        "sassUse" => "@use",
        "sassForward" => "@forward",
        _ => return usize::MAX,
    };
    let mut search_start = 0usize;
    while let Some(relative_keyword_start) = style_source[search_start..].find(keyword) {
        let keyword_start = search_start + relative_keyword_start;
        let after_keyword = &style_source[keyword_start + keyword.len()..];
        let Some(relative_source_start) = after_keyword.find(source) else {
            search_start = keyword_start + keyword.len();
            continue;
        };
        let between_keyword_and_source = &after_keyword[..relative_source_start];
        if !between_keyword_and_source.contains(';') && !between_keyword_and_source.contains('{') {
            return keyword_start;
        }
        search_start = keyword_start + keyword.len();
    }
    usize::MAX
}

fn is_platform_alias_omena_query_symlink_link(link: &OmenaQuerySymlinkChainLinkV0) -> bool {
    matches!(
        (link.link_path.as_str(), link.target_path.as_str()),
        ("/var", "/private/var") | ("/tmp", "/private/tmp") | ("/etc", "/private/etc")
    )
}
