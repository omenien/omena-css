use super::shared::*;

/// RFC-0007-F (#46): composes-target validation restricted to outcomes that are fully resolvable
/// from a single file with no cross-file `--source` context. Only `composes: x` / `composes: x, y`
/// (Local edges) target the file's own selectors and can be checked here; `composes: x from global`
/// (Global edges) reference no concrete selector and produce nothing; `composes: x from './other'`
/// (External edges) require the sibling module's facts, which a single-file invocation does not have,
/// so they are deliberately skipped to avoid a false `missingComposedSelector`/`missingComposedModule`.
/// @value imports are always cross-file and are likewise excluded.
pub fn summarize_omena_query_css_modules_local_composes_style_diagnostics(
    target_style_path: &str,
    target_source: &str,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let dialect = omena_parser_dialect_for_style_path(target_style_path);
    let target_facts = collect_omena_query_omena_parser_style_facts_raw(target_source, dialect);
    let target_class_names = target_facts
        .selectors
        .iter()
        .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
        .map(|selector| selector.name.as_str())
        .collect::<BTreeSet<_>>();
    let mut diagnostics = Vec::new();

    for edge in target_facts.css_module_composes_edges {
        // Global edges resolve to no concrete selector; External edges need cross-file facts.
        // Both are outside the single-file-resolvable surface, so only Local edges are validated.
        if edge.kind != ParsedCssModuleComposesEdgeKind::Local {
            continue;
        }
        let start: u32 = edge.range.start().into();
        let end: u32 = edge.range.end().into();
        let range = parser_range_for_byte_span(
            target_source,
            ParserByteSpanV0 {
                start: start as usize,
                end: end as usize,
            },
        );
        for target_name in edge.target_names {
            if target_class_names.contains(target_name.as_str()) {
                continue;
            }
            diagnostics.push(OmenaQueryStyleDiagnosticV0 {
                code: "missingComposedSelector",
                severity: "warning",
                provenance: vec![
                    "omena-parser.css-modules-composes-facts",
                    "omena-query.css-modules-resolution-diagnostics",
                ],
                range,
                message: format!(
                    "Selector '.{}' not found in this file for composes.",
                    target_name
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

    diagnostics
}

pub fn summarize_omena_query_css_modules_resolution_style_diagnostics(
    target_style_path: &str,
    target_source: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let style_source_refs = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();
    let style_fact_entries = collect_omena_query_style_fact_entries(style_source_refs.as_slice());
    summarize_omena_query_css_modules_resolution_style_diagnostics_from_entries(
        target_style_path,
        target_source,
        &style_fact_entries,
        package_manifests,
    )
}

/// Substrate-threaded core of the css-modules resolution pass (RFC 0009 Pillar B
/// stage-2, #65). `style_fact_entries` is the substrate's ENTRIES slot; this pass never
/// computed a Sass cross-file resolution (composes targets resolve per-edge via
/// `resolve_style_module_source`).
pub(super) fn summarize_omena_query_css_modules_resolution_style_diagnostics_from_entries(
    target_style_path: &str,
    target_source: &str,
    style_fact_entries: &[OmenaQueryStyleFactEntry],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let available_style_paths = style_fact_entries
        .iter()
        .map(|entry| entry.style_path.as_str())
        .collect::<BTreeSet<_>>();
    let facts_by_path = style_fact_entries
        .iter()
        .map(|entry| (entry.style_path.as_str(), entry.facts.clone()))
        .collect::<BTreeMap<_, _>>();
    let dialect = omena_parser_dialect_for_style_path(target_style_path);
    let target_facts = collect_omena_query_omena_parser_style_facts_raw(target_source, dialect);
    let mut diagnostics = Vec::new();

    for edge in target_facts.css_module_composes_edges {
        if edge.kind == ParsedCssModuleComposesEdgeKind::Global {
            continue;
        }
        let start: u32 = edge.range.start().into();
        let end: u32 = edge.range.end().into();
        let range = parser_range_for_byte_span(
            target_source,
            ParserByteSpanV0 {
                start: start as usize,
                end: end as usize,
            },
        );
        let target_style = if edge.kind == ParsedCssModuleComposesEdgeKind::External {
            let Some(source) = edge.import_source.as_deref() else {
                continue;
            };
            let Some(resolved_style_path) = resolve_style_module_source(
                target_style_path,
                source,
                &available_style_paths,
                package_manifests,
            ) else {
                diagnostics.push(OmenaQueryStyleDiagnosticV0 {
                    code: "missingComposedModule",
                    severity: "warning",
                    provenance: vec![
                        "omena-parser.css-modules-composes-facts",
                        "omena-resolver.style-module-resolution",
                    ],
                    range,
                    message: format!("Cannot resolve composed CSS Module '{}'.", source),
                    tags: Vec::new(),
                    create_custom_property: None,
                    cascade_narrowing: None,
                    cascade_confidence: None,
                    polynomial_provenance: None,
                    cross_file_scc: None,
                });
                continue;
            };
            resolved_style_path
        } else {
            target_style_path.to_string()
        };
        let target_class_names = facts_by_path
            .get(target_style.as_str())
            .map(|facts| facts.class_selector_names.as_slice())
            .unwrap_or(&[])
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();

        for target_name in edge.target_names {
            if target_class_names.contains(target_name.as_str()) {
                continue;
            }
            let message = if let Some(source) = edge.import_source.as_deref() {
                format!(
                    "Selector '.{}' not found in composed module '{}'.",
                    target_name, source
                )
            } else {
                format!(
                    "Selector '.{}' not found in this file for composes.",
                    target_name
                )
            };
            diagnostics.push(OmenaQueryStyleDiagnosticV0 {
                code: "missingComposedSelector",
                severity: "warning",
                provenance: vec![
                    "omena-parser.css-modules-composes-facts",
                    "omena-query.css-modules-resolution-diagnostics",
                ],
                range,
                message,
                tags: Vec::new(),
                create_custom_property: None,
                cascade_narrowing: None,
                cascade_confidence: None,
                polynomial_provenance: None,
                cross_file_scc: None,
            });
        }
    }

    let mut reported_missing_value_modules = BTreeSet::new();
    for edge in target_facts.css_module_value_import_edges {
        let start: u32 = edge.range.start().into();
        let end: u32 = edge.range.end().into();
        let range = parser_range_for_byte_span(
            target_source,
            ParserByteSpanV0 {
                start: start as usize,
                end: end as usize,
            },
        );
        let Some(resolved_style_path) = resolve_style_module_source(
            target_style_path,
            &edge.import_source,
            &available_style_paths,
            package_manifests,
        ) else {
            if reported_missing_value_modules.insert(edge.import_source.clone()) {
                diagnostics.push(OmenaQueryStyleDiagnosticV0 {
                    code: "missingValueModule",
                    severity: "warning",
                    provenance: vec![
                        "omena-parser.css-modules-value-facts",
                        "omena-resolver.style-module-resolution",
                    ],
                    range,
                    message: format!(
                        "Cannot resolve imported @value module '{}'.",
                        edge.import_source
                    ),
                    tags: Vec::new(),
                    create_custom_property: None,
                    cascade_narrowing: None,
                    cascade_confidence: None,
                    polynomial_provenance: None,
                    cross_file_scc: None,
                });
            }
            continue;
        };
        let target_value_names = facts_by_path
            .get(resolved_style_path.as_str())
            .map(|facts| facts.css_module_value_definition_names.as_slice())
            .unwrap_or(&[])
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        if target_value_names.contains(edge.remote_name.as_str()) {
            continue;
        }
        let message = if edge.local_name == edge.remote_name {
            format!(
                "@value '{}' not found in '{}'.",
                edge.remote_name, edge.import_source
            )
        } else {
            format!(
                "@value '{}' not found in '{}' for local binding '{}'.",
                edge.remote_name, edge.import_source, edge.local_name
            )
        };
        diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code: "missingImportedValue",
            severity: "warning",
            provenance: vec![
                "omena-parser.css-modules-value-facts",
                "omena-query.css-modules-resolution-diagnostics",
            ],
            range,
            message,
            tags: Vec::new(),
            create_custom_property: None,
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
        });
    }

    diagnostics
}
