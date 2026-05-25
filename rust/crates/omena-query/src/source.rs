use super::*;
use serde::Serialize;

pub type OmenaQueryTsconfigPathMappingV0 = omena_resolver::OmenaResolverTsconfigPathMappingV0;
pub type OmenaQueryBundlerPathAliasMappingV0 =
    omena_resolver::OmenaResolverBundlerPathAliasMappingV0;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStyleResolutionInputsV0 {
    pub package_manifests: Vec<OmenaQueryStylePackageManifestV0>,
    pub tsconfig_path_mappings: Vec<OmenaQueryTsconfigPathMappingV0>,
    pub bundler_path_mappings: Vec<OmenaQueryBundlerPathAliasMappingV0>,
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
