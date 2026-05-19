use super::*;
use omena_parser::lex;
use omena_syntax::SyntaxKind;
use omena_transform_passes::{
    TransformClassNameRewriteV0, TransformCssModuleComposesResolutionV0,
    TransformCssModuleValueResolutionV0, TransformDesignTokenRouteV0, TransformImportInlineV0,
    TransformModuleEvaluationV0, inline_css_imports,
    resolve_static_css_modules_local_value_resolutions_from_source,
};
use std::borrow::Cow;

use super::stylesheet_evaluation::{
    derive_static_scss_stylesheet_module_variable_exports,
    derive_static_stylesheet_module_evaluation,
};

pub fn summarize_omena_query_transform_plan_from_source(
    style_path: &str,
    style_source: &str,
    target_label: &str,
    target_support: OmenaQueryTargetFeatureSupportV0,
    target_options: OmenaQueryTargetTransformOptionsV0,
    print_options: OmenaQueryTransformPrintOptionsV0,
) -> OmenaQueryTransformPlanSummaryV0 {
    summarize_omena_query_transform_plan_from_source_with_context(
        style_path,
        style_source,
        target_label,
        target_support,
        target_options,
        print_options,
        &TransformExecutionContextV0::default(),
    )
}

pub fn summarize_omena_query_transform_plan_from_source_with_context(
    style_path: &str,
    style_source: &str,
    target_label: &str,
    target_support: OmenaQueryTargetFeatureSupportV0,
    target_options: OmenaQueryTargetTransformOptionsV0,
    print_options: OmenaQueryTransformPrintOptionsV0,
    context: &TransformExecutionContextV0,
) -> OmenaQueryTransformPlanSummaryV0 {
    let dialect = omena_parser_dialect_for_style_path(style_path);
    let bundle = summarize_omena_transform_bundle_from_source(style_path, style_source, dialect);
    let target = plan_target_transforms(target_label, target_support, target_options);
    let execution_context = merge_target_options_transform_context(context, target_options);
    summarize_omena_query_transform_plan_from_parts(TransformPlanPartsV0 {
        style_path,
        style_source,
        dialect,
        bundle,
        target,
        target_query: None,
        print_options,
        context: &execution_context,
    })
}

pub fn summarize_omena_query_transform_plan_from_target_query(
    style_path: &str,
    style_source: &str,
    target_query: &str,
    target_options: OmenaQueryTargetTransformOptionsV0,
    print_options: OmenaQueryTransformPrintOptionsV0,
) -> OmenaQueryTransformPlanSummaryV0 {
    summarize_omena_query_transform_plan_from_target_query_with_context(
        style_path,
        style_source,
        target_query,
        target_options,
        print_options,
        &TransformExecutionContextV0::default(),
    )
}

pub fn summarize_omena_query_transform_plan_from_target_query_with_context(
    style_path: &str,
    style_source: &str,
    target_query: &str,
    target_options: OmenaQueryTargetTransformOptionsV0,
    print_options: OmenaQueryTransformPrintOptionsV0,
    context: &TransformExecutionContextV0,
) -> OmenaQueryTransformPlanSummaryV0 {
    let dialect = omena_parser_dialect_for_style_path(style_path);
    let bundle = summarize_omena_transform_bundle_from_source(style_path, style_source, dialect);
    let target_query_plan = plan_target_transforms_from_query(target_query, target_options);
    let target = target_query_plan.transform_plan.clone();
    let execution_context = merge_target_options_transform_context(context, target_options);
    summarize_omena_query_transform_plan_from_parts(TransformPlanPartsV0 {
        style_path,
        style_source,
        dialect,
        bundle,
        target,
        target_query: Some(target_query_plan),
        print_options,
        context: &execution_context,
    })
}

struct TransformPlanPartsV0<'a> {
    style_path: &'a str,
    style_source: &'a str,
    dialect: OmenaParserStyleDialect,
    bundle: TransformBundleSourceSummaryV0,
    target: TransformTargetPlanV0,
    target_query: Option<OmenaQueryTransformTargetQueryPlanV0>,
    print_options: OmenaQueryTransformPrintOptionsV0,
    context: &'a TransformExecutionContextV0,
}

fn summarize_omena_query_transform_plan_from_parts(
    parts: TransformPlanPartsV0<'_>,
) -> OmenaQueryTransformPlanSummaryV0 {
    let egg = plan_egg_rewrite_passes_for_source(parts.style_source);
    let custom_property_fixed_point = summarize_static_css_custom_property_fixed_point_from_source(
        parts.style_source,
        parts.dialect,
    );

    let mut combined_passes = Vec::new();
    extend_passes_from_ids(&parts.bundle.planned_pass_ids, &mut combined_passes);
    extend_passes_from_ids(&parts.target.planned_pass_ids, &mut combined_passes);
    extend_passes_from_ids(&egg.planned_pass_ids, &mut combined_passes);
    combined_passes.push(TransformPassKind::PrintCss);
    combined_passes.sort_by_key(|pass| pass.ordinal());
    combined_passes.dedup();

    let combined_plan = plan_transform_passes(&combined_passes);
    let semantic_signature = format!(
        "omena-query-transform:{}:{}",
        parts.style_path,
        parts.style_source.len()
    );
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        parts.style_source,
        parts.dialect,
        &combined_passes,
        parts.context,
    );
    let print = print_transform_execution_artifact_with_dialect_and_source(
        parts.style_path,
        parts.style_source,
        parts.dialect,
        semantic_signature,
        &combined_passes,
        parts.print_options,
        &execution,
    );
    let egg_witnesses = execute_egg_rewrite_witnesses_for_css_source(
        parts.style_source,
        &execution.output_css,
        &egg.planned_pass_ids,
    );
    let semantic_removal_count = execution.semantic_removals.len();
    let combined_pass_ids = combined_plan.ordered_pass_ids.clone();
    let combined_violated_dag_edge_count = combined_plan.violated_dag_edge_count;

    OmenaQueryTransformPlanSummaryV0 {
        schema_version: "0",
        product: "omena-query.transform-plan",
        style_path: parts.style_path.to_string(),
        dialect: omena_parser_style_dialect_label(parts.dialect),
        bundle: parts.bundle,
        target: parts.target,
        target_query: parts.target_query,
        egg,
        egg_witnesses,
        custom_property_fixed_point,
        print,
        execution,
        semantic_removal_count,
        combined_plan,
        combined_pass_ids,
        combined_violated_dag_edge_count,
        ready_surfaces: vec![
            "transformBundlePlan",
            "transformTargetPlan",
            "transformEggPlan",
            "transformEggExecutionWitnesses",
            "customPropertyLeastFixedPoint",
            "transformPrintArtifact",
            "transformExecutionRuntime",
            "combinedTransformPassPlan",
        ],
    }
}

pub fn execute_omena_query_transform_passes_from_source(
    style_path: &str,
    style_source: &str,
    requested_pass_ids: &[String],
) -> OmenaQueryTransformExecuteSummaryV0 {
    execute_omena_query_transform_passes_from_source_with_context(
        style_path,
        style_source,
        requested_pass_ids,
        &TransformExecutionContextV0::default(),
    )
}

pub fn summarize_omena_query_consumer_check_style_source(
    style_path: &str,
    style_source: &str,
) -> OmenaQueryConsumerCheckSummaryV0 {
    let dialect = omena_parser_dialect_for_style_path(style_path);
    let parse_result = parse(style_source, dialect);
    let style_facts = summarize_omena_query_omena_parser_style_facts(style_source, dialect);

    OmenaQueryConsumerCheckSummaryV0 {
        schema_version: "0",
        product: "omena-query.consumer-check-style-source",
        style_path: style_path.to_string(),
        dialect: omena_parser_style_dialect_label(dialect),
        token_count: parse_result.token_count(),
        parser_error_count: parse_result.errors().len(),
        class_selector_count: style_facts.class_selector_names.len(),
        custom_property_count: style_facts.custom_property_names.len(),
        keyframe_count: style_facts.keyframe_names.len(),
        ready_surfaces: vec![
            "consumerCheckFacade",
            "parserFactSummary",
            "styleDocumentDiagnostics",
        ],
    }
}

pub fn execute_omena_query_consumer_build_style_source(
    style_path: &str,
    style_source: &str,
    requested_pass_ids: &[String],
) -> OmenaQueryConsumerBuildSummaryV0 {
    execute_omena_query_consumer_build_style_source_with_context(
        style_path,
        style_source,
        requested_pass_ids,
        &TransformExecutionContextV0::default(),
    )
}

pub fn execute_omena_query_consumer_build_style_source_with_context(
    style_path: &str,
    style_source: &str,
    requested_pass_ids: &[String],
    context: &TransformExecutionContextV0,
) -> OmenaQueryConsumerBuildSummaryV0 {
    let pass_ids = if requested_pass_ids.is_empty() {
        all_transform_pass_kinds()
            .into_iter()
            .map(|pass| pass.id().to_string())
            .collect::<Vec<_>>()
    } else {
        requested_pass_ids.to_vec()
    };
    let context = merge_single_source_transform_context(style_path, style_source, context);
    let execution_summary = execute_omena_query_transform_passes_from_source_with_context(
        style_path,
        style_source,
        &pass_ids,
        &context,
    );

    OmenaQueryConsumerBuildSummaryV0 {
        schema_version: "0",
        product: "omena-query.consumer-build-style-source",
        style_path: style_path.to_string(),
        dialect: omena_parser_style_dialect_label(omena_parser_dialect_for_style_path(style_path)),
        requested_pass_ids: requested_pass_ids.to_vec(),
        target_query: None,
        unknown_pass_ids: execution_summary.unknown_pass_ids,
        semantic_removal_count: execution_summary.semantic_removal_count,
        execution: execution_summary.execution,
        ready_surfaces: vec![
            "consumerBuildFacade",
            "singleSourceTransformContextProducer",
            "transformExecutionRuntime",
            "transformPassOutcomeContract",
        ],
    }
}

pub fn execute_omena_query_consumer_build_style_source_with_engine_input_context(
    style_path: &str,
    style_source: &str,
    requested_pass_ids: &[String],
    input: &EngineInputV2,
    closed_style_world: bool,
) -> OmenaQueryConsumerBuildSummaryV0 {
    let context_summary = summarize_omena_query_transform_context_from_engine_input(
        input,
        style_path,
        closed_style_world,
    );
    let mut summary = execute_omena_query_consumer_build_style_source_with_context(
        style_path,
        style_source,
        requested_pass_ids,
        &context_summary.context,
    );
    summary
        .ready_surfaces
        .push("semanticReachabilityTransformContext");
    summary
        .ready_surfaces
        .push("expressionDomainSelectorProjection");
    summary
}

pub fn execute_omena_query_consumer_build_style_sources_with_context(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    requested_pass_ids: &[String],
    context: &TransformExecutionContextV0,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Result<OmenaQueryConsumerBuildSummaryV0, String> {
    let Some(target_source) = find_target_style_source(target_style_path, style_sources) else {
        return Err(format!(
            "target style path {target_style_path:?} was not found in workspace style sources"
        ));
    };
    let context = merge_workspace_transform_context(
        target_style_path,
        style_sources,
        context,
        package_manifests,
    );
    let mut summary = execute_omena_query_consumer_build_style_source_with_context(
        target_style_path,
        target_source,
        requested_pass_ids,
        &context,
    );
    summary
        .ready_surfaces
        .push("multiSourceTransformContextProducer");
    Ok(summary)
}

pub fn execute_omena_query_consumer_build_style_sources(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    requested_pass_ids: &[String],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Result<OmenaQueryConsumerBuildSummaryV0, String> {
    execute_omena_query_consumer_build_style_sources_with_context(
        target_style_path,
        style_sources,
        requested_pass_ids,
        &TransformExecutionContextV0::default(),
        package_manifests,
    )
}

pub fn execute_omena_query_consumer_build_style_source_for_target_query(
    style_path: &str,
    style_source: &str,
    target_query: &str,
) -> OmenaQueryConsumerBuildSummaryV0 {
    execute_omena_query_consumer_build_style_source_for_target_query_with_options(
        style_path,
        style_source,
        target_query,
        conservative_omena_query_target_options(),
    )
}

pub fn execute_omena_query_consumer_build_style_source_for_target_query_with_options(
    style_path: &str,
    style_source: &str,
    target_query: &str,
    target_options: OmenaQueryTargetTransformOptionsV0,
) -> OmenaQueryConsumerBuildSummaryV0 {
    execute_omena_query_consumer_build_style_source_for_target_query_with_context_and_options(
        style_path,
        style_source,
        target_query,
        &TransformExecutionContextV0::default(),
        target_options,
    )
}

pub fn execute_omena_query_consumer_build_style_source_for_target_query_with_context_and_options(
    style_path: &str,
    style_source: &str,
    target_query: &str,
    context: &TransformExecutionContextV0,
    target_options: OmenaQueryTargetTransformOptionsV0,
) -> OmenaQueryConsumerBuildSummaryV0 {
    let context = merge_single_source_transform_context(style_path, style_source, context);
    let plan = summarize_omena_query_transform_plan_from_target_query_with_context(
        style_path,
        style_source,
        target_query,
        target_options,
        default_omena_query_transform_print_options(),
        &context,
    );
    let requested_pass_ids = plan
        .combined_pass_ids
        .iter()
        .map(|pass_id| (*pass_id).to_string())
        .collect::<Vec<_>>();

    OmenaQueryConsumerBuildSummaryV0 {
        schema_version: "0",
        product: "omena-query.consumer-build-style-source",
        style_path: plan.style_path,
        dialect: plan.dialect,
        requested_pass_ids,
        target_query: plan.target_query,
        unknown_pass_ids: Vec::new(),
        semantic_removal_count: plan.semantic_removal_count,
        execution: plan.execution,
        ready_surfaces: vec![
            "consumerBuildFacade",
            "targetQueryBuildFacade",
            "singleSourceTransformContextProducer",
            "transformExecutionRuntime",
            "transformPassOutcomeContract",
        ],
    }
}

pub fn execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    target_query: &str,
    context: &TransformExecutionContextV0,
    target_options: OmenaQueryTargetTransformOptionsV0,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Result<OmenaQueryConsumerBuildSummaryV0, String> {
    let Some(target_source) = find_target_style_source(target_style_path, style_sources) else {
        return Err(format!(
            "target style path {target_style_path:?} was not found in workspace style sources"
        ));
    };
    let context = merge_workspace_transform_context(
        target_style_path,
        style_sources,
        context,
        package_manifests,
    );
    let mut summary =
        execute_omena_query_consumer_build_style_source_for_target_query_with_context_and_options(
            target_style_path,
            target_source,
            target_query,
            &context,
            target_options,
        );
    summary
        .ready_surfaces
        .push("multiSourceTransformContextProducer");
    Ok(summary)
}

pub fn execute_omena_query_consumer_build_style_sources_for_target_query_with_options(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    target_query: &str,
    target_options: OmenaQueryTargetTransformOptionsV0,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Result<OmenaQueryConsumerBuildSummaryV0, String> {
    execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options(
        target_style_path,
        style_sources,
        target_query,
        &TransformExecutionContextV0::default(),
        target_options,
        package_manifests,
    )
}

fn derive_single_source_transform_context(
    style_path: &str,
    style_source: &str,
) -> TransformExecutionContextV0 {
    summarize_omena_query_transform_context_from_sources(
        style_path,
        [(style_path, style_source)],
        &[],
    )
    .context
}

fn merge_single_source_transform_context(
    style_path: &str,
    style_source: &str,
    context: &TransformExecutionContextV0,
) -> TransformExecutionContextV0 {
    merge_transform_context(
        derive_single_source_transform_context(style_path, style_source),
        context,
    )
}

fn merge_workspace_transform_context(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    context: &TransformExecutionContextV0,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> TransformExecutionContextV0 {
    let style_refs = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();
    let derived = summarize_omena_query_transform_context_from_sources(
        target_style_path,
        style_refs,
        package_manifests,
    )
    .context;
    merge_transform_context(derived, context)
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
        closed_style_world,
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
        summarize_omena_query_transform_context_from_sources(target_style_path, style_sources, &[])
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

fn merge_transform_context(
    mut merged: TransformExecutionContextV0,
    context: &TransformExecutionContextV0,
) -> TransformExecutionContextV0 {
    merged.closed_style_world = merged.closed_style_world || context.closed_style_world;
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
        .any(|name| name == normalized_class_name)
}

fn normalize_reachable_class_name(name: &str) -> Option<&str> {
    let name = name.trim().strip_prefix('.').unwrap_or(name.trim());
    (!name.is_empty()).then_some(name)
}

fn merge_target_options_transform_context(
    context: &TransformExecutionContextV0,
    target_options: OmenaQueryTargetTransformOptionsV0,
) -> TransformExecutionContextV0 {
    let mut merged = context.clone();
    if target_options.drop_dark_mode_media_queries {
        merged.drop_dark_mode_media_queries = true;
    }
    merged
}

fn find_target_style_source<'a>(
    target_style_path: &str,
    style_sources: &'a [OmenaQueryStyleSourceInputV0],
) -> Option<&'a str> {
    style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)
        .map(|source| source.style_source.as_str())
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

pub fn list_omena_query_transform_pass_summaries() -> Vec<OmenaQueryTransformPassSummaryV0> {
    all_transform_pass_kinds()
        .into_iter()
        .map(|kind| OmenaQueryTransformPassSummaryV0 {
            id: kind.id(),
            title: kind.title(),
            reads_semantic_graph: kind.reads_semantic_graph(),
            reads_cascade_model: kind.reads_cascade_model(),
        })
        .collect()
}

pub fn execute_omena_query_transform_passes_from_source_with_context(
    style_path: &str,
    style_source: &str,
    requested_pass_ids: &[String],
    context: &TransformExecutionContextV0,
) -> OmenaQueryTransformExecuteSummaryV0 {
    let mut requested_passes = Vec::new();
    let mut unknown_pass_ids = Vec::new();

    for pass_id in requested_pass_ids {
        match transform_pass_kind_from_id(pass_id) {
            Some(pass) => requested_passes.push(pass),
            None => unknown_pass_ids.push(pass_id.clone()),
        }
    }

    let dialect = omena_parser_dialect_for_style_path(style_path);
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        style_source,
        dialect,
        &requested_passes,
        context,
    );
    let semantic_removal_count = execution.semantic_removals.len();

    OmenaQueryTransformExecuteSummaryV0 {
        schema_version: "0",
        product: "omena-query.transform-execute",
        style_path: style_path.to_string(),
        requested_pass_ids: requested_pass_ids.to_vec(),
        unknown_pass_ids,
        execution,
        semantic_removal_count,
        ready_surfaces: vec!["transformExecutionRuntime", "transformPassOutcomeContract"],
    }
}

pub fn summarize_omena_query_transform_context_from_sources<'a>(
    target_style_path: &str,
    styles: impl IntoIterator<Item = (&'a str, &'a str)>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
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
            &available_style_paths,
            &source_by_path,
            package_manifests,
        );
        let scss_module_uses = derive_static_scss_module_use_evaluations_for_transform_context(
            entry,
            &available_style_paths,
            &source_by_path,
            package_manifests,
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
                package_manifests,
            );
        context.css_module_value_resolutions =
            derive_css_module_value_resolutions_for_transform_context(
                entry,
                &style_fact_entries,
                &available_style_paths,
                &source_by_path,
                package_manifests,
            );
        context.design_token_routes = derive_design_token_routes_for_transform_context(
            entry,
            &style_fact_entries,
            package_manifests,
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
            "directImportInlineProducer",
        ],
    }
}

fn derive_static_stylesheet_module_evaluation_for_transform_context(
    style_source: &str,
    dialect: OmenaParserStyleDialect,
    import_inlines: &[TransformImportInlineV0],
    scss_module_uses: &[StaticScssModuleUseEvaluation],
) -> Option<TransformModuleEvaluationV0> {
    let evaluation_source = derive_import_aware_static_stylesheet_module_evaluation_source(
        style_source,
        dialect,
        import_inlines,
    );
    let evaluation_source = derive_scss_use_aware_static_stylesheet_module_evaluation_source(
        evaluation_source.as_ref(),
        dialect,
        scss_module_uses,
    );
    derive_static_stylesheet_module_evaluation(evaluation_source.as_ref(), dialect).or_else(|| {
        (evaluation_source.as_ref() != style_source).then(|| TransformModuleEvaluationV0 {
            evaluator: static_stylesheet_module_system_evaluator_label(dialect).to_string(),
            evaluated_css: evaluation_source.into_owned(),
        })
    })
}

fn derive_import_aware_static_stylesheet_module_evaluation_source<'a>(
    style_source: &'a str,
    dialect: OmenaParserStyleDialect,
    import_inlines: &[TransformImportInlineV0],
) -> Cow<'a, str> {
    if import_inlines.is_empty() {
        return Cow::Borrowed(style_source);
    }
    let (inlined_source, mutation_count) =
        inline_css_imports(style_source, dialect, import_inlines);
    if mutation_count == 0 {
        Cow::Borrowed(style_source)
    } else {
        Cow::Owned(inlined_source)
    }
}

fn static_stylesheet_module_system_evaluator_label(
    dialect: OmenaParserStyleDialect,
) -> &'static str {
    match dialect {
        OmenaParserStyleDialect::Scss | OmenaParserStyleDialect::Sass => {
            "omena-query-static-scss-module-system-evaluator"
        }
        OmenaParserStyleDialect::Less => "omena-query-static-less-module-system-evaluator",
        OmenaParserStyleDialect::Css => "omena-query-static-css-module-system-evaluator",
    }
}

#[derive(Debug, Clone)]
struct StaticScssModuleUseEvaluation {
    source: String,
    namespace_kind: Option<&'static str>,
    namespace: Option<String>,
    evaluated_css: String,
    variable_exports: BTreeMap<String, String>,
}

fn derive_static_scss_module_use_evaluations_for_transform_context(
    entry: &OmenaQueryStyleFactEntry,
    available_style_paths: &BTreeSet<&str>,
    source_by_path: &BTreeMap<String, String>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<StaticScssModuleUseEvaluation> {
    if !matches!(
        omena_parser_dialect_for_style_path(entry.style_path.as_str()),
        OmenaParserStyleDialect::Scss | OmenaParserStyleDialect::Sass
    ) {
        return Vec::new();
    }

    entry
        .facts
        .sass_module_edges
        .iter()
        .filter(|edge| edge.kind == "sassUse")
        .filter(|edge| matches!(edge.namespace_kind, Some("alias") | Some("default")))
        .filter_map(|edge| {
            let resolved = resolve_style_module_source(
                entry.style_path.as_str(),
                edge.source.as_str(),
                available_style_paths,
                package_manifests,
            )?;
            let source = source_by_path.get(resolved.as_str())?;
            let evaluation =
                derive_static_stylesheet_module_evaluation(source, OmenaParserStyleDialect::Scss);
            Some(StaticScssModuleUseEvaluation {
                source: edge.source.clone(),
                namespace_kind: edge.namespace_kind,
                namespace: edge.namespace.clone(),
                evaluated_css: evaluation
                    .map(|evaluation| evaluation.evaluated_css)
                    .unwrap_or_else(|| source.clone()),
                variable_exports: derive_static_scss_stylesheet_module_variable_exports(source),
            })
        })
        .collect()
}

fn derive_scss_use_aware_static_stylesheet_module_evaluation_source<'a>(
    style_source: &'a str,
    dialect: OmenaParserStyleDialect,
    scss_module_uses: &[StaticScssModuleUseEvaluation],
) -> Cow<'a, str> {
    if !matches!(
        dialect,
        OmenaParserStyleDialect::Scss | OmenaParserStyleDialect::Sass
    ) || scss_module_uses.is_empty()
    {
        return Cow::Borrowed(style_source);
    }
    let source = replace_static_scss_namespaced_module_variables(style_source, scss_module_uses);
    let (source, mutation_count) = inline_static_scss_use_rules(&source, dialect, scss_module_uses);
    if mutation_count == 0 && source == style_source {
        Cow::Borrowed(style_source)
    } else {
        Cow::Owned(source)
    }
}

fn replace_static_scss_namespaced_module_variables(
    source: &str,
    scss_module_uses: &[StaticScssModuleUseEvaluation],
) -> String {
    let mut output = source.to_string();
    for module_use in scss_module_uses {
        if !matches!(module_use.namespace_kind, Some("alias") | Some("default")) {
            continue;
        }
        let Some(namespace) = module_use.namespace.as_deref() else {
            continue;
        };
        for (name, value) in &module_use.variable_exports {
            output =
                replace_static_scss_namespaced_variable_reference(&output, namespace, name, value);
        }
    }
    output
}

fn replace_static_scss_namespaced_variable_reference(
    source: &str,
    namespace: &str,
    name: &str,
    value: &str,
) -> String {
    let needle = format!("{namespace}.${name}");
    if !source.contains(needle.as_str()) {
        return source.to_string();
    }

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0usize;
    while let Some(offset) = source[cursor..].find(needle.as_str()) {
        let start = cursor + offset;
        let end = start + needle.len();
        if static_scss_reference_boundary_is_safe(source, start, end) {
            output.push_str(&source[cursor..start]);
            output.push_str(value);
            cursor = end;
        } else {
            output.push_str(&source[cursor..end]);
            cursor = end;
        }
    }
    output.push_str(&source[cursor..]);
    output
}

fn static_scss_reference_boundary_is_safe(source: &str, start: usize, end: usize) -> bool {
    let before_safe = source[..start]
        .chars()
        .next_back()
        .is_none_or(|ch| !static_scss_identifier_char(ch));
    let after_safe = source[end..]
        .chars()
        .next()
        .is_none_or(|ch| !static_scss_identifier_char(ch));
    before_safe && after_safe
}

fn static_scss_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')
}

fn inline_static_scss_use_rules(
    source: &str,
    dialect: OmenaParserStyleDialect,
    scss_module_uses: &[StaticScssModuleUseEvaluation],
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();
    let mut depth = 0usize;
    let mut index = 0usize;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
            SyntaxKind::AtKeyword
                if depth == 0 && tokens[index].text.eq_ignore_ascii_case("@use") =>
            {
                let Some(end_index) = static_scss_use_rule_semicolon(tokens, index) else {
                    index += 1;
                    continue;
                };
                let start = transform_token_start(&tokens[index]);
                let end = transform_token_end(&tokens[end_index]);
                if let Some(source_name) =
                    static_scss_use_rule_source_name(tokens, index + 1, end_index)
                    && let Some(module_use) = scss_module_uses
                        .iter()
                        .find(|module_use| module_use.source == source_name)
                {
                    replacements.push((start, end, module_use.evaluated_css.clone()));
                }
                index = end_index + 1;
                continue;
            }
            _ => {}
        }
        index += 1;
    }

    apply_transform_source_replacements(source, replacements)
}

fn static_scss_use_rule_semicolon(
    tokens: &[omena_parser::LexedToken],
    at_use_index: usize,
) -> Option<usize> {
    let mut index = at_use_index + 1;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::Semicolon => return Some(index),
            SyntaxKind::LeftBrace | SyntaxKind::RightBrace => return None,
            _ => index += 1,
        }
    }
    None
}

fn static_scss_use_rule_source_name(
    tokens: &[omena_parser::LexedToken],
    start_index: usize,
    end_index: usize,
) -> Option<String> {
    tokens[start_index..end_index]
        .iter()
        .find(|token| matches!(token.kind, SyntaxKind::String | SyntaxKind::Url))
        .map(|token| token.text.trim_matches('"').trim_matches('\'').to_string())
}

fn apply_transform_source_replacements(
    source: &str,
    mut replacements: Vec<(usize, usize, String)>,
) -> (String, usize) {
    if replacements.is_empty() {
        return (source.to_string(), 0);
    }
    replacements.sort_by_key(|replacement| replacement.0);
    let mut output = source.to_string();
    let mut mutation_count = 0usize;
    for (start, end, replacement) in replacements.into_iter().rev() {
        if start > end || end > output.len() {
            continue;
        }
        output.replace_range(start..end, replacement.as_str());
        mutation_count += 1;
    }
    (output, mutation_count)
}

fn transform_token_start(token: &omena_parser::LexedToken) -> usize {
    let start: u32 = token.range.start().into();
    start as usize
}

fn transform_token_end(token: &omena_parser::LexedToken) -> usize {
    let end: u32 = token.range.end().into();
    end as usize
}

fn extend_passes_from_ids(ids: &[&'static str], passes: &mut Vec<TransformPassKind>) {
    for candidate in all_transform_pass_kinds() {
        if ids.contains(&candidate.id()) && !passes.contains(&candidate) {
            passes.push(candidate);
        }
    }
}

fn transform_pass_kind_from_id(pass_id: &str) -> Option<TransformPassKind> {
    all_transform_pass_kinds()
        .into_iter()
        .find(|candidate| candidate.id() == pass_id)
}

fn derive_import_inlines_for_transform_context(
    entry: &OmenaQueryStyleFactEntry,
    available_style_paths: &BTreeSet<&str>,
    source_by_path: &BTreeMap<String, String>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<TransformImportInlineV0> {
    entry
        .facts
        .sass_module_edges
        .iter()
        .filter(|edge| edge.kind == "sassImport")
        .filter_map(|edge| {
            let resolved = resolve_style_module_source(
                entry.style_path.as_str(),
                edge.source.as_str(),
                available_style_paths,
                package_manifests,
            )?;
            let replacement_css = source_by_path.get(resolved.as_str())?.clone();
            Some(TransformImportInlineV0 {
                import_source: edge.source.clone(),
                replacement_css,
            })
        })
        .collect()
}

fn derive_class_name_rewrites_for_transform_context(
    entry: &OmenaQueryStyleFactEntry,
) -> Vec<TransformClassNameRewriteV0> {
    if !entry.style_path.contains(".module.") {
        return Vec::new();
    }

    entry
        .facts
        .class_selector_names
        .iter()
        .enumerate()
        .map(|(index, name)| TransformClassNameRewriteV0 {
            original_name: name.clone(),
            rewritten_name: stable_transform_context_class_rewrite(name, index),
        })
        .collect()
}

fn derive_css_module_composes_resolutions_for_transform_context(
    entry: &OmenaQueryStyleFactEntry,
    entries: &[OmenaQueryStyleFactEntry],
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<TransformCssModuleComposesResolutionV0> {
    let facts_by_path = entries
        .iter()
        .map(|entry| (entry.style_path.as_str(), entry.facts.clone()))
        .collect::<BTreeMap<_, _>>();
    let composes_graph = collect_css_modules_composes_adjacency(
        &facts_by_path,
        available_style_paths,
        package_manifests,
    );
    let mut resolutions = BTreeMap::<String, BTreeSet<String>>::new();

    for edge in &entry.facts.css_module_composes_edges {
        for owner in &edge.owner_selector_names {
            let exports = resolutions.entry(owner.clone()).or_default();
            exports.insert(owner.clone());
            for target in &edge.target_names {
                exports.insert(target.clone());
            }
            for target in css_module_composes_closure_for_context(
                &composes_graph,
                entry.style_path.as_str(),
                owner,
            ) {
                exports.insert(target.selector_name);
            }
        }
    }

    resolutions
        .into_iter()
        .map(
            |(local_class_name, exported_class_names)| TransformCssModuleComposesResolutionV0 {
                local_class_name,
                exported_class_names: exported_class_names.into_iter().collect(),
            },
        )
        .collect()
}

fn derive_css_module_value_resolutions_for_transform_context(
    entry: &OmenaQueryStyleFactEntry,
    _entries: &[OmenaQueryStyleFactEntry],
    available_style_paths: &BTreeSet<&str>,
    source_by_path: &BTreeMap<String, String>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<TransformCssModuleValueResolutionV0> {
    let mut resolutions_by_name = BTreeMap::<String, String>::new();
    let mut blocked_names = Vec::<String>::new();

    for edge in &entry.facts.css_module_value_import_edges {
        if blocked_names.iter().any(|name| name == &edge.local_name) {
            continue;
        }
        let Some(resolved_style_path) = resolve_style_module_source(
            entry.style_path.as_str(),
            edge.import_source.as_str(),
            available_style_paths,
            package_manifests,
        ) else {
            continue;
        };
        let Some(source) = source_by_path.get(resolved_style_path.as_str()) else {
            continue;
        };
        let target_resolutions = resolve_static_css_modules_local_value_resolutions_from_source(
            source,
            omena_parser_dialect_for_style_path(resolved_style_path.as_str()),
        );
        let Some(target_resolution) = target_resolutions
            .iter()
            .find(|resolution| resolution.local_name == edge.remote_name)
        else {
            continue;
        };

        if let Some(existing) = resolutions_by_name.get(&edge.local_name) {
            if existing != &target_resolution.resolved_value {
                resolutions_by_name.remove(&edge.local_name);
                blocked_names.push(edge.local_name.clone());
            }
            continue;
        }
        resolutions_by_name.insert(
            edge.local_name.clone(),
            target_resolution.resolved_value.clone(),
        );
    }

    resolutions_by_name
        .into_iter()
        .map(
            |(local_name, resolved_value)| TransformCssModuleValueResolutionV0 {
                local_name,
                resolved_value,
            },
        )
        .collect()
}

fn derive_design_token_routes_for_transform_context(
    entry: &OmenaQueryStyleFactEntry,
    entries: &[OmenaQueryStyleFactEntry],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<TransformDesignTokenRouteV0> {
    let workspace_declarations = entries
        .iter()
        .flat_map(|entry| {
            collect_omena_bridge_design_token_workspace_declarations_from_source(
                entry.style_path.as_str(),
                entry.style_source.as_str(),
            )
        })
        .collect::<Vec<_>>();
    let reachable_declarations = filter_import_reachable_design_token_workspace_declarations(
        entry.style_path.as_str(),
        entries,
        &workspace_declarations,
        package_manifests,
    );
    let local_decl_names = entry
        .facts
        .custom_property_decl_names
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();

    entry
        .facts
        .custom_property_ref_names
        .iter()
        .filter(|name| !local_decl_names.contains(*name))
        .filter_map(|name| {
            let candidates = reachable_declarations
                .iter()
                .filter(|declaration| declaration.file_path != entry.style_path)
                .filter(|declaration| declaration.name == *name)
                .filter(|declaration| design_token_route_value_is_safe(&declaration.value))
                .collect::<Vec<_>>();
            let [candidate] = candidates.as_slice() else {
                return None;
            };
            Some(TransformDesignTokenRouteV0 {
                token_name: (*name).clone(),
                routed_value: candidate.value.clone(),
            })
        })
        .collect()
}

fn design_token_route_value_is_safe(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty() && !value.chars().any(|ch| matches!(ch, ';' | '{' | '}'))
}

fn css_module_composes_closure_for_context(
    graph: &BTreeMap<CssModulesComposesNode, BTreeSet<CssModulesComposesNode>>,
    style_path: &str,
    selector_name: &str,
) -> BTreeSet<CssModulesComposesNode> {
    let start = CssModulesComposesNode {
        style_path: style_path.to_string(),
        selector_name: selector_name.to_string(),
    };
    let mut closure = BTreeSet::new();
    let mut pending = VecDeque::from([start]);

    while let Some(current) = pending.pop_front() {
        let Some(targets) = graph.get(&current) else {
            continue;
        };
        for target in targets {
            if closure.insert(target.clone()) {
                pending.push_back(target.clone());
            }
        }
    }

    closure
}

fn stable_transform_context_class_rewrite(name: &str, index: usize) -> String {
    let sanitized = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    let sanitized = if sanitized.is_empty() {
        "class"
    } else {
        sanitized.as_str()
    };
    format!("_{}_{}", sanitized, index)
}
