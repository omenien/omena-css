use std::collections::{BTreeMap, BTreeSet};

use omena_parser::{
    ParsedAnimationFactKind, ParsedCssModuleComposesEdgeKind, ParsedSelectorFactKind,
};

use super::cascade_checker::summarize_query_cascade_checker_diagnostics;
use super::*;

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
            range: candidate.range,
            message: format!(
                "CSS custom property '{}' not found in indexed style tokens.",
                candidate.name
            ),
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
    if declarations_by_name.is_empty() {
        return Vec::new();
    }

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
                        range,
                        message: format!(
                            "CSS custom property '{}' resolves to the guaranteed-invalid value.",
                            entry.name
                        ),
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
    let facts = collect_style_facts(source, dialect);
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
            range,
            message: format!("@keyframes '{}' not found in this file.", animation.name),
            create_custom_property: None,
        })
        .collect()
}

pub fn summarize_omena_query_missing_sass_symbol_diagnostics(
    style_uri: &str,
    source: &str,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let dialect = omena_parser_dialect_for_style_path(style_uri);
    let facts = collect_style_facts(source, dialect);
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
            range: parser_range_for_byte_span(source, byte_span),
            message: format!(
                "{} not found in this file.",
                format_query_sass_symbol_label(symbol.symbol_kind, symbol.name.as_str())
            ),
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
    diagnostics.extend(summarize_omena_query_missing_sass_symbol_diagnostics(
        style_uri, source,
    ));
    OmenaQueryStyleDiagnosticsForFileV0 {
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
            "missingSassSymbolDiagnostics",
        ],
    }
}

pub fn summarize_omena_query_style_diagnostics_for_workspace_file(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
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
    Some(summary)
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
    let target_facts = collect_style_facts(target_source, dialect);
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
                    range,
                    message: format!("Cannot resolve composed CSS Module '{}'.", source),
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
                range,
                message,
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
                    range,
                    message: format!(
                        "Cannot resolve imported @value module '{}'.",
                        edge.import_source
                    ),
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
            range,
            message,
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
    let (mut used_selectors, unresolved_dynamic_usage) =
        collect_omena_query_source_selector_usage_by_style(
            &available_style_paths,
            source_documents,
            package_manifests,
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
    let target_facts = collect_style_facts(target_source, dialect);
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
                range: parser_range_for_byte_span(
                    target_source,
                    ParserByteSpanV0 {
                        start: start as usize,
                        end: end as usize,
                    },
                ),
                message: format!("Selector '.{}' is declared but never used.", selector.name),
                create_custom_property: None,
            })
        })
        .collect()
}

fn collect_omena_query_source_selector_usage_by_style(
    available_style_paths: &BTreeSet<&str>,
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> (BTreeMap<String, BTreeSet<String>>, BTreeSet<String>) {
    let mut used_selectors = BTreeMap::<String, BTreeSet<String>>::new();
    let mut unresolved_dynamic_usage = BTreeSet::<String>::new();

    for document in source_documents {
        let imports = summarize_omena_query_source_import_declarations(&document.source_source);
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

        let index = summarize_omena_query_source_syntax_index(
            &document.source_source,
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
            used_selectors
                .entry(target_style_path)
                .or_default()
                .insert(selector_name);
        }
    }

    (used_selectors, unresolved_dynamic_usage)
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
