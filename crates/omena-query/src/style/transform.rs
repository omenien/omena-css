use super::*;
use omena_transform_passes::{
    TransformClassNameRewriteV0, TransformCssModuleComposesResolutionV0, TransformImportInlineV0,
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
    summarize_omena_query_transform_plan_from_parts(TransformPlanPartsV0 {
        style_path,
        style_source,
        dialect,
        bundle,
        target,
        target_query: None,
        print_options,
        context,
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
    summarize_omena_query_transform_plan_from_parts(TransformPlanPartsV0 {
        style_path,
        style_source,
        dialect,
        bundle,
        target,
        target_query: Some(target_query_plan),
        print_options,
        context,
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
    let egg = plan_egg_rewrite_passes(false, false);

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
    let print = print_transform_execution_artifact(
        parts.style_path,
        semantic_signature,
        &combined_passes,
        parts.print_options,
        &execution,
    );
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
        print,
        execution,
        combined_plan,
        combined_pass_ids,
        combined_violated_dag_edge_count,
        ready_surfaces: vec![
            "transformBundlePlan",
            "transformTargetPlan",
            "transformEggPlan",
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
    let pass_ids = if requested_pass_ids.is_empty() {
        all_transform_pass_kinds()
            .into_iter()
            .map(|pass| pass.id().to_string())
            .collect::<Vec<_>>()
    } else {
        requested_pass_ids.to_vec()
    };
    let context = derive_single_source_transform_context(style_path, style_source);
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
        execution: execution_summary.execution,
        ready_surfaces: vec![
            "consumerBuildFacade",
            "singleSourceTransformContextProducer",
            "transformExecutionRuntime",
            "transformPassOutcomeContract",
        ],
    }
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
    let context = derive_single_source_transform_context(style_path, style_source);
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

    OmenaQueryTransformExecuteSummaryV0 {
        schema_version: "0",
        product: "omena-query.transform-execute",
        style_path: style_path.to_string(),
        requested_pass_ids: requested_pass_ids.to_vec(),
        unknown_pass_ids,
        execution,
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

    let mut context = TransformExecutionContextV0 {
        reachable_class_names: derive_reachable_class_names_for_transform_context(
            &style_fact_entries,
        ),
        reachable_keyframe_names: derive_reachable_keyframe_names_for_transform_context(
            &style_fact_entries,
        ),
        reachable_value_names: derive_reachable_value_names_for_transform_context(
            &style_fact_entries,
        ),
        reachable_custom_property_names:
            derive_reachable_custom_property_names_for_transform_context(&style_fact_entries),
        ..Default::default()
    };

    if let Some(entry) = target_entry {
        context.import_inlines = derive_import_inlines_for_transform_context(
            entry,
            &available_style_paths,
            &source_by_path,
            package_manifests,
        );
        context.class_name_rewrites = derive_class_name_rewrites_for_transform_context(entry);
        context.css_module_composes_resolutions =
            derive_css_module_composes_resolutions_for_transform_context(entry);
    }

    OmenaQueryTransformContextFromSourcesSummaryV0 {
        schema_version: "0",
        product: "omena-query.transform-context",
        target_style_path: target_style_path.to_string(),
        style_count,
        import_inline_count: context.import_inlines.len(),
        class_name_rewrite_count: context.class_name_rewrites.len(),
        css_module_composes_resolution_count: context.css_module_composes_resolutions.len(),
        reachable_class_name_count: context.reachable_class_names.len(),
        reachable_keyframe_name_count: context.reachable_keyframe_names.len(),
        reachable_value_name_count: context.reachable_value_names.len(),
        reachable_custom_property_name_count: context.reachable_custom_property_names.len(),
        context,
        ready_surfaces: vec![
            "transformContextProducer",
            "reachableNameSeedProducer",
            "cssModuleClassRewriteProducer",
            "cssModuleComposesResolutionProducer",
            "directImportInlineProducer",
        ],
    }
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
) -> Vec<TransformCssModuleComposesResolutionV0> {
    let mut resolutions = BTreeMap::<String, BTreeSet<String>>::new();

    for edge in &entry.facts.css_module_composes_edges {
        for owner in &edge.owner_selector_names {
            let exports = resolutions.entry(owner.clone()).or_default();
            exports.insert(owner.clone());
            for target in &edge.target_names {
                exports.insert(target.clone());
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

fn derive_reachable_class_names_for_transform_context(
    entries: &[OmenaQueryStyleFactEntry],
) -> Vec<String> {
    entries
        .iter()
        .flat_map(|entry| entry.facts.class_selector_names.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn derive_reachable_keyframe_names_for_transform_context(
    entries: &[OmenaQueryStyleFactEntry],
) -> Vec<String> {
    entries
        .iter()
        .flat_map(|entry| entry.facts.keyframe_names.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn derive_reachable_value_names_for_transform_context(
    entries: &[OmenaQueryStyleFactEntry],
) -> Vec<String> {
    entries
        .iter()
        .flat_map(|entry| {
            entry
                .facts
                .css_module_value_definition_names
                .iter()
                .chain(entry.facts.icss_export_names.iter())
                .cloned()
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn derive_reachable_custom_property_names_for_transform_context(
    entries: &[OmenaQueryStyleFactEntry],
) -> Vec<String> {
    entries
        .iter()
        .flat_map(|entry| entry.facts.custom_property_decl_names.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
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
