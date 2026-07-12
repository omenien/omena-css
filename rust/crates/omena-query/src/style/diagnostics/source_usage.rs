use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};

use super::shared::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaQueryCssModuleExportUsageStatusV0 {
    Used,
    Unused,
    Skipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaQueryCssModulesUnusedExportSkipReasonV0 {
    NoSourceDocuments,
    UnresolvedImportEdge,
    UnresolvedStyleImport,
    UnresolvedDynamicUsage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCssModuleExportUsageV0 {
    pub module_id: OmenaQueryModuleIdV0,
    pub style_path: String,
    pub export_name: String,
    pub status: OmenaQueryCssModuleExportUsageStatusV0,
    pub precision: FactPrecision,
    pub skip_reasons: Vec<OmenaQueryCssModulesUnusedExportSkipReasonV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCssModulesUnusedExportDiagnosticV0 {
    pub code: &'static str,
    pub severity: &'static str,
    pub module_id: OmenaQueryModuleIdV0,
    pub style_path: String,
    pub export_name: String,
    pub message: String,
    pub precision: FactPrecision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCssModulesUnusedExportSkipReasonCountV0 {
    pub reason: OmenaQueryCssModulesUnusedExportSkipReasonV0,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCssModulesExportUsageReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub export_count: usize,
    pub used_export_count: usize,
    pub unused_export_count: usize,
    pub skipped_export_count: usize,
    pub unresolved_import_edge_count: usize,
    pub exports: Vec<OmenaQueryCssModuleExportUsageV0>,
    pub diagnostics: Vec<OmenaQueryCssModulesUnusedExportDiagnosticV0>,
    pub skip_reason_counts: Vec<OmenaQueryCssModulesUnusedExportSkipReasonCountV0>,
}

pub fn summarize_omena_query_css_modules_export_usage(
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    classname_transform: Option<&str>,
) -> OmenaQueryCssModulesExportUsageReportV0 {
    let style_source_refs = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();
    let style_fact_entries = collect_omena_query_style_fact_entries(style_source_refs.as_slice());
    let resolution =
        summarize_css_modules_cross_file_resolution(&style_fact_entries, package_manifests);
    let shared = collect_omena_query_unused_selector_shared(
        &style_fact_entries,
        source_documents,
        package_manifests,
        classname_transform,
        &[],
        &[],
        &[],
        None,
    );
    let exact_precision =
        omena_query_core::fact_precision_from_analysis_precision(&OmenaQueryAnalysisPrecisionV0 {
            product: "omena-query.analysis-precision".to_string(),
            value_domain: "styleModuleResolution".to_string(),
            flow_sensitivity: "sourceSelectorUsage".to_string(),
            context_sensitivity: "perModuleExport".to_string(),
            revision_axis: "workspaceSnapshot".to_string(),
        });

    let mut exports = Vec::new();
    let mut skip_reason_counts = BTreeMap::new();
    for entry in &style_fact_entries {
        let skip_reasons = export_usage_skip_reasons(
            entry.style_path.as_str(),
            source_documents,
            resolution.unresolved_import_edge_count,
            shared.as_ref(),
        );
        let used_in_module = shared
            .as_ref()
            .and_then(|shared| shared.used_selectors.get(entry.style_path.as_str()));
        for export_name in entry
            .facts
            .class_selector_names
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
        {
            let status = if !skip_reasons.is_empty() {
                OmenaQueryCssModuleExportUsageStatusV0::Skipped
            } else if used_in_module.is_some_and(|used| used.contains(export_name.as_str())) {
                OmenaQueryCssModuleExportUsageStatusV0::Used
            } else {
                OmenaQueryCssModuleExportUsageStatusV0::Unused
            };
            for reason in &skip_reasons {
                *skip_reason_counts.entry(*reason).or_insert(0usize) += 1;
            }
            exports.push(OmenaQueryCssModuleExportUsageV0 {
                module_id: OmenaQueryModuleIdV0::new(entry.style_path.clone()),
                style_path: entry.style_path.clone(),
                export_name,
                status,
                precision: if status == OmenaQueryCssModuleExportUsageStatusV0::Skipped {
                    FactPrecision::Unknown
                } else {
                    exact_precision
                },
                skip_reasons: skip_reasons.clone(),
            });
        }
    }
    exports.sort_by(|left, right| {
        left.style_path
            .cmp(&right.style_path)
            .then_with(|| left.export_name.cmp(&right.export_name))
    });

    let diagnostics = exports
        .iter()
        .filter(|export| export.status == OmenaQueryCssModuleExportUsageStatusV0::Unused)
        .map(|export| OmenaQueryCssModulesUnusedExportDiagnosticV0 {
            code: "unusedModuleExport",
            severity: "hint",
            module_id: export.module_id.clone(),
            style_path: export.style_path.clone(),
            export_name: export.export_name.clone(),
            message: format!(
                "CSS Module export '.{}' is declared but never used.",
                export.export_name
            ),
            precision: export.precision,
        })
        .collect::<Vec<_>>();
    let used_export_count = exports
        .iter()
        .filter(|export| export.status == OmenaQueryCssModuleExportUsageStatusV0::Used)
        .count();
    let skipped_export_count = exports
        .iter()
        .filter(|export| export.status == OmenaQueryCssModuleExportUsageStatusV0::Skipped)
        .count();

    OmenaQueryCssModulesExportUsageReportV0 {
        schema_version: "0",
        product: "omena-query.css-modules-export-usage",
        export_count: exports.len(),
        used_export_count,
        unused_export_count: diagnostics.len(),
        skipped_export_count,
        unresolved_import_edge_count: resolution.unresolved_import_edge_count,
        exports,
        diagnostics,
        skip_reason_counts: skip_reason_counts
            .into_iter()
            .map(
                |(reason, count)| OmenaQueryCssModulesUnusedExportSkipReasonCountV0 {
                    reason,
                    count,
                },
            )
            .collect(),
    }
}

fn export_usage_skip_reasons(
    style_path: &str,
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    unresolved_import_edge_count: usize,
    shared: Option<&OmenaQueryUnusedSelectorSharedV0>,
) -> Vec<OmenaQueryCssModulesUnusedExportSkipReasonV0> {
    let mut reasons = BTreeSet::new();
    if source_documents.is_empty() {
        reasons.insert(OmenaQueryCssModulesUnusedExportSkipReasonV0::NoSourceDocuments);
    }
    if unresolved_import_edge_count > 0 {
        reasons.insert(OmenaQueryCssModulesUnusedExportSkipReasonV0::UnresolvedImportEdge);
    }
    if shared.is_some_and(|shared| shared.has_unresolved_style_import) {
        reasons.insert(OmenaQueryCssModulesUnusedExportSkipReasonV0::UnresolvedStyleImport);
    }
    if shared.is_some_and(|shared| shared.unresolved_dynamic_usage.contains(style_path)) {
        reasons.insert(OmenaQueryCssModulesUnusedExportSkipReasonV0::UnresolvedDynamicUsage);
    }
    reasons.into_iter().collect()
}

pub fn summarize_omena_query_unused_selector_style_diagnostics(
    target_style_path: &str,
    target_source: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    classname_transform: Option<&str>,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    summarize_omena_query_unused_selector_style_diagnostics_with_path_mappings(
        target_style_path,
        target_source,
        style_sources,
        source_documents,
        package_manifests,
        classname_transform,
        &[],
        &[],
    )
}

#[allow(clippy::too_many_arguments)]
pub fn summarize_omena_query_unused_selector_style_diagnostics_with_path_mappings(
    target_style_path: &str,
    target_source: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    classname_transform: Option<&str>,
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    if source_documents.is_empty() {
        return Vec::new();
    }

    let style_source_refs = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();
    let style_fact_entries = collect_omena_query_style_fact_entries(style_source_refs.as_slice());
    summarize_omena_query_unused_selector_style_diagnostics_with_path_mappings_from_entries(
        target_style_path,
        target_source,
        &style_fact_entries,
        source_documents,
        package_manifests,
        classname_transform,
        bundler_path_mappings,
        tsconfig_path_mappings,
        &[],
        None,
    )
}

/// Substrate-threaded core of the unused-selector pass (RFC 0009 Pillar B stage-2,
/// #65). `style_fact_entries` is the substrate's ENTRIES slot; source usage now
/// consumes precomputed source syntax indexes when callers provide them, while
/// retaining the text-backed import/syntax fallback for non-indexed callers.
#[allow(clippy::too_many_arguments)]
/// Target-INDEPENDENT core of the unused-selector pass (rfcs#111 C1 slice
/// 2): source-side import resolution across every source document, selector
/// usage attribution, and composes propagation — the wave computes this ONCE
/// and every target consumes it. Owned types by construction, so the product
/// shares behind an `Arc` without borrowing the substrate.
pub(in crate::style) struct OmenaQueryUnusedSelectorSharedV0 {
    used_selectors: BTreeMap<String, BTreeSet<String>>,
    unresolved_dynamic_usage: BTreeSet<String>,
    has_unresolved_style_import: bool,
}

#[allow(clippy::too_many_arguments)]
pub(in crate::style) fn collect_omena_query_unused_selector_shared(
    style_fact_entries: &[OmenaQueryStyleFactEntry],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    classname_transform: Option<&str>,
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
    disk_style_path_identities: &[OmenaResolverStyleModuleDiskCandidateIdentityV0],
    resolver_identity_index: Option<&OmenaResolverStyleModuleConfirmationIdentityIndexV0>,
) -> Option<OmenaQueryUnusedSelectorSharedV0> {
    if source_documents.is_empty() {
        return None;
    }

    let available_style_paths = style_fact_entries
        .iter()
        .map(|entry| entry.style_path.as_str())
        .collect::<BTreeSet<_>>();
    let facts_by_path = style_fact_entries
        .iter()
        .map(|entry| (entry.style_path.as_str(), entry.facts.clone()))
        .collect::<BTreeMap<_, _>>();
    let aliases_by_path = collect_classname_transform_aliases(&facts_by_path, classname_transform);
    let (mut used_selectors, unresolved_dynamic_usage, has_unresolved_style_import) =
        collect_omena_query_source_selector_usage_by_style(SourceSelectorUsageResolutionContext {
            available_style_paths: &available_style_paths,
            source_documents,
            package_manifests,
            aliases_by_path: &aliases_by_path,
            bundler_path_mappings,
            tsconfig_path_mappings,
            disk_style_path_identities,
            resolver_identity_index,
        });
    let composes_graph = collect_css_modules_composes_adjacency(
        &facts_by_path,
        &available_style_paths,
        package_manifests,
    );
    propagate_omena_query_composes_usage(&composes_graph, &mut used_selectors);
    Some(OmenaQueryUnusedSelectorSharedV0 {
        used_selectors,
        unresolved_dynamic_usage,
        has_unresolved_style_import,
    })
}

/// The per-target remainder: two set lookups plus a parse of the target.
pub(in crate::style) fn summarize_omena_query_unused_selector_style_diagnostics_with_shared(
    target_style_path: &str,
    target_source: &str,
    shared: &OmenaQueryUnusedSelectorSharedV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    if shared.unresolved_dynamic_usage.contains(target_style_path) {
        return Vec::new();
    }
    // RFC-0007-J (#50): when a source document imports a style module via a specifier we cannot
    // resolve (e.g. a workspace alias `@/styles/a.module.scss` with no tsconfig/bundler path
    // mapping wired in), we do not know which module its `cx('foo')`/`styles.foo` references point
    // at — so we cannot prove any selector is unused. References/goto stay lenient with that
    // ambiguity; the negative assertion (`unusedSelector`) must be conservative to match, instead
    // of dimming every selector in the file. Treat such documents as "possibly using" and skip the
    // lint for this target rather than emitting a wall of false positives.
    if shared.has_unresolved_style_import {
        return Vec::new();
    }

    let dialect = omena_parser_dialect_for_style_path(target_style_path);
    let target_facts = collect_omena_query_omena_parser_style_facts_raw(target_source, dialect);
    let used_in_target = shared
        .used_selectors
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
                provenance: omena_query_evidence_graph_provenance![
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
                cascade_narrowing: None,
                cascade_confidence: None,
                polynomial_provenance: None,
                cross_file_scc: None,
            })
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn summarize_omena_query_unused_selector_style_diagnostics_with_path_mappings_from_entries(
    target_style_path: &str,
    target_source: &str,
    style_fact_entries: &[OmenaQueryStyleFactEntry],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    classname_transform: Option<&str>,
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
    disk_style_path_identities: &[OmenaResolverStyleModuleDiskCandidateIdentityV0],
    resolver_identity_index: Option<&OmenaResolverStyleModuleConfirmationIdentityIndexV0>,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let Some(shared) = collect_omena_query_unused_selector_shared(
        style_fact_entries,
        source_documents,
        package_manifests,
        classname_transform,
        bundler_path_mappings,
        tsconfig_path_mappings,
        disk_style_path_identities,
        resolver_identity_index,
    ) else {
        return Vec::new();
    };
    summarize_omena_query_unused_selector_style_diagnostics_with_shared(
        target_style_path,
        target_source,
        &shared,
    )
}

struct SourceSelectorUsageResolutionContext<'a> {
    available_style_paths: &'a BTreeSet<&'a str>,
    source_documents: &'a [OmenaQuerySourceDocumentInputV0],
    package_manifests: &'a [OmenaQueryStylePackageManifestV0],
    aliases_by_path: &'a BTreeMap<String, BTreeMap<String, BTreeSet<String>>>,
    bundler_path_mappings: &'a [OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &'a [OmenaResolverTsconfigPathMappingV0],
    disk_style_path_identities: &'a [OmenaResolverStyleModuleDiskCandidateIdentityV0],
    resolver_identity_index: Option<&'a OmenaResolverStyleModuleConfirmationIdentityIndexV0>,
}

fn collect_omena_query_source_selector_usage_by_style(
    context: SourceSelectorUsageResolutionContext<'_>,
) -> (BTreeMap<String, BTreeSet<String>>, BTreeSet<String>, bool) {
    let mut used_selectors = BTreeMap::<String, BTreeSet<String>>::new();
    let mut unresolved_dynamic_usage = BTreeSet::<String>::new();
    // RFC-0007-J (#50): tracks whether any document imports a style-like specifier we failed to
    // resolve (an unwired workspace alias). Such a document's selector usages cannot be attributed
    // to a concrete module, so the caller treats the file as "possibly used" instead of dimming
    // every selector.
    let mut has_unresolved_style_import = false;

    for document in context.source_documents {
        if let Some(index) = document
            .source_syntax_index
            .as_ref()
            .filter(|index| source_syntax_index_has_style_usage_facts(index))
        {
            let mut index = index.clone();
            crate::canonicalize_omena_query_source_selector_references(
                &mut index.selector_references,
            );
            if document.has_unresolved_style_import {
                has_unresolved_style_import = true;
            }
            collect_omena_query_source_selector_usage_from_syntax_index(
                document,
                &index,
                context.available_style_paths,
                context.aliases_by_path,
                &mut used_selectors,
                &mut unresolved_dynamic_usage,
            );
            continue;
        }

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
            let Some(style_path) =
                resolve_style_module_source_with_path_mappings_and_identity_index(
                    &document.source_path,
                    &import.specifier,
                    context.available_style_paths,
                    context.package_manifests,
                    context.bundler_path_mappings,
                    context.tsconfig_path_mappings,
                    context.disk_style_path_identities,
                    context.resolver_identity_index,
                )
            else {
                if specifier_targets_style_module(&import.specifier) {
                    has_unresolved_style_import = true;
                }
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
            if let Some(canonical_names) = context
                .aliases_by_path
                .get(target_style_path.as_str())
                .and_then(|aliases| aliases.get(selector_name.as_str()))
            {
                used_for_style.extend(canonical_names.iter().cloned());
            } else {
                used_for_style.insert(selector_name);
            }
        }
    }

    (
        used_selectors,
        unresolved_dynamic_usage,
        has_unresolved_style_import,
    )
}

fn collect_omena_query_source_selector_usage_from_syntax_index(
    document: &OmenaQuerySourceDocumentInputV0,
    index: &OmenaQuerySourceSyntaxIndexV0,
    available_style_paths: &BTreeSet<&str>,
    aliases_by_path: &BTreeMap<String, BTreeMap<String, BTreeSet<String>>>,
    used_selectors: &mut BTreeMap<String, BTreeSet<String>>,
    unresolved_dynamic_usage: &mut BTreeSet<String>,
) {
    let single_imported_style_target_uri = (index.imported_style_bindings.len() == 1)
        .then(|| index.imported_style_bindings[0].style_uri.clone());
    for access in &index.style_property_accesses {
        let Some(target_style_path) = access
            .target_style_uri
            .clone()
            .or_else(|| single_imported_style_target_uri.clone())
        else {
            continue;
        };
        let target_style_path =
            source_usage_available_style_path(target_style_path, available_style_paths);
        let Some(selector_name) =
            source_reference_text_selector_name(&document.source_source, access.byte_span)
        else {
            unresolved_dynamic_usage.insert(target_style_path);
            continue;
        };
        record_omena_query_used_source_selector(
            target_style_path,
            selector_name,
            aliases_by_path,
            used_selectors,
        );
    }
    for reference in &index.selector_references {
        let Some(target_style_path) = reference.target_style_uri.clone() else {
            continue;
        };
        let target_style_path =
            source_usage_available_style_path(target_style_path, available_style_paths);
        let Some(selector_name) = reference.selector_name.clone().or_else(|| {
            source_reference_text_selector_name(&document.source_source, reference.byte_span)
        }) else {
            unresolved_dynamic_usage.insert(target_style_path);
            continue;
        };
        record_omena_query_used_source_selector(
            target_style_path,
            selector_name,
            aliases_by_path,
            used_selectors,
        );
    }
}

fn source_syntax_index_has_style_usage_facts(index: &OmenaQuerySourceSyntaxIndexV0) -> bool {
    index
        .style_property_accesses
        .iter()
        .any(|access| access.target_style_uri.is_some())
        || (index.imported_style_bindings.len() == 1 && !index.style_property_accesses.is_empty())
        || index
            .selector_references
            .iter()
            .any(|reference| reference.target_style_uri.is_some())
}

fn source_usage_available_style_path(
    target_style_path: String,
    available_style_paths: &BTreeSet<&str>,
) -> String {
    if available_style_paths.contains(target_style_path.as_str()) {
        return target_style_path;
    }
    available_style_paths
        .iter()
        .find(|available_style_path| {
            source_usage_style_paths_equivalent(target_style_path.as_str(), available_style_path)
        })
        .map(|available_style_path| (*available_style_path).to_string())
        .unwrap_or(target_style_path)
}

fn source_usage_style_paths_equivalent(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    source_usage_style_identity(left) == source_usage_style_identity(right)
}

fn source_usage_style_identity(path_or_uri: &str) -> String {
    let path = if let Some(path) = source_usage_file_uri_path(path_or_uri) {
        PathBuf::from(path)
    } else {
        PathBuf::from(path_or_uri)
    };
    source_usage_normalize_path(
        source_usage_canonicalize_existing_path_or_parent(path.as_path()).unwrap_or(path),
    )
    .to_string_lossy()
    .replace('\\', "/")
}

fn source_usage_file_uri_path(uri: &str) -> Option<String> {
    let path = uri.strip_prefix("file://")?;
    source_usage_percent_decode_uri_path(path)
}

fn source_usage_percent_decode_uri_path(raw_path: &str) -> Option<String> {
    let bytes = raw_path.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = bytes
                .get(index + 1)
                .and_then(|byte| source_usage_hex_value(*byte))?;
            let low = bytes
                .get(index + 2)
                .and_then(|byte| source_usage_hex_value(*byte))?;
            decoded.push((high << 4) | low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

fn source_usage_hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn source_usage_canonicalize_existing_path_or_parent(path: &Path) -> Option<PathBuf> {
    if let Ok(canonical) = fs::canonicalize(path) {
        return Some(canonical);
    }

    let mut current = path.to_path_buf();
    let mut suffix = Vec::<OsString>::new();
    while let Some(parent) = current.parent() {
        if let Some(file_name) = current.file_name() {
            suffix.push(file_name.to_os_string());
        }
        if let Ok(mut canonical_parent) = fs::canonicalize(parent) {
            for segment in suffix.iter().rev() {
                canonical_parent.push(segment);
            }
            return Some(canonical_parent);
        }
        current = parent.to_path_buf();
    }
    None
}

fn source_usage_normalize_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(_) | Component::RootDir | Component::Prefix(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    normalized
}

fn record_omena_query_used_source_selector(
    target_style_path: String,
    selector_name: String,
    aliases_by_path: &BTreeMap<String, BTreeMap<String, BTreeSet<String>>>,
    used_selectors: &mut BTreeMap<String, BTreeSet<String>>,
) {
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

/// Whether an import specifier names a CSS-family style module (so failing to resolve it is a
/// style-resolution gap worth treating conservatively, RFC-0007-J #50) rather than an ordinary
/// JS/TS dependency. A query string or hash on the specifier (e.g. `?inline`) is ignored.
fn specifier_targets_style_module(specifier: &str) -> bool {
    let path = specifier
        .split(['?', '#'])
        .next()
        .unwrap_or(specifier)
        .to_ascii_lowercase();
    path.ends_with(".css")
        || path.ends_with(".scss")
        || path.ends_with(".sass")
        || path.ends_with(".less")
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

#[cfg(test)]
mod export_usage_tests {
    use super::*;

    #[test]
    fn css_modules_interface_export_usage_reprojects_existing_selector_usage() {
        let style_sources = vec![
            OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace/base.module.css".to_string(),
                style_source: ".base {}".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace/app.module.css".to_string(),
                style_source: ".composed { composes: base from \"./base.module.css\"; } .ghost {}"
                    .to_string(),
            },
        ];
        let source_documents = vec![OmenaQuerySourceDocumentInputV0 {
            source_path: "/workspace/App.tsx".to_string(),
            source_source: r#"import styles from "./app.module.css";
export const App = () => <div className={styles.composed} />;"#
                .to_string(),
            source_syntax_index: None,
            has_unresolved_style_import: false,
        }];

        let report = summarize_omena_query_css_modules_export_usage(
            &style_sources,
            &source_documents,
            &[],
            None,
        );

        assert_eq!(report.used_export_count, 2);
        assert_eq!(report.unused_export_count, 1);
        assert_eq!(report.skipped_export_count, 0);
        assert_eq!(report.diagnostics[0].export_name, "ghost");
        assert_eq!(report.diagnostics[0].precision, FactPrecision::Exact);
        assert!(report.exports.iter().any(|export| {
            export.style_path.ends_with("base.module.css")
                && export.export_name == "base"
                && export.status == OmenaQueryCssModuleExportUsageStatusV0::Used
        }));
    }

    #[test]
    fn css_modules_interface_unresolved_edges_skip_unused_export_claims() {
        let style_sources = vec![OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/app.module.css".to_string(),
            style_source: ".button { composes: missing from \"./missing.module.css\"; } .ghost {}"
                .to_string(),
        }];
        let source_documents = vec![OmenaQuerySourceDocumentInputV0 {
            source_path: "/workspace/App.tsx".to_string(),
            source_source: r#"import styles from "./app.module.css";
export const App = () => <div className={styles.button} />;"#
                .to_string(),
            source_syntax_index: None,
            has_unresolved_style_import: false,
        }];

        let report = summarize_omena_query_css_modules_export_usage(
            &style_sources,
            &source_documents,
            &[],
            None,
        );

        assert_eq!(report.unresolved_import_edge_count, 1);
        assert_eq!(report.unused_export_count, 0);
        assert_eq!(report.skipped_export_count, 2);
        assert!(report.diagnostics.is_empty());
        assert!(report.exports.iter().all(|export| {
            export.status == OmenaQueryCssModuleExportUsageStatusV0::Skipped
                && export.precision == FactPrecision::Unknown
                && export
                    .skip_reasons
                    .contains(&OmenaQueryCssModulesUnusedExportSkipReasonV0::UnresolvedImportEdge)
        }));
    }
}
