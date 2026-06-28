use super::*;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub type OmenaQueryTsconfigPathMappingV0 = omena_resolver::OmenaResolverTsconfigPathMappingV0;
pub type OmenaQueryBundlerPathAliasMappingV0 =
    omena_resolver::OmenaResolverBundlerPathAliasMappingV0;
pub type OmenaQueryStyleModuleDiskCandidateIdentityV0 =
    omena_resolver::OmenaResolverStyleModuleDiskCandidateIdentityV0;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStyleResolutionInputsV0 {
    pub package_manifests: Vec<OmenaQueryStylePackageManifestV0>,
    pub tsconfig_path_mappings: Vec<OmenaQueryTsconfigPathMappingV0>,
    pub bundler_path_mappings: Vec<OmenaQueryBundlerPathAliasMappingV0>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub disk_style_path_identities: Vec<OmenaQueryStyleModuleDiskCandidateIdentityV0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_sif_cache_fingerprint: Option<String>,
}

pub fn summarize_omena_query_source_import_declarations(
    source: &str,
) -> OmenaQuerySourceImportDeclarationSummaryV0 {
    omena_bridge::summarize_omena_bridge_source_import_declarations(source)
}

pub fn summarize_omena_query_source_import_declarations_for_source_language(
    source_path: &str,
    source: &str,
    source_language: Option<&str>,
) -> OmenaQuerySourceImportDeclarationSummaryV0 {
    omena_bridge::summarize_omena_bridge_source_import_declarations_for_source_language(
        source_path,
        source,
        source_language,
    )
}

pub fn resolve_omena_query_style_uri_for_specifier(
    base_document_uri: &str,
    workspace_folder_uri: Option<&str>,
    specifier: &str,
) -> Option<String> {
    omena_bridge::resolve_omena_bridge_style_uri_for_specifier(
        base_document_uri,
        workspace_folder_uri,
        specifier,
    )
}

pub fn resolve_omena_query_style_uri_for_specifier_with_package_manifests(
    base_document_uri: &str,
    workspace_folder_uri: Option<&str>,
    specifier: &str,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Option<String> {
    let resolver_package_manifests = package_manifests
        .iter()
        .map(|manifest| OmenaResolverStylePackageManifestV0 {
            package_json_path: manifest.package_json_path.clone(),
            package_json_source: manifest.package_json_source.clone(),
        })
        .collect::<Vec<_>>();
    omena_bridge::resolve_omena_bridge_style_uri_for_specifier_with_package_manifests(
        base_document_uri,
        workspace_folder_uri,
        specifier,
        resolver_package_manifests.as_slice(),
    )
}

pub fn resolve_omena_query_style_uri_for_specifier_with_resolution_inputs(
    base_document_uri: &str,
    workspace_folder_uri: Option<&str>,
    specifier: &str,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Option<String> {
    let bridge_inputs = omena_bridge::OmenaBridgeStyleResolutionInputsV0 {
        package_manifests: resolution_inputs
            .package_manifests
            .iter()
            .map(|manifest| OmenaResolverStylePackageManifestV0 {
                package_json_path: manifest.package_json_path.clone(),
                package_json_source: manifest.package_json_source.clone(),
            })
            .collect(),
        tsconfig_path_mappings: resolution_inputs.tsconfig_path_mappings.clone(),
        bundler_path_mappings: resolution_inputs.bundler_path_mappings.clone(),
        disk_style_path_identities: resolution_inputs.disk_style_path_identities.clone(),
    };
    omena_bridge::resolve_omena_bridge_style_uri_for_specifier_with_resolution_inputs(
        base_document_uri,
        workspace_folder_uri,
        specifier,
        &bridge_inputs,
    )
}

pub fn load_omena_query_workspace_style_resolution_inputs(
    workspace_folder_uri: Option<&str>,
    configured_package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> OmenaQueryStyleResolutionInputsV0 {
    let resolver_package_manifests = configured_package_manifests
        .iter()
        .map(|manifest| OmenaResolverStylePackageManifestV0 {
            package_json_path: manifest.package_json_path.clone(),
            package_json_source: manifest.package_json_source.clone(),
        })
        .collect::<Vec<_>>();
    let bridge_inputs = omena_bridge::load_omena_bridge_workspace_style_resolution_inputs(
        workspace_folder_uri,
        resolver_package_manifests.as_slice(),
    );
    OmenaQueryStyleResolutionInputsV0 {
        package_manifests: bridge_inputs
            .package_manifests
            .into_iter()
            .map(|manifest| OmenaQueryStylePackageManifestV0 {
                package_json_path: manifest.package_json_path,
                package_json_source: manifest.package_json_source,
            })
            .collect(),
        tsconfig_path_mappings: bridge_inputs.tsconfig_path_mappings,
        bundler_path_mappings: bridge_inputs.bundler_path_mappings,
        disk_style_path_identities: bridge_inputs.disk_style_path_identities,
        external_sif_cache_fingerprint: None,
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryBridgeExternalSifResolutionV0 {
    pub external_sifs: Vec<OmenaQueryExternalSifInputV0>,
    pub bridge_urls: Vec<String>,
    pub generation_count: usize,
}

pub fn resolve_omena_query_bridge_external_sifs_for_style_sources(
    style_sources: &[OmenaQueryStyleSourceInputV0],
    existing_external_sifs: &[OmenaQueryExternalSifInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> OmenaQueryBridgeExternalSifResolutionV0 {
    let seeds = style_sources
        .iter()
        .flat_map(|source| bridge_external_sif_seeds_for_style_source(source, resolution_inputs))
        .collect::<BTreeSet<_>>();
    resolve_omena_query_bridge_external_sifs_for_seed_pairs(
        seeds.into_iter(),
        existing_external_sifs,
        resolution_inputs,
    )
}

pub fn resolve_omena_query_bridge_external_sifs_for_seed_pairs(
    seeds: impl Iterator<Item = (String, String)>,
    existing_external_sifs: &[OmenaQueryExternalSifInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> OmenaQueryBridgeExternalSifResolutionV0 {
    let mut state =
        BridgeExternalSifResolutionState::new(existing_external_sifs, resolution_inputs);

    for (verbatim_source, resolved_url) in seeds {
        state.enqueue_alias(verbatim_source, resolved_url);
    }

    while let Some(sif) = state.worklist.pop_front() {
        let base_file_uri = sif.canonical_url.clone();
        for forward in &sif.exports.forwards {
            let specifier = forward.canonical_url.as_str();
            if !bridge_external_sif_specifier_is_readable(specifier) {
                continue;
            }
            let Some(child_url) =
                resolve_omena_query_style_uri_for_specifier_with_resolution_inputs(
                    base_file_uri.as_str(),
                    None,
                    specifier,
                    state.resolution_inputs,
                )
                .filter(|uri| uri.starts_with("file://"))
            else {
                continue;
            };
            let alias_key = if specifier.starts_with('.') || specifier.starts_with("file://") {
                child_url.clone()
            } else {
                specifier.to_string()
            };
            state.enqueue_alias(alias_key, child_url);
        }
    }

    state.into_resolution()
}

struct BridgeExternalSifResolutionState<'a> {
    resolution_inputs: &'a OmenaQueryStyleResolutionInputsV0,
    emitted_keys: BTreeSet<String>,
    generated_by_resolved_url: BTreeMap<String, omena_sif::OmenaSifV1>,
    bridge_urls: BTreeSet<String>,
    external_sifs: Vec<OmenaQueryExternalSifInputV0>,
    worklist: VecDeque<omena_sif::OmenaSifV1>,
    generation_count: usize,
}

impl<'a> BridgeExternalSifResolutionState<'a> {
    fn new(
        existing_external_sifs: &[OmenaQueryExternalSifInputV0],
        resolution_inputs: &'a OmenaQueryStyleResolutionInputsV0,
    ) -> Self {
        Self {
            resolution_inputs,
            emitted_keys: existing_external_sifs
                .iter()
                .flat_map(|input| [input.canonical_url.clone(), input.sif.canonical_url.clone()])
                .collect(),
            generated_by_resolved_url: existing_external_sifs
                .iter()
                .map(|input| (input.sif.canonical_url.clone(), input.sif.clone()))
                .collect(),
            bridge_urls: BTreeSet::new(),
            external_sifs: Vec::new(),
            worklist: VecDeque::new(),
            generation_count: 0,
        }
    }

    fn into_resolution(self) -> OmenaQueryBridgeExternalSifResolutionV0 {
        OmenaQueryBridgeExternalSifResolutionV0 {
            external_sifs: self.external_sifs,
            bridge_urls: self.bridge_urls.into_iter().collect(),
            generation_count: self.generation_count,
        }
    }

    fn enqueue_alias(&mut self, alias_key: String, resolved_url: String) {
        if self.emitted_keys.contains(alias_key.as_str()) {
            return;
        }
        self.bridge_urls.insert(alias_key.clone());
        self.bridge_urls.insert(resolved_url.clone());
        if let Some(sif) = self
            .generated_by_resolved_url
            .get(resolved_url.as_str())
            .cloned()
        {
            self.emitted_keys.insert(alias_key.clone());
            self.emitted_keys.insert(sif.canonical_url.clone());
            self.external_sifs.push(OmenaQueryExternalSifInputV0 {
                canonical_url: alias_key,
                sif,
            });
            return;
        }
        let cache_context = omena_bridge::OmenaBridgeExternalSifCacheContextV0 {
            freshness_fingerprint: self
                .resolution_inputs
                .external_sif_cache_fingerprint
                .clone(),
        };
        let Ok(sif) = generate_omena_bridge_sif_for_resolved_style_path_with_cache_context(
            resolved_url.as_str(),
            &cache_context,
        ) else {
            return;
        };
        self.generation_count = self.generation_count.saturating_add(1);
        self.generated_by_resolved_url
            .insert(sif.canonical_url.clone(), sif.clone());
        self.emitted_keys.insert(alias_key.clone());
        self.emitted_keys.insert(sif.canonical_url.clone());
        self.bridge_urls.insert(sif.canonical_url.clone());
        self.worklist.push_back(sif.clone());
        self.external_sifs.push(OmenaQueryExternalSifInputV0 {
            canonical_url: alias_key,
            sif,
        });
    }
}

fn bridge_external_sif_seeds_for_style_source(
    source: &OmenaQueryStyleSourceInputV0,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Vec<(String, String)> {
    let Some(module_sources) =
        summarize_omena_query_sass_module_sources(&source.style_path, &source.style_source)
    else {
        return Vec::new();
    };
    let base_uri = style_source_path_as_file_uri(source.style_path.as_str());
    module_sources
        .module_use_edges
        .iter()
        .map(|edge| edge.source.as_str())
        .chain(
            module_sources
                .module_forward_sources
                .iter()
                .map(String::as_str),
        )
        .filter_map(|specifier| {
            if !bridge_external_sif_specifier_is_readable(specifier) {
                return None;
            }
            let resolved_url = if specifier.starts_with("file://") {
                specifier.to_string()
            } else {
                resolve_omena_query_style_uri_for_specifier_with_resolution_inputs(
                    base_uri.as_str(),
                    None,
                    specifier,
                    resolution_inputs,
                )?
            };
            resolved_url
                .starts_with("file://")
                .then(|| (specifier.to_string(), resolved_url))
        })
        .collect()
}

fn bridge_external_sif_specifier_is_readable(specifier: &str) -> bool {
    !specifier.starts_with("sass:")
        && !specifier.starts_with("http://")
        && !specifier.starts_with("https://")
}

fn style_source_path_as_file_uri(path: &str) -> String {
    if path.starts_with("file://") {
        path.to_string()
    } else {
        format!("file://{path}")
    }
}

pub fn summarize_omena_query_source_syntax_index(
    source: &str,
    imported_style_bindings: Vec<OmenaQuerySourceImportedStyleBindingV0>,
    classnames_bind_bindings: Vec<String>,
) -> OmenaQuerySourceSyntaxIndexV0 {
    omena_bridge::summarize_omena_bridge_source_syntax_index(
        source,
        imported_style_bindings,
        classnames_bind_bindings,
    )
}

pub fn summarize_omena_query_source_syntax_index_for_source_language(
    source_path: &str,
    source: &str,
    source_language: Option<&str>,
    imported_style_bindings: Vec<OmenaQuerySourceImportedStyleBindingV0>,
    classnames_bind_bindings: Vec<String>,
) -> OmenaQuerySourceSyntaxIndexV0 {
    omena_bridge::summarize_omena_bridge_source_syntax_index_for_source_language(
        source_path,
        source,
        source_language,
        imported_style_bindings,
        classnames_bind_bindings,
    )
}

pub fn summarize_omena_query_source_binding_index(
    source: &str,
    imported_style_bindings: Vec<OmenaQuerySourceImportedStyleBindingV0>,
    classnames_bind_bindings: Vec<String>,
) -> OmenaQuerySourceBindingIndexV0 {
    omena_bridge::summarize_omena_bridge_source_binding_index(
        source,
        imported_style_bindings,
        classnames_bind_bindings,
    )
}

pub fn summarize_omena_query_source_binding_index_for_source_language(
    source_path: &str,
    source: &str,
    source_language: Option<&str>,
    imported_style_bindings: Vec<OmenaQuerySourceImportedStyleBindingV0>,
    classnames_bind_bindings: Vec<String>,
) -> OmenaQuerySourceBindingIndexV0 {
    omena_bridge::summarize_omena_bridge_source_binding_index_for_source_language(
        source_path,
        source,
        source_language,
        imported_style_bindings,
        classnames_bind_bindings,
    )
}

pub fn summarize_omena_query_source_control_flow_graph_for_source_language(
    source_path: &str,
    source: &str,
    source_language: Option<&str>,
    variable_name: &str,
    reference_byte_offset: usize,
) -> Option<crate::OmenaQuerySourceControlFlowGraphCaptureV0> {
    omena_bridge::summarize_omena_bridge_source_control_flow_graph_for_source_language(
        source_path,
        source,
        source_language,
        variable_name,
        reference_byte_offset,
    )
}

pub fn summarize_omena_query_source_type_fact_control_flow_graph_for_source_language(
    source_path: &str,
    source: &str,
    source_language: Option<&str>,
    variable_name: &str,
    reference_byte_offset: usize,
) -> Option<crate::OmenaQuerySourceTypeFactControlFlowGraphV0> {
    omena_bridge::summarize_omena_bridge_source_type_fact_control_flow_graph_for_source_language(
        source_path,
        source,
        source_language,
        variable_name,
        reference_byte_offset,
    )
}

pub fn collect_omena_query_vue_style_module_bindings(
    source_path: &str,
    source: &str,
    source_language: Option<&str>,
) -> Vec<String> {
    omena_bridge::collect_omena_bridge_vue_style_module_bindings(
        source_path,
        source,
        source_language,
    )
}

pub fn canonicalize_omena_query_source_selector_references(
    references: &mut Vec<OmenaQuerySourceSelectorReferenceFactV0>,
) {
    omena_bridge::canonicalize_source_selector_references(references);
}
