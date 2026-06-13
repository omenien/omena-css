use std::collections::{BTreeMap, BTreeSet};

#[cfg(test)]
use super::external_sif::summarize_sass_module_cross_file_resolution_with_external_sifs;
use super::external_sif::{
    OmenaQueryExternalSifResolutionContext, collect_sif_exported_sass_symbol_keys,
    find_omena_query_external_sif_for_edge,
};
use super::sass_builtins::{builtin_sass_symbol_exports, sass_builtin_module_name};
use super::shared::*;

pub(crate) type SassSymbolKey = (&'static str, Option<String>, String);

/// Fold the Sass-identifier name component so `_` and `-` compare equal.
///
/// Sass treats `$a-b` and `$a_b` (and likewise mixin/function names) as the same
/// identifier, so symbol keys must canonicalize the name before lookup. CSS
/// custom properties never flow through this key space, so they are untouched.
pub(super) fn fold_sass_symbol_name(name: &str) -> String {
    name.replace('_', "-")
}

pub(super) fn sass_symbol_key(
    symbol_kind: &'static str,
    namespace: Option<String>,
    name: String,
) -> SassSymbolKey {
    let folded = fold_sass_symbol_name(&name);
    (symbol_kind, namespace, folded)
}

pub(in crate::style) fn collect_visible_sass_symbol_keys(
    target_style_path: &str,
    facts_by_path: &BTreeMap<&str, &OmenaQueryOmenaParserStyleFactsV0>,
    resolution: &OmenaQuerySassModuleCrossFileResolutionV0,
    external_sif_context: OmenaQueryExternalSifResolutionContext<'_>,
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
                        external_sif_context,
                        &mut visiting,
                    )
                })
                .unwrap_or_default()
        } else if edge.status == "external" {
            find_omena_query_external_sif_for_edge(edge, external_sif_context)
                .map(|sif| {
                    collect_sif_exported_sass_symbol_keys(
                        &sif.sif,
                        external_sif_context.external_sifs,
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

#[cfg(test)]
pub(crate) fn collect_omena_query_visible_sass_symbol_keys_for_workspace_file(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    external_sifs: &[OmenaQueryExternalSifInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> BTreeSet<SassSymbolKey> {
    let style_source_refs = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();
    let style_fact_entries = collect_omena_query_style_fact_entries(style_source_refs.as_slice());
    let facts_by_path = style_fact_entries
        .iter()
        .map(|entry| (entry.style_path.as_str(), &entry.facts))
        .collect::<BTreeMap<_, _>>();
    let resolution = summarize_sass_module_cross_file_resolution_with_external_sifs(
        &style_fact_entries,
        package_manifests,
        resolution_inputs.bundler_path_mappings.as_slice(),
        resolution_inputs.tsconfig_path_mappings.as_slice(),
        external_sifs,
    );
    collect_visible_sass_symbol_keys(
        target_style_path,
        &facts_by_path,
        &resolution,
        OmenaQueryExternalSifResolutionContext {
            package_manifests,
            bundler_path_mappings: resolution_inputs.bundler_path_mappings.as_slice(),
            tsconfig_path_mappings: resolution_inputs.tsconfig_path_mappings.as_slice(),
            external_sifs,
        },
    )
}

fn collect_exported_sass_symbol_keys(
    style_path: &str,
    facts_by_path: &BTreeMap<&str, &OmenaQueryOmenaParserStyleFactsV0>,
    resolution: &OmenaQuerySassModuleCrossFileResolutionV0,
    external_sif_context: OmenaQueryExternalSifResolutionContext<'_>,
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
                        collect_exported_sass_symbol_keys(
                            path,
                            facts_by_path,
                            resolution,
                            external_sif_context,
                            visiting,
                        )
                    })
                    .unwrap_or_default()
            } else if edge.status == "external" {
                find_omena_query_external_sif_for_edge(edge, external_sif_context)
                    .map(|sif| {
                        collect_sif_exported_sass_symbol_keys(
                            &sif.sif,
                            external_sif_context.external_sifs,
                        )
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
    let matches_filter = |filter_name: &String| {
        sass_forward_filter_name_matches_symbol(
            filter_name,
            edge.forward_prefix.as_deref(),
            symbol_kind,
            name,
        )
    };
    match edge.visibility_filter_kind {
        Some("show") => edge.visibility_filter_names.iter().any(matches_filter),
        Some("hide") => !edge.visibility_filter_names.iter().any(matches_filter),
        _ => true,
    }
}

pub(super) fn sass_forward_filter_name_matches_symbol(
    filter_name: &str,
    prefix: Option<&str>,
    symbol_kind: &'static str,
    name: &str,
) -> bool {
    let exposed_name = apply_sass_forward_prefix(prefix, name);
    filter_name == exposed_name
        || filter_name.trim_start_matches('$') == exposed_name
        || (symbol_kind != "variable" && filter_name.trim_start_matches('@') == exposed_name)
}

pub(super) fn apply_sass_forward_prefix(prefix: Option<&str>, name: &str) -> String {
    match prefix {
        Some(prefix) if prefix.contains('*') => prefix.replace('*', name),
        Some(prefix) => format!("{prefix}{name}"),
        None => name.to_string(),
    }
}
