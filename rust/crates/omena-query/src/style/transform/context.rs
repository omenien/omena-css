use super::css_modules::{
    derive_class_name_rewrites_for_transform_context,
    derive_css_module_composes_resolutions_for_transform_context,
    derive_css_module_value_resolutions_for_transform_context,
};
use super::design_tokens::derive_design_token_routes_for_transform_context;
use super::imports::derive_import_inlines_for_transform_context;
use super::static_stylesheet::{
    derive_static_scss_module_use_evaluations_for_transform_context,
    derive_static_stylesheet_module_evaluation_for_transform_context,
};
use super::*;
use crate::types::{
    OmenaQueryEngineInputModuleAttributionV0, normalize_omena_query_style_path,
    resolve_omena_query_style_path_against_known,
};
use omena_syntax::ident::ClassNameV0;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy)]
pub(super) struct TransformResolutionContext<'a> {
    pub(super) package_manifests: &'a [OmenaQueryStylePackageManifestV0],
    pub(super) bundler_path_mappings: &'a [OmenaResolverBundlerPathAliasMappingV0],
    pub(super) tsconfig_path_mappings: &'a [OmenaResolverTsconfigPathMappingV0],
    pub(super) disk_style_path_identities: &'a [OmenaResolverStyleModuleDiskCandidateIdentityV0],
}

impl<'a> TransformResolutionContext<'a> {
    pub(super) fn from_resolution_inputs(
        resolution_inputs: &'a OmenaQueryStyleResolutionInputsV0,
    ) -> Self {
        Self {
            package_manifests: resolution_inputs.package_manifests.as_slice(),
            bundler_path_mappings: resolution_inputs.bundler_path_mappings.as_slice(),
            tsconfig_path_mappings: resolution_inputs.tsconfig_path_mappings.as_slice(),
            disk_style_path_identities: resolution_inputs.disk_style_path_identities.as_slice(),
        }
    }

    pub(super) fn resolve_style_module_source(
        self,
        from_style_path: &str,
        source: &str,
        available_style_paths: &BTreeSet<&str>,
    ) -> Option<String> {
        self.resolve_style_module(from_style_path, source, available_style_paths)
            .resolved_style_path
    }

    pub(super) fn resolve_style_module(
        self,
        from_style_path: &str,
        source: &str,
        available_style_paths: &BTreeSet<&str>,
    ) -> OmenaResolverStyleModuleResolutionV0 {
        let load_path_roots = super::super::collect_load_path_roots(available_style_paths);
        let load_path_root_refs = load_path_roots
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let resolver_package_manifests = self
            .package_manifests
            .iter()
            .map(|manifest| OmenaResolverStylePackageManifestV0 {
                package_json_path: manifest.package_json_path.clone(),
                package_json_source: manifest.package_json_source.clone(),
            })
            .collect::<Vec<_>>();
        summarize_omena_resolver_style_module_resolution_with_confirmation_inputs(
            from_style_path,
            source,
            available_style_paths,
            self.disk_style_path_identities,
            &resolver_package_manifests,
            self.bundler_path_mappings,
            self.tsconfig_path_mappings,
            &load_path_root_refs,
            OmenaResolverStyleModuleConfirmationOptionsV0 {
                allow_disk_confirmation: true,
                ..OmenaResolverStyleModuleConfirmationOptionsV0::default()
            },
        )
    }
}

pub(super) fn merge_transform_context(
    mut merged: TransformExecutionContextV0,
    context: &TransformExecutionContextV0,
) -> TransformExecutionContextV0 {
    merged.drop_dark_mode_media_queries =
        merged.drop_dark_mode_media_queries || context.drop_dark_mode_media_queries;
    if context.vendor_prefix_policy.is_some() {
        merged.vendor_prefix_policy = context.vendor_prefix_policy;
    }
    if context.supports_target_capability.is_some() {
        merged.supports_target_capability = context.supports_target_capability;
    }
    merge_context_list(
        &mut merged.reachable_class_names,
        &context.reachable_class_names,
    );
    merge_context_list(
        &mut merged.reachable_keyframe_names,
        &context.reachable_keyframe_names,
    );
    merge_context_list(
        &mut merged.reachable_value_names,
        &context.reachable_value_names,
    );
    merge_context_list(
        &mut merged.reachable_custom_property_names,
        &context.reachable_custom_property_names,
    );

    if context.scss_module_evaluation.is_some() {
        merged.scss_module_evaluation = context.scss_module_evaluation.clone();
    }
    if context.less_module_evaluation.is_some() {
        merged.less_module_evaluation = context.less_module_evaluation.clone();
    }
    if !context.import_inlines.is_empty() {
        merge_context_records_by_key(
            &mut merged.import_inlines,
            &context.import_inlines,
            |inline| inline.import_source.as_str(),
        );
    }
    if !context.class_name_rewrites.is_empty() {
        merge_class_context_records_by_key(
            &mut merged.class_name_rewrites,
            &context.class_name_rewrites,
            |rewrite| rewrite.original_name.as_str(),
        );
    }
    if !context.css_module_composes_resolutions.is_empty() {
        merge_class_context_records_by_key(
            &mut merged.css_module_composes_resolutions,
            &context.css_module_composes_resolutions,
            |resolution| resolution.local_class_name.as_str(),
        );
    }
    if !context.css_module_value_resolutions.is_empty() {
        merge_context_records_by_key(
            &mut merged.css_module_value_resolutions,
            &context.css_module_value_resolutions,
            |resolution| resolution.local_name.as_str(),
        );
    }
    if !context.design_token_routes.is_empty() {
        merge_context_records_by_key(
            &mut merged.design_token_routes,
            &context.design_token_routes,
            |route| route.token_name.as_str(),
        );
    }
    if context.cascade_environment.is_some() {
        merged.cascade_environment = context.cascade_environment.clone();
    }

    expand_reachable_class_names_through_composes(&mut merged);
    merged
}

fn expand_reachable_class_names_through_composes(context: &mut TransformExecutionContextV0) {
    let mut changed = true;
    while changed {
        changed = false;
        for resolution in &context.css_module_composes_resolutions {
            if !class_name_is_reachable(
                &resolution.local_class_name,
                &context.reachable_class_names,
            ) {
                continue;
            }
            for exported_class_name in &resolution.exported_class_names {
                if !class_name_is_reachable(exported_class_name, &context.reachable_class_names) {
                    context
                        .reachable_class_names
                        .push(exported_class_name.clone());
                    changed = true;
                }
            }
        }
    }
    context.reachable_class_names.sort();
    context.reachable_class_names.dedup();
}

fn class_name_is_reachable(class_name: &str, reachable_class_names: &[String]) -> bool {
    let Some(normalized_class_name) = normalize_reachable_class_name(class_name) else {
        return false;
    };
    reachable_class_names
        .iter()
        .filter_map(|name| normalize_reachable_class_name(name))
        .any(|name| css_identifier_names_match(name, normalized_class_name))
}

fn normalize_reachable_class_name(name: &str) -> Option<&str> {
    let name = name.trim().strip_prefix('.').unwrap_or(name.trim());
    (!name.is_empty()).then_some(name)
}

pub(super) fn css_identifier_names_match(left: &str, right: &str) -> bool {
    ClassNameV0::new(left).same_as(&ClassNameV0::new(right))
}

pub(super) fn merge_target_options_transform_context(
    context: &TransformExecutionContextV0,
    target_options: OmenaQueryTargetTransformOptionsV0,
) -> TransformExecutionContextV0 {
    let mut merged = context.clone();
    if target_options.drop_dark_mode_media_queries {
        merged.drop_dark_mode_media_queries = true;
    }
    merged
}

pub(super) fn find_target_style_source<'a>(
    target_style_path: &str,
    style_sources: &'a [OmenaQueryStyleSourceInputV0],
) -> Option<&'a str> {
    style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)
        .map(|source| source.style_source.as_str())
}

pub(super) fn summarize_omena_query_transform_context_from_sources_with_resolution_context<'a>(
    target_style_path: &str,
    styles: impl IntoIterator<Item = (&'a str, &'a str)>,
    resolution_context: TransformResolutionContext<'_>,
) -> OmenaQueryTransformContextFromSourcesSummaryV0 {
    derive_omena_query_transform_context_from_sources_with_resolution_context(
        target_style_path,
        styles,
        resolution_context,
    )
    .summary
}

pub(super) struct OmenaQueryTransformContextFromSourcesDerivationV0 {
    pub(super) summary: OmenaQueryTransformContextFromSourcesSummaryV0,
    pub(super) style_fact_entries: Vec<OmenaQueryStyleFactEntry>,
}

pub(super) fn derive_omena_query_transform_context_from_sources_with_resolution_context<'a>(
    target_style_path: &str,
    styles: impl IntoIterator<Item = (&'a str, &'a str)>,
    resolution_context: TransformResolutionContext<'_>,
) -> OmenaQueryTransformContextFromSourcesDerivationV0 {
    let style_sources = styles.into_iter().collect::<Vec<_>>();
    let style_count = style_sources.len();
    let style_fact_entries = collect_omena_query_style_fact_entries(style_sources.as_slice());
    let source_by_path = style_sources
        .iter()
        .map(|(style_path, style_source)| ((*style_path).to_string(), (*style_source).to_string()))
        .collect::<BTreeMap<_, _>>();
    let available_style_paths = style_fact_entries
        .iter()
        .map(|entry| entry.style_path.as_str())
        .collect::<BTreeSet<_>>();
    let known_style_paths = available_style_paths
        .iter()
        .map(|path| normalize_omena_query_style_path(path))
        .collect::<Vec<_>>();
    let canonical_target_style_path =
        resolve_omena_query_style_path_against_known(target_style_path, &known_style_paths)
            .unwrap_or_else(|| normalize_omena_query_style_path(target_style_path));
    let target_entry = style_fact_entries.iter().find(|entry| {
        normalize_omena_query_style_path(entry.style_path.as_str()) == canonical_target_style_path
    });

    let mut context = TransformExecutionContextV0::default();

    if let Some(entry) = target_entry {
        context.import_inlines = derive_import_inlines_for_transform_context(
            entry,
            &style_fact_entries,
            &available_style_paths,
            &source_by_path,
            resolution_context,
        );
        let scss_module_uses = derive_static_scss_module_use_evaluations_for_transform_context(
            entry,
            &available_style_paths,
            &source_by_path,
            resolution_context,
        );
        match omena_parser_dialect_for_style_path(entry.style_path.as_str()) {
            OmenaParserStyleDialect::Scss | OmenaParserStyleDialect::Sass => {
                let dialect = omena_parser_dialect_for_style_path(entry.style_path.as_str());
                context.scss_module_evaluation =
                    derive_static_stylesheet_module_evaluation_for_transform_context(
                        entry.style_source.as_str(),
                        dialect,
                        &context.import_inlines,
                        &scss_module_uses,
                    );
            }
            OmenaParserStyleDialect::Less => {
                context.less_module_evaluation =
                    derive_static_stylesheet_module_evaluation_for_transform_context(
                        entry.style_source.as_str(),
                        OmenaParserStyleDialect::Less,
                        &context.import_inlines,
                        &[],
                    );
            }
            OmenaParserStyleDialect::Css => {}
        }
        context.class_name_rewrites = derive_class_name_rewrites_for_transform_context(entry);
        context.css_module_composes_resolutions =
            derive_css_module_composes_resolutions_for_transform_context(
                entry,
                &style_fact_entries,
                &available_style_paths,
                resolution_context,
            );
        context.css_module_value_resolutions =
            derive_css_module_value_resolutions_for_transform_context(
                entry,
                &style_fact_entries,
                &available_style_paths,
                &source_by_path,
                resolution_context,
            );
        context.design_token_routes = derive_design_token_routes_for_transform_context(
            entry,
            &style_fact_entries,
            resolution_context,
        );
    }

    let summary = OmenaQueryTransformContextFromSourcesSummaryV0 {
        schema_version: "0",
        product: "omena-query.transform-context",
        target_style_path: target_style_path.to_string(),
        style_count,
        import_inline_count: context.import_inlines.len(),
        class_name_rewrite_count: context.class_name_rewrites.len(),
        css_module_composes_resolution_count: context.css_module_composes_resolutions.len(),
        css_module_value_resolution_count: context.css_module_value_resolutions.len(),
        design_token_route_count: context.design_token_routes.len(),
        reachable_class_name_count: context.reachable_class_names.len(),
        reachable_keyframe_name_count: context.reachable_keyframe_names.len(),
        reachable_value_name_count: context.reachable_value_names.len(),
        reachable_custom_property_name_count: context.reachable_custom_property_names.len(),
        context,
        ready_surfaces: vec![
            "transformContextProducer",
            "stylesheetModuleEvaluationProducer",
            "cssModuleClassRewriteProducer",
            "cssModuleComposesResolutionProducer",
            "cssModuleValueResolutionProducer",
            "designTokenRouteProducer",
            "transitiveImportInlineProducer",
        ],
    };
    OmenaQueryTransformContextFromSourcesDerivationV0 {
        summary,
        style_fact_entries,
    }
}

pub(super) struct OmenaQueryEngineInputTransformContextDerivationV0 {
    pub(super) module_reachability: OmenaQueryEngineInputModuleReachabilityV0,
    pub(super) reachability_precision: Option<FactPrecision>,
    pub(super) closed_set_enumeration_candidate: bool,
}

pub fn summarize_omena_query_transform_context_from_engine_input(
    input: &EngineInputV2,
    target_style_path: &str,
    closed_world_requested: bool,
) -> OmenaQueryTransformContextFromEngineInputSummaryV0 {
    derive_omena_query_transform_context_from_engine_input(
        input,
        target_style_path,
        closed_world_requested,
    )
    .module_reachability
    .into_summary()
}

pub fn derive_omena_query_module_reachability_from_engine_input(
    input: &EngineInputV2,
    target_style_path: &str,
    closed_world_requested: bool,
) -> OmenaQueryEngineInputModuleReachabilityV0 {
    derive_omena_query_transform_context_from_engine_input(
        input,
        target_style_path,
        closed_world_requested,
    )
    .module_reachability
}

pub(super) fn derive_omena_query_transform_context_from_engine_input(
    input: &EngineInputV2,
    target_style_path: &str,
    closed_world_requested: bool,
) -> OmenaQueryEngineInputTransformContextDerivationV0 {
    let mut known_style_paths = input
        .styles
        .iter()
        .map(|style| normalize_omena_query_style_path(style.file_path.as_str()))
        .collect::<Vec<_>>();
    known_style_paths.sort();
    known_style_paths.dedup();
    let canonical_target_style_path = resolve_omena_query_style_path_against_known(
        target_style_path,
        known_style_paths.as_slice(),
    )
    .unwrap_or_else(|| normalize_omena_query_style_path(target_style_path));
    let (projection_summary, projection_precisions) =
        omena_query_core::summarize_omena_query_expression_domain_selector_projection_with_precision_and_style_path_resolver(
            input,
            resolve_omena_query_style_path_against_known,
        );
    let precision_by_projection = projection_precisions
        .iter()
        .map(|entry| {
            (
                (entry.graph_id.as_str(), entry.node_id.as_str()),
                entry.precision,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut reachable_class_names = BTreeSet::new();
    let mut reachability_sources = Vec::new();
    let mut reachability_precision_ceiling: Option<FactPrecision> = None;
    let mut closed_set_enumeration_candidate = true;
    let mut selected_projection_count = 0_usize;
    let mut targeted_class_names_by_style_path = BTreeMap::<String, BTreeSet<String>>::new();
    let mut targeted_projection_count_by_style_path = BTreeMap::<String, usize>::new();
    let mut unattributed_class_names = BTreeSet::new();
    let mut unattributed_projection_count = 0_usize;
    let mut projected_class_names = BTreeSet::new();

    for projection in &projection_summary.projections {
        projected_class_names.extend(projection.selector_names.iter().cloned());
        if projection.target_style_paths.is_empty() {
            unattributed_projection_count += 1;
            unattributed_class_names.extend(projection.selector_names.iter().cloned());
        } else {
            let mut has_unresolved_target = false;
            for target_style_path in projection
                .target_style_paths
                .iter()
                .collect::<BTreeSet<_>>()
            {
                let Some(target_style_path) = resolve_omena_query_style_path_against_known(
                    target_style_path,
                    known_style_paths.as_slice(),
                ) else {
                    let target_style_path = normalize_omena_query_style_path(target_style_path);
                    *targeted_projection_count_by_style_path
                        .entry(target_style_path.clone())
                        .or_default() += 1;
                    targeted_class_names_by_style_path
                        .entry(target_style_path)
                        .or_default()
                        .extend(projection.selector_names.iter().cloned());
                    has_unresolved_target = true;
                    continue;
                };
                *targeted_projection_count_by_style_path
                    .entry(target_style_path.clone())
                    .or_default() += 1;
                targeted_class_names_by_style_path
                    .entry(target_style_path)
                    .or_default()
                    .extend(projection.selector_names.iter().cloned());
            }
            if has_unresolved_target {
                unattributed_projection_count += 1;
                unattributed_class_names.extend(projection.selector_names.iter().cloned());
            }
        }
        let projection_targets_current_style = projection.target_style_paths.is_empty()
            || projection.target_style_paths.iter().any(|path| {
                resolve_omena_query_style_path_against_known(path, known_style_paths.as_slice())
                    .is_none_or(|path| path == canonical_target_style_path)
            });
        if projection_targets_current_style {
            selected_projection_count += 1;
            closed_set_enumeration_candidate &=
                matches!(projection.value_kind, "bottom" | "exact" | "finiteSet");
            reachable_class_names.extend(projection.selector_names.iter().cloned());
            let projection_precision = precision_by_projection
                .get(&(projection.graph_id.as_str(), projection.node_id.as_str()))
                .copied()
                .unwrap_or(FactPrecision::Unknown);
            reachability_precision_ceiling = Some(
                reachability_precision_ceiling.map_or(projection_precision, |current| {
                    current.bounded_by(projection_precision)
                }),
            );
            reachability_sources.push(OmenaQuerySemanticReachabilitySourceV0 {
                graph_id: projection.graph_id.clone(),
                file_path: projection.file_path.clone(),
                node_id: projection.node_id.clone(),
                target_style_paths: projection.target_style_paths.clone(),
                value_kind: projection.value_kind,
                reduced_product: projection.reduced_product.clone(),
                selector_names: projection.selector_names.clone(),
                certainty: projection.certainty,
            });
        }
    }

    let semantic_context = TransformExecutionContextV0 {
        reachable_class_names: reachable_class_names.into_iter().collect(),
        ..TransformExecutionContextV0::default()
    };
    let style_sources = input
        .styles
        .iter()
        .filter_map(|style| {
            style
                .source
                .as_deref()
                .map(|source| (style.file_path.as_str(), source))
        })
        .collect::<Vec<_>>();
    let source_context_summary = (!style_sources.is_empty()).then(|| {
        super::summarize_omena_query_transform_context_from_sources(
            target_style_path,
            style_sources,
            &[],
        )
    });
    let context = if let Some(source_context_summary) = &source_context_summary {
        merge_transform_context(source_context_summary.context.clone(), &semantic_context)
    } else {
        semantic_context
    };

    let mut ready_surfaces = vec![
        "expressionDomainSelectorProjection",
        "semanticReachabilityTransformContext",
    ];
    if source_context_summary.is_some() {
        ready_surfaces.push("engineInputStyleSourceTransformContext");
    }

    let summary = OmenaQueryTransformContextFromEngineInputSummaryV0 {
        schema_version: "0",
        product: "omena-query.transform-context-from-engine-input",
        input_version: input.version.clone(),
        target_style_path: target_style_path.to_string(),
        closed_world_requested,
        style_source_count: source_context_summary
            .as_ref()
            .map_or(0, |summary| summary.style_count),
        projection_count: projection_summary.projection_count,
        selected_projection_count: reachability_sources.len(),
        import_inline_count: context.import_inlines.len(),
        class_name_rewrite_count: context.class_name_rewrites.len(),
        css_module_composes_resolution_count: context.css_module_composes_resolutions.len(),
        css_module_value_resolution_count: context.css_module_value_resolutions.len(),
        design_token_route_count: context.design_token_routes.len(),
        reachable_class_name_count: context.reachable_class_names.len(),
        reachable_keyframe_name_count: context.reachable_keyframe_names.len(),
        reachable_value_name_count: context.reachable_value_names.len(),
        reachable_custom_property_name_count: context.reachable_custom_property_names.len(),
        reachability_sources,
        context,
        ready_surfaces,
    };
    let targeted_class_names_by_style_path = targeted_class_names_by_style_path
        .into_iter()
        .map(|(path, names)| (path, names.into_iter().collect()))
        .collect();
    let declared_class_names_by_style_path = input
        .styles
        .iter()
        .map(|style| {
            let mut class_names = style
                .document
                .selectors
                .iter()
                .filter_map(|selector| {
                    let name = selector
                        .canonical_name
                        .as_deref()
                        .unwrap_or(selector.name.as_str())
                        .trim();
                    let name = name.strip_prefix('.').unwrap_or(name);
                    (!name.is_empty()).then(|| name.to_string())
                })
                .collect::<Vec<_>>();
            class_names.sort();
            class_names.dedup();
            (
                normalize_omena_query_style_path(style.file_path.as_str()),
                class_names,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let module_reachability = OmenaQueryEngineInputModuleReachabilityV0::new(
        summary,
        known_style_paths,
        OmenaQueryEngineInputModuleAttributionV0::new(
            declared_class_names_by_style_path,
            targeted_class_names_by_style_path,
            targeted_projection_count_by_style_path,
        ),
        unattributed_class_names.into_iter().collect(),
        unattributed_projection_count,
        projected_class_names.into_iter().collect(),
    );

    OmenaQueryEngineInputTransformContextDerivationV0 {
        reachability_precision: reachability_precision_ceiling,
        closed_set_enumeration_candidate: selected_projection_count > 0
            && closed_set_enumeration_candidate,
        module_reachability,
    }
}

fn merge_context_list(target: &mut Vec<String>, additional: &[String]) {
    for item in additional {
        if !target.contains(item) {
            target.push(item.clone());
        }
    }
    target.sort();
}

fn merge_context_records_by_key<T, F>(target: &mut Vec<T>, overrides: &[T], key: F)
where
    T: Clone,
    F: Fn(&T) -> &str,
{
    for item in overrides {
        let item_key = key(item);
        if let Some(existing) = target.iter_mut().find(|existing| key(existing) == item_key) {
            *existing = item.clone();
        } else {
            target.push(item.clone());
        }
    }
    target.sort_by(|left, right| key(left).cmp(key(right)));
}

fn merge_class_context_records_by_key<T, F>(target: &mut Vec<T>, overrides: &[T], key: F)
where
    T: Clone,
    F: Fn(&T) -> &str,
{
    // Precedence follows the caller's module partition and then source declaration order.
    // An explicit record replaces a derived record under the same canonical class key;
    // raw spelling order never selects a winner. A future module-qualified compound key
    // adds module identity to this normalized name component rather than decoding it again.
    let mut merged = Vec::with_capacity(target.len() + overrides.len());
    append_class_context_records_first_witness(&mut merged, overrides, &key);
    append_class_context_records_first_witness(&mut merged, target, &key);
    *target = merged;
}

fn append_class_context_records_first_witness<T, F>(target: &mut Vec<T>, candidates: &[T], key: &F)
where
    T: Clone,
    F: Fn(&T) -> &str,
{
    let occupied_canonical_keys = target
        .iter()
        .map(|existing| ClassNameV0::new(key(existing)).canonical_key())
        .collect::<BTreeSet<_>>();
    let mut admitted_raw_keys = target
        .iter()
        .map(|existing| key(existing).to_string())
        .collect::<BTreeSet<_>>();
    for item in candidates {
        let item_key = ClassNameV0::new(key(item));
        if !occupied_canonical_keys.contains(&item_key.canonical_key())
            && admitted_raw_keys.insert(key(item).to_string())
        {
            target.push(item.clone());
        }
    }
}

pub(super) fn merge_module_css_module_contexts_first_witness(
    left: &[TransformModuleCssModuleContextV0],
    right: &[TransformModuleCssModuleContextV0],
) -> Vec<TransformModuleCssModuleContextV0> {
    let mut merged = Vec::<TransformModuleCssModuleContextV0>::new();
    for context in left.iter().chain(right) {
        let target = if let Some(index) = merged
            .iter()
            .position(|candidate| candidate.module_instance == context.module_instance)
        {
            &mut merged[index]
        } else {
            let index = merged.len();
            merged.push(TransformModuleCssModuleContextV0::new(
                context.module_instance.clone(),
            ));
            &mut merged[index]
        };
        append_class_context_records_first_witness(
            &mut target.class_name_rewrites,
            context.class_name_rewrites.as_slice(),
            &|rewrite: &TransformClassNameRewriteV0| rewrite.original_name.as_str(),
        );
        append_class_context_records_first_witness(
            &mut target.composes_resolutions,
            context.composes_resolutions.as_slice(),
            &|resolution: &TransformCssModuleComposesResolutionV0| {
                resolution.local_class_name.as_str()
            },
        );
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use omena_query_transform_runner::{
        TransformClassNameRewriteV0, TransformCssModuleComposesResolutionV0,
        TransformCssModuleValueResolutionV0, TransformDesignTokenRouteV0, TransformImportInlineV0,
    };

    fn class_rewrite(original_name: &str, rewritten_name: &str) -> TransformClassNameRewriteV0 {
        TransformClassNameRewriteV0 {
            original_name: original_name.to_string(),
            rewritten_name: rewritten_name.to_string(),
        }
    }

    #[test]
    fn explicit_class_rewrite_precedence_ignores_escape_spelling_order() {
        for (derived_name, explicit_name) in [(r#"\E9 tat"#, "état"), ("état", r#"\E9 tat"#)] {
            let derived = TransformExecutionContextV0 {
                class_name_rewrites: vec![class_rewrite(derived_name, "_derived")],
                css_module_composes_resolutions: vec![TransformCssModuleComposesResolutionV0 {
                    local_class_name: derived_name.to_string(),
                    exported_class_names: vec!["derived".to_string()],
                }],
                ..TransformExecutionContextV0::default()
            };
            let explicit = TransformExecutionContextV0 {
                class_name_rewrites: vec![class_rewrite(explicit_name, "_explicit")],
                css_module_composes_resolutions: vec![TransformCssModuleComposesResolutionV0 {
                    local_class_name: explicit_name.to_string(),
                    exported_class_names: vec!["explicit".to_string()],
                }],
                ..TransformExecutionContextV0::default()
            };

            let merged = merge_transform_context(derived, &explicit);

            assert_eq!(
                merged.class_name_rewrites,
                vec![class_rewrite(explicit_name, "_explicit")]
            );
            assert_eq!(
                merged.css_module_composes_resolutions,
                vec![TransformCssModuleComposesResolutionV0 {
                    local_class_name: explicit_name.to_string(),
                    exported_class_names: vec!["explicit".to_string()],
                }]
            );
            println!(
                "canonical-merge derived={derived_name:?} explicit={explicit_name:?} winner={:?}",
                merged.class_name_rewrites
            );
        }
    }

    #[test]
    fn class_merge_normalization_does_not_apply_to_other_context_keys() {
        let derived = TransformExecutionContextV0 {
            import_inlines: vec![TransformImportInlineV0 {
                import_source: r#"\E9 tat"#.to_string(),
                replacement_css: "derived".to_string(),
            }],
            css_module_value_resolutions: vec![TransformCssModuleValueResolutionV0 {
                local_name: r#"\E9 tat"#.to_string(),
                resolved_value: "derived".to_string(),
            }],
            design_token_routes: vec![TransformDesignTokenRouteV0 {
                token_name: r#"\E9 tat"#.to_string(),
                routed_value: "derived".to_string(),
            }],
            ..TransformExecutionContextV0::default()
        };
        let explicit = TransformExecutionContextV0 {
            import_inlines: vec![TransformImportInlineV0 {
                import_source: "état".to_string(),
                replacement_css: "explicit".to_string(),
            }],
            css_module_value_resolutions: vec![TransformCssModuleValueResolutionV0 {
                local_name: "état".to_string(),
                resolved_value: "explicit".to_string(),
            }],
            design_token_routes: vec![TransformDesignTokenRouteV0 {
                token_name: "état".to_string(),
                routed_value: "explicit".to_string(),
            }],
            ..TransformExecutionContextV0::default()
        };

        let merged = merge_transform_context(derived, &explicit);

        assert_eq!(merged.import_inlines.len(), 2);
        assert_eq!(merged.css_module_value_resolutions.len(), 2);
        assert_eq!(merged.design_token_routes.len(), 2);
    }

    fn module_context(
        module: &str,
        original_name: &str,
        rewritten_name: &str,
    ) -> TransformModuleCssModuleContextV0 {
        TransformModuleCssModuleContextV0::new(omena_parser::ModuleInstanceKeyV0::unconfigured(
            omena_parser::ModuleIdV0::new(module),
        ))
        .with_class_name_rewrites(vec![class_rewrite(original_name, rewritten_name)])
    }

    fn merge_module_css_module_contexts_last_witness(
        left: &[TransformModuleCssModuleContextV0],
        right: &[TransformModuleCssModuleContextV0],
    ) -> Vec<TransformModuleCssModuleContextV0> {
        merge_module_css_module_contexts_first_witness(right, left)
    }

    #[test]
    fn module_context_first_witness_merge_obeys_left_regular_band_laws() {
        let u = vec![
            module_context("src/a.module.css", "shared", "_u"),
            module_context("src/b.module.css", "own", "_b"),
        ];
        let v = vec![
            module_context("src/a.module.css", "shared", "_v"),
            module_context("src/c.module.css", "own", "_c"),
        ];
        let w = vec![module_context("src/a.module.css", "third", "_w")];

        let uv = merge_module_css_module_contexts_first_witness(&u, &v);
        assert_eq!(
            merge_module_css_module_contexts_first_witness(&uv, &w),
            merge_module_css_module_contexts_first_witness(
                &u,
                &merge_module_css_module_contexts_first_witness(&v, &w),
            ),
            "associativity"
        );
        assert_eq!(
            merge_module_css_module_contexts_first_witness(&u, &u),
            u,
            "idempotence"
        );
        assert_eq!(
            merge_module_css_module_contexts_first_witness(&uv, &u),
            uv,
            "left-regular-band absorption"
        );

        let last_wins_uv = merge_module_css_module_contexts_last_witness(&u, &v);
        let last_wins_uv_then_u = merge_module_css_module_contexts_last_witness(&last_wins_uv, &u);
        assert_ne!(
            last_wins_uv_then_u, last_wins_uv,
            "a last-wins variant must fail the absorption control"
        );
    }
}
