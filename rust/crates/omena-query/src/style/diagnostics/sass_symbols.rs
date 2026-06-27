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
    omena_semantic::fold_sass_symbol_name(name)
}

pub(super) fn sass_symbol_key(
    symbol_kind: &'static str,
    namespace: Option<String>,
    name: String,
) -> SassSymbolKey {
    let key = omena_semantic::sass_symbol_key(symbol_kind, namespace, name);
    (key.symbol_kind, key.namespace, key.name)
}

struct QueryVisibleSassSymbolsResolver<'a> {
    facts_by_path: &'a BTreeMap<&'a str, &'a OmenaQueryOmenaParserStyleFactsV0>,
    resolution: &'a OmenaQuerySassModuleCrossFileResolutionV0,
    external_sif_context: OmenaQueryExternalSifResolutionContext<'a>,
}

impl omena_semantic::SassModuleVisibleSymbolsResolverV0 for QueryVisibleSassSymbolsResolver<'_> {
    fn own_symbol_declaration_keys(&self, style_path: &str) -> BTreeSet<(&'static str, String)> {
        self.facts_by_path
            .get(style_path)
            .map(|facts| own_sass_symbol_declaration_keys(facts))
            .unwrap_or_default()
    }

    fn builtin_module_exports(&self, source: &str) -> Option<BTreeSet<(&'static str, String)>> {
        sass_builtin_module_name(source).map(builtin_sass_symbol_exports)
    }

    fn external_module_exports(
        &self,
        edge: &omena_semantic::SassModuleGraphEdgeFactV0,
    ) -> BTreeSet<(&'static str, String)> {
        self.resolution
            .edges
            .iter()
            .find(|candidate| {
                candidate.from_style_path == edge.from_style_path
                    && candidate.edge_kind == edge.edge_kind
                    && candidate.rule_ordinal == edge.rule_ordinal
                    && candidate.source == edge.source
            })
            .and_then(|query_edge| {
                find_omena_query_external_sif_for_edge(query_edge, self.external_sif_context)
            })
            .map(|sif| {
                collect_sif_exported_sass_symbol_keys(
                    &sif.sif,
                    self.external_sif_context.external_sifs,
                )
            })
            .unwrap_or_default()
    }
}

pub(in crate::style) fn collect_visible_sass_symbol_keys(
    target_style_path: &str,
    facts_by_path: &BTreeMap<&str, &OmenaQueryOmenaParserStyleFactsV0>,
    resolution: &OmenaQuerySassModuleCrossFileResolutionV0,
    external_sif_context: OmenaQueryExternalSifResolutionContext<'_>,
) -> BTreeSet<SassSymbolKey> {
    let semantic_edges = semantic_sass_module_edges_from_query_resolution(resolution);
    let resolver = QueryVisibleSassSymbolsResolver {
        facts_by_path,
        resolution,
        external_sif_context,
    };
    omena_semantic::collect_visible_sass_symbol_keys(
        target_style_path,
        semantic_edges.as_slice(),
        &resolver,
    )
    .into_iter()
    .map(|key| (key.symbol_kind, key.namespace, key.name))
    .collect()
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

pub(super) fn sass_forward_filter_name_matches_symbol(
    filter_name: &str,
    prefix: Option<&str>,
    symbol_kind: &'static str,
    name: &str,
) -> bool {
    omena_semantic::sass_forward_filter_name_matches_symbol(filter_name, prefix, symbol_kind, name)
}

pub(super) fn apply_sass_forward_prefix(prefix: Option<&str>, name: &str) -> String {
    omena_semantic::apply_sass_forward_prefix(prefix, name)
}

fn semantic_sass_module_edges_from_query_resolution(
    resolution: &OmenaQuerySassModuleCrossFileResolutionV0,
) -> Vec<omena_semantic::SassModuleGraphEdgeFactV0> {
    resolution
        .edges
        .iter()
        .map(|edge| omena_semantic::SassModuleGraphEdgeFactV0 {
            from_style_path: edge.from_style_path.clone(),
            edge_kind: edge.edge_kind,
            source: edge.source.clone(),
            rule_ordinal: edge.rule_ordinal,
            namespace_kind: edge.namespace_kind,
            namespace: edge.namespace.clone(),
            forward_prefix: edge.forward_prefix.clone(),
            visibility_filter_kind: edge.visibility_filter_kind,
            visibility_filter_names: edge.visibility_filter_names.clone(),
            resolved_style_path: edge.resolved_style_path.clone(),
            status: edge.status,
            configuration_signature: edge.configuration_signature.clone(),
            configuration_variable_count: edge.configuration_variable_count,
            invalid_configuration_variable_names: edge.invalid_configuration_variable_names.clone(),
            module_instance_identity_key: edge.module_instance_identity_key.clone(),
        })
        .collect()
}
