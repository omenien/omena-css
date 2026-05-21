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
            edge_kind,
            edge.from_style_path.clone(),
            edge.resolved_style_path.clone(),
            Some(edge.source.clone()),
            None,
            None,
            None,
            edge.imported_names.clone(),
            edge.status,
            provenance,
        ));
    }

    for edge in &css_modules_resolution.composes_closure_edges {
        edges.push(build_omena_query_cross_file_summary_edge(
            "cssModulesComposesClosure",
            edge.from_style_path.clone(),
            Some(edge.target_style_path.clone()),
            None,
            Some(edge.owner_selector_name.clone()),
            None,
            Some(edge.target_selector_name.clone()),
            vec![edge.target_selector_name.clone()],
            "reachable",
            vec![
                "omena-query.css-modules-cross-file-resolution",
                "omena-parser.css-module-composes-facts",
            ],
        ));
    }

    for edge in &css_modules_resolution.value_closure_edges {
        edges.push(build_omena_query_cross_file_summary_edge(
            "cssModulesValueClosure",
            edge.from_style_path.clone(),
            Some(edge.target_style_path.clone()),
            None,
            None,
            Some(edge.value_name.clone()),
            Some(edge.target_value_name.clone()),
            vec![edge.target_value_name.clone()],
            "reachable",
            vec![
                "omena-query.css-modules-cross-file-resolution",
                "omena-parser.css-module-value-facts",
            ],
        ));
    }

    for edge in &css_modules_resolution.icss_closure_edges {
        edges.push(build_omena_query_cross_file_summary_edge(
            "cssModulesIcssClosure",
            edge.from_style_path.clone(),
            Some(edge.target_style_path.clone()),
            None,
            None,
            Some(edge.name.clone()),
            Some(edge.target_name.clone()),
            vec![edge.target_name.clone()],
            "reachable",
            vec![
                "omena-query.css-modules-cross-file-resolution",
                "omena-parser.icss-facts",
            ],
        ));
    }

    for edge in &sass_module_resolution.edges {
        edges.push(build_omena_query_cross_file_summary_edge(
            edge.edge_kind,
            edge.from_style_path.clone(),
            edge.resolved_style_path.clone(),
            Some(edge.source.clone()),
            None,
            edge.namespace.clone(),
            edge.forward_prefix.clone(),
            edge.visibility_filter_names.clone(),
            edge.status,
            vec![
                "omena-query.sass-module-cross-file-resolution",
                "omena-parser.sass-module-facts",
            ],
        ));
    }

    for edge in &sass_module_resolution.graph_closure_edges {
        edges.push(build_omena_query_cross_file_summary_edge(
            "sassModuleGraphClosure",
            edge.from_style_path.clone(),
            Some(edge.target_style_path.clone()),
            None,
            None,
            edge.namespace.clone(),
            edge.forward_prefix.clone(),
            edge.visibility_filter_names.clone(),
            "reachable",
            vec![
                "omena-query.sass-module-cross-file-resolution",
                "omena-parser.sass-module-facts",
            ],
        ));
    }

    for entry in style_fact_entries {
        let local_declarations = entry
            .facts
            .custom_property_decl_names
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        for name in &entry.facts.custom_property_ref_names {
            let target_style_path = if local_declarations.contains(name.as_str()) {
                Some(entry.style_path.clone())
            } else {
                None
            };
            let status = if target_style_path.is_some() {
                "localResolved"
            } else {
                "unresolvedReference"
            };
            edges.push(build_omena_query_cross_file_summary_edge(
                "styleDesignTokenReference",
                entry.style_path.clone(),
                target_style_path,
                None,
                None,
                Some(name.clone()),
                None,
                vec![name.clone()],
                status,
                vec![
                    "omena-query.style-semantic-graph-batch",
                    "omena-parser.custom-property-facts",
                ],
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

fn build_omena_query_cross_file_summary_edge(
    edge_kind: &'static str,
    from_style_path: String,
    target_style_path: Option<String>,
    source: Option<String>,
    owner_selector_name: Option<String>,
    local_name: Option<String>,
    remote_name: Option<String>,
    target_names: Vec<String>,
    status: &'static str,
    provenance: Vec<&'static str>,
) -> OmenaQueryCrossFileSummaryEdgeV0 {
    let edge_id = omena_query_cross_file_summary_edge_id(
        edge_kind,
        from_style_path.as_str(),
        target_style_path.as_deref(),
        source.as_deref(),
        owner_selector_name.as_deref(),
        local_name.as_deref(),
        remote_name.as_deref(),
        target_names.as_slice(),
    );
    let linear_provenance = summarize_omena_query_linear_provenance(provenance.as_slice());

    OmenaQueryCrossFileSummaryEdgeV0 {
        edge_id,
        edge_kind,
        from_style_path,
        target_style_path,
        source,
        owner_selector_name,
        local_name,
        remote_name,
        target_names,
        status,
        provenance,
        linear_provenance,
    }
}

fn omena_query_cross_file_summary_edge_id(
    edge_kind: &str,
    from_style_path: &str,
    target_style_path: Option<&str>,
    source: Option<&str>,
    owner_selector_name: Option<&str>,
    local_name: Option<&str>,
    remote_name: Option<&str>,
    target_names: &[String],
) -> String {
    format!(
        "{}|from:{}|target:{}|source:{}|owner:{}|local:{}|remote:{}|names:{}",
        edge_kind,
        from_style_path,
        target_style_path.unwrap_or("-"),
        source.unwrap_or("-"),
        owner_selector_name.unwrap_or("-"),
        local_name.unwrap_or("-"),
        remote_name.unwrap_or("-"),
        target_names.join(",")
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
        for provenance in &edge.provenance {
            stable_omena_query_hash_piece(&mut hash, provenance);
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
