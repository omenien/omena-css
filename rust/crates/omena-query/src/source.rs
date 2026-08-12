use super::*;
use omena_cascade::{
    DomClassTokenizationV0, OrderedTokenWordV0, TokenSupportV0, tokenize_dom_class_attribute_v0,
};
use omena_query_core::{
    AbstractClassValueV0, ClassBoundaryEffectV0, ExternalStringTypeFactsV0, FirstWitnessErrorV0,
    GuardAtomV0, GuardedTokenInputV0, GuardedTokenLanguageV0, GuardedTokenMapInputV0,
    GuardedTokenMapV0, StringTypeFactsV2, TokenObserverProjectionV0,
    abstract_class_value_from_facts, abstract_class_value_kind, join_abstract_class_values,
    top_class_value,
};
use omena_syntax::ident::CanonicalClassKeyV0;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub type OmenaQueryTsconfigPathMappingV0 = omena_resolver::OmenaResolverTsconfigPathMappingV0;
pub type OmenaQueryBundlerPathAliasMappingV0 =
    omena_resolver::OmenaResolverBundlerPathAliasMappingV0;
pub type OmenaQueryStyleModuleDiskCandidateIdentityV0 =
    omena_resolver::OmenaResolverStyleModuleDiskCandidateIdentityV0;

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
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
    resolve_omena_query_bridge_external_sifs_for_style_sources_with_optional_cache_storage(
        style_sources,
        existing_external_sifs,
        resolution_inputs,
        None,
    )
}

pub fn resolve_omena_query_bridge_external_sifs_for_style_sources_with_cache_storage(
    style_sources: &[OmenaQueryStyleSourceInputV0],
    existing_external_sifs: &[OmenaQueryExternalSifInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    cache_storage: &omena_bridge::OmenaBridgeExternalSifStorageV0,
) -> OmenaQueryBridgeExternalSifResolutionV0 {
    resolve_omena_query_bridge_external_sifs_for_style_sources_with_optional_cache_storage(
        style_sources,
        existing_external_sifs,
        resolution_inputs,
        Some(cache_storage),
    )
}

fn resolve_omena_query_bridge_external_sifs_for_style_sources_with_optional_cache_storage(
    style_sources: &[OmenaQueryStyleSourceInputV0],
    existing_external_sifs: &[OmenaQueryExternalSifInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    cache_storage: Option<&omena_bridge::OmenaBridgeExternalSifStorageV0>,
) -> OmenaQueryBridgeExternalSifResolutionV0 {
    let seeds = style_sources
        .iter()
        .flat_map(|source| bridge_external_sif_seeds_for_style_source(source, resolution_inputs))
        .collect::<BTreeSet<_>>();
    resolve_omena_query_bridge_external_sifs_for_seed_pairs_with_optional_cache_storage(
        seeds.into_iter(),
        existing_external_sifs,
        resolution_inputs,
        cache_storage,
    )
}

pub fn resolve_omena_query_bridge_external_sifs_for_seed_pairs(
    seeds: impl Iterator<Item = (String, String)>,
    existing_external_sifs: &[OmenaQueryExternalSifInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> OmenaQueryBridgeExternalSifResolutionV0 {
    resolve_omena_query_bridge_external_sifs_for_seed_pairs_with_optional_cache_storage(
        seeds,
        existing_external_sifs,
        resolution_inputs,
        None,
    )
}

pub fn resolve_omena_query_bridge_external_sifs_for_seed_pairs_with_cache_storage(
    seeds: impl Iterator<Item = (String, String)>,
    existing_external_sifs: &[OmenaQueryExternalSifInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    cache_storage: &omena_bridge::OmenaBridgeExternalSifStorageV0,
) -> OmenaQueryBridgeExternalSifResolutionV0 {
    resolve_omena_query_bridge_external_sifs_for_seed_pairs_with_optional_cache_storage(
        seeds,
        existing_external_sifs,
        resolution_inputs,
        Some(cache_storage),
    )
}

fn resolve_omena_query_bridge_external_sifs_for_seed_pairs_with_optional_cache_storage(
    seeds: impl Iterator<Item = (String, String)>,
    existing_external_sifs: &[OmenaQueryExternalSifInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    cache_storage: Option<&omena_bridge::OmenaBridgeExternalSifStorageV0>,
) -> OmenaQueryBridgeExternalSifResolutionV0 {
    let mut state = BridgeExternalSifResolutionState::new(
        existing_external_sifs,
        resolution_inputs,
        cache_storage,
    );

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
    cache_storage: Option<&'a omena_bridge::OmenaBridgeExternalSifStorageV0>,
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
        cache_storage: Option<&'a omena_bridge::OmenaBridgeExternalSifStorageV0>,
    ) -> Self {
        Self {
            resolution_inputs,
            cache_storage,
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
        let Ok(sif) =
            omena_bridge::generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_and_storage(
                resolved_url.as_str(),
                &cache_context,
                self.cache_storage,
            )
        else {
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

pub fn summarize_omena_query_source_syntax_index_with_type_fact_attempts(
    source: &str,
    imported_style_bindings: Vec<OmenaQuerySourceImportedStyleBindingV0>,
    classnames_bind_bindings: Vec<String>,
) -> OmenaQuerySourceSyntaxIndexWithTypeFactAttemptsV0 {
    omena_bridge::summarize_omena_bridge_source_syntax_index_with_type_fact_attempts(
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

pub fn summarize_omena_query_source_syntax_index_for_source_language_with_type_fact_attempts(
    source_path: &str,
    source: &str,
    source_language: Option<&str>,
    imported_style_bindings: Vec<OmenaQuerySourceImportedStyleBindingV0>,
    classnames_bind_bindings: Vec<String>,
) -> OmenaQuerySourceSyntaxIndexWithTypeFactAttemptsV0 {
    omena_bridge::summarize_omena_bridge_source_syntax_index_for_source_language_with_type_fact_attempts(
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySourcePrecisionReferenceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub source_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_language: Option<String>,
    pub variable_name: String,
    pub reference_byte_offset: usize,
    pub resolved_tier: &'static str,
    pub resolved_value: AbstractClassValueV0,
    pub precision: OmenaQueryAnalysisPrecisionV0,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_cause: Option<&'static str>,
}

pub fn resolve_omena_query_source_precision_for_source(
    source_path: &str,
    source: &str,
    source_language: Option<&str>,
    variable_name: &str,
    reference_byte_offset: usize,
) -> OmenaQuerySourcePrecisionReferenceV0 {
    let precision =
        source_diagnostic_precision("classValueResolution", "sourceControlFlow", "sameFile");
    let Some(capture) = summarize_omena_query_source_control_flow_graph_for_source_language(
        source_path,
        source,
        source_language,
        variable_name,
        reference_byte_offset,
    ) else {
        return source_precision_reference(
            source_path,
            source_language,
            variable_name,
            reference_byte_offset,
            top_class_value(),
            precision,
            Some("noFlowCapture"),
        );
    };

    let resolved_flow = resolve_source_precision_flow_from_snapshot(
        &capture.snapshot,
        capture.binding.symbol_ordinal,
    )
    .unwrap_or(ResolvedSourcePrecisionFlowV0 {
        value: top_class_value(),
        top_cause: Some("ambiguousFlowSnapshot"),
    });
    let top_cause = if abstract_class_value_kind(&resolved_flow.value) == "top" {
        resolved_flow.top_cause
    } else {
        None
    };

    source_precision_reference(
        source_path,
        source_language,
        variable_name,
        reference_byte_offset,
        resolved_flow.value,
        precision,
        top_cause,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaQueryClassSitePlaneV0 {
    Cfg,
    TypeFact,
    Joined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaQueryClassSiteUnknownCauseV0 {
    SiteNotEnumerated,
    SourceValueUnavailable,
    NonFiniteRawLanguage,
    TypeFactNotProvided,
    TypeFactRefused,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryClassSiteTypeFactInputV0 {
    pub site_byte_span: ParserByteSpanV0,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facts: Option<StringTypeFactsV2>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal_cause: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryClassSiteTokenProvenanceV0 {
    pub token: OmenaQueryCanonicalClassTokenV0,
    pub must: bool,
    pub may: bool,
    pub planes: Vec<OmenaQueryClassSitePlaneV0>,
    pub boundary_provenance: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub guard_conditions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OmenaQueryCanonicalClassTokenV0(CanonicalClassKeyV0);

impl OmenaQueryCanonicalClassTokenV0 {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Serialize for OmenaQueryCanonicalClassTokenV0 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryClassSiteValueV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub source_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_language: Option<String>,
    pub attribute_name: String,
    pub site_byte_span: ParserByteSpanV0,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_byte_span: Option<ParserByteSpanV0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_value: Option<String>,
    pub target_style_uris: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ordered_word: Option<OrderedTokenWordV0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub support: Option<TokenSupportV0>,
    pub token_provenance: Vec<OmenaQueryClassSiteTokenProvenanceV0>,
    pub contributing_planes: Vec<OmenaQueryClassSitePlaneV0>,
    pub precision_tier: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_cause: Option<OmenaQueryClassSiteUnknownCauseV0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_fact_cause: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClassSitePlaneProjectionV0 {
    ordered_word: Option<OrderedTokenWordV0>,
    support: TokenSupportV0,
    precision_tier: &'static str,
}

pub fn resolve_omena_query_class_site_values_for_source(
    source_path: &str,
    source: &str,
    source_language: Option<&str>,
) -> Vec<OmenaQueryClassSiteValueV0> {
    resolve_omena_query_class_site_values_for_source_with_type_facts(
        source_path,
        source,
        source_language,
        &[],
    )
}

pub fn resolve_omena_query_class_site_values_for_source_with_type_facts(
    source_path: &str,
    source: &str,
    source_language: Option<&str>,
    type_facts: &[OmenaQueryClassSiteTypeFactInputV0],
) -> Vec<OmenaQueryClassSiteValueV0> {
    let imports = summarize_omena_query_source_import_declarations_for_source_language(
        source_path,
        source,
        source_language,
    );
    let base_document_uri = style_source_path_as_file_uri(source_path);
    let imported_style_bindings = imports
        .imports
        .iter()
        .filter(|import| source_specifier_is_style_module(&import.specifier))
        .filter_map(|import| {
            resolve_omena_query_style_uri_for_specifier(&base_document_uri, None, &import.specifier)
                .map(|style_uri| OmenaQuerySourceImportedStyleBindingV0 {
                    binding: import.binding.clone(),
                    style_uri,
                })
        })
        .collect::<Vec<_>>();
    let classnames_bindings = imports
        .imports
        .iter()
        .filter(|import| import.specifier == "classnames/bind")
        .map(|import| import.binding.clone())
        .collect::<Vec<_>>();
    let binding_index = summarize_omena_query_source_binding_index_for_source_language(
        source_path,
        source,
        source_language,
        imported_style_bindings,
        classnames_bindings,
    );
    binding_index
        .class_attribute_sites
        .iter()
        .map(|site| {
            let type_fact = type_facts
                .iter()
                .find(|fact| fact.site_byte_span == site.site_byte_span);
            class_site_value_from_binding_fact(
                source_path,
                source,
                source_language,
                site,
                type_fact,
            )
        })
        .collect()
}

pub fn resolve_omena_query_class_site_value_for_source(
    source_path: &str,
    source: &str,
    source_language: Option<&str>,
    site_byte_span: ParserByteSpanV0,
) -> Option<OmenaQueryClassSiteValueV0> {
    resolve_omena_query_class_site_values_for_source(source_path, source, source_language)
        .into_iter()
        .find(|site| site.site_byte_span == site_byte_span)
}

pub fn build_omena_query_guarded_token_map_for_site(
    site: &OmenaQueryClassSiteValueV0,
) -> Result<GuardedTokenMapV0, FirstWitnessErrorV0> {
    let mut tokens = Vec::new();
    for provenance in &site.token_provenance {
        let token = GuardedTokenLanguageV0::concrete(provenance.token.as_str());
        let observers = TokenObserverProjectionV0::exact(&token);
        if provenance.guard_conditions.is_empty() {
            tokens.push(GuardedTokenInputV0 {
                token,
                guards: Vec::new(),
                observers,
            });
            continue;
        }
        tokens.extend(
            provenance
                .guard_conditions
                .iter()
                .map(|condition| GuardedTokenInputV0 {
                    token: token.clone(),
                    guards: vec![guard_atom_from_source_condition(condition)],
                    observers: observers.clone(),
                }),
        );
    }
    if tokens.is_empty()
        && let Some(raw_language) = site
            .raw_value
            .as_ref()
            .filter(|raw| raw.starts_with('`') && raw.contains("${"))
    {
        let token = GuardedTokenLanguageV0::symbolic(raw_language.clone());
        tokens.push(GuardedTokenInputV0 {
            observers: TokenObserverProjectionV0::exact(&token),
            token,
            guards: Vec::new(),
        });
    }
    GuardedTokenMapV0::build(GuardedTokenMapInputV0 {
        tokens,
        site_usage_guards: Vec::new(),
    })
}

fn guard_atom_from_source_condition(condition: &str) -> GuardAtomV0 {
    let condition = condition.trim();
    let negated = condition
        .strip_prefix("!(")
        .and_then(|condition| condition.strip_suffix(')'));
    GuardAtomV0 {
        atom: negated.unwrap_or(condition).trim().to_string(),
        polarity: negated.is_none(),
    }
}

fn class_site_value_from_binding_fact(
    source_path: &str,
    source: &str,
    source_language: Option<&str>,
    site: &crate::OmenaQuerySourceClassAttributeSiteFactV0,
    type_fact: Option<&OmenaQueryClassSiteTypeFactInputV0>,
) -> OmenaQueryClassSiteValueV0 {
    let cfg_projection = site
        .source_facts
        .as_ref()
        .and_then(|facts| class_site_projection_from_values(facts.values.as_deref()));
    let type_projection = type_fact
        .and_then(|input| input.facts.as_ref())
        .and_then(|facts| class_site_projection_from_values(facts.values.as_deref()));

    let mut must = BTreeSet::new();
    let mut may = BTreeSet::new();
    let mut contributing_planes = Vec::new();
    for (plane, projection) in [
        (OmenaQueryClassSitePlaneV0::Cfg, cfg_projection.as_ref()),
        (
            OmenaQueryClassSitePlaneV0::TypeFact,
            type_projection.as_ref(),
        ),
    ] {
        if let Some(projection) = projection {
            contributing_planes.push(plane);
            must.extend(projection.support.must().iter().cloned());
            may.extend(projection.support.may().iter().cloned());
        }
    }
    let mut guard_conditions_by_token = BTreeMap::<CanonicalClassKeyV0, Vec<String>>::new();
    for guarded in &site.guarded_tokens {
        if let DomClassTokenizationV0::Known { word, .. } =
            tokenize_dom_class_attribute_v0(Some(&guarded.token))
        {
            for token in word.tokens() {
                may.insert(token.clone());
                guard_conditions_by_token
                    .entry(token.clone())
                    .or_default()
                    .push(guarded.condition.clone());
            }
        }
    }
    if !site.guarded_tokens.is_empty()
        && !contributing_planes.contains(&OmenaQueryClassSitePlaneV0::Cfg)
    {
        contributing_planes.push(OmenaQueryClassSitePlaneV0::Cfg);
    }
    if contributing_planes.contains(&OmenaQueryClassSitePlaneV0::Cfg)
        && contributing_planes.contains(&OmenaQueryClassSitePlaneV0::TypeFact)
    {
        contributing_planes.push(OmenaQueryClassSitePlaneV0::Joined);
    }
    let support = (!may.is_empty())
        .then(|| TokenSupportV0::new(must.iter().cloned(), may.iter().cloned()))
        .flatten();
    let ordered_word = match (cfg_projection.as_ref(), type_projection.as_ref()) {
        (Some(cfg), Some(type_fact)) if cfg.ordered_word == type_fact.ordered_word => {
            cfg.ordered_word.clone()
        }
        (Some(cfg), None) => cfg.ordered_word.clone(),
        (None, Some(type_fact)) => type_fact.ordered_word.clone(),
        _ => site.ordered_word.clone(),
    };
    let mut token_provenance = may
        .iter()
        .map(|token| {
            let mut planes = [
                (OmenaQueryClassSitePlaneV0::Cfg, cfg_projection.as_ref()),
                (
                    OmenaQueryClassSitePlaneV0::TypeFact,
                    type_projection.as_ref(),
                ),
            ]
            .into_iter()
            .filter_map(|(plane, projection)| {
                projection
                    .is_some_and(|projection| projection.support.may().contains(token))
                    .then_some(plane)
            })
            .collect::<Vec<_>>();
            if guard_conditions_by_token.contains_key(token)
                && !planes.contains(&OmenaQueryClassSitePlaneV0::Cfg)
            {
                planes.push(OmenaQueryClassSitePlaneV0::Cfg);
            }
            if planes.contains(&OmenaQueryClassSitePlaneV0::Cfg)
                && planes.contains(&OmenaQueryClassSitePlaneV0::TypeFact)
            {
                planes.push(OmenaQueryClassSitePlaneV0::Joined);
            }
            let mut boundary_provenance = Vec::new();
            if planes.contains(&OmenaQueryClassSitePlaneV0::Cfg) {
                boundary_provenance.push(class_boundary_effect_label(site.boundary_effect));
            }
            if guard_conditions_by_token.contains_key(token) {
                boundary_provenance.push("guardedClassToken".to_string());
            }
            OmenaQueryClassSiteTokenProvenanceV0 {
                token: OmenaQueryCanonicalClassTokenV0(token.clone()),
                must: must.contains(token),
                may: true,
                planes,
                boundary_provenance,
                guard_conditions: guard_conditions_by_token
                    .get(token)
                    .cloned()
                    .unwrap_or_default(),
            }
        })
        .collect::<Vec<_>>();
    token_provenance.sort_by(|left, right| left.token.cmp(&right.token));

    let precision_tier = cfg_projection
        .as_ref()
        .map(|projection| projection.precision_tier)
        .or_else(|| {
            type_projection
                .as_ref()
                .map(|projection| projection.precision_tier)
        })
        .or_else(|| (!site.guarded_tokens.is_empty()).then_some("finiteSet"))
        .unwrap_or("top");
    let unknown_cause = if support.is_some() && !may.is_empty() {
        None
    } else if site.source_facts.is_some()
        || type_fact.and_then(|input| input.facts.as_ref()).is_some()
    {
        Some(OmenaQueryClassSiteUnknownCauseV0::NonFiniteRawLanguage)
    } else {
        Some(OmenaQueryClassSiteUnknownCauseV0::SourceValueUnavailable)
    };
    let type_fact_cause = match type_fact {
        None => Some("typeFactNotProvided".to_string()),
        Some(input) if input.facts.is_none() => Some(
            input
                .refusal_cause
                .clone()
                .unwrap_or_else(|| "typeFactRefused".to_string()),
        ),
        Some(_) => None,
    };

    OmenaQueryClassSiteValueV0 {
        schema_version: "0",
        product: "omena-query.class-site-value",
        source_path: source_path.to_string(),
        source_language: source_language.map(str::to_string),
        attribute_name: site.attribute_name.clone(),
        site_byte_span: site.site_byte_span,
        value_byte_span: site.value_byte_span,
        raw_value: site
            .value_byte_span
            .and_then(|span| source.get(span.start..span.end))
            .map(str::to_string),
        target_style_uris: site.target_style_uris.clone(),
        ordered_word,
        support,
        token_provenance,
        contributing_planes,
        precision_tier,
        unknown_cause,
        type_fact_cause,
    }
}

fn class_site_projection_from_values(
    values: Option<&[String]>,
) -> Option<ClassSitePlaneProjectionV0> {
    let values = values?;
    if values.is_empty() {
        return None;
    }
    let mut words = Vec::with_capacity(values.len());
    for value in values {
        let DomClassTokenizationV0::Known { word, .. } =
            tokenize_dom_class_attribute_v0(Some(value))
        else {
            return None;
        };
        words.push(word);
    }
    let may = words
        .iter()
        .flat_map(|word| word.tokens().iter().cloned())
        .collect::<BTreeSet<_>>();
    let mut must = words
        .first()
        .map(|word| word.tokens().iter().cloned().collect::<BTreeSet<_>>())?;
    for word in words.iter().skip(1) {
        let word = word.tokens().iter().cloned().collect::<BTreeSet<_>>();
        must = must.intersection(&word).cloned().collect();
    }
    let ordered_word = words
        .iter()
        .all(|word| word == &words[0])
        .then(|| words[0].clone());
    Some(ClassSitePlaneProjectionV0 {
        ordered_word,
        support: TokenSupportV0::new(must, may)?,
        precision_tier: if values.len() == 1 {
            "exact"
        } else {
            "finiteSet"
        },
    })
}

fn class_boundary_effect_label(effect: ClassBoundaryEffectV0) -> String {
    match effect {
        ClassBoundaryEffectV0::ConcatInsideToken => "concatInsideToken",
        ClassBoundaryEffectV0::ConcatAtTokenBoundary => "concatAtTokenBoundary",
        ClassBoundaryEffectV0::UnknownBoundary => "unknownBoundary",
    }
    .to_string()
}

fn source_specifier_is_style_module(specifier: &str) -> bool {
    [".css", ".scss", ".sass", ".less"]
        .iter()
        .any(|extension| specifier.ends_with(extension))
}

#[derive(Clone, PartialEq, Eq)]
struct ResolvedSourcePrecisionFlowV0 {
    value: AbstractClassValueV0,
    top_cause: Option<&'static str>,
}

fn resolve_source_precision_flow_from_snapshot(
    snapshot: &crate::OmenaQuerySourceFlowBlockGraphSnapshotV0,
    symbol_ordinal: usize,
) -> Option<ResolvedSourcePrecisionFlowV0> {
    let predecessors = source_precision_predecessor_block_ids(&snapshot.blocks);
    let mut states = snapshot
        .blocks
        .iter()
        .map(|block| (block.id.clone(), None::<ResolvedSourcePrecisionFlowV0>))
        .collect::<BTreeMap<_, _>>();

    for _ in 0..std::cmp::max(snapshot.blocks.len() * 2, 1) {
        let mut changed = false;
        for block in &snapshot.blocks {
            let incoming = source_precision_incoming_state(block, &predecessors, &states);
            let next = apply_source_precision_block(block, symbol_ordinal, incoming);
            if states.get(&block.id).and_then(Clone::clone) != next {
                states.insert(block.id.clone(), next);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    let exit = snapshot
        .blocks
        .iter()
        .find(|block| block.id == "exit")
        .or_else(|| snapshot.blocks.last())?;
    states.get(&exit.id).and_then(Clone::clone)
}

fn source_precision_predecessor_block_ids(
    blocks: &[crate::OmenaQuerySourceFlowBlockSnapshotV0],
) -> BTreeMap<String, Vec<String>> {
    let mut predecessors = BTreeMap::<String, Vec<String>>::new();
    for block in blocks {
        for successor in &block.successor_block_ids {
            predecessors
                .entry(successor.clone())
                .or_default()
                .push(block.id.clone());
        }
    }
    predecessors
}

fn source_precision_incoming_state(
    block: &crate::OmenaQuerySourceFlowBlockSnapshotV0,
    predecessors: &BTreeMap<String, Vec<String>>,
    states: &BTreeMap<String, Option<ResolvedSourcePrecisionFlowV0>>,
) -> Option<ResolvedSourcePrecisionFlowV0> {
    predecessors
        .get(&block.id)
        .into_iter()
        .flat_map(|ids| ids.iter())
        .filter_map(|id| states.get(id).and_then(Clone::clone))
        .reduce(join_source_precision_flows)
}

fn apply_source_precision_block(
    block: &crate::OmenaQuerySourceFlowBlockSnapshotV0,
    symbol_ordinal: usize,
    incoming: Option<ResolvedSourcePrecisionFlowV0>,
) -> Option<ResolvedSourcePrecisionFlowV0> {
    if block.symbol_ordinal != Some(symbol_ordinal)
        || !matches!(block.transfer_kind, "assignFacts" | "concatFacts")
    {
        return incoming;
    }

    let Some(facts) = block.facts.as_ref() else {
        return Some(ResolvedSourcePrecisionFlowV0 {
            value: top_class_value(),
            top_cause: Some("missingValueFacts"),
        });
    };

    let external_facts = ExternalStringTypeFactsV0 {
        kind: facts.kind.clone(),
        constraint_kind: facts.constraint_kind.clone(),
        values: facts.values.clone(),
        prefix: facts.prefix.clone(),
        suffix: facts.suffix.clone(),
        min_len: facts.min_len,
        max_len: facts.max_len,
        char_must: facts.char_must.clone(),
        char_may: facts.char_may.clone(),
        may_include_other_chars: facts.may_include_other_chars,
    };

    Some(ResolvedSourcePrecisionFlowV0 {
        value: abstract_class_value_from_facts(&external_facts),
        top_cause: None,
    })
}

fn join_source_precision_flows(
    left: ResolvedSourcePrecisionFlowV0,
    right: ResolvedSourcePrecisionFlowV0,
) -> ResolvedSourcePrecisionFlowV0 {
    let value = join_abstract_class_values(&left.value, &right.value);
    let top_cause = if abstract_class_value_kind(&value) == "top" {
        left.top_cause.or(right.top_cause).or(Some("joinedTop"))
    } else {
        None
    };
    ResolvedSourcePrecisionFlowV0 { value, top_cause }
}

fn source_precision_reference(
    source_path: &str,
    source_language: Option<&str>,
    variable_name: &str,
    reference_byte_offset: usize,
    resolved_value: AbstractClassValueV0,
    precision: OmenaQueryAnalysisPrecisionV0,
    top_cause: Option<&'static str>,
) -> OmenaQuerySourcePrecisionReferenceV0 {
    let resolved_tier = abstract_class_value_kind(&resolved_value);
    OmenaQuerySourcePrecisionReferenceV0 {
        schema_version: "0",
        product: "omena-query.source-precision-reference",
        source_path: source_path.to_string(),
        source_language: source_language.map(str::to_string),
        variable_name: variable_name.to_string(),
        reference_byte_offset,
        resolved_tier,
        resolved_value,
        precision,
        top_cause,
    }
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
