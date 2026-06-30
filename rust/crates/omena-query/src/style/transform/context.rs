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
use std::borrow::Cow;
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
        .resolved_style_path
    }
}

pub(super) fn merge_transform_context(
    mut merged: TransformExecutionContextV0,
    context: &TransformExecutionContextV0,
) -> TransformExecutionContextV0 {
    merged.drop_dark_mode_media_queries =
        merged.drop_dark_mode_media_queries || context.drop_dark_mode_media_queries;
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
        merge_context_records_by_key(
            &mut merged.class_name_rewrites,
            &context.class_name_rewrites,
            |rewrite| rewrite.original_name.as_str(),
        );
    }
    if !context.css_module_composes_resolutions.is_empty() {
        merge_context_records_by_key(
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
    left == right || decode_css_identifier_escapes(left) == decode_css_identifier_escapes(right)
}

pub(super) fn decode_css_identifier_escapes(text: &str) -> Cow<'_, str> {
    if !text.contains('\\') {
        return Cow::Borrowed(text);
    }

    let mut output = String::with_capacity(text.len());
    let mut index = 0usize;
    while index < text.len() {
        let Some(ch) = text[index..].chars().next() else {
            break;
        };
        if ch != '\\' {
            output.push(ch);
            index += ch.len_utf8();
            continue;
        }

        index += ch.len_utf8();
        let Some(next) = text[index..].chars().next() else {
            output.push('\\');
            break;
        };
        if next.is_ascii_hexdigit() {
            let hex_start = index;
            let mut hex_end = index;
            let mut digit_count = 0usize;
            while hex_end < text.len() && digit_count < 6 {
                let Some(candidate) = text[hex_end..].chars().next() else {
                    break;
                };
                if !candidate.is_ascii_hexdigit() {
                    break;
                }
                hex_end += candidate.len_utf8();
                digit_count += 1;
            }
            if let Some(decoded) = u32::from_str_radix(&text[hex_start..hex_end], 16)
                .ok()
                .and_then(char::from_u32)
            {
                output.push(decoded);
            }
            index = hex_end;
            if let Some(terminator) = text[index..].chars().next()
                && terminator.is_ascii_whitespace()
            {
                index += terminator.len_utf8();
            }
            continue;
        }

        output.push(next);
        index += next.len_utf8();
    }

    Cow::Owned(output)
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
    let target_entry = style_fact_entries
        .iter()
        .find(|entry| entry.style_path == target_style_path);

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
            resolution_context.package_manifests,
        );
    }

    OmenaQueryTransformContextFromSourcesSummaryV0 {
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
    }
}

pub fn summarize_omena_query_transform_context_from_engine_input(
    input: &EngineInputV2,
    target_style_path: &str,
    closed_style_world: bool,
) -> OmenaQueryTransformContextFromEngineInputSummaryV0 {
    let projection_summary = summarize_omena_query_expression_domain_selector_projection(input);
    let mut reachable_class_names = BTreeSet::new();
    let mut reachability_sources = Vec::new();

    for projection in &projection_summary.projections {
        if projection.target_style_paths.is_empty()
            || projection
                .target_style_paths
                .iter()
                .any(|path| path == target_style_path)
        {
            reachable_class_names.extend(projection.selector_names.iter().cloned());
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

    OmenaQueryTransformContextFromEngineInputSummaryV0 {
        schema_version: "0",
        product: "omena-query.transform-context-from-engine-input",
        input_version: input.version.clone(),
        target_style_path: target_style_path.to_string(),
        closed_style_world,
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
