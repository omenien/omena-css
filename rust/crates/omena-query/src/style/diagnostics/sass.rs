use std::collections::{BTreeMap, BTreeSet};

use omena_parser::{
    ParsedExtendTargetFactKind, ParsedSassModuleEdgeFactKind, ParsedSelectorFactKind,
};

use super::super::parser_facade::collect_omena_query_omena_parser_style_facts_raw;
use super::external_sif::{
    OmenaQueryExternalSifResolutionContext,
    summarize_sass_module_cross_file_resolution_with_external_sifs,
};
use super::render::{canonical_sass_module_cycle, render_sass_module_cycle_from};
use super::sass_builtins::is_omena_query_sass_builtin_symbol_reference_resolved;
use super::sass_symbols::{SassSymbolKey, collect_visible_sass_symbol_keys, sass_symbol_key};
use super::shared::*;
use super::substrate::OmenaQueryWorkspaceDiagnosticsSubstrateV0;
use super::substrate::collect_sass_module_graph_reachable_style_paths;
use super::types::LSP_DIAGNOSTIC_TAG_DEPRECATED;

pub fn summarize_omena_query_missing_sass_symbol_diagnostics(
    style_uri: &str,
    source: &str,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let dialect = omena_parser_dialect_for_style_path(style_uri);
    let facts = collect_omena_query_omena_parser_style_facts_raw(source, dialect);
    let mut declarations = BTreeSet::<SassSymbolKey>::new();
    let mut emitted = BTreeSet::new();
    let mut diagnostics = Vec::new();

    for symbol in facts.sass_symbols {
        let key = sass_symbol_key(
            symbol.symbol_kind,
            symbol.namespace.clone(),
            symbol.name.clone(),
        );
        if omena_query_sass_symbol_fact_kind_is_declaration(symbol.kind) {
            declarations.insert(key);
            continue;
        }
        if !omena_query_sass_symbol_fact_kind_is_reference(symbol.kind) {
            continue;
        }
        if declarations.contains(&key) {
            continue;
        }
        if is_omena_query_sass_builtin_symbol_reference_resolved(
            &facts.sass_module_edges,
            symbol.symbol_kind,
            symbol.namespace.as_deref(),
            symbol.name.as_str(),
        ) {
            continue;
        }

        let start: u32 = symbol.range.start().into();
        let end: u32 = symbol.range.end().into();
        let byte_span = ParserByteSpanV0 {
            start: start as usize,
            end: end as usize,
        };
        if !emitted.insert((
            symbol.symbol_kind,
            symbol.namespace.clone(),
            symbol.name.clone(),
            byte_span.start,
            byte_span.end,
        )) {
            continue;
        }
        diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code: "missingSassSymbol",
            severity: "warning",
            provenance: omena_query_evidence_graph_provenance![
                "omena-parser.sass-symbol-facts",
                "omena-query.style-diagnostics",
            ],
            range: parser_range_for_byte_span(source, byte_span),
            message: format!(
                "{} not found in this file.",
                format_query_sass_symbol_label(symbol.symbol_kind, symbol.name.as_str())
            ),
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

pub fn summarize_omena_query_missing_extend_target_diagnostics(
    style_uri: &str,
    source: &str,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let dialect = omena_parser_dialect_for_style_path(style_uri);
    if !matches!(
        dialect,
        OmenaParserStyleDialect::Scss | OmenaParserStyleDialect::Sass
    ) {
        return Vec::new();
    }

    let facts = collect_omena_query_omena_parser_style_facts_raw(source, dialect);
    if facts.extend_targets.is_empty() {
        return Vec::new();
    }

    let mut declared_placeholders = BTreeSet::new();
    let mut declared_classes = BTreeSet::new();
    for selector in &facts.selectors {
        match selector.kind {
            ParsedSelectorFactKind::Placeholder => {
                declared_placeholders.insert(selector.name.clone());
            }
            ParsedSelectorFactKind::Class => {
                declared_classes.insert(selector.name.clone());
            }
            ParsedSelectorFactKind::Id => {}
        }
    }

    let mut emitted = BTreeSet::new();
    let mut diagnostics = Vec::new();
    for target in &facts.extend_targets {
        if target.optional {
            continue;
        }
        let (resolved, label) = match target.kind {
            ParsedExtendTargetFactKind::Placeholder => (
                declared_placeholders.contains(&target.name),
                format!("%{}", target.name),
            ),
            ParsedExtendTargetFactKind::Class => (
                declared_classes.contains(&target.name),
                format!(".{}", target.name),
            ),
        };
        if resolved {
            continue;
        }
        let start: u32 = target.range.start().into();
        let end: u32 = target.range.end().into();
        let byte_span = ParserByteSpanV0 {
            start: start as usize,
            end: end as usize,
        };
        if !emitted.insert((byte_span.start, byte_span.end)) {
            continue;
        }
        diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code: "missingExtendTarget",
            severity: "error",
            provenance: omena_query_evidence_graph_provenance![
                "omena-parser.extend-target-facts",
                "omena-query.missing-extend-target-diagnostics",
            ],
            range: parser_range_for_byte_span(source, byte_span),
            message: format!(
                "@extend target '{label}' does not exist in this file. dart-sass rejects this as a hard error."
            ),
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

pub(super) fn summarize_omena_query_missing_extend_target_diagnostics_for_workspace(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    substrate: &OmenaQueryWorkspaceDiagnosticsSubstrateV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let Some(target) = style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)
    else {
        return Vec::new();
    };
    let dialect = omena_parser_dialect_for_style_path(target_style_path);
    if !matches!(
        dialect,
        OmenaParserStyleDialect::Scss | OmenaParserStyleDialect::Sass
    ) {
        return Vec::new();
    }

    let target_facts =
        collect_omena_query_omena_parser_style_facts_raw(target.style_source.as_str(), dialect);
    if target_facts.extend_targets.is_empty() {
        return Vec::new();
    }

    let resolution = &substrate.sass_resolution_without_manifests;
    let reachable_paths =
        collect_sass_module_graph_reachable_style_paths(target_style_path, resolution);

    let mut declared_placeholders = BTreeSet::new();
    let mut declared_classes = BTreeSet::new();
    for source in style_sources {
        if !reachable_paths.contains(source.style_path.as_str()) {
            continue;
        }
        let facts = collect_omena_query_omena_parser_style_facts_raw(
            source.style_source.as_str(),
            omena_parser_dialect_for_style_path(source.style_path.as_str()),
        );
        for selector in facts.selectors {
            match selector.kind {
                ParsedSelectorFactKind::Placeholder => {
                    declared_placeholders.insert(selector.name);
                }
                ParsedSelectorFactKind::Class => {
                    declared_classes.insert(selector.name);
                }
                ParsedSelectorFactKind::Id => {}
            }
        }
    }

    let mut emitted = BTreeSet::new();
    let mut diagnostics = Vec::new();
    for extend_target in &target_facts.extend_targets {
        if extend_target.optional {
            continue;
        }
        let (resolved, label) = match extend_target.kind {
            ParsedExtendTargetFactKind::Placeholder => (
                declared_placeholders.contains(&extend_target.name),
                format!("%{}", extend_target.name),
            ),
            ParsedExtendTargetFactKind::Class => (
                declared_classes.contains(&extend_target.name),
                format!(".{}", extend_target.name),
            ),
        };
        if resolved {
            continue;
        }
        let start: u32 = extend_target.range.start().into();
        let end: u32 = extend_target.range.end().into();
        let byte_span = ParserByteSpanV0 {
            start: start as usize,
            end: end as usize,
        };
        if !emitted.insert((byte_span.start, byte_span.end)) {
            continue;
        }
        diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code: "missingExtendTarget",
            severity: "error",
            provenance: omena_query_evidence_graph_provenance![
                "omena-parser.extend-target-facts",
                "omena-query.missing-extend-target-diagnostics",
            ],
            range: parser_range_for_byte_span(target.style_source.as_str(), byte_span),
            message: format!(
                "@extend target '{label}' does not exist in the visible Sass module graph. dart-sass rejects this as a hard error."
            ),
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

pub fn summarize_omena_query_sass_import_deprecation_hints(
    style_uri: &str,
    source: &str,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let dialect = omena_parser_dialect_for_style_path(style_uri);
    if !matches!(
        dialect,
        OmenaParserStyleDialect::Scss | OmenaParserStyleDialect::Sass
    ) {
        return Vec::new();
    }

    let facts = collect_omena_query_omena_parser_style_facts_raw(source, dialect);
    facts
        .sass_module_edges
        .into_iter()
        .filter(|edge| edge.kind == ParsedSassModuleEdgeFactKind::Import)
        .filter(|edge| !edge.media_qualified && !sass_import_is_plain_css(edge.source.as_str()))
        .map(|edge| {
            let start: u32 = edge.range.start().into();
            let end: u32 = edge.range.end().into();
            OmenaQueryStyleDiagnosticV0 {
                code: "deprecatedSassImport",
                severity: "information",
                provenance: omena_query_evidence_graph_provenance![
                    "omena-parser.sass-module-edges",
                    "omena-query.sass-import-deprecation-hints",
                ],
                range: parser_range_for_byte_span(
                    source,
                    ParserByteSpanV0 {
                        start: start as usize,
                        end: end as usize,
                    },
                ),
                message: "Sass @import is deprecated; prefer @use or @forward.".to_string(),
                tags: vec![LSP_DIAGNOSTIC_TAG_DEPRECATED],
                create_custom_property: None,
                cascade_narrowing: None,
                cascade_confidence: None,
                polynomial_provenance: None,
                cross_file_scc: None,
            }
        })
        .collect()
}

pub(super) fn sass_import_is_plain_css(source: &str) -> bool {
    let trimmed = source.trim();
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("url(") {
        return true;
    }
    if lower.starts_with("//") || lower.contains("://") {
        return true;
    }
    if lower.ends_with(".css") {
        return true;
    }
    false
}

pub fn summarize_omena_query_missing_sass_symbol_diagnostics_for_workspace(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    summarize_omena_query_missing_sass_symbol_diagnostics_for_workspace_with_sifs(
        target_style_path,
        style_sources,
        package_manifests,
        &[],
        &[],
        &[],
    )
}

fn summarize_omena_query_missing_sass_symbol_diagnostics_for_workspace_with_sifs(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    external_sifs: &[OmenaQueryExternalSifInputV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    if !style_sources
        .iter()
        .any(|source| source.style_path == target_style_path)
    {
        return Vec::new();
    }
    let style_source_refs = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();
    let style_fact_entries = collect_omena_query_style_fact_entries(style_source_refs.as_slice());
    let resolution = summarize_sass_module_cross_file_resolution_with_external_sifs(
        &style_fact_entries,
        package_manifests,
        bundler_path_mappings,
        tsconfig_path_mappings,
        external_sifs,
    );
    summarize_omena_query_missing_sass_symbol_diagnostics_for_workspace_with_sifs_from_substrate(
        target_style_path,
        style_sources,
        package_manifests,
        external_sifs,
        bundler_path_mappings,
        tsconfig_path_mappings,
        &style_fact_entries,
        &resolution,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn summarize_omena_query_missing_sass_symbol_diagnostics_for_workspace_with_sifs_from_substrate(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    external_sifs: &[OmenaQueryExternalSifInputV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
    style_fact_entries: &[OmenaQueryStyleFactEntry],
    resolution: &OmenaQuerySassModuleCrossFileResolutionV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let Some(target) = style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)
    else {
        return Vec::new();
    };
    let facts_by_path = style_fact_entries
        .iter()
        .map(|entry| (entry.style_path.as_str(), &entry.facts))
        .collect::<BTreeMap<_, _>>();
    let external_sif_context = OmenaQueryExternalSifResolutionContext {
        package_manifests,
        bundler_path_mappings,
        tsconfig_path_mappings,
        external_sifs,
    };
    let visible_symbols = collect_visible_sass_symbol_keys(
        target_style_path,
        &facts_by_path,
        resolution,
        external_sif_context,
    );
    let facts = collect_omena_query_omena_parser_style_facts_raw(
        target.style_source.as_str(),
        omena_parser_dialect_for_style_path(target_style_path),
    );
    let mut emitted = BTreeSet::new();
    let mut diagnostics = Vec::new();

    for symbol in facts.sass_symbols {
        if !omena_query_sass_symbol_fact_kind_is_reference(symbol.kind) {
            continue;
        }
        let key = sass_symbol_key(
            symbol.symbol_kind,
            symbol.namespace.clone(),
            symbol.name.clone(),
        );
        if visible_symbols.contains(&key) {
            continue;
        }
        if is_omena_query_sass_builtin_symbol_reference_resolved(
            &facts.sass_module_edges,
            symbol.symbol_kind,
            symbol.namespace.as_deref(),
            symbol.name.as_str(),
        ) {
            continue;
        }

        let start: u32 = symbol.range.start().into();
        let end: u32 = symbol.range.end().into();
        let byte_span = ParserByteSpanV0 {
            start: start as usize,
            end: end as usize,
        };
        if !emitted.insert((
            symbol.symbol_kind,
            symbol.namespace.clone(),
            symbol.name.clone(),
            byte_span.start,
            byte_span.end,
        )) {
            continue;
        }
        diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code: "missingSassSymbol",
            severity: "warning",
            provenance: omena_query_evidence_graph_provenance![
                "omena-parser.sass-symbol-facts",
                "omena-query.graph-aware-sass-diagnostics",
            ],
            range: parser_range_for_byte_span(target.style_source.as_str(), byte_span),
            message: format!(
                "{} not found in the visible Sass module graph.",
                format_query_sass_symbol_label(symbol.symbol_kind, symbol.name.as_str())
            ),
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

pub(super) fn summarize_omena_query_sass_use_cycle_diagnostics_for_workspace(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    substrate: &OmenaQueryWorkspaceDiagnosticsSubstrateV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let Some(target) = style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)
    else {
        return Vec::new();
    };
    let resolution = &substrate.sass_resolution;
    if resolution.cycles.is_empty() {
        return Vec::new();
    }

    let target_facts = collect_omena_query_omena_parser_style_facts_raw(
        target.style_source.as_str(),
        omena_parser_dialect_for_style_path(target_style_path),
    );

    let mut emitted = BTreeSet::new();
    let mut diagnostics = Vec::new();

    for cycle in &resolution.cycles {
        if !cycle.path.iter().any(|node| node == target_style_path) {
            continue;
        }
        let canonical_cycle = canonical_sass_module_cycle(&cycle.path);
        let Some(next_module) = cycle
            .path
            .windows(2)
            .find(|window| window[0] == target_style_path)
            .map(|window| window[1].clone())
        else {
            continue;
        };
        let Some(loop_edge) = resolution.edges.iter().find(|edge| {
            edge.from_style_path == target_style_path
                && edge.resolved_style_path.as_deref() == Some(next_module.as_str())
        }) else {
            continue;
        };
        let Some(fact) = target_facts.sass_module_edges.iter().find(|fact| {
            fact.source == loop_edge.source
                && parsed_sass_module_edge_fact_kind_matches(fact.kind, loop_edge.edge_kind)
        }) else {
            continue;
        };
        let start: u32 = fact.range.start().into();
        let end: u32 = fact.range.end().into();
        let byte_span = ParserByteSpanV0 {
            start: start as usize,
            end: end as usize,
        };
        if !emitted.insert((byte_span.start, byte_span.end, canonical_cycle.clone())) {
            continue;
        }
        diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code: "sassUseCycle",
            severity: "error",
            provenance: omena_query_evidence_graph_provenance![
                "omena-query.sass-module-cross-file-resolution",
                "omena-query.sass-use-cycle-diagnostics",
            ],
            range: parser_range_for_byte_span(target.style_source.as_str(), byte_span),
            message: format!(
                "Sass module loop: {}. dart-sass rejects this as a hard error.",
                render_sass_module_cycle_from(&canonical_cycle, target_style_path)
            ),
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

pub(super) fn summarize_omena_query_unresolved_sass_import_diagnostics_for_workspace(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    substrate: &OmenaQueryWorkspaceDiagnosticsSubstrateV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let Some(target) = style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)
    else {
        return Vec::new();
    };
    let resolution = &substrate.sass_resolution_without_path_mappings;

    let target_facts = collect_omena_query_omena_parser_style_facts_raw(
        target.style_source.as_str(),
        omena_parser_dialect_for_style_path(target_style_path),
    );

    unresolved_sass_import_diagnostics_from_edges(
        target_style_path,
        target.style_source.as_str(),
        &target_facts,
        resolution.edges.iter().filter_map(|edge| {
            (edge.from_style_path == target_style_path).then_some(
                ResolvedSassImportEdgeForDiagnostic {
                    source: edge.source.as_str(),
                    edge_kind: edge.edge_kind,
                    status: edge.status,
                },
            )
        }),
    )
}

pub fn summarize_omena_query_target_unresolved_sass_import_diagnostics_for_workspace_paths(
    target_style_path: &str,
    target_style_source: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let available_style_paths = style_sources
        .iter()
        .map(|source| source.style_path.as_str())
        .collect::<BTreeSet<_>>();
    let load_path_roots = collect_load_path_roots(&available_style_paths);
    let load_path_root_refs = load_path_roots
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let resolver_package_manifests = package_manifests
        .iter()
        .map(|manifest| OmenaResolverStylePackageManifestV0 {
            package_json_path: manifest.package_json_path.clone(),
            package_json_source: manifest.package_json_source.clone(),
        })
        .collect::<Vec<_>>();
    let target_facts = collect_omena_query_omena_parser_style_facts_raw(
        target_style_source,
        omena_parser_dialect_for_style_path(target_style_path),
    );
    let edge_resolutions = target_facts
        .sass_module_edges
        .iter()
        .map(|fact| {
            let edge_kind = parsed_sass_module_edge_fact_kind_label(fact.kind);
            let resolution = summarize_omena_resolver_style_module_resolution_with_load_path_roots(
                target_style_path,
                fact.source.as_str(),
                &available_style_paths,
                &resolver_package_manifests,
                &[],
                &[],
                &load_path_root_refs,
            );
            let status = if resolution.resolution_kind == "externalIgnored" {
                "external"
            } else if resolution.resolved_style_path.is_some() {
                "resolved"
            } else {
                "unresolved"
            };
            ResolvedSassImportEdgeForDiagnostic {
                source: fact.source.as_str(),
                edge_kind,
                status,
            }
        })
        .collect::<Vec<_>>();

    let mut diagnostics = unresolved_sass_import_diagnostics_from_edges(
        target_style_path,
        target_style_source,
        &target_facts,
        edge_resolutions,
    );
    apply_omena_query_checker_product_gate_to_style_diagnostics(&mut diagnostics);
    diagnostics
}

#[derive(Debug, Clone, Copy)]
struct ResolvedSassImportEdgeForDiagnostic<'a> {
    source: &'a str,
    edge_kind: &'a str,
    status: &'a str,
}

fn unresolved_sass_import_diagnostics_from_edges<'a>(
    _target_style_path: &str,
    target_style_source: &str,
    target_facts: &omena_parser::ParsedStyleFacts,
    edges: impl IntoIterator<Item = ResolvedSassImportEdgeForDiagnostic<'a>>,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let mut emitted = BTreeSet::new();
    let mut diagnostics = Vec::new();

    for edge in edges {
        if edge.status != "unresolved" || !sass_module_source_is_workspace_local(edge.source) {
            continue;
        }
        let Some(fact) = target_facts.sass_module_edges.iter().find(|fact| {
            fact.source == edge.source
                && parsed_sass_module_edge_fact_kind_matches(fact.kind, edge.edge_kind)
        }) else {
            continue;
        };
        let start: u32 = fact.range.start().into();
        let end: u32 = fact.range.end().into();
        let byte_span = ParserByteSpanV0 {
            start: start as usize,
            end: end as usize,
        };
        if !emitted.insert((byte_span.start, byte_span.end)) {
            continue;
        }
        diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code: "missingModule",
            severity: "error",
            provenance: omena_query_evidence_graph_provenance![
                "omena-query.sass-module-cross-file-resolution",
                "omena-query.unresolved-sass-import-diagnostics",
            ],
            range: parser_range_for_byte_span(target_style_source, byte_span),
            message: format!(
                "Cannot resolve Sass module '{}'. dart-sass rejects this as a hard error.",
                edge.source
            ),
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

pub(in crate::style) fn sass_module_source_is_workspace_local(source: &str) -> bool {
    let trimmed = source.trim();
    trimmed.starts_with("./") || trimmed.starts_with("../") || trimmed.starts_with('/')
}

fn parsed_sass_module_edge_fact_kind_matches(
    fact_kind: ParsedSassModuleEdgeFactKind,
    edge_kind: &str,
) -> bool {
    matches!(
        (fact_kind, edge_kind),
        (ParsedSassModuleEdgeFactKind::Use, "sassUse")
            | (ParsedSassModuleEdgeFactKind::Forward, "sassForward")
            | (ParsedSassModuleEdgeFactKind::Import, "sassImport")
    )
}

fn parsed_sass_module_edge_fact_kind_label(
    fact_kind: ParsedSassModuleEdgeFactKind,
) -> &'static str {
    match fact_kind {
        ParsedSassModuleEdgeFactKind::Use => "sassUse",
        ParsedSassModuleEdgeFactKind::Forward => "sassForward",
        ParsedSassModuleEdgeFactKind::Import => "sassImport",
    }
}
