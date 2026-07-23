use std::collections::{BTreeMap, BTreeSet};

use super::{
    OmenaQuerySourceDocumentInputV0, OmenaQueryStyleFactEntry, OmenaQueryStylePackageManifestV0,
    SourceSelectorUsageResolutionContext, collect_classname_transform_aliases,
    collect_css_modules_composes_adjacency, collect_omena_query_source_selector_usage_by_style,
    propagate_omena_query_composes_usage,
};
use crate::{
    OmenaResolverBundlerPathAliasMappingV0, OmenaResolverStyleModuleConfirmationIdentityIndexV0,
    OmenaResolverStyleModuleDiskCandidateIdentityV0, OmenaResolverTsconfigPathMappingV0,
};

#[cfg(all(feature = "salsa-memo", any(test, feature = "test-support")))]
thread_local! {
    static UNUSED_SELECTOR_SHARED_WALK_COUNT: std::cell::Cell<u64> =
        const { std::cell::Cell::new(0) };
}

#[cfg(all(feature = "salsa-memo", any(test, feature = "test-support")))]
pub fn reset_unused_selector_shared_walk_count_for_test() {
    UNUSED_SELECTOR_SHARED_WALK_COUNT.with(|count| count.set(0));
}

#[cfg(all(feature = "salsa-memo", any(test, feature = "test-support")))]
pub fn read_unused_selector_shared_walk_count_for_test() -> u64 {
    UNUSED_SELECTOR_SHARED_WALK_COUNT.with(|count| count.get())
}

#[cfg(all(feature = "salsa-memo", any(test, feature = "test-support")))]
fn record_unused_selector_shared_walk_for_test() {
    UNUSED_SELECTOR_SHARED_WALK_COUNT.with(|count| count.set(count.get() + 1));
}

/// Source-side import resolution, selector attribution, and composes
/// propagation shared by every style target in a workspace revision.
///
/// All fields are owned so fixed-revision readers can borrow one computed
/// result without retaining the diagnostics substrate or cloning its maps.
#[derive(Clone, PartialEq, Eq)]
pub(in crate::style) struct OmenaQueryUnusedSelectorSharedV0 {
    pub(super) used_selectors: BTreeMap<String, BTreeSet<String>>,
    pub(super) unresolved_dynamic_usage: BTreeSet<String>,
    pub(super) has_unresolved_style_import: bool,
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
    #[cfg(all(feature = "salsa-memo", any(test, feature = "test-support")))]
    record_unused_selector_shared_walk_for_test();

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
