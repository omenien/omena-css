use std::collections::{BTreeMap, BTreeSet};

use omena_parser::{
    ParsedAnimationFactKind, ParsedCssModuleComposesEdgeKind, ParsedSassModuleEdgeFactKind,
    ParsedSelectorFactKind,
};

use super::cascade_checker::summarize_query_cascade_checker_diagnostics;
use super::diagnostic_suppressions::apply_omena_query_style_diagnostic_suppressions;
use super::parser_facade::collect_omena_query_omena_parser_style_facts_raw;
use super::*;

const LSP_DIAGNOSTIC_TAG_UNNECESSARY: u8 = 1;
const LSP_DIAGNOSTIC_TAG_DEPRECATED: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OmenaQueryExternalModuleModeV0 {
    Ignored,
    Sif,
}

pub fn summarize_omena_query_missing_custom_property_diagnostics(
    style_uri: &str,
    source: &str,
    candidates: &[OmenaQueryStyleHoverCandidateV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let declaration_names = candidates
        .iter()
        .filter(|candidate| candidate.kind == "customPropertyDeclaration")
        .map(|candidate| candidate.name.as_str())
        .collect::<BTreeSet<_>>();
    if declaration_names.is_empty() {
        return Vec::new();
    }

    let insertion_range = end_of_source_range(source);
    candidates
        .iter()
        .filter(|candidate| {
            candidate.kind == "customPropertyReference"
                && !declaration_names.contains(candidate.name.as_str())
        })
        .map(|candidate| OmenaQueryStyleDiagnosticV0 {
            code: "missingCustomProperty",
            severity: "warning",
            provenance: vec![
                "omena-parser.custom-property-facts",
                "omena-query.style-diagnostics",
            ],
            range: candidate.range,
            message: format!(
                "CSS custom property '{}' not found in indexed style tokens.",
                candidate.name
            ),
            tags: Vec::new(),
            create_custom_property: Some(OmenaQueryCreateCustomPropertyActionV0 {
                uri: style_uri.to_string(),
                range: insertion_range,
                new_text: format!("\n\n:root {{\n  {}: ;\n}}\n", candidate.name),
                property_name: candidate.name.clone(),
            }),
        })
        .collect()
}

pub fn summarize_omena_query_cascade_aware_style_diagnostics(
    style_uri: &str,
    source: &str,
    candidates: &[OmenaQueryStyleHoverCandidateV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let declarations_by_name = candidates
        .iter()
        .filter(|candidate| candidate.kind == "customPropertyDeclaration")
        .map(|candidate| (candidate.name.as_str(), candidate.range))
        .collect::<BTreeMap<_, _>>();

    let dialect = omena_parser_dialect_for_style_path(style_uri);
    let mut diagnostics =
        summarize_static_css_custom_property_fixed_point_from_source(source, dialect)
            .entries
            .into_iter()
            .filter(|entry| entry.guaranteed_invalid)
            .filter_map(|entry| {
                declarations_by_name
                    .get(entry.name.as_str())
                    .copied()
                    .map(|range| OmenaQueryStyleDiagnosticV0 {
                        code: "guaranteedInvalidCustomProperty",
                        severity: "warning",
                        provenance: vec![
                            "omena-transform-passes.custom-property-lfp",
                            "omena-query.cascade-aware-diagnostics",
                        ],
                        range,
                        message: format!(
                            "CSS custom property '{}' resolves to the guaranteed-invalid value.",
                            entry.name
                        ),
                        tags: Vec::new(),
                        create_custom_property: None,
                    })
            })
            .collect::<Vec<_>>();

    diagnostics.extend(summarize_query_cascade_checker_diagnostics(
        style_uri, source,
    ));

    diagnostics
}

pub fn summarize_omena_query_missing_keyframes_diagnostics(
    style_uri: &str,
    source: &str,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let dialect = omena_parser_dialect_for_style_path(style_uri);
    let facts = collect_omena_query_omena_parser_style_facts_raw(source, dialect);
    let declared_keyframes = facts
        .animations
        .iter()
        .filter(|animation| animation.kind == ParsedAnimationFactKind::KeyframesDeclaration)
        .map(|animation| animation.name.clone())
        .collect::<BTreeSet<_>>();
    let mut emitted = BTreeSet::new();

    facts
        .animations
        .into_iter()
        .filter(|animation| animation.kind == ParsedAnimationFactKind::AnimationNameReference)
        .filter(|animation| !declared_keyframes.contains(animation.name.as_str()))
        .filter_map(|animation| {
            let start: u32 = animation.range.start().into();
            let end: u32 = animation.range.end().into();
            let byte_span = ParserByteSpanV0 {
                start: start as usize,
                end: end as usize,
            };
            if !emitted.insert((animation.name.clone(), byte_span.start, byte_span.end)) {
                return None;
            }
            Some((animation, parser_range_for_byte_span(source, byte_span)))
        })
        .map(|(animation, range)| OmenaQueryStyleDiagnosticV0 {
            code: "missingKeyframes",
            severity: "warning",
            provenance: vec![
                "omena-parser.animation-facts",
                "omena-query.style-diagnostics",
            ],
            range,
            message: format!("@keyframes '{}' not found in this file.", animation.name),
            tags: Vec::new(),
            create_custom_property: None,
        })
        .collect()
}

pub fn summarize_omena_query_missing_sass_symbol_diagnostics(
    style_uri: &str,
    source: &str,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let dialect = omena_parser_dialect_for_style_path(style_uri);
    let facts = collect_omena_query_omena_parser_style_facts_raw(source, dialect);
    let mut declarations = BTreeSet::<(&'static str, Option<String>, String)>::new();
    let mut emitted = BTreeSet::new();
    let mut diagnostics = Vec::new();

    for symbol in facts.sass_symbols {
        let key = (
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
            provenance: vec![
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
        .map(|edge| {
            let start: u32 = edge.range.start().into();
            let end: u32 = edge.range.end().into();
            OmenaQueryStyleDiagnosticV0 {
                code: "deprecatedSassImport",
                severity: "information",
                provenance: vec![
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
            }
        })
        .collect()
}

pub fn summarize_omena_query_missing_sass_symbol_diagnostics_for_workspace(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let Some(target) = style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)
    else {
        return Vec::new();
    };
    let style_source_refs = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();
    let style_fact_entries = collect_omena_query_style_fact_entries(style_source_refs.as_slice());
    let facts_by_path = style_fact_entries
        .iter()
        .map(|entry| (entry.style_path.as_str(), &entry.facts))
        .collect::<BTreeMap<_, _>>();
    let resolution =
        summarize_sass_module_cross_file_resolution(&style_fact_entries, package_manifests);
    let visible_symbols =
        collect_visible_sass_symbol_keys(target_style_path, &facts_by_path, &resolution);
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
            provenance: vec![
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
        });
    }

    diagnostics
}

pub fn summarize_omena_query_style_diagnostics_for_file(
    style_uri: &str,
    source: &str,
    candidates: &[OmenaQueryStyleHoverCandidateV0],
) -> OmenaQueryStyleDiagnosticsForFileV0 {
    let mut diagnostics =
        summarize_omena_query_missing_custom_property_diagnostics(style_uri, source, candidates);
    diagnostics.extend(summarize_omena_query_cascade_aware_style_diagnostics(
        style_uri, source, candidates,
    ));
    diagnostics.extend(summarize_omena_query_missing_keyframes_diagnostics(
        style_uri, source,
    ));
    diagnostics.extend(summarize_omena_query_sass_import_deprecation_hints(
        style_uri, source,
    ));
    diagnostics.extend(summarize_omena_query_missing_sass_symbol_diagnostics(
        style_uri, source,
    ));
    let mut summary = OmenaQueryStyleDiagnosticsForFileV0 {
        schema_version: "0",
        product: "omena-query.diagnostics-for-file",
        file_uri: style_uri.to_string(),
        file_kind: "style",
        diagnostic_count: diagnostics.len(),
        diagnostics,
        ready_surfaces: vec![
            "missingCustomPropertyDiagnostics",
            "cascadeAwareDiagnostics",
            "missingKeyframesDiagnostics",
            "sassImportDeprecationHints",
            "missingSassSymbolDiagnostics",
        ],
    };
    apply_omena_query_style_diagnostic_suppressions(source, &mut summary);
    summary
}

pub fn summarize_omena_query_style_diagnostics_for_workspace_file(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    classname_transform: Option<&str>,
) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode(
        target_style_path,
        style_sources,
        source_documents,
        package_manifests,
        classname_transform,
        OmenaQueryExternalModuleModeV0::Ignored,
    )
}

pub fn summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    classname_transform: Option<&str>,
    external_mode: OmenaQueryExternalModuleModeV0,
) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
    let target = style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)?;
    let candidates =
        summarize_omena_query_style_hover_candidates(target_style_path, &target.style_source)?;
    let mut summary = summarize_omena_query_style_diagnostics_for_file(
        target_style_path,
        &target.style_source,
        candidates.candidates.as_slice(),
    );
    summary
        .diagnostics
        .retain(|diagnostic| diagnostic.code != "missingSassSymbol");
    summary.diagnostics.extend(
        summarize_omena_query_missing_sass_symbol_diagnostics_for_workspace(
            target_style_path,
            style_sources,
            package_manifests,
        ),
    );
    summary.diagnostics.extend(
        summarize_omena_query_css_modules_resolution_style_diagnostics(
            target_style_path,
            &target.style_source,
            style_sources,
            package_manifests,
        ),
    );
    summary
        .diagnostics
        .extend(summarize_omena_query_unused_selector_style_diagnostics(
            target_style_path,
            &target.style_source,
            style_sources,
            source_documents,
            package_manifests,
            classname_transform,
        ));
    summary.diagnostic_count = summary.diagnostics.len();
    push_omena_query_ready_surface(
        &mut summary.ready_surfaces,
        "cssModulesComposesResolutionDiagnostics",
    );
    push_omena_query_ready_surface(
        &mut summary.ready_surfaces,
        "cssModulesValueResolutionDiagnostics",
    );
    push_omena_query_ready_surface(&mut summary.ready_surfaces, "unusedSelectorDiagnostics");
    push_omena_query_ready_surface(
        &mut summary.ready_surfaces,
        "graphAwareSassSymbolDiagnostics",
    );
    if external_mode == OmenaQueryExternalModuleModeV0::Sif {
        let top_any_external_symbol_ranges =
            collect_omena_query_external_top_any_sass_symbol_ranges(
                target_style_path,
                style_sources,
                package_manifests,
            );
        summary.diagnostics.retain(|diagnostic| {
            diagnostic.code != "missingSassSymbol"
                || !top_any_external_symbol_ranges.contains(&diagnostic.range)
        });
        summary
            .diagnostics
            .extend(summarize_omena_query_external_sif_boundary_diagnostics(
                target_style_path,
                style_sources,
                package_manifests,
            ));
        push_omena_query_ready_surface(
            &mut summary.ready_surfaces,
            "externalSifBoundaryDiagnostics",
        );
    }
    apply_omena_query_style_diagnostic_suppressions(&target.style_source, &mut summary);
    summary.diagnostic_count = summary.diagnostics.len();
    Some(summary)
}

fn collect_omena_query_external_top_any_sass_symbol_ranges(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> BTreeSet<ParserRangeV0> {
    let Some(target) = style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)
    else {
        return BTreeSet::new();
    };
    let style_source_refs = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();
    let style_fact_entries = collect_omena_query_style_fact_entries(style_source_refs.as_slice());
    let resolution =
        summarize_sass_module_cross_file_resolution(&style_fact_entries, package_manifests);
    let top_any_namespaces = resolution
        .edges
        .iter()
        .filter(|edge| edge.from_style_path == target_style_path)
        .filter(|edge| edge.status == "external")
        .filter_map(|edge| match edge.edge_kind {
            "sassUse"
                if edge.namespace_kind == Some("default")
                    || edge.namespace_kind == Some("alias") =>
            {
                edge.namespace.clone().map(Some)
            }
            "sassUse" if edge.namespace_kind == Some("wildcard") => Some(None),
            "sassImport" => Some(None),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    if top_any_namespaces.is_empty() {
        return BTreeSet::new();
    }

    let facts = collect_omena_query_omena_parser_style_facts_raw(
        target.style_source.as_str(),
        omena_parser_dialect_for_style_path(target_style_path),
    );
    facts
        .sass_symbols
        .into_iter()
        .filter(|symbol| omena_query_sass_symbol_fact_kind_is_reference(symbol.kind))
        .filter(|symbol| top_any_namespaces.contains(&symbol.namespace))
        .map(|symbol| {
            let start: u32 = symbol.range.start().into();
            let end: u32 = symbol.range.end().into();
            parser_range_for_byte_span(
                target.style_source.as_str(),
                ParserByteSpanV0 {
                    start: start as usize,
                    end: end as usize,
                },
            )
        })
        .collect()
}

fn summarize_omena_query_external_sif_boundary_diagnostics(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let Some(target) = style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)
    else {
        return Vec::new();
    };
    let style_source_refs = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();
    let style_fact_entries = collect_omena_query_style_fact_entries(style_source_refs.as_slice());
    let resolution =
        summarize_sass_module_cross_file_resolution(&style_fact_entries, package_manifests);
    let external_sources = resolution
        .edges
        .iter()
        .filter(|edge| edge.from_style_path == target_style_path)
        .filter(|edge| edge.status == "external")
        .map(|edge| edge.source.as_str())
        .collect::<BTreeSet<_>>();
    if external_sources.is_empty() {
        return Vec::new();
    }

    let facts = collect_omena_query_omena_parser_style_facts_raw(
        target.style_source.as_str(),
        omena_parser_dialect_for_style_path(target_style_path),
    );
    let mut emitted = BTreeSet::new();
    facts
        .sass_module_edges
        .into_iter()
        .filter(|edge| external_sources.contains(edge.source.as_str()))
        .filter_map(|edge| {
            if !emitted.insert((edge.kind, edge.source.clone())) {
                return None;
            }
            let state = OmenaResolverBoundaryStateV0::missing(
                None,
                "SIF mode requires a local SIF artifact for this external Sass module",
            );
            let start: u32 = edge.range.start().into();
            let end: u32 = edge.range.end().into();
            Some(OmenaQueryStyleDiagnosticV0 {
                code: "missingExternalSif",
                severity: "warning",
                provenance: vec![
                    "omena-resolver.boundary-state",
                    "omena-query.external-sif-boundary-diagnostics",
                ],
                range: parser_range_for_byte_span(
                    target.style_source.as_str(),
                    ParserByteSpanV0 {
                        start: start as usize,
                        end: end as usize,
                    },
                ),
                message: format!(
                    "External Sass module '{}' is {} ({}); generate or provide a SIF artifact, or use --external ignored.",
                    edge.source, state.state_name, state.top_name
                ),
                tags: Vec::new(),
                create_custom_property: None,
            })
        })
        .collect()
}

type SassSymbolKey = (&'static str, Option<String>, String);

fn sass_symbol_key(
    symbol_kind: &'static str,
    namespace: Option<String>,
    name: String,
) -> SassSymbolKey {
    (symbol_kind, namespace, name)
}

fn collect_visible_sass_symbol_keys(
    target_style_path: &str,
    facts_by_path: &BTreeMap<&str, &OmenaQueryOmenaParserStyleFactsV0>,
    resolution: &OmenaQuerySassModuleCrossFileResolutionV0,
) -> BTreeSet<SassSymbolKey> {
    let mut visible = BTreeSet::new();
    if let Some(facts) = facts_by_path.get(target_style_path) {
        visible.extend(
            own_sass_symbol_declaration_keys(facts)
                .into_iter()
                .map(|(symbol_kind, name)| sass_symbol_key(symbol_kind, None, name)),
        );
    }

    for edge in resolution
        .edges
        .iter()
        .filter(|edge| edge.from_style_path == target_style_path)
    {
        let exported = if let Some(module_name) = sass_builtin_module_name(edge.source.as_str()) {
            builtin_sass_symbol_exports(module_name)
        } else if edge.status == "resolved" {
            let mut visiting = BTreeSet::new();
            edge.resolved_style_path
                .as_deref()
                .map(|path| {
                    collect_exported_sass_symbol_keys(
                        path,
                        facts_by_path,
                        resolution,
                        &mut visiting,
                    )
                })
                .unwrap_or_default()
        } else {
            BTreeSet::new()
        };

        match edge.edge_kind {
            "sassUse"
                if edge.namespace_kind == Some("default")
                    || edge.namespace_kind == Some("alias") =>
            {
                if let Some(namespace) = edge.namespace.clone() {
                    visible.extend(exported.into_iter().map(|(symbol_kind, name)| {
                        sass_symbol_key(symbol_kind, Some(namespace.clone()), name)
                    }));
                }
            }
            "sassUse" if edge.namespace_kind == Some("wildcard") => {
                visible.extend(
                    exported
                        .into_iter()
                        .map(|(symbol_kind, name)| sass_symbol_key(symbol_kind, None, name)),
                );
            }
            "sassImport" => {
                visible.extend(
                    exported
                        .into_iter()
                        .map(|(symbol_kind, name)| sass_symbol_key(symbol_kind, None, name)),
                );
            }
            _ => {}
        }
    }

    visible
}

fn collect_exported_sass_symbol_keys(
    style_path: &str,
    facts_by_path: &BTreeMap<&str, &OmenaQueryOmenaParserStyleFactsV0>,
    resolution: &OmenaQuerySassModuleCrossFileResolutionV0,
    visiting: &mut BTreeSet<String>,
) -> BTreeSet<(&'static str, String)> {
    if !visiting.insert(style_path.to_string()) {
        return BTreeSet::new();
    }

    let mut exported = facts_by_path
        .get(style_path)
        .map(|facts| own_sass_symbol_declaration_keys(facts))
        .unwrap_or_default();

    for edge in resolution
        .edges
        .iter()
        .filter(|edge| edge.from_style_path == style_path)
        .filter(|edge| edge.edge_kind == "sassForward" || edge.edge_kind == "sassImport")
    {
        let module_exports =
            if let Some(module_name) = sass_builtin_module_name(edge.source.as_str()) {
                builtin_sass_symbol_exports(module_name)
            } else if edge.status == "resolved" {
                edge.resolved_style_path
                    .as_deref()
                    .map(|path| {
                        collect_exported_sass_symbol_keys(path, facts_by_path, resolution, visiting)
                    })
                    .unwrap_or_default()
            } else {
                BTreeSet::new()
            };

        for (symbol_kind, name) in module_exports {
            if !sass_forward_visibility_allows(edge, symbol_kind, name.as_str()) {
                continue;
            }
            let exported_name = if edge.edge_kind == "sassForward" {
                apply_sass_forward_prefix(edge.forward_prefix.as_deref(), name.as_str())
            } else {
                name
            };
            exported.insert((symbol_kind, exported_name));
        }
    }

    visiting.remove(style_path);
    exported
}

fn own_sass_symbol_declaration_keys(
    facts: &OmenaQueryOmenaParserStyleFactsV0,
) -> BTreeSet<(&'static str, String)> {
    facts
        .sass_symbol_facts
        .iter()
        .filter(|fact| is_omena_query_sass_symbol_declaration_kind(fact.kind))
        .map(|fact| (fact.symbol_kind, fact.name.clone()))
        .collect()
}

fn sass_forward_visibility_allows(
    edge: &OmenaQuerySassModuleEdgeResolutionV0,
    symbol_kind: &'static str,
    name: &str,
) -> bool {
    let prefixed = apply_sass_forward_prefix(edge.forward_prefix.as_deref(), name);
    let matches_filter = |filter_name: &String| {
        filter_name == name
            || filter_name == prefixed.as_str()
            || filter_name.trim_start_matches('$') == name
            || filter_name.trim_start_matches('$') == prefixed.as_str()
            || (symbol_kind != "variable" && filter_name.trim_start_matches('@') == name)
    };
    match edge.visibility_filter_kind {
        Some("show") => edge.visibility_filter_names.iter().any(matches_filter),
        Some("hide") => !edge.visibility_filter_names.iter().any(matches_filter),
        _ => true,
    }
}

fn apply_sass_forward_prefix(prefix: Option<&str>, name: &str) -> String {
    match prefix {
        Some(prefix) if prefix.contains('*') => prefix.replace('*', name),
        Some(prefix) => format!("{prefix}{name}"),
        None => name.to_string(),
    }
}

fn is_omena_query_sass_builtin_symbol_reference_resolved(
    edges: &[omena_parser::ParsedSassModuleEdgeFact],
    symbol_kind: &'static str,
    namespace: Option<&str>,
    name: &str,
) -> bool {
    edges
        .iter()
        .filter(|edge| edge.kind == ParsedSassModuleEdgeFactKind::Use)
        .filter_map(|edge| {
            sass_builtin_module_name(edge.source.as_str()).map(|module| (edge, module))
        })
        .any(|(edge, module)| {
            let namespace_matches =
                match (namespace, edge.namespace_kind, edge.namespace.as_deref()) {
                    (Some(reference_namespace), Some("default" | "alias"), Some(use_namespace)) => {
                        reference_namespace == use_namespace
                    }
                    (None, Some("wildcard"), _) => true,
                    _ => false,
                };
            namespace_matches && sass_builtin_module_has_symbol(module, symbol_kind, name)
        })
}

fn sass_builtin_module_name(source: &str) -> Option<&str> {
    source.strip_prefix("sass:")
}

fn builtin_sass_symbol_exports(module: &str) -> BTreeSet<(&'static str, String)> {
    let mut exports = BTreeSet::new();
    for name in sass_builtin_module_function_names(module) {
        exports.insert(("function", (*name).to_string()));
    }
    for name in sass_builtin_module_mixin_names(module) {
        exports.insert(("mixin", (*name).to_string()));
    }
    for name in sass_builtin_module_variable_names(module) {
        exports.insert(("variable", (*name).to_string()));
    }
    exports
}

fn sass_builtin_module_has_symbol(module: &str, symbol_kind: &'static str, name: &str) -> bool {
    match symbol_kind {
        "function" => sass_builtin_module_function_names(module).contains(&name),
        "mixin" => sass_builtin_module_mixin_names(module).contains(&name),
        "variable" => sass_builtin_module_variable_names(module).contains(&name),
        _ => false,
    }
}

fn sass_builtin_module_function_names(module: &str) -> &'static [&'static str] {
    match module {
        "color" => &[
            "adjust",
            "alpha",
            "blue",
            "channel",
            "change",
            "complement",
            "desaturate",
            "fade-in",
            "fade-out",
            "grayscale",
            "green",
            "hsl",
            "hsla",
            "hue",
            "ie-hex-str",
            "invert",
            "is-legacy",
            "is-missing",
            "is-powerless",
            "lighten",
            "lightness",
            "mix",
            "opacify",
            "opacity",
            "red",
            "same",
            "saturate",
            "saturation",
            "scale",
            "space",
            "to-gamut",
            "to-space",
            "transparentize",
        ],
        "math" => &[
            "abs",
            "acos",
            "asin",
            "atan",
            "atan2",
            "ceil",
            "clamp",
            "compatible",
            "cos",
            "div",
            "floor",
            "hypot",
            "is-unitless",
            "log",
            "max",
            "min",
            "percentage",
            "pow",
            "random",
            "round",
            "sin",
            "sqrt",
            "tan",
            "unit",
        ],
        "list" => &[
            "append",
            "index",
            "is-bracketed",
            "join",
            "length",
            "separator",
            "set-nth",
            "slash",
            "nth",
            "zip",
        ],
        "map" => &[
            "deep-merge",
            "deep-remove",
            "get",
            "has-key",
            "keys",
            "merge",
            "remove",
            "set",
            "values",
        ],
        "string" => &[
            "index",
            "insert",
            "length",
            "quote",
            "slice",
            "split",
            "to-lower-case",
            "to-upper-case",
            "unique-id",
            "unquote",
        ],
        "selector" => &[
            "append",
            "extend",
            "is-superselector",
            "nest",
            "parse",
            "replace",
            "simple-selectors",
            "unify",
        ],
        "meta" => &[
            "accepts-content",
            "calc-args",
            "calc-name",
            "call",
            "content-exists",
            "feature-exists",
            "function-exists",
            "get-function",
            "global-variable-exists",
            "inspect",
            "keywords",
            "mixin-exists",
            "module-functions",
            "module-mixins",
            "module-variables",
            "type-of",
            "variable-exists",
        ],
        _ => &[],
    }
}

fn sass_builtin_module_mixin_names(module: &str) -> &'static [&'static str] {
    match module {
        "meta" => &["load-css"],
        _ => &[],
    }
}

fn sass_builtin_module_variable_names(module: &str) -> &'static [&'static str] {
    match module {
        "math" => &["e", "epsilon", "max-safe-integer", "min-safe-integer", "pi"],
        _ => &[],
    }
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
        });
    }

    diagnostics
}

pub fn summarize_omena_query_unused_selector_style_diagnostics(
    target_style_path: &str,
    target_source: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    classname_transform: Option<&str>,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    if source_documents.is_empty() {
        return Vec::new();
    }

    let style_source_refs = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();
    let style_fact_entries = collect_omena_query_style_fact_entries(style_source_refs.as_slice());
    let available_style_paths = style_fact_entries
        .iter()
        .map(|entry| entry.style_path.as_str())
        .collect::<BTreeSet<_>>();
    let facts_by_path = style_fact_entries
        .iter()
        .map(|entry| (entry.style_path.as_str(), entry.facts.clone()))
        .collect::<BTreeMap<_, _>>();
    let aliases_by_path = collect_classname_transform_aliases(&facts_by_path, classname_transform);
    let (mut used_selectors, unresolved_dynamic_usage) =
        collect_omena_query_source_selector_usage_by_style(
            &available_style_paths,
            source_documents,
            package_manifests,
            &aliases_by_path,
        );
    if unresolved_dynamic_usage.contains(target_style_path) {
        return Vec::new();
    }

    let composes_graph = collect_css_modules_composes_adjacency(
        &facts_by_path,
        &available_style_paths,
        package_manifests,
    );
    propagate_omena_query_composes_usage(&composes_graph, &mut used_selectors);

    let dialect = omena_parser_dialect_for_style_path(target_style_path);
    let target_facts = collect_omena_query_omena_parser_style_facts_raw(target_source, dialect);
    let used_in_target = used_selectors
        .get(target_style_path)
        .cloned()
        .unwrap_or_default();
    let mut emitted = BTreeSet::new();

    target_facts
        .selectors
        .into_iter()
        .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
        .filter(|selector| !used_in_target.contains(selector.name.as_str()))
        .filter_map(|selector| {
            let start: u32 = selector.range.start().into();
            let end: u32 = selector.range.end().into();
            if !emitted.insert(selector.name.clone()) {
                return None;
            }
            Some(OmenaQueryStyleDiagnosticV0 {
                code: "unusedSelector",
                severity: "hint",
                provenance: vec![
                    "omena-parser.selector-facts",
                    "omena-query.source-selector-usage",
                ],
                range: parser_range_for_byte_span(
                    target_source,
                    ParserByteSpanV0 {
                        start: start as usize,
                        end: end as usize,
                    },
                ),
                message: format!("Selector '.{}' is declared but never used.", selector.name),
                tags: vec![LSP_DIAGNOSTIC_TAG_UNNECESSARY],
                create_custom_property: None,
            })
        })
        .collect()
}

fn collect_omena_query_source_selector_usage_by_style(
    available_style_paths: &BTreeSet<&str>,
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    aliases_by_path: &BTreeMap<String, BTreeMap<String, BTreeSet<String>>>,
) -> (BTreeMap<String, BTreeSet<String>>, BTreeSet<String>) {
    let mut used_selectors = BTreeMap::<String, BTreeSet<String>>::new();
    let mut unresolved_dynamic_usage = BTreeSet::<String>::new();

    for document in source_documents {
        let imports = summarize_omena_query_source_import_declarations_for_source_language(
            document.source_path.as_str(),
            &document.source_source,
            None,
        );
        let mut imported_style_bindings = Vec::new();
        let mut classnames_bind_bindings = Vec::new();
        for import in imports.imports {
            if import.specifier == "classnames/bind" {
                classnames_bind_bindings.push(import.binding);
                continue;
            }
            let Some(style_path) = resolve_style_module_source(
                &document.source_path,
                &import.specifier,
                available_style_paths,
                package_manifests,
            ) else {
                continue;
            };
            imported_style_bindings.push(OmenaQuerySourceImportedStyleBindingV0 {
                binding: import.binding,
                style_uri: style_path,
            });
        }
        if imported_style_bindings.is_empty() {
            continue;
        }

        let index = summarize_omena_query_source_syntax_index_for_source_language(
            document.source_path.as_str(),
            &document.source_source,
            None,
            imported_style_bindings,
            classnames_bind_bindings,
        );
        for reference in index.selector_references {
            let Some(target_style_path) = reference.target_style_uri else {
                continue;
            };
            let Some(selector_name) = reference.selector_name.or_else(|| {
                source_reference_text_selector_name(&document.source_source, reference.byte_span)
            }) else {
                unresolved_dynamic_usage.insert(target_style_path);
                continue;
            };
            let used_for_style = used_selectors.entry(target_style_path.clone()).or_default();
            if let Some(canonical_names) = aliases_by_path
                .get(target_style_path.as_str())
                .and_then(|aliases| aliases.get(selector_name.as_str()))
            {
                used_for_style.extend(canonical_names.iter().cloned());
            } else {
                used_for_style.insert(selector_name);
            }
        }
    }

    (used_selectors, unresolved_dynamic_usage)
}

fn collect_classname_transform_aliases(
    facts_by_path: &BTreeMap<&str, OmenaQueryOmenaParserStyleFactsV0>,
    classname_transform: Option<&str>,
) -> BTreeMap<String, BTreeMap<String, BTreeSet<String>>> {
    let mut aliases_by_path = BTreeMap::<String, BTreeMap<String, BTreeSet<String>>>::new();
    for (style_path, facts) in facts_by_path {
        let aliases = aliases_by_path
            .entry((*style_path).to_string())
            .or_default();
        for selector_name in &facts.class_selector_names {
            for alias in classname_transform_aliases(selector_name.as_str(), classname_transform) {
                aliases
                    .entry(alias)
                    .or_default()
                    .insert(selector_name.clone());
            }
        }
    }
    aliases_by_path
}

fn classname_transform_aliases(name: &str, classname_transform: Option<&str>) -> Vec<String> {
    match classname_transform.unwrap_or("asIs") {
        "camelCase" => keep_original_plus_transformed(name, to_ascii_camel_case(name)),
        "camelCaseOnly" => vec![to_ascii_camel_case(name)],
        "dashes" => keep_original_plus_transformed(name, dashes_to_ascii_camel(name)),
        "dashesOnly" => vec![dashes_to_ascii_camel(name)],
        _ => vec![name.to_string()],
    }
}

fn keep_original_plus_transformed(name: &str, transformed: String) -> Vec<String> {
    if transformed == name {
        vec![name.to_string()]
    } else {
        vec![name.to_string(), transformed]
    }
}

fn dashes_to_ascii_camel(name: &str) -> String {
    transform_ascii_separated_name(name, |byte| byte == b'-')
}

fn to_ascii_camel_case(name: &str) -> String {
    transform_ascii_separated_name(name, |byte| byte == b'-' || byte == b'_' || byte == b' ')
}

fn transform_ascii_separated_name(name: &str, is_separator: impl Fn(u8) -> bool) -> String {
    let mut output = String::with_capacity(name.len());
    let mut capitalize_next = false;
    for byte in name.bytes() {
        if is_separator(byte) {
            capitalize_next = true;
            continue;
        }
        if capitalize_next {
            output.push((byte as char).to_ascii_uppercase());
            capitalize_next = false;
            continue;
        }
        output.push(byte as char);
    }
    output
}

fn propagate_omena_query_composes_usage(
    composes_graph: &BTreeMap<CssModulesComposesNode, BTreeSet<CssModulesComposesNode>>,
    used_selectors: &mut BTreeMap<String, BTreeSet<String>>,
) {
    let mut used_nodes = used_selectors
        .iter()
        .flat_map(|(style_path, selectors)| {
            selectors
                .iter()
                .map(|selector_name| CssModulesComposesNode {
                    style_path: style_path.clone(),
                    selector_name: selector_name.clone(),
                })
        })
        .collect::<BTreeSet<_>>();

    let mut changed = true;
    while changed {
        changed = false;
        for (owner, targets) in composes_graph {
            if !used_nodes.contains(owner) {
                continue;
            }
            for target in targets {
                if used_nodes.insert(target.clone()) {
                    used_selectors
                        .entry(target.style_path.clone())
                        .or_default()
                        .insert(target.selector_name.clone());
                    changed = true;
                }
            }
        }
    }
}
