use super::*;

pub(super) fn summarize_omena_query_cross_file_summary(
    style_fact_entries: &[OmenaQueryStyleFactEntry],
    css_modules_resolution: &OmenaQueryCssModulesCrossFileResolutionV0,
    sass_module_resolution: &OmenaQuerySassModuleCrossFileResolutionV0,
) -> OmenaQueryCrossFileSummaryV0 {
    let mut edges = Vec::new();

    for edge in &css_modules_resolution.edges {
        let edge_kind = match edge.import_kind {
            "composes" => "cssModulesComposesImport",
            "value" => "cssModulesValueImport",
            "icss" => "cssModulesIcssImport",
            _ => "cssModulesImport",
        };
        let provenance = match edge.import_kind {
            "composes" => vec![
                "omena-query.css-modules-cross-file-resolution",
                "omena-parser.css-module-composes-facts",
            ],
            "value" => vec![
                "omena-query.css-modules-cross-file-resolution",
                "omena-parser.css-module-value-facts",
            ],
            "icss" => vec![
                "omena-query.css-modules-cross-file-resolution",
                "omena-parser.icss-facts",
            ],
            _ => vec!["omena-query.css-modules-cross-file-resolution"],
        };
        edges.push(build_omena_query_cross_file_summary_edge(
            OmenaQueryCrossFileSummaryEdgeInput {
                edge_kind,
                from_kind: "style",
                from_path: edge.from_style_path.clone(),
                target_kind: edge.resolved_style_path.as_ref().map(|_| "style"),
                target_path: edge.resolved_style_path.clone(),
                source: Some(edge.source.clone()),
                owner_selector_name: None,
                local_name: None,
                remote_name: None,
                target_names: edge.imported_names.clone(),
                status: edge.status,
                provenance,
            },
        ));
    }

    for edge in &css_modules_resolution.composes_closure_edges {
        edges.push(build_omena_query_cross_file_summary_edge(
            OmenaQueryCrossFileSummaryEdgeInput {
                edge_kind: "cssModulesComposesClosure",
                from_kind: "style",
                from_path: edge.from_style_path.clone(),
                target_kind: Some("style"),
                target_path: Some(edge.target_style_path.clone()),
                source: None,
                owner_selector_name: Some(edge.owner_selector_name.clone()),
                local_name: None,
                remote_name: Some(edge.target_selector_name.clone()),
                target_names: vec![edge.target_selector_name.clone()],
                status: "reachable",
                provenance: vec![
                    "omena-query.css-modules-cross-file-resolution",
                    "omena-parser.css-module-composes-facts",
                ],
            },
        ));
    }

    for edge in &css_modules_resolution.value_closure_edges {
        edges.push(build_omena_query_cross_file_summary_edge(
            OmenaQueryCrossFileSummaryEdgeInput {
                edge_kind: "cssModulesValueClosure",
                from_kind: "style",
                from_path: edge.from_style_path.clone(),
                target_kind: Some("style"),
                target_path: Some(edge.target_style_path.clone()),
                source: None,
                owner_selector_name: None,
                local_name: Some(edge.value_name.clone()),
                remote_name: Some(edge.target_value_name.clone()),
                target_names: vec![edge.target_value_name.clone()],
                status: "reachable",
                provenance: vec![
                    "omena-query.css-modules-cross-file-resolution",
                    "omena-parser.css-module-value-facts",
                ],
            },
        ));
    }

    for edge in &css_modules_resolution.icss_closure_edges {
        edges.push(build_omena_query_cross_file_summary_edge(
            OmenaQueryCrossFileSummaryEdgeInput {
                edge_kind: "cssModulesIcssClosure",
                from_kind: "style",
                from_path: edge.from_style_path.clone(),
                target_kind: Some("style"),
                target_path: Some(edge.target_style_path.clone()),
                source: None,
                owner_selector_name: None,
                local_name: Some(edge.name.clone()),
                remote_name: Some(edge.target_name.clone()),
                target_names: vec![edge.target_name.clone()],
                status: "reachable",
                provenance: vec![
                    "omena-query.css-modules-cross-file-resolution",
                    "omena-parser.icss-facts",
                ],
            },
        ));
    }

    for edge in &sass_module_resolution.edges {
        edges.push(build_omena_query_cross_file_summary_edge(
            OmenaQueryCrossFileSummaryEdgeInput {
                edge_kind: edge.edge_kind,
                from_kind: "style",
                from_path: edge.from_style_path.clone(),
                target_kind: edge.resolved_style_path.as_ref().map(|_| "style"),
                target_path: edge.resolved_style_path.clone(),
                source: Some(edge.source.clone()),
                owner_selector_name: None,
                local_name: edge.namespace.clone(),
                remote_name: edge.forward_prefix.clone(),
                target_names: edge.visibility_filter_names.clone(),
                status: edge.status,
                provenance: vec![
                    "omena-query.sass-module-cross-file-resolution",
                    "omena-parser.sass-module-facts",
                ],
            },
        ));
    }

    for edge in &sass_module_resolution.graph_closure_edges {
        edges.push(build_omena_query_cross_file_summary_edge(
            OmenaQueryCrossFileSummaryEdgeInput {
                edge_kind: "sassModuleGraphClosure",
                from_kind: "style",
                from_path: edge.from_style_path.clone(),
                target_kind: Some("style"),
                target_path: Some(edge.target_style_path.clone()),
                source: None,
                owner_selector_name: None,
                local_name: edge.namespace.clone(),
                remote_name: edge.forward_prefix.clone(),
                target_names: edge.visibility_filter_names.clone(),
                status: "reachable",
                provenance: vec![
                    "omena-query.sass-module-cross-file-resolution",
                    "omena-parser.sass-module-facts",
                ],
            },
        ));
    }

    let design_token_declarations = collect_design_token_declarations_by_name(style_fact_entries);
    let design_token_reachability =
        collect_design_token_reachable_style_paths_by_origin(sass_module_resolution);

    for entry in style_fact_entries {
        let local_declarations = entry
            .facts
            .custom_property_decl_names
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        for name in &entry.facts.custom_property_ref_names {
            let target = resolve_design_token_reference_target(
                entry.style_path.as_str(),
                name.as_str(),
                &local_declarations,
                &design_token_declarations,
                &design_token_reachability,
            );
            let provenance = target.provenance();
            let target_style_path = target.target_style_path;
            let status = target.status;
            edges.push(build_omena_query_cross_file_summary_edge(
                OmenaQueryCrossFileSummaryEdgeInput {
                    edge_kind: "styleDesignTokenReference",
                    from_kind: "style",
                    from_path: entry.style_path.clone(),
                    target_kind: target_style_path.as_ref().map(|_| "style"),
                    target_path: target_style_path,
                    source: None,
                    owner_selector_name: None,
                    local_name: Some(name.clone()),
                    remote_name: None,
                    target_names: vec![name.clone()],
                    status,
                    provenance,
                },
            ));
        }
    }

    edges.sort_by_key(|edge| edge.edge_id.clone());
    let summary_hash = stable_omena_query_cross_file_summary_hash(&edges);

    OmenaQueryCrossFileSummaryV0 {
        schema_version: "0",
        product: "omena-query.cross-file-summary",
        status: "summaryEdgeSeed",
        summary_scope: "styleSemanticGraphBatch",
        style_count: style_fact_entries.len(),
        summary_edge_count: edges.len(),
        summary_hash,
        edges,
        capabilities: OmenaQueryCrossFileSummaryCapabilitiesV0 {
            css_modules_composes_edges_ready: true,
            css_modules_value_edges_ready: true,
            css_modules_icss_edges_ready: true,
            sass_module_edges_ready: true,
            style_design_token_reference_edges_ready: true,
            source_selector_reference_edges_ready: false,
            stable_summary_hash_ready: true,
            linear_provenance_ready: true,
        },
        next_priorities: vec![
            "sourceSelectorReferenceSummaryEdges",
            "summaryEdgeEquivalenceGate",
        ],
    }
}

#[derive(Debug, Clone)]
struct DesignTokenReferenceTarget {
    target_style_path: Option<String>,
    status: &'static str,
    resolution_provenance: Option<&'static str>,
}

impl DesignTokenReferenceTarget {
    fn provenance(&self) -> Vec<&'static str> {
        let mut provenance = vec![
            "omena-query.style-semantic-graph-batch",
            "omena-parser.custom-property-facts",
        ];
        if let Some(resolution_provenance) = self.resolution_provenance {
            provenance.push(resolution_provenance);
        }
        provenance
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct DesignTokenReachableStylePath {
    distance: usize,
    target_style_path: String,
}

fn collect_design_token_declarations_by_name(
    style_fact_entries: &[OmenaQueryStyleFactEntry],
) -> BTreeMap<String, BTreeSet<String>> {
    let mut declarations_by_name = BTreeMap::<String, BTreeSet<String>>::new();
    for entry in style_fact_entries {
        for name in &entry.facts.custom_property_decl_names {
            declarations_by_name
                .entry(name.clone())
                .or_default()
                .insert(entry.style_path.clone());
        }
    }
    declarations_by_name
}

fn collect_design_token_reachable_style_paths_by_origin(
    sass_module_resolution: &OmenaQuerySassModuleCrossFileResolutionV0,
) -> BTreeMap<String, Vec<DesignTokenReachableStylePath>> {
    let mut reachable_by_origin =
        BTreeMap::<String, BTreeSet<DesignTokenReachableStylePath>>::new();

    for edge in &sass_module_resolution.edges {
        if edge.status != "resolved" {
            continue;
        }
        let Some(target_style_path) = edge.resolved_style_path.as_ref() else {
            continue;
        };
        reachable_by_origin
            .entry(edge.from_style_path.clone())
            .or_default()
            .insert(DesignTokenReachableStylePath {
                distance: 1,
                target_style_path: target_style_path.clone(),
            });
    }

    for edge in &sass_module_resolution.graph_closure_edges {
        reachable_by_origin
            .entry(edge.from_style_path.clone())
            .or_default()
            .insert(DesignTokenReachableStylePath {
                distance: edge.depth,
                target_style_path: edge.target_style_path.clone(),
            });
    }

    reachable_by_origin
        .into_iter()
        .map(|(origin, reachable)| (origin, reachable.into_iter().collect()))
        .collect()
}

fn resolve_design_token_reference_target(
    from_style_path: &str,
    name: &str,
    local_declarations: &BTreeSet<&str>,
    declarations_by_name: &BTreeMap<String, BTreeSet<String>>,
    reachable_by_origin: &BTreeMap<String, Vec<DesignTokenReachableStylePath>>,
) -> DesignTokenReferenceTarget {
    if local_declarations.contains(name) {
        return DesignTokenReferenceTarget {
            target_style_path: Some(from_style_path.to_string()),
            status: "localResolved",
            resolution_provenance: None,
        };
    }

    let Some(declaration_paths) = declarations_by_name.get(name) else {
        return DesignTokenReferenceTarget {
            target_style_path: None,
            status: "unresolvedReference",
            resolution_provenance: None,
        };
    };

    let Some(reachable_paths) = reachable_by_origin.get(from_style_path) else {
        return DesignTokenReferenceTarget {
            target_style_path: None,
            status: "unresolvedReference",
            resolution_provenance: None,
        };
    };

    let target_style_path = reachable_paths
        .iter()
        .filter(|reachable| declaration_paths.contains(reachable.target_style_path.as_str()))
        .min_by_key(|reachable| (reachable.distance, reachable.target_style_path.as_str()))
        .map(|reachable| reachable.target_style_path.clone());

    if let Some(target_style_path) = target_style_path {
        DesignTokenReferenceTarget {
            target_style_path: Some(target_style_path),
            status: "importResolved",
            resolution_provenance: Some("omena-query.sass-module-cross-file-resolution"),
        }
    } else {
        DesignTokenReferenceTarget {
            target_style_path: None,
            status: "unresolvedReference",
            resolution_provenance: None,
        }
    }
}

pub fn summarize_omena_query_source_selector_reference_cross_file_summary(
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> OmenaQueryCrossFileSummaryV0 {
    let definitions =
        super::source_refs::summarize_omena_query_style_selector_definitions(style_sources);
    let references = super::source_refs::collect_omena_query_source_selector_references(
        style_sources,
        source_documents,
        package_manifests,
    );
    let mut edges = references
        .into_iter()
        .map(|reference| {
            let candidate = reference.candidate;
            let source_candidate = OmenaQuerySourceSelectorCandidateV0 {
                kind: candidate.kind,
                name: candidate.name.clone(),
                range: candidate.range,
                source: candidate.source,
                target_style_uri: candidate.target_style_uri.clone(),
            };
            let matched_definitions =
                resolve_omena_query_style_selector_definitions_for_source_candidate(
                    &source_candidate,
                    definitions.as_slice(),
                );
            let target_names = if candidate.kind == "sourceSelectorPrefixReference" {
                matched_definitions
                    .iter()
                    .map(|definition| definition.name.clone())
                    .collect::<Vec<_>>()
            } else {
                vec![candidate.name.clone()]
            };
            let status = if matched_definitions.is_empty() {
                "unresolved"
            } else {
                "resolved"
            };
            build_omena_query_cross_file_summary_edge(OmenaQueryCrossFileSummaryEdgeInput {
                edge_kind: candidate.kind,
                from_kind: "source",
                from_path: candidate.uri,
                target_kind: candidate.target_style_uri.as_ref().map(|_| "style"),
                target_path: candidate.target_style_uri,
                source: None,
                owner_selector_name: None,
                local_name: Some(candidate.name),
                remote_name: None,
                target_names,
                status,
                provenance: vec![
                    "omena-query.source-selector-references",
                    "omena-query.style-selector-definitions",
                ],
            })
        })
        .collect::<Vec<_>>();

    edges.sort_by_key(|edge| edge.edge_id.clone());
    let summary_hash = stable_omena_query_cross_file_summary_hash(&edges);

    OmenaQueryCrossFileSummaryV0 {
        schema_version: "0",
        product: "omena-query.cross-file-summary",
        status: "sourceSelectorSummaryEdgeSeed",
        summary_scope: "sourceSelectorReferences",
        style_count: style_sources.len(),
        summary_edge_count: edges.len(),
        summary_hash,
        edges,
        capabilities: OmenaQueryCrossFileSummaryCapabilitiesV0 {
            css_modules_composes_edges_ready: false,
            css_modules_value_edges_ready: false,
            css_modules_icss_edges_ready: false,
            sass_module_edges_ready: false,
            style_design_token_reference_edges_ready: false,
            source_selector_reference_edges_ready: true,
            stable_summary_hash_ready: true,
            linear_provenance_ready: true,
        },
        next_priorities: vec!["sourceSelectorReferenceSummaryEquivalenceGate"],
    }
}

pub fn summarize_omena_query_workspace_cross_file_summary(
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> OmenaQueryCrossFileSummaryV0 {
    let style_pairs = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();
    let style_fact_entries = super::collect_omena_query_style_fact_entries(style_pairs.as_slice());
    let css_modules_resolution =
        super::summarize_css_modules_cross_file_resolution(&style_fact_entries, package_manifests);
    let sass_module_resolution =
        super::summarize_sass_module_cross_file_resolution(&style_fact_entries, package_manifests);
    let style_summary = summarize_omena_query_cross_file_summary(
        &style_fact_entries,
        &css_modules_resolution,
        &sass_module_resolution,
    );
    let source_summary = summarize_omena_query_source_selector_reference_cross_file_summary(
        style_sources,
        source_documents,
        package_manifests,
    );

    merge_omena_query_cross_file_summaries(
        "workspaceSummaryEdgeSeed",
        "workspaceStyleAndSource",
        style_sources.len(),
        &[style_summary, source_summary],
    )
}

fn merge_omena_query_cross_file_summaries(
    status: &'static str,
    summary_scope: &'static str,
    style_count: usize,
    summaries: &[OmenaQueryCrossFileSummaryV0],
) -> OmenaQueryCrossFileSummaryV0 {
    let mut edges = summaries
        .iter()
        .flat_map(|summary| summary.edges.clone())
        .collect::<Vec<_>>();
    edges.sort_by_key(|edge| edge.edge_id.clone());
    edges.dedup_by(|left, right| left.edge_id == right.edge_id);
    let summary_hash = stable_omena_query_cross_file_summary_hash(edges.as_slice());

    OmenaQueryCrossFileSummaryV0 {
        schema_version: "0",
        product: "omena-query.cross-file-summary",
        status,
        summary_scope,
        style_count,
        summary_edge_count: edges.len(),
        summary_hash,
        edges,
        capabilities: merge_omena_query_cross_file_summary_capabilities(summaries),
        next_priorities: vec!["workspaceSummaryHashInvalidationGate"],
    }
}

fn merge_omena_query_cross_file_summary_capabilities(
    summaries: &[OmenaQueryCrossFileSummaryV0],
) -> OmenaQueryCrossFileSummaryCapabilitiesV0 {
    OmenaQueryCrossFileSummaryCapabilitiesV0 {
        css_modules_composes_edges_ready: summaries
            .iter()
            .any(|summary| summary.capabilities.css_modules_composes_edges_ready),
        css_modules_value_edges_ready: summaries
            .iter()
            .any(|summary| summary.capabilities.css_modules_value_edges_ready),
        css_modules_icss_edges_ready: summaries
            .iter()
            .any(|summary| summary.capabilities.css_modules_icss_edges_ready),
        sass_module_edges_ready: summaries
            .iter()
            .any(|summary| summary.capabilities.sass_module_edges_ready),
        style_design_token_reference_edges_ready: summaries.iter().any(|summary| {
            summary
                .capabilities
                .style_design_token_reference_edges_ready
        }),
        source_selector_reference_edges_ready: summaries
            .iter()
            .any(|summary| summary.capabilities.source_selector_reference_edges_ready),
        stable_summary_hash_ready: summaries
            .iter()
            .all(|summary| summary.capabilities.stable_summary_hash_ready),
        linear_provenance_ready: summaries
            .iter()
            .all(|summary| summary.capabilities.linear_provenance_ready),
    }
}

#[derive(Debug, Clone)]
struct OmenaQueryCrossFileSummaryEdgeInput {
    edge_kind: &'static str,
    from_kind: &'static str,
    target_kind: Option<&'static str>,
    from_path: String,
    target_path: Option<String>,
    source: Option<String>,
    owner_selector_name: Option<String>,
    local_name: Option<String>,
    remote_name: Option<String>,
    target_names: Vec<String>,
    status: &'static str,
    provenance: Vec<&'static str>,
}

fn build_omena_query_cross_file_summary_edge(
    input: OmenaQueryCrossFileSummaryEdgeInput,
) -> OmenaQueryCrossFileSummaryEdgeV0 {
    let edge_id = omena_query_cross_file_summary_edge_id(&input);
    let linear_provenance = summarize_omena_query_linear_provenance(input.provenance.as_slice());

    OmenaQueryCrossFileSummaryEdgeV0 {
        edge_id,
        edge_kind: input.edge_kind,
        from_kind: input.from_kind,
        from_path: input.from_path,
        target_kind: input.target_kind,
        target_path: input.target_path,
        source: input.source,
        owner_selector_name: input.owner_selector_name,
        local_name: input.local_name,
        remote_name: input.remote_name,
        target_names: input.target_names,
        status: input.status,
        provenance: input.provenance,
        linear_provenance,
    }
}

fn omena_query_cross_file_summary_edge_id(input: &OmenaQueryCrossFileSummaryEdgeInput) -> String {
    format!(
        "{}|fromKind:{}|from:{}|targetKind:{}|target:{}|source:{}|owner:{}|local:{}|remote:{}|names:{}",
        input.edge_kind,
        input.from_kind,
        input.from_path,
        input.target_kind.unwrap_or("-"),
        input.target_path.as_deref().unwrap_or("-"),
        input.source.as_deref().unwrap_or("-"),
        input.owner_selector_name.as_deref().unwrap_or("-"),
        input.local_name.as_deref().unwrap_or("-"),
        input.remote_name.as_deref().unwrap_or("-"),
        input.target_names.join(",")
    )
}

fn stable_omena_query_cross_file_summary_hash(
    edges: &[OmenaQueryCrossFileSummaryEdgeV0],
) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    stable_omena_query_hash_piece(&mut hash, "omena-query.cross-file-summary");
    stable_omena_query_hash_piece(&mut hash, "0");
    for edge in edges {
        stable_omena_query_hash_piece(&mut hash, edge.edge_id.as_str());
        stable_omena_query_hash_piece(&mut hash, edge.status);
        stable_omena_query_hash_piece(&mut hash, edge.linear_provenance.semiring_identifier());
        let term_count = edge.linear_provenance.term_count.to_string();
        stable_omena_query_hash_piece(&mut hash, term_count.as_str());
        for term in &edge.linear_provenance.terms {
            let coefficient = term.coefficient.to_string();
            stable_omena_query_hash_piece(&mut hash, coefficient.as_str());
            stable_omena_query_hash_piece(&mut hash, term.label);
        }
    }
    format!("{hash:016x}")
}

fn stable_omena_query_hash_piece(hash: &mut u64, piece: &str) {
    for byte in piece.as_bytes() {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x100000001b3);
    }
    *hash ^= 0xff;
    *hash = hash.wrapping_mul(0x100000001b3);
}
